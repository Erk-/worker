use byteorder::{LE, ReadBytesExt as _};
use crate::{
    cache::DiscordCache,
    config::Config,
    events::DiscordEventHandler,
    lavalink::PlaybackManager,
    queue::QueueManager,
    Result,
};
use futures::compat::Future01CompatExt as _;
use hyper::{
    client::{Client as HyperClient, HttpConnector},
    Body,
};
use hyper_tls::HttpsConnector;
use parking_lot::RwLock;
use redis_async::{
    client::{self as redis_client, PairedConnection},
    resp::{FromResp, RespValue},
};
use serenity::{
    http::Client as SerenityHttpClient,
    model::event::GatewayEvent,
};
use std::sync::Arc;

pub struct WorkerState {
    pub cache: Arc<RwLock<DiscordCache>>,
    pub config: Arc<Config>,
    pub http: Arc<HyperClient<HttpsConnector<HttpConnector>, Body>>,
    pub playback: Arc<PlaybackManager>,
    pub queue: Arc<QueueManager>,
    pub redis: Arc<PairedConnection>,
    pub serenity: Arc<SerenityHttpClient>,
}

pub struct Worker {
    discord: DiscordEventHandler,
    redis_popper: PairedConnection,
    state: Arc<WorkerState>,
}

impl Worker {
    pub async fn new(config: Config) -> Result<Self> {
        let config = Arc::new(config);
        debug!("Initializing hyper client");
        let hyper = Arc::new(
            HyperClient::builder().build(HttpsConnector::new(4)?)
        );
        debug!("Initialized hyper client");
        let serenity = Arc::new(SerenityHttpClient::new(
            Arc::clone(&hyper),
            Arc::new(config.discord_token.clone()),
        )?);
        debug!("Initialized serenity http client");

        let discord_cache = Arc::new(RwLock::new(DiscordCache::default()));
        let queue = Arc::new(QueueManager::new(
            Arc::clone(&config),
            Arc::clone(&hyper),
        ));

        let playback = Arc::new(PlaybackManager::new(
            Arc::clone(&config),
            Arc::clone(&hyper),
        ));

        debug!("Initializing redis paired connection");
        let redis_addr = config.redis.addr()?;
        debug!("Connecting to {}...", redis_addr);
        let redis = await!(redis_client::paired_connect(&redis_addr).compat())?;
        debug!("Made first connection to redis, making second...");
        let redis2 = await!(redis_client::paired_connect(&redis_addr).compat())?;
        debug!("Connected two redis connections");

        let state = Arc::new(WorkerState {
            cache: discord_cache,
            redis: Arc::new(redis),
            config,
            http: hyper,
            playback,
            queue,
            serenity,
        });

        let discord = DiscordEventHandler::new(Arc::clone(&state));

        Ok(Self {
            redis_popper: redis2,
            discord,
            state,
        })
    }

    pub async fn run(&self) -> Result<()> {
        loop {
            let (event, shard_id) = match await!(self.recv()) {
                Ok(v) => v,
                Err(why) => {
                    warn!("Error receiving redis payload: {:?}", why);

                    continue;
                }
            };

            trace!("Updating cache");
            self.state.cache.write().update(&event);
            trace!("Updated cache");
            trace!("Dispatching event to discord dispatcher");

            if let Err(why) = self.discord.dispatch(event, shard_id) {
                warn!("Error dispatching event to discord handler: {:?}", why);
            }

            trace!("Dispatched event to discord dispatcher");
        }
    }

    async fn blpop(&self) -> Result<Vec<RespValue>> {
        let array = resp_array!["BLPOP", "sharder:from", 0];

        await!(self.redis_popper.send(array).compat()).map_err(From::from)
    }

    async fn recv(&self) -> Result<(GatewayEvent, u64)> {
        let mut parts = await!(self.blpop())?;

        let part = if parts.len() == 2 {
            parts.remove(1)
        } else {
            warn!("blpop result part count != 2: {:?}", parts);

            None?;

            unreachable!();
        };

        let mut message: Vec<u8> = match FromResp::from_resp(part) {
            Ok(msg) => msg,
            Err(why) => {
                warn!("Err parsing part to bytes: {:?}", why);

                None?;

                unreachable!();
            },
        };
        let message_len = message.len();
        let shard_id = {
            let mut shard_bytes = &message[message_len - 2..];
            shard_bytes.read_u16::<LE>()? as u64
        };
        message.truncate(message_len - 2);

        let event = serde_json::from_slice(&message)?;

        trace!("Got event: {:?}", event);

        Ok((event, shard_id))
    }
}
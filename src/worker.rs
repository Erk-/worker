use byteorder::{LE, ReadBytesExt as _};
use crate::{
    bridges::discord::DiscordEventHandler,
    cache::Cache,
    config::Config,
    commands::{self, Command},
    discord_fm::DiscordFm,
    error::Result,
    lavalink_msgs,
    radios::RadioList,
    services::{
        lavalink::LavalinkManager,
        queue::QueueManager,
    },
    utils,
};
use futures::{
    channel::oneshot::{self, Sender as OneshotSender},
    compat::Future01CompatExt as _,
    future::TryFutureExt as _,
};
use hyper::{
    client::{Client as HyperClient, HttpConnector},
    Body,
};
use hyper_tls::HttpsConnector;
use redis_async::{
    client::{self as redis_client, PairedConnection},
    resp::{FromResp, RespValue},
};
use serenity::{
    http::Client as SerenityHttpClient,
    model::event::GatewayEvent,
};
use std::{
    collections::HashMap,
    sync::Arc,
};

pub struct WorkerState {
    pub cache: Arc<Cache>,
    pub config: Arc<Config>,
    pub commands: HashMap<String, Arc<Box<dyn Command<'static> + Send + Sync>>>,
    pub discord_fm: DiscordFm,
    pub http: Arc<HyperClient<HttpsConnector<HttpConnector>, Body>>,
    pub playback: Arc<LavalinkManager>,
    pub queue: Arc<QueueManager>,
    pub radios: RadioList,
    pub redis: Arc<PairedConnection>,
    pub serenity: Arc<SerenityHttpClient>,
}

pub struct Worker {
    discord: DiscordEventHandler,
    lavalink_msgs_shutdown_tx: Option<OneshotSender<()>>,
    redis_popper: PairedConnection,
    state: Arc<WorkerState>,
}

impl Worker {
    pub async fn new(config: Config) -> Result<Self> {
        let discord_fm = DiscordFm::new()?;
        let radios = RadioList::new()?;
        debug!("Initializing redis paired connection");
        let redis_addr = config.redis.addr()?;
        debug!("Connecting to {}...", redis_addr);
        let redis = Arc::new(await!(redis_client::paired_connect(&redis_addr).compat())?);
        debug!("Made first connection to redis, making second...");
        let redis2 = await!(redis_client::paired_connect(&redis_addr).compat())?;
        debug!("Made second connection to redis, making third...");
        let redis3 = await!(redis_client::paired_connect(&redis_addr).compat())?;
        debug!("Connected three redis connections");

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

        let cache = Arc::new(Cache::new(
            Arc::clone(&config),
            Arc::clone(&redis),
        ));
        let queue = Arc::new(QueueManager::new(
            Arc::clone(&config),
            Arc::clone(&hyper),
        ));

        let playback = Arc::new(LavalinkManager::new(
            Arc::clone(&config),
            Arc::clone(&hyper),
        ));

        let state = Arc::new(WorkerState {
            cache,
            config,
            commands: commands::map(),
            discord_fm,
            http: hyper,
            playback,
            queue,
            radios,
            redis,
            serenity,
        });

        let (tx, rx) = oneshot::channel();

        utils::spawn(lavalink_msgs::from_lavalink(redis3, Arc::clone(&state), rx).map_err(|why| {
            warn!("Err with lavalink:from: {:?}", why);
        }));
        let discord = DiscordEventHandler::new(Arc::clone(&state));

        Ok(Self {
            lavalink_msgs_shutdown_tx: Some(tx),
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

            trace!("Dispatching event to discord dispatcher");
            self.discord.dispatch(event, shard_id);
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

impl Drop for Worker {
    fn drop(&mut self) {
        if let Some(tx) = self.lavalink_msgs_shutdown_tx.take() {
            if let Err(why) = tx.send(()) {
                warn!("Err sending shutdown to lavalink msgs: {:?}", why);
            }
        }
    }
}

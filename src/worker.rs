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
use std::sync::Arc;

pub struct WorkerState {
    pub cache: Arc<Cache>,
    pub config: Arc<Config>,
    pub commands: Arc<Vec<Arc<&'static (dyn Command<'static> + Send + Sync)>>>,
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

        let commands: Vec<Arc<&'static (dyn Command + Send + Sync)>>;
        {
            use self::commands::*;

            commands = vec![
                Arc::new(&about::COMMAND_INSTANCE),
                Arc::new(&cancel::COMMAND_INSTANCE),
                Arc::new(&choose::COMMAND_INSTANCE),
                Arc::new(&clear::COMMAND_INSTANCE),
                Arc::new(&discordfm::COMMAND_INSTANCE),
                Arc::new(&dump::COMMAND_INSTANCE),
                Arc::new(&help::COMMAND_INSTANCE),
                Arc::new(&invite::COMMAND_INSTANCE),
                Arc::new(&join::COMMAND_INSTANCE),
                Arc::new(&leave::COMMAND_INSTANCE),
                Arc::new(&load::COMMAND_INSTANCE),
                Arc::new(&pause::COMMAND_INSTANCE),
                Arc::new(&ping::COMMAND_INSTANCE),
                Arc::new(&play::COMMAND_INSTANCE),
                Arc::new(&playing::COMMAND_INSTANCE),
                Arc::new(&providers::COMMAND_INSTANCE),
                Arc::new(&queue::COMMAND_INSTANCE),
                Arc::new(&radio::COMMAND_INSTANCE),
                Arc::new(&remove::COMMAND_INSTANCE),
                Arc::new(&restart::COMMAND_INSTANCE),
                Arc::new(&resume::COMMAND_INSTANCE),
                Arc::new(&seek::COMMAND_INSTANCE),
                Arc::new(&skip::COMMAND_INSTANCE),
                Arc::new(&soundcloud::COMMAND_INSTANCE),
                Arc::new(&volume::COMMAND_INSTANCE),
                Arc::new(&youtube::COMMAND_INSTANCE),
            ];
        }
        let commands = Arc::new(commands);
        debug!("Initialized commands");

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
            commands,
            discord_fm,
            http: hyper,
            playback,
            queue,
            radios,
            redis,
            serenity,
        });

        utils::spawn(lavalink_msgs::from_lavalink(redis3, Arc::clone(&state)).map_err(|why| {
            warn!("Err with lavalink:from: {:?}", why);
        }));
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

use crate::{
    command::Context,
    commands,
    error::{Error, Result},
    worker::WorkerState,
};
use futures::future::TryFutureExt;
use hyper::client::{Client, HttpConnector};
use hyper_tls::HttpsConnector;
use lavalink::model::VoiceUpdate;
use parking_lot::RwLock;
use serenity::model::event::{Event, GatewayEvent, MessageCreateEvent};
use std::sync::Arc;

pub type HyperHttpClient = Client<HttpsConnector<HttpConnector>>;

pub struct DiscordEventHandler {
    state: Arc<WorkerState>,
    current_user_id: Arc<RwLock<u64>>,
}

impl DiscordEventHandler {
    pub fn new(state: Arc<WorkerState>) -> Self {
        Self {
            current_user_id: Arc::new(RwLock::new(0)),
            state,
        }
    }

    pub async fn dispatch(
        &self,
        event: GatewayEvent,
        shard_id: u64,
    ) -> Result<()> {
        use self::Event::*;
        use self::GatewayEvent::Dispatch;

        match event {
            Dispatch(_, Ready(e)) => {
                let id = e.ready.user.id.0;
                *self.current_user_id.write() = id;
                info!("Received Ready event! User id: {:?}", id);
            }
            Dispatch(_, MessageCreate(e)) => {
                trace!("Received MessageCreate event");

                await!(on_message(e, Arc::clone(&self.state), shard_id))?;
            },
            Dispatch(_, VoiceServerUpdate(e)) => {
                trace!("Received VoiceServerUpdate event: {:?}", &e);

                let guild_id = match e.guild_id {
                    Some(guild_id) => guild_id.0,
                    None => {
                        trace!("No guild id for voice server update");
                        return Ok(());
                    }
                };

                let session_id = {
                    let discord_cache = self.state.cache.read();

                    match discord_cache.get_user_voice_state(&guild_id, &self.current_user_id.read()) {
                        Some(voice_state) => voice_state.session_id.clone(),
                        None => {
                            error!("bot user has no voice state to get session id");
                            return Ok(());
                        }
                    }
                };

                let update = VoiceUpdate::new(
                    session_id,
                    guild_id.to_string(),
                    e.token,
                    e.endpoint.unwrap(),
                );

                match await!(self.state.playback.voice_update(update)) {
                    Ok(()) => {
                        trace!("Sent voice server update to lavalink server");
                    },
                    Err(why) => {
                        warn!(
                            "Error sending voice update to lavalink server: {:?}",
                            why,
                        );
                    },
                }
            },
            _ => {}
        }

        Ok(())
    }
}

async fn get_prefix(_guild_id: u64) -> Result<String> {
    // todo dynamic prefix
    Ok(">".into())
}

async fn on_message(
    event: MessageCreateEvent,
    state: Arc<WorkerState>,
    shard_id: u64,
) -> Result<()> {
    let msg = event.message;
    if msg.author.bot {
        return Ok(());
    }

    let content = msg.content.clone();

    let guild_id = msg.guild_id?.0;

    let prefix = await!(get_prefix(guild_id))?;
    if !content.starts_with(&prefix) {
        return Ok(());
    }

    let content_trimmed: String = content.chars().skip(prefix.len()).collect();
    let mut content_iter = content_trimmed.split_whitespace();
    let command_name = content_iter.next()?;

    {
        let alias = command_name.to_lowercase();
        let ctx = Context {
            state,
            shard_id,
            msg,
            args: content_iter.map(|s| s.to_string()).collect(),
            alias: alias.clone(),
        };

        let result = match &*alias {
            "connect" | "j" | "join" => await!(commands::join::run(ctx)),
            "disconnect" | "l" | "leave" => await!(commands::leave::run(ctx)),
            _ => None?,
        };

        if let Err(why) = result {
            match why {
                Error::None(_) => debug!("none error running command"),
                other => error!("error running command: {:?}", other),
            }
        }
    }

    Ok(())
}

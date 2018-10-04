use crate::{
    commands::{self, Context, Response},
    config::Config,
    error::{Error, Result},
    worker::WorkerState,
};
use futures::{
    compat::{Future01CompatExt as _, TokioDefaultSpawner},
    future::{FutureExt as _, TryFutureExt as _},
};
use lavalink::model::VoiceUpdate;
use serenity::model::event::{
    Event,
    GatewayEvent,
    MessageCreateEvent,
    ReadyEvent,
    VoiceServerUpdateEvent,
};
use std::{
    borrow::Cow,
    sync::Arc,
};
use tokio::prelude::Future as Future01;

pub struct DiscordEventHandler {
    state: Arc<WorkerState>,
}

impl DiscordEventHandler {
    pub fn new(state: Arc<WorkerState>) -> Self {
        Self {
            state,
        }
    }

    pub fn dispatch(
        &self,
        event: GatewayEvent,
        shard_id: u64,
    ) -> Result<()> {
        trace!(
            "Discord dispatcher received event on shard {}: {:?}",
            shard_id,
            event,
        );
        use self::Event::*;
        use self::GatewayEvent::Dispatch;

        match event {
            Dispatch(_, Ready(e)) => self.ready(e),
            Dispatch(_, MessageCreate(e)) => {
                let future = message_create(e, shard_id, Arc::clone(&self.state))
                    .boxed()
                    .compat(TokioDefaultSpawner)
                    .map_err(|why| {
                        warn!(
                            "Error dispatching message create: {:?}",
                            why,
                        );
                    });

                tokio::spawn(future);
            },
            Dispatch(_, VoiceServerUpdate(e)) => {
                let future = voice_server_update(e, Arc::clone(&self.state))
                    .boxed()
                    .compat(TokioDefaultSpawner)
                    .map_err(|why| {
                        warn!(
                            "Error dispatching voice server update: {:?}",
                            why,
                        );
                    });

                tokio::spawn(future);
            },
            _ => {},
        }

        Ok(())
    }

    fn ready(&self, event: ReadyEvent) {
        info!("Received Ready event! User id: {:?}", event.ready.user.id);
    }
}

async fn get_prefixes<'a>(
    config: &'a Arc<Config>,
    _guild_id: u64,
) -> Result<Vec<Cow<'a, str>>> {
    // todo
    let mut prefixes = Vec::with_capacity(config.bot_prefixes.len());

    for prefix in &config.bot_prefixes {
        prefixes.push(Cow::from(prefix));
    }

    Ok(prefixes)
}

async fn message_create(
    event: MessageCreateEvent,
    shard_id: u64,
    state: Arc<WorkerState>,
) -> Result<()> {
    trace!("Received MessageCreate event: {:?}", event);

    let msg = event.message;

    if msg.author.bot {
        return Ok(());
    }

    let content = msg.content.clone();

    let guild_id = msg.guild_id?.0;

    trace!("Getting guild prefix");
    let prefixes = await!(get_prefixes(&state.config, guild_id))?;

    let prefix = {
        match prefixes.iter().find(|prefix| content.starts_with(prefix.as_ref())) {
            Some(prefix) => prefix,
            None => {
                trace!("Message doesn't start with prefix");

                return Ok(());
            },
        }
    };

    let content_trimmed: String = content.chars().skip(prefix.len()).collect();
    let content_iter = content_trimmed.split_whitespace().collect::<Vec<&str>>();
    trace!("content iter: {:?}", content_iter);
    let mut content_iter = content_iter.iter();
    trace!("Determining command name");
    let command_name = content_iter.next()?;

    trace!("Command name: {}", command_name);

    {
        let alias = command_name.to_lowercase();
        let channel_id = msg.channel_id.0;
        let ctx = Context {
            args: content_iter.map(|s| s.to_string()).collect(),
            alias: alias.clone(),
            state: Arc::clone(&state),
            msg,
            shard_id,
        };

        let result = match &*alias {
            "about" => await!(commands::about::run(ctx)),
            "dfm" | "discordfm" | "discord.fm" => {
                await!(commands::discordfm::run(ctx))
            },
            "dump" => await!(commands::dump::run(ctx)),
            "invite" => await!(commands::invite::run(ctx)),
            "join" | "j" | "connect" => await!(commands::join::run(ctx)),
            "leave" | "l" | "disconnect" | "stop" => {
                await!(commands::leave::run(ctx))
            },
            "load" => await!(commands::load::run(ctx)),
            "pause" | "hold" => await!(commands::pause::run(ctx)),
            "ping" => await!(commands::ping::run(ctx)),
            "play" | "p" => await!(commands::play::run(ctx)),
            "playing" | "np" | "nowplaying" | "current" => {
                await!(commands::playing::run(ctx))
            },
            "providers" => await!(commands::providers::run(ctx)),
            "queue" | "q" | "que" => await!(commands::queue::run(ctx)),
            "radio" => await!(commands::radio::run(ctx)),
            "remove" => await!(commands::remove::run(ctx)),
            "restart" => await!(commands::restart::run(ctx)),
            "resume" | "unpause" => await!(commands::resume::run(ctx)),
            "seek" => await!(commands::seek::run(ctx)),
            "skip" | "s" | "next" => await!(commands::skip::run(ctx)),
            "soundcloud" | "sc" => await!(commands::soundcloud::run(ctx)),
            "volume" | "v" => await!(commands::volume::run(ctx)),
            _ => {
                trace!("No command matched alias: {}", alias);

                return Ok(());
            },
        };

        match result {
            Ok(Response::Text(content)) => {
                await!(state.serenity.send_message(
                    channel_id,
                    |mut m| {
                        m.content(content);

                        m
                    },
                ).compat())?;
            },
            Err(Error::None(_)) => debug!("None error running command"),
            Err(other) => error!("Error running command: {:?}", other),
        }
    }

    Ok(())
}

async fn voice_server_update(
    event: VoiceServerUpdateEvent,
    state: Arc<WorkerState>,
) -> Result<()> {
    debug!("Received VoiceServerUpdate event: {:?}", &event);

    let guild_id = event.guild_id.map(|x| x.0)?;

    debug!("Got guild id");

    let session_id = {
        await!(state.cache.voice_state(guild_id, state.config.discord_user_id))??.session_id
    };

    debug!("Got session id for current user");

    let update = VoiceUpdate::new(
        session_id,
        guild_id.to_string(),
        event.token,
        event.endpoint.unwrap(),
    );

    match await!(state.playback.voice_update(update)) {
        Ok(()) => {
            debug!("Sent voice server update to lavalink server");
        },
        Err(why) => {
            warn!(
                "Error sending voice update to lavalink server: {:?}",
                why,
            );
        },
    }

    Ok(())
}

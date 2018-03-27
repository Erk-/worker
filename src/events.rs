use error::Error;
use command::{CommandManager, Context};
use futures::prelude::*;
use tokio_core::reactor::Handle;
use std::rc::Rc;
use std::cell::RefCell;
use serenity::model::event::{GatewayEvent, MessageCreateEvent};
use serenity::http::Client as SerenityHttpClient;
use cache::DiscordCache;
use serenity::gateway::Shard;
use lavalink_futures::reexports::OwnedMessage;
use futures::future;
use shards::ShardManager;
use tungstenite::Message as TungsteniteMessage;
use lavalink_futures::nodes::NodeManager;
use lavalink::model::VoiceUpdate;

pub struct DiscordEventHandler {
    handle: Handle,
    serenity_http: Rc<SerenityHttpClient>,
    command_manager: Rc<RefCell<CommandManager>>,
    discord_cache: Rc<RefCell<DiscordCache>>,
    node_manager: Rc<RefCell<NodeManager>>,
    current_user_id: Option<u64>,
}

impl DiscordEventHandler {
    pub fn new(
        handle: Handle, 
        serenity_http: Rc<SerenityHttpClient>, 
        command_manager: Rc<RefCell<CommandManager>>,
        discord_cache: Rc<RefCell<DiscordCache>>,
        node_manager: Rc<RefCell<NodeManager>>,
    ) -> Result<Self, Error> {
        Ok(Self {
            handle,
            serenity_http,
            command_manager,
            discord_cache,
            node_manager,
            current_user_id: None,
        })
    }

    pub fn on_event(&mut self, event: GatewayEvent, shard: Rc<RefCell<Shard>>) {
        use GatewayEvent::Dispatch;
        use Event::*;

        match event {
            Dispatch(_, Ready(e)) => {
                self.current_user_id = Some(e.ready.user.id.0);
                info!("Received Ready event! User id: {:?}", self.current_user_id);
            },
            Dispatch(_, MessageCreate(e)) => {
                trace!("Received MessageCreate event");

                let future = on_message(
                    e, 
                    self.command_manager.clone(), 
                    self.handle.clone(), 
                    self.serenity_http.clone(),
                    self.discord_cache.clone(),
                    shard,
                    self.node_manager.clone(),
                ).map_err(|e| match e {
                    Error::None(_) => debug!("none error handling MessageCreate"),
                    _ => error!("error handling MessageCreate: {:?}", e),
                });

                self.handle.spawn(future);
            },
            Dispatch(_, VoiceServerUpdate(e)) => {
                trace!("Received VoiceServerUpdate event: {:?}", &e);

                let guild_id = match e.guild_id {
                    Some(guild_id) => guild_id.0,
                    None => {
                        trace!("No guild id for voice server update");
                        return;
                    }
                };

                let session_id = match &self.current_user_id {
                    Some(user_id) => {
                        let discord_cache = self.discord_cache.borrow();

                        match discord_cache.get_user_voice_state(&guild_id, &user_id) {
                            Some(voice_state) => voice_state.session_id.clone(),
                            None => {
                                error!("bot user has no voice state to get session id");
                                return;
                            }
                        }
                    },
                    None => {
                        error!("received voice server update before ready event!");
                        return;
                    }
                };

                let node_manager = self.node_manager.borrow();
                let mut player_manager = node_manager.player_manager.borrow_mut();

                let mut player = match player_manager.get_mut(&guild_id) {
                    Some(player) => player,
                    None => {
                        error!("no player for guild {} to send voice server update to", &guild_id);
                        return;
                    }
                };

                let json = ::serde_json::to_string(&VoiceUpdate::new(
                    session_id,
                    guild_id.to_string(),
                    e.token,
                    e.endpoint.unwrap(),
                )).expect("could not encode VoiceUpdate as json");

                if let Err(e) = player.send(OwnedMessage::Text(json)) {
                    error!("error sending voice server update to lavalink node: {:?}", e);
                } else {
                    trace!("sent voice server update to lavalink node");
                }
            }
            _ => {}
        }
    }
}

#[async]
fn get_prefix(_guild_id: u64) -> Result<String, Error> {
    // todo dynamic prefix
    Ok(">".into())
}

#[async]
fn on_message(
    event: MessageCreateEvent, 
    command_manager: Rc<RefCell<CommandManager>>, 
    handle: Handle, 
    serenity_http: Rc<SerenityHttpClient>,
    discord_cache: Rc<RefCell<DiscordCache>>,
    shard: Rc<RefCell<Shard>>,
    node_manager: Rc<RefCell<NodeManager>>,
) -> Result<(), Error> {
    let msg = event.message;

    if msg.author.bot {
        return Ok(());
    }
    
    let content = msg.content.clone();

    // msg.guild_id() returns None because msg events only contain the channel id
    // so getting the guild id depends on cache which isnt ready for futures branch
    //let guild_id = msg.guild_id()?.0;
    let guild_id = 0u64;

    let prefix = await!(get_prefix(guild_id))?;
    if !content.starts_with(&prefix) {
        return Ok(());
    }
    
    let content_trimmed: String = content.chars().skip(prefix.len()).collect();
    let mut content_iter = content_trimmed.split_whitespace();
    let command_name = content_iter.next()?;

    let command_manager = command_manager.borrow();
    let command = command_manager.get(&command_name.to_lowercase())?;

    let context = Context {
        handle: handle.clone(), 
        serenity_http: serenity_http,
        msg,
        args: content_iter.map(|s| s.to_string()).collect(),
        discord_cache: discord_cache,
        shard,
        node_manager,
    };

    let future = (command.executor)(context).map_err(|e| match e {
        Error::None(_) => debug!("none error running command"),
        _ => error!("error running command: {:?}", e),
    });

    handle.spawn(future);
    Ok(())
}

type LavalinkHandlerFuture<T> = Box<Future<Item = T, Error = ()>>;

pub struct LavalinkEventHandler {
    shard_manager: Rc<ShardManager>,
}

impl LavalinkEventHandler {
    pub fn new(shard_manager: Rc<ShardManager>) -> Self {
        Self {
            shard_manager,
        }
    }
}

impl ::lavalink_futures::EventHandler for LavalinkEventHandler {
    fn forward(&mut self, shard_id: u64, message: &str) -> LavalinkHandlerFuture<Option<OwnedMessage>> {
        let shard = match self.shard_manager.get_shard(&shard_id) {
            Some(shard) => shard,
            None => {
                error!("could not get shard {} from manager", shard_id);
                return box future::ok(None);
            },
        };

        let mut lock = shard.borrow_mut();
        if let Err(e) = lock.send(TungsteniteMessage::Text(message.to_owned())) {
            error!("error sending message to shard {} websocket {}: {:?}", shard_id, message, e);
        }

        box future::ok(None)
    }

    fn is_connected(&mut self, shard_id: u64) -> LavalinkHandlerFuture<bool> {
        debug!("is connected: {}", shard_id);
        box future::ok(true)
    }

    fn is_valid(&mut self, guild_id: &str, channel_id: Option<String>) -> LavalinkHandlerFuture<bool> {
        debug!("is valid: guild_id: {}, channel_id: {:?}", guild_id, channel_id);
        box future::ok(true)
    }

    fn track_end(&mut self, track: String, reason: String) -> LavalinkHandlerFuture<()> {
        debug!("track end: track: {}, reason: {}", track, reason);
        box future::ok(())
    }

    fn track_exception(&mut self, track: String, error: String) -> LavalinkHandlerFuture<()> {
        debug!("track exception: track: {}, error: {}", track, error);
        box future::ok(())
    }

    fn track_stuck(&mut self, track: String, threshold_ms: i64) -> LavalinkHandlerFuture<()> {
        debug!("track stuck: track: {}, threshold_ms: {}", track, threshold_ms);
        box future::ok(())
    }
}
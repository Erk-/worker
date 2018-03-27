use error::Error;
use futures::prelude::*;
use tokio_core::reactor::Handle;
use std::collections::HashMap;
use serenity::model::channel::Message;
use std::rc::Rc;
use serenity::http::Client as SerenityHttpClient;
use cache::DiscordCache;
use std::cell::RefCell;
use serenity::gateway::Shard;
use serenity::builder::CreateMessage;
use lavalink_futures::nodes::NodeManager;

pub struct Command {
    pub names: Vec<&'static str>,
    pub description: &'static str,
    pub executor: fn(Context) -> Box<Future<Item = (), Error = Error>>,
}

pub struct Context {
    pub handle: Handle,
    pub serenity_http: Rc<SerenityHttpClient>,
    pub msg: Message,
    pub args: Vec<String>,
    pub discord_cache: Rc<RefCell<DiscordCache>>,
    pub shard: Rc<RefCell<Shard>>,
    pub node_manager: Rc<RefCell<NodeManager>>,
}

impl Context {
    pub fn send_message<F>(&self, m: F)
    where F: FnOnce(CreateMessage) -> CreateMessage + 'static {
        let channel_id = self.msg.channel_id.0;

        let future = self.serenity_http.send_message(channel_id, m)
            .map(move |m| trace!("Sent message to channel {}: {}", channel_id, m.content))
            .map_err(|e| error!("Error sending message {:?}", e));

        self.handle.spawn(future);
    }
}

type ICommand = Rc<Command>;

pub struct CommandManager {
    pub handle: Handle,
    pub commands: HashMap<String, ICommand>,
}

impl CommandManager {
    pub fn new(handle: Handle) -> Self {
        Self {
            handle,
            commands: Default::default(),
        }
    }

    pub fn add(&mut self, command: ICommand) {
        let names = &command.names;
        for name in names {
            self.commands.insert(name.to_string(), command.clone());
        }
    }

    pub fn get(&self, name: &str) -> Result<ICommand, Error> {
        Ok(self.commands.get(name)?.clone())
    }
}
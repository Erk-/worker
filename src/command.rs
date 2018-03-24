use error::Error;
use futures::prelude::*;
use tokio_core::reactor::Handle;
use std::collections::HashMap;
use serenity::model::channel::Message;
use std::rc::Rc;
use serenity::http::Client as SerenityHttpClient;
use cache::DiscordCache;
use std::cell::RefCell;

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
use error::Error;
use futures::Future;
use futures::future;
use tokio_core::reactor::Handle;
use std::collections::HashMap;
use serenity::model::channel::Message;
use std::rc::Rc;
use std::sync::RwLock;
use serenity::http::Client as SerenityHttpClient;

pub trait Command: 'static {
    fn names(&self) -> Vec<&'static str>;

    fn run(&mut self, Context, Message, Vec<String>) -> Box<Future<Item = (), Error = Error>>;
}

pub struct Context {
    pub handle: Handle,
    pub serenity_http: Rc<SerenityHttpClient>,
}

type ICommand = Rc<RwLock<Command>>;

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
        let command_lock = command.read().expect("could not get read lock on command");
        let names = command_lock.names();

        for name in names {
            self.commands.insert(name.to_string(), command.clone());
        }
    }
}

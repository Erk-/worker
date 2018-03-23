use error::Error;
use futures::prelude::*;
use tokio_core::reactor::Handle;
use std::collections::HashMap;
use serenity::model::channel::Message;
use std::rc::Rc;
use serenity::http::Client as SerenityHttpClient;
use std::str::SplitWhitespace;

pub type CommandFuture = Box<Future<Item = (), Error = Error>>;

/*pub trait Command: 'static {
    fn names(&self) -> Vec<&'static str>;

    fn description(&self) -> &'static str;

    fn run(&mut self, Context) -> CommandFuture;
}*/

pub struct Command {
    pub names: Vec<&'static str>,
    pub description: &'static str,
    pub executor: fn(Context) -> CommandFuture,
}

impl Command {
    pub fn run(&self, ctx: Context) -> CommandFuture {
        (self.executor)(ctx)
    }
}

pub struct Context<'a> {
    pub handle: Handle,
    pub serenity_http: Rc<SerenityHttpClient>,
    pub msg: Message,
    pub args: SplitWhitespace<'a>,
}

type ICommand = Rc<Command>;

/*pub struct CommandManager {
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
}*/

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
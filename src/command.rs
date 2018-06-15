use error::Error;
use cache::DiscordCache;

use futures::prelude::*;
use tokio_core::reactor::Handle;
use std::collections::HashMap;
use std::rc::Rc;
use std::cell::RefCell;
use serenity::model::channel::Message;
use serenity::http::Client as SerenityHttpClient;
use serenity::gateway::Shard;
use serenity::builder::CreateMessage;
use lavalink_futures::nodes::NodeManager;

pub type CommandResult = Result<Response, Error>;
type CommandExecutor = fn(Context) -> Box<Future<Item = Response, Error = Error>>;
type ICommand = Rc<Command>;

pub struct Command {
    pub names: Vec<&'static str>,
    pub description: &'static str,
    pub executor: CommandExecutor,
}

pub struct Context {
    pub handle: Handle,
    pub serenity_http: Rc<SerenityHttpClient>,
    pub discord_cache: Rc<RefCell<DiscordCache>>,
    pub node_manager: Rc<RefCell<NodeManager>>,
    pub shard: Rc<RefCell<Shard>>,
    pub msg: Message,
    pub args: Vec<String>,
}

pub enum Response {
    Text(String),
}

impl Response {
    pub fn text<S: Into<String>>(content: S) -> CommandResult {
        Ok(Response::Text(content.into()))
    }
}

#[async]
pub fn run(executor: CommandExecutor, ctx: Context) -> Result<(), Error> {
    let serenity_http = ctx.serenity_http.clone();
    let channel_id = ctx.msg.channel_id.0;

    let response = await!((executor)(ctx))?;
    let m = match response {
        Response::Text(content) => |mut m: CreateMessage| { m.content(content); m },
    };

    let future = serenity_http.send_message(channel_id, m)
        .map(move |m| trace!("Sent message to channel {}: {}", channel_id, m.content))
        .map_err(From::from);

    await!(future)
}

pub struct CommandManager {
    pub handle: Handle,
    pub commands: HashMap<String, ICommand>,
}

impl CommandManager {
    pub fn new(handle: Handle, commands: Vec<Command>) -> Self {
        let commands = commands.into_iter()
            .map(Rc::new)
            .flat_map(|command| command.names.iter()
                .map(ToString::to_string)
                .collect::<Vec<_>>()
                .into_iter()
                .map(move |name| (name, command.clone())))
            .collect();

        Self {
            handle,
            commands,
        }
    }

    pub fn get(&self, name: &str) -> Result<ICommand, Error> {
        Ok(self.commands.get(name)?.clone())
    }
}
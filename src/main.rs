#![feature(proc_macro, conservative_impl_trait, generators, try_trait, box_syntax)]

#[macro_use]
extern crate log;
extern crate env_logger;
extern crate serenity;
extern crate lavalink_futures;
extern crate futures_await as futures;
extern crate tokio_core;
extern crate hyper;
extern crate hyper_tls;
extern crate native_tls;
extern crate tungstenite;
extern crate serde_json;
extern crate regex;

mod error;

use error::Error;
use futures::prelude::*;
use tokio_core::reactor::{Core, Handle};
use std::env;
use hyper::Client as HyperClient;
use hyper_tls::HttpsConnector;
use std::rc::Rc;
use serenity::gateway::Shard;
use serenity::model::event::{Event, GatewayEvent, MessageCreateEvent, ReadyEvent};
use serenity::http::Client as SerenityHttpClient;
use serenity::model::channel::Message;
use serenity::Error as SerenityError;
use futures::Future;
use futures::future::FutureResult;
use regex::Regex;
use std::collections::HashMap;
use std::cell::RefCell;

fn main() {
    env_logger::init();

    let mut core = Core::new().expect("Error creating event loop");
    let future = try_main(core.handle()).map_err(Box::new);
    println!("Error running future: {:?}", core.run(future));
}

#[async]
fn try_main(handle: Handle) -> Result<(), Error> {
    let token = format!("Bot {}", env::var("DISCORD_TOKEN").expect("Error no discord token"));

    let mut shard = await!(Shard::new(
        token.clone(), [0, 1], handle.clone()
    ))?;

    let http_client = Rc::new(HyperClient::configure()
        .connector(HttpsConnector::new(4, &handle)?)
        .build(&handle));

    let serenity_http = SerenityHttpClient::new(
        http_client.clone(), handle.clone(), Rc::new(token)
    );

    let command_manager = Rc::new(RefCell::new(CommandManager::default()));
    command_manager.add(vec![TestCommand {}]);

    let event_handler = EventHandler::new(handle.clone(), serenity_http, command_manager.clone())?;

    #[async]
    for message in shard.messages() {
        let event = shard.parse(message)?;
        shard.process(&event);

        use GatewayEvent::Dispatch;
        use Event::*;

        match event {
            Dispatch(_, Ready(e)) => event_handler.on_ready(e),
            Dispatch(_, MessageCreate(e)) => event_handler.on_message(e),
            _ => {
                // ya nothing else
            }
        }
    }

    Ok(())
}

struct EventHandler {
    handle: Handle,
    serenity_http: SerenityHttpClient,
    split_regex: Regex,
    command_manager: Rc<RefCell<CommandManager>>,
}

const PREFIX: &'static str = ">";

impl EventHandler {
    fn new(handle: Handle, serenity_http: SerenityHttpClient, command_manager: Rc<RefCell<CommandManager>>) -> Result<Self, Error> {
        let split_regex = Regex::new(r"\s+")?;

        Ok(Self {
            handle,
            serenity_http,
            split_regex,
            command_manager,
        })
    }

    fn on_ready(&self, _: ReadyEvent) {
        println!("Connected to discord");
    }

    fn on_message(&self, event: MessageCreateEvent) {
        let msg = event.message;
        let content = msg.content;
        println!("{}#{}: {}", msg.author.name, msg.author.discriminator, &content);
        
        if !content.starts_with(PREFIX) {
            // no
            return;
        }

        let content = &content[PREFIX.len()..];
        println!("content: {}", content);

        let mut content_iter = self.split_regex.split(&content);
        let command_name = match content_iter.next() {
            Some(c) => c,
            None => {
                // no
                return;
            }
        };
        println!("command_name: {}", command_name);

        let command = match self.command_manager.commands.get(&command_name.lowercase()) {
            Some(command) => command,
            None => {
                debug!("invalid command {}", command_name);
                return;
            }
        };

        let args = content_iter.map(|s| s.to_string()).collect::<Vec<String>>();
        println!("args: {:?}", args);

        let future = command.run(msg, args)
            .map_err(|e| error!("oh no couldnt run command: {:?}", e));
        self.handle.spawn(future);

        //if msg.content == "DABBOT IS GOOD" {
        //    self.handle.spawn(send_message(&self.serenity_http, msg.channel_id.0, "hel"));
        //}
    }
}

fn send_message(serenity_http: &SerenityHttpClient, channel_id: u64, content: &str) -> impl Future<Item = (), Error = ()> {
    serenity_http.send_message(channel_id, |m| m.content(content))
        .map(|m| debug!("Sent message {:?}", m))
        .map_err(|e| error!("Error sending message {:?}", e))
}

trait Command {
    fn names(&self) -> Vec<&'static str>;

    fn run(&mut self, Message, Vec<String>) -> Box<Future<Item = (), Error = Error>>;
}

#[derive(Default)]
struct CommandManager {
    commands: HashMap<String, Command + 'static>,
}

impl CommandManager {
    fn add<C: Command + 'static>(&mut self, commands: Vec<C>) {
        for command in commands.into_iter() {
            let names = command.names();
            for name in names {
                self.commands.insert(name, command);
            }
        }
    }
}

struct TestCommand;

impl Command for TestCommand {
    fn names(&self) -> Vec<&'static str> {
        vec!["test", "t", "meme"]
    }

    #[async]
    fn run(&mut self, msg: Message, args: Vec<String>) -> Result<(), Error> {
        // lol do some shit

        Ok(())
    }
}
use error::Error;
use command::{CommandManager, Command, Context};
use futures::prelude::*;
use futures::Future;
use tokio_core::reactor::{Core, Handle};
use std::env;
use std::rc::Rc;
use std::cell::RefCell;
use std::sync::RwLock;
use hyper::Client as HyperClient;
use hyper_tls::HttpsConnector;
use serenity::gateway::Shard;
use serenity::model::event::{Event, GatewayEvent, MessageCreateEvent, ReadyEvent};
use serenity::http::Client as SerenityHttpClient;
use regex::Regex;

pub struct EventHandler {
    handle: Handle,
    serenity_http: Rc<SerenityHttpClient>,
    split_regex: Regex,
    command_manager: Rc<RefCell<CommandManager>>,
}

const PREFIX: &'static str = ">";

impl EventHandler {
    pub fn new(handle: Handle, serenity_http: Rc<SerenityHttpClient>, command_manager: Rc<RefCell<CommandManager>>) -> Result<Self, Error> {
        let split_regex = Regex::new(r"\s+")?;

        Ok(Self {
            handle,
            serenity_http,
            split_regex,
            command_manager,
        })
    }

    pub fn on_event(&self, event: GatewayEvent) {
        use GatewayEvent::Dispatch;
        use Event::*;

        match event {
            Dispatch(_, Ready(e)) => self.on_ready(e),
            Dispatch(_, MessageCreate(e)) => self.on_message(e),
            _ => {
                // ya nothing else
            }
        }
    }

    fn on_ready(&self, _: ReadyEvent) {
        println!("Connected to discord");
    }

    fn on_message(&self, event: MessageCreateEvent) {
        let msg = event.message;
        
        let content = msg.content.clone();
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

        let mut command_manager = self.command_manager.borrow_mut();
        let mut command = match command_manager.commands.get_mut(&command_name.to_lowercase()) {
            Some(command) => command.write().expect("could not get write lock on command"),
            None => {
                // invalid command
                return;
            }
        };

        let args = content_iter.map(|s| s.to_string()).collect::<Vec<String>>();
        println!("args: {:?}", args);

        let context = Context {
            handle: self.handle.clone(), 
            serenity_http: self.serenity_http.clone(),
        };

        let future = command.run(context, msg, args)
            .map_err(|e| error!("oh no couldnt run command: {:?}", e));

        self.handle.spawn(future);
    }
}
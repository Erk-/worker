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
mod command;
mod commands;
mod events;

use error::Error;
use command::{CommandManager, Command};
use events::EventHandler;
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

    let serenity_http = Rc::new(SerenityHttpClient::new(
        http_client.clone(), handle.clone(), Rc::new(token)
    ));

    let mut command_manager = CommandManager::new(handle.clone());
    command_manager.add(Rc::new(RwLock::new(commands::TestCommand {})));
    
    let command_manager = Rc::new(RefCell::new(command_manager));

    let event_handler = EventHandler::new(
        handle.clone(), 
        serenity_http.clone(), 
        command_manager.clone()
    )?;

    #[async]
    for message in shard.messages() {
        let event = shard.parse(message)?;
        shard.process(&event);
        event_handler.on_event(event);
    }

    Ok(())
}
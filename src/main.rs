#![feature(proc_macro, conservative_impl_trait, generators, try_trait, box_syntax, match_default_bindings)]

#[macro_use] extern crate serde_derive;
#[macro_use] extern crate log;
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
extern crate toml;
extern crate serde;

mod error;
mod command;
mod commands;
mod events;
mod cache;
mod config;

use error::Error;
use command::{CommandManager};
use events::EventHandler;
use futures::prelude::*;
use tokio_core::reactor::{Core, Handle};
use std::rc::Rc;
use std::cell::RefCell;
use hyper::Client as HyperClient;
use hyper_tls::HttpsConnector;
use serenity::gateway::Shard;
use serenity::model::event::{Event, GatewayEvent};
use serenity::http::Client as SerenityHttpClient;
use cache::DiscordCache;

fn main() {
    env_logger::init();

    let mut core = Core::new().expect("Error creating event loop");
    let future = try_main(core.handle()).map_err(Box::new);
    println!("Error running future: {:?}", core.run(future));
}

#[async]
fn try_main(handle: Handle) -> Result<(), Error> {
    let config = config::load("config.toml").expect("Could not load config.toml");
    let token = config.discord_token.clone();

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
    command_manager.add(Rc::new(commands::test()));
    let command_manager = Rc::new(RefCell::new(command_manager));

    let discord_cache = Rc::new(RefCell::new(DiscordCache::default()));

    let mut event_handler = EventHandler::new(
        handle.clone(), 
        serenity_http.clone(), 
        command_manager.clone(),
        discord_cache.clone(),
    )?;

    #[async]
    for message in shard.messages() {
        let event = shard.parse(message)?;
        shard.process(&event);
        discord_cache.borrow_mut().update(&event);
        event_handler.on_event(event);
    }

    Ok(())
}
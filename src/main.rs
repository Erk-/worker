#![feature(proc_macro, conservative_impl_trait, generators, try_trait, box_syntax, match_default_bindings)]

#[macro_use] extern crate serde_derive;
#[macro_use] extern crate serde_json;
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
extern crate regex;
extern crate toml;
extern crate serde;
extern crate futures_stream_select_all;

mod error;
mod command;
mod commands;
mod events;
mod cache;
mod config;
mod shards;

use error::Error;
use command::{CommandManager};
use events::EventHandler;
use futures::prelude::*;
use tokio_core::reactor::{Core, Handle};
use std::rc::Rc;
use std::cell::RefCell;
use hyper::Client as HyperClient;
use hyper_tls::HttpsConnector;
use serenity::model::event::{Event, GatewayEvent};
use serenity::http::Client as SerenityHttpClient;
use cache::DiscordCache;
use tungstenite::Error as TungsteniteError;
use futures_stream_select_all::select_all;

fn main() {
    env_logger::init();

    let mut core = Core::new().expect("Error creating event loop");
    let future = try_main(core.handle()).map_err(Box::new);

    if let Err(e) = core.run(future) {
        println!("Error running future: {:?}", e);
    }
}

#[async]
fn try_main(handle: Handle) -> Result<(), Error> {
    let config = config::load("config.toml").expect("Could not load config.toml");
    let token = config.discord_token.clone();
    let sharding = config.sharding();

    let shard_manager = await!(shards::create_shard_manager(
        handle.clone(), token.clone(), sharding,
    ))?;

    let http_client = Rc::new(HyperClient::configure()
        .connector(HttpsConnector::new(4, &handle)?)
        .build(&handle));

    let serenity_http = Rc::new(SerenityHttpClient::new(
        http_client.clone(), handle.clone(), Rc::new(token)
    ));

    let mut command_manager = CommandManager::new(handle.clone());
    command_manager.add(Rc::new(commands::test()));
    command_manager.add(Rc::new(commands::join()));
    command_manager.add(Rc::new(commands::leave()));
    let command_manager = Rc::new(RefCell::new(command_manager));

    let discord_cache = Rc::new(RefCell::new(DiscordCache::default()));

    let mut event_handler = EventHandler::new(
        handle.clone(), 
        serenity_http.clone(), 
        command_manager.clone(),
        discord_cache.clone(),
    )?;

    let shards = shard_manager.shards();
    let streams = shards.into_iter()
        .map(|shard| {
            let stream = shard.borrow_mut().messages();
            stream.map(move |result| {
                (shard.clone(), result)
            })
        })
        .collect::<Vec<_>>();

    #[async]
    for (shard, message) in select_all::<_, _, TungsteniteError>(streams) {
        let event = {
            let mut lock = shard.borrow_mut();
            let event = lock.parse(message)?;
            lock.process(&event);
            event
        };

        discord_cache.borrow_mut().update(&event);
        event_handler.on_event(event, shard);
    }

    Ok(())
}

/*struct MessageSink;

impl Sink for MessageSink {
    type SinkItem = TungsteniteMessage;
    type Error = Error;

    fn start_send(&mut self, item: Self::SinkItem) -> StartSend<Self::SinkItem, Self::SinkError> {
        /*if self.messages.len() == 8 {
            // buffer is full
            return AsyncSink::NotReady;
        }

        self.messages.push(item);*/
        AsyncSink::Ready
    }

    fn poll_complete(&mut self) -> Poll<(), Self::SinkError> {
        /*if self.messages.is_empty() {*/
            Ok(Async::Ready(()))
        /*} else {
            Ok(Async::NotReady)
        }*/
    }
}*/
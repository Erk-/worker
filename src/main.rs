#![feature(proc_macro, generators, try_trait, box_syntax)]

#[macro_use] extern crate serde_derive;
#[macro_use] extern crate serde_json;
#[macro_use] extern crate log;
extern crate env_logger;
extern crate futures_await as futures;
extern crate futures_stream_select_all;
extern crate humantime;
extern crate hyper;
extern crate hyper_tls;
extern crate lavalink;
extern crate lavalink_futures;
extern crate native_tls;
extern crate regex;
extern crate serde;
extern crate serenity;
extern crate tokio_core;
extern crate toml;
extern crate tungstenite;
extern crate websocket;
//extern crate owo; TODO USE THIS

mod cache;
mod command;
mod commands;
mod config;
mod error;
mod events;
mod queue;
mod shards;
mod streams;

use cache::DiscordCache;
use command::CommandManager;
use error::Error;
use events::{DiscordEventHandler, LavalinkEventHandler};
use queue::QueueManager;
use streams::PlaybackManager;

use futures::prelude::{async, await};
use futures_stream_select_all::select_all;
use hyper::Client as HyperClient;
use hyper_tls::HttpsConnector;
use lavalink_futures::nodes::NodeManager;
use serenity::http::Client as SerenityHttpClient;
use serenity::model::event::{Event, GatewayEvent};
use std::cell::{Cell, RefCell};
use std::rc::Rc;
use tokio_core::reactor::{Core, Handle};
use tungstenite::Error as TungsteniteError;

fn main() {
    env_logger::init();

    let mut core = Core::new().expect("Error creating event loop");
    let future = try_main(core.handle()).map_err(Box::new);

    if let Err(e) = core.run(future) {
        error!("Error running future: {:?}", e);
        panic!(e);
    }
}

#[async]
fn try_main(handle: Handle) -> Result<(), Error> {
    let config = config::load("config.toml").expect("Could not load config.toml");
    let token = config.discord_token.clone();
    let sharding = config.sharding();

    let shard_manager = Rc::new(await!(shards::create_shard_manager(
        handle.clone(),
        token.clone(),
        sharding,
    ))?);

    let http_client = Rc::new(
        HyperClient::configure()
            .connector(HttpsConnector::new(4, &handle)?)
            .build(&handle),
    );

    let serenity_http = Rc::new(SerenityHttpClient::new(
        http_client.clone(),
        handle.clone(),
        Rc::new(token),
    )?);

    let mut command_manager = CommandManager::new(
        handle.clone(),
        commands::create(),
    );
    let command_manager = Rc::new(RefCell::new(command_manager));

    let mut queue_manager = QueueManager::default();
    let queue_manager = Rc::new(RefCell::new(queue_manager));

    let discord_cache = Rc::new(RefCell::new(DiscordCache::default()));

    let playback_manager = Rc::new(RefCell::new(PlaybackManager::new(queue_manager.clone())));

    let lavalink_event_handler = RefCell::new(box LavalinkEventHandler::new(
        shard_manager.clone(),
        playback_manager.clone(),
    ));

    let mut node_manager = NodeManager::new(handle.clone(), lavalink_event_handler);

    for node_config in config.node_configs().into_iter() {
        let future = node_manager.add_node(node_config);
        node_manager = await!(future)?;
    }
    let node_manager = Rc::new(RefCell::new(node_manager));

    {
        let mut playback_manager = playback_manager.borrow_mut();
        playback_manager.set_node_manager(node_manager.clone());
    }

    let config = Rc::new(Cell::new(config));
    let mut event_handler = DiscordEventHandler::new(
        handle.clone(),
        http_client.clone(),
        serenity_http.clone(),
        command_manager.clone(),
        discord_cache.clone(),
        node_manager,
        queue_manager,
        playback_manager,
        config,
    )?;

    let shards = shard_manager.shards();
    let streams = shards
        .into_iter()
        .map(|shard| {
            let stream = shard.borrow_mut().messages();
            stream.map(move |message| (shard.clone(), message))
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

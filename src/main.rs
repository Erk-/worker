#![feature(
    async_await,
    await_macro,
    futures_api,
    min_const_fn,
    pin,
    plugin,
    slice_concat_ext,
    trace_macros,
    try_blocks,
    try_from,
    try_trait,
    underscore_imports,
)]
#![allow(dead_code)] // todo: before release, undo this

#[macro_use] extern crate log;
#[macro_use] extern crate redis_async as redis;
#[macro_use] extern crate serde_derive;
#[macro_use] extern crate serde_json;

mod bridges;
mod cache;
mod commands;
mod config;
mod discord_fm;
mod error;
mod lavalink_msgs;
mod radios;
mod services;
mod utils;
mod worker;

pub use crate::error::{Error, Result};

use crate::{
    config::Config,
    worker::Worker,
};
use futures::{
    compat::TokioDefaultSpawner,
    future::{FutureExt as _, TryFutureExt as _},
};
use hyper::rt::Future as _;
use std::env;

const RUST_LOG_DEFAULT: &'static str = "debug,hyper=info,tokio_reactor=info,\
lavalink_http_server_requester=info,lavalink_queue_requester=info";

fn main() {
    trace_macros!(false);

    if env::var("RUST_LOG").is_err() {
        env::set_var("RUST_LOG", RUST_LOG_DEFAULT);
    }

    env_logger::init();

    tokio::run(try_main().boxed().compat(TokioDefaultSpawner).map_err(|why| {
        warn!("Error running tokio loop: {:?}", why);
    }));
}

async fn try_main() -> Result<()> {
    let config = Config::new("config.toml").expect("Could not load config.toml");
    let worker = await!(Worker::new(config))?;
    await!(worker.run())?;

    Ok(())
}

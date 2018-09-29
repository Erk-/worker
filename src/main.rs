#![feature(
    async_await,
    await_macro,
    futures_api,
    min_const_fn,
    pin,
    plugin,
    trace_macros,
    try_from,
    try_trait,
    underscore_imports,
)]
#![allow(dead_code)] // todo: before release, undo this

#[macro_use] extern crate log;
#[macro_use] extern crate redis_async as redis;
#[macro_use] extern crate serde_derive;
#[macro_use] extern crate serde_json;

mod cache;
mod commands;
mod config;
mod error;
mod events;
mod lavalink;
mod queue;
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

fn main() {
    trace_macros!(false);

    if env::var("RUST_LOG").is_err() {
        env::set_var("RUST_LOG", "debug,hyper=info,tokio_reactor=info");
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

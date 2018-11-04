#![feature(
    async_await,
    await_macro,
    futures_api,
    pin,
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
    compat::{Future01CompatExt, Stream01CompatExt},
    future::{FutureExt as _, TryFutureExt as _},
    stream::StreamExt,
};
use hyper::rt::Future as _;
use std::{
    env,
    time::{Duration, Instant},
};
use tokio_signal::unix::{SIGTERM, Signal};
use tokio::{
    runtime::Runtime,
    timer::Delay,
};

const RUST_LOG_DEFAULT: &'static str = "info,hyper=info,tokio_reactor=info,\
lavalink_http_server_requester=info,lavalink_queue_requester=info";

fn main() -> Result<()> {
    if env::var("RUST_LOG").is_err() {
        env::set_var("RUST_LOG", RUST_LOG_DEFAULT);
    }

    env_logger::init();

    let mut rt = Runtime::new()?;

    rt.block_on(try_main().boxed().compat().map(|what| {
        warn!("Completed tokio loop: {:?}", what);
    }).map_err(|why| {
        warn!("Error running tokio loop: {:?}", why);
    })).expect("Err with reactor");

    Ok(())
}

async fn try_main() -> Result<()> {
    {
        let config = Config::new("config.toml")?;
        let worker = await!(Worker::new(config))?;

        let mut signal_future = signal().boxed();
        let mut worker_future = worker.run().boxed();

        futures::select! {
            signal_future => {
                info!("Got SIGTERM signal; shutting down.");
            },
            worker_future => {
                error!("Worker ended: {:?}", worker_future);
            },
        }
    }

    info!("Sleeping for 15 seconds to wait for futures to finish up...");
    await!(Delay::new(Instant::now() + Duration::from_secs(15)).compat())?;

    info!("Exiting try_main");

    Ok(())
}

async fn signal() -> Result<()> {
    let mut signal = await!(Signal::new(SIGTERM).compat())?.compat();

    while let Some(Ok(int)) = await!(signal.next()) {
        if int == SIGTERM {
            break;
        } else {
            warn!("Got signal other than SIGTERM: {}", int);
        }
    }

    Ok(())
}

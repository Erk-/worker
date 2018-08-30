#![feature(
    async_await,
    await_macro,
    box_syntax,
    const_fn,
    decl_macro,
    futures_api,
    generators,
    integer_atomics,
    pin,
    plugin,
    proc_macro_non_items,
    try_trait,
)]

#[macro_use] extern crate log;
#[macro_use] extern crate redis_async as redis;
#[macro_use] extern crate serde_derive;
#[macro_use] extern crate serde_json;

mod cache;
mod command;
mod commands;
mod config;
mod error;
mod events;
mod queue;
mod streams;
mod worker;

pub use crate::error::{Error, Result};

use crate::{
    config::Config,
    worker::Worker,
};

fn main() {
    env_logger::init();

    tokio::run_async(async {
        await!(try_main()).unwrap();
    });
}

async fn try_main() -> Result<()> {
    let config = Config::new("config.toml").expect("Could not load config.toml");
    let worker = await!(Worker::new(config))?;
    await!(worker.run())?;

    Ok(())
}

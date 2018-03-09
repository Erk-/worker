#![feature(proc_macro, conservative_impl_trait, generators, try_trait, box_syntax)]

//#[macro_use]
//extern crate error_chain;
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
use serenity::model::channel::{Message, Channel};

fn main() {
    let mut core = Core::new().expect("Error creating event loop");
    let future = try_main(core.handle()).map_err(Box::new);
    println!("Error running future: {:?}", core.run(future));
}
#[async]
fn try_main(handle: Handle) -> Result<(), Error> {
    let http_client = Rc::new(HyperClient::configure()
        .connector(HttpsConnector::new(4, &handle)?)
        .build(&handle));
    
    let token = env::var("DISCORD_TOKEN").expect("Error no discord token");
    let mut shard = await!(Shard::new(token, [0, 1], handle.clone()))?;

    #[async]
    for message in shard.messages() {
        let event = shard.parse(message)?;
        shard.process(&event);

        use GatewayEvent::Dispatch;
        use Event::*;

        match event {
            Dispatch(_, Ready(e)) => on_ready(e),
            Dispatch(_, MessageCreate(e)) => on_message(e),
            _ => {
                // ya nothing else
            }
        }
    }

    Ok(())
}

fn on_ready(_: ReadyEvent) {
    println!("Connected to discord");
}

fn on_message(event: MessageCreateEvent) {
    let msg = event.message;
    println!("{}#{}: {}", msg.author.name, msg.author.discriminator, msg.content);
}
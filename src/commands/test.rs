use error::Error;
use futures::Future;
use futures::future;
use tokio_core::reactor::Handle;
use std::collections::HashMap;
use serenity::model::channel::Message;
use std::rc::Rc;
use std::sync::RwLock;
use command::Command;
use serenity::http::Client as SerenityHttpClient;

pub struct TestCommand;

impl Command for TestCommand {
    fn names(&self) -> Vec<&'static str> {
        vec!["test", "t", "meme"]
    }
    
    fn run(&mut self, handle: Handle, serenity_http: Rc<SerenityHttpClient>, msg: Message, args: Vec<String>) -> Box<Future<Item = (), Error = Error>> {
        let future = serenity_http.send_message(msg.channel_id.0, |m| m.content("HELLO WORLD"))
            .map(|m| debug!("Sent message {:?}", m))
            .map_err(|e| error!("Error sending message {:?}", e));

        handle.spawn(future);

        box future::ok(())
    }
}
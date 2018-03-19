use error::Error;
use futures::Future;
use futures::future;
use serenity::model::channel::Message;
use command::{Command, Context};

pub struct TestCommand;

impl Command for TestCommand {
    fn names(&self) -> Vec<&'static str> {
        vec!["test", "t", "meme"]
    }

    fn description(&self) -> &'static str {
        "Testing command lol"
    }
    
    fn run(&mut self, ctx: Context, msg: Message, _args: Vec<String>) -> Box<Future<Item = (), Error = Error>> {
        let future = ctx.serenity_http.send_message(msg.channel_id.0, |m| m.content("HELLO WORLD"))
            .map(|m| debug!("Sent message {:?}", m))
            .map_err(|e| error!("Error sending message {:?}", e));

        ctx.handle.spawn(future);

        box future::ok(())
    }
}
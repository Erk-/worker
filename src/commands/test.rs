use futures::prelude::*;
use command::{Command, Context};
use error::Error;

pub fn test() -> Command {
    Command {
        names: vec!["test", "t", "meme"],
        description: "Testing command lol",
        executor: run,
    }
}

#[async(boxed)]
fn run(ctx: Context) -> Result<(), Error> {
    let args = ctx.args;
    
    let future = ctx.serenity_http.send_message(ctx.msg.channel_id.0, |m| m.content(format!("HELLO WORLD {:?}", args)))
        .map(|m| debug!("Sent message {:?}", m))
        .map_err(From::from);

    await!(future)
}
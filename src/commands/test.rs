use futures::prelude::*;
use command::{Command, Context, CommandFuture};

pub fn test() -> Command {
    Command {
        names: vec!["test", "t", "meme"],
        description: "Testing command lol",
        executor: run,
    }
}

fn run(ctx: Context) -> CommandFuture {
    let args = ctx.args.map(|s| s.to_string()).collect::<Vec<String>>();
    
    box ctx.serenity_http.send_message(ctx.msg.channel_id.0, |m| m.content(format!("HELLO WORLD {:?}", args)))
        .map(|m| debug!("Sent message {:?}", m))
        .map_err(From::from)
}

/*pub struct TestCommand;

impl Command for TestCommand {
    fn names(&self) -> Vec<&'static str> {
        vec!["test", "t", "meme"]
    }

    fn description(&self) -> &'static str {
        "Testing command lol"
    }
    
    fn run(&mut self, ctx: Context) -> CommandFuture {
        let args = ctx.args.map(|s| s.to_string()).collect::<Vec<String>>();
        
        box ctx.serenity_http.send_message(ctx.msg.channel_id.0, |m| m.content(format!("HELLO WORLD {:?}", args)))
            .map(|m| debug!("Sent message {:?}", m))
            .map_err(From::from)
    }
}*/
use command::{Command, CommandResult, Context, Response};

use futures::prelude::*;

pub fn pause() -> Command {
    Command {
        names: vec!["pause", "hold"],
        description: "Pause the current song",
        executor: run,
    }
}

#[async(boxed)]
fn run(ctx: Context) -> CommandResult {
    let guild_id = ctx.msg.guild_id?.0;
    let playback_manager = ctx.playback_manager.borrow();

    if let Err(e) = playback_manager.pause(guild_id) {
        error!("error pausing {:?}", e);
        Response::text("error pausing")
    } else {
        Response::text("put it on hold")
    }
}

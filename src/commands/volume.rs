use command::{Command, CommandResult, Context, Response};

use futures::prelude::*;

pub fn volume() -> Command {
    Command {
        names: vec!["volume", "vol"],
        description: "Change the track volume",
        executor: run,
    }
}

#[async(boxed)]
#[cfg(not(feature = "patron"))]
fn run(_ctx: Context) -> CommandResult {
    Response::text("give me money to use this")
}

#[async(boxed)]
#[cfg(feature = "patron")]
fn run(ctx: Context) -> CommandResult {
    if ctx.args.len() != 1 {
        return Response::text("invalid args");
    }

    let volume = ctx.args[0].parse::<i32>()?;
    if volume < 0 || volume > 150 {
        return Response::text("volume gotta be in [0, 150] or else");
    }

    let guild_id = ctx.msg.guild_id?.0;
    let playback_manager = ctx.playback_manager.borrow();

    if let Err(e) = playback_manager.volume(guild_id, volume) {
        error!("error pausing {:?}", e);
        Response::text("error pausing")
    } else {
        Response::text("put it on hold")
    }
}

use crate::command::{Command, CommandResult, Context, Response};

pub fn volume() -> Command {
    Command {
        names: vec!["volume", "vol"],
        description: "Change the track volume",
    }
}

#[cfg(not(feature = "patron"))]
async fn run(_ctx: Context) -> CommandResult {
    Response::text("give me money to use this")
}

#[cfg(feature = "patron")]
async fn run(ctx: Context) -> CommandResult {
    if ctx.args.len() != 1 {
        return Response::text("invalid args");
    }

    let volume = ctx.args[0].parse::<i32>()?;
    if volume < 0 || volume > 150 {
        return Response::text("volume gotta be in [0, 150] or else");
    }

    let guild_id = ctx.msg.guild_id?.0;
    let playback_manager = ctx.playback_manager.lock();

    if let Err(e) = playback_manager.volume(guild_id, volume) {
        error!("error pausing {:?}", e);
        Response::text("error pausing")
    } else {
        Response::text("put it on hold")
    }
}

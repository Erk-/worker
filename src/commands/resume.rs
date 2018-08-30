use crate::command::{Command, CommandResult, Context, Response};

pub fn resume() -> Command {
    Command {
        names: vec!["resume", "unpause", "unhold"],
        description: "Resumes the current song",
    }
}

fn run(ctx: Context) -> CommandResult {
    let guild_id = ctx.msg.guild_id?.0;
    let playback_manager = ctx.playback_manager.lock();

    if let Err(e) = playback_manager.resume(guild_id) {
        error!("error resuming {:?}", e);
        Response::text("error resuming")
    } else {
        Response::text("ok")
    }
}

use command::{Command, CommandResult, Context, Response};

use futures::prelude::*;

pub fn playing() -> Command {
    Command {
        names: vec!["playing", "np", "nowplaying", "current"],
        description: "Get the currently playing song",
        executor: run,
    }
}

#[async(boxed)]
fn run(ctx: Context) -> CommandResult {
    let guild_id = ctx.msg.guild_id?.0;
    let playback_manager = ctx.playback_manager.borrow();
    let state = playback_manager.current(guild_id)?;

    info!("state said {:?}", state);
    Response::text(format!("**Currently Playing:** {}", state))
}
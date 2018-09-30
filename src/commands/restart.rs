use super::prelude::*;

pub const fn description() -> &'static str {
    "Restarts the current song"
}

pub const fn names() -> &'static [&'static str] {
    &["restart"]
}

pub async fn run(ctx: Context) -> CommandResult {
    let guild_id = ctx.msg.guild_id?.0;

    match await!(ctx.state.playback.seek(guild_id, 0)) {
        Ok(()) => {
            Response::text("Restarted the song!")
        }
        Err(why) => {
            warn!("Err restarting song for {}: {:?}", guild_id, why);

            Response::err("There was an error restarting the song.")
        },
    }
}

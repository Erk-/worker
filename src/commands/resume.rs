use super::prelude::*;

pub const fn description() -> &'static str {
    "Resumes the current song"
}

pub const fn names() -> &'static [&'static str] {
    &["unpause", "resume"]
}

pub async fn run(ctx: Context) -> CommandResult {
    let guild_id = ctx.msg.guild_id?.0;

    match await!(ctx.state.playback.resume(guild_id)) {
        Ok(()) => Response::text("Resumed"),
        Err(why) => {
            warn!("Error resuming guild {}: {:?}", guild_id, why);

            Response::text("There was an error resuming")
        },
    }
}

use super::prelude::*;

#[allow(dead_code)]
pub fn description() -> &'static str {
    "Pause the current song"
}

#[allow(dead_code)]
pub fn names() -> &'static [&'static str] {
    &["hold", "pause"]
}

pub async fn run(ctx: Context) -> CommandResult {
    let guild_id = ctx.msg.guild_id?.0;
    match await!(ctx.state.playback.pause(guild_id)) {
        Ok(()) => Response::text("paused"),
        Err(why) => {
            warn!("Error pausing guild id {}: {:?}", guild_id, why);

            Response::text("error pausing")
        },
    }
}

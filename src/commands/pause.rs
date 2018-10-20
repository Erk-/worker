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
    let guild_id = ctx.guild_id()?;

    match await!(ctx.state.playback.pause(guild_id)) {
        Ok(()) => Response::text("The music was paused. Use the `resume` command to play the music again."),
        Err(why) => {
            warn!("Error pausing guild id {}: {:?}", guild_id, why);

            Response::err("There was an error pausing the music.")
        },
    }
}

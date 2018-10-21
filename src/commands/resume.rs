use super::prelude::*;

pub const fn description() -> &'static str {
    "Resumes the current song."
}

pub fn names() -> &'static [&'static str] {
    &["unpause", "resume"]
}

pub async fn run(ctx: Context) -> CommandResult {
    let guild_id = ctx.guild_id()?;

    match await!(ctx.state.playback.resume(guild_id)) {
        Ok(()) => Response::text("Resumed music playback!"),
        Err(why) => {
            warn!("Error resuming guild {}: {:?}", guild_id, why);

            Response::err("There was an error resuming the music.")
        },
    }
}

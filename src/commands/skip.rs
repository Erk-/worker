use super::prelude::*;

pub const fn description() -> &'static str {
    "Skips the current song."
}

pub fn names() -> &'static [&'static str] {
    &["skip", "s", "next"]
}

pub async fn run(ctx: Context) -> CommandResult {
    let guild_id = ctx.guild_id()?;

    match await!(ctx.state.playback.skip(guild_id)) {
        Ok(()) => Response::text("Skipped"),
        Err(why) => {
            warn!("Error skipping guild {}: {:?}", guild_id, why);

            Response::err("There was an error skipping the song.")
        },
    }
}

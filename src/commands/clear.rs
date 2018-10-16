use super::prelude::*;

pub const fn description() -> &'static str {
    "Clears the song queue"
}

pub const fn names() -> &'static [&'static str] {
    &["clear"]
}

pub async fn run(ctx: Context) -> CommandResult {
    let guild_id = ctx.msg.guild_id?.0;

    match await!(ctx.state.queue.clear(guild_id)) {
        Ok(()) => Response::text("Cleared the song queue!"),
        Err(why) => {
            warn!("Err clearing queue for {}: {:?}", guild_id, why);

            Response::text("Cleared the song queue!")
        },
    }
}

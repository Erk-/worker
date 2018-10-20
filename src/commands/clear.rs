use super::prelude::*;

pub const fn description() -> &'static str {
    "Clears the song queue"
}

pub fn names() -> &'static [&'static str] {
    &["clear"]
}

pub async fn run(ctx: Context) -> CommandResult {
    let guild_id = ctx.guild_id()?;

    match await!(ctx.state.queue.clear(guild_id)) {
        Ok(()) | Err(Error::LavalinkQueueRequester(QueueError::NotFound)) => {
            Response::text("Cleared the song queue!")
        },
        Err(why) => {
            warn!("Err clearing queue for {}: {:?}", guild_id, why);

            Response::err("There was an error clearing the queue.")
        },
    }
}

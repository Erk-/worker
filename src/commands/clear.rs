use super::prelude::*;

pub static COMMAND_INSTANCE: ClearCommand = ClearCommand;

pub struct ClearCommand;

impl ClearCommand {
    async fn _run(ctx: Context) -> CommandResult {
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
}

impl<'a> Command<'a> for ClearCommand {
    fn names(&self) -> &'static [&'static str] {
        &["clear"]
    }

    fn description(&self) -> &'static str {
        "Clears the song queue."
    }

    fn run(&self, ctx: Context) -> FutureObj<'a, CommandResult> {
        FutureObj::new(Self::_run(ctx).boxed())
    }
}

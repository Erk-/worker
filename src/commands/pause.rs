use super::prelude::*;

pub static COMMAND_INSTANCE: PauseCommand = PauseCommand;

pub struct PauseCommand;

impl PauseCommand {
    async fn _run(ctx: Context) -> CommandResult {
        let guild_id = ctx.guild_id()?;

        match await!(ctx.state.playback.pause(guild_id)) {
            Ok(()) => {
                Response::text(
                    "The music was paused. Use the `resume` command to play the music again.",
                )
            },
            Err(why) => {
                warn!("Error pausing guild id {}: {:?}", guild_id, why);

                Response::err("There was an error pausing the music.")
            },
        }
    }
}

impl<'a> Command<'a> for PauseCommand {
    fn description(&self) -> &'static str {
        "Pauses the current song."
    }

    fn names(&self) -> &'static [&'static str] {
        &["hold", "pause"]
    }

    fn run(&self, ctx: Context) -> RunFuture<'a> {
        RunFuture::new(Self::_run(ctx).boxed())
    }
}

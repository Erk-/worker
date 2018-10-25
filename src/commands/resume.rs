use super::prelude::*;

pub struct ResumeCommand;

impl ResumeCommand {
    async fn _run(ctx: Context) -> CommandResult {
        let guild_id = ctx.guild_id()?;

        match await!(ctx.state.playback.resume(guild_id)) {
            Ok(()) => Response::text("Resumed music playback!"),
            Err(why) => {
                warn!("Error resuming guild {}: {:?}", guild_id, why);

                Response::err("There was an error resuming the music.")
            },
        }
    }
}

impl<'a> Command<'a> for ResumeCommand {
    fn names(&self) -> &'static [&'static str] {
        &["unpause", "resume"]
    }

    fn description(&self) -> &'static str {
        "Resumes the current song."
    }

    fn run(&self, ctx: Context) -> RunFuture<'a> {
        RunFuture::new(Self::_run(ctx).boxed())
    }
}

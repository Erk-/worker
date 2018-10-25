use super::prelude::*;

pub struct RestartCommand;

impl RestartCommand {
    async fn _run(ctx: Context) -> CommandResult {
        let guild_id = ctx.guild_id()?;

        match await!(ctx.state.playback.seek(guild_id, 0)) {
            Ok(()) => Response::text("Restarted the song!"),
            Err(why) => {
                warn!("Err restarting song for {}: {:?}", guild_id, why);

                Response::err("There was an error restarting the song.")
            },
        }
    }
}

impl<'a> Command<'a> for RestartCommand {
    fn names(&self) -> &'static [&'static str] {
        &["restart", "rs"]
    }

    fn description(&self) -> &'static str {
        "Restarts the current song."
    }

    fn run(&self, ctx: Context) -> RunFuture<'a> {
        RunFuture::new(Self::_run(ctx).boxed())
    }
}

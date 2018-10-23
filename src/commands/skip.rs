use super::prelude::*;

pub static COMMAND_INSTANCE: SkipCommand = SkipCommand;

pub struct SkipCommand;

impl SkipCommand {
    async fn _run(ctx: Context) -> CommandResult {
        let guild_id = ctx.guild_id()?;

        match await!(ctx.state.playback.skip(guild_id)) {
            Ok(()) => Response::text("Skipped"),
            Err(why) => {
                warn!("Error skipping guild {}: {:?}", guild_id, why);

                Response::err("There was an error skipping the song.")
            },
        }
    }
}

impl<'a> Command<'a> for SkipCommand {
    fn names(&self) -> &'static [&'static str] {
        &["skip", "s", "next"]
    }

    fn description(&self) -> &'static str {
        "Skips the current song."
    }

    fn run(&self, ctx: Context) -> RunFuture<'a> {
        RunFuture::new(Self::_run(ctx).boxed())
    }
}

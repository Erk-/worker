use super::prelude::*;

pub static COMMAND_INSTANCE: ShuffleCommand = ShuffleCommand;

pub struct ShuffleCommand;

impl ShuffleCommand {
    async fn _run(ctx: Context) -> CommandResult {
        let guild_id = ctx.guild_id()?;

        match await!(ctx.state.queue.shuffle(guild_id)) {
            Ok(()) => Response::text("Shuffled the song queue."),
            Err(why) => {
                warn!("Error shuffling guild {}: {:?}", guild_id, why);

                Response::err("There was an error shuffling the song.")
            },
        }
    }
}

impl<'a> Command<'a> for ShuffleCommand {
    fn names(&self) -> &'static [&'static str] {
        &["shuffle", "shufle"]
    }

    fn description(&self) -> &'static str {
        "Shuffles the current song queue."
    }

    fn run(&self, ctx: Context) -> RunFuture<'a> {
        info!("running shuffle");
        RunFuture::new(Self::_run(ctx).boxed())
    }
}

use super::prelude::*;

pub static COMMAND_INSTANCE: SeekCommand = SeekCommand;
const ERROR_SEEKING: &'static str = "There was an error seeking the song.";

pub struct SeekCommand;

impl SeekCommand {
    async fn _run(ctx: Context) -> CommandResult {
        let guild_id = ctx.guild_id()?;

        match await!(ctx.is_playing()) {
            Ok(true) => {},
            Ok(false) => {
                return super::no_song();
            },
            Err(_) => {
                return Response::err(ERROR_SEEKING);
            },
        };

        match await!(ctx.state.playback.seek(guild_id, 0)) {
            Ok(()) => {
                Response::text(
                    "Jumped to the specified position. Use `!!!playing` to see the current song & \
                     position.",
                )
            },
            Err(why) => {
                warn!("Err seeking song for {} to {}: {:?}", guild_id, 0, why);

                Response::err(ERROR_SEEKING)
            },
        }
    }
}

impl<'a> Command<'a> for SeekCommand {
    fn names(&self) -> &'static [&'static str] {
        &["seek", "jump"]
    }

    fn description(&self) -> &'static str {
        "Skips to a certain timestamp in the current song."
    }

    fn run(&self, ctx: Context) -> RunFuture<'a> {
        RunFuture::new(Self::_run(ctx).boxed())
    }
}

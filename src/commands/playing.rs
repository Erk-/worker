use super::prelude::*;

pub struct PlayingCommand;

impl PlayingCommand {
    async fn _run(ctx: Context) -> CommandResult {
        let guild_id = ctx.guild_id()?;
        let state = match await!(ctx.state.playback.current(guild_id)) {
            Ok(state) => state,
            Err(why) => {
                warn!("Err getting state for {}: {:?}", guild_id, why);

                return Response::err("There was an error getting the current song.");
            },
        };

        debug!("Player state for {}: {:?}", guild_id, state);

        Response::text(state.to_string())
    }
}

impl<'a> Command<'a> for PlayingCommand {
    fn names(&self) -> &'static [&'static str] {
        &["current", "currently", "nowplaying", "np", "playing"]
    }

    fn description(&self) -> &'static str {
        "Gets the currently playing song."
    }

    fn run(&self, ctx: Context) -> RunFuture<'a> {
        RunFuture::new(Self::_run(ctx).boxed())
    }
}

use super::prelude::*;

pub const fn description() -> &'static str {
    "Get the currently playing song"
}

pub const fn names() -> &'static [&'static str] {
    &["currently", "nowplaying", "np", "playing"]
}

pub async fn run(ctx: Context) -> CommandResult {
    let guild_id = ctx.msg.guild_id?.0;
    let state = match await!(ctx.state.playback.current(guild_id)) {
        Ok(state) => state,
        Err(why) => {
            warn!("Err getting state for {}: {:?}", guild_id, why);

            return Response::text(
                "There was an error getting the currents song",
            );
        },
    };

    info!("state said {:?}", state);
    Response::text(format!("**Currently Playing:** {}", state))
}

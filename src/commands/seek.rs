use super::prelude::*;

pub const fn description() -> &'static str {
    "Skips to a time in the current song"
}

pub const fn names() -> &'static [&'static str] {
    &["seek", "jump"]
}

const ERROR_SEEKING: &'static str = "There was an error seeking the song.";

pub async fn run(ctx: Context) -> CommandResult {
    let guild_id = ctx.msg.guild_id?.0;

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
            Response::text("Jumped to the specified position. Use `!!!playing` to see the current song & position.")
        }
        Err(why) => {
            warn!("Err seeking song for {} to {}: {:?}", guild_id, 0, why);

            Response::err(ERROR_SEEKING)
        },
    }
}

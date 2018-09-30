use crate::utils;
use super::prelude::*;

pub const fn description() -> &'static str {
    "plays a song"
}

pub const fn names() -> &'static [&'static str] {
    &["play", "p", "search", "youtube", "soundcloud"]
}

pub async fn run(ctx: Context) -> CommandResult {
    if ctx.args.len() < 1 {
        return Response::text("You need to say the link to the song or the name of what you want to play");
    }

    let query = ctx.args.join(" ");

    let mut tracks = match await!(ctx.state.playback.search(query.clone())) {
        Ok(tracks) => tracks,
        Err(why) => {
            warn!("Err searching tracks for query '{}': {:?}", query, why);

            return Response::err("There was an error searching for that.");
        },
    };

    let song = tracks.tracks.remove(0);

    match await!(ctx.state.playback.play(ctx.msg.guild_id?.0, song.track)) {
        Ok(()) => {
            Response::text(format!(
                "Now playing **{}** by **{}** `[{}]`",
                song.info.title,
                song.info.author,
                utils::track_length_readable(song.info.length as u64),
            ))
        },
        Err(why) => {
            warn!("Err playing song: {:?}", why);

            Response::err("There was an error playing the song.")
        },
    }
}

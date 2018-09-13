use super::prelude::*;

pub const fn description() -> &'static str {
    "plays a song"
}

pub const fn names() -> &'static [&'static str] {
    &["play", "p", "search", "youtube", "soundcloud"]
}

pub async fn run(mut ctx: Context) -> CommandResult {
    if ctx.args.len() < 1 {
        return Response::text("You need to say the link to the song or the name of what you want to play");
    }

    let query = ctx.args.remove(0);

    let mut tracks = match await!(ctx.state.playback.search(query.clone())) {
        Ok(tracks) => tracks,
        Err(why) => {
            warn!("Err searching tracks for query '{}': {:?}", query, why);

            return Response::text("There was an error searching for that");
        },
    };

    let song = tracks.tracks.remove(0);

    match await!(ctx.state.playback.play(ctx.msg.guild_id?.0, song.track)) {
        Ok(()) => {
            let seconds_total = (song.info.length as f64 / 1000f64).floor() as i64;
            let minutes = (seconds_total as f64 / 60f64).floor();
            let seconds = seconds_total % 60;
            let text = format!(
                "Now playing **{}** by **{}** `[{:02}:{:02}]`",
                song.info.title,
                song.info.author,
                minutes,
                seconds,
            );

            Response::text(text)
        },
        Err(why) => {
            warn!("Err playing song: {:?}", why);

            Response::text("There was an error playing the song")
        },
    }
}

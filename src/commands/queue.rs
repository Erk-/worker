use std::fmt::Write as _;
use super::prelude::*;

pub const fn description() -> &'static str {
    "Shows the song queue"
}

pub const fn names() -> &'static [&'static str] {
    &["queue", "q"]
}

pub async fn run(ctx: Context) -> CommandResult {
    let guild_id = ctx.msg.guild_id?.0;

    let queue = match await!(ctx.state.queue.get(guild_id)) {
        Ok(queue) => queue,
        Err(why) => {
            warn!("Err getting queue for {}: {:?}", guild_id, why);

            return Response::text("There was an error getting the queue");
        },
    };

    let current = match await!(ctx.state.playback.current(guild_id)) {
        Ok(current) => current,
        Err(why) => {
            warn!("Err getting current music for {}: {:?}", guild_id, why);

            return Response::text("There was an error getting the current song in the queue");
        },
    };

    let mut s = format!("__Currently playing:__\n{}\n\n", current);

    if queue.is_empty() {
        s.push_str("There are no songs in the queue.");
    } else {
        for (idx, item) in queue.iter().enumerate() {
            write!(
                s,
                "`{:02}` **{}** by **{}** `[foo`]",
                idx,
                item.song_title,
                item.song_author,
            );
        }
    }

    Response::text(s)
}

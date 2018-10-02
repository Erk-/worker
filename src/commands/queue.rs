use crate::utils;
use lavalink_queue_requester::model::QueuedItem;
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

    let queue = match await!(ctx.queue()) {
        Ok(queue) => queue,
        Err(why) => {
            warn!("Err getting queue for {}: {:?}", guild_id, why);

            return Response::err("There was an error getting the queue.");
        },
    };

    let current = match await!(ctx.state.playback.current(guild_id)) {
        Ok(current) => current,
        Err(why) => {
            warn!("Err getting current music for {}: {:?}", guild_id, why);

            return Response::err(
                "There was an error getting the current song in the queue",
            );
        },
    };

    let mut s = format!("{}\n\n", current);

    if queue.is_empty() {
        s.push_str("There are no songs in the queue.");
    } else {
        format_queue(queue, &mut s);
    }

    Response::text(s)
}

fn format_queue(queue: impl IntoIterator<Item = QueuedItem>, buf: &mut String) {
    for (idx, item) in queue.into_iter().enumerate() {
        write!(
            buf,
            "`{:02}` **{}** by **{}** `[{}]`\n",
            idx + 1,
            item.song_title,
            item.song_author,
            utils::track_length_readable(item.song_length as u64),
        );
    }
}

#[cfg(test)]
mod tests {
    use super::QueuedItem;

    #[test]
    fn test_queue_one_song() {
        let item = vec![QueuedItem {
            song_id: 5,
            song_author: "xKito Music".to_owned(),
            song_identifier: "zcn4-taGvlg".to_owned(),
            song_length: 184_000,
            song_source: "youtube".to_owned(),
            song_stream: false,
            song_title: "she - Prismatic".to_owned(),
            song_track: "QAAAdAIAD3NoZSAtIFByaXNtYXRpYwALeEtpdG8gTXVzaWMAAAAAAA\
LOwAALemNuNC10YUd2bGcAAQAraHR0cHM6Ly93d3cueW91dHViZS5jb20vd2F0Y2g/dj16Y240LXRhR\
3ZsZwAHeW91dHViZQAAAAAAAAAA".to_owned(),
            song_url: "https://www.youtube.com/watch?v=zcn4-taGvlg".to_owned().into(),
        }];

        let mut buf = String::new();

        super::format_queue(item, &mut buf);

        assert_eq!(
            buf,
            "`01` **she - Prismatic** by **xKito Music** `[3m 4s]`\n",
        );
    }
}

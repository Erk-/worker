use crate::utils;
use lavalink_queue_requester::model::QueuedItem;
use serenity::utils::MessageBuilder;
use std::fmt::Write as _;
use super::prelude::*;

pub struct QueueCommand;

impl QueueCommand {
    async fn _run(ctx: Context) -> CommandResult {
        let guild_id = ctx.guild_id()?;

        trace!("Requested page: {:?}", ctx.args.first());

        let page = Self::calculate_page(ctx.args.first().map(|x| &**x));
        let start = page * 10;

        let queue = match await!(ctx.queue(10, start)) {
            Ok(queue) => queue,
            Err(why) => {
                warn!("Err getting queue for {}: {:?}", guild_id, why);

                return Response::err("There was an error getting the queue.");
            },
        };

        let mut msg = MessageBuilder::new();

        match await!(ctx.state.playback.current(guild_id)) {
            Ok(current) => {
                write!(msg.0, "{}", current)?;
            },
            Err(why) => {
                warn!("Err getting current music for {}: {:?}", guild_id, why);

                msg.0.push_str("There was an error getting the current song.");
            },
        }

        msg.0.push_str("\n\n__Queue__:\n");

        if page == 0 && queue.is_empty() {
            msg.0.push_str("There are no songs in the queue.");
        } else if queue.is_empty() {
            msg.0.push_str("There are no songs on this page of the queue.");
        } else {
            Self::format_queue(queue, &mut msg, start as usize);
        }

        if msg.0.len() > 2000 {
            msg.0.truncate(1997);
            msg.0.push_str("...");
        }

        Response::text(msg.build())
    }

    fn calculate_page(arg: Option<&str>) -> u32 {
        let mut requested = arg.and_then(|x| x.parse().ok()).unwrap_or(0);

        if requested > 0 {
            requested -= 1;
        }

        requested
    }

    fn format_queue(
        queue: impl IntoIterator<Item = QueuedItem>,
        msg: &mut MessageBuilder,
        start: usize,
    ) {
        for (idx, item) in queue.into_iter().enumerate() {
            write!(msg.0, "`{:02}`", start + idx + 1);

            msg.0.push(' ');
            msg.push_bold_safe(item.song_title);
            msg.0.push_str(" by ");
            msg.push_bold_safe(item.song_author);
            msg.0.push(' ');

            let length = utils::track_length_readable(item.song_length as u64);
            msg.push_mono_safe(format!("[{}]", length));
            msg.0.push('\n');
        }
    }
}

impl<'a> Command<'a> for QueueCommand {
    fn names(&self) -> &'static [&'static str] {
        &["queue", "q", "que"]
    }

    fn description(&self) -> &'static str {
        "Shows the song queue."
    }

    fn run(&self, ctx: Context) -> RunFuture<'a> {
        RunFuture::new(Self::_run(ctx).boxed())
    }
}

#[cfg(test)]
mod tests {
    use serenity::utils::MessageBuilder;
    use super::{QueueCommand, QueuedItem};

    #[test]
    fn test_page_0() {
        assert_eq!(QueueCommand::calculate_page(Some("0")), 0);
        assert_eq!(QueueCommand::calculate_page(Some("1")), 0);
    }

    #[test]
    fn test_page_no_arg() {
        assert_eq!(QueueCommand::calculate_page(None), 0);
    }

    #[test]
    fn test_page_numbered() {
        assert_eq!(QueueCommand::calculate_page(Some("7")), 6);
        assert_eq!(QueueCommand::calculate_page(Some("1500")), 1499);
    }

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

        let mut msg = MessageBuilder::new();

        super::QueueCommand::format_queue(item, &mut msg, 0);

        assert_eq!(
            msg.build(),
            "`01` **she - Prismatic** by **xKito Music** `[3m 4s]`\n",
        );
    }
}

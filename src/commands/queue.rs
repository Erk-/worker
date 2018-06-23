use command::{Command, Context, CommandResult, Response};

use futures::prelude::*;
use lavalink::decoder;
use std::time::Duration;

pub fn queue() -> Command {
    Command {
        names: vec!["queue", "q"],
        description: "Show the queue",
        executor: run,
    }
}

#[async(boxed)]
fn run(ctx: Context) -> CommandResult {
    let guild_id = ctx.msg.guild_id?.0;

    let (size, queue) = {
        let mut queue_manager = ctx.queue_manager.try_borrow_mut()?;
        let queue_lock = queue_manager.get_or_create(guild_id);
        let queue = queue_lock.borrow();

        (queue.size(), queue.peek())
    };

    let mut formatted = queue.iter()
        .filter_map(|track| decoder::decode_track_base64(&track).ok())
        .enumerate()
        .map(|e| format!("`{}` {} by {} ({:#?})", e.0, e.1.title, e.1.author, Duration::from_millis(e.1.length)).to_string())
        .collect::<Vec<_>>();
    
    formatted.truncate(10);
    let content = formatted.join("\n");

    Response::text(format!("**Queue:** {} tracks: \n\n{}", size, content))
}
use command::{Command, Context, CommandResult, Response};

use futures::prelude::*;

pub fn skip() -> Command {
    Command {
        names: vec!["skip", "s", "next"],
        description: "Skip the current song",
        executor: run,
    }
}

#[async(boxed)]
fn run(ctx: Context) -> CommandResult {
    let user_id = ctx.msg.author.id.0;
    let guild_id = ctx.msg.guild_id?.0;

    let cache_lock = ctx.discord_cache.borrow();
    let voice_state = cache_lock.get_user_voice_state(&guild_id, &user_id);
    
    if voice_state.is_none() {
        return Response::text("NO VOICE STATE");
    }
    
    let playback_manager = ctx.playback_manager.borrow();
    
    if let Err(e) = playback_manager.play_next_guild(guild_id, true) {
        error!("error playing {:?}", e);
        Response::text("error skipping")
    } else {
        Response::text("alright skipped")
    }
}
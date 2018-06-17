use command::{Command, Context, CommandResult, Response};

use futures::prelude::*;

pub fn play() -> Command {
    Command {
        names: vec!["play", "p"],
        description: "plays a song",
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

    let node_manager = ctx.node_manager.borrow_mut();
    let mut player_manager = node_manager.player_manager.borrow_mut();
    
    let mut player = match player_manager.get_mut(&guild_id) {
        Some(player) => player,
        None => {
            return Response::text("no player in this guild");
        }
    };
    
    if let Err(e) = player.play("QAAAoQIALk1JTkUgRElBTU9ORFMgfCBtaU5FQ1JBRlQgUEFST0RZIE9GIFRBS0UgT04gTUUAGU1pbmVDcmFmdCBBd2Vzb21lIFBhcm9keXMAAAAAAAOKQAALZGdoYTlTMzlZNk0AAQAraHR0cHM6Ly93d3cueW91dHViZS5jb20vd2F0Y2g/dj1kZ2hhOVMzOVk2TQAHeW91dHViZQAAAAAAAAAA==", None, None) {
        error!("error playing track: {:?}", e);
        Response::text("error playing track")
    } else {
        Response::text("playing song")
    }
}
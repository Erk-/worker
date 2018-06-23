use command::{Command, CommandResult, Context, Response};

use futures::prelude::*;
use serenity::constants::VoiceOpCode;
use tungstenite::Message as TungsteniteMessage;

pub fn leave() -> Command {
    Command {
        names: vec!["leave", "l", "disconnect"],
        description: "leaves the voice channel",
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
        return Response::text("no voice state");
    }

    let mut node_manager = ctx.node_manager.borrow_mut();
    if node_manager.remove_player(&guild_id)? {
        trace!("removed player for guild {}", &guild_id);
    }

    let none_channel_id: Option<u64> = None;

    let map = json!({
        "op": VoiceOpCode::SessionDescription.num(),
        "d": {
            "channel_id": none_channel_id,
            "guild_id": guild_id,
            "self_deaf": true,
            "self_mute": false,
        }
    });

    let mut shard_lock = ctx.shard.borrow_mut();

    match shard_lock.send(TungsteniteMessage::Text(map.to_string())) {
        Ok(_) => Response::text("left voice channel"),
        Err(e) => {
            error!("Error leaving voice channel {:?}", e);
            Response::text("error leaving voice channel")
        }
    }
}

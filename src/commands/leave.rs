use futures::prelude::*;
use command::{Command, Context};
use error::Error;
use tungstenite::Message as TungsteniteMessage;
use serenity::constants::VoiceOpCode;

pub fn leave() -> Command {
    Command {
        names: vec!["leave", "l", "disconnect"],
        description: "leaves the voice channel",
        executor: run,
    }
}

#[async(boxed)]
fn run(ctx: Context) -> Result<(), Error> {
    let channel_id = ctx.msg.channel_id.0;
    let user_id = ctx.msg.author.id.0;

    let (guild_id, voice_state) = {
        let cache_lock = ctx.discord_cache.borrow();
        let guild_id = cache_lock.get_guild_by_channel(&channel_id)?.clone();
        let voice_state = cache_lock.get_user_voice_state(&guild_id, &user_id);
        (guild_id, voice_state)
    };

    let voice_state = match voice_state {
        Some(voice_state) => voice_state,
        None => {
            ctx.send_message(|m| m.content("NO VOICE STATE"));
            return Ok(());
        },
    };

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
    shard_lock.send(TungsteniteMessage::Text(map.to_string()))?;
    
    Ok(())
}
use futures::prelude::*;
use command::{Command, Context};
use error::Error;
use tungstenite::Message as TungsteniteMessage;
use serenity::constants::VoiceOpCode;

pub fn join() -> Command {
    Command {
        names: vec!["join", "j", "connect"],
        description: "joins the voice channel",
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
    
    {
        let player_manager = node_manager.player_manager.borrow();
        if player_manager.has(&guild_id) {
            ctx.send_message(|m| m.content("YO WE ALREADY PLAYING LMAO"));
            return Ok(());
        }
    }

    node_manager.create_player(guild_id, None)?;
    trace!("created audio player for guild {}", &guild_id);

    let map = json!({
        "op": VoiceOpCode::SessionDescription.num(),
        "d": {
            "channel_id": voice_state.channel_id,
            "guild_id": guild_id,
            "self_deaf": true,
            "self_mute": false,
        }
    });

    let mut shard_lock = ctx.shard.borrow_mut();
    
    match shard_lock.send(TungsteniteMessage::Text(map.to_string())) {
        Ok(_) => ctx.send_message(|m| m.content("joined vc")),
        Err(e) => error!("Error joining voice channel {:?}", e),
    }
    
    Ok(())
}
use futures::prelude::*;
use command::{Command, Context};
use error::Error;
use tungstenite::Message as TungsteniteMessage;
use serenity::constants::VoiceOpCode;

pub fn play() -> Command {
    Command {
        names: vec!["play", "p"],
        description: "plays a song",
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

    let node_manager = ctx.node_manager.borrow_mut();
    let mut player_manager = node_manager.player_manager.borrow_mut();
    
    let mut player = match player_manager.get_mut(&guild_id) {
        Some(player) => player,
        None => {
            ctx.send_message(|m| m.content("no player in this guild"));
            return Ok(());
        }
    };
    
    player.play(
        "QAAAoQIALk1JTkUgRElBTU9ORFMgfCBtaU5FQ1JBRlQgUEFST0RZIE9GIFRBS0UgT04gTUUAGU1pbmVDcmFmdCBBd2Vzb21lIFBhcm9keXMAAAAAAAOKQAALZGdoYTlTMzlZNk0AAQAraHR0cHM6Ly93d3cueW91dHViZS5jb20vd2F0Y2g/dj1kZ2hhOVMzOVk2TQAHeW91dHViZQAAAAAAAAAA", 
        None, None
    )?;

    ctx.send_message(|m| m.content("playing song!"));
    
    Ok(())
}
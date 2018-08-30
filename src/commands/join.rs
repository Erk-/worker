use serenity::constants::VoiceOpCode;
use super::prelude::*;

#[allow(dead_code)]
pub fn description() -> String {
    "joins the voice channel".to_owned()
}

#[allow(dead_code)]
pub fn names() -> Vec<String> {
    vec![
        "connect".to_owned(),
        "j".to_owned(),
        "join".to_owned(),
    ]
}

pub async fn run(ctx: Context) -> CommandResult {
    let user_id = ctx.msg.author.id.0;
    let guild_id = ctx.msg.guild_id?.0;

    let cache_lock = ctx.state.cache.read();
    let voice_state = cache_lock.get_user_voice_state(&guild_id, &user_id);

    let voice_state = match voice_state {
        Some(voice_state) => voice_state,
        None => {
            return Response::text("no voice state");
        }
    };

    // todo
    // let mut node_manager = ctx.node_manager.lock();

    // {
    //     let player_manager = node_manager.player_manager.lock();
    //     if player_manager.has(&guild_id) {
    //         return Response::text("YO WE ALREADY PLAYING LMAO");
    //     }
    // }

    // node_manager.create_player(guild_id, None)?;
    trace!("created audio player for guild {}", &guild_id);

    // let map = json!({
    //     "op": VoiceOpCode::SessionDescription.num(),
    //     "d": {
    //         "channel_id": voice_state.channel_id,
    //         "guild_id": guild_id,
    //         "self_deaf": true,
    //         "self_mute": false,
    //     }
    // });

    // todo
    // let mut shard_lock = ctx.shard.lock();

    // match shard_lock.send(TungsteniteMessage::Text(map.to_string())) {
    //     Ok(_) => Response::text("joined voice channel"),
    //     Err(e) => {
    //         error!("Error joining voice channel {:?}", e);
    //         Response::text("error joining voice channel!")
    //     }
    // }
    Response::text("")
}

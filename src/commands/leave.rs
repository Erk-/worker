use serenity::constants::VoiceOpCode;
use super::prelude::*;

#[allow(dead_code)]
pub fn description() -> String {
    "leaves the voice channel".to_owned()
}

#[allow(dead_code)]
pub fn names() -> Vec<String> {
    vec![
        "disconnect".to_owned(),
        "l".to_owned(),
        "leave".to_owned(),
    ]
}

pub async fn run(ctx: Context) -> CommandResult {
    let user_id = ctx.msg.author.id.0;
    let guild_id = ctx.msg.guild_id?.0;

    let cache_lock = ctx.state.cache.read();
    let voice_state = cache_lock.get_user_voice_state(&guild_id, &user_id);

    if voice_state.is_none() {
        return Response::text("no voice state");
    }

    match await!(ctx.state.playback.stop(guild_id)) {
        Ok(()) => Response::text("left voice channel"),
        Err(e) => {
            error!("Error leaving voice channel {:?}", e);
            Response::text("error leaving voice channel")
        }
    }
}

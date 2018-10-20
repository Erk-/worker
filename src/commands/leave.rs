use crate::services::lavalink::PlaybackManager;
use redis_async::client::PairedConnection;
use serenity::constants::VoiceOpCode;
use std::sync::Arc;
use super::prelude::*;

pub const fn description() -> &'static str {
    "leaves the voice channel"
}

pub fn names() -> &'static [&'static str] {
    &[
        "disconnect",
        "l",
        "leave",
    ]
}

pub async fn run(ctx: Context) -> CommandResult {
    let gid = ctx.msg.guild_id?.0;
    let sid = ctx.shard_id;

    match await!(leave(sid, gid, &ctx.state.playback, &ctx.state.redis)) {
        Ok(()) => Response::text("Stopped playing music & left the voice channel."),
        Err(why) => {
            error!(
                "Error stopping in guild {}: {:?}",
                gid,
                why,
            );

            Response::err("There was an error leaving the voice channel.")
        }
    }
}

pub async fn leave<'a>(
    shard_id: u64,
    guild_id: u64,
    playback: &'a Arc<PlaybackManager>,
    redis: &'a Arc<PairedConnection>,
) -> Result<()> {
    let map = serde_json::to_vec(&json!({
        "op": VoiceOpCode::SessionDescription.num(),
        "d": {
            "channel_id": None::<Option<u64>>,
            "guild_id": guild_id,
            "self_deaf": true,
            "self_mute": false,
        },
    }))?;
    let key = format!("sharder:to:{}", shard_id);
    let cmd = resp_array!["RPUSH", key, map];

    redis.send_and_forget(cmd);
    redis.send_and_forget(resp_array![
        "DEL",
        format!("j:{}", guild_id)
    ]);
    redis.send_and_forget(resp_array![
        "DEL",
        format!("c:{}", guild_id)
    ]);

    await!(playback.stop(guild_id))?;

    Ok(())
}

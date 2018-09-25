use serenity::constants::VoiceOpCode;
use super::prelude::*;

pub const fn description() -> &'static str {
    "leaves the voice channel"
}

pub const fn names() -> &'static [&'static str] {
    &[
        "disconnect",
        "l",
        "leave",
    ]
}

pub async fn run(ctx: Context) -> CommandResult {
    let guild_id = ctx.msg.guild_id?.0;

    let map = serde_json::to_vec(&json!({
        "op": VoiceOpCode::SessionDescription.num(),
        "d": {
            "channel_id": None::<Option<u64>>,
            "guild_id": guild_id,
            "self_deaf": true,
            "self_mute": false,
        },
    }))?;
    await!(ctx.to_sharder(map))?;

    match await!(ctx.state.playback.stop(guild_id)) {
        Ok(()) => Response::text("Stopped playing music & left the voice channel."),
        Err(why) => {
            error!(
                "Error stopping in guild {}: {:?}",
                guild_id,
                why,
            );

            Response::text("There was an error leaving the voice channel.")
        }
    }
}

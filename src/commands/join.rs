use serenity::constants::VoiceOpCode;
use super::prelude::*;

pub const fn description() -> &'static str {
    "joins the voice channel"
}

pub const fn names() -> &'static [&'static str] {
    &[
        "connect",
        "j",
        "join",
    ]
}

pub async fn run(ctx: Context) -> CommandResult {
    let user_id = ctx.msg.author.id.0;
    let guild_id = ctx.msg.guild_id?.0;

    let voice_state = match await!(ctx.state.cache.voice_state(guild_id, user_id)) {
        Ok(Some(voice_state)) => voice_state.clone(),
        Ok(None) => return Response::text("You aren't in a voice channel."),
        Err(why) => {
            warn!("Err getting user voice state: {:?}", why);
            return Response::text("There was an error joining the voice channel.");
        },
    };

    trace!("Created audio player for guild {}", guild_id);

    let map = serde_json::to_vec(&json!({
        "op": VoiceOpCode::SessionDescription.num(),
        "d": {
            "channel_id": voice_state.channel_id,
            "guild_id": guild_id,
            "self_deaf": true,
            "self_mute": false,
        },
    }))?;
    await!(ctx.to_sharder(map))?;

    Response::text("Joined the voice channel.")
}

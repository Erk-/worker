use serenity::constants::VoiceOpCode;
use super::prelude::*;

#[derive(Copy, Clone, Debug, Eq, Hash, PartialEq, PartialOrd, Ord)]
pub enum Join {
    AlreadyInChannel,
    Successful,
    UserNotInChannel,
}

impl Join {
    pub fn into_response(self) -> CommandResult {
        match self {
            Join::AlreadyInChannel | Join::Successful => {
                if self == Join::AlreadyInChannel {
                    trace!("Already in the user's voice channel");
                } else if self == Join::Successful {
                    trace!("Succesfully joined the user's voice channel");
                }

                Response::text("Joined the voice channel.")
            },
            Join::UserNotInChannel => {
                Response::err("You aren't in a voice channel.")
            },
        }
    }
}

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
    let join = match await!(join_ctx(&ctx)) {
        Ok(join) => join,
        Err(why) => {
            warn!("Err joining user voice state: {:?}", why);

            return Response::err(
                "There was an error joining the voice channel.",
            );
        },
    };

    let current = await!(ctx.current())?;

    if current.is_playing() {
        return join.into_response();
    }

    let song = match await!(ctx.queue_pop())? {
        Some(song) => song,
        None => return join.into_response(),
    };

    match await!(ctx.state.playback.play(ctx.msg.guild_id?.0, song.track)) {
        Ok(()) => {
            Response::text("Joined the voice channel!

Leaving off from your last queue.")
        },
        Err(why) => {
            warn!("Err playing next song: {:?}", why);

            Response::err("Joined the voice channel!")
        },
    }
}

pub async fn join_ctx(
    ctx: &Context,
) -> Result<Join> {
    let user_id = ctx.msg.author.id.0;
    let guild_id = ctx.msg.guild_id?.0;

    // Check if the user is in a guild.
    let user = match await!(ctx.state.cache.voice_state(guild_id, user_id))? {
        Some(user) => user,
        None => return Ok(Join::UserNotInChannel),
    };

    let bot_id = ctx.state.config.discord_user_id;

    // Check if the bot is already in the requested channel.
    if let Some(bot) = await!(ctx.state.cache.voice_state(guild_id, bot_id))? {
        trace!(
            "Bot's channel ID: {}; user's channel ID: {}",
            bot.channel_id,
            user.channel_id,
        );

        if bot.channel_id == user.channel_id {
            return Ok(Join::AlreadyInChannel);
        }
    }

    trace!("Serializing audio player for guild {}", guild_id);
    let map = serde_json::to_vec(&json!({
        "op": VoiceOpCode::SessionDescription.num(),
        "d": {
            "channel_id": user.channel_id,
            "guild_id": guild_id,
            "self_deaf": true,
            "self_mute": false,
        },
    }))?;
    trace!("Serialized audio player");
    trace!("Sending SessionDescription payload to sharder: {:?}", map);
    await!(ctx.to_sharder(map))?;
    trace!("Sent SessionDescription payload to sharder");

    Ok(Join::Successful)
}

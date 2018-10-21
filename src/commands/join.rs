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
    "Joins the voice channel."
}

pub fn names() -> &'static [&'static str] {
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

    let song = match await!(ctx.queue_pop()) {
        Ok(Some(song)) => song,
        Ok(None) | Err(_) => return join.into_response(),
    };

    match await!(ctx.state.playback.play(ctx.guild_id()?, song.track)) {
        Ok(true) => {
            Response::text("Joined the voice channel!

Leaving off from your last queue.")
        },
        Ok(false) => {
            Response::text("Joined the voice channel!")
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
    let guild_id = ctx.guild_id()?;

    trace!("Checking if G:{};U:{} is in a voice channel", guild_id, user_id);

    // Check if the user is in a voice channel.
    let user = match await!(ctx.state.cache.voice_state(guild_id, user_id))? {
        Some(user) => user,
        None => return Ok(Join::UserNotInChannel),
    };

    trace!("User voice state: {:?}", user);

    let bot_id = ctx.state.config.discord_user_id;

    trace!("Checking if bot is already in voice channel");
    // Check if the bot is already in the requested channel.
    if let Some(bot) = await!(ctx.state.cache.voice_state(guild_id, bot_id))? {
        trace!(
            "Bot's channel ID: {}; user's channel ID: {}",
            bot.channel_id,
            user.channel_id,
        );

        if bot.channel_id == user.channel_id {
            trace!("Bot is in user voice channel already");
            return Ok(Join::AlreadyInChannel);
        }
    }

    trace!("Bot is not in user voice channel");

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

    await!(ctx.state.redis.send(resp_array![
        "SET",
        format!("j:{}", guild_id),
        ctx.msg.channel_id.0 as usize
    ]).compat())?;

    Ok(Join::Successful)
}

use serenity::constants::VoiceOpCode;
use std::fmt::{Display, Formatter, Result as FmtResult};
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

pub struct JoinCommand;

impl JoinCommand {
    async fn _run(ctx: Context) -> CommandResult {
        let request = JoinRequest::pop(&ctx);

        match await!(Self::join(request)) {
            Ok(response) => Response::text(response.to_string()),
            Err(why) => {
                warn!("Err joining channel: {:?}", why);

                Response::err("There was an error joining the channel!")
            },
        }
    }

    pub async fn join(req: JoinRequest) -> Result<JoinResponse> {
        let user_id = req.ctx.msg.author.id.0;
        let guild_id = req.ctx.guild_id()?;

        trace!("Checking if G:{};U:{} is in a voice channel", guild_id, user_id);

        // Check if the user is in a voice channel.
        let user = match await!(req.ctx.state.cache.voice_state(guild_id, user_id))? {
            Some(user) => user,
            None => return Ok(JoinResponse::not_in_channel(req)),
        };

        trace!("User voice state: {:?}", user);

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
        await!(req.ctx.to_sharder(map))?;
        trace!("Sent SessionDescription payload to sharder");

        await!(req.ctx.state.cache.inner.set_join(
            guild_id,
            req.ctx.msg.channel_id.0,
        ))?;

        if !req.pop {
            return Ok(JoinResponse::successful_without_pop(req));
        }

        let current = await!(req.ctx.current())?;

        if current.is_playing() {
            return Ok(JoinResponse::already_playing(req));
        }

        let song = match await!(req.ctx.queue_pop()) {
            Ok(Some(song)) => song,
            Ok(None) => return Ok(JoinResponse::empty_pop(req)),
            Err(why) => return Ok(JoinResponse::error_popping(req, why)),
        };

        match await!(req.ctx.state.playback.play(req.ctx.guild_id()?, song.track)) {
            Ok(true) => Ok(JoinResponse::playing_next(req)),
            Ok(false) => Ok(JoinResponse::queued(req)),
            Err(why) => Ok(JoinResponse::error_playing(req, why)),
        }
    }

    pub async fn join_ctx(ctx: &Context) -> Result<Join> {
        let user_id = ctx.msg.author.id.0;
        let guild_id = ctx.guild_id()?;

        trace!(
            "Checking if G:{};U:{} is in a voice channel",
            guild_id,
            user_id
        );

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

        await!(ctx.state.cache.inner.set_join(
            guild_id,
            ctx.msg.channel_id.0
        ))?;

        Ok(Join::Successful)
    }
}

impl<'a> Command<'a> for JoinCommand {
    fn names(&self) -> &'static [&'static str] {
        &["connect", "j", "join"]
    }

    fn description(&self) -> &'static str {
        "Joins the voice channel."
    }

    fn run(&self, ctx: Context) -> RunFuture<'a> {
        RunFuture::new(Self::_run(ctx).boxed())
    }
}

pub struct JoinResponse<'a> {
    pub pop: PopStatus,
    pub request: JoinRequest<'a>,
    pub state: JoinState,
}

impl<'a> JoinResponse<'a> {
    fn already_in_channel(request: JoinRequest<'a>) -> Self {
        Self {
            pop: PopStatus::None,
            state: JoinState::AlreadyInChannel,
            request,
        }
    }

    fn already_playing(request: JoinRequest<'a>) -> Self {
        Self {
            pop: PopStatus::AlreadyPlaying,
            state: JoinState::Successful,
            request,
        }
    }

    fn empty_pop(request: JoinRequest<'a>) -> Self {
        Self {
            pop: PopStatus::None,
            state: JoinState::Successful,
            request,
        }
    }

    fn error_playing(request: JoinRequest<'a>, why: Error) -> Self {
        Self {
            pop: PopStatus::ErrorPlaying(why),
            state: JoinState::Successful,
            request,
        }
    }

    fn error_popping(request: JoinRequest<'a>, why: Error) -> Self {
        Self {
            pop: PopStatus::ErrorPopping(why),
            state: JoinState::Successful,
            request,
        }
    }

    fn not_in_channel(request: JoinRequest<'a>) -> Self {
        Self {
            pop: PopStatus::None,
            state: JoinState::UserNotInChannel,
            request
        }
    }

    fn playing_next(request: JoinRequest<'a>) -> Self {
        Self {
            pop: PopStatus::Playing,
            state: JoinState::Successful,
            request,
        }
    }

    fn queued(request: JoinRequest<'a>) -> Self {
        Self {
            pop: PopStatus::Queued,
            state: JoinState::Successful,
            request,
        }
    }

    fn successful_without_pop(request: JoinRequest<'a>) -> Self {
        Self {
            pop: PopStatus::None,
            state: JoinState::Successful,
            request,
        }
    }
}

impl<'a> Display for JoinResponse<'a> {
    fn fmt(&self, f: &mut Formatter) -> FmtResult {
        match self.state {
            JoinState::AlreadyInChannel => {
                return f.write_str("Joined the voice channel.");
            },
            JoinState::UserNotInChannel => {
                return f.write_str("It looks like you aren't in a voice channel.");
            },
            JoinState::Successful => {},
        }

        f.write_str(match self.pop {
            PopStatus::AlreadyPlaying => "A song is already playing.",
            PopStatus::ErrorPopping(_) => {
                "There was an error getting the next song in the queue."
            },
            PopStatus::ErrorPlaying(_) => {
                "There was an error playing the song."
            },
            PopStatus::None => "Didn't retrieve anything from the queue.",
            PopStatus::Playing => "Joined the voice channel and now playing the next song in the queue!",
            PopStatus::Queued => "Added the song to the queue!",
        })
    }
}

pub enum PopStatus {
    AlreadyPlaying,
    ErrorPopping(Error),
    ErrorPlaying(Error),
    None,
    Playing,
    Queued,
}

#[derive(Copy, Clone, Debug, Eq, Hash, PartialEq, PartialOrd, Ord)]
pub enum JoinState {
    AlreadyInChannel,
    Successful,
    UserNotInChannel,
}

impl JoinState {
    pub fn into_response(self) -> CommandResult {
        match self {
            JoinState::AlreadyInChannel | JoinState::Successful => {
                if self == JoinState::AlreadyInChannel {
                    trace!("Already in the user's voice channel");
                } else if self == JoinState::Successful {
                    trace!("Succesfully joined the user's voice channel");
                }

                Response::text("Joined the voice channel.")
            },
            JoinState::UserNotInChannel => {
                Response::err("You aren't in a voice channel.")
            },
        }
    }
}

pub struct JoinRequest<'a> {
    pub ctx: &'a Context,
    pub pop: bool,
}

impl<'a> JoinRequest<'a> {
    pub fn no_pop(ctx: &'a Context) -> Self {
        Self {
            pop: false,
            ctx,
        }
    }

    pub fn pop(ctx: &'a Context) -> Self {
        Self {
            pop: true,
            ctx,
        }
    }
}

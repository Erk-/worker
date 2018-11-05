use crate::{
    services::lavalink::LavalinkManager,
    cache::Cache,
};
use serenity::constants::VoiceOpCode;
use std::sync::Arc;
use super::prelude::*;

pub struct LeaveCommand;

impl LeaveCommand {
    async fn _run(ctx: Context) -> CommandResult {
        let gid = ctx.guild_id()?;
        let sid = ctx.shard_id;

        match await!(Self::leave(sid, gid, &ctx.state.playback, Arc::clone(&ctx.state.cache))) {
            Ok(()) => Response::text("Stopped playing music & left the voice channel."),
            Err(why) => {
                error!("Error stopping in guild {}: {:?}", gid, why,);

                Response::err("There was an error leaving the voice channel.")
            },
        }
    }

    pub async fn leave<'a>(
        shard_id: u64,
        guild_id: u64,
        playback: &'a Arc<LavalinkManager>,
        redis: Arc<Cache>,
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

        await!(redis.inner.sharder_msg(shard_id, map))?;
        await!(redis.inner.delete_join(guild_id))?;
        await!(redis.inner.delete_choices(guild_id))?;

        await!(playback.stop(guild_id))?;

        Ok(())
    }
}

impl<'a> Command<'a> for LeaveCommand {
    fn names(&self) -> &'static [&'static str] {
        &["disconnect", "l", "leave", "stop"]
    }

    fn description(&self) -> &'static str {
        "Leaves the voice channel."
    }

    fn run(&self, ctx: Context) -> RunFuture<'a> {
        RunFuture::new(Self::_run(ctx).boxed())
    }
}

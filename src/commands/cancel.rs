use redis_async::client::PairedConnection;
use std::sync::Arc;
use super::{
    choose,
    prelude::*,
};

pub const fn description() -> &'static str {
    "Cancels the current song selection"
}

pub fn names() -> &'static [&'static str] {
    &["cancel"]
}

pub async fn run(ctx: Context) -> CommandResult {
    cancel(&ctx.state.redis, ctx.msg.guild_id?.0)
}


pub(super) fn cancel(
    redis: &Arc<PairedConnection>,
    guild_id: u64,
) -> Result<Response> {
    choose::delete_selection(&redis, guild_id);

    Response::text("Selection cancelled!")
}

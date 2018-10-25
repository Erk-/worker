use redis_async::client::PairedConnection;
use std::sync::Arc;
use super::{
    choose,
    prelude::*,
};

pub struct CancelCommand;

impl CancelCommand {
    async fn _run(ctx: Context) -> CommandResult {
        Self::cancel(&ctx.state.redis, ctx.guild_id()?)
    }

    pub(super) fn cancel(redis: &Arc<PairedConnection>, guild_id: u64) -> Result<Response> {
        choose::ChooseCommand::delete_selection(&redis, guild_id);

        Response::text("Selection cancelled!")
    }
}

impl<'a> Command<'a> for CancelCommand {
    fn names(&self) -> &'static [&'static str] {
        &["cancel"]
    }

    fn description(&self) -> &'static str {
        "Cancels the current song selection."
    }

    fn run(&self, ctx: Context) -> RunFuture<'a> {
        RunFuture::new(Self::_run(ctx).boxed())
    }
}

use crate::cache::Cache;
use std::sync::Arc;
use super::{
    choose,
    prelude::*,
};

pub struct CancelCommand;

impl CancelCommand {
    async fn _run(ctx: Context) -> CommandResult {
        Self::cancel(Arc::clone(&ctx.state.cache), ctx.guild_id()?)
    }

    pub(super) fn cancel(redis: Arc<Cache>, guild_id: u64) -> Result<Response> {
        choose::ChooseCommand::delete_selection(redis, guild_id);

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

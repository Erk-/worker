use super::{choose, prelude::*};
use redis_async::client::PairedConnection;
use std::sync::Arc;

pub static COMMAND_INSTANCE: CancelCommand = CancelCommand;

pub struct CancelCommand;

impl<'a> Command<'a> for CancelCommand {
    fn names(&self) -> &'static [&'static str] {
        &["cancel"]
    }

    fn description(&self) -> &'static str {
        "Cancels the current song selection."
    }

    fn run(&self, ctx: Context) -> FutureObj<'a, CommandResult> {
        FutureObj::new(_run(ctx).boxed())
    }
}

async fn _run(ctx: Context) -> CommandResult {
    cancel(&ctx.state.redis, ctx.guild_id()?)
}

pub(super) fn cancel(redis: &Arc<PairedConnection>, guild_id: u64) -> Result<Response> {
    choose::ChooseCommand::delete_selection(&redis, guild_id);

    Response::text("Selection cancelled!")
}

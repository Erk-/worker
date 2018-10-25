use super::{
    play::Provider,
    prelude::*,
};

pub struct YouTubeCommand;

impl YouTubeCommand {
    async fn _run(ctx: Context) -> CommandResult {
        await!(super::play::PlayCommand::base(&ctx, Provider::YouTube))
    }
}

impl<'a> Command<'a> for YouTubeCommand {
    fn names(&self) -> &'static [&'static str] {
        &["youtube", "yt"]
    }

    fn description(&self) -> &'static str {
        "Searches YouTube for a song."
    }

    fn run(&self, ctx: Context) -> RunFuture<'a> {
        RunFuture::new(Self::_run(ctx).boxed())
    }
}

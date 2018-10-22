use super::{
    play::Provider,
    prelude::*,
};

pub static COMMAND_INSTANCE: SoundCloudCommand = SoundCloudCommand;

pub struct SoundCloudCommand;

impl SoundCloudCommand {
    async fn _run(ctx: Context) -> CommandResult {
        await!(super::play::PlayCommand::base(&ctx, Provider::SoundCloud))
    }
}

impl<'a> Command<'a> for SoundCloudCommand {
    fn names(&self) -> &'static [&'static str] {
        &["soundcloud", "sc"]
    }

    fn description(&self) -> &'static str {
        "Searches SoundCloud for a song."
    }

    fn run(&self, ctx: Context) -> RunFuture<'a> {
        RunFuture::new(Self::_run(ctx).boxed())
    }
}

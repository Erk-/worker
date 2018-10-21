use super::prelude::*;

pub static COMMAND_INSTANCE: ProvidersCommand = ProvidersCommand;

pub struct ProvidersCommand;

impl ProvidersCommand {
    async fn _run() -> CommandResult {
        Response::text(
            "Available music providers: youtube, soundcloud, bandcamp, vimeo, twitch, beam.pro, \
             http",
        )
    }
}

impl<'a> Command<'a> for ProvidersCommand {
    fn names(&self) -> &'static [&'static str] {
        &["providers"]
    }

    fn description(&self) -> &'static str {
        "Shows information about audio providers."
    }

    fn run(&self, _: Context) -> RunFuture<'a> {
        RunFuture::new(Self::_run().boxed())
    }
}

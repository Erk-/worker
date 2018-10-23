use super::prelude::*;

pub static COMMAND_INSTANCE: PingCommand = PingCommand;

pub struct PingCommand;

impl PingCommand {
    async fn _run() -> CommandResult {
        Response::text("Pong!")
    }
}

impl<'a> Command<'a> for PingCommand {
    fn names(&self) -> &'static [&'static str] {
        &["ping"]
    }

    fn description(&self) -> &'static str {
        "Ping pong!"
    }

    fn run(&self, _: Context) -> RunFuture<'a> {
        RunFuture::new(Self::_run().boxed())
    }
}

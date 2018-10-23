use super::prelude::*;

pub static COMMAND_INSTANCE: HelpCommand = HelpCommand;

pub struct HelpCommand;

impl HelpCommand {
    async fn _run() -> CommandResult {
        Response::text("A list of our commands is available here: <https://dabbot.org/commands>")
    }
}

impl<'a> Command<'a> for HelpCommand {
    fn description(&self) -> &'static str {
        "Links to our list of commands."
    }

    fn names(&self) -> &'static [&'static str] {
        &["help", "h"]
    }

    fn run(&self, _: Context) -> RunFuture<'a> {
        RunFuture::new(Self::_run().boxed())
    }
}

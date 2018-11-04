use super::prelude::*;

pub struct RepeatCommand;

impl RepeatCommand {
    async fn _run() -> CommandResult {
        // TODO(Proximyst): Implement loop command
        Response::err("This command has not been implemented yet!")
    }
}

impl<'a> Command<'a> for RepeatCommand {
    fn names(&self) -> &'static [&'static str] {
        &["loop", "repeat"]
    }

    fn description(&self) -> &'static str {
        "Loops a queue or a single song, depending on the setting."
    }

    fn run(&self, _: Context) -> RunFuture<'a> {
        RunFuture::new(Self::_run().boxed())
    }
}

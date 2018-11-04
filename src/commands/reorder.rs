use super::prelude::*;

pub struct RepeatCommand;

impl RepeatCommand {
    async fn _run() -> CommandResult {
        // TODO(Proximyst,zeyla): Implement reorder command
        Response::err("This command has not been implemented yet and is being worked on!")
    }
}

impl<'a> Command<'a> for RepeatCommand {
    fn names(&self) -> &'static [&'static str] {
        &["reorder"]
    }

    fn description(&self) -> &'static str {
        "Reorders the position of a song in the queue."
    }

    fn run(&self, _: Context) -> RunFuture<'a> {
        RunFuture::new(Self::_run().boxed())
    }
}

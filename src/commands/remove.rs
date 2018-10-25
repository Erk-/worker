use super::prelude::*;

pub struct RemoveCommand;

impl RemoveCommand {
    async fn _run() -> CommandResult {
        // TODO(Proximyst): Implement remove command
        Response::err("This command has not been implemented yet!")
    }
}

impl<'a> Command<'a> for RemoveCommand {
    fn names(&self) -> &'static [&'static str] {
        &["remove", "rm"]
    }

    fn description(&self) -> &'static str {
        "Removes a song from the queue."
    }

    fn run(&self, _: Context) -> RunFuture<'a> {
        RunFuture::new(Self::_run().boxed())
    }
}

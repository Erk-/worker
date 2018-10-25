use super::prelude::*;

pub struct InviteCommand;

impl InviteCommand {
    async fn _run() -> CommandResult {
        Response::text("Invite dabBot: <https://dabbot.org/invite>")
    }
}

impl<'a> Command<'a> for InviteCommand {
    fn names(&self) -> &'static [&'static str] {
        &["invite", "inv"]
    }

    fn description(&self) -> &'static str {
        "Displays a link to invite the bot."
    }

    fn run(&self, _: Context) -> RunFuture<'a> {
        RunFuture::new(Self::_run().boxed())
    }
}

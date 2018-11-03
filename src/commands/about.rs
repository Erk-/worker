use super::prelude::*;

pub struct AboutCommand;

impl AboutCommand {
    async fn _run(ctx: Context) -> CommandResult {
        Response::text(format!(
            "**dabBot**
Command prefix: `{prefix}`
Invite me to your server: <https://dabbot.org/invite>
Support server: <https://dabbot.org/support>
Github: <https://github.com/dabbotorg>",
            prefix = ctx.prefix()?
        ))
    }
}

impl<'a> Command<'a> for AboutCommand {
    fn names(&self) -> &'static [&'static str] {
        &["about", "info"]
    }

    fn description(&self) -> &'static str {
        "Displays information about the bot."
    }

    fn run(&self, ctx: Context) -> RunFuture<'a> {
        RunFuture::new(Self::_run(ctx).boxed())
    }
}

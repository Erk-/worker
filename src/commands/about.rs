use super::prelude::*;

pub static COMMAND_INSTANCE: AboutCommand = AboutCommand;

pub struct AboutCommand;

impl<'a> Command<'a> for AboutCommand {
    fn names(&self) -> &'static [&'static str] {
        &["about", "info"]
    }

    fn description(&self) -> &'static str {
        "Displays information about the bot."
    }

    fn run(&self, ctx: Context) -> FutureObj<'a, CommandResult> {
        FutureObj::new(_run(ctx).boxed())
    }
}

async fn _run(ctx: Context) -> CommandResult {
    let prefix = ctx.state.config.bot_prefixes.first()?;

    Response::text(format!("
**dabBot**
Command prefix: `{prefix}`
Invite me to your server: <https://dabbot.org/invite>
Support server: <https://dabbot.org/support>
Github: <https://github.com/dabbotorg>
", prefix=prefix))
}

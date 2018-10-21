use super::prelude::*;

pub const fn description() -> &'static str {
    "Displays information about the bot."
}

pub fn names() -> &'static [&'static str] {
    &["about", "info"]
}

pub async fn run(ctx: Context) -> CommandResult {
    let prefix = ctx.state.config.bot_prefixes.first()?;

    Response::text(format!("
**dabBot**
Command prefix: `{prefix}`
Invite me to your server: <https://dabbot.org/invite>
Support server: <https://dabbot.org/support>
Github: <https://github.com/dabbotorg>
", prefix=prefix))
}

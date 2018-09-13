use super::prelude::*;

pub const fn description() -> &'static str {
    "Information about the bot"
}

pub const fn names() -> &'static [&'static str] {
    &["about"]
}

pub async fn run(_: Context) -> CommandResult {
    Response::text(r#"
**dabBot**
Command prefix: `!!!`
Invite me to your server: https://dabbot.org/invite
Support server: https://dabbot.org/support
Github: https://github.com/dabbotorg
"#)
}

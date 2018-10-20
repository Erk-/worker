use super::prelude::*;

pub const fn description() -> &'static str {
    "A link to invite the bot"
}

pub fn names() -> &'static [&'static str] {
    &["invite", "inv"]
}

pub async fn run(_: Context) -> CommandResult {
    Response::text("Invite dabBot: <https://dabbot.org/invite>")
}

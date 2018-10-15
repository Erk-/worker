use super::prelude::*;

pub const fn description() -> &'static str {
    "A list of commands."
}

pub const fn names() -> &'static [&'static str] {
    &["help"]
}

pub async fn run(_: Context) -> CommandResult {
    Response::text(
        "A list of our commands is available here: <https://dabbot.org/commands>",
    )
}

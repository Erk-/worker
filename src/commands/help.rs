use super::prelude::*;

pub const fn description() -> &'static str {
    "Links to our list of commands."
}

pub fn names() -> &'static [&'static str] {
    &["help", "h"]
}

pub async fn run(_: Context) -> CommandResult {
    Response::text(
        "A list of our commands is available here: <https://dabbot.org/commands>",
    )
}

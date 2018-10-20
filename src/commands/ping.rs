use super::prelude::*;

pub const fn description() -> &'static str {
    "Ping pong"
}

pub fn names() -> &'static [&'static str] {
    &["ping"]
}

pub async fn run(_: Context) -> CommandResult {
    Response::text("Pong!")
}

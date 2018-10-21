use super::prelude::*;

pub const fn description() -> &'static str {
    "Removes a song from the queue."
}

pub fn names() -> &'static [&'static str] {
    &["remove", "rm"]
}

pub async fn run(_: Context) -> CommandResult {
    // TODO(Proximyst): Implement remove command
    Response::text("TBA")
}

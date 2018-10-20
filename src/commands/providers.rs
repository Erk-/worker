use super::prelude::*;

pub const fn description() -> &'static str {
    "Information about audio providers."
}

pub const fn names() -> &'static [&'static str] {
    &["providers"]
}

pub async fn run(_: Context) -> CommandResult {
    Response::text("Available music providers: youtube, soundcloud, bandcamp, vimeo, twitch, beam.pro, http")
}

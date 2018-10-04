use super::{
    play::Provider,
    prelude::*,
};

pub const fn description() -> &'static str {
    "Search SoundCloud for a song."
}

pub const fn names() -> &'static [&'static str] {
    &["soundcloud", "sc"]
}

pub async fn run(ctx: Context) -> CommandResult {
    await!(super::play::base(&ctx, Provider::SoundCloud))
}

use super::{
    play::Provider,
    prelude::*,
};

pub const fn description() -> &'static str {
    "Searches YouTube for a song."
}

pub const fn names() -> &'static [&'static str] {
    &["youtube", "yt"]
}

pub async fn run(ctx: Context) -> CommandResult {
    await!(super::play::base(&ctx, Provider::YouTube))
}

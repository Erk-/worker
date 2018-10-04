use crate::utils;
use lavalink::rest::Load;
use std::fmt::{Display, Formatter, Result as FmtResult, Write as _};
use super::prelude::*;

#[derive(Copy, Clone, Debug, Eq, Hash, PartialEq, PartialOrd, Ord)]
pub enum Provider {
    SoundCloud,
    YouTube,
}

impl Provider {
    pub fn prefix(self) -> &'static str {
        use self::Provider::*;

        match self {
            SoundCloud => "scsearch",
            YouTube => "ytsearch"
        }
    }
}

impl Display for Provider {
    fn fmt(&self, f: &mut Formatter) -> FmtResult {
        f.write_str(self.prefix())?;
        f.write_char(':')?;

        Ok(())
    }
}

impl From<Provider> for String {
    fn from(provider: Provider) -> String {
        provider.prefix().to_owned()
    }
}

pub const fn description() -> &'static str {
    "plays a song"
}

pub const fn names() -> &'static [&'static str] {
    &["play", "p", "search"]
}

pub async fn run(ctx: Context) -> CommandResult {
    await!(base(&ctx, Provider::YouTube))
}

pub async fn base(
    ctx: &Context,
    provider: Provider,
) -> CommandResult {
    if ctx.args.len() < 1 {
        return Response::err("You need to say the link to the song or the name of what you want to play");
    }

    let query = ctx.args.join(" ");

    let mut tracks = match await!(search(&ctx, &query, provider)) {
        Ok(tracks) => tracks,
        Err(why) => {
            warn!(
                "Err searching tracks for query '{}' in provider {}: {:?}",
                query,
                provider.to_string(),
                why,
            );

            return Response::err("There was an error searching for that.");
        },
    };

    let song = tracks.tracks.remove(0);

    await!(super::join::join_ctx(&ctx))?;

    match await!(ctx.state.playback.play(ctx.msg.guild_id?.0, song.track)) {
        Ok(()) => {
            Response::text(format!(
                "Now playing **{}** by **{}** `[{}]`",
                song.info.title,
                song.info.author,
                utils::track_length_readable(song.info.length as u64),
            ))
        },
        Err(why) => {
            warn!("Err playing song: {:?}", why);

            Response::err("There was an error playing the song.")
        },
    }
}

pub async fn search<'a>(
    ctx: &'a Context,
    query: impl AsRef<str> + 'a,
    provider: Provider,
) -> Result<Load> {
    await!(_search(ctx, query.as_ref(), provider))
}

async fn _search<'a>(
    ctx: &'a Context,
    query: &'a str,
    provider: Provider,
) -> Result<Load> {
    let term = format!("{}{}", provider, query);

    await!(ctx.state.playback.search(term))
}

use crate::utils;
use lavalink::rest::{Load, LoadType};
use serenity::utils::MessageBuilder;
use std::fmt::{Display, Formatter, Result as FmtResult, Write as _};
use super::{
    join::Join,
    prelude::*,
};

#[derive(Copy, Clone, Debug, Eq, Hash, PartialEq, PartialOrd, Ord)]
pub enum Provider {
    SoundCloud,
    URL,
    YouTube,
}

impl Provider {
    pub fn prefix(self) -> &'static str {
        use self::Provider::*;

        match self {
            SoundCloud => "scsearch",
            URL => "",
            YouTube => "ytsearch"
        }
    }
}

impl Display for Provider {
    fn fmt(&self, f: &mut Formatter) -> FmtResult {
        if *self == Provider::URL {
            return Ok(());
        }

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

pub fn names() -> &'static [&'static str] {
    &["play", "p", "search"]
}

pub async fn run(ctx: Context) -> CommandResult {
    await!(base(&ctx, Provider::YouTube))
}

pub async fn base(
    ctx: &Context,
    mut provider: Provider,
) -> CommandResult {
    if ctx.args.len() < 1 {
        return Response::err("You need to say the link to the song or the name of what you want to play");
    }

    let query = ctx.args.join(" ");

    if query.starts_with("https://") || query.starts_with("http://") {
        provider = Provider::URL;
    }

    let load = match await!(search(&ctx, &query, provider)) {
        Ok(load) => load,
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

    info!("load: {:?}", load.load_type);

    match load.load_type {
        LoadType::LoadFailed => {
            return Response::err("There was an error searching for that!");
        },
        LoadType::NoMatches => {
            return Response::err("It looks like there aren't any results for that!");
        },
        LoadType::PlaylistLoaded => {
            await!(handle_playlist(&ctx, load))
        },
        LoadType::SearchResult | LoadType::TrackLoaded => {
            await!(handle_search(&ctx, load))
        },
    }
}

async fn handle_search(ctx: &Context, mut load: Load) -> CommandResult {
    let guild_id = ctx.guild_id()?;

    if load.tracks.is_empty() {
        return Response::text("It looks like there aren't any results for that!");
    }

    if load.tracks.len() == 1 {
        return await!(super::choose::select(&ctx, load.tracks.remove(0).track));
    }

    load.tracks.truncate(5);

    let mut blobs = load.tracks
        .iter()
        .map(|t| t.track.clone())
        .rev()
        .collect::<Vec<_>>();

    debug!("Deleting existing choose for guild {}", guild_id);
    ctx.state.redis.send_and_forget(resp_array![
        "DEL",
        format!("c:{}", guild_id)
    ]);
    debug!("Deleted existing choose for guild {}", guild_id);
    debug!("Setting choose for guild {}", guild_id);
    ctx.state.redis.send_and_forget(resp_array![
        "LPUSH",
        format!("c:{}", guild_id)
    ].append(&mut blobs));
    debug!("Set choose for guild {}", guild_id);

    let mut msg = MessageBuilder::new();

    for (idx, track) in load.tracks.iter().enumerate() {
        write!(msg.0, "`{}` ", idx + 1)?;
        msg.push_safe(&track.info.title);
        msg.0.push_str(" by ");
        msg.push_safe(&track.info.author);
        write!(msg.0, " `[{}]`", utils::track_length_readable(track.info.length as u64))?;
        msg.0.push('\n');
    }

    let prefix = ctx.state.config.bot_prefixes.first()?;

    msg.0.push_str("\n**To choose**, use `");
    msg.push_safe(&prefix);
    msg.0.push_str("choose <number>`
Example: `");
    msg.push_safe(&prefix);
    msg.0.push_str("choose 2` would pick the second option.
**To cancel**, use `");
    msg.push_safe(&prefix);
    msg.0.push_str("cancel`.");

    Response::text(msg.build())
}

async fn handle_playlist(ctx: &Context, load: Load) -> CommandResult {
    let guild_id = ctx.guild_id()?;

    if load.tracks.is_empty() {
        return Response::text("It looks like that playlist is empty!");
    }

    let tracks = load.tracks
        .iter()
        .map(|t| t.track.clone())
        .collect::<Vec<_>>();

    let track_count = tracks.len();

    await!(ctx.state.queue.add_multiple(guild_id, tracks))?;

    let join = await!(super::join::join_ctx(&ctx))?;

    let mut content = format!("Loaded {} songs from the playlist", track_count);

    if let Some(name) = load.playlist_info.name {
        write!(content, " **{}**", name)?;
    }

    content.push('!');

    match join {
        Join::UserNotInChannel => {
            return Response::text(content);
        },
        Join::AlreadyInChannel | Join::Successful => {},
    }

    let current = await!(ctx.current())?;

    if current.is_playing() {
        return Response::text(content);
    }

    let song = match await!(ctx.queue_pop()) {
        Ok(Some(song)) => song,
        Ok(None) | Err(_) => return Response::text(content),
    };

    match await!(ctx.state.playback.play(ctx.guild_id()?, song.track)) {
        Ok(true) => {
            content.push_str("\n\nJoined the voice channel and started playing the next song!");

            Response::text(content)
        },
        Ok(false) => {
            content.push_str("\n\nJoined the voice channel and added the songs to the queue.");

            Response::text(content)
        }
        Err(why) => {
            warn!("Err playing next song: {:?}", why);

            Response::text(content)
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

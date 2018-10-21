use super::prelude::*;

pub const fn description() -> &'static str {
    "Discord.FM playlists"
}

pub fn names() -> &'static [&'static str] {
    &["dfm", "discordfm"]
}

pub async fn run(ctx: Context) -> CommandResult {
    if ctx.args.is_empty() {
        let prefix = ctx.state.config.bot_prefixes.first()?;

        Response::text(format!("Uses a song playlist from the now defunct Discord.FM
Usage: `{}dfm <library>`

**Available libraries:**
{}", prefix, ctx.state.discord_fm.list))
    } else {
        let guild_id = ctx.guild_id()?;

        let query = ctx.args.join(" ").to_lowercase();

        let library = match ctx.state.discord_fm.libraries.get(&query) {
            Some(library) => library,
            None => {
                let prefix = ctx.state.config.bot_prefixes.first()?;

                return Response::text(format!(
                    "Invalid library! Use `{}dfm` to see usage & libraries.",
                    prefix,
                ));
            },
        };

        let amount = library.items.len();
        let tracks = library.items.iter().map(|item| item.track.clone()).collect();

        debug!("Adding {} to queue for guild: {}", amount, guild_id);
        let songs = await!(ctx.state.queue.add_multiple(guild_id, tracks))?;
        let song_count = songs.len();

        Response::text(format!(
            "Added {} ({} songs) to the song queue.",
            library.name,
            song_count,
        ))
    }
}

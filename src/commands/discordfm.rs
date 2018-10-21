use super::prelude::*;

pub static COMMAND_INSTANCE: DfmCommand = DfmCommand;

pub struct DfmCommand;

impl DfmCommand {
    async fn _run(ctx: Context) -> CommandResult {
        if ctx.args.is_empty() {
            let prefix = ctx.state.config.bot_prefixes.first()?;

            return Response::text(format!(
                "Uses a song playlist from the now defunct Discord.FM
Usage: `{}dfm <library>`

**Available libraries:**
{}",
                prefix, ctx.state.discord_fm.list
            ));
        }
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
        let tracks = library
            .items
            .iter()
            .map(|item| item.track.clone())
            .collect();

        debug!("Adding {} to queue for guild: {}", amount, guild_id);
        let songs = await!(ctx.state.queue.add_multiple(guild_id, tracks))?;
        let song_count = songs.len();

        Response::text(format!(
            "Added {} ({} songs) to the song queue.",
            library.name, song_count,
        ))
    }
}

impl<'a> Command<'a> for DfmCommand {
    fn names(&self) -> &'static [&'static str] {
        &["dfm", "discordfm", "discord.fm", "d.fm"]
    }

    fn description(&self) -> &'static str {
        "Plays a Discord.FM playlist."
    }

    fn run(&self, ctx: Context) -> RunFuture<'a> {
        RunFuture::new(Self::_run(ctx).boxed())
    }
}

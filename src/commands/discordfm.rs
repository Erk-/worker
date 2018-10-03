use super::prelude::*;

pub const fn description() -> &'static str {
    "Discord.FM playlists"
}

pub const fn names() -> &'static [&'static str] {
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
        let guild_id = ctx.msg.guild_id?.0;

        let query = ctx.args.join(" ");

        let list = match ctx.state.discord_fm.lists.get(&query) {
            Some(list) => list,
            None => {
                let prefix = ctx.state.config.bot_prefixes.first()?;

                return Response::text(format!(
                    "Invalid library! Use `{}dfm` to see usage & libraries.",
                    prefix,
                ));
            },
        };

        for item in list {
            debug!("Adding to queue for guild {}: {}", guild_id, item.track);

            await!(ctx.state.queue.add(guild_id, item.track.clone()))?;
        }

        Response::text("Added the library to the queue.")
    }
}

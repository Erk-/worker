use dump::DumpRequester;
use super::prelude::*;

pub struct DumpCommand;

impl DumpCommand {
    async fn _run(ctx: Context) -> CommandResult {
        let guild_id = ctx.guild_id()?;

        let queue = match await!(ctx.state.queue.get(guild_id)) {
            Ok(queue) => queue,
            Err(why) => {
                warn!(
                    "There was an error getting the queue for guild {}: {:?}",
                    guild_id, why,
                );

                return Response::err("There was an error getting the queue.");
            },
        };

        let tracks = queue
            .into_iter()
            .map(|item| item.song_track)
            .collect::<Vec<_>>();

        trace!("Serializing dump tracks");
        let body = serde_json::to_vec_pretty(&tracks)?;
        trace!("Serialized dump tracks");

        let dump = await!(
            ctx.state
                .http
                .dump(
                    &ctx.state.config.dump.post_address,
                    &ctx.state.config.dump.authorization,
                    body,
                )
                .compat()
        )?;

        Response::text(format!(
            "A dump of your song queue was created! Link: https://{addr}/{uuid}

Load this playlist with `{prefix}load https://{addr}/{uuid}`",
            addr = ctx.state.config.dump.display_address,
            prefix = ctx.state.config.bot_prefixes.first()?,
            uuid = dump.uuid,
        ))
    }
}

impl<'a> Command<'a> for DumpCommand {
    fn description(&self) -> &'static str {
        "Dumps the current queue to be played with the load command."
    }

    fn names(&self) -> &'static [&'static str] {
        &["dump", "d"]
    }

    fn run(&self, ctx: Context) -> RunFuture<'a> {
        RunFuture::new(Self::_run(ctx).boxed())
    }
}

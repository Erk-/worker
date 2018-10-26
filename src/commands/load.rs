use dump::DumpRequester;
use std::fmt::Write as _;
use super::{
    join::{JoinCommand, JoinRequest},
    prelude::*,
};

pub struct LoadCommand;

impl LoadCommand {
    async fn _new(ctx: Context) -> CommandResult {
        let guild_id = ctx.guild_id()?;

        let query = match ctx.args.first() {
            Some(query) => query,
            None => {
                return Response::text("You need to say the URL to the queue dump!");
            },
        };

        let uuid = {
            if query.starts_with("http") {
                let slash = match query.rfind('/') {
                    Some(slash) => slash,
                    None => {
                        return Response::text("That doesn't look like a valid load URL.");
                    },
                };

                let slice = &query[slash + 1..];

                if slice.is_empty() {
                    return Response::text("That doesn't look like a valid load URL.");
                }

                slice
            } else {
                &query
            }
        };

        let request = ctx.state.http.retrieve(
            &ctx.state.config.dump.display_address,
            &ctx.state.config.dump.authorization,
            uuid,
        );

        let body = match await!(request.compat()) {
            Ok(body) => body,
            Err(why) => {
                warn!("Err getting dump for {}: {:?}", uuid, why);

                return Response::text("There was an error getting the playlist.");
            },
        };

        let tracks = serde_json::from_slice::<Vec<String>>(&body)?;
        let track_count = tracks.len();

        await!(ctx.state.queue.add_multiple(guild_id, tracks))?;

        let mut content = format!("Loaded {} songs from the playlist!", track_count);

        match await!(JoinCommand::join(JoinRequest::pop(&ctx))) {
            Ok(resp) => {
                write!(content, "\n\n{}", resp);

                Response::text(content)
            },
            Err(why) => {
                warn!("Err joining voice channel: {:?}", why);

                Response::text(content)
            },
        }
    }
}

impl<'a> Command<'a> for LoadCommand {
    fn names(&self) -> &'static [&'static str] {
        &["load"]
    }

    fn description(&self) -> &'static str {
        "Loads a queue of songs from the dump command."
    }

    fn run(&self, ctx: Context) -> RunFuture<'a> {
        RunFuture::new(Self::_new(ctx).boxed())
    }
}

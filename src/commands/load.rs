use dump::DumpRequester;
use super::{
    join::Join,
    prelude::*,
};

pub static COMMAND_INSTANCE: LoadCommand = LoadCommand;

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

        for track in tracks.into_iter() {
            await!(ctx.state.queue.add(guild_id, track))?;
        }

        let join = await!(super::join::JoinCommand::join_ctx(&ctx))?;

        let mut content = format!("Loaded {} songs from the playlist!", track_count);

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
            },
            Err(why) => {
                warn!("Err playing next song: {:?}", why);

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

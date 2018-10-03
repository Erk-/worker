use dump::DumpRequester;
use super::prelude::*;

pub const fn description() -> &'static str {
    "Loads a queue of songs from the dump command."
}

pub const fn names() -> &'static [&'static str] {
    &["load"]
}

pub async fn run(ctx: Context) -> CommandResult {
    let guild_id = ctx.msg.guild_id?.0;

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
        &ctx.state.config.dump.address,
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

    Response::text(format!("Loaded {} songs from the playlist!", track_count))
}

use super::prelude::*;

pub const fn description() -> &'static str {
    "Streams a radio or displays a list of them all."
}

pub fn names() -> &'static [&'static str] {
    &["radio", "r"]
}

pub async fn run(ctx: Context) -> CommandResult {
    if ctx.args.is_empty() {
        let prefix = ctx.state.config.bot_prefixes.first()?;

        Response::text(format!(
            "View the radios here: <https://dabbot.org/radios>

To play a radio, use \
             `{prefix}radio <name here>`.

For example, use `{prefix}radio Radio Here`",
            prefix = prefix
        ))
    } else {
        let query = ctx.args.join(" ");

        let radio = match ctx.state.radios.get(&query) {
            Some(radio) => radio,
            None => {
                return Response::text(format!(
                    "Invalid station! For usage & stations, use `{}radio`",
                    ctx.state.config.bot_prefixes.first()?,
                ));
            },
        };

        let results = match await!(ctx.state.playback.search(radio.url.clone())) {
            Ok(tracks) => tracks,
            Err(why) => {
                warn!("Err searching tracks for query '{}': {:?}", query, why);

                return Response::err("There was an error searching for that.");
            },
        };

        let radio = results.tracks.first()?;

        await!(super::join::join_ctx(&ctx))?;

        match await!(
            ctx.state
                .playback
                .play(ctx.guild_id()?, radio.track.clone())
        ) {
            Ok(true) => {
                Response::text(format!(
                    "Now playing **{}** by **{}**.",
                    radio.info.title, radio.info.author,
                ))
            },
            Ok(false) => {
                Response::text(format!(
                    "Added **{}** by **{}** to the queue.",
                    radio.info.title, radio.info.author,
                ))
            },
            Err(why) => {
                warn!("Err playing radio: {:?}", why);

                Response::err("There was an error playing the radio.")
            },
        }
    }
}

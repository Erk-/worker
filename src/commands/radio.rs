use super::prelude::*;

pub const fn description() -> &'static str {
    "Information about the bot"
}

pub const fn names() -> &'static [&'static str] {
    &["about"]
}

pub async fn run(ctx: Context) -> CommandResult {
    if ctx.args.is_empty() {
        Response::text("radios here")
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

        match await!(ctx.state.playback.play(ctx.msg.guild_id?.0, radio.track.clone())) {
            Ok(()) => {
                Response::text(format!(
                    "Now playing **{}** by **{}**",
                    radio.info.title,
                    radio.info.author,
                ))
            },
            Err(why) => {
                warn!("Err playing radio: {:?}", why);

                Response::err("There was an error playing the radio.")
            },
        }
    }
}

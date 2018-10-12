use crate::utils;
use lavalink::decoder;
use redis_async::client::PairedConnection;
use serenity::utils::MessageBuilder;
use std::sync::Arc;
use super::prelude::*;

pub const fn description() -> &'static str {
    "Chooses a song from a selection."
}

pub const fn names() -> &'static [&'static str] {
    &["choose", "c", "chose"]
}

pub async fn run(ctx: Context) -> CommandResult {
    let guild_id = ctx.msg.guild_id?.0;

    let cmd = resp_array![
        "LRANGE",
        format!("c:{}", guild_id),
        0,
        5
    ];
    let mut selection: Vec<String> = match await!(ctx.state.redis.send(cmd).compat()) {
        Ok(selection) => selection,
        Err(why) => {
            warn!("Err getting selection: {:?}", why);

            return Response::err("There was an error getting the choices.");
        },
    };

    if selection.is_empty() {
        let prefix = ctx.state.config.bot_prefixes.first()?;

        let mut msg = MessageBuilder::new();
        msg.0.push_str("There's no selection active in this guild - are you sure you ran `");
        msg.push_safe(&prefix);
        msg.0.push_str("play`?

To play a song...
* Join a voice channel
* Use `");
        msg.push_safe(&prefix);
        msg.0.push_str("play <song name/link>`
* Choose one of the song options with `");
        msg.push_safe(&prefix);
        msg.0.push_str("choose <song number>`");

        return Response::text(msg.build());
    }

    if let Some(arg) = ctx.args.first() {
        let num = match arg.parse::<usize>() {
            Ok(num @ 1 ... 5) => num - 1,
            Ok(_) | Err(_) => {
                return Response::err(
                    "You must provide a number between 1 and 5!",
                );
            },
        };

        await!(super::join::join_ctx(&ctx))?;

        let track = selection.remove(num);
        let song = decoder::decode_track_base64(&track)?;

        delete_selection(&ctx.state.redis, guild_id);

        match await!(ctx.state.playback.play(guild_id, track)) {
            Ok(()) => {
                Response::text(format!(
                    "Now playing **{}** by **{}** `[{}]`",
                    song.title,
                    song.author,
                    utils::track_length_readable(song.length as u64),
                ))
            },
            Err(why) => {
                warn!("Err playing song: {:?}", why);

                Response::err("There was an error playing the song.")
            },
        }
    } else {
        super::cancel::cancel(&ctx.state.redis, guild_id)
    }
}

pub(super) fn delete_selection(redis: &Arc<PairedConnection>, guild_id: u64) {
    redis.send_and_forget(resp_array![
        "DEL",
        format!("c:{}", guild_id)
    ]);
}

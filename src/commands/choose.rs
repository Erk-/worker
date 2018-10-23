use crate::utils;
use lavalink::decoder;
use redis_async::client::PairedConnection;
use serenity::utils::MessageBuilder;
use std::sync::Arc;
use super::prelude::*;

pub static COMMAND_INSTANCE: ChooseCommand = ChooseCommand;

pub struct ChooseCommand;

impl ChooseCommand {
    async fn _run(ctx: Context) -> CommandResult {
        let guild_id = ctx.guild_id()?;

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
            // push_safe is used to filter @everyone and other pings
            msg.push_safe(format!("There's no selection active in this guild - are you sure you ran `{prefix}play`?

To play a song...
* Join a voice channel
* Use `{prefix}play <song name/link>`
* Choose one of the song options with `{prefix}choose <option>`", prefix=prefix));

            return Response::text(msg.build());
        }

        if let Some(arg) = ctx.args.first() {
            let num = match arg.parse::<usize>() {
                Ok(num @ 1 ... 5) => num - 1,
                _ => {
                    return Response::err(
                        "You must provide a number between 1 and 5!",
                    );
                },
            };

            await!(Self::select(&ctx, selection.remove(num)))
        } else {
            super::cancel::CancelCommand::cancel(&ctx.state.redis, guild_id)
        }
    }

    pub(super) fn delete_selection(redis: &Arc<PairedConnection>, guild_id: u64) {
        redis.send_and_forget(resp_array![
            "DEL",
            format!("c:{}", guild_id)
        ]);
    }

    pub(super) async fn select(ctx: &Context, track: String) -> CommandResult {
        let guild_id = ctx.guild_id()?;

        await!(super::join::JoinCommand::join_ctx(&ctx))?;

        let song = decoder::decode_track_base64(&track)?;

        Self::delete_selection(&ctx.state.redis, guild_id);

        match await!(ctx.state.playback.play(guild_id, track)) {
            Ok(true) => {
                Response::text(format!(
                    "Now playing **{}** by **{}** `[{}]`",
                    song.title,
                    song.author,
                    utils::track_length_readable(song.length as u64),
                ))
            },
            Ok(false) => {
                Response::text(format!(
                    "Added **{}** by **{}** `[{}]` to the queue.",
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
    }
}

impl<'a> Command<'a> for ChooseCommand {
    fn names(&self) -> &'static [&'static str] {
        &["choose", "c", "chose"]
    }

    fn description(&self) -> &'static str {
        "Chooses a song from a selection screen."
    }

    fn run(&self, ctx: Context) -> RunFuture<'a> {
        RunFuture::new(Self::_run(ctx).boxed())
    }
}

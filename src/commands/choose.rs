use crate::utils;
use lavalink::decoder;
use crate::cache::Cache;
use futures::{
    future,
    compat::Compat,
};
use serenity::utils::MessageBuilder;
use std::sync::Arc;
use super::{
    join::{JoinCommand, JoinRequest},
    prelude::*,
};

pub struct ChooseCommand;

impl ChooseCommand {
    async fn _run(ctx: Context) -> CommandResult {
        let guild_id = ctx.guild_id()?;

        let mut selection: Vec<String> = match await!(ctx.state.cache.inner.get_choices_ranged(guild_id, 0, 5)) {
            Ok(selection) => selection,
            Err(why) => {
                warn!("Err getting selection: {:?}", why);

                return Response::err("There was an error getting the choices.");
            },
        };

        if selection.is_empty() {
            let prefix = ctx.prefix()?;

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
            super::cancel::CancelCommand::cancel(Arc::clone(&ctx.state.cache), guild_id)
        }
    }

    pub(super) fn delete_selection(cache: Arc<Cache>, guild_id: u64) {
        let _ = tokio::spawn(async move {
            let _ = await!(cache.inner.delete_choices(guild_id));
            Ok(())
        }.boxed().compat()); // Forget it
    }

    pub(super) async fn select(ctx: &Context, track: String) -> CommandResult {
        let guild_id = ctx.guild_id()?;

        await!(JoinCommand::join(JoinRequest::no_pop(&ctx)))?;

        let song = decoder::decode_track_base64(&track)?;

        Self::delete_selection(Arc::clone(&ctx.state.cache), guild_id);

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

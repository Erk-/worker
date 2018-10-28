use super::prelude::*;

pub struct ShuffleCommand;

impl ShuffleCommand {
    async fn _run(ctx: Context) -> CommandResult {
        let guild_id = ctx.guild_id()?;
        match await!(ctx.state.queue.get_limit(guild_id, 2)) {
            Ok(queue) => {
                if queue.len() < 2 {
                    return Response::err("To shuffle, you need at least 2 songs in the queue!");
                } // else {} - If the queue is >= 2, it'll shuffle like usual
            },
            Err(why) => {
                warn!("Error getting the queue of {}: {:?}", guild_id, why);

                return Response::err("There was an error getting your queue.");
            }
        }

        match await!(ctx.state.queue.shuffle(guild_id)) {
            Ok(()) => Response::text("Shuffled the song queue."),
            Err(why) => {
                warn!("Error shuffling guild {}: {:?}", guild_id, why);

                Response::err("There was an error shuffling the song.")
            },
        }
    }
}

impl<'a> Command<'a> for ShuffleCommand {
    fn names(&self) -> &'static [&'static str] {
        &["shuffle", "shufle"]
    }

    fn description(&self) -> &'static str {
        "Shuffles the current song queue."
    }

    fn run(&self, ctx: Context) -> RunFuture<'a> {
        trace!("running shuffle");

        RunFuture::new(Self::_run(ctx).boxed())
    }
}

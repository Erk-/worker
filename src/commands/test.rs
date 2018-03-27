use futures::prelude::*;
use command::{Command, Context};
use error::Error;

pub fn test() -> Command {
    Command {
        names: vec!["test", "t", "meme"],
        description: "Testing command lol",
        executor: run,
    }
}

#[async(boxed)]
fn run(ctx: Context) -> Result<(), Error> {
    let channel_id = ctx.msg.channel_id.0;
    let user_id = ctx.msg.author.id.0;

    let (guild_id, voice_state, shard_info) = {
        let cache_lock = ctx.discord_cache.borrow();
        let guild_id = cache_lock.get_guild_by_channel(&channel_id)?.clone();
        let voice_state = cache_lock.get_user_voice_state(&guild_id, &user_id);
        let shard_lock = ctx.shard.borrow();
        let shard_info = shard_lock.shard_info();
        (guild_id, voice_state, shard_info)
    };
    
    let future = ctx.serenity_http.send_message(channel_id, |m| m
        .content(
            format!("guild id: {}\nvoice state: {:?}\nshard info: {:?}", 
                guild_id, voice_state, shard_info)
        ))
        .map(|m| debug!("Sent message {:?}", m))
        .map_err(From::from);

    await!(future)
}
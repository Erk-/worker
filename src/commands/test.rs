use crate::command::{Command, CommandResult, Context, Response};

pub fn test() -> Command {
    Command {
        names: vec!["test", "t", "meme"],
        description: "Testing command lol",
    }
}

async fn run(ctx: Context) -> CommandResult {
    let user_id = ctx.msg.author.id.0;
    let guild_id = ctx.msg.guild_id?.0;

    let cache_lock = ctx.discord_cache.lock();
    let voice_state = cache_lock.get_user_voice_state(&guild_id, &user_id);
    let shard_lock = ctx.shard.lock();
    let shard_info = shard_lock.shard_info();

    let mut queue_manager = ctx.queue_manager.lock();
    let queue_lock = queue_manager.get_or_create(guild_id);
    let queue = queue_lock.lock();

    Response::text(format!(
        "guild id: {}\nvoice state: {:?}\nshard info: {:?}\nqueue size: {:?}",
        guild_id,
        voice_state,
        shard_info,
        queue.size()
    ))
}

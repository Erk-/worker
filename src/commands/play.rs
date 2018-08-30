use crate::command::{Command, CommandResult, Context, Response};

pub fn play() -> Command {
    Command {
        names: vec!["play", "p", "search", "youtube", "soundcloud"],
        description: "plays a song",
    }
}

async fn run(ctx: Context) -> CommandResult {
    // if ctx.args.len() < 1 {
    //     return Response::text("NEED ARGS");
    // }
    // let arg = ctx.args[0].clone();
    // let has_arg = arg.starts_with("-");

    // let tracks = {
    //     let id = if has_arg {
    //         ctx.args[1..].join(" ")
    //     } else {
    //         ctx.args.join(" ")
    //     };

    //     let (host, password) = {
    //         let node_manager = ctx.node_manager.lock();
    //         let node = node_manager.get_node(&node_manager.best_node()?)?;
    //         let node = node.lock();

    //         (node.http_host.clone(), node.password.clone())
    //     };

    //     debug!("requesting tracks for {}", id);
    //     let is_search = id.starts_with("ytsearch:") || id.starts_with("scsearch:");
    //     let load = await!(ctx.http_client.load_tracks(host.clone(), password.clone(), id.clone()))?;

    //     if load.tracks.len() < 1 && !is_search {
    //         let prefix = if ctx.alias == "soundcloud" { "scsearch:" } else { "ytsearch:" };
    //         let id = format!("{}:{}", prefix, id);
    //         await!(ctx.http_client.load_tracks(host, password, id))?.tracks
    //     } else {
    //         load.tracks
    //     }
    // };
    // debug!("returned tracks: {:?}", &tracks);
    // // TODO: choose command

    // let user_id = ctx.msg.author.id.0;
    // let guild_id = ctx.msg.guild_id?.0;

    // {
    //     let mut queue_manager = ctx.queue_manager.lock();
    //     let queue_lock = queue_manager.get_or_create(guild_id);
    //     let mut queue = queue_lock.lock();

    //     if !has_arg {
    //         let track = tracks[0].clone().track;
    //         queue.push_back(track);
    //     } else if arg == "-first" {
    //         let track = tracks[0].clone().track;
    //         queue.push_front(track);
    //     } else if arg == "-playlist" {
    //         let tracks = tracks.iter().map(|t| t.track.clone()).collect();
    //         queue.push_back_many(tracks);
    //     }
    // }

    // let cache_lock = ctx.discord_cache.lock();
    // let voice_state = cache_lock.get_user_voice_state(&guild_id, &user_id);

    // if voice_state.is_none() {
    //     return Response::text("NO VOICE STATE");
    // }

    // let playback_manager = ctx.playback_manager.lock();
    // if let Err(e) = playback_manager.play_next_guild(guild_id, false) {
    //     error!("error playing track: {:?}", e);
    //     Response::text("error playing track")
    // } else {
    //     Response::text("enqueued or playing,")
    // }
    Response::text("todo")
}

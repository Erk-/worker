use command::{Command, Context, CommandResult, Response};

use futures::prelude::*;
use lavalink::rest::hyper::LavalinkRestRequester;

pub fn play() -> Command {
    Command {
        names: vec!["play", "p"],
        description: "plays a song",
        executor: run,
    }
}

#[async(boxed)]
fn run(ctx: Context) -> CommandResult {
    if ctx.args.len() < 1 {
        return Response::text("NEED ARGS");
    }
    let arg = ctx.args[0].clone();
    let has_arg = arg.starts_with("-");

    let tracks = {
        let id = if has_arg { ctx.args[1..].join(" ") } else { ctx.args.join(" ") };

        let (host, password) = {
            let node_manager = ctx.node_manager.borrow_mut();
            let node = node_manager.get_node(node_manager.best_node()?)?;

            (node.http_host.clone(), node.password.clone())
        };

        debug!("requesting tracks for {}", id);
        await!(ctx.http_client.load_tracks(host, password, id))?
    };
    debug!("returned tracks: {:?}", &tracks);

    let user_id = ctx.msg.author.id.0;
    let guild_id = ctx.msg.guild_id?.0;
    
    {
        let mut queue_manager = ctx.queue_manager.try_borrow_mut()?;
        let queue_lock = queue_manager.get_or_create(guild_id);
        let mut queue = queue_lock.try_borrow_mut()?;

        if !has_arg {
            let track = tracks[0].clone().track;
            queue.push_back(track);
        } else if arg == "-first" {
            let track = tracks[0].clone().track;
            queue.push_front(track);
        } else if arg == "-playlist" {
            let tracks = tracks.iter().map(|t| t.track.clone()).collect();
            queue.push_back_many(tracks);
        }
        
    }
    
    let cache_lock = ctx.discord_cache.borrow();
    let voice_state = cache_lock.get_user_voice_state(&guild_id, &user_id);

    if voice_state.is_none() {
        return Response::text("NO VOICE STATE");
    }

    let playback_manager = ctx.playback_manager.borrow();
    if let Err(e) = playback_manager.play_next_guild(guild_id, false) {
        error!("error playing track: {:?}", e);
        Response::text("error playing track")
    } else {
        Response::text("enqueued or playing,")
    }
}
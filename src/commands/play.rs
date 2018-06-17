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
    let tracks = {
        let id = ctx.args[0].clone();

        let (host, password) = {
            let node_manager = ctx.node_manager.borrow_mut();
            let node = node_manager.get_node(node_manager.best_node()?)?;

            (node.http_host.clone(), node.password.clone())
        };

        debug!("requesting tracks for {}", &id);
        await!(ctx.http_client.load_tracks(&host, &password, id))?
    };

    let user_id = ctx.msg.author.id.0;
    let guild_id = ctx.msg.guild_id?.0;
    
    let cache_lock = ctx.discord_cache.borrow();
    let voice_state = cache_lock.get_user_voice_state(&guild_id, &user_id);

    if voice_state.is_none() {
        return Response::text("NO VOICE STATE");
    }

    let node_manager = ctx.node_manager.borrow_mut();
    let mut player_manager = node_manager.player_manager.borrow_mut();
    
    let mut player = match player_manager.get_mut(&guild_id) {
        Some(player) => player,
        None => {
            return Response::text("no player in this guild");
        }
    };

    if let Err(e) = player.play(&tracks[0].track, None, None) {
        error!("error playing track: {:?}", e);
        Response::text("error playing track")
    } else {
        Response::text("playing song")
    }
}
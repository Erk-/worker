use crate::{
    error::Result,
    utils,
    worker::WorkerState,
};
use futures::{
    channel::oneshot::Receiver as OneshotReceiver,
    compat::Future01CompatExt as _,
    future::{FutureExt, TryFutureExt},
};
use lavalink::decoder::{self, DecodedTrack};
use redis_async::{
    client::PairedConnection,
    resp::{FromResp, RespValue},
};
use std::sync::Arc;

#[derive(Clone, Debug, Deserialize, Serialize)]
struct SongPlayed {
    guild_id: u64,
    song_id: i64,
    track: String,
}

pub async fn from_lavalink(
    redis: PairedConnection,
    state: Arc<WorkerState>,
    shutdown_channel: OneshotReceiver<()>,
) -> Result<()> {
    let mut listener_future = lavalink_listener(redis, state).boxed();
    let mut shutdown_future = shutdown_channel.boxed();

    futures::select! {
        listener_future => {},
        shutdown_future => {
            if let Err(why) = shutdown_future {
                warn!("Err with lavalink msgs future: {:?}", why);
            }
        },
    }

    info!("Exiting from_lavalink");

    Ok(())
}

async fn lavalink_listener(
    redis: PairedConnection,
    state: Arc<WorkerState>,
) -> Result<()> {
    loop {
        let res: Result<()> = try {
            let cmd = resp_array!["BLPOP", "lavalink:from", 0];
            let mut parts: Vec<RespValue> = await!(redis.send(cmd).compat())?;

            let part = if parts.len() == 2 {
                parts.remove(1)
            } else {
                None?;

                unreachable!();
            };

            let message: Vec<u8> = FromResp::from_resp(part)?;

            let played = serde_json::from_slice::<SongPlayed>(&message)?;

            trace!("New song played: {:?}", played);

            trace!("Deserializing new song track info");
            let track = decoder::decode_track_base64(&played.track)?;
            debug!("Deserialized new song track info: {:?}", track);

            utils::spawn(handle_song(played, track, Arc::clone(&state)).map_err(|why| {
                warn!("Err with song handler: {:?}", why);
            }));
        };

        if let Err(why) = res {
            warn!("Err with event loop: {:?}", why);
        }
    }
}

async fn handle_song(
    played: SongPlayed,
    track: DecodedTrack,
    state: Arc<WorkerState>,
) -> Result<()> {
    let channel_id: String = await!(state.redis.send(resp_array![
        "GET",
        format!("j:{}", played.guild_id)
    ]).compat())?;
    let id = channel_id.parse::<u64>()?;

    let msg = format!(
        "Now playing **{}** by **{}** `[{}]`",
        track.title,
        track.author,
        utils::track_length_readable(track.length),
    );

    await!(state.serenity.send_message(id, |mut m| {
        m.content(msg);

        m
    }).compat())?;

    Ok(())
}

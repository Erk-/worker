use crate::{
    config::Config,
    error::Error,
    queue::QueueManager,
};
use futures::compat::Future01CompatExt;
use humantime::format_duration;
use hyper::{
    client::HttpConnector,
    Body,
    Client,
};
use hyper_tls::HttpsConnector;
use lavalink::{
    decoder::DecodedTrack,
    model::VoiceUpdate,
};
use lavalink_http_server_requester::AudioManagerRequester;
use parking_lot::Mutex;
use std::{
    fmt::{Display, Formatter, Result as FmtResult},
    sync::Arc,
    time::Duration,
};

#[derive(Debug, Deserialize, Serialize)]
pub struct PlayerStateResponse {
    pub track: Option<String>,
    pub paused: bool,
    pub position: i64,
}

#[derive(Debug)]
pub struct PlayerState {
    pub track: Option<DecodedTrack>,
    pub paused: bool,
    pub position: i64,
}

impl Display for PlayerState {
    fn fmt(&self, f: &mut Formatter) -> FmtResult {
        match self.track.as_ref() {
            Some(track) => write!(f,
                "{}{} by {} (`{}/{}`) {}",
                (if self.paused { "(paused)" } else { "" }),
                track.title,
                track.author,
                format_duration(Duration::from_millis(self.position as u64)),
                format_duration(Duration::from_millis(track.length)),
                track.url.as_ref().unwrap_or(&"(no url)".to_owned())),
            None => write!(f, "nothing playing ")
        }
    }
}

pub struct PlaybackManager {
    config: Arc<Config>,
    http: Arc<Client<HttpsConnector<HttpConnector>, Body>>,
}

impl PlaybackManager {
    pub fn new(
        config: Arc<Config>,
        http: Arc<Client<HttpsConnector<HttpConnector>, Body>>,
    ) -> Self {
        Self {
            config,
            http,
        }
    }

    pub async fn play_next_guild(
        &self,
        guild_id: u64,
        _force: bool,
    ) -> Result<(), Error> {
        await!(self.http.audio_skip(self.address(), guild_id).compat())?;

        Ok(())
    }

    pub async fn play(
        &self,
        guild_id: u64,
        track: String,
    ) -> Result<(), Error> {
        debug!("trying to play {} in {}", track, guild_id);

        await!(self.http.audio_play(self.address(), guild_id, track).compat())?;

        Ok(())
    }

    pub async fn pause(&self, guild_id: u64) -> Result<(), Error> {
        await!(self.http.audio_pause(self.address(), guild_id, true).compat())?;

        Ok(())
    }

    pub async fn resume(&self, guild_id: u64) -> Result<(), Error> {
        await!(self.http.audio_pause(self.address(), guild_id, false).compat())?;

        Ok(())
    }

    pub async fn stop(&self, guild_id: u64) -> Result<(), Error> {
        await!(self.http.audio_stop(self.address(), guild_id).compat())?;

        Ok(())
    }

    pub async fn current(&self, guild_id: u64) -> Result<PlayerState, Error> {
        // todo
        unreachable!();
        // let current = await!(self.http.audio_get(
        //     self.address(),
        //     guild_id,
        // ).compat())?;

        // Ok(current)
    }

    pub async fn voice_update(&self, voice_update: VoiceUpdate) -> Result<(), Error> {
        let json = serde_json::to_string(&voice_update)?;
        await!(self.http.audio_voice_update(self.address(), json).compat())?;

        Ok(())
    }

    #[cfg(feature = "patron")]
    pub fn volume(&self, guild_id: u64, volume: i32) -> Result<(), Error> {
        let node_manager_lock = self.node_manager.as_ref()?;
        let node_manager = node_manager_lock.lock();

        let mut player_manager = node_manager.player_manager.lock();
        let player = player_manager.get_mut(&guild_id)?;

        player.volume(volume)?;

        Ok(())
    }

    #[inline]
    fn address(&self) -> &str {
        &self.config.lavalink.address
    }
}

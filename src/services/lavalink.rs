use crate::{
    config::Config,
    error::{Error, Result},
    utils,
};
use futures::compat::Future01CompatExt as _;
use hyper::{
    client::HttpConnector,
    Body,
    Client,
};
use hyper_tls::HttpsConnector;
use lavalink::{
    decoder::{self, DecodedTrack},
    model::VoiceUpdate,
    rest::Load,
};
use lavalink_http_server_requester::{
    model::AudioPlayerState,
    AudioManagerRequester as _,
};
use std::{
    convert::TryFrom,
    fmt::{Display, Formatter, Result as FmtResult},
    result::Result as StdResult,
    sync::Arc,
};

#[derive(Debug, Deserialize, Serialize)]
pub struct PlayerStateResponse {
    pub track: Option<String>,
    pub paused: bool,
    pub position: i64,
}

#[derive(Debug)]
pub struct PlayerState {
    pub guild_id: u64,
    pub paused: bool,
    pub position: i64,
    pub time: i64,
    pub track: Option<DecodedTrack>,
    pub volume: i32,
}

impl PlayerState {
    #[inline]
    pub fn is_playing(&self) -> bool {
        self.track.is_some()
    }
}

impl Display for PlayerState {
    fn fmt(&self, f: &mut Formatter) -> FmtResult {
        match self.track.as_ref() {
            Some(track) => {
                let status = if self.paused {
                    "Paused"
                } else {
                    "Currently Playing"
                };

                write!(f,
                    "{}: **{}** by **{}** `[{}/{}]`\n{}",
                    status,
                    track.title,
                    track.author,
                    utils::track_length_readable(self.position as u64),
                    utils::track_length_readable(track.length),
                    track.url.as_ref().unwrap_or(&"(no url)".to_owned()),
                )
            },
            None => write!(f, "No song is currently playing.")
        }
    }
}

impl TryFrom<AudioPlayerState> for PlayerState {
    type Error = Error;

    fn try_from(state: AudioPlayerState) -> StdResult<Self, Self::Error> {
        let AudioPlayerState {
            guild_id,
            paused,
            position,
            time,
            track,
            volume,
        } = state;

        let decoded = match track {
            Some(bytes) => Some(decoder::decode_track_base64(bytes)?),
            None => None,
        };

        Ok(Self {
            track: decoded,
            guild_id,
            paused,
            position,
            time,
            volume,
        })
    }
}

pub struct LavalinkManager {
    config: Arc<Config>,
    http: Arc<Client<HttpsConnector<HttpConnector>, Body>>,
}

impl LavalinkManager {
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
    ) -> Result<()> {
        await!(self.http.audio_skip(self.address(), guild_id).compat())?;

        Ok(())
    }

    pub async fn play(
        &self,
        guild_id: u64,
        track: String,
    ) -> Result<bool> {
        debug!("trying to play {} in {}", track, guild_id);

        await!(self.http.audio_play(
            self.address(),
            guild_id,
            track,
        ).compat()).map_err(From::from)
    }

    pub async fn pause(&self, guild_id: u64) -> Result<()> {
        await!(self.http.audio_pause(self.address(), guild_id, true).compat())?;

        Ok(())
    }

    pub async fn resume(&self, guild_id: u64) -> Result<()> {
        await!(self.http.audio_pause(self.address(), guild_id, false).compat())?;

        Ok(())
    }

    pub async fn search(&self, text: String) -> Result<Load> {
        await!(self.http.audio_search(
            self.address(),
            text,
        ).compat()).map_err(From::from)
    }

    pub async fn seek(&self, guild_id: u64, position: i64) -> Result<()> {
        await!(self.http.audio_seek(
            self.address(),
            guild_id,
            position,
        ).compat()).map_err(From::from)
    }

    pub async fn skip(&self, guild_id: u64) -> Result<()> {
        await!(self.http.audio_skip(self.address(), guild_id).compat())?;

        Ok(())
    }

    pub async fn stop(&self, guild_id: u64) -> Result<()> {
        await!(self.http.audio_stop(self.address(), guild_id).compat())?;

        Ok(())
    }

    pub async fn current(&self, guild_id: u64) -> Result<PlayerState> {
        trace!("Getting player for guild {}", guild_id);

        let state = await!(self.http.audio_player(
            self.address(),
            guild_id,
        ).compat())?;

        trace!("Current player state: {:?}", state);

        PlayerState::try_from(state)
    }

    pub async fn voice_update(&self, voice_update: VoiceUpdate) -> Result<()> {
        trace!("Serializing voice update");
        let json = serde_json::to_string(&voice_update)?;
        trace!("Sending voice update: {}", json);
        await!(self.http.audio_voice_update(self.address(), json).compat())?;
        trace!("Sent voice update");

        Ok(())
    }

    #[cfg(feature = "patron")]
    pub async fn volume(&self, guild_id: u64, volume: u64) -> Result<()> {
        let addr = self.address();
        await!(self.http.audio_volume(addr, guild_id, volume).compat())?;

        Ok(())
    }

    #[inline]
    fn address(&self) -> &str {
        &self.config.lavalink.address
    }
}

#[cfg(test)]
mod tests {
    use lavalink::decoder::DecodedTrack;
    use super::PlayerState;

    fn state_empty() -> PlayerState {
        PlayerState {
            guild_id: 1,
            paused: true,
            position: 6250,
            time: 0,
            track: None,
            volume: 100,
        }
    }

    fn state_track() -> PlayerState {
        let mut state = state_empty();

        state.track = Some(DecodedTrack {
            author: "xKito Music".to_owned(),
            identifier: "zcn4-taGvlg".to_owned(),
            length: 184_000,
            source: "youtube".to_owned(),
            stream: false,
            title: "she - Prismatic".to_owned(),
            url: "https://www.youtube.com/watch?v=zcn4-taGvlg".to_owned().into(),
            version: 1,
        });

        state
    }

    #[test]
    fn test_is_playing() {
        assert!(!state_empty().is_playing());
        assert!(state_track().is_playing());
    }

    #[test]
    fn test_state_format() {
        assert_eq!(
            format!("{}", state_empty()),
            "No song is currently playing.",
        );

        assert_eq!(
            format!("{}", state_track()),
            "Paused: **she - Prismatic** by **xKito Music** `[6s/3m 4s]`
https://www.youtube.com/watch?v=zcn4-taGvlg");
    }
}

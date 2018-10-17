use crate::{
    config::Config,
    error::Result,
};
use futures::compat::Future01CompatExt as _;
use hyper::{
    client::HttpConnector,
    Body,
    Client,
};
use hyper_tls::HttpsConnector;
use lavalink_queue_requester::{
    model::{QueuedItem, Song, SongQueued},
    QueueRequester,
};
use std::{
    sync::Arc,
    u32,
};

pub struct QueueManager {
    config: Arc<Config>,
    http: QueueRequester<HttpsConnector<HttpConnector>>,
}

impl QueueManager {
    pub fn new(
        config: Arc<Config>,
        http: Arc<Client<HttpsConnector<HttpConnector>, Body>>,
    ) -> Self {
        Self {
            http: QueueRequester {
                client: http,
            },
            config,
        }
    }

    #[inline]
    pub async fn add<'a>(
        &'a self,
        guild_id: u64,
        track: impl Into<String> + 'a,
    ) -> Result<SongQueued> {
        await!(self._add(guild_id, track.into()))
    }

    async fn _add(&self, guild_id: u64, track: String) -> Result<SongQueued> {
        let mut songs = await!(self.http.add_tracks(
            self.address(),
            guild_id.to_string(),
            vec![track],
        ))?;

        if songs.is_empty() {
            None?;

            unreachable!();
        } else {
            Ok(songs.remove(0))
        }

        // Ok(songs.remove(0))
    }

    #[inline]
    pub async fn add_multiple<'a>(
        &'a self,
        guild_id: u64,
        tracks: Vec<impl Into<String> + 'a>,
    ) -> Result<Vec<SongQueued>> {
        let tracks = tracks.into_iter().map(Into::into).collect();

        await!(self._add_multiple(guild_id, tracks))
    }

    async fn _add_multiple(
        &self,
        guild_id: u64,
        tracks: Vec<String>,
    ) -> Result<Vec<SongQueued>> {
        await!(self.http.add_tracks(
            self.address(),
            guild_id.to_string(),
            tracks,
        )).map_err(From::from)
    }

    pub async fn clear(&self, guild_id: u64) -> Result<()> {
        await!(self.http.delete_queue(
            self.address(),
            guild_id.to_string(),
        )).map_err(From::from)
    }

    pub async fn get(&self, guild_id: u64) -> Result<Vec<QueuedItem>> {
        await!(self.get_limit(guild_id, u32::MAX))
    }

    pub async fn get_limit(&self, guild_id: u64, limit: u32) -> Result<Vec<QueuedItem>> {
        await!(self.http.get_queue_with_limit(
            self.address(),
            guild_id.to_string(),
            limit,
        )).map_err(From::from)
    }

    pub async fn pop(&self, guild_id: u64) -> Result<Option<Song>> {
        await!(self.http.pop_queue(
            self.address(),
            guild_id.to_string(),
        )).map_err(From::from)
    }

    #[inline]
    fn address(&self) -> &str {
        &self.config.queue.address
    }
}

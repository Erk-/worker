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
    QueueRequester as _,
};
use std::sync::Arc;

pub struct QueueManager {
    config: Arc<Config>,
    http: Arc<Client<HttpsConnector<HttpConnector>, Body>>,
}

impl QueueManager {
    pub fn new(
        config: Arc<Config>,
        http: Arc<Client<HttpsConnector<HttpConnector>, Body>>,
    ) -> Self {
        Self {
            config,
            http,
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
        await!(self.http.add_track(
            self.address(),
            guild_id.to_string(),
            track,
        ).compat()).map_err(From::from)
    }

    pub async fn clear(&self, guild_id: u64) -> Result<()> {
        await!(self.http.delete_queue(
            self.address(),
            guild_id.to_string(),
        ).compat()).map_err(From::from)
    }

    pub async fn get(&self, guild_id: u64) -> Result<Vec<QueuedItem>> {
        await!(self.http.get_queue(
            self.address(),
            guild_id.to_string(),
        ).compat()).map_err(From::from)
    }

    pub async fn pop(&self, guild_id: u64) -> Result<Option<Song>> {
        await!(self.http.pop_queue(
            self.address(),
            guild_id.to_string(),
        ).compat()).map_err(From::from)
    }

    #[inline]
    fn address(&self) -> &str {
        &self.config.queue.address
    }
}

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
    model::QueuedItem,
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

    pub async fn get(&self, guild_id: u64) -> Result<Vec<QueuedItem>> {
        await!(self.http.get_queue(
            self.address(),
            guild_id.to_string(),
        ).compat()).map_err(From::from)
    }

    #[inline]
    fn address(&self) -> &str {
        &self.config.queue.address
    }
}

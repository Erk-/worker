use crate::Result;
use std::{
    fs::File,
    io::prelude::*,
    net::{IpAddr, SocketAddr},
    path::Path,
    str::FromStr as _,
};

#[derive(Deserialize, Debug)]
struct Sharding {
    pub lower: u64,
    pub upper: u64,
    pub total: u64,
}

#[derive(Deserialize, Debug, Clone)]
pub struct Node {
    pub http_host: String,
    pub websocket_host: String,
    pub password: String,
}

#[derive(Deserialize, Debug)]
pub struct Config {
    pub bot_prefixes: Vec<String>,
    pub discord_token: String,
    pub discord_user_id: u64,
    pub lavalink: LavalinkConfig,
    pub postgres_addr: String,
    pub queue: QueueConfig,
    pub owners: Vec<u64>,
    pub owo_token: String,
    pub redis: RedisConfig,
}

impl Config {
    pub fn new(path: impl AsRef<Path>) -> Result<Config> {
        Self::_new(path.as_ref())
    }

    fn _new(path: &Path) -> Result<Config> {
        let mut file = File::open(&path)?;
        let mut contents = Vec::new();
        file.read_to_end(&mut contents)?;

        toml::from_slice(&contents).map_err(From::from)
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct LavalinkConfig {
    pub address: String,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct QueueConfig {
    pub address: String,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct RedisConfig {
    database: i64,
    host: String,
    password: Option<String>,
    port: u16,
}

impl RedisConfig {
    pub fn addr(&self) -> Result<SocketAddr> {
        let ip_addr = IpAddr::from_str(&self.host)?;

        Ok(SocketAddr::new(ip_addr, self.port))
    }
}

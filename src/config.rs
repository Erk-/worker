use std::fs::File;
use std::io::prelude::*;
use std::io::Result as IOResult;

#[derive(Deserialize, Debug, Clone)]
pub struct Node {
    pub http_host: String,
    pub websocket_host: String,
    pub password: String,
}

#[derive(Deserialize, Debug)]
pub struct Config {
    pub bot_prefix: String,
    pub discord_token: String,
    pub lavalink_shards: u64,
    pub lavalink_user_id: u64,
    pub lavalink_nodes: Vec<Node>,
    pub postgres_addr: String,
    pub owners: Vec<u64>,
}

pub fn load(path: &str) -> IOResult<Config> {
    let mut file = File::open(path)?;
    let mut contents = String::new();
    file.read_to_string(&mut contents)?;
    let config: Config = ::toml::from_str(&contents)
        .expect("could not parse config");
    Ok(config)
}
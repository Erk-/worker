use std::fs::File;
use std::io::prelude::*;
use std::io::Result as IOResult;
use lavalink_futures::nodes::NodeConfig;

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
    pub bot_prefix: String,
    pub discord_token: String,
    pub lavalink_user_id: u64,
    pub lavalink_nodes: Vec<Node>,
    pub postgres_addr: String,
    pub owners: Vec<u64>,
    sharding: Sharding,
}

pub fn load(path: &str) -> IOResult<Config> {
    let mut file = File::open(path)?;
    let mut contents = String::new();
    file.read_to_string(&mut contents)?;
    let config: Config = ::toml::from_str(&contents)
        .expect("could not parse config");
    Ok(config)
}

impl Config {
    pub fn sharding(&self) -> [u64; 3] {
        let sharding = &self.sharding;
        [sharding.lower, sharding.upper, sharding.total]
    }

    pub fn node_configs(&self) -> Vec<NodeConfig> {
        let num_shards = self.sharding.total;
        let user_id = self.lavalink_user_id;

        self.lavalink_nodes.iter()
            .map(move |node| NodeConfig {
                http_host: node.http_host.clone(),
                num_shards,
                password: node.password.clone(),
                user_id: user_id.to_string(),
                websocket_host: node.websocket_host.clone(),
            })
            .collect()
    }
}
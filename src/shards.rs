use error::Error;

use futures::prelude::{async, await};
use serenity::gateway::Shard;
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;
use std::time::Duration;
use tokio_core::reactor::{Handle, Timeout};

type IShard = Rc<RefCell<Shard>>;

pub struct ShardManager {
    shards: HashMap<u64, IShard>,
}

#[async]
pub fn create_shard_manager(
    handle: Handle,
    token: String,
    range: [u64; 3],
) -> Result<ShardManager, Error> {
    let mut shards = HashMap::with_capacity((range[1] - range[0]) as usize);
    let token = Rc::new(token);

    for shard_id in range[0]..range[1] + 1 {
        info!("Starting shard id {}", shard_id);
        let shard = await!(Shard::new(
            token.clone(),
            [shard_id, range[2]],
            handle.clone(),
        ))?;

        shards.insert(shard_id, Rc::new(RefCell::new(shard)));

        await!(Timeout::new(Duration::from_secs(5), &handle)?)?;
    }

    Ok(ShardManager { shards })
}

impl ShardManager {
    pub fn shards(&self) -> Vec<IShard> {
        self.shards.values().map(|shard| shard.clone()).collect()
    }

    pub fn get_shard(&self, shard_id: &u64) -> Option<IShard> {
        Some(self.shards.get(shard_id)?.clone())
    }
}

use parking_lot::Mutex;
use std::{
    collections::{HashMap, VecDeque},
    sync::Arc,
};

#[derive(Default)]
pub struct QueueManager {
    queues: HashMap<u64, Arc<Mutex<Queue>>>,
}

impl QueueManager {
    pub fn get_or_create(&mut self, guild_id: u64) -> Arc<Mutex<Queue>> {
        if self.queues.contains_key(&guild_id) {
            return Arc::clone(&self.queues.get(&guild_id).unwrap());
        }
        debug!("creating a queue for {}", guild_id);
        let queue = Arc::new(Mutex::new(Queue::new(guild_id)));
        self.queues.insert(guild_id, Arc::clone(&queue));
        queue
    }
}

// TODO: move to postgres
pub struct Queue {
    pub guild_id: u64,
    queue: VecDeque<String>,
}

impl Queue {
    pub fn new(guild_id: u64) -> Self {
        Self {
            guild_id,
            queue: VecDeque::new(),
        }
    }

    pub fn pop_front(&mut self) -> Option<String> {
        self.queue.pop_front()
    }

    pub fn push_front(&mut self, track: String) {
        self.queue.push_front(track)
    }

    pub fn push_back(&mut self, track: String) {
        self.queue.push_back(track)
    }

    pub fn push_back_many(&mut self, tracks: Vec<String>) {
        for track in tracks.iter() {
            self.queue.push_back(track.clone());
        }
    }

    pub fn peek(&self) -> Vec<String> {
        Vec::from(self.queue.clone())
    }

    pub fn size(&self) -> usize {
        self.queue.len()
    }
}

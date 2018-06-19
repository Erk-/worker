use std::collections::HashMap;
use std::rc::Rc;
use std::cell::RefCell;

#[derive(Default)]
pub struct QueueManager {
    queues: HashMap<u64, Rc<RefCell<Queue>>>
}

impl QueueManager {
    pub fn get_or_create(&mut self, guild_id: u64) -> Rc<RefCell<Queue>> {
        if self.queues.contains_key(&guild_id) {
            return Rc::clone(&self.queues.get(&guild_id).unwrap())
        }
        debug!("creating a queue for {}", guild_id);
        let queue = Rc::new(RefCell::new(Queue::new(guild_id)));
        self.queues.insert(guild_id, Rc::clone(&queue));
        queue
    }
}

// TODO: move to postgres
pub struct Queue {
    pub guild_id: u64,
    queue: Vec<String>
}

impl Queue {
    pub fn new(guild_id: u64) -> Self {
        Self {
            guild_id,
            queue: Vec::new(),
        }
    }

    pub fn pop(&mut self) -> Option<String> {
        self.queue.pop()
    }

    pub fn push(&mut self, track: String) {
        self.queue.push(track)
    }

    pub fn size(&self) -> usize {
        self.queue.len()
    }
}
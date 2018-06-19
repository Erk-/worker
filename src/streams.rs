use error::Error;
use queue::QueueManager;

use std::rc::Rc;
use std::cell::RefCell;
use lavalink_futures::nodes::NodeManager;
use lavalink_futures::player::AudioPlayer;

#[derive(Default)]
pub struct PlaybackManager {
    node_manager: Option<Rc<RefCell<NodeManager>>>,
    queue_manager: Rc<RefCell<QueueManager>>
}

impl PlaybackManager {
    pub fn new(queue_manager: Rc<RefCell<QueueManager>>) -> Self {
        Self {
            node_manager: None,
            queue_manager
        }
    }

    pub fn set_node_manager(&mut self, node_manager: Rc<RefCell<NodeManager>>) {
        self.node_manager = Some(node_manager);
    }

    pub fn play_next_guild(&self, guild_id: u64, force: bool) -> Result<(), Error> {
        let node_manager_lock = self.node_manager.as_ref()?;
        let node_manager = node_manager_lock.borrow();
        
        let mut player_manager = node_manager.player_manager.try_borrow_mut()?;
        let player = player_manager.get_mut(&guild_id)?;

        self.play_next(player, force)?;
        Ok(())
    }

    pub fn play_next(&self, player: &mut AudioPlayer, force: bool) -> Result<(), Error> {
        // check that the player is not in use before popping queue
        // TODO: check player.track.is_some() (it doesn't work right now thanks zeyla)
        if player.position > 0 && player.time > 0 && !force  {
            debug!("dropping play command in {} - already in use", &player.guild_id);
            return Ok(())
        }

        debug!("trying to acquire queue locks.");
        let mut queue_manager = self.queue_manager.try_borrow_mut()?;
        let queue_lock = queue_manager.get_or_create(player.guild_id);
        let mut queue = queue_lock.try_borrow_mut()?;
        debug!("queue locks acquired.");

        let next = match queue.pop() {
            Some(t) => t,
            None => {
                debug!("queue was empty? {:?}", queue.size());
                return Ok(())
            },
        };
        debug!("queue popped {}", next);
        self.play(player, &next)?;
        Ok(())
    }

    pub fn play(&self, player: &mut AudioPlayer, track: &str) -> Result<(), Error> {
        debug!("trying to play {} in {}", track, &player.guild_id);
        player.play(track, None, None)?;
        Ok(())
    }
}
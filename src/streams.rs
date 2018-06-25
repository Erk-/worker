use error::Error;
use queue::QueueManager;

use lavalink::decoder::{DecodedTrack, decode_track_base64};
use lavalink_futures::nodes::NodeManager;
use lavalink_futures::player::AudioPlayer;
use std::cell::RefCell;
use std::collections::HashMap;
use std::fmt::{Display, Formatter, Result as FmtResult};
use std::rc::Rc;

#[derive(Debug)]
pub struct PlayerState {
    pub track: Option<DecodedTrack>,
    pub paused: bool,
    pub position: i64,
}

impl Display for PlayerState {
    fn fmt(&self, f: &mut Formatter) -> FmtResult {
        match self.track.as_ref() {
            Some(track) => write!(f, 
                "{}{} by {} (`{}/{}`) {}",
                (if self.paused { "(paused)" } else { "" }),
                track.title,
                track.author,
                self.position,
                track.length,
                track.url.as_ref().unwrap_or(&"(no url)".to_owned())),
            None => write!(f, "nothing playing ")
        }
    }
}

#[derive(Default)]
pub struct PlaybackManager {
    node_manager: Option<Rc<RefCell<NodeManager>>>,
    queue_manager: Rc<RefCell<QueueManager>>,
    track_cache: RefCell<HashMap<u64, Option<String>>>,
}

impl PlaybackManager {
    pub fn new(queue_manager: Rc<RefCell<QueueManager>>) -> Self {
        Self {
            node_manager: None,
            queue_manager,
            track_cache: RefCell::new(HashMap::default())
        }
    }

    fn set_cache_track(&self, guild_id: u64, track: Option<String>) {
        let mut cache = self.track_cache.borrow_mut();
        cache.insert(guild_id, track);
    }

    fn get_cache_track(&self, guild_id: u64) -> Option<String> {
        let cache = self.track_cache.borrow();
        // TODO: syntax sugar this (i'm sure it's possible)
        match cache.get(&guild_id) {
            Some(inner) => {
                match inner {
                    Some(ref s) => Some(s.clone()),
                    None => None,
                }
            },
            None => None,
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
        if player.position > 0 && player.time > 0 && !force {
            debug!(
                "dropping play command in {} - already in use",
                &player.guild_id
            );
            return Ok(());
        }

        debug!("trying to acquire queue locks.");
        let mut queue_manager = self.queue_manager.try_borrow_mut()?;
        let queue_lock = queue_manager.get_or_create(player.guild_id);
        let mut queue = queue_lock.try_borrow_mut()?;
        debug!("queue locks acquired.");

        let next = match queue.pop_front() {
            Some(t) => t,
            None => {
                debug!("queue was empty? {:?}", queue.size());
                self.set_cache_track(player.guild_id, None);
                return Ok(());
            }
        };
        debug!("queue popped {}", next);
        self.play(player, next)?;
        Ok(())
    }

    pub fn play(&self, player: &mut AudioPlayer, track: String) -> Result<(), Error> {
        debug!("trying to play {} in {}", track, &player.guild_id);
        self.set_cache_track(player.guild_id, Some(track.clone()));
        player.play(&track, None, None)?;
        Ok(())
    }

    pub fn pause(&self, guild_id: u64) -> Result<(), Error> {
        self.play_state(guild_id, true)
    }
    pub fn resume(&self, guild_id: u64) -> Result<(), Error> {
        self.play_state(guild_id, false)
    }

    fn play_state(&self, guild_id: u64, pause: bool) -> Result<(), Error> {
        let node_manager_lock = self.node_manager.as_ref()?;
        let node_manager = node_manager_lock.borrow();

        let mut player_manager = node_manager.player_manager.try_borrow_mut()?;
        let player = player_manager.get_mut(&guild_id)?;

        player.pause(pause)?;

        Ok(())
    }

    pub fn current(&self, guild_id: u64) -> Result<PlayerState, Error> {
        let node_manager_lock = self.node_manager.as_ref()?;
        let node_manager = node_manager_lock.borrow();

        let mut player_manager = node_manager.player_manager.try_borrow_mut()?;
        let player = player_manager.get_mut(&guild_id)?;

        info!("player.track doing a {:?}", &player.track);

        let track = match self.get_cache_track(guild_id) {
            Some(track) => Some(decode_track_base64(&track)?),
            None => None,
        };
        
        Ok(PlayerState{
            track,
            paused: player.paused,
            position: player.position,
        })
    }

    #[cfg(feature = "patron")]
    pub fn volume(&self, guild_id: u64, volume: i32) -> Result<(), Error> {
        let node_manager_lock = self.node_manager.as_ref()?;
        let node_manager = node_manager_lock.borrow();

        let mut player_manager = node_manager.player_manager.try_borrow_mut()?;
        let player = player_manager.get_mut(&guild_id)?;

        player.volume(volume)?;

        Ok(())
    }
}

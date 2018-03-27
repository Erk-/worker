use std::collections::HashMap;
use serenity::model::channel::Channel;
use serenity::model::event::GatewayEvent::{self, Dispatch};
use serenity::model::id::ChannelId;
use serenity::model::voice::VoiceState;
use std::cell::RefCell;

#[derive(Default)]
pub struct DiscordCache {
    // <channel id, guild id>
    channel_guild_ids: HashMap<u64, u64>,
    
    // <guild id, <user id, voice state>>
    voice_state_updates: HashMap<u64, RefCell<HashMap<u64, CacheVoiceState>>>
}

impl DiscordCache {
    pub fn update(&mut self, event: &GatewayEvent) {
        use Event::*;

        match &event {
            Dispatch(_, ChannelCreate(e)) => {
                if let Channel::Guild(ref lock) = e.channel {
                    let channel = lock.borrow();
                    let channel_id = channel.id.0;
                    let guild_id = channel.guild_id.0;
                    trace!("channel create: channel_id={} guild_id={}", &channel_id, &guild_id);
                    self.channel_guild_ids.insert(channel_id, guild_id);
                }
            },
            Dispatch(_, ChannelDelete(e)) => {
                if let Channel::Guild(ref lock) = e.channel {
                    let channel = lock.borrow();
                    let channel_id = channel.id.0;
                    trace!("channel delete: channel_id={}", &channel_id);
                    self.channel_guild_ids.remove(&channel_id);
                }
            },
            Dispatch(_, GuildCreate(e)) => {
                let guild_id = e.guild.id.0;

                for ChannelId(channel_id) in e.guild.channels.keys() {
                    trace!("guild create: channel_id={} guild_id={}", &channel_id, &guild_id);
                    self.channel_guild_ids.insert(channel_id.clone(), guild_id);
                }

                let inner = RefCell::new(e.guild.voice_states.iter()
                    .filter(|(_, voice_state)| voice_state.channel_id.is_some())
                    .map(|(user_id, voice_state)| {
                        let VoiceState { ref channel_id, ref session_id, ref token, .. } = voice_state;
                        let state = CacheVoiceState {
                            channel_id: channel_id.unwrap().0,
                            session_id: session_id.to_owned(),
                            token: token.to_owned(),
                        };
                        (user_id.0, state)
                    })
                    .collect());

                self.voice_state_updates.insert(guild_id, inner);
            },
            Dispatch(_, GuildDelete(e)) => {
                let guild_id = e.guild.id.0;

                let channel_ids = self.channel_guild_ids.iter()
                    .filter_map(move |(channel_id, guild_id_1)| {
                        if guild_id_1 == &guild_id {
                            Some(channel_id.clone())
                        } else {
                            None
                        }
                    })
                    .collect::<Vec<u64>>();
                
                for channel_id in channel_ids.iter() {
                    trace!("guild delete channel_id={}", &channel_id);
                    self.channel_guild_ids.remove(channel_id);
                }

                let has_guild = self.voice_state_updates.contains_key(&guild_id);
                if has_guild {
                    self.voice_state_updates.remove(&guild_id);
                }
            },
            Dispatch(_, VoiceStateUpdate(e)) => {
                let guild_id = match e.guild_id {
                    Some(guild_id) => guild_id.0,
                    None => {
                        trace!("received voice state update without guild id");
                        return;
                    }
                };
                
                let VoiceState { ref channel_id, ref session_id, ref token, ref user_id, .. } = e.voice_state;
                let user_id = user_id.0;

                let has_guild = self.voice_state_updates.contains_key(&guild_id);
                let has_voice_state = has_guild && match self.voice_state_updates.get(&guild_id) {
                    Some(ref inner) => inner.borrow().contains_key(&user_id),
                    None => false
                };

                match channel_id {
                    Some(channel_id) => {
                        let state = CacheVoiceState {
                            channel_id: channel_id.0, 
                            session_id: session_id.to_owned(), 
                            token: token.to_owned(),
                        };

                        trace!("inserting voice state {:?} for user {}", &state, &user_id);
                        if has_guild {
                            if let Some(ref mut inner) = self.voice_state_updates.get(&guild_id) {
                                inner.borrow_mut().insert(user_id, state);
                            } else {
                                error!("user has voice state but could not get inner state map");
                            }
                        } else {
                            trace!("creating map for guild {} voice states", &guild_id);
                            let mut map = HashMap::default();
                            map.insert(user_id, state);
                            self.voice_state_updates.insert(guild_id, RefCell::new(map));
                        }
                    },
                    None if has_voice_state => {
                        let mut inner_len: usize;

                        if let Some(ref mut inner) = self.voice_state_updates.get(&guild_id) {
                            trace!("removing voice state for user {}", &user_id);
                            let mut inner = inner.borrow_mut();
                            inner.remove(&user_id);
                            inner_len = inner.len();
                        } else {
                            error!("user has voice state but could not get inner state map");
                            return;
                        }

                        if inner_len == 0 {
                            trace!("removing empty voice state map for guild {}", &guild_id);
                            self.voice_state_updates.remove(&guild_id);
                        }
                    }, 
                    _ => {},
                }
            }
            _ => {},
        }
    }

    pub fn get_guild_by_channel(&self, channel_id: &u64) -> Option<&u64> {
        self.channel_guild_ids.get(channel_id)
    }

    pub fn get_user_voice_state(&self, guild_id: &u64, user_id: &u64) -> Option<CacheVoiceState> {
        let inner = self.voice_state_updates.get(guild_id)?.borrow();
        Some(inner.get(user_id)?.clone())
    }
}

#[derive(Debug, Clone)]
pub struct CacheVoiceState {
    pub channel_id: u64,
    pub session_id: String,
    pub token: Option<String>,
}
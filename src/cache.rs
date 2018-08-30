use parking_lot::RwLock;
use serenity::model::{
    event::{Event, GatewayEvent},
    voice::VoiceState,
};
use std::collections::HashMap;

#[derive(Default)]
pub struct DiscordCache {
    // <guild id, <user id, voice state>>
    voice_state_updates: HashMap<u64, RwLock<HashMap<u64, CacheVoiceState>>>,
}

impl DiscordCache {
    pub fn update(&mut self, event: &GatewayEvent) {
        use self::{
            Event::*,
            GatewayEvent::Dispatch,
        };

        match &event {
            Dispatch(_, GuildCreate(e)) => {
                let guild_id = e.guild.id.0;

                let inner = RwLock::new(
                    e.guild
                        .voice_states
                        .iter()
                        .filter(|(_, voice_state)| voice_state.channel_id.is_some())
                        .map(|(user_id, voice_state)| {
                            let VoiceState {
                                ref channel_id,
                                ref session_id,
                                ref token,
                                deaf: _,
                                mute: _,
                                self_deaf: _,
                                self_mute: _,
                                suppress: _,
                                user_id: _,
                            } = voice_state;
                            let state = CacheVoiceState {
                                channel_id: channel_id.unwrap().0,
                                session_id: session_id.to_owned(),
                                token: token.to_owned(),
                            };

                            (user_id.0, state)
                        })
                        .collect(),
                );

                self.voice_state_updates.insert(guild_id, inner);
            }
            Dispatch(_, GuildDelete(e)) => {
                let guild_id = e.guild.id.0;

                if self.voice_state_updates.contains_key(&guild_id) {
                    self.voice_state_updates.remove(&guild_id);
                }
            }
            Dispatch(_, VoiceStateUpdate(e)) => {
                let guild_id = match e.guild_id {
                    Some(guild_id) => guild_id.0,
                    None => {
                        trace!("received voice state update without guild id");

                        return;
                    }
                };

                let VoiceState {
                    ref channel_id,
                    ref session_id,
                    ref token,
                    ref user_id,
                    deaf: _,
                    mute: _,
                    self_deaf: _,
                    self_mute: _,
                    suppress: _,
                } = e.voice_state;
                let user_id = user_id.0;

                let has_guild = self.voice_state_updates.contains_key(&guild_id);
                let has_voice_state = has_guild && match self.voice_state_updates.get(&guild_id) {
                    Some(ref inner) => inner.read().contains_key(&user_id),
                    None => false,
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
                                inner.write().insert(user_id, state);
                            } else {
                                error!("user has voice state but could not get inner state map");
                            }
                        } else {
                            trace!("creating map for guild {} voice states", &guild_id);
                            let mut map = HashMap::default();
                            map.insert(user_id, state);
                            self.voice_state_updates.insert(guild_id, RwLock::new(map));
                        }
                    },
                    None if has_voice_state => {
                        let inner_len = if let Some(ref mut inner) = self.voice_state_updates.get(&guild_id) {
                            trace!("removing voice state for user {}", &user_id);
                            let mut inner = inner.write();
                            inner.remove(&user_id);

                            inner.len()
                        } else {
                            error!("user has voice state but could not get inner state map");
                            return;
                        };

                        if inner_len == 0 {
                            trace!("removing empty voice state map for guild {}", &guild_id);
                            self.voice_state_updates.remove(&guild_id);
                        }
                    },
                    _ => {}
                }
            }
            _ => {}
        }
    }

    pub fn get_user_voice_state(&self, guild_id: &u64, user_id: &u64) -> Option<CacheVoiceState> {
        let inner = self.voice_state_updates.get(guild_id)?.read();
        Some(inner.get(user_id)?.clone())
    }
}

#[derive(Clone, Debug)]
pub struct CacheVoiceState {
    pub channel_id: u64,
    pub session_id: String,
    pub token: Option<String>,
}

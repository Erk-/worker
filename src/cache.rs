use parking_lot::RwLock;
use serenity::model::{
    event::{
        Event,
        GatewayEvent,
        GuildCreateEvent,
        GuildDeleteEvent,
        VoiceStateUpdateEvent,
    },
    voice::VoiceState,
};
use std::collections::HashMap;

#[derive(Default)]
pub struct DiscordCache {
    // <guild id, <user id, voice state>>
    pub voice_state_updates: HashMap<u64, RwLock<HashMap<u64, CacheVoiceState>>>,
}

impl DiscordCache {
    pub fn update(&mut self, event: &GatewayEvent) {
        use self::{
            Event::*,
            GatewayEvent::Dispatch,
        };

        match &event {
            Dispatch(_, GuildCreate(e)) => self.guild_create(e),
            Dispatch(_, GuildDelete(e)) => self.guild_delete(e),
            Dispatch(_, VoiceStateUpdate(e)) => self.voice_state_update(e),
            _ => {}
        }
    }

    pub fn get_user_voice_state(&self, guild_id: &u64, user_id: &u64) -> Option<CacheVoiceState> {
        let inner = self.voice_state_updates.get(guild_id)?.read();
        Some(inner.get(user_id)?.clone())
    }

    pub fn guild_has_user(&self, guild_id: u64, user_id: u64) -> bool {
        self.voice_state_updates.get(&guild_id)
            .map(|inner| inner.read().contains_key(&user_id))
            .unwrap_or(false)
    }

    fn guild_create(&mut self, e: &GuildCreateEvent) {
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

    fn guild_delete(&mut self, event: &GuildDeleteEvent) {
        let guild_id = event.guild.id.0;

        if self.voice_state_updates.contains_key(&guild_id) {
            self.voice_state_updates.remove(&guild_id);
        }
    }

    fn voice_state_update(&mut self, event: &VoiceStateUpdateEvent) {
        let guild_id = match event.guild_id {
            Some(guild_id) => guild_id.0,
            None => {
                trace!("received voice state update without guild id");

                return;
            },
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
        } = event.voice_state;
        let user_id = user_id.0;

        let (has_guild, has_user) = (
            self.voice_state_updates.contains_key(&guild_id),
            self.guild_has_user(guild_id, user_id),
        );

        if let Some(channel_id) = channel_id {
            let state = CacheVoiceState {
                channel_id: channel_id.0,
                session_id: session_id.clone(),
                token: token.clone(),
            };

            trace!("inserting voice state {:?} for user {}", state, user_id);

            if has_guild {
                if let Some(ref mut inner) = self.voice_state_updates.get(&guild_id) {
                    inner.write().insert(user_id, state);
                } else {
                    error!("user has voice state but could not get inner state map");
                }
            } else {
                trace!("creating map for guild {} voice states", guild_id);
                let mut map = HashMap::default();
                map.insert(user_id, state);
                self.voice_state_updates.insert(guild_id, RwLock::new(map));
            }
        } else if has_user {
            let remove = {
                let mut state = self.voice_state_updates.get_mut(&guild_id);

                if let Some(ref mut inner) = state {
                    trace!("removing voice state for user {}", &user_id);
                    let mut inner = inner.write();
                    inner.remove(&user_id);

                    inner.is_empty()
                } else {
                    error!("user has voice state but could not get inner state map");

                    return;
                }
            };

            if remove {
                trace!(
                    "removing empty voice state map for guild {}",
                    guild_id,
                );

                self.voice_state_updates.remove(&guild_id);
            }
        }
    }
}

#[derive(Clone, Debug)]
pub struct CacheVoiceState {
    pub channel_id: u64,
    pub session_id: String,
    pub token: Option<String>,
}

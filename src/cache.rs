use std::collections::HashMap;
use serenity::model::channel::Channel;
use serenity::model::event::GatewayEvent::{self, Dispatch};
use serenity::model::id::ChannelId;

#[derive(Default)]
pub struct DiscordCache {
    // <channel id, guild id>
    channel_guild_ids: HashMap<u64, u64>,
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
                    trace!("channel create: channel_id = {}, guild_id = {}", &channel_id, &guild_id);
                    self.channel_guild_ids.insert(channel_id, guild_id);
                }
            },
            Dispatch(_, ChannelDelete(e)) => {
                if let Channel::Guild(ref lock) = e.channel {
                    let channel = lock.borrow();
                    let channel_id = channel.id.0;
                    trace!("channel delete: channel_id = {}", &channel_id);
                    self.channel_guild_ids.remove(&channel_id);
                }
            },
            Dispatch(_, GuildCreate(e)) => {
                let guild_id = e.guild.id.0;

                for ChannelId(channel_id) in e.guild.channels.keys() {
                    trace!("guild create: channel_id = {}, guild_id = {}", &channel_id, &guild_id);
                    self.channel_guild_ids.insert(channel_id.clone(), guild_id);
                }
            },
            Dispatch(_, GuildDelete(e)) => {
                let id = e.guild.id.0;

                let channel_ids = self.channel_guild_ids.iter()
                    .filter_map(move |(channel_id, guild_id)| {
                        if guild_id == &id {
                            Some(channel_id.clone())
                        } else {
                            None
                        }
                    })
                    .collect::<Vec<u64>>();
                
                for channel_id in channel_ids.iter() {
                    trace!("guild delete: channel_id = {}", &channel_id);
                    self.channel_guild_ids.remove(channel_id);
                }
            },
            _ => {},
        }
    }

    pub fn get_guild_by_channel(&self, channel_id: &u64) -> Option<&u64> {
        self.channel_guild_ids.get(channel_id)
    }
}
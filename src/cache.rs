use cache::{
    model::VoiceState,
    Cache as DabbotCache,
};
use crate::{
    config::Config,
    error::Result,
};
use redis_async::client::PairedConnection;
use serenity::model::event::*;
use std::sync::Arc;

pub struct Cache {
    config: Arc<Config>,
    inner: DabbotCache,
}

impl Cache {
    pub fn new(config: Arc<Config>, redis: Arc<PairedConnection>) -> Self {
        Self {
            inner: DabbotCache::new(redis),
            config,
        }
    }

    pub async fn dispatch<'a>(&'a self, event: &'a GatewayEvent) -> Result<()> {
        use self::{
            Event::*,
            GatewayEvent::Dispatch,
        };

        match event {
            Dispatch(_, GuildCreate(e)) => {
                await!(self.guild_create(e)).map_err(From::from)
            },
            Dispatch(_, GuildDelete(e)) => {
                await!(self.guild_delete(e)).map_err(From::from)
            },
            Dispatch(_, VoiceServerUpdate(e)) => {
                self.voice_server_update(e);

                Ok(())
            },
            Dispatch(_, VoiceStateUpdate(e)) => {
                self.voice_state_update(e);

                Ok(())
            },
            _ => Ok(()),
        }
    }

    pub async fn voice_state(
        &self,
        guild_id: u64,
        user_id: u64,
    ) -> Result<Option<VoiceState>> {
        await!(self.inner.get_voice_state(
            guild_id,
            user_id,
        )).map_err(From::from)
    }

    async fn guild_create<'a>(&'a self, e: &'a GuildCreateEvent) -> Result<()> {
        let now = std::time::Instant::now();
        await!(self.inner.upsert_guild(&e.guild));
        info!("Upsert for {} took: {:?}", e.guild.id.0, now.elapsed());

        Ok(())
    }

    async fn guild_delete<'a>(&'a self, e: &'a GuildDeleteEvent) -> Result<()> {
        await!(self.inner.delete_voice_states(e.guild.id.0))?;

        Ok(())
    }

    fn voice_server_update<'a>(
        &'a self,
        e: &'a VoiceServerUpdateEvent,
    ) {
        let (guild_id, endpoint) = match (e.guild_id, e.endpoint.as_ref()) {
            (Some(id), Some(endpoint)) => (id.0, endpoint.clone()),
            _ => {
                warn!("Voice server update without full data: {:?}", e);

                return;
            },
        };

        self.inner.upsert_voice_state_info(
            guild_id,
            self.config.discord_user_id,
            endpoint,
            e.token.clone(),
        );
    }

    fn voice_state_update<'a>(
        &'a self,
        e: &'a VoiceStateUpdateEvent,
    ) {
        let guild_id = match e.guild_id {
            Some(id) => id.0,
            None => return,
        };

        self.inner.upsert_voice_state(guild_id, &e.voice_state);
    }
}

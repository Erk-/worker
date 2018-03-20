use error::Error;
use command::{CommandManager, Context};
use futures::Future;
use tokio_core::reactor::Handle;
use std::rc::Rc;
use std::cell::RefCell;
use serenity::model::event::{
    GatewayEvent, MessageCreateEvent, ReadyEvent, ResumedEvent, 
    GuildCreateEvent, GuildDeleteEvent, VoiceStateUpdateEvent, 
    VoiceServerUpdateEvent
};
use serenity::http::Client as SerenityHttpClient;
use regex::Regex;

pub struct EventHandler {
    handle: Handle,
    serenity_http: Rc<SerenityHttpClient>,
    split_regex: Regex,
    command_manager: Rc<RefCell<CommandManager>>,
}

const PREFIX: &'static str = ">";

impl EventHandler {
    pub fn new(handle: Handle, serenity_http: Rc<SerenityHttpClient>, command_manager: Rc<RefCell<CommandManager>>) -> Result<Self, Error> {
        let split_regex = Regex::new(r"\s+")?;

        Ok(Self {
            handle,
            serenity_http,
            split_regex,
            command_manager,
        })
    }

    pub fn on_event(&self, event: GatewayEvent) {
        use GatewayEvent::Dispatch;
        use Event::*;

        match event {
            Dispatch(_, Ready(e)) => self.on_ready(e),
            Dispatch(_, Resumed(e)) => self.on_resumed(e),
            Dispatch(_, MessageCreate(e)) => self.on_message(e),
            Dispatch(_, GuildCreate(e)) => self.on_guild_create(e),
            Dispatch(_, GuildDelete(e)) => self.on_guild_delete(e),
            Dispatch(_, VoiceStateUpdate(e)) => self.on_voice_state_update(e),
            Dispatch(_, VoiceServerUpdate(e)) => self.on_voice_server_update(e),
            _ => {
                // ya nothing else
            }
        }
    }

    fn on_ready(&self, _: ReadyEvent) {
        info!("Connected to discord!");
    }

    fn on_resumed(&self, _: ResumedEvent) {
        info!("Resumed connection to discord");
    }

    fn on_message(&self, event: MessageCreateEvent) {
        let msg = event.message;
        let content = msg.content.clone();
        println!("{}#{}: {}", msg.author.name, msg.author.discriminator, &content);
        
        if !content.starts_with(PREFIX) {
            return;
        }

        let content = &content[PREFIX.len()..];
        let mut content_iter = self.split_regex.split(&content);
        let command_name = match content_iter.next() {
            Some(c) => c,
            None => {
                // no
                return;
            }
        };

        let mut command_manager = self.command_manager.borrow_mut();
        let mut command = match command_manager.commands.get_mut(&command_name.to_lowercase()) {
            Some(command) => command.write().expect("could not get write lock on command"),
            None => {
                // invalid command
                return;
            }
        };
        
        let context = Context {
            handle: self.handle.clone(), 
            serenity_http: self.serenity_http.clone(),
            args: content_iter,
        };

        let future = command.run(context, msg)
            .map_err(|e| error!("oh no couldnt run command: {:?}", e));

        self.handle.spawn(future);
    }

    fn on_guild_create(&self, _: GuildCreateEvent) {
    }

    fn on_guild_delete(&self, _: GuildDeleteEvent) {
    }

    fn on_voice_state_update(&self, _: VoiceStateUpdateEvent) {
    }

    fn on_voice_server_update(&self, event: VoiceServerUpdateEvent) {
        debug!("voice server update: {:?}", event);
    }
}

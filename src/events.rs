use error::Error;
use command::{CommandManager, Context};
use futures::prelude::*;
use tokio_core::reactor::Handle;
use std::rc::Rc;
use std::cell::RefCell;
use serenity::model::event::{GatewayEvent, MessageCreateEvent};
use serenity::http::Client as SerenityHttpClient;

pub struct EventHandler {
    handle: Handle,
    serenity_http: Rc<SerenityHttpClient>,
    command_manager: Rc<RefCell<CommandManager>>,
}

impl EventHandler {
    pub fn new(handle: Handle, serenity_http: Rc<SerenityHttpClient>, command_manager: Rc<RefCell<CommandManager>>) -> Result<Self, Error> {
        Ok(Self {
            handle,
            serenity_http,
            command_manager,
        })
    }

    pub fn on_event(&self, event: GatewayEvent) {
        use GatewayEvent::Dispatch;
        use Event::*;

        match event {
            Dispatch(_, MessageCreate(e)) => {
                let future = on_message(e, 
                    self.command_manager.clone(), 
                    self.handle.clone(), 
                    self.serenity_http.clone()
                ).map_err(|e| match e {
                    Error::None(_) => {},
                    _ => error!("error handling MessageCreate: {:?}", e),
                });

                self.handle.spawn(future);
            },
            _ => {
                // ya nothing else
            }
        }
    }
}

#[async]
fn get_prefix(_guild_id: u64) -> Result<String, Error> {
    // todo dynamic prefix
    Ok(">".into())
}

#[async]
fn on_message(
    event: MessageCreateEvent, 
    command_manager: Rc<RefCell<CommandManager>>, 
    handle: Handle, 
    serenity_http: Rc<SerenityHttpClient>
) -> Result<(), Error> {
    let msg = event.message;
    let content = msg.content.clone();
    let guild_id = msg.guild_id()?.0;

    let prefix = await!(get_prefix(guild_id))?;
    if !content.starts_with(&prefix) {
        return Ok(());
    }
    
    let content_trimmed: String = content.chars().skip(prefix.len()).collect();
    let mut content_iter = content_trimmed.split_whitespace();
    let command_name = content_iter.next()?;

    let command_manager = command_manager.borrow();
    let command = command_manager.get(&command_name.to_lowercase())?;

    let context = Context {
        handle: handle.clone(), 
        serenity_http: serenity_http,
        msg,
        args: content_iter,
    };

    let future = command.run(context).map_err(|e| match e {
        Error::None(_) => {},
        _ => error!("error running command: {:?}", e),
    });

    handle.spawn(future);
    Ok(())
}
use error::Error;
use command::{CommandManager, Command, Context};
use futures::prelude::*;
use tokio_core::reactor::Handle;
use std::rc::Rc;
use std::cell::RefCell;
use serenity::model::event::{GatewayEvent, MessageCreateEvent};
use serenity::http::Client as SerenityHttpClient;
use regex::{Regex, Split as RegexSplit};

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
                let future = on_message(
                    e, 
                    self.command_manager.clone(), 
                    self.handle.clone(), 
                    self.serenity_http.clone()
                ).map_err(|_| ());

                self.handle.spawn(future);
            },
            _ => {
                // ya nothing else
            }
        }
    }
}

fn split_content<'a>(content: String, prefix: String) -> RegexSplit<'a, 'a> {
    lazy_static! {
        static ref SPLIT_REGEX: Regex = Regex::new(r"\s+").unwrap();
    }

    let content = &content[prefix.len()..];
    SPLIT_REGEX.split(content)
}

#[async]
fn get_prefix(_guild_id: u64) -> Result<String, Error> {
    // todo dynamic prefix
    Ok(">".into())
}

fn get_command(command_manager: Rc<RefCell<CommandManager>>, name: &str) -> Result<Rc<Command>, Error> {
    let command_manager = command_manager.borrow();
    command_manager.get(name)
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

    //let content = &content[prefix.len()..];
    let mut content_iter = split_content(content, prefix);
    let command_name = content_iter.next()?;
    
    let command = get_command(command_manager, &command_name.to_lowercase())?;

    let context = Context {
        handle: handle, 
        serenity_http: serenity_http,
        msg,
        args: content_iter,
    };

    let future = command.run(context);

    if let Err(e) = await!(future) {
        error!("oh no couldnt run command: {:?}", e);
    }
    
    Ok(())
}
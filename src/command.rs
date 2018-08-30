use crate::{
    worker::WorkerState,
    Result,
};
use futures::{
    compat::Future01CompatExt,
    FutureExt,
    TryFutureExt,
};
use serenity::{
    builder::CreateMessage,
    model::channel::Message,
};
use std::{
    future::FutureObj,
    sync::Arc,
};

pub type CommandResult = Result<Response>;

pub trait Command: Sync {
    fn description(&self) -> String;
    fn executor(&self, ctx: Context) -> FutureObj<CommandResult>;
    fn names(&self) -> Vec<String>;
}

pub struct Context {
    pub alias: String,
    pub args: Vec<String>,
    pub shard_id: u64,
    pub state: Arc<WorkerState>,
    pub msg: Message,
}

pub enum Response {
    Text(String),
}

impl Response {
    pub fn text(content: impl Into<String>) -> CommandResult {
        Self::_text(content.into())
    }

    fn _text(content: String) -> CommandResult {
        Ok(Response::Text(content.into()))
    }
}

pub async fn run(command: Box<Command + 'static>, ctx: Context) -> Result<()> {
    let serenity_http = Arc::clone(&ctx.state.serenity);
    let channel_id = ctx.msg.channel_id.0;

    let response = await!(command.executor(ctx))?;
    let m = match response {
        Response::Text(content) => |mut m: CreateMessage| {
            m.content(content);

            m
        },
    };

    let future = serenity_http
        .send_message(channel_id, m)
        .compat()
        .map_ok(move |msg| {
            trace!("Sent message to channel {}: {}", channel_id, msg.content);
        })
        .unit_error()
        .boxed();

    await!(future);

    Ok(())
}

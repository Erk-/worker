mod prelude;

pub mod about;
pub mod invite;
pub mod join;
pub mod leave;
pub mod pause;
pub mod ping;
pub mod play;
pub mod playing;
pub mod providers;
pub mod queue;
pub mod remove;
pub mod restart;
pub mod resume;
pub mod seek;
pub mod skip;
pub mod volume;

use crate::{
    lavalink::PlayerState,
    worker::WorkerState,
    Result,
};
use serenity::model::channel::Message;
use std::sync::Arc;

pub type CommandResult = Result<Response>;

pub struct Context {
    pub alias: String,
    pub args: Vec<String>,
    pub shard_id: u64,
    pub state: Arc<WorkerState>,
    pub msg: Message,
}

impl Context {
    pub async fn current(&self) -> Result<PlayerState> {
        let id = self.msg.guild_id?.0;

        await!(self.state.playback.current(id)).map_err(From::from)
    }

    pub async fn is_playing(&self) -> Result<bool> {
        await!(self.current())?;

        Ok(true)
    }

    pub async fn to_sharder(&self, payload: Vec<u8>) -> Result<()> {
        let key = format!("sharder:to:{}", self.shard_id);
        let cmd = resp_array!["RPUSH", key, payload];

        debug!("cmd: {:?}", cmd);

        self.state.redis.send_and_forget(cmd);

        Ok(())
    }
}

#[derive(Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
pub enum Response {
    Text(String),
}

impl Response {
    #[inline]
    pub fn err(content: impl AsRef<str>) -> CommandResult {
        Self::_err(content.as_ref())
    }

    fn _err(content: &str) -> CommandResult {
        Ok(Response::Text(format!("⚠ {}", content)))
    }

    #[inline]
    pub fn text(content: impl Into<String>) -> CommandResult {
        Self::_text(content.into())
    }

    fn _text(content: String) -> CommandResult {
        Ok(Response::Text(content.into()))
    }
}

fn no_song() -> Result<Response> {
    Response::text("No music is queued or playing on this guild! Add some using `!!!play <song name/link>`")
}

#[cfg(test)]
mod tests {
    use super::Response;

    #[test]
    fn test_responses() {
        assert_eq!(
            Response::err("foo").unwrap(),
            Response::Text("⚠ foo".to_owned()),
        );
        assert_eq!(
            Response::err("").unwrap(),
            Response::Text("⚠ ".to_owned()),
        );
        assert_eq!(
            Response::text("hello").unwrap(),
            Response::Text("hello".to_owned()),
        );
        assert_eq!(
            Response::text("").unwrap(),
            Response::Text("".to_owned()),
        );
    }
}

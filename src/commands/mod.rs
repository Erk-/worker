mod prelude;

pub mod about;
pub mod cancel;
pub mod choose;
pub mod clear;
pub mod discordfm;
pub mod dump;
pub mod help;
pub mod invite;
pub mod join;
pub mod leave;
pub mod load;
pub mod pause;
pub mod ping;
pub mod play;
pub mod playing;
pub mod providers;
pub mod queue;
pub mod radio;
pub mod remove;
pub mod restart;
pub mod resume;
pub mod seek;
pub mod shuffle;
pub mod skip;
pub mod soundcloud;
pub mod volume;
pub mod youtube;

use crate::{
    services::lavalink::PlayerState,
    worker::WorkerState,
    Result,
};
use futures::compat::Future01CompatExt;
use futures::future::FutureObj;
use lavalink_queue_requester::model::{QueuedItem, Song};
use self::{
    about::AboutCommand,
    cancel::CancelCommand,
    choose::ChooseCommand,
    clear::ClearCommand,
    discordfm::DfmCommand,
    dump::DumpCommand,
    help::HelpCommand,
    invite::InviteCommand,
    join::JoinCommand,
    leave::LeaveCommand,
    load::LoadCommand,
    pause::PauseCommand,
    ping::PingCommand,
    play::PlayCommand,
    playing::PlayingCommand,
    providers::ProvidersCommand,
    queue::QueueCommand,
    radio::RadioCommand,
    remove::RemoveCommand,
    restart::RestartCommand,
    resume::ResumeCommand,
    seek::SeekCommand,
    shuffle::ShuffleCommand,
    skip::SkipCommand,
    soundcloud::SoundCloudCommand,
    volume::VolumeCommand,
    youtube::YouTubeCommand,
};
use serenity::model::channel::Message;
use std::{
    collections::HashMap,
    sync::Arc,
};

pub type CommandResult = Result<Response>;
pub type RunFuture<'a> = FutureObj<'a, CommandResult>;

pub trait Command<'a>: Send + Sync {
    fn names(&self) -> &'static [&'static str];
    fn description(&self) -> &'static str;
    fn run(&self, ctx: Context) -> RunFuture<'a>;
}

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

    pub fn guild_id(&self) -> Option<u64> {
        self.msg.guild_id.map(|id| id.0)
    }

    pub async fn is_playing(&self) -> Result<bool> {
        await!(self.current())?;

        Ok(true)
    }

    pub fn prefix(&self) -> Option<&str> {
        self.state.config.bot_prefixes.first().map(|x| &**x)
    }

    pub async fn queue(
        &self,
        limit: u32,
        offset: u32,
    ) -> Result<Vec<QueuedItem>> {
        await!(self.state.queue.get_offset(self.guild_id()?, limit, offset))
    }

    pub async fn queue_pop(&self) -> Result<Option<Song>> {
        await!(self.state.queue.pop(self.guild_id()?))
    }

    pub async fn send_message<'a>(
        &'a self,
        content: impl AsRef<str> + 'a,
    ) -> Result<Message> {
        await!(self._send_message(content.as_ref()))
    }

    async fn _send_message<'a>(&'a self, content: &'a str) -> Result<Message> {
        await!(self.state.serenity.send_message(self.msg.channel_id.0, |mut m| {
            m.content(content);

            m
        }).compat()).map_err(From::from)
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

pub fn map() -> HashMap<String, Arc<Box<dyn Command<'static> + Send + Sync>>> {
    let mut map: HashMap<String, Arc<Box<dyn Command<'static> + Send + Sync>>> = HashMap::new();

    let commands: Vec<Arc<Box<dyn Command<'static> + Send + Sync>>> = vec![
        Arc::new(Box::new(AboutCommand)),
        Arc::new(Box::new(CancelCommand)),
        Arc::new(Box::new(ChooseCommand)),
        Arc::new(Box::new(ClearCommand)),
        Arc::new(Box::new(DfmCommand)),
        Arc::new(Box::new(DumpCommand)),
        Arc::new(Box::new(HelpCommand)),
        Arc::new(Box::new(InviteCommand)),
        Arc::new(Box::new(JoinCommand)),
        Arc::new(Box::new(LeaveCommand)),
        Arc::new(Box::new(LoadCommand)),
        Arc::new(Box::new(PauseCommand)),
        Arc::new(Box::new(PingCommand)),
        Arc::new(Box::new(PlayCommand)),
        Arc::new(Box::new(PlayingCommand)),
        Arc::new(Box::new(ProvidersCommand)),
        Arc::new(Box::new(QueueCommand)),
        Arc::new(Box::new(RadioCommand)),
        Arc::new(Box::new(RemoveCommand)),
        Arc::new(Box::new(RestartCommand)),
        Arc::new(Box::new(ResumeCommand)),
        Arc::new(Box::new(SeekCommand)),
        Arc::new(Box::new(ShuffleCommand)),
        Arc::new(Box::new(SkipCommand)),
        Arc::new(Box::new(SoundCloudCommand)),
        Arc::new(Box::new(VolumeCommand)),
        Arc::new(Box::new(YouTubeCommand)),
    ];

    for command in commands.into_iter() {
        let cmd = Arc::new(command);

        for name in cmd.names() {
            if map.contains_key(name.to_owned()) {
                error!("Duplicate entry in commands map: {}", name);
            }

            map.insert((*name).to_owned(), Arc::clone(&cmd));
        }
    }

    map
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

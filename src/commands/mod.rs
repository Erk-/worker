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
    command::Response,
    Result,
};

fn no_song() -> Result<Response> {
    Response::text("No music is queued or playing on this guild! Add some using `!!!play <song name/link>`")
}

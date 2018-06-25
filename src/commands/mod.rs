// todo: it would be so hot if this could not repeat itself so much
// figure it out if i can do some big magic with a macro here
mod join;
mod leave;
mod pause;
mod play;
mod playing;
mod queue;
mod resume;
mod skip;
mod test;
mod volume;

use command::Command;

use self::join::join;
use self::leave::leave;
use self::pause::pause;
use self::play::play;
use self::playing::playing;
use self::queue::queue;
use self::resume::resume;
use self::skip::skip;
use self::test::test;
use self::volume::volume;

pub fn create() -> Vec<Command> {
    // todo: should this be alphabetized or grouped as-is
    vec![
        // voice state
        join(),
        leave(),
        // player state
        play(),
        pause(),
        resume(),
        volume(),
        playing(),
        // queue state
        skip(),
        queue(),
        // admin\debug
        test(),
    ]
}
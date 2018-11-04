use super::prelude::*;

const ERROR_SEEKING: &'static str = "There was an error seeking the song.";

pub struct SeekCommand;

impl SeekCommand {
    async fn _run(ctx: Context) -> CommandResult {
        let guild_id = ctx.guild_id()?;

        match await!(ctx.is_playing()) {
            Ok(true) => {},
            Ok(false) => {
                return super::no_song();
            },
            Err(_) => {
                return Response::err(ERROR_SEEKING);
            },
        };

        let arg = match ctx.args.first() {
            Some(arg) => arg,
            None => {
                return Response::text(format!("You need to say where you want to seek to!

Example: `{}seek 3:40` to seek to 3 minutes and 40 seconds", ctx.prefix()?));
            },
        };

        let position = match Self::argument_time(arg) {
            Some(position) => position,
            None => {
                return Response::text(format!("That doesn't look like a valid time format.

Example: `{}seek 3:40` to seek to 3 minutes and 40 seconds", ctx.prefix()?));
            },
        };

        match await!(ctx.state.playback.seek(guild_id, position)) {
            Ok(()) => {
                Response::text(
                    "Jumped to the specified position. Use `!!!playing` to see the current song & \
                     position.",
                )
            },
            Err(why) => {
                warn!("Err seeking song for {} to {}: {:?}", guild_id, 0, why);

                Response::err(ERROR_SEEKING)
            },
        }
    }

    fn argument_time(input: &str) -> Option<i64> {
        let parts = input.split(':').collect::<Vec<_>>();
        let part_count = parts.len();

        if part_count > 3 {
            return None;
        }

        let part_count = parts.len();

        let mut numbers = [0; 3];

        for (idx, part) in parts.into_iter().enumerate() {
            let num = part.parse::<u8>().ok()?;

            if num > 59 {
                return None;
            }

            numbers[idx] = num;
        }

        let mut hours = 0;
        let mut minutes = 0;
        let seconds;

        match part_count {
            1 => {
                seconds = numbers[0];
            },
            2 => {
                minutes = numbers[0];
                seconds = numbers[1];
            },
            3 => {
                hours = numbers[0];
                minutes = numbers[1];
                seconds = numbers[2];
            },
            _ => return None,
        }

        let mut time = i64::from(seconds);

        if minutes > 0 {
            time += i64::from(minutes) * 60;
        }

        if hours > 0 {
            time += i64::from(hours) * 60 * 60;
        }

        Some(time * 1000)
    }
}

impl<'a> Command<'a> for SeekCommand {
    fn names(&self) -> &'static [&'static str] {
        &["seek", "jump"]
    }

    fn description(&self) -> &'static str {
        "Skips to a certain timestamp in the current song."
    }

    fn run(&self, ctx: Context) -> RunFuture<'a> {
        RunFuture::new(Self::_run(ctx).boxed())
    }
}

#[cfg(test)]
mod tests {
    use crate::error::Result;
    use super::SeekCommand;

    #[test]
    fn test_argument_time() -> Result<()> {
        assert_eq!(SeekCommand::argument_time("1")?, 1000);
        assert_eq!(SeekCommand::argument_time("2:3")?, 123_000);
        assert_eq!(SeekCommand::argument_time("20:31")?, 1_231_000);
        assert_eq!(SeekCommand::argument_time("2:34:21")?, 9_261_000);
        assert!(SeekCommand::argument_time("hi").is_none());
        assert!(SeekCommand::argument_time("1:30:20a").is_none());

        Ok(())
    }
}

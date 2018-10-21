use super::prelude::*;

pub static COMMAND_INSTANCE: VolumeCommand = VolumeCommand;

pub struct VolumeCommand;

impl VolumeCommand {
    #[cfg(not(feature = "patron"))]
    async fn _run(_: Context) -> CommandResult {
        Response::text(
            r#"**The volume command is dabBot premium only!**

Donate for the `Volume Control` tier on Patreon at https://patreon.com/dabbot to gain access."#,
        )
    }

    #[cfg(feature = "patron")]
    async fn _run(ctx: Context) -> CommandResult {
        let arg_len = ctx.args.len();

        match arg_len {
            0 => {
                let guild_id = ctx.guild_id()?;

                let player = match await!(ctx.state.playback.current(guild_id)) {
                    Ok(player) => player,
                    Err(why) => {
                        warn!("Err getting current player: {:?}", why);

                        return Response::err("There was an error getting the volume");
                    },
                };

                Response::text(format!("The volume is currently {}", player.volume))
            },
            1 => {
                let volume = match ctx.args[0].parse::<u64>() {
                    Ok(volume @ 0...150) => volume,
                    Ok(_) | Err(_) => {
                        return Response::text("The volume must be between 0 and 150");
                    },
                };

                let guild_id = ctx.guild_id()?;

                match await!(ctx.state.playback.volume(guild_id, volume)) {
                    Ok(()) => Response::text("Updated the volume."),
                    Err(why) => {
                        warn!(
                            "Error updating volume to {} for {}: {:?}",
                            volume, guild_id, why,
                        );

                        Response::err("There was an error updating the volume.")
                    },
                }
            },
            _ => Response::err("You supplied too many arguments!"),
        }
    }
}

impl<'a> Command<'a> for VolumeCommand {
    fn names(&self) -> &'static [&'static str] {
        &["volume", "vol", "v"]
    }

    fn description(&self) -> &'static str {
        "Changes the track volume."
    }

    fn run(&self, ctx: Context) -> RunFuture<'a> {
        RunFuture::new(Self::_run(ctx).boxed())
    }
}

#[cfg(feature = "patron")]
pub async fn run(ctx: Context) -> CommandResult {
    let arg_len = ctx.args.len();

    if arg_len == 0 {
        let guild_id = ctx.guild_id()?;

        let player = match await!(ctx.state.playback.current(guild_id)) {
            Ok(player) => player,
            Err(why) => {
                warn!("Err getting current player: {:?}", why);

                return Response::err("There was an error getting the volume");
            },
        };

        Response::text(format!("The volume is currently {}", player.volume))
    } else if arg_len == 1 {
        let volume = match ctx.args[0].parse::<u64>() {
            Ok(volume @ 0...150) => volume,
            Ok(_) | Err(_) => {
                return Response::text("The volume must be between 0 and 150");
            },
        };

        let guild_id = ctx.guild_id()?;

        match await!(ctx.state.playback.volume(guild_id, volume)) {
            Ok(()) => Response::text("Updated the volume."),
            Err(why) => {
                warn!(
                    "Error updating volume to {} for {}: {:?}",
                    volume, guild_id, why,
                );

                Response::err("There was an error updating the volume.")
            },
        }
    } else {
        Response::err("You passed too many arguments!")
    }
}

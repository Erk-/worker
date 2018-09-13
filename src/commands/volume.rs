use super::prelude::*;

pub const fn description() -> &'static str {
    "Change the track volume"
}

pub const fn names() -> &'static [&'static str] {
    &["volume", "vol", "v"]
}

#[cfg(not(feature = "patron"))]
pub async fn run(_: Context) -> CommandResult {
    Response::text(r#"**The volume command is dabBot premium only!**

Donate for the `Volume Control` tier on Patreon at https://patreon.com/dabbot to gain access."#)
}

#[cfg(feature = "patron")]
pub async fn run(ctx: Context) -> CommandResult {
    let arg_len = ctx.args.len();

    if arg_len == 0 {
        let guild_id = ctx.msg.guild_id?.0;

        let player = match await!(ctx.state.playback.current(guild_id)) {
            Ok(player) => player,
            Err(why) => {
                warn!("Err getting current player: {:?}", why);

                return Response::text("There was an error getting the volume");
            },
        };

        Response::text(format!("The volume is currently {}", player.volume))
    } else if arg_len == 1 {
        let volume = match ctx.args[0].parse::<u64>() {
            Ok(volume @ 0 ... 150) => volume,
            Ok(_) | Err(_) => {
                return Response::text("The volume must be between 0 and 150");
            },
        };

        let guild_id = ctx.msg.guild_id?.0;

        match await!(ctx.state.playback.volume(guild_id, volume)) {
            Ok(()) => Response::text("Updated the volume"),
            Err(why) => {
                warn!(
                    "Error updating volume to {} for {}: {:?}",
                    volume,
                    guild_id,
                    why,
                );

                Response::text("Error updating the volume")
            },
        }
    } else {
        Response::text("You passed too many arguments!")
    }
}

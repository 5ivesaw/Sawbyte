use azalea::app::{App, AppExit, Plugin, PluginGroup};
use azalea::core::tick::GameTick;
use azalea::ecs::prelude::*;
use azalea::entity::{HasClientLoaded, Jumping, LocalEntity, LookDirection, Physics, Position};
use azalea::local_player::WorldHolder;
use azalea::physics::{
    PhysicsSystems,
    client_movement::{ClientMovementState, DirectionStates},
};
use azalea::{Account, Client, ClientBuilder, Event, NoState};
use sawbyte_core::{Button, InputFrame, tape};
use std::fs::File;
use std::io::{BufReader, BufWriter, Write};
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use std::time::Duration;

#[derive(Resource)]
struct Replay {
    frames: Vec<InputFrame>,
    index: usize,
    settle: u32,
    started: bool,
    finished: bool,
    trace: BufWriter<File>,
}

struct InputPlugin(Arc<Mutex<Option<Replay>>>);
impl Plugin for InputPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(self.0.lock().unwrap().take().unwrap())
            .add_systems(GameTick, apply_inputs.before(PhysicsSystems))
            .add_systems(
                GameTick,
                record_state.after(PhysicsSystems).after(apply_inputs),
            );
    }
}

// Run in the physics schedule, not the asynchronously delivered Event::Tick.
#[allow(clippy::type_complexity)]
fn apply_inputs(
    mut replay: ResMut<Replay>,
    mut players: Query<
        (&mut ClientMovementState, &mut Jumping, &mut LookDirection),
        (With<LocalEntity>, With<HasClientLoaded>),
    >,
) {
    let Ok((mut movement, mut jumping, mut look)) = players.single_mut() else {
        return;
    };
    if replay.finished {
        return;
    }
    let frame = if replay.settle > 0 {
        replay.settle -= 1;
        InputFrame::default()
    } else {
        if !replay.started {
            eprintln!("SAWBYTE_REPLAY_STARTED");
        }
        replay.started = true;
        replay.frames.get(replay.index).copied().unwrap_or_default()
    };
    let b = frame.buttons;
    movement.move_direction = DirectionStates {
        forward: b.pressed(Button::Forward),
        backward: b.pressed(Button::Back),
        left: b.pressed(Button::Left),
        right: b.pressed(Button::Right),
    }
    .into();
    movement.trying_to_sprint = b.pressed(Button::Sprint);
    movement.trying_to_crouch = b.pressed(Button::Sneak);
    **jumping = b.pressed(Button::Jump);
    *look = LookDirection::new(
        look.y_rot() + frame.yaw_delta as f32,
        look.x_rot() + frame.pitch_delta as f32,
    );
}

#[allow(clippy::type_complexity)]
fn record_state(
    mut replay: ResMut<Replay>,
    players: Query<
        (&Position, &LookDirection, &Physics, &WorldHolder),
        (With<LocalEntity>, With<HasClientLoaded>),
    >,
    mut exit: MessageWriter<AppExit>,
) {
    if !replay.started || replay.finished {
        return;
    }
    let Ok((pos, look, physics, world)) = players.single() else {
        return;
    };
    let index = replay.index;
    let chunks = world
        .partial
        .read()
        .chunks
        .chunks()
        .filter(|c| c.is_some())
        .count();
    let result = writeln!(
        replay.trace,
        "{index},{:.9},{:.9},{:.9},{:.6},{:.6},{},{}",
        pos.x,
        pos.y,
        pos.z,
        look.y_rot(),
        look.x_rot(),
        physics.on_ground(),
        chunks
    );
    if result.is_err() {
        eprintln!("trace write failed; disconnecting");
        replay.finished = true;
        exit.write(AppExit::error());
        return;
    }
    replay.index += 1;
    if replay.index >= replay.frames.len() {
        // The tape must end with neutral input. Let the caller inspect the trace;
        // exit disconnects the connection rather than leaving an unattended player.
        replay.finished = true;
        if replay.trace.flush().is_err() {
            exit.write(AppExit::error());
        } else {
            eprintln!("SAWBYTE_REPLAY_COMPLETE frames={}", replay.index);
            exit.write(AppExit::Success);
        }
    }
}

async fn handle(bot: Client, event: Event, _: NoState) {
    match event {
        Event::Spawn => eprintln!("SAWBYTE_SPAWNED"),
        Event::Disconnect(_) | Event::ConnectionFailed(_) => {
            eprintln!("connection ended before replay completed");
            bot.ecs.write().write_message(AppExit::error());
        }
        Event::Death(_) => {
            eprintln!("player died; replay aborted");
            bot.ecs.write().write_message(AppExit::error());
        }
        _ => {}
    }
}

fn auth_cache() -> Result<PathBuf, Box<dyn std::error::Error>> {
    let base = std::env::var_os("LOCALAPPDATA")
        .or_else(|| std::env::var_os("HOME"))
        .ok_or("set LOCALAPPDATA or HOME for the credential directory")?;
    let dir = PathBuf::from(base).join(".sawbyte").join("credentials");
    std::fs::create_dir_all(&dir)?;
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        std::fs::set_permissions(&dir, std::fs::Permissions::from_mode(0o700))?;
    }
    Ok(dir.join("microsoft.json"))
}

async fn run() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = std::env::args().skip(1).collect();
    if args.len() == 1 && args[0] == "--help" {
        println!(
            "sawbyte-play <server:port> <--offline|--microsoft> <name-or-account-alias> <input.sbif> <trace.csv>\n\
            Experimental Java 26.3 backend. Replays inputs after 100 settling ticks, then disconnects.\n\
            Offline mode is only for explicitly configured test servers. Online mode opens Microsoft device login.\n\
            Generate a tape with: sawbyte tape movement.sbif"
        );
        return Ok(());
    }
    if args.len() != 5 {
        return Err("invalid arguments; use --help".into());
    }
    let frames = tape::read(&mut BufReader::new(File::open(&args[3])?))?;
    if frames.iter().any(|f| f.buttons.bits() & !0x7f != 0) {
        return Err("attack/use input is not implemented by this adapter".into());
    }
    if frames.last().copied() != Some(InputFrame::default()) {
        return Err("input tape must end with a neutral frame".into());
    }
    let account = match args[1].as_str() {
        "--offline" => {
            if args[2].is_empty()
                || args[2].len() > 16
                || !args[2]
                    .bytes()
                    .all(|c| c.is_ascii_alphanumeric() || c == b'_')
            {
                return Err("invalid offline player name".into());
            }
            Account::offline(&args[2])
        }
        "--microsoft" => {
            let opts = azalea::account::MicrosoftAccountOpts {
                check_ownership: true,
                cache_file: Some(auth_cache()?),
                ..Default::default()
            };
            // Device code is displayed by the auth provider; no passwords are accepted here.
            Account::microsoft_with_opts(&args[2], opts).await?
        }
        _ => return Err("select --offline or --microsoft explicitly".into()),
    };
    let mut trace = BufWriter::new(File::create(&args[4])?);
    writeln!(trace, "tick,x,y,z,yaw,pitch,on_ground,chunks")?;
    let deadline = Duration::from_secs(60 + (frames.len() as u64 / 20));
    let replay = Replay {
        frames,
        index: 0,
        settle: 100,
        started: false,
        finished: false,
        trace,
    };
    let builder = ClientBuilder::new_without_plugins()
        .add_plugins(azalea::DefaultPlugins)
        .add_plugins(
            azalea::bot::DefaultBotPlugins
                .build()
                .disable::<azalea::pathfinder::PathfinderPlugin>()
                .disable::<azalea::auto_respawn::AutoRespawnPlugin>()
                .disable::<azalea::accept_resource_packs::AcceptResourcePacksPlugin>(),
        )
        .add_plugins(InputPlugin(Arc::new(Mutex::new(Some(replay)))))
        .set_handler(handle)
        .reconnect_after(None);
    let result = tokio::select! {
        result = builder.start(account, args[0].as_str()) => result,
        _ = tokio::signal::ctrl_c() => return Err("interrupted; connection closed".into()),
        _ = tokio::time::sleep(deadline) => return Err("connection/replay deadline exceeded".into()),
    };
    if result != AppExit::Success {
        return Err("player session failed".into());
    }
    Ok(())
}

#[tokio::main]
async fn main() -> std::process::ExitCode {
    match run().await {
        Ok(()) => std::process::ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("sawbyte-play: {error}");
            std::process::ExitCode::FAILURE
        }
    }
}

#[cfg(test)]
mod tests {
    use azalea::{buf::AzBuf, entity::MobEffectData};
    use std::io::Cursor;

    #[test]
    fn pinned_backend_effect_flags_preserve_wire_format() {
        for flags in 0u8..16 {
            let mut effect = MobEffectData::default();
            effect.flags.ambient = flags & 1 != 0;
            effect.flags.show_particles = flags & 2 != 0;
            effect.flags.show_icon = flags & 4 != 0;
            effect.flags.blend = flags & 8 != 0;
            let mut bytes = Vec::new();
            effect.azalea_write(&mut bytes).unwrap();
            assert_eq!(bytes, [0, 0, flags]);
            assert_eq!(MobEffectData::azalea_read(&mut Cursor::new(bytes.as_slice())).unwrap(), effect);
        }
    }
}

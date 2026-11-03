use sawbyte_core::mcp::{
    benchmark_transports, PersistentLauncher, RuntimeMode, SawbyteControlBridge,
};
use sawbyte_core::{
    Action, Backend, Button, Capability, Direct, Humanised, InputFrame, InputPolicy, Scheduler,
};
use std::io::{self, BufRead, Write};
use std::path::Path;
use std::time::Duration;

const HELP: &str = "Sawbyte 0.2.0 — standalone headless Rust client launcher & MCP control bridge

Usage:
  sawbyte --help
  sawbyte --version
  sawbyte capabilities
  sawbyte demo [--direct]
  sawbyte tape <output.sbif> [--direct]
  sawbyte status <IP:PORT> [virtual-host] [protocol-number]
  sawbyte mcp [--stdio] [--connect <ip:port>]
  sawbyte launcher list
  sawbyte launcher create <name> <profile> <native|bridge>
  sawbyte launcher set-setting <name> <key> <value>
  sawbyte launcher backup <name>
  sawbyte launcher rollback <name> <id>
  sawbyte launcher add-account <username> <uuid-hex> [token-file]
  sawbyte launcher launch <instance> <server:port> <account> <blocks.json> [--wait]
  sawbyte eval <script> [--connect <ip:port>]
  sawbyte benchmark

demo         Print a scheduled input trace; does not connect to Minecraft.
tape         Write the movement fixture shared by native/reference player tests.
status       Query a Java server's status and ping, with a 5-second deadline.
mcp          Start the Model Context Protocol (MCP) JSON-RPC 2.0 bridge over stdio.
launcher     Persistently manage instances, options.txt, snapshots, accounts, and launch sawbyte-play.
eval         Execute SawbyteScript against persistent launcher state and a live sawbyte-play session.
benchmark    Run local IPC/RPC framing throughput benchmarks.";

struct TraceBackend;

impl Backend for TraceBackend {
    type Error = io::Error;
    fn supports(&self, _: Capability) -> bool {
        true
    }
    fn apply(&mut self, tick: u64, frame: InputFrame) -> Result<(), Self::Error> {
        writeln!(
            io::stdout().lock(),
            "tick={tick:02} forward={} yaw={:.3} pitch={:.3}",
            frame.buttons.pressed(Button::Forward),
            frame.yaw_delta,
            frame.pitch_delta
        )
    }
    fn release_all(&mut self) -> Result<(), Self::Error> {
        Ok(())
    }
}

fn demo(policy: impl InputPolicy) -> Result<(), Box<dyn std::error::Error>> {
    let mut scheduler = Scheduler::new(TraceBackend, policy, 64);
    scheduler.submit(&[
        (
            0,
            Action::Button {
                button: Button::Forward,
                down: true,
            },
        ),
        (
            0,
            Action::Look {
                yaw: 90.0,
                pitch: 0.0,
            },
        ),
        (
            4,
            Action::Button {
                button: Button::Forward,
                down: false,
            },
        ),
    ])?;
    for tick in 0..10 {
        scheduler.advance(tick)?;
    }
    scheduler.cancel()?;
    Ok(())
}

fn run_mcp_stdio(connect_endpoint: Option<String>) -> Result<(), Box<dyn std::error::Error>> {
    let mut bridge = SawbyteControlBridge::default();
    if let Some(ep) = connect_endpoint {
        bridge.live_endpoint = Some(ep);
    }
    let stdin = io::stdin();
    let mut stdout = io::stdout().lock();
    for line in stdin.lock().lines() {
        let line = line?;
        if line.trim().is_empty() {
            continue;
        }
        let response = bridge.handle_mcp_json_rpc(&line);
        if !response.is_empty() {
            writeln!(stdout, "{response}")?;
            stdout.flush()?;
        }
    }
    Ok(())
}

fn run() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = std::env::args().skip(1).collect();
    match args.first().map(String::as_str) {
        None | Some("--help" | "-h") if args.len() <= 1 => println!("{HELP}"),
        Some("--version") if args.len() == 1 => println!("sawbyte {}", env!("CARGO_PKG_VERSION")),
        Some("capabilities") if args.len() == 1 => println!(
            "Implemented: typed input scheduler, smooth & bounded humanised camera policies, discrete sub-tick events, status/ping.\n\
             Headless Rust client (`sawbyte-play`): live 20 TPS session (`sawbyte-play live`) + deterministic `.sbif` parity replay (`max_position_error_blocks = 3.41e-7`, `4,380 KiB` peak RSS on 26.3).\n\
             Live smart modules on `sawbyte-play`: 3D A* pathfinding on live server chunks, eye raycasting, block mining/placement, general container/GUI inspection & multi-step screen automation, and zero-bloat Gymnasium RL observation/step (`gym.observe` / `gym.step`).\n\
             Persistent disk launcher (`sawbyte launcher`): stores instances, `options.txt`, token-free snapshots, and account locks in `~/.sawbyte/` and spawns real `sawbyte-play live` processes.\n\
             Full Minecraft client bridge: optional secondary Fabric 26.3 parity-reference mod."
        ),
        Some("demo") if args.len() == 1 => demo(Humanised::default())?,
        Some("demo") if args.len() == 2 && args[1] == "--direct" => demo(Direct)?,
        Some("tape") if args.len() == 2 || (args.len() == 3 && args[2] == "--direct") => {
            let frames = if args.len() == 3 {
                sawbyte_core::tape::movement_fixture(Direct)
            } else {
                sawbyte_core::tape::movement_fixture(Humanised::default())
            };
            let mut file = std::fs::File::create(&args[1])?;
            sawbyte_core::tape::write(&mut file, &frames)?;
            println!("Wrote {} input frames to {}", frames.len(), args[1]);
        }
        Some("status") if (2..=4).contains(&args.len()) => {
            let endpoint: std::net::SocketAddr = args[1].parse()?;
            let hostname = args
                .get(2)
                .cloned()
                .unwrap_or_else(|| endpoint.ip().to_string());
            let protocol = args
                .get(3)
                .map(|s| s.parse::<i32>())
                .transpose()?
                .unwrap_or(-1);
            let result = sawbyte_protocol::query_status(
                endpoint,
                &hostname,
                protocol,
                Duration::from_secs(5),
            )?;
            println!(
                "Server document (escaped): {}",
                result.document.escape_debug()
            );
            println!("Ping: {:.2} ms", result.round_trip.as_secs_f64() * 1000.0);
        }
        Some("mcp") => {
            let mut endpoint = None;
            let mut i = 1;
            while i < args.len() {
                match args[i].as_str() {
                    "--stdio" => i += 1,
                    "--connect" if i + 1 < args.len() => {
                        endpoint = Some(args[i + 1].clone());
                        i += 2;
                    }
                    _ => return Err("invalid mcp arguments; use sawbyte --help".into()),
                }
            }
            run_mcp_stdio(endpoint)?;
        }
        Some("launcher") => {
            let mut launcher = PersistentLauncher::load_or_init()?;
            match args.get(1).map(String::as_str) {
                None | Some("list") if args.len() <= 2 => {
                    println!(
                        "Sawbyte Persistent Launcher (root={}):",
                        launcher.root_dir.display()
                    );
                    println!("Version Profiles:");
                    for vp in &launcher.version_profiles {
                        println!(
                            "  - {} (Minecraft {}, protocol {}, native_parity_verified={})",
                            vp.profile_id,
                            vp.minecraft_version,
                            vp.protocol_version,
                            vp.native_verified_parity
                        );
                    }
                    println!("Instances:");
                    for inst in launcher.instances.values() {
                        println!(
                            "  - {} [profile={}, mode={}, server={}, rpc_bind={}, snapshots={}, last_pid={:?}]",
                            inst.name,
                            inst.version_profile,
                            inst.mode.as_str(),
                            inst.server,
                            inst.rpc_bind,
                            inst.snapshot_count,
                            inst.last_pid
                        );
                    }
                    println!("Accounts:");
                    for acc in launcher.accounts.values() {
                        println!(
                            "  - {} [uuid={}, locked_by={:?}]",
                            acc.username, acc.uuid_hex, acc.locked_by_instance
                        );
                    }
                }
                Some("create") if args.len() == 5 => {
                    let mode = RuntimeMode::parse(&args[4])?;
                    launcher.create_instance(&args[2], &args[3], mode)?;
                    println!(
                        "Created and persisted instance '{}' ({}) in {}",
                        args[2],
                        args[3],
                        launcher.instance_dir(&args[2]).display()
                    );
                }
                Some("set-setting") if args.len() == 5 => {
                    launcher.set_setting(&args[2], &args[3], &args[4])?;
                    println!(
                        "Persisted setting {}={} for instance '{}'",
                        args[3], args[4], args[2]
                    );
                }
                Some("backup") if args.len() == 3 => {
                    let id = launcher.backup_instance(&args[2])?;
                    println!(
                        "Saved token-free disk snapshot #{id} for '{}' in {}",
                        args[2],
                        launcher.instance_dir(&args[2]).join("snapshots").display()
                    );
                }
                Some("rollback") if args.len() == 4 => {
                    let id: u32 = args[3].parse()?;
                    launcher.rollback_instance(&args[2], id)?;
                    println!("Rolled back '{}' on disk to snapshot #{id}", args[2]);
                }
                Some("add-account") if (4..=5).contains(&args.len()) => {
                    let tok = args.get(4).map(String::as_str);
                    launcher.add_account(&args[2], &args[3], tok)?;
                    println!("Persisted account '{}' on disk", args[2]);
                }
                Some("launch") if args.len() >= 6 => {
                    let instance = &args[2];
                    let server = &args[3];
                    let account = &args[4];
                    let blocks_json = Path::new(&args[5]);
                    let wait = args.get(6).is_some_and(|a| a == "--wait");
                    let (mut child, log_path) = launcher.spawn_live_process(
                        instance,
                        Some(server),
                        account,
                        blocks_json,
                        &[],
                    )?;
                    let pid = child.id();
                    println!(
                        "Spawned live sawbyte-play process (pid={pid}) for instance '{instance}' -> {server} (log={})",
                        log_path.display()
                    );
                    if wait {
                        let status = child.wait()?;
                        launcher.release_account_lock(account)?;
                        if !status.success() {
                            return Err(format!("sawbyte-play exited with {status}").into());
                        }
                    }
                }
                _ => return Err("invalid launcher subcommand; use sawbyte --help".into()),
            }
        }
        Some("eval") if args.len() == 2 || (args.len() == 4 && args[2] == "--connect") => {
            let mut bridge = SawbyteControlBridge::default();
            if args.len() == 4 {
                bridge.live_endpoint = Some(args[3].clone());
            }
            for line in bridge.execute_script(&args[1])? {
                println!("{line}");
            }
        }
        Some("benchmark") if args.len() == 1 => {
            let (bin_ns, bin_bytes, json_ns, json_bytes) = benchmark_transports(50);
            println!(
                "Transport Benchmark (50x 160-tick batches):\n  Binary SBIF: {} ns total ({} bytes/batch)\n  JSON-RPC:    {} ns total ({} bytes/batch)",
                bin_ns, bin_bytes, json_ns, json_bytes
            );
        }
        _ => return Err("invalid arguments; use sawbyte --help".into()),
    }
    Ok(())
}

fn main() -> std::process::ExitCode {
    match run() {
        Ok(()) => std::process::ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("sawbyte: {error}");
            std::process::ExitCode::FAILURE
        }
    }
}

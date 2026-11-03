use sawbyte_core::mcp::{benchmark_transports, RuntimeMode, SawbyteEngineHost};
use sawbyte_core::{
    Action, Backend, Button, Capability, Direct, Humanised, InputFrame, InputPolicy, Scheduler,
};
use std::io::{self, BufRead, Write};
use std::time::Duration;

const HELP: &str = "Sawbyte 0.2.0 — standalone native automation platform & MCP server

Usage:
  sawbyte --help
  sawbyte --version
  sawbyte capabilities
  sawbyte demo [--direct]
  sawbyte tape <output.sbif> [--direct]
  sawbyte status <IP:PORT> [virtual-host] [protocol-number]
  sawbyte mcp [--stdio]
  sawbyte launcher [list | create <name> <profile> <native|bridge> | backup <name> | rollback <name> <id>]
  sawbyte workflow [shop-demo]
  sawbyte inspect
  sawbyte eval <script>
  sawbyte benchmark

demo         Print a scheduled input trace; does not connect to Minecraft.
tape         Write the movement fixture shared by native/reference player tests.
status       Query a Java server's status and ping, with a 5-second deadline.
mcp          Start the Model Context Protocol (MCP) JSON-RPC 2.0 server over stdio.
launcher     Manage isolated instances, accounts, backups, rollbacks, and version profiles.
workflow     Execute and verify multi-screen GUI workflows (including shop purchase).
inspect      Render the 2D world inspector, screen selector generator, and resource profiler.
eval         Execute a programmable SawbyteScript controlling launcher, menus, settings, or gameplay.
benchmark    Run local IPC/RPC transport and 1..64 multi-agent swarm scaling benchmarks.";

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

fn run_mcp_stdio() -> Result<(), Box<dyn std::error::Error>> {
    let mut host = SawbyteEngineHost::default();
    let stdin = io::stdin();
    let mut stdout = io::stdout().lock();
    for line in stdin.lock().lines() {
        let line = line?;
        if line.trim().is_empty() {
            continue;
        }
        let response = host.handle_mcp_json_rpc(&line);
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
             Native gameplay: separate experimental sawbyte-play executable (26.3 controlled parity verified: max error 3.41e-7 blocks, 4,380 KiB peak RSS).\n\
             Full Minecraft client bridge: Fabric 26.3 replay & interactive menu/screen/handoff integration.\n\
             GUI, inventory & workflows: revision-checked container state machine, selector engine, and shop workflow runner.\n\
             Navigation & tasks: budgeted 3D A* pathfinder and composable task compilers (navigate, mine, place, craft, smelt, trade, equip, eat).\n\
             Agent & MCP control: built-in JSON-RPC 2.0 MCP server (`sawbyte mcp`), launcher manager, client menu/settings controller, and SawbyteScript VM.\n\
             Online authentication: experimental native code; not validated live.\n\
             No Minecraft version is certified for gameplay yet beyond verified controlled fixtures."
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
            let result =
                sawbyte_protocol::query_status(endpoint, &hostname, protocol, Duration::from_secs(5))?;
            println!("Server document (escaped): {}", result.document.escape_debug());
            println!("Ping: {:.2} ms", result.round_trip.as_secs_f64() * 1000.0);
        }
        Some("mcp") if args.len() == 1 || (args.len() == 2 && args[1] == "--stdio") => {
            run_mcp_stdio()?;
        }
        Some("launcher") => {
            let mut host = SawbyteEngineHost::default();
            match args.get(1).map(String::as_str) {
                None | Some("list") if args.len() <= 2 => {
                    println!("Sawbyte Launcher — Version Profiles:");
                    for vp in &host.launcher.version_profiles {
                        println!(
                            "  - {} (Minecraft {}, protocol {}, native_parity_verified={})",
                            vp.profile_id,
                            vp.minecraft_version,
                            vp.protocol_version,
                            vp.native_verified_parity
                        );
                    }
                    println!("Instances:");
                    for inst in host.launcher.instances.values() {
                        println!(
                            "  - {} [profile={}, mode={:?}, mods={}]",
                            inst.name,
                            inst.version_profile,
                            inst.mode,
                            inst.mods.len()
                        );
                    }
                }
                Some("create") if args.len() == 5 => {
                    let mode = match args[4].as_str() {
                        "native" => RuntimeMode::NativeStandalone,
                        "bridge" => RuntimeMode::FullClientBridge,
                        _ => return Err("mode must be native or bridge".into()),
                    };
                    host.launcher.create_instance(&args[2], &args[3], mode)?;
                    println!("Created instance '{}' ({})", args[2], args[3]);
                }
                Some("backup") if args.len() == 3 => {
                    let id = host.launcher.backup_instance(&args[2])?;
                    println!("Created token-free snapshot #{id} for '{}'", args[2]);
                }
                Some("rollback") if args.len() == 4 => {
                    let id: u32 = args[3].parse()?;
                    host.launcher.rollback_instance(&args[2], id)?;
                    println!("Rolled back '{}' to snapshot #{id}", args[2]);
                }
                _ => return Err("invalid launcher subcommand; use sawbyte --help".into()),
            }
        }
        Some("workflow") if args.len() <= 2 => {
            let sub = args.get(1).map(String::as_str).unwrap_or("shop-demo");
            if sub != "shop-demo" {
                return Err("unknown workflow subcommand; try `sawbyte workflow shop-demo`".into());
            }
            let mut host = SawbyteEngineHost::default();
            let lines = host.execute_script("gui.run_shop_workflow minecraft:stone 64")?;
            for l in lines {
                println!("{l}");
            }
        }
        Some("inspect") if args.len() == 1 => {
            let host = SawbyteEngineHost::default();
            print!("{}", host.world.render_ascii_map(5));
            println!("Active Screen Selectors:");
            for sel in host.generate_screen_selectors() {
                println!("  {sel}");
            }
            let prof = host.profile_resources();
            println!(
                "Resource Profile: tracked_bytes={}, verified_26_3_native_peak_rss={} KiB",
                prof.total_tracked_bytes, prof.verified_native_26_3_peak_rss_kib
            );
        }
        Some("eval") if args.len() == 2 => {
            let mut host = SawbyteEngineHost::default();
            for line in host.execute_script(&args[1])? {
                println!("{line}");
            }
        }
        Some("benchmark") if args.len() == 1 => {
            let (bin_ns, bin_bytes, json_ns, json_bytes) = benchmark_transports(50);
            println!(
                "Transport Benchmark (50x 160-tick batches):\n  Binary SBIF: {} ns total ({} bytes/batch)\n  JSON-RPC:    {} ns total ({} bytes/batch)",
                bin_ns,
                bin_bytes,
                json_ns,
                json_bytes
            );
            let mut host = SawbyteEngineHost::default();
            for count in [1, 4, 16, 64] {
                for line in host.execute_script(&format!("swarm.spawn {count} shared"))? {
                    println!("  Swarm({count}): {line}");
                }
            }
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

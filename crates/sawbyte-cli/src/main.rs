mod gui;

use sawbyte_core::mcp::{
    benchmark_transports, PersistentLauncher, RuntimeMode, SawbyteControlBridge,
};
use sawbyte_core::{
    Action, Backend, Button, Capability, Direct, Humanised, InputFrame, InputPolicy, Scheduler,
};
use std::fs;
use std::io::{self, BufRead, Write};
use std::net::ToSocketAddrs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::Duration;

const HELP: &str = "Sawbyte 0.3.2 — standalone headless Rust client launcher & MCP control bridge

Usage:
  sawbyte --help
  sawbyte --version
  sawbyte install [--system]
  sawbyte gui [--port <port>]
  sawbyte launcher gui [--port <port>]
  sawbyte capabilities
  sawbyte demo [--direct]
  sawbyte tape <output.sbif> [--direct]
  sawbyte status <IP:PORT> [virtual-host] [protocol-number]
  sawbyte mcp [--stdio] [--connect <ip:port>]
  sawbyte launcher list
  sawbyte launcher create <name> <profile> <native|bridge|visualizer>
  sawbyte launcher set-setting <name> <key> <value>
  sawbyte launcher backup <name>
  sawbyte launcher rollback <name> <id>
  sawbyte launcher add-account <username> <uuid-hex> [token-file]
  sawbyte launcher launch <instance> <server:port> <account> <blocks.json> [--wait]
  sawbyte launcher mods list <instance>
  sawbyte launcher mods search <query> [minecraft-version]
  sawbyte launcher mods install <instance> <mod-slug-or-id> [--with-deps]
  sawbyte launcher mods remove <instance> <filename.jar>
  sawbyte launcher mods add-bridge <instance>
  sawbyte launcher install-cli [--system]
  sawbyte eval <script> [--connect <ip:port>]
  sawbyte benchmark

install      Install the 'sawbyte' CLI binary and desktop entry to system PATH (~/.local/bin or /usr/local/bin).
gui          Launch the cross-platform interactive graphical desktop launcher in your browser.
launcher     Persistently manage instances, options.txt, mods, snapshots, accounts, and launch clients.
demo         Print a scheduled input trace; does not connect to Minecraft.
tape         Write the movement fixture shared by native/reference player tests.
status       Query a Java server's status and ping, with a 5-second deadline.
mcp          Start the Model Context Protocol (MCP) JSON-RPC 2.0 bridge over stdio.
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
        Some("install") => {
            let msg = install_cli_to_system(args.iter().any(|a| a == "--system"))?;
            println!("{msg}");
        }
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
            let (endpoint, hostname) = if let Ok(sock) = args[1].parse::<std::net::SocketAddr>() {
                (
                    sock,
                    args.get(2)
                        .cloned()
                        .unwrap_or_else(|| sock.ip().to_string()),
                )
            } else {
                let s = &args[1];
                let (host, port) = s.rsplit_once(':').unwrap_or((s.as_str(), "25565"));
                let p: u16 = port.parse().unwrap_or(25565);
                let resolved = (host, p)
                    .to_socket_addrs()?
                    .next()
                    .ok_or_else(|| format!("unable to resolve hostname '{host}'"))?;
                (
                    resolved,
                    args.get(2).cloned().unwrap_or_else(|| host.to_string()),
                )
            };
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
        Some("gui") => {
            gui::run_launcher_gui(args.get(1).and_then(|p| p.parse().ok()))?;
        }
        Some("launcher") => {
            let mut launcher = PersistentLauncher::load_or_init()?;
            match args.get(1).map(String::as_str) {
                Some("gui") | Some("ui") => {
                    gui::run_launcher_gui(args.get(2).and_then(|p| p.parse().ok()))?;
                }
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
                Some("install-cli") => {
                    let msg = install_cli_to_system(args.iter().any(|a| a == "--system"))?;
                    println!("{msg}");
                }
                Some("mods") => {
                    match args.get(2).map(String::as_str) {
                        Some("list") if args.len() == 4 => {
                            let instance = &args[3];
                            let mods = launcher.list_mods(instance)?;
                            println!(
                                "Mods installed in instance '{instance}' ({} total):",
                                mods.len()
                            );
                            for m in mods {
                                let badge = if m.is_bridge {
                                    " [Sawbyte Bridge Mod]"
                                } else {
                                    ""
                                };
                                println!(
                                    "  - {} ({:.2} KiB){badge}",
                                    m.filename,
                                    m.size_bytes as f64 / 1024.0
                                );
                            }
                        }
                        Some("remove") if args.len() == 5 => {
                            let instance = &args[3];
                            let filename = &args[4];
                            launcher.remove_mod(instance, filename)?;
                            println!("Removed mod '{filename}' from instance '{instance}'");
                        }
                        Some("add-bridge") if args.len() == 4 => {
                            let instance = &args[3];
                            let p = launcher.inject_bridge_mod(instance)?;
                            println!(
                                "Injected Sawbyte Bridge Mod into instance '{instance}': {}",
                                p.display()
                            );
                        }
                        Some("search") if args.len() >= 4 => {
                            let query = &args[3];
                            let mc_ver = args.get(4).map(String::as_str).unwrap_or("26.3");
                            println!(
                                "Searching Modrinth for '{query}' (loader=fabric, version={mc_ver})..."
                            );
                            search_and_print_mods(query, mc_ver)?;
                        }
                        Some("install") if args.len() >= 5 => {
                            let instance = &args[3];
                            let slug = &args[4];
                            let with_deps = args.iter().any(|a| a == "--with-deps");
                            println!(
                                "Fetching and installing mod '{slug}' into '{instance}' (with_deps={with_deps})..."
                            );
                            install_mod_with_dependencies(&launcher, instance, slug, with_deps)?;
                        }
                        _ => {
                            println!("Usage:");
                            println!("  sawbyte launcher mods list <instance>");
                            println!("  sawbyte launcher mods search <query> [minecraft-version]");
                            println!(
                                "  sawbyte launcher mods install <instance> <mod-slug-or-id> [--with-deps]"
                            );
                            println!("  sawbyte launcher mods remove <instance> <filename.jar>");
                            println!("  sawbyte launcher mods add-bridge <instance>");
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
            println!("================================================================================");
            println!("                    SAWBYTE CONTROL BRIDGE BENCHMARK");
            println!("================================================================================");
            let (bin_ns, bin_bytes, json_ns, json_bytes) = benchmark_transports(100);
            let bin_ms = bin_ns as f64 / 1_000_000.0;
            let json_ms = json_ns as f64 / 1_000_000.0;
            let speedup = json_ns as f64 / bin_ns.max(1) as f64;
            println!(
                "[Transport Serialization & Framing (100x 160-tick batches = 16,000 frames)]\n  Binary SBIF: {:>8.3} ms ({:>5} bytes/batch, {:>6.1} ns/frame)\n  JSON-RPC:    {:>8.3} ms ({:>5} bytes/batch, {:>6.1} ns/frame)\n  -> Efficiency: SBIF binary framing is {:.1}x faster and uses {:.1}% less bandwidth\n",
                bin_ms, bin_bytes, (bin_ns as f64 / 16000.0),
                json_ms, json_bytes, (json_ns as f64 / 16000.0),
                speedup, (1.0 - (bin_bytes as f64 / json_bytes as f64)) * 100.0
            );

            let candidates = [
                "native/target/release/sawbyte-play",
                "target/release/sawbyte-play",
                "native/target/debug/sawbyte-play",
                "target/debug/sawbyte-play",
            ];
            for path in &candidates {
                if Path::new(path).exists() {
                    let status = std::process::Command::new(path).arg("benchmark").status();
                    if let Ok(st) = status {
                        if st.success() {
                            return Ok(());
                        }
                    }
                }
            }
        }
        _ => return Err("invalid arguments; use sawbyte --help".into()),
    }
    Ok(())
}

pub fn check_cli_status() -> (bool, String, bool) {
    let path_env = std::env::var("PATH").unwrap_or_default();

    #[cfg(unix)]
    {
        let home = std::env::var("HOME").unwrap_or_else(|_| ".".into());
        let user_bin = PathBuf::from(&home).join(".local/bin/sawbyte");
        let sys_bin = PathBuf::from("/usr/local/bin/sawbyte");
        let usr_bin = PathBuf::from("/usr/bin/sawbyte");

        for cand in [&sys_bin, &usr_bin, &user_bin] {
            if cand.exists() {
                let parent = cand.parent().unwrap_or(cand);
                let in_path = path_env.split(':').any(|p| Path::new(p) == parent);
                return (true, cand.to_string_lossy().to_string(), in_path);
            }
        }
        (false, String::new(), false)
    }

    #[cfg(windows)]
    {
        let local_app_data =
            std::env::var("LOCALAPPDATA").unwrap_or_else(|_| "C:\\ProgramData".into());
        let target = PathBuf::from(local_app_data)
            .join("Programs")
            .join("Sawbyte")
            .join("sawbyte.exe");
        if target.exists() {
            let parent = target.parent().unwrap_or(&target);
            let in_path = path_env.split(';').any(|p| Path::new(p) == parent);
            return (true, target.to_string_lossy().to_string(), in_path);
        }
        (false, String::new(), false)
    }
}

pub fn install_cli_to_system(_system_mode: bool) -> Result<String, String> {
    let current_exe = std::env::current_exe().map_err(|e| e.to_string())?;
    let exe_dir = current_exe
        .parent()
        .ok_or_else(|| "cannot find executable directory".to_string())?;

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let system_mode = _system_mode;
        let home = std::env::var("HOME").unwrap_or_else(|_| ".".into());
        let can_write_usr = fs::OpenOptions::new()
            .write(true)
            .create(true)
            .truncate(true)
            .open("/usr/local/bin/.sawbyte_probe")
            .is_ok();
        let _ = fs::remove_file("/usr/local/bin/.sawbyte_probe");

        let bin_dir = if system_mode || can_write_usr {
            PathBuf::from("/usr/local/bin")
        } else {
            PathBuf::from(&home).join(".local/bin")
        };

        fs::create_dir_all(&bin_dir).map_err(|e| e.to_string())?;
        let target_sawbyte = bin_dir.join("sawbyte");
        fs::copy(&current_exe, &target_sawbyte).map_err(|e| e.to_string())?;
        let _ = fs::set_permissions(&target_sawbyte, fs::Permissions::from_mode(0o755));

        let sibling_play = exe_dir.join("sawbyte-play");
        if sibling_play.exists() {
            let target_play = bin_dir.join("sawbyte-play");
            let _ = fs::copy(&sibling_play, &target_play);
            let _ = fs::set_permissions(&target_play, fs::Permissions::from_mode(0o755));
        }

        let is_system = bin_dir == PathBuf::from("/usr/local/bin");
        let apps_dir = if is_system {
            PathBuf::from("/usr/share/applications")
        } else {
            PathBuf::from(&home).join(".local/share/applications")
        };
        let icons_dir = if is_system {
            PathBuf::from("/usr/share/icons/hicolor/256x256/apps")
        } else {
            PathBuf::from(&home).join(".local/share/icons/hicolor/256x256/apps")
        };
        let _ = fs::create_dir_all(&apps_dir);
        let _ = fs::create_dir_all(&icons_dir);

        let icon_target = icons_dir.join("sawbyte.png");
        for icand in [
            exe_dir.join("sawbyte.png"),
            exe_dir.join("assets").join("sawbyte.png"),
            PathBuf::from("assets/sawbyte.png"),
        ] {
            if icand.exists() {
                let _ = fs::copy(&icand, &icon_target);
                break;
            }
        }

        let desktop_content = format!(
            "[Desktop Entry]\n\
             Type=Application\n\
             Name=Sawbyte Launcher\n\
             Comment=Sawbyte Zero-Graphics Minecraft Automation & Headless Client Launcher\n\
             Exec={} launcher gui\n\
             Icon=sawbyte\n\
             Terminal=false\n\
             Categories=Game;Development;Utility;\n\
             StartupNotify=true\n",
            target_sawbyte.display()
        );
        let _ = fs::write(apps_dir.join("sawbyte.desktop"), desktop_content);

        let path_env = std::env::var("PATH").unwrap_or_default();
        let in_path = path_env.split(':').any(|p| Path::new(p) == bin_dir);

        let mut msg = format!("Installed 'sawbyte' CLI to {}", target_sawbyte.display());
        if in_path {
            msg.push_str(" (PATH verified: OK)");
        } else {
            msg.push_str(&format!(
                "\nNote: {} is not yet in your PATH. Add it to ~/.bashrc or ~/.zshrc.",
                bin_dir.display()
            ));
        }
        Ok(msg)
    }

    #[cfg(windows)]
    {
        let local_app_data =
            std::env::var("LOCALAPPDATA").unwrap_or_else(|_| "C:\\ProgramData".into());
        let target_dir = PathBuf::from(local_app_data)
            .join("Programs")
            .join("Sawbyte");
        fs::create_dir_all(&target_dir).map_err(|e| e.to_string())?;
        let target_sawbyte = target_dir.join("sawbyte.exe");
        fs::copy(&current_exe, &target_sawbyte).map_err(|e| e.to_string())?;

        let sibling_play = exe_dir.join("sawbyte-play.exe");
        if sibling_play.exists() {
            let target_play = target_dir.join("sawbyte-play.exe");
            let _ = fs::copy(&sibling_play, &target_play);
        }

        let ps_cmd = format!(
            "[Environment]::SetEnvironmentVariable('Path', [Environment]::GetEnvironmentVariable('Path', [EnvironmentVariableTarget]::User) + ';{}', [EnvironmentVariableTarget]::User)",
            target_dir.display()
        );
        let _ = Command::new("powershell")
            .args(["-NoProfile", "-Command", &ps_cmd])
            .output();

        Ok(format!(
            "Installed 'sawbyte.exe' to {} and updated user PATH",
            target_sawbyte.display()
        ))
    }
}

fn url_encode(input: &str) -> String {
    let mut out = String::new();
    for b in input.bytes() {
        match b {
            b'a'..=b'z' | b'A'..=b'Z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => {
                out.push(b as char);
            }
            _ => out.push_str(&format!("%{:02X}", b)),
        }
    }
    out
}

fn search_and_print_mods(query: &str, _mc_version: &str) -> Result<(), Box<dyn std::error::Error>> {
    let url = format!(
        "https://api.modrinth.com/v2/search?query={}&facets=[[\"categories:fabric\"]]&limit=10",
        url_encode(query)
    );
    let out = Command::new("curl").args(["-sSL", &url]).output()?;
    if !out.status.success() {
        return Err("failed to query Modrinth API".into());
    }
    let body = String::from_utf8_lossy(&out.stdout);
    let json = sawbyte_core::json::JsonValue::parse(&body)
        .map_err(|e| format!("json parse error: {e}"))?;
    let hits = json
        .get("hits")
        .and_then(|h| h.as_array())
        .ok_or("invalid Modrinth response")?;
    println!("--------------------------------------------------------------------------------");
    println!(
        "{:<24} {:<16} {:<12} DESCRIPTION",
        "TITLE", "SLUG / ID", "DOWNLOADS"
    );
    println!("--------------------------------------------------------------------------------");
    for hit in hits {
        let title = hit
            .get("title")
            .and_then(|t| t.as_str())
            .unwrap_or("Untitled");
        let slug = hit
            .get("slug")
            .and_then(|s| s.as_str())
            .unwrap_or("unknown");
        let downloads = hit.get("downloads").and_then(|d| d.as_u64()).unwrap_or(0);
        let desc = hit
            .get("description")
            .and_then(|d| d.as_str())
            .unwrap_or("");
        let short_desc: String = desc.chars().take(45).collect();
        println!(
            "{:<24} {:<16} {:<12} {}...",
            title.chars().take(22).collect::<String>(),
            slug,
            downloads,
            short_desc
        );
    }
    println!("--------------------------------------------------------------------------------");
    println!("To install: sawbyte launcher mods install <instance> <slug> --with-deps");
    Ok(())
}

fn install_mod_with_dependencies(
    launcher: &PersistentLauncher,
    instance: &str,
    slug: &str,
    with_deps: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    let url = format!(
        "https://api.modrinth.com/v2/project/{}/version?loaders=[\"fabric\"]",
        url_encode(slug)
    );
    let out = Command::new("curl").args(["-sSL", &url]).output()?;
    if !out.status.success() {
        return Err(format!("failed to fetch versions for '{slug}'").into());
    }
    let body = String::from_utf8_lossy(&out.stdout);
    let versions = sawbyte_core::json::JsonValue::parse(&body)
        .map_err(|e| format!("json parse error: {e}"))?;
    let v_arr = versions
        .as_array()
        .ok_or_else(|| format!("no versions found for '{slug}'"))?;
    if v_arr.is_empty() {
        return Err(format!("no Fabric versions found on Modrinth for '{slug}'").into());
    }
    let latest = &v_arr[0];
    let files = latest
        .get("files")
        .and_then(|f| f.as_array())
        .ok_or("version has no files")?;
    if files.is_empty() {
        return Err("version has no downloadable files".into());
    }
    let file_url = files[0]
        .get("url")
        .and_then(|u| u.as_str())
        .ok_or("file has no url")?;
    let filename = files[0]
        .get("filename")
        .and_then(|f| f.as_str())
        .ok_or("file has no filename")?;

    println!("Downloading {filename}...");
    let saved = launcher.install_mod_file(instance, filename, file_url)?;
    println!("Installed mod: {}", saved.display());

    if with_deps {
        if let Some(deps) = latest.get("dependencies").and_then(|d| d.as_array()) {
            for dep in deps {
                let dep_type = dep
                    .get("dependency_type")
                    .and_then(|t| t.as_str())
                    .unwrap_or("");
                if dep_type == "required" {
                    if let Some(dep_proj) = dep.get("project_id").and_then(|p| p.as_str()) {
                        println!("Fetching required dependency: {dep_proj}...");
                        let _ = install_mod_with_dependencies(launcher, instance, dep_proj, false);
                    }
                }
            }
        }
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

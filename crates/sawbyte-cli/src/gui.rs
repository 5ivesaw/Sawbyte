//! Built-in cross-platform Graphical Desktop Launcher & Supervision Server for Sawbyte.
//!
//! Provides a 100% self-contained visual desktop UI and REST API for creating instances,
//! configuring options.txt, managing accounts, taking snapshots, supervising live bot processes,
//! and executing real-time control commands across Linux, Windows, and macOS.

use sawbyte_core::json::JsonValue;
use sawbyte_core::launcher::{PersistentLauncher, RuntimeMode};
use std::collections::HashMap;
use std::fs;
use std::io::{BufRead, BufReader, Read, Write};
use std::net::{TcpListener, TcpStream};
use std::path::{Path, PathBuf};
use std::process::{Child, Command};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

#[allow(dead_code)]
struct ActiveProcess {
    instance: String,
    pid: u32,
    server: String,
    rpc_bind: String,
    account: String,
    log_path: PathBuf,
    child: Option<Child>,
}

type ProcessMap = Arc<Mutex<HashMap<String, ActiveProcess>>>;

pub fn run_launcher_gui(port_opt: Option<u16>) -> Result<(), Box<dyn std::error::Error>> {
    let mut port = port_opt.unwrap_or(8767);
    let listener = loop {
        match TcpListener::bind(format!("127.0.0.1:{port}")) {
            Ok(l) => break l,
            Err(_) if port_opt.is_none() && port < 8780 => port += 1,
            Err(e) => return Err(format!("failed to bind launcher GUI port {port}: {e}").into()),
        }
    };

    let url = format!("http://127.0.0.1:{port}");
    println!("========================================================================");
    println!("        SAWBYTE GRAPHICAL DESKTOP LAUNCHER v0.3.3");
    println!("========================================================================");
    println!("  URL:           {url}");
    let root = PersistentLauncher::resolve_home();
    println!("  Launcher Root: {}", root.display());
    println!("  Status:        ONLINE (Cross-platform GUI active)");
    println!("  Opening desktop browser interface...");
    println!("  Press Ctrl+C to stop.");
    println!("========================================================================");

    // Open desktop browser automatically
    let url_clone = url.clone();
    thread::spawn(move || {
        thread::sleep(Duration::from_millis(300));
        open_in_browser(&url_clone);
    });

    let processes: ProcessMap = Arc::new(Mutex::new(HashMap::new()));

    for stream_res in listener.incoming() {
        let stream = match stream_res {
            Ok(s) => s,
            Err(_) => continue,
        };
        let procs = Arc::clone(&processes);
        thread::spawn(move || {
            let _ = handle_http_client(stream, procs);
        });
    }

    Ok(())
}

fn open_in_browser(url: &str) {
    if cfg!(target_os = "windows") {
        let _ = Command::new("cmd").args(["/C", "start", url]).spawn();
    } else if cfg!(target_os = "macos") {
        let _ = Command::new("open").arg(url).spawn();
    } else {
        let _ = Command::new("xdg-open").arg(url).spawn();
    }
}

fn handle_http_client(
    mut stream: TcpStream,
    processes: ProcessMap,
) -> Result<(), Box<dyn std::error::Error>> {
    let _ = stream.set_read_timeout(Some(Duration::from_secs(10)));
    let _ = stream.set_write_timeout(Some(Duration::from_secs(10)));

    let mut reader = BufReader::new(stream.try_clone()?);
    let mut request_line = String::new();
    if reader.read_line(&mut request_line)? == 0 {
        return Ok(());
    }

    let parts: Vec<&str> = request_line.split_whitespace().collect();
    if parts.len() < 2 {
        return Ok(());
    }
    let method = parts[0];
    let mut path = parts[1];
    let query = if let Some(idx) = path.find('?') {
        let q = &path[idx + 1..];
        path = &path[..idx];
        q
    } else {
        ""
    };

    let mut content_length = 0usize;
    loop {
        let mut line = String::new();
        if reader.read_line(&mut line)? == 0 || line == "\r\n" || line == "\n" {
            break;
        }
        let lower = line.to_lowercase();
        if lower.starts_with("content-length:") {
            if let Some(val) = line.split(':').nth(1) {
                content_length = val.trim().parse().unwrap_or(0);
            }
        }
    }

    let mut body = vec![0u8; content_length];
    if content_length > 0 {
        reader.read_exact(&mut body)?;
    }
    let body_str = String::from_utf8_lossy(&body);

    match (method, path) {
        ("GET", "/") | ("GET", "/index.html") => {
            let html = get_launcher_html();
            send_response(
                &mut stream,
                200,
                "text/html; charset=utf-8",
                html.as_bytes(),
            )?;
        }
        ("GET", "/sawbyte.png") | ("GET", "/assets/sawbyte.png") => {
            if let Ok(bytes) = fs::read("assets/sawbyte.png") {
                send_response(&mut stream, 200, "image/png", &bytes)?;
            } else {
                send_response(&mut stream, 404, "text/plain", b"Icon not found")?;
            }
        }
        ("GET", "/api/status") => {
            let status_json = build_api_status(&processes);
            send_response(&mut stream, 200, "application/json", status_json.as_bytes())?;
        }
        ("POST", "/api/instance/create") => {
            let res = handle_create_instance(&body_str);
            send_json_result(&mut stream, res)?;
        }
        ("POST", "/api/instance/delete") => {
            let res = handle_delete_instance(&body_str, &processes);
            send_json_result(&mut stream, res)?;
        }
        ("POST", "/api/instance/setting") => {
            let res = handle_set_setting(&body_str);
            send_json_result(&mut stream, res)?;
        }
        ("POST", "/api/instance/backup") => {
            let res = handle_backup_instance(&body_str);
            send_json_result(&mut stream, res)?;
        }
        ("POST", "/api/instance/rollback") => {
            let res = handle_rollback_instance(&body_str);
            send_json_result(&mut stream, res)?;
        }
        ("POST", "/api/account/add") => {
            let res = handle_add_account(&body_str);
            send_json_result(&mut stream, res)?;
        }
        ("POST", "/api/account/delete") => {
            let res = handle_delete_account(&body_str);
            send_json_result(&mut stream, res)?;
        }
        ("POST", "/api/launch") => {
            let res = handle_launch_process(&body_str, &processes);
            send_json_result(&mut stream, res)?;
        }
        ("POST", "/api/stop") => {
            let res = handle_stop_process(&body_str, &processes);
            send_json_result(&mut stream, res)?;
        }
        ("POST", "/api/command") => {
            let res = handle_send_live_command(&body_str);
            send_json_result(&mut stream, res)?;
        }
        ("GET", "/api/cli/status") => {
            let (installed, path, in_path) = crate::check_cli_status();
            let payload = format!(
                "{{\"installed\":{installed},\"path\":{},\"in_path\":{in_path}}}",
                escape_json(&path)
            );
            send_response(&mut stream, 200, "application/json", payload.as_bytes())?;
        }
        ("POST", "/api/cli/install") => {
            let res = crate::install_cli_to_system(false);
            send_json_result(
                &mut stream,
                res.map(|msg| {
                    let (installed, path, in_path) = crate::check_cli_status();
                    format!(
                        "{{\"ok\":true,\"message\":{},\"installed\":{installed},\"path\":{},\"in_path\":{in_path}}}",
                        escape_json(&msg),
                        escape_json(&path)
                    )
                }),
            )?;
        }
        ("POST", "/api/terminal/execute") => {
            let res = handle_terminal_execute(&body_str);
            send_json_result(&mut stream, res)?;
        }
        ("GET", "/api/mods/list") => {
            let instance = query
                .split('&')
                .find(|p| p.starts_with("instance="))
                .and_then(|p| p.split('=').nth(1))
                .unwrap_or("default-native");
            let res = handle_mods_list(instance);
            send_json_result(&mut stream, res)?;
        }
        ("POST", "/api/mods/inject_bridge") => {
            let res = handle_mods_inject_bridge(&body_str);
            send_json_result(&mut stream, res)?;
        }
        ("POST", "/api/mods/install") => {
            let res = handle_mods_install(&body_str);
            send_json_result(&mut stream, res)?;
        }
        ("POST", "/api/mods/remove") => {
            let res = handle_mods_remove(&body_str);
            send_json_result(&mut stream, res)?;
        }
        ("GET", "/api/mods/search") => {
            let q = query
                .split('&')
                .find(|p| p.starts_with("query="))
                .and_then(|p| p.split('=').nth(1))
                .unwrap_or("");
            let clean_q = q.replace('+', " ");
            let res = handle_mods_search(&clean_q);
            send_json_result(&mut stream, res)?;
        }
        ("GET", "/api/visualizer/frame") => {
            let instance = query
                .split('&')
                .find(|p| p.starts_with("instance="))
                .and_then(|p| p.split('=').nth(1))
                .unwrap_or("default-native");
            let mode = query
                .split('&')
                .find(|p| p.starts_with("mode="))
                .and_then(|p| p.split('=').nth(1))
                .unwrap_or("first_person");
            let w = query
                .split('&')
                .find(|p| p.starts_with("w="))
                .and_then(|p| p.split('=').nth(1))
                .unwrap_or("160");
            let h = query
                .split('&')
                .find(|p| p.starts_with("h="))
                .and_then(|p| p.split('=').nth(1))
                .unwrap_or("100");
            let res = handle_visualizer_frame(instance, w, h, mode);
            send_json_result(&mut stream, res)?;
        }
        ("GET", "/api/visualizer/state") => {
            let instance = query
                .split('&')
                .find(|p| p.starts_with("instance="))
                .and_then(|p| p.split('=').nth(1))
                .unwrap_or("default-native");
            let res = handle_visualizer_state(instance);
            send_json_result(&mut stream, res)?;
        }
        ("GET", "/api/milestones") => {
            let instance = query
                .split('&')
                .find(|p| p.starts_with("instance="))
                .and_then(|p| p.split('=').nth(1))
                .unwrap_or("default-native");
            let res = handle_get_milestones(instance);
            send_json_result(&mut stream, res)?;
        }
        ("POST", "/api/milestone/save") => {
            let res = handle_post_milestone(&body_str);
            send_json_result(&mut stream, res)?;
        }
        ("POST", "/api/instance/optimize_minimal") => {
            let res = handle_optimize_minimal(&body_str);
            send_json_result(&mut stream, res)?;
        }
        ("GET", "/api/releases/status") => {
            let res = handle_releases_status();
            send_json_result(&mut stream, res)?;
        }
        ("POST", "/api/releases/install") => {
            let res = handle_releases_install(&body_str);
            send_json_result(&mut stream, res)?;
        }
        ("POST", "/api/mcp/call") => {
            let res = handle_mcp_call(&body_str);
            send_json_result(&mut stream, res)?;
        }
        ("GET", "/api/logs") => {
            let instance = query
                .split('&')
                .find(|p| p.starts_with("instance="))
                .and_then(|p| p.split('=').nth(1))
                .unwrap_or("");
            let logs = read_instance_logs(instance);
            let payload = format!(
                "{{\"instance\":\"{instance}\",\"logs\":{}}}",
                escape_json(&logs)
            );
            send_response(&mut stream, 200, "application/json", payload.as_bytes())?;
        }
        _ => {
            send_response(&mut stream, 404, "text/plain", b"Not Found")?;
        }
    }

    Ok(())
}

fn send_response(
    stream: &mut TcpStream,
    code: u16,
    content_type: &str,
    body: &[u8],
) -> std::io::Result<()> {
    let status_line = match code {
        200 => "HTTP/1.1 200 OK",
        400 => "HTTP/1.1 400 Bad Request",
        404 => "HTTP/1.1 404 Not Found",
        500 => "HTTP/1.1 500 Internal Server Error",
        _ => "HTTP/1.1 200 OK",
    };
    let headers = format!(
        "{status_line}\r\nContent-Type: {content_type}\r\nContent-Length: {}\r\nAccess-Control-Allow-Origin: *\r\nConnection: close\r\n\r\n",
        body.len()
    );
    stream.write_all(headers.as_bytes())?;
    stream.write_all(body)?;
    stream.flush()
}

fn send_json_result(stream: &mut TcpStream, res: Result<String, String>) -> std::io::Result<()> {
    match res {
        Ok(json) => send_response(stream, 200, "application/json", json.as_bytes()),
        Err(err) => {
            let payload = format!("{{\"ok\":false,\"error\":{}}}", escape_json(&err));
            send_response(stream, 400, "application/json", payload.as_bytes())
        }
    }
}

fn escape_json(s: &str) -> String {
    let mut out = String::from("\"");
    for c in s.chars() {
        match c {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            _ => out.push(c),
        }
    }
    out.push('"');
    out
}

fn build_api_status(processes: &ProcessMap) -> String {
    let launcher = match PersistentLauncher::load_or_init() {
        Ok(l) => l,
        Err(_) => return "{\"ok\":false,\"error\":\"could not load launcher database\"}".into(),
    };

    let mut json = String::from("{\"ok\":true,");
    json.push_str(&format!(
        "\"root_dir\":{},",
        escape_json(&launcher.root_dir.to_string_lossy())
    ));
    json.push_str("\"version\":\"0.3.3\",");

    // Version profiles
    json.push_str("\"version_profiles\":[");
    for (i, vp) in launcher.version_profiles.iter().enumerate() {
        if i > 0 {
            json.push(',');
        }
        json.push_str(&format!(
            "{{\"id\":\"{}\",\"version\":\"{}\",\"protocol\":{},\"native\":{},\"bridge\":{}}}",
            vp.profile_id,
            vp.minecraft_version,
            vp.protocol_version,
            vp.native_verified_parity,
            vp.bridge_supported
        ));
    }
    json.push_str("],");

    // Running process checks
    let mut active = processes.lock().unwrap();
    // Prune finished processes
    active.retain(|_, p| {
        if let Some(child) = &mut p.child {
            matches!(child.try_wait(), Ok(None))
        } else {
            false
        }
    });

    // Instances
    json.push_str("\"instances\":[");
    let mut first = true;
    for inst in launcher.instances.values() {
        if !first {
            json.push(',');
        }
        first = false;
        let running = active.contains_key(&inst.name);
        let pid = if running {
            active.get(&inst.name).map(|p| p.pid).unwrap_or(0)
        } else {
            0
        };
        json.push_str(&format!(
            "{{\"name\":\"{}\",\"profile\":\"{}\",\"mode\":\"{}\",\"server\":\"{}\",\"rpc_bind\":\"{}\",\"snapshots\":{},\"running\":{},\"pid\":{}}}",
            inst.name, inst.version_profile, inst.mode.as_str(), inst.server, inst.rpc_bind, inst.snapshot_count, running, pid
        ));
    }
    json.push_str("],");

    // Accounts
    json.push_str("\"accounts\":[");
    first = true;
    for acc in launcher.accounts.values() {
        if !first {
            json.push(',');
        }
        first = false;
        let locked = acc.locked_by_instance.as_deref().unwrap_or("");
        json.push_str(&format!(
            "{{\"username\":\"{}\",\"uuid\":\"{}\",\"locked_by\":\"{}\"}}",
            acc.username, acc.uuid_hex, locked
        ));
    }
    json.push_str("]}");

    json
}

fn handle_create_instance(body: &str) -> Result<String, String> {
    let json = JsonValue::parse(body).map_err(|e| format!("invalid json: {e}"))?;
    let name = json
        .get("name")
        .and_then(|v| v.as_str())
        .ok_or("missing 'name'")?;
    let profile = json
        .get("profile")
        .and_then(|v| v.as_str())
        .unwrap_or("sawbyte-26.3");
    let mode_str = json
        .get("mode")
        .and_then(|v| v.as_str())
        .unwrap_or("native");
    let mode = RuntimeMode::parse(mode_str)?;

    let mut launcher = PersistentLauncher::load_or_init()?;
    launcher.create_instance(name, profile, mode)?;

    if let Some(server) = json.get("server").and_then(|v| v.as_str()) {
        if let Some(inst) = launcher.instances.get_mut(name) {
            inst.server = server.to_owned();
        }
        let _ = launcher.save();
    }
    if let Some(rpc) = json.get("rpc_bind").and_then(|v| v.as_str()) {
        if let Some(inst) = launcher.instances.get_mut(name) {
            inst.rpc_bind = rpc.to_owned();
        }
        let _ = launcher.save();
    }

    Ok(format!(
        "{{\"ok\":true,\"message\":\"Instance '{name}' created successfully\"}}"
    ))
}

fn handle_delete_instance(body: &str, processes: &ProcessMap) -> Result<String, String> {
    let json = JsonValue::parse(body).map_err(|e| format!("invalid json: {e}"))?;
    let name = json
        .get("name")
        .and_then(|v| v.as_str())
        .ok_or("missing 'name'")?;

    // Stop process if running
    let mut procs = processes.lock().unwrap();
    if let Some(mut p) = procs.remove(name) {
        if let Some(child) = &mut p.child {
            let _ = child.kill();
        }
    }

    let mut launcher = PersistentLauncher::load_or_init()?;
    launcher.delete_instance(name)?;
    Ok(format!(
        "{{\"ok\":true,\"message\":\"Instance '{name}' deleted\"}}"
    ))
}

fn handle_set_setting(body: &str) -> Result<String, String> {
    let json = JsonValue::parse(body).map_err(|e| format!("invalid json: {e}"))?;
    let instance = json
        .get("instance")
        .and_then(|v| v.as_str())
        .ok_or("missing 'instance'")?;
    let key = json
        .get("key")
        .and_then(|v| v.as_str())
        .ok_or("missing 'key'")?;
    let value = json
        .get("value")
        .and_then(|v| v.as_str())
        .ok_or("missing 'value'")?;

    let mut launcher = PersistentLauncher::load_or_init()?;
    launcher.set_setting(instance, key, value)?;
    Ok(format!(
        "{{\"ok\":true,\"message\":\"Setting '{key}={value}' saved\"}}"
    ))
}

fn handle_backup_instance(body: &str) -> Result<String, String> {
    let json = JsonValue::parse(body).map_err(|e| format!("invalid json: {e}"))?;
    let instance = json
        .get("instance")
        .and_then(|v| v.as_str())
        .ok_or("missing 'instance'")?;

    let mut launcher = PersistentLauncher::load_or_init()?;
    let id = launcher.backup_instance(instance)?;
    Ok(format!(
        "{{\"ok\":true,\"snapshot_id\":{id},\"message\":\"Snapshot #{id} created\"}}"
    ))
}

fn handle_rollback_instance(body: &str) -> Result<String, String> {
    let json = JsonValue::parse(body).map_err(|e| format!("invalid json: {e}"))?;
    let instance = json
        .get("instance")
        .and_then(|v| v.as_str())
        .ok_or("missing 'instance'")?;
    let id = json
        .get("id")
        .and_then(|v| v.as_i64())
        .ok_or("missing 'id'")? as u32;

    let mut launcher = PersistentLauncher::load_or_init()?;
    launcher.rollback_instance(instance, id)?;
    Ok(format!(
        "{{\"ok\":true,\"message\":\"Rolled back to snapshot #{id}\"}}"
    ))
}

fn handle_add_account(body: &str) -> Result<String, String> {
    let json = JsonValue::parse(body).map_err(|e| format!("invalid json: {e}"))?;
    let username = json
        .get("username")
        .and_then(|v| v.as_str())
        .ok_or("missing 'username'")?;
    let uuid = json
        .get("uuid")
        .and_then(|v| v.as_str())
        .unwrap_or("00000000000030008000000000000001");

    let mut launcher = PersistentLauncher::load_or_init()?;
    launcher.add_account(username, uuid, None)?;
    Ok(format!(
        "{{\"ok\":true,\"message\":\"Account '{username}' added\"}}"
    ))
}

fn handle_delete_account(body: &str) -> Result<String, String> {
    let json = JsonValue::parse(body).map_err(|e| format!("invalid json: {e}"))?;
    let username = json
        .get("username")
        .and_then(|v| v.as_str())
        .ok_or("missing 'username'")?;

    let mut launcher = PersistentLauncher::load_or_init()?;
    launcher.delete_account(username)?;
    Ok(format!(
        "{{\"ok\":true,\"message\":\"Account '{username}' removed\"}}"
    ))
}

fn handle_launch_process(body: &str, processes: &ProcessMap) -> Result<String, String> {
    let json = JsonValue::parse(body).map_err(|e| format!("invalid json: {e}"))?;
    let instance = json
        .get("instance")
        .and_then(|v| v.as_str())
        .ok_or("missing 'instance'")?;
    let server = json
        .get("server")
        .and_then(|v| v.as_str())
        .unwrap_or("127.0.0.1:25565");
    let account = json
        .get("account")
        .and_then(|v| v.as_str())
        .unwrap_or("SawbyteNative");
    let blocks_path = json
        .get("blocks_json")
        .and_then(|v| v.as_str())
        .unwrap_or("block-profile.json");

    let mut procs = processes.lock().unwrap();
    if procs.contains_key(instance) {
        return Err(format!("instance '{instance}' is already running"));
    }

    let mut launcher = PersistentLauncher::load_or_init()?;
    let blocks_file = Path::new(blocks_path);
    let (child, log_path) =
        launcher.spawn_live_process(instance, Some(server), account, blocks_file, &[])?;

    let pid = child.id();
    let rpc_bind = launcher
        .instances
        .get(instance)
        .map(|i| i.rpc_bind.clone())
        .unwrap_or_else(|| "127.0.0.1:25580".into());

    procs.insert(
        instance.to_owned(),
        ActiveProcess {
            instance: instance.to_owned(),
            pid,
            server: server.to_owned(),
            rpc_bind: rpc_bind.clone(),
            account: account.to_owned(),
            log_path: log_path.clone(),
            child: Some(child),
        },
    );

    Ok(format!(
        "{{\"ok\":true,\"pid\":{pid},\"server\":\"{server}\",\"rpc_bind\":\"{rpc_bind}\",\"log\":\"{}\"}}",
        log_path.display()
    ))
}

fn handle_stop_process(body: &str, processes: &ProcessMap) -> Result<String, String> {
    let json = JsonValue::parse(body).map_err(|e| format!("invalid json: {e}"))?;
    let instance = json
        .get("instance")
        .and_then(|v| v.as_str())
        .ok_or("missing 'instance'")?;

    let mut procs = processes.lock().unwrap();
    if let Some(mut p) = procs.remove(instance) {
        if let Some(child) = &mut p.child {
            let _ = child.kill();
        }
        let mut launcher = PersistentLauncher::load_or_init()?;
        let _ = launcher.release_account_lock(&p.account);
        Ok(format!(
            "{{\"ok\":true,\"message\":\"Instance '{instance}' stopped\"}}"
        ))
    } else {
        Err(format!("instance '{instance}' was not running"))
    }
}

fn handle_send_live_command(body: &str) -> Result<String, String> {
    let json = JsonValue::parse(body).map_err(|e| format!("invalid json: {e}"))?;
    let connect = json
        .get("connect")
        .and_then(|v| v.as_str())
        .unwrap_or("127.0.0.1:25580");
    let cmd = json
        .get("command")
        .and_then(|v| v.as_str())
        .ok_or("missing 'command'")?;

    let mut stream = TcpStream::connect_timeout(
        &connect.parse().map_err(|e| format!("bad address: {e}"))?,
        Duration::from_secs(5),
    )
    .map_err(|e| format!("could not connect to live session at {connect}: {e}"))?;

    let _ = stream.set_read_timeout(Some(Duration::from_secs(10)));
    let _ = stream.set_write_timeout(Some(Duration::from_secs(10)));

    writeln!(stream, "{cmd}").map_err(|e| e.to_string())?;
    stream.flush().map_err(|e| e.to_string())?;

    let mut reader = BufReader::new(stream);
    let mut reply = String::new();
    reader.read_line(&mut reply).map_err(|e| e.to_string())?;

    Ok(format!(
        "{{\"ok\":true,\"reply\":{}}}",
        escape_json(reply.trim())
    ))
}

fn handle_terminal_execute(body: &str) -> Result<String, String> {
    let json = JsonValue::parse(body).map_err(|e| format!("invalid json: {e}"))?;
    let cmd = json
        .get("command")
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .trim();
    if cmd.is_empty() {
        return Ok("{\"ok\":true,\"output\":\"\"}".to_string());
    }

    let mut parts: Vec<&str> = cmd.split_whitespace().collect();
    if parts
        .first()
        .is_some_and(|&p| p == "sawbyte" || p == "./sawbyte")
    {
        parts.remove(0);
    }

    let current_exe = std::env::current_exe().unwrap_or_else(|_| PathBuf::from("sawbyte"));
    let mut child_cmd = Command::new(&current_exe);
    for part in &parts {
        child_cmd.arg(part);
    }
    let output = child_cmd
        .output()
        .map_err(|e| format!("failed to execute command: {e}"))?;
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    let full = if stderr.is_empty() {
        stdout.to_string()
    } else if stdout.is_empty() {
        stderr.to_string()
    } else {
        format!("{stdout}\n{stderr}")
    };
    let exit_code = output.status.code().unwrap_or(0);
    Ok(format!(
        "{{\"ok\":true,\"output\":{},\"exit_code\":{exit_code}}}",
        escape_json(&full)
    ))
}

fn handle_mods_list(instance: &str) -> Result<String, String> {
    let launcher = PersistentLauncher::load_or_init()?;
    let mods = launcher.list_mods(instance)?;
    let mut out = format!("{{\"ok\":true,\"instance\":\"{instance}\",\"mods\":[");
    let mut bridge_installed = false;
    for (i, m) in mods.iter().enumerate() {
        if i > 0 {
            out.push(',');
        }
        if m.is_bridge {
            bridge_installed = true;
        }
        out.push_str(&format!(
            "{{\"filename\":{},\"size_bytes\":{},\"is_bridge\":{}}}",
            escape_json(&m.filename),
            m.size_bytes,
            m.is_bridge
        ));
    }
    out.push_str(&format!("],\"bridge_installed\":{bridge_installed}}}"));
    Ok(out)
}

fn handle_mods_inject_bridge(body: &str) -> Result<String, String> {
    let json = JsonValue::parse(body).map_err(|e| format!("invalid json: {e}"))?;
    let instance = json
        .get("instance")
        .and_then(|v| v.as_str())
        .ok_or("missing 'instance'")?;
    let launcher = PersistentLauncher::load_or_init()?;
    let path = launcher.inject_bridge_mod(instance)?;
    Ok(format!(
        "{{\"ok\":true,\"message\":\"Sawbyte Bridge Mod injected\",\"path\":{}}}",
        escape_json(&path.to_string_lossy())
    ))
}

fn handle_mods_remove(body: &str) -> Result<String, String> {
    let json = JsonValue::parse(body).map_err(|e| format!("invalid json: {e}"))?;
    let instance = json
        .get("instance")
        .and_then(|v| v.as_str())
        .ok_or("missing 'instance'")?;
    let filename = json
        .get("filename")
        .and_then(|v| v.as_str())
        .ok_or("missing 'filename'")?;
    let launcher = PersistentLauncher::load_or_init()?;
    launcher.remove_mod(instance, filename)?;
    Ok(format!(
        "{{\"ok\":true,\"message\":\"Removed mod '{filename}'\"}}"
    ))
}

fn handle_mods_install(body: &str) -> Result<String, String> {
    let json = JsonValue::parse(body).map_err(|e| format!("invalid json: {e}"))?;
    let instance = json
        .get("instance")
        .and_then(|v| v.as_str())
        .ok_or("missing 'instance'")?;
    let launcher = PersistentLauncher::load_or_init()?;

    if let (Some(filename), Some(url)) = (
        json.get("filename").and_then(|v| v.as_str()),
        json.get("url").and_then(|v| v.as_str()),
    ) {
        let p = launcher.install_mod_file(instance, filename, url)?;
        return Ok(format!(
            "{{\"ok\":true,\"message\":\"Installed {filename}\",\"path\":{}}}",
            escape_json(&p.to_string_lossy())
        ));
    }

    let slug = json
        .get("slug")
        .and_then(|v| v.as_str())
        .ok_or("missing 'slug' or 'url'")?;
    let with_deps = json
        .get("with_deps")
        .and_then(|v| v.as_bool())
        .unwrap_or(true);

    let v_url = format!("https://api.modrinth.com/v2/project/{slug}/version?loaders=[\"fabric\"]");
    let out = Command::new("curl")
        .args(["-sSL", &v_url])
        .output()
        .map_err(|e| e.to_string())?;
    if !out.status.success() {
        return Err(format!("failed to fetch version for '{slug}'"));
    }
    let body_str = String::from_utf8_lossy(&out.stdout);
    let versions = JsonValue::parse(&body_str).map_err(|e| format!("json parse error: {e}"))?;
    let v_arr = versions.as_array().ok_or("no versions found on Modrinth")?;
    if v_arr.is_empty() {
        return Err(format!("no Fabric versions available for '{slug}'"));
    }
    let latest = &v_arr[0];
    let files = latest
        .get("files")
        .and_then(|f| f.as_array())
        .ok_or("no files in version")?;
    if files.is_empty() {
        return Err("no files found in latest version".into());
    }
    let file_url = files[0]
        .get("url")
        .and_then(|u| u.as_str())
        .ok_or("missing file url")?;
    let filename = files[0]
        .get("filename")
        .and_then(|f| f.as_str())
        .ok_or("missing filename")?;

    let path = launcher.install_mod_file(instance, filename, file_url)?;
    let mut installed_list = vec![filename.to_string()];

    if with_deps {
        if let Some(deps) = latest.get("dependencies").and_then(|d| d.as_array()) {
            for dep in deps {
                let dep_type = dep
                    .get("dependency_type")
                    .and_then(|t| t.as_str())
                    .unwrap_or("");
                if dep_type == "required" {
                    if let Some(dep_proj) = dep.get("project_id").and_then(|p| p.as_str()) {
                        let dep_url = format!(
                            "https://api.modrinth.com/v2/project/{dep_proj}/version?loaders=[\"fabric\"]"
                        );
                        if let Ok(dep_out) = Command::new("curl").args(["-sSL", &dep_url]).output()
                        {
                            if dep_out.status.success() {
                                if let Ok(dep_json) =
                                    JsonValue::parse(&String::from_utf8_lossy(&dep_out.stdout))
                                {
                                    if let Some(dep_arr) = dep_json.as_array() {
                                        if let Some(dep_latest) = dep_arr.first() {
                                            if let Some(dfiles) =
                                                dep_latest.get("files").and_then(|f| f.as_array())
                                            {
                                                if let Some(dfile) = dfiles.first() {
                                                    if let (Some(durl), Some(dname)) = (
                                                        dfile.get("url").and_then(|u| u.as_str()),
                                                        dfile
                                                            .get("filename")
                                                            .and_then(|n| n.as_str()),
                                                    ) {
                                                        if launcher
                                                            .install_mod_file(instance, dname, durl)
                                                            .is_ok()
                                                        {
                                                            installed_list.push(dname.to_string());
                                                        }
                                                    }
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    let installed_json = installed_list
        .iter()
        .map(|s| escape_json(s))
        .collect::<Vec<_>>()
        .join(",");
    Ok(format!(
        "{{\"ok\":true,\"message\":\"Mod installed successfully\",\"installed\":[{installed_json}],\"path\":{}}}",
        escape_json(&path.to_string_lossy())
    ))
}

fn handle_mods_search(query: &str) -> Result<String, String> {
    let clean_query = query.trim();
    let url = format!(
        "https://api.modrinth.com/v2/search?query={}&facets=[[\"categories:fabric\"]]&limit=16",
        url_encode(clean_query)
    );
    let out = Command::new("curl")
        .args(["-sSL", &url])
        .output()
        .map_err(|e| format!("failed to query Modrinth: {e}"))?;
    if !out.status.success() {
        return Err("Modrinth search failed".into());
    }
    Ok(String::from_utf8_lossy(&out.stdout).to_string())
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

fn get_instance_rpc_endpoint(instance: &str) -> Result<String, String> {
    let launcher = PersistentLauncher::load_or_init()?;
    let inst = launcher
        .instances
        .get(instance)
        .ok_or_else(|| format!("instance '{instance}' not found"))?;
    Ok(inst.rpc_bind.clone())
}

fn send_rpc_command(connect: &str, cmd: &str) -> Result<String, String> {
    let mut stream = TcpStream::connect(connect)
        .map_err(|e| format!("cannot connect to bot RPC at {connect}: {e}"))?;
    let _ = stream.set_read_timeout(Some(Duration::from_secs(5)));
    let _ = stream.set_write_timeout(Some(Duration::from_secs(5)));
    writeln!(stream, "{cmd}").map_err(|e| e.to_string())?;
    stream.flush().map_err(|e| e.to_string())?;
    let mut reader = BufReader::new(stream);
    let mut reply = String::new();
    reader.read_line(&mut reply).map_err(|e| e.to_string())?;
    Ok(reply.trim().to_string())
}

fn handle_visualizer_frame(instance: &str, w: &str, h: &str, mode: &str) -> Result<String, String> {
    let endpoint = get_instance_rpc_endpoint(instance)?;
    let cmd = format!("inspect.frame {w} {h} {mode}");
    let reply = send_rpc_command(&endpoint, &cmd)?;
    if let Some(data_uri) = reply.strip_prefix("frame:") {
        Ok(format!(
            "{{\"ok\":true,\"data_uri\":{}}}",
            escape_json(data_uri)
        ))
    } else {
        Err(format!("invalid frame response: {reply}"))
    }
}

fn handle_visualizer_state(instance: &str) -> Result<String, String> {
    let endpoint = get_instance_rpc_endpoint(instance)?;
    let reply = send_rpc_command(&endpoint, "inspect.visualizer_state")?;
    Ok(format!("{{\"ok\":true,\"state\":{reply}}}"))
}

fn handle_get_milestones(instance: &str) -> Result<String, String> {
    let endpoint = get_instance_rpc_endpoint(instance);
    let rpc_text = endpoint
        .ok()
        .and_then(|ep| send_rpc_command(&ep, "inspect.milestones").ok());
    let log_content = fs::read_to_string("milestones.log").unwrap_or_default();
    Ok(format!(
        "{{\"ok\":true,\"summary\":{},\"log\":{}}}",
        escape_json(&rpc_text.unwrap_or_default()),
        escape_json(&log_content)
    ))
}

fn handle_post_milestone(body: &str) -> Result<String, String> {
    let json = JsonValue::parse(body).map_err(|e| format!("invalid json: {e}"))?;
    let instance = json
        .get("instance")
        .and_then(|v| v.as_str())
        .unwrap_or("default-native");
    let name = json
        .get("name")
        .and_then(|v| v.as_str())
        .ok_or("missing 'name'")?;
    let notes = json.get("notes").and_then(|v| v.as_str()).unwrap_or("");
    let endpoint = get_instance_rpc_endpoint(instance)?;
    let cmd = format!("milestone.save {name} {notes}");
    let reply = send_rpc_command(&endpoint, &cmd)?;
    Ok(format!("{{\"ok\":true,\"reply\":{}}}", escape_json(&reply)))
}

fn handle_optimize_minimal(body: &str) -> Result<String, String> {
    let json = JsonValue::parse(body).map_err(|e| format!("invalid json: {e}"))?;
    let instance = json
        .get("instance")
        .and_then(|v| v.as_str())
        .ok_or("missing 'instance'")?;
    let mut launcher = PersistentLauncher::load_or_init()?;
    launcher.set_setting(instance, "graphicsMode", "0")?;
    launcher.set_setting(instance, "renderDistance", "2")?;
    launcher.set_setting(instance, "simulationDistance", "2")?;
    launcher.set_setting(instance, "smoothLighting", "0")?;
    launcher.set_setting(instance, "particles", "2")?;
    launcher.set_setting(instance, "entityShadows", "false")?;
    launcher.set_setting(instance, "clouds", "0")?;
    launcher.set_setting(instance, "biomeBlendRadius", "0")?;
    launcher.set_setting(instance, "maxFps", "30")?;
    launcher.set_setting(instance, "mipmapLevels", "0")?;
    launcher.set_setting(instance, "viewBobbing", "false")?;
    if let Some(inst) = launcher.instances.get_mut(instance) {
        inst.mode = RuntimeMode::NativeMinimalVisualizer;
    }
    launcher.save()?;
    Ok("{\"ok\":true,\"message\":\"Instance optimized to Native Minimal Low-Poly Visualizer mode.\"}".to_string())
}

fn read_instance_logs(instance: &str) -> String {
    if instance.is_empty() {
        return String::new();
    }
    let root = PersistentLauncher::resolve_home();
    let inst_dir = root.join("instances").join(instance);
    let session_log = inst_dir.join("session.log");
    let latest_log = inst_dir.join("logs").join("latest.log");
    let log_file = if session_log.exists() {
        session_log
    } else if latest_log.exists() {
        latest_log
    } else {
        return "No log file found for this instance yet.".into();
    };
    match fs::read_to_string(&log_file) {
        Ok(text) => {
            let lines: Vec<&str> = text.lines().collect();
            let start = if lines.len() > 80 {
                lines.len() - 80
            } else {
                0
            };
            lines[start..].join("\n")
        }
        Err(e) => format!("Could not read log file: {e}"),
    }
}

fn handle_releases_status() -> Result<String, String> {
    let launcher = PersistentLauncher::load_or_init()?;
    let (cli_installed, cli_path, _) = crate::check_cli_status();
    let headless_res = PersistentLauncher::locate_sawbyte_play_binary();
    let headless_installed = headless_res.is_ok();
    let headless_path = headless_res
        .map(|p| p.to_string_lossy().to_string())
        .unwrap_or_else(|_| {
            launcher
                .root_dir
                .join("bin")
                .join(if cfg!(windows) {
                    "sawbyte-play.exe"
                } else {
                    "sawbyte-play"
                })
                .to_string_lossy()
                .to_string()
        });
    let bridge_path = launcher
        .root_dir
        .join("editions")
        .join("sawbyte-bridge-0.3.3.jar");
    let bridge_installed = bridge_path.exists();
    let profile_path = launcher.ensure_block_profile(Path::new("block-profile.json"));
    let play_artifact = if cfg!(target_os = "windows") {
        "sawbyte-play-windows-x64.zip"
    } else if cfg!(target_os = "macos") {
        "sawbyte-play-macos-arm64.tar.gz"
    } else {
        "sawbyte-play-linux-x64.tar.gz"
    };
    Ok(format!(
        "{{\"ok\":true,\"play_artifact\":\"{play_artifact}\",\"cli_installed\":{cli_installed},\"cli_path\":{},\"headless_installed\":{headless_installed},\"headless_path\":{},\"bridge_installed\":{bridge_installed},\"profile_installed\":true,\"profile_path\":{}}}",
        escape_json(&cli_path),
        escape_json(&headless_path),
        escape_json(&profile_path.to_string_lossy())
    ))
}

fn handle_releases_install(body: &str) -> Result<String, String> {
    let json = JsonValue::parse(body).map_err(|e| format!("invalid json: {e}"))?;
    let edition = json
        .get("edition")
        .and_then(|v| v.as_str())
        .unwrap_or("all");
    let mut launcher = PersistentLauncher::load_or_init()?;
    let bin_dir = launcher.root_dir.join("bin");
    let ed_dir = launcher.root_dir.join("editions");
    fs::create_dir_all(&bin_dir).map_err(|e| e.to_string())?;
    fs::create_dir_all(&ed_dir).map_err(|e| e.to_string())?;

    if edition == "block_profile" || edition == "all" {
        let _ = launcher.ensure_block_profile(Path::new("block-profile.json"));
    }
    if edition == "headless" || edition == "potato" || edition == "all" {
        if !launcher.instances.contains_key("headless-26-3") {
            let _ = launcher.create_instance(
                "headless-26-3",
                "sawbyte-26.3",
                RuntimeMode::NativeStandalone,
            );
        }
        if edition == "potato" || edition == "all" {
            let _ = launcher.create_instance(
                "potato-26-3",
                "fabric-minimal",
                RuntimeMode::NativeMinimalVisualizer,
            );
        }
    }
    if edition == "fabric_bridge" || edition == "all" {
        if !launcher.instances.contains_key("fabric-bridge-26-3") {
            let _ = launcher.create_instance(
                "fabric-bridge-26-3",
                "fabric-26.3",
                RuntimeMode::FullClientBridge,
            );
        }
        let _ = launcher.inject_bridge_mod("fabric-bridge-26-3");
    }
    if edition == "cli" || edition == "all" {
        let _ = crate::install_cli_to_system(false);
    }

    Ok(format!(
        "{{\"ok\":true,\"message\":\"Provisioned '{edition}' edition in {}\"}}",
        bin_dir.display()
    ))
}

fn handle_mcp_call(body: &str) -> Result<String, String> {
    let json = JsonValue::parse(body).map_err(|e| format!("invalid json: {e}"))?;
    let name = json
        .get("name")
        .and_then(|v| v.as_str())
        .unwrap_or("code_execute");
    let script = json
        .get("script")
        .and_then(|v| v.as_str())
        .unwrap_or("inspect.status");
    let bridge = sawbyte_core::mcp::SawbyteControlBridge { live_endpoint: Some("127.0.0.1:25580".to_string()), ..Default::default() };
    let req = format!(
        "{{\"jsonrpc\":\"2.0\",\"id\":1,\"method\":\"tools/call\",\"params\":{{\"name\":{name_q},\"script\":{script_q}}}}}",
        name_q = escape_json(name),
        script_q = escape_json(script)
    );
    Ok(bridge.handle_mcp_json_rpc(&req))
}

fn get_launcher_html() -> &'static str {
    include_str!("../../../launcher/renderer/index.html")
}

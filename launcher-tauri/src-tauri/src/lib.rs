#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use sawbyte_core::launcher::{PersistentLauncher, RuntimeMode};
use serde::{Deserialize, Serialize};
use std::io::{BufRead, BufReader, Write};
use std::net::TcpStream;
use std::path::Path;
use std::time::Duration;

#[derive(Serialize, Deserialize, Clone)]
pub struct InstanceDto {
    pub name: String,
    pub version_profile: String,
    pub mode: String,
    pub server: String,
    pub rpc_bind: String,
    pub snapshot_count: u32,
    pub last_pid: Option<u32>,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct AccountDto {
    pub username: String,
    pub uuid_hex: String,
    pub is_offline: bool,
    pub locked_by: Option<String>,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct LauncherStateDto {
    pub root_dir: String,
    pub instances: Vec<InstanceDto>,
    pub accounts: Vec<AccountDto>,
}

#[tauri::command]
pub fn get_launcher_state() -> Result<LauncherStateDto, String> {
    let launcher = PersistentLauncher::load_or_init()?;
    let instances = launcher
        .instances
        .values()
        .map(|i| InstanceDto {
            name: i.name.clone(),
            version_profile: i.version_profile.clone(),
            mode: i.mode.as_str().to_string(),
            server: i.server.clone(),
            rpc_bind: i.rpc_bind.clone(),
            snapshot_count: i.snapshot_count,
            last_pid: i.last_pid,
        })
        .collect();
    let accounts = launcher
        .accounts
        .values()
        .map(|a| AccountDto {
            username: a.username.clone(),
            uuid_hex: a.uuid_hex.clone(),
            is_offline: a.token_file.is_none(),
            locked_by: a.locked_by_instance.clone(),
        })
        .collect();
    Ok(LauncherStateDto {
        root_dir: launcher.root_dir.to_string_lossy().to_string(),
        instances,
        accounts,
    })
}

#[tauri::command]
pub fn create_instance(name: String, profile: String, mode: String, server: String) -> Result<LauncherStateDto, String> {
    let mut launcher = PersistentLauncher::load_or_init()?;
    let rm = RuntimeMode::parse(&mode)?;
    launcher.create_instance(&name, &profile, rm)?;
    if !server.trim().is_empty() {
        let _ = launcher.set_setting(&name, "server", server.trim());
    }
    get_launcher_state()
}

#[tauri::command]
pub fn add_account(username: String, uuid_hex: String) -> Result<LauncherStateDto, String> {
    let mut launcher = PersistentLauncher::load_or_init()?;
    let clean_uuid = if uuid_hex.trim().len() == 32 {
        uuid_hex.trim().to_string()
    } else {
        "00000000000030008000000000000001".to_string()
    };
    launcher.add_account(username.trim(), &clean_uuid, None)?;
    get_launcher_state()
}

#[tauri::command]
pub fn launch_instance(
    instance_name: String,
    edition: String,
    target_mode: String,
    server_addr: String,
    account_name: String,
    rpc_bind: String,
) -> Result<String, String> {
    let mut launcher = PersistentLauncher::load_or_init()?;
    let mode = match edition.as_str() {
        "potato" | "visualizer" => RuntimeMode::NativeMinimalVisualizer,
        "bridge" | "fabric" => RuntimeMode::FullClientBridge,
        _ => RuntimeMode::NativeStandalone,
    };
    let profile = match edition.as_str() {
        "potato" | "visualizer" => "fabric-minimal",
        "bridge" | "fabric" => "fabric-26.3",
        _ => "sawbyte-26.3",
    };
    if !launcher.instances.contains_key(&instance_name) {
        launcher.create_instance(&instance_name, profile, mode)?;
    }
    if !launcher.accounts.contains_key(&account_name) {
        let _ = launcher.add_account(&account_name, "00000000000030008000000000000001", None);
    }
    if !rpc_bind.trim().is_empty() {
        let _ = launcher.set_setting(&instance_name, "rpc_bind", rpc_bind.trim());
    }
    let effective_server = if target_mode == "singleplayer" {
        "singleplayer".to_string()
    } else {
        server_addr
    };
    let _ = launcher.set_setting(&instance_name, "server", &effective_server);
    let (child, log_path) = launcher.spawn_live_process(
        &instance_name,
        Some(&effective_server),
        &account_name,
        Path::new("block-profile.json"),
        &[],
    )?;
    Ok(format!(
        "Launched '{}' (PID {}) -> {} [log: {}]",
        instance_name,
        child.id(),
        effective_server,
        log_path.display()
    ))
}

#[tauri::command]
pub fn send_bot_command(endpoint: String, command: String) -> Result<String, String> {
    let ep = if endpoint.trim().is_empty() {
        "127.0.0.1:25580"
    } else {
        endpoint.trim()
    };
    let mut stream = TcpStream::connect_timeout(
        &ep.parse().map_err(|e| format!("invalid endpoint {ep}: {e}"))?,
        Duration::from_millis(1500),
    )
    .map_err(|e| format!("failed to connect to bot at {ep}: {e}"))?;
    let _ = stream.set_read_timeout(Some(Duration::from_millis(2500)));
    let _ = stream.set_write_timeout(Some(Duration::from_millis(1500)));
    writeln!(stream, "{}", command.trim()).map_err(|e| e.to_string())?;
    stream.flush().map_err(|e| e.to_string())?;
    let mut reader = BufReader::new(stream);
    let mut line = String::new();
    reader.read_line(&mut line).map_err(|e| e.to_string())?;
    Ok(line.trim().replace("\\n", "\n"))
}

pub fn run() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![
            get_launcher_state,
            create_instance,
            add_account,
            launch_instance,
            send_bot_command,
        ])
        .run(tauri::generate_context!())
        .expect("error while running Sawbyte Tauri application");
}

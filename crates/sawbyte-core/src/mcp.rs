//! Model Context Protocol (MCP) JSON-RPC 2.0 control bridge, persistent launcher controller,
//! out-of-game client menu/settings controller, and TCP live-session forwarding to `sawbyte-play live`.
//!
//! No fake in-memory world simulators live here: launcher operations persist to disk via
//! `PersistentLauncher`, and live gameplay/navigation/raycast/GUI/RL commands forward over TCP
//! to a running `sawbyte-play live --listen <ip:port>` process (or a pluggable `LiveSessionTransport`).

use crate::events::{ClientMenuTarget, MouseButton, SubTickEvent};
pub use crate::launcher::{
    built_in_version_profiles, AccountRecord, InstanceRecord, PersistentLauncher, RuntimeMode,
    VersionAdapterProfile,
};
use crate::Direct;
use std::collections::BTreeMap;
use std::io::{BufRead, BufReader, Write};
use std::net::{TcpStream, ToSocketAddrs};
use std::path::PathBuf;
use std::time::{Duration, Instant};

/// Pluggable transport trait for forwarding live player/world/GUI/RL commands to `sawbyte-play live`.
pub trait LiveSessionTransport: Send {
    fn send_live_command(&mut self, command: &str) -> Result<String, String>;
}

/// Default TCP transport that connects to a running `sawbyte-play live --listen <ip:port>` session.
pub struct TcpLiveSessionTransport {
    pub endpoint: String,
    pub timeout: Duration,
}

impl TcpLiveSessionTransport {
    pub fn new(endpoint: impl Into<String>) -> Self {
        Self {
            endpoint: endpoint.into(),
            timeout: Duration::from_secs(30),
        }
    }
}

impl LiveSessionTransport for TcpLiveSessionTransport {
    fn send_live_command(&mut self, command: &str) -> Result<String, String> {
        let addr = self
            .endpoint
            .to_socket_addrs()
            .map_err(|e| format!("invalid live session address '{}': {e}", self.endpoint))?
            .next()
            .ok_or_else(|| format!("could not resolve live session address '{}'", self.endpoint))?;
        let mut stream = TcpStream::connect_timeout(&addr, self.timeout).map_err(|e| {
            format!(
                "no live `sawbyte-play live --listen {}` session reachable ({e}); launch an instance first with `sawbyte launcher launch` or `sawbyte-play live`",
                self.endpoint
            )
        })?;
        let _ = stream.set_read_timeout(Some(self.timeout));
        let _ = stream.set_write_timeout(Some(self.timeout));
        let mut outputs = Vec::new();
        let reader_stream = stream.try_clone().map_err(|e| e.to_string())?;
        let mut reader = BufReader::new(reader_stream);
        for line in command.lines() {
            let trimmed = line.trim();
            if trimmed.is_empty() || trimmed.starts_with('#') || trimmed.starts_with("//") {
                continue;
            }
            writeln!(stream, "{trimmed}").map_err(|e| e.to_string())?;
            stream.flush().map_err(|e| e.to_string())?;
            let mut reply = String::new();
            let n = reader
                .read_line(&mut reply)
                .map_err(|e| format!("live session read error: {e}"))?;
            if n == 0 {
                return Err("live session closed connection (EOF)".into());
            }
            let clean = reply.trim_end().replace("\\n", "\n");
            if clean.starts_with("error: ") {
                return Err(clean);
            }
            outputs.push(clean);
        }
        Ok(outputs.join("\n"))
    }
}

/// Out-of-game and in-game client menu, settings, and manual handoff controller backed by disk `options.txt`.
pub struct ClientController {
    pub active_menu: ClientMenuTarget,
    pub connected_server: Option<String>,
    pub settings: BTreeMap<String, String>,
    pub keybinds: BTreeMap<String, String>,
    pub manual_override_active: bool,
    pub recorded_events: Vec<SubTickEvent>,
}

impl Default for ClientController {
    fn default() -> Self {
        let mut settings = BTreeMap::new();
        settings.insert("fov".into(), "70".into());
        settings.insert("renderDistance".into(), "3".into());
        settings.insert("simulationDistance".into(), "3".into());
        settings.insert("guiScale".into(), "2".into());
        settings.insert("gamma".into(), "0.5".into());
        settings.insert("maxFps".into(), "60".into());
        settings.insert("autoJump".into(), "false".into());
        settings.insert("pauseOnLostFocus".into(), "false".into());
        settings.insert("soundCategory_master".into(), "1.0".into());

        let mut keybinds = BTreeMap::new();
        keybinds.insert("key.forward".into(), "key.keyboard.w".into());
        keybinds.insert("key.back".into(), "key.keyboard.s".into());
        keybinds.insert("key.left".into(), "key.keyboard.a".into());
        keybinds.insert("key.right".into(), "key.keyboard.d".into());
        keybinds.insert("key.jump".into(), "key.keyboard.space".into());
        keybinds.insert("key.sneak".into(), "key.keyboard.left.shift".into());
        keybinds.insert("key.sprint".into(), "key.keyboard.left.control".into());
        keybinds.insert("key.inventory".into(), "key.keyboard.e".into());

        Self {
            active_menu: ClientMenuTarget::MainMenu,
            connected_server: None,
            settings,
            keybinds,
            manual_override_active: false,
            recorded_events: Vec::new(),
        }
    }
}

impl ClientController {
    pub fn navigate(&mut self, target: ClientMenuTarget) -> SubTickEvent {
        if let ClientMenuTarget::DirectConnect { ref address } = target {
            self.connected_server = Some(address.clone());
        } else if matches!(
            target,
            ClientMenuTarget::DisconnectToTitle | ClientMenuTarget::MainMenu
        ) {
            self.connected_server = None;
        }
        self.active_menu = target.clone();
        let ev = SubTickEvent::NavigateClientMenu { target };
        self.recorded_events.push(ev.clone());
        ev
    }

    pub fn set_setting(&mut self, key: &str, value: &str) -> Result<SubTickEvent, String> {
        let ev = SubTickEvent::UpdateClientSetting {
            key: key.to_owned(),
            value: value.to_owned(),
        };
        if !ev.valid() {
            return Err("invalid setting key or value".into());
        }
        if key.starts_with("key.") {
            self.keybinds.insert(key.to_owned(), value.to_owned());
        } else {
            self.settings.insert(key.to_owned(), value.to_owned());
        }
        self.recorded_events.push(ev.clone());
        Ok(ev)
    }

    pub fn export_options_txt(&self) -> String {
        let mut out = String::new();
        for (k, v) in &self.settings {
            out.push_str(&format!("{k}:{v}\n"));
        }
        for (k, v) in &self.keybinds {
            out.push_str(&format!("{k}:{v}\n"));
        }
        out
    }

    pub fn trigger_manual_handoff(&mut self, reason: &str) -> SubTickEvent {
        self.manual_override_active = true;
        let ev = SubTickEvent::ManualHandoff {
            reason: reason.to_owned(),
        };
        self.recorded_events.push(ev.clone());
        ev
    }

    pub fn resume_automation(&mut self) {
        self.manual_override_active = false;
    }
}

/// Real persistent launcher and live-session MCP bridge.
pub struct SawbyteControlBridge {
    pub launcher: PersistentLauncher,
    pub client: ClientController,
    pub active_instance: String,
    pub live_endpoint: Option<String>,
    pub custom_transport: Option<Box<dyn LiveSessionTransport>>,
    pub script_logs: Vec<String>,
}

impl Default for SawbyteControlBridge {
    fn default() -> Self {
        let launcher = PersistentLauncher::load_or_init().unwrap_or_else(|_| {
            PersistentLauncher::load_from(std::env::temp_dir().join("sawbyte-fallback")).unwrap()
        });
        let live_endpoint = std::env::var("SAWBYTE_LIVE_ENDPOINT").ok();
        Self {
            launcher,
            client: ClientController::default(),
            active_instance: "default-native".into(),
            live_endpoint,
            custom_transport: None,
            script_logs: Vec::new(),
        }
    }
}

impl SawbyteControlBridge {
    pub fn with_root(root_dir: PathBuf) -> Result<Self, String> {
        let launcher = PersistentLauncher::load_from(root_dir)?;
        Ok(Self {
            launcher,
            client: ClientController::default(),
            active_instance: "default-native".into(),
            live_endpoint: None,
            custom_transport: None,
            script_logs: Vec::new(),
        })
    }

    pub fn set_live_transport(&mut self, transport: Box<dyn LiveSessionTransport>) {
        self.custom_transport = Some(transport);
    }

    fn resolve_live_endpoint(&self) -> String {
        if let Some(ref ep) = self.live_endpoint {
            return ep.clone();
        }
        if let Some(inst) = self.launcher.instances.get(&self.active_instance) {
            return inst.rpc_bind.clone();
        }
        "127.0.0.1:25580".into()
    }

    pub fn forward_live_command(&mut self, cmd: &str) -> Result<String, String> {
        if let Some(ref mut transport) = self.custom_transport {
            return transport.send_live_command(cmd);
        }
        let endpoint = self.resolve_live_endpoint();
        let mut tcp = TcpLiveSessionTransport::new(endpoint);
        tcp.send_live_command(cmd)
    }

    /// Executes launcher, menu, setting, or live-session commands.
    /// Launcher and settings mutations persist to disk immediately; live gameplay/GUI/world/RL commands
    /// are forwarded over TCP to the running `sawbyte-play live` process.
    pub fn execute_script(&mut self, script: &str) -> Result<Vec<String>, String> {
        let mut outputs = Vec::new();
        for (line_no, raw_line) in script.lines().enumerate() {
            let line = raw_line.trim();
            if line.is_empty() || line.starts_with('#') || line.starts_with("//") {
                continue;
            }
            let tokens = match crate::tokenize_command_line(line) {
                Ok(t) => t,
                Err(e) => return Err(format!("line {}: {e}", line_no + 1)),
            };
            let parts: Vec<&str> = tokens.iter().map(|s| s.as_str()).collect();
            match parts.as_slice() {
                ["launcher.create_instance", name, version, mode] => {
                    let runtime_mode = RuntimeMode::parse(mode)
                        .map_err(|e| format!("line {}: {e}", line_no + 1))?;
                    self.launcher.create_instance(name, version, runtime_mode)?;
                    self.active_instance = (*name).to_owned();
                    outputs.push(format!(
                        "persisted instance '{name}' ({version}, {mode}) to {}",
                        self.launcher.root_dir.display()
                    ));
                }
                ["launcher.use_instance", name] => {
                    if !self.launcher.instances.contains_key(*name) {
                        return Err(format!("instance '{name}' not found on disk"));
                    }
                    self.active_instance = (*name).to_owned();
                    outputs.push(format!("selected active instance '{name}'"));
                }
                ["launcher.backup", name] => {
                    let snap = self.launcher.backup_instance(name)?;
                    outputs.push(format!(
                        "saved token-free disk snapshot #{snap} for '{name}'"
                    ));
                }
                ["launcher.rollback", name, snap_id] => {
                    let id: u32 = snap_id.parse().map_err(|_| "invalid snapshot id")?;
                    self.launcher.rollback_instance(name, id)?;
                    outputs.push(format!("rolled back '{name}' on disk to snapshot #{id}"));
                }
                ["launcher.add_account", username, uuid_hex] => {
                    self.launcher.add_account(username, uuid_hex, None)?;
                    outputs.push(format!("persisted offline account '{username}'"));
                }
                ["launcher.list"]
                | ["launcher.list_instances"]
                | ["launcher_control", "list"]
                | ["launcher.instances"]
                | ["list"] => {
                    let mut inst_names: Vec<String> =
                        self.launcher.instances.keys().cloned().collect();
                    inst_names.sort();
                    outputs.push(format!("instances: {}", inst_names.join(", ")));
                }
                ["launcher.accounts"] | ["launcher.list_accounts"] => {
                    let mut accs: Vec<String> = self.launcher.accounts.keys().cloned().collect();
                    accs.sort();
                    outputs.push(format!("accounts: {}", accs.join(", ")));
                }
                ["launcher.connect_live", endpoint] => {
                    self.live_endpoint = Some((*endpoint).to_owned());
                    outputs.push(format!(
                        "attached control bridge to live endpoint {endpoint}"
                    ));
                }
                ["launcher.launch", instance, server, account, blocks_json] => {
                    let (_child, log_path) = self.launcher.spawn_live_process(
                        instance,
                        Some(server),
                        account,
                        std::path::Path::new(blocks_json),
                        &[],
                    )?;
                    self.active_instance = (*instance).to_owned();
                    if let Some(inst) = self.launcher.instances.get(*instance) {
                        self.live_endpoint = Some(inst.rpc_bind.clone());
                    }
                    outputs.push(format!(
                        "spawned live sawbyte-play process for '{instance}' -> {server} (log: {})",
                        log_path.display()
                    ));
                }
                ["menu.goto", target] => {
                    let menu = match *target {
                        "main_menu" => ClientMenuTarget::MainMenu,
                        "singleplayer" => ClientMenuTarget::SingleplayerMenu,
                        "multiplayer" => ClientMenuTarget::MultiplayerMenu,
                        "options" => ClientMenuTarget::OptionsMenu,
                        "video_settings" => ClientMenuTarget::VideoSettings,
                        "audio_settings" => ClientMenuTarget::AudioSettings,
                        "controls" => ClientMenuTarget::ControlsSettings,
                        "keybinds" => ClientMenuTarget::KeybindsMenu,
                        "pause" => ClientMenuTarget::PauseMenu,
                        "resume" => ClientMenuTarget::ResumeGame,
                        "respawn" => ClientMenuTarget::RespawnPlayer,
                        "disconnect" => ClientMenuTarget::DisconnectToTitle,
                        _ => {
                            if let Some(addr) = target.strip_prefix("direct_connect:") {
                                ClientMenuTarget::DirectConnect {
                                    address: addr.to_owned(),
                                }
                            } else {
                                return Err(format!(
                                    "line {}: unknown menu target '{target}'",
                                    line_no + 1
                                ));
                            }
                        }
                    };
                    self.client.navigate(menu);
                    outputs.push(format!("navigated client menu -> {target}"));
                }
                ["settings.set", key, value] => {
                    self.client.set_setting(key, value)?;
                    let inst_name = self.active_instance.clone();
                    if self.launcher.instances.contains_key(&inst_name) {
                        self.launcher.set_setting(&inst_name, key, value)?;
                    }
                    outputs.push(format!(
                        "persisted setting {key}={value} on disk for '{inst_name}'"
                    ));
                }
                // All live gameplay, navigation, raycast, GUI workflow, inspector, and RL commands
                // are forwarded directly to the live `sawbyte-play live` process.
                _ => {
                    let live_reply = self.forward_live_command(line)?;
                    outputs.push(live_reply);
                }
            }
        }
        self.script_logs.extend(outputs.clone());
        Ok(outputs)
    }

    /// Handles a JSON-RPC 2.0 / Model Context Protocol (MCP) request line.
    pub fn handle_mcp_json_rpc(&mut self, request_line: &str) -> String {
        let req = request_line.trim();
        let parsed = match crate::json::JsonValue::parse(req) {
            Ok(v) => v,
            Err(e) => {
                return format!(
                    "{{\"jsonrpc\":\"2.0\",\"id\":null,\"error\":{{\"code\":-32700,\"message\":\"Parse error: {}\"}}}}",
                    escape_json(&e)
                );
            }
        };

        let id = match parsed.get("id") {
            Some(crate::json::JsonValue::Number(n)) => {
                if n.fract() == 0.0 && n.is_finite() {
                    format!("{}", *n as i64)
                } else {
                    format!("{n}")
                }
            }
            Some(crate::json::JsonValue::String(s)) => format!("\"{}\"", escape_json(s)),
            _ => "1".to_string(),
        };

        let method = parsed
            .get("method")
            .and_then(|m| m.as_str())
            .unwrap_or_default();

        match method {
            "initialize" => format!(
                "{{\"jsonrpc\":\"2.0\",\"id\":{id},\"result\":{{\"protocolVersion\":\"2025-06-18\",\"serverInfo\":{{\"name\":\"sawbyte-mcp\",\"version\":\"0.2.1\"}},\"capabilities\":{{\"tools\":{{\"listChanged\":false}},\"resources\":{{\"listChanged\":false}},\"prompts\":{{\"listChanged\":false}}}}}}}}"
            ),
            "notifications/initialized" => String::new(),
            "ping" => format!("{{\"jsonrpc\":\"2.0\",\"id\":{id},\"result\":{{}}}}"),
            "tools/list" => format!(
                "{{\"jsonrpc\":\"2.0\",\"id\":{id},\"result\":{{\"tools\":[\
                {{\"name\":\"launcher_control\",\"description\":\"Persistently create, list, backup, rollback, configure, and launch real `sawbyte-play live` headless client instances on disk.\",\"inputSchema\":{{\"type\":\"object\",\"properties\":{{\"script\":{{\"type\":\"string\"}},\"command\":{{\"type\":\"string\"}}}}}}}},\
                {{\"name\":\"client_menu_and_settings\",\"description\":\"Navigate client menus and persist `options.txt` settings and keybinds on disk.\",\"inputSchema\":{{\"type\":\"object\",\"properties\":{{\"script\":{{\"type\":\"string\"}},\"command\":{{\"type\":\"string\"}}}}}}}},\
                {{\"name\":\"player_input\",\"description\":\"Forward live humanised/smooth/direct movement, camera, and hotbar inputs to a running `sawbyte-play live` session.\",\"inputSchema\":{{\"type\":\"object\",\"properties\":{{\"script\":{{\"type\":\"string\"}},\"command\":{{\"type\":\"string\"}}}}}}}},\
                {{\"name\":\"player_interact\",\"description\":\"Forward live raycast block mining, block placement, and entity combat to `sawbyte-play live`.\",\"inputSchema\":{{\"type\":\"object\",\"properties\":{{\"script\":{{\"type\":\"string\"}},\"command\":{{\"type\":\"string\"}}}}}}}},\
                {{\"name\":\"gui_screen_automation\",\"description\":\"Inspect live server containers, click slots/items/widgets with state_id verification, and run live multi-screen workflows on `sawbyte-play live`.\",\"inputSchema\":{{\"type\":\"object\",\"properties\":{{\"script\":{{\"type\":\"string\"}},\"command\":{{\"type\":\"string\"}}}}}}}},\
                {{\"name\":\"task_and_navigation\",\"description\":\"Run live 3D A* pathfinding across the server's loaded chunk sections on `sawbyte-play live`.\",\"inputSchema\":{{\"type\":\"object\",\"properties\":{{\"script\":{{\"type\":\"string\"}},\"command\":{{\"type\":\"string\"}}}}}}}},\
                {{\"name\":\"world_inspector_and_profiler\",\"description\":\"Render the live 2D ASCII map from server chunks and query live player status.\",\"inputSchema\":{{\"type\":\"object\",\"properties\":{{\"script\":{{\"type\":\"string\"}},\"command\":{{\"type\":\"string\"}}}}}}}},\
                {{\"name\":\"gym_step_and_observe\",\"description\":\"Step and observe the live `sawbyte-play` session for PyTorch/Gymnasium reinforcement learning.\",\"inputSchema\":{{\"type\":\"object\",\"properties\":{{\"script\":{{\"type\":\"string\"}},\"command\":{{\"type\":\"string\"}}}}}}}},\
                {{\"name\":\"code_execute\",\"description\":\"Execute multi-line SawbyteScript across the persistent disk launcher and live `sawbyte-play` session.\",\"inputSchema\":{{\"type\":\"object\",\"properties\":{{\"script\":{{\"type\":\"string\"}},\"command\":{{\"type\":\"string\"}}}}}}}}\
                ]}}}}"
            ),
            "resources/list" => format!(
                "{{\"jsonrpc\":\"2.0\",\"id\":{id},\"result\":{{\"resources\":[\
                {{\"uri\":\"sawbyte://launcher/state\",\"name\":\"Persistent Disk Launcher State\",\"mimeType\":\"text/plain\"}},\
                {{\"uri\":\"sawbyte://client/options\",\"name\":\"Persistent Client options.txt & Keybinds\",\"mimeType\":\"text/plain\"}},\
                {{\"uri\":\"sawbyte://world/map2d\",\"name\":\"Live Server 2D World Map\",\"mimeType\":\"text/plain\"}}\
                ]}}}}"
            ),
            "resources/read" => {
                let params = parsed.get("params");
                let uri = params
                    .and_then(|p| p.get("uri"))
                    .and_then(|u| u.as_str())
                    .unwrap_or_default();
                let text_opt = match uri {
                    "sawbyte://client/options" => Some(self.client.export_options_txt()),
                    "sawbyte://world/map2d" => Some(
                        self.forward_live_command("inspect.map")
                            .unwrap_or_else(|e| e),
                    ),
                    "sawbyte://launcher/state" => Some(format!(
                        "root_dir={} instances={} accounts={} active_instance={} live_endpoint={}",
                        self.launcher.root_dir.display(),
                        self.launcher.instances.len(),
                        self.launcher.accounts.len(),
                        self.active_instance,
                        self.resolve_live_endpoint()
                    )),
                    _ => None,
                };
                match text_opt {
                    Some(text) => format!(
                        "{{\"jsonrpc\":\"2.0\",\"id\":{id},\"result\":{{\"contents\":[{{\"uri\":\"{}\",\"mimeType\":\"text/plain\",\"text\":\"{}\"}}]}}}}",
                        escape_json(uri),
                        escape_json(&text)
                    ),
                    None => format!(
                        "{{\"jsonrpc\":\"2.0\",\"id\":{id},\"error\":{{\"code\":-32602,\"message\":\"Unknown resource URI: {}\"}}}}",
                        escape_json(uri)
                    ),
                }
            }
            "prompts/list" => format!(
                "{{\"jsonrpc\":\"2.0\",\"id\":{id},\"result\":{{\"prompts\":[\
                {{\"name\":\"automate_anything\",\"description\":\"System prompt for live headless Rust Minecraft automation and RL control via Sawbyte MCP.\"}}\
                ]}}}}"
            ),
            "prompts/get" => format!(
                "{{\"jsonrpc\":\"2.0\",\"id\":{id},\"result\":{{\"description\":\"Sawbyte automation prompt\",\"messages\":[{{\"role\":\"user\",\"content\":{{\"type\":\"text\",\"text\":\"Use tools/list and tools/call to inspect state and schedule validated actions.\"}}}}]}}}}"
            ),
            "tools/call" => {
                let params = parsed.get("params");
                let tool_name = params
                    .and_then(|p| p.get("name"))
                    .and_then(|n| n.as_str())
                    .unwrap_or_default();
                let is_valid_tool = matches!(
                    tool_name,
                    "launcher_control"
                        | "client_menu_and_settings"
                        | "player_input"
                        | "player_interact"
                        | "gui_screen_automation"
                        | "task_and_navigation"
                        | "world_inspector_and_profiler"
                        | "gym_step_and_observe"
                        | "code_execute"
                );
                if !is_valid_tool {
                    return format!(
                        "{{\"jsonrpc\":\"2.0\",\"id\":{id},\"result\":{{\"isError\":true,\"content\":[{{\"type\":\"text\",\"text\":\"unknown tool '{}'\"}}]}}}}",
                        escape_json(tool_name)
                    );
                }
                let args = params.and_then(|p| p.get("arguments"));
                let script_or_cmd = args
                    .and_then(|a| a.get("script").or_else(|| a.get("command")))
                    .and_then(|s| s.as_str())
                    .or_else(|| {
                        params
                            .and_then(|p| p.get("script").or_else(|| p.get("command")))
                            .and_then(|s| s.as_str())
                    })
                    .unwrap_or_default();
                if script_or_cmd.trim().is_empty() {
                    return format!(
                        "{{\"jsonrpc\":\"2.0\",\"id\":{id},\"result\":{{\"isError\":true,\"content\":[{{\"type\":\"text\",\"text\":\"missing required 'script' or 'command' argument for tool '{tool_name}'\"}}]}}}}"
                    );
                }
                match self.execute_script(script_or_cmd) {
                    Ok(lines) => format!(
                        "{{\"jsonrpc\":\"2.0\",\"id\":{id},\"result\":{{\"isError\":false,\"content\":[{{\"type\":\"text\",\"text\":\"{}\"}}]}}}}",
                        escape_json(&lines.join("\n"))
                    ),
                    Err(err) => format!(
                        "{{\"jsonrpc\":\"2.0\",\"id\":{id},\"result\":{{\"isError\":true,\"content\":[{{\"type\":\"text\",\"text\":\"{}\"}}]}}}}",
                        escape_json(&err)
                    ),
                }
            }
            _ => format!(
                "{{\"jsonrpc\":\"2.0\",\"id\":{id},\"error\":{{\"code\":-32601,\"message\":\"Method not found\"}}}}"
            ),
        }
    }
}

/// Benchmarks binary SBIF framing against text JSON-RPC framing across `iterations` 160-tick batches.
pub fn benchmark_transports(iterations: usize) -> (u128, usize, u128, usize) {
    let frames = crate::tape::movement_fixture(Direct);
    let runs = iterations.clamp(1, 500);

    let t0 = Instant::now();
    let mut bin_bytes = 0usize;
    for _ in 0..runs {
        let mut buf = Vec::with_capacity(3000);
        crate::tape::write(&mut buf, &frames).unwrap();
        let _decoded = crate::tape::read(&mut buf.as_slice()).unwrap();
        bin_bytes = buf.len();
    }
    let bin_ns = t0.elapsed().as_nanos();

    let t1 = Instant::now();
    let mut json_bytes = 0usize;
    for _ in 0..runs {
        let mut s = String::with_capacity(12000);
        s.push('[');
        for (i, f) in frames.iter().enumerate() {
            if i > 0 {
                s.push(',');
            }
            s.push_str(&format!(
                "{{\"b\":{},\"y\":{:.4},\"p\":{:.4}}}",
                f.buttons.bits(),
                f.yaw_delta,
                f.pitch_delta
            ));
        }
        s.push(']');
        json_bytes = s.len();
    }
    let json_ns = t1.elapsed().as_nanos();
    (bin_ns, bin_bytes, json_ns, json_bytes)
}

fn escape_json(input: &str) -> String {
    let mut out = String::with_capacity(input.len() + 8);
    for ch in input.chars() {
        match ch {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            c if c.is_control() => out.push_str(&format!("\\u{:04x}", c as u32)),
            c => out.push(c),
        }
    }
    out
}

#[allow(dead_code)]
const _: MouseButton = MouseButton::Left;

#[cfg(test)]
mod tests {
    use super::*;
    use std::net::TcpListener;

    #[test]
    fn mcp_bridge_persists_launcher_to_disk_and_forwards_live_commands_over_tcp() {
        let temp_root =
            std::env::temp_dir().join(format!("sawbyte-mcp-bridge-test-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&temp_root);

        // Start a real loopback TCP listener representing a running `sawbyte-play live --listen` process
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let addr = listener.local_addr().unwrap().to_string();
        let (stop_tx, stop_rx) = std::sync::mpsc::channel();
        let server_thread = std::thread::spawn(move || {
            listener.set_nonblocking(true).unwrap();
            while stop_rx.try_recv().is_err() {
                match listener.accept() {
                    Ok((stream, _)) => {
                        let _ = stream.set_nonblocking(false);
                        let mut writer = match stream.try_clone() {
                            Ok(w) => w,
                            Err(_) => continue,
                        };
                        let reader = BufReader::new(stream);
                        for line in reader.lines().map_while(std::result::Result::ok) {
                            let reply = format!("live_ack:{line}");
                            let _ = writeln!(writer, "{reply}");
                            let _ = writer.flush();
                        }
                    }
                    Err(ref e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                        std::thread::sleep(Duration::from_millis(5));
                    }
                    Err(_) => break,
                }
            }
        });

        let mut bridge = SawbyteControlBridge::with_root(temp_root.clone()).unwrap();
        bridge.live_endpoint = Some(addr.clone());

        let init_resp =
            bridge.handle_mcp_json_rpc(r#"{"jsonrpc":"2.0","id":1,"method":"initialize"}"#);
        assert!(init_resp.contains("sawbyte-mcp"));

        let script_req = format!(
            r#"{{"jsonrpc":"2.0","id":2,"method":"tools/call","params":{{"name":"code_execute","script":"launcher.create_instance rl-node-1 sawbyte-26.3 native\nsettings.set fov 95\nlauncher.backup rl-node-1\nlauncher.connect_live {addr}\ntask.navigate 4 -60 4\nwait.until screen_open 40 \"Chest\"\nwait.ticks 10\ngym.observe"}}}}"#
        );
        let call_resp = bridge.handle_mcp_json_rpc(&script_req);
        assert!(call_resp.contains("\"isError\":false"));
        assert!(call_resp.contains("live_ack:task.navigate 4 -60 4"));
        assert!(call_resp.contains(r#"live_ack:wait.until screen_open 40 \"Chest\""#));
        assert!(call_resp.contains("live_ack:wait.ticks 10"));
        assert!(call_resp.contains("live_ack:gym.observe"));

        // Verify disk persistence across reload
        let reloaded = PersistentLauncher::load_from(temp_root.clone()).unwrap();
        let inst = reloaded.instances.get("rl-node-1").unwrap();
        assert_eq!(inst.settings.get("fov").map(String::as_str), Some("95"));
        assert_eq!(inst.snapshot_count, 1);

        let _ = stop_tx.send(());
        let _ = server_thread.join();
        let _ = std::fs::remove_dir_all(&temp_root);
    }
}

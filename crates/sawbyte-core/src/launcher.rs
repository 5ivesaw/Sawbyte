#![allow(clippy::all, warnings, unused)]
#![allow(
    clippy::all,
    clippy::collapsible_if,
    clippy::collapsible_else_if,
    clippy::field_reassign_with_default,
    clippy::unnecessary_cast,
    clippy::too_many_arguments,
    clippy::needless_borrows_for_generic_args,
    clippy::manual_flatten,
    clippy::ptr_arg,
    clippy::redundant_closure,
    clippy::single_match
)]
//! Persistent disk-backed launcher (`~/.sawbyte/launcher.json` or `$SAWBYTE_HOME/launcher.json`)
//! managing isolated instances, `options.txt` files, token-free disk snapshots, account lockfiles,
//! and real `sawbyte-play` OS child process spawning and supervision.

use std::collections::BTreeMap;
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct VersionAdapterProfile {
    pub profile_id: String,
    pub minecraft_version: String,
    pub protocol_version: i32,
    pub native_verified_parity: bool,
    pub bridge_supported: bool,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ModRecord {
    pub filename: String,
    pub size_bytes: u64,
    pub is_bridge: bool,
}

pub fn built_in_version_profiles() -> Vec<VersionAdapterProfile> {
    vec![
        VersionAdapterProfile {
            profile_id: "sawbyte-26.3".into(),
            minecraft_version: "26.3".into(),
            protocol_version: 777,
            native_verified_parity: true,
            bridge_supported: true,
        },
        VersionAdapterProfile {
            profile_id: "fabric-26.3".into(),
            minecraft_version: "26.3".into(),
            protocol_version: 777,
            native_verified_parity: true,
            bridge_supported: true,
        },
        VersionAdapterProfile {
            profile_id: "fabric-minimal".into(),
            minecraft_version: "26.3".into(),
            protocol_version: 777,
            native_verified_parity: true,
            bridge_supported: true,
        },
        VersionAdapterProfile {
            profile_id: "sawbyte-full-26.3".into(),
            minecraft_version: "26.3".into(),
            protocol_version: 777,
            native_verified_parity: true,
            bridge_supported: true,
        },
        VersionAdapterProfile {
            profile_id: "sawbyte-26.2".into(),
            minecraft_version: "26.2".into(),
            protocol_version: 776,
            native_verified_parity: false,
            bridge_supported: false,
        },
    ]
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RuntimeMode {
    NativeStandalone,
    FullClientBridge,
    NativeMinimalVisualizer,
    HorizonRustEngine,
}

impl RuntimeMode {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::NativeStandalone => "native",
            Self::FullClientBridge => "bridge",
            Self::NativeMinimalVisualizer => "visualizer",
            Self::HorizonRustEngine => "horizon",
        }
    }

    pub fn parse(s: &str) -> Result<Self, String> {
        match s {
            "native" => Ok(Self::NativeStandalone),
            "bridge" => Ok(Self::FullClientBridge),
            "visualizer" | "native-visualizer" | "minimal" => Ok(Self::NativeMinimalVisualizer),
            "sawbyte-client" | "sawbyte-full" | "horizon" | "horizon-rust" | "horizon-offscreen" | "horizon-wgpu" => Ok(Self::HorizonRustEngine),
            _ => Err(format!(
                "unknown runtime mode '{s}'; expected 'native', 'bridge', 'visualizer', or 'horizon'"
            )),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct InstanceRecord {
    pub name: String,
    pub version_profile: String,
    pub mode: RuntimeMode,
    pub server: String,
    pub rpc_bind: String,
    pub settings: BTreeMap<String, String>,
    pub snapshot_count: u32,
    pub last_pid: Option<u32>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AccountRecord {
    pub username: String,
    pub uuid_hex: String,
    pub token_file: Option<String>,
    pub locked_by_instance: Option<String>,
}

pub struct PersistentLauncher {
    pub root_dir: PathBuf,
    pub instances: BTreeMap<String, InstanceRecord>,
    pub accounts: BTreeMap<String, AccountRecord>,
    pub version_profiles: Vec<VersionAdapterProfile>,
}

impl PersistentLauncher {
    pub fn resolve_home() -> PathBuf {
        if let Ok(custom) = std::env::var("SAWBYTE_HOME") {
            if !custom.trim().is_empty() {
                return PathBuf::from(custom);
            }
        }
        if let Ok(home) = std::env::var("HOME") {
            return PathBuf::from(home).join(".sawbyte");
        }
        if let Ok(profile) = std::env::var("USERPROFILE") {
            return PathBuf::from(profile).join(".sawbyte");
        }
        std::env::temp_dir().join("sawbyte-home")
    }

    pub fn load_or_init() -> Result<Self, String> {
        Self::load_from(Self::resolve_home())
    }

    pub fn load_from(root_dir: PathBuf) -> Result<Self, String> {
        fs::create_dir_all(root_dir.join("instances")).map_err(|e| e.to_string())?;
        fs::create_dir_all(root_dir.join("locks")).map_err(|e| e.to_string())?;
        fs::create_dir_all(root_dir.join("bin")).map_err(|e| e.to_string())?;
        fs::create_dir_all(root_dir.join("editions")).map_err(|e| e.to_string())?;
        let state_file = root_dir.join("launcher.db");
        let mut launcher = Self {
            root_dir,
            instances: BTreeMap::new(),
            accounts: BTreeMap::new(),
            version_profiles: built_in_version_profiles(),
        };
        if state_file.exists() {
            let content = fs::read_to_string(&state_file).map_err(|e| e.to_string())?;
            launcher.deserialize(&content)?;
        } else {
            launcher.create_instance(
                "default-native",
                "sawbyte-26.3",
                RuntimeMode::NativeStandalone,
            )?;
            launcher.add_account("SawbyteNative", "00000000000030008000000000000001", None)?;
            launcher.save()?;
        }
        Ok(launcher)
    }

    pub fn save(&self) -> Result<(), String> {
        fs::create_dir_all(&self.root_dir).map_err(|e| e.to_string())?;
        let tmp = self.root_dir.join("launcher.db.tmp");
        let target = self.root_dir.join("launcher.db");
        let mut out = String::from("# Sawbyte Persistent Launcher State v1\n");
        for inst in self.instances.values() {
            let pid_str = inst
                .last_pid
                .map_or_else(|| "none".into(), |p| p.to_string());
            out.push_str(&format!(
                "INSTANCE|{}|{}|{}|{}|{}|{}|{}\n",
                inst.name,
                inst.version_profile,
                inst.mode.as_str(),
                inst.server,
                inst.rpc_bind,
                inst.snapshot_count,
                pid_str
            ));
            for (k, v) in &inst.settings {
                out.push_str(&format!("SETTING|{}|{}|{}\n", inst.name, k, v));
            }
        }
        for acc in self.accounts.values() {
            let tok = acc.token_file.as_deref().unwrap_or("none");
            let lock = acc.locked_by_instance.as_deref().unwrap_or("none");
            out.push_str(&format!(
                "ACCOUNT|{}|{}|{}|{}\n",
                acc.username, acc.uuid_hex, tok, lock
            ));
        }
        fs::write(&tmp, out.as_bytes()).map_err(|e| e.to_string())?;
        fs::rename(&tmp, &target).map_err(|e| e.to_string())?;
        Ok(())
    }

    fn deserialize(&mut self, text: &str) -> Result<(), String> {
        self.instances.clear();
        self.accounts.clear();
        for raw in text.lines() {
            let line = raw.trim();
            if line.is_empty() || line.starts_with('#') {
                continue;
            }
            let parts: Vec<&str> = line.split('|').collect();
            match parts.as_slice() {
                ["INSTANCE", name, vp, mode, server, rpc, snaps, pid] => {
                    self.instances.insert(
                        (*name).to_owned(),
                        InstanceRecord {
                            name: (*name).to_owned(),
                            version_profile: (*vp).to_owned(),
                            mode: RuntimeMode::parse(mode)?,
                            server: (*server).to_owned(),
                            rpc_bind: (*rpc).to_owned(),
                            settings: BTreeMap::new(),
                            snapshot_count: snaps.parse().unwrap_or(0),
                            last_pid: (*pid != "none").then(|| pid.parse().ok()).flatten(),
                        },
                    );
                }
                ["SETTING", inst_name, key, val] => {
                    if let Some(inst) = self.instances.get_mut(*inst_name) {
                        inst.settings.insert((*key).to_owned(), (*val).to_owned());
                    }
                }
                ["ACCOUNT", user, uuid, tok, lock] => {
                    self.accounts.insert(
                        (*user).to_owned(),
                        AccountRecord {
                            username: (*user).to_owned(),
                            uuid_hex: (*uuid).to_owned(),
                            token_file: (*tok != "none").then(|| (*tok).to_owned()),
                            locked_by_instance: (*lock != "none").then(|| (*lock).to_owned()),
                        },
                    );
                }
                _ => {}
            }
        }
        Ok(())
    }

    pub fn instance_dir(&self, name: &str) -> PathBuf {
        self.root_dir.join("instances").join(name)
    }

    pub fn create_instance(
        &mut self,
        name: &str,
        version_profile: &str,
        mode: RuntimeMode,
    ) -> Result<(), String> {
        if name.is_empty()
            || name.len() > 48
            || !name
                .bytes()
                .all(|b| b.is_ascii_alphanumeric() || b == b'-' || b == b'_')
        {
            return Err("invalid instance name (use ASCII alphanumeric, '-', or '_')".into());
        }
        if !self
            .version_profiles
            .iter()
            .any(|v| v.profile_id == version_profile)
        {
            return Err(format!("unknown version profile '{version_profile}'"));
        }
        let mut settings = BTreeMap::new();
        settings.insert("fov".into(), "70".into());
        settings.insert("renderDistance".into(), "3".into());
        settings.insert("simulationDistance".into(), "3".into());
        settings.insert("maxFps".into(), "60".into());
        settings.insert("autoJump".into(), "false".into());
        settings.insert("pauseOnLostFocus".into(), "false".into());
        settings.insert("policy".into(), "humanised".into());
        if version_profile == "fabric-minimal" || mode == RuntimeMode::NativeMinimalVisualizer {
            settings.insert("graphicsMode".into(), "0".into());
            settings.insert("renderDistance".into(), "2".into());
            settings.insert("simulationDistance".into(), "2".into());
            settings.insert("smoothLighting".into(), "0".into());
            settings.insert("particles".into(), "2".into());
            settings.insert("entityShadows".into(), "false".into());
            settings.insert("clouds".into(), "0".into());
            settings.insert("biomeBlendRadius".into(), "0".into());
            settings.insert("maxFps".into(), "30".into());
            settings.insert("mipmapLevels".into(), "0".into());
            settings.insert("viewBobbing".into(), "false".into());
        }
        let dir = self.instance_dir(name);
        fs::create_dir_all(dir.join("snapshots")).map_err(|e| e.to_string())?;
        fs::create_dir_all(dir.join("mods")).map_err(|e| e.to_string())?;
        Self::write_options_file(&dir.join("options.txt"), &settings)?;
        self.instances.insert(
            name.to_owned(),
            InstanceRecord {
                name: name.to_owned(),
                version_profile: version_profile.to_owned(),
                mode,
                server: "127.0.0.1:25565".into(),
                rpc_bind: "127.0.0.1:25580".into(),
                settings,
                snapshot_count: 0,
                last_pid: None,
            },
        );
        self.save()
    }

    pub fn delete_instance(&mut self, name: &str) -> Result<(), String> {
        if self.instances.remove(name).is_none() {
            return Err(format!("instance '{name}' not found"));
        }
        let dir = self.instance_dir(name);
        if dir.exists() {
            let _ = fs::remove_dir_all(&dir);
        }
        self.save()
    }

    pub fn delete_account(&mut self, username: &str) -> Result<(), String> {
        if self.accounts.remove(username).is_none() {
            return Err(format!("account '{username}' not found"));
        }
        self.save()
    }

    pub fn set_setting(
        &mut self,
        instance_name: &str,
        key: &str,
        value: &str,
    ) -> Result<(), String> {
        if key.is_empty()
            || key.contains('|')
            || value.contains('|')
            || key.contains('\n')
            || value.contains('\n')
        {
            return Err("invalid setting key or value".into());
        }
        let dir = self.instance_dir(instance_name);
        let inst = self
            .instances
            .get_mut(instance_name)
            .ok_or_else(|| format!("instance '{instance_name}' not found"))?;
        if key == "server" {
            inst.server = value.to_owned();
        } else if key == "rpc_bind" {
            inst.rpc_bind = value.to_owned();
        } else {
            inst.settings.insert(key.to_owned(), value.to_owned());
        }
        fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
        Self::write_options_file(&dir.join("options.txt"), &inst.settings)?;
        self.save()
    }

    fn write_options_file(path: &Path, settings: &BTreeMap<String, String>) -> Result<(), String> {
        let mut out = String::new();
        for (k, v) in settings {
            out.push_str(&format!("{k}:{v}\n"));
        }
        fs::write(path, out.as_bytes()).map_err(|e| e.to_string())
    }

    pub fn mods_dir(&self, instance_name: &str) -> PathBuf {
        self.instance_dir(instance_name).join("mods")
    }

    pub fn list_mods(&self, instance_name: &str) -> Result<Vec<ModRecord>, String> {
        let dir = self.mods_dir(instance_name);
        if !dir.exists() {
            return Ok(Vec::new());
        }
        let mut list = Vec::new();
        let entries = fs::read_dir(&dir).map_err(|e| e.to_string())?;
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_file() && path.extension().is_some_and(|ext| ext == "jar") {
                let filename = path
                    .file_name()
                    .unwrap_or_default()
                    .to_string_lossy()
                    .to_string();
                let size_bytes = fs::metadata(&path).map(|m| m.len()).unwrap_or(0);
                let is_bridge = filename.starts_with("sawbyte-bridge");
                list.push(ModRecord {
                    filename,
                    size_bytes,
                    is_bridge,
                });
            }
        }
        list.sort_by(|a, b| a.filename.cmp(&b.filename));
        Ok(list)
    }

    pub fn remove_mod(&self, instance_name: &str, filename: &str) -> Result<(), String> {
        if filename.contains('/') || filename.contains('\\') || !filename.ends_with(".jar") {
            return Err("invalid mod filename".into());
        }
        let path = self.mods_dir(instance_name).join(filename);
        if path.exists() {
            fs::remove_file(&path).map_err(|e| e.to_string())?;
        }
        Ok(())
    }

    pub fn install_mod_file(
        &self,
        instance_name: &str,
        filename: &str,
        url: &str,
    ) -> Result<PathBuf, String> {
        if filename.contains('/') || filename.contains('\\') || !filename.ends_with(".jar") {
            return Err("invalid mod filename; must end with .jar".into());
        }
        let mods_dir = self.mods_dir(instance_name);
        fs::create_dir_all(&mods_dir).map_err(|e| e.to_string())?;
        let target = mods_dir.join(filename);

        let status = Command::new("curl")
            .args(["-sSL", "-o", target.to_str().unwrap(), url])
            .status()
            .map_err(|e| format!("failed to execute curl: {e}"))?;
        if !status.success() {
            return Err(format!(
                "curl download failed with status {status} for {url}"
            ));
        }
        Ok(target)
    }

    pub fn inject_bridge_mod(&self, instance_name: &str) -> Result<PathBuf, String> {
        let mods_dir = self.mods_dir(instance_name);
        fs::create_dir_all(&mods_dir).map_err(|e| e.to_string())?;
        let target = mods_dir.join("sawbyte-bridge-0.3.4.jar");

        let mut found = None;
        if let Ok(current_exe) = std::env::current_exe() {
            if let Some(parent) = current_exe.parent() {
                for name in [
                    "sawbyte-bridge-0.3.4.jar",
                    "sawbyte-bridge-0.3.2.jar",
                    "sawbyte-bridge-0.3.1.jar",
                    "sawbyte-bridge-26.3.jar",
                ] {
                    let p = parent.join(name);
                    if p.exists() {
                        found = Some(p);
                        break;
                    }
                }
            }
        }
        if found.is_none() {
            for cand in [
                "sawbyte-bridge-0.3.4.jar",
                "sawbyte-bridge-0.3.2.jar",
                "sawbyte-bridge-0.3.1.jar",
                "sawbyte-bridge-26.3.jar",
                "bridge/build/libs/sawbyte-bridge-0.3.3.jar",
                "bridge/build/libs/sawbyte-bridge-0.3.2.jar",
                "bridge/build/libs/sawbyte-bridge-0.3.1.jar",
                "bridge/build/libs/sawbyte-bridge-26.3.jar",
                "run/bin/sawbyte-bridge-0.3.3.jar",
                "run/bin/sawbyte-bridge-0.3.2.jar",
                "run/bin/sawbyte-bridge-0.3.1.jar",
                "release-assets/sawbyte-bridge-0.3.3.jar",
                "release-assets/sawbyte-bridge-0.3.2.jar",
                "release-assets/sawbyte-bridge-0.3.1.jar",
            ] {
                let p = PathBuf::from(cand);
                if p.exists() {
                    found = Some(p);
                    break;
                }
            }
        }

        if let Some(src) = found {
            fs::copy(&src, &target).map_err(|e| e.to_string())?;
            return Ok(target);
        }

        for home_cand in [
            self.root_dir.join("editions").join("sawbyte-bridge-0.3.4.jar"),
            self.root_dir.join("editions").join("sawbyte-bridge-26.3.jar"),
            self.root_dir.join("bin").join("sawbyte-bridge-0.3.4.jar"),
        ] {
            if home_cand.exists() {
                fs::copy(&home_cand, &target).map_err(|e| e.to_string())?;
                return Ok(target);
            }
        }

        // If not found locally, fetch official release asset (latest resolver with tag fallback)
        let latest_url =
            "https://github.com/5ivesaw/Sawbyte/releases/latest/download/sawbyte-bridge-0.3.3.jar";
        if let Ok(installed) = self.install_mod_file(instance_name, "sawbyte-bridge-0.3.4.jar", latest_url) {
            let _ = fs::copy(&installed, self.root_dir.join("editions").join("sawbyte-bridge-0.3.4.jar"));
            return Ok(installed);
        }
        let release_url =
            "https://github.com/5ivesaw/Sawbyte/releases/download/v0.3.4/sawbyte-bridge-0.3.4.jar";
        let installed = self.install_mod_file(instance_name, "sawbyte-bridge-0.3.4.jar", release_url)?;
        let _ = fs::copy(&installed, self.root_dir.join("editions").join("sawbyte-bridge-0.3.4.jar"));
        Ok(installed)
    }

    /// Writes a token-free snapshot file on disk under `<instance>/snapshots/snapshot-<id>.cfg`.
    pub fn backup_instance(&mut self, name: &str) -> Result<u32, String> {
        let dir = self.instance_dir(name);
        let inst = self
            .instances
            .get_mut(name)
            .ok_or_else(|| format!("instance '{name}' not found"))?;
        let next_id = inst.snapshot_count + 1;
        let snap_dir = dir.join("snapshots");
        fs::create_dir_all(&snap_dir).map_err(|e| e.to_string())?;
        let snap_path = snap_dir.join(format!("snapshot-{next_id}.cfg"));
        let mut data = format!(
            "version_profile={}\nmode={}\nserver={}\nrpc_bind={}\n",
            inst.version_profile,
            inst.mode.as_str(),
            inst.server,
            inst.rpc_bind
        );
        for (k, v) in &inst.settings {
            if !k.contains("token") && !k.contains("secret") {
                data.push_str(&format!("setting.{k}={v}\n"));
            }
        }
        fs::write(&snap_path, data.as_bytes()).map_err(|e| e.to_string())?;
        inst.snapshot_count = next_id;
        self.save()?;
        Ok(next_id)
    }

    pub fn rollback_instance(&mut self, name: &str, snapshot_id: u32) -> Result<(), String> {
        let dir = self.instance_dir(name);
        let snap_path = dir
            .join("snapshots")
            .join(format!("snapshot-{snapshot_id}.cfg"));
        if !snap_path.exists() {
            return Err(format!(
                "snapshot #{snapshot_id} does not exist on disk for '{name}'"
            ));
        }
        let content = fs::read_to_string(&snap_path).map_err(|e| e.to_string())?;
        let inst = self
            .instances
            .get_mut(name)
            .ok_or_else(|| format!("instance '{name}' not found"))?;
        let mut restored_settings = BTreeMap::new();
        for line in content.lines() {
            if let Some((k, v)) = line.split_once('=') {
                match k {
                    "version_profile" => inst.version_profile = v.to_owned(),
                    "mode" => inst.mode = RuntimeMode::parse(v)?,
                    "server" => inst.server = v.to_owned(),
                    "rpc_bind" => inst.rpc_bind = v.to_owned(),
                    _ if k.starts_with("setting.") => {
                        restored_settings.insert(k["setting.".len()..].to_owned(), v.to_owned());
                    }
                    _ => {}
                }
            }
        }
        inst.settings = restored_settings;
        Self::write_options_file(&dir.join("options.txt"), &inst.settings)?;
        self.save()
    }

    pub fn validate_username(username: &str) -> Result<(), String> {
        if username.is_empty()
            || username.len() > 16
            || !username
                .bytes()
                .all(|b| b.is_ascii_alphanumeric() || b == b'_')
        {
            return Err("invalid username (must be 1..=16 ASCII alphanumeric or '_')".into());
        }
        Ok(())
    }

    pub fn add_account(
        &mut self,
        username: &str,
        uuid_hex: &str,
        token_file: Option<&str>,
    ) -> Result<(), String> {
        Self::validate_username(username)?;
        if uuid_hex.len() != 32 || !uuid_hex.bytes().all(|b| b.is_ascii_hexdigit()) {
            return Err("invalid uuid_hex (must be 32 ASCII hexadecimal characters)".into());
        }
        let existing_lock = self
            .accounts
            .get(username)
            .and_then(|a| a.locked_by_instance.clone());
        self.accounts.insert(
            username.to_owned(),
            AccountRecord {
                username: username.to_owned(),
                uuid_hex: uuid_hex.to_owned(),
                token_file: token_file.map(str::to_owned),
                locked_by_instance: existing_lock,
            },
        );
        self.save()
    }

    pub fn acquire_account_lock(
        &mut self,
        username: &str,
        instance_name: &str,
    ) -> Result<(), String> {
        Self::validate_username(username)?;
        let lock_file = self.root_dir.join("locks").join(format!("{username}.lock"));
        let acc = self
            .accounts
            .get_mut(username)
            .ok_or_else(|| format!("account '{username}' not found"))?;
        if let Some(holder) = &acc.locked_by_instance {
            return Err(format!(
                "account '{username}' is already locked by instance '{holder}'"
            ));
        }
        let mut file = fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&lock_file)
            .map_err(|e| format!("cannot acquire account lock for '{username}': {e}"))?;
        if let Err(error) = file.write_all(instance_name.as_bytes()) {
            drop(file);
            let _ = fs::remove_file(&lock_file);
            return Err(format!(
                "cannot write account lock for '{username}': {error}"
            ));
        }
        drop(file);
        acc.locked_by_instance = Some(instance_name.to_owned());
        if let Err(error) = self.save() {
            if let Some(acc) = self.accounts.get_mut(username) {
                acc.locked_by_instance = None;
            }
            let _ = fs::remove_file(&lock_file);
            return Err(error);
        }
        Ok(())
    }

    pub fn release_account_lock(&mut self, username: &str) -> Result<(), String> {
        Self::validate_username(username)?;
        let lock_file = self.root_dir.join("locks").join(format!("{username}.lock"));
        let _ = fs::remove_file(&lock_file);
        if let Some(acc) = self.accounts.get_mut(username) {
            acc.locked_by_instance = None;
        }
        self.save()
    }

    pub fn locate_sawbyte_play_binary() -> Result<PathBuf, String> {
        if let Ok(env_bin) = std::env::var("SAWBYTE_PLAY_BIN") {
            let p = PathBuf::from(env_bin);
            if p.exists() {
                return Ok(p);
            }
        }
        let exe_ext = if cfg!(windows) { ".exe" } else { "" };
        let bin_name = format!("sawbyte-play{exe_ext}");
        if let Ok(current_exe) = std::env::current_exe() {
            if let Some(parent) = current_exe.parent() {
                let sibling = parent.join(&bin_name);
                if sibling.exists() {
                    return Ok(sibling);
                }
            }
        }
        let home_bin = Self::resolve_home().join("bin").join(&bin_name);
        if home_bin.exists() {
            return Ok(home_bin);
        }
        if let Ok(home) = std::env::var("HOME") {
            let local_bin = PathBuf::from(home).join(".local/bin").join(&bin_name);
            if local_bin.exists() {
                return Ok(local_bin);
            }
        }
        if let Ok(local_app) = std::env::var("LOCALAPPDATA") {
            let win_bin = PathBuf::from(local_app)
                .join("Programs")
                .join("Sawbyte")
                .join(&bin_name);
            if win_bin.exists() {
                return Ok(win_bin);
            }
        }
        for candidate in [
            PathBuf::from(format!("native/target/release/{bin_name}")),
            PathBuf::from(format!("native/target/debug/{bin_name}")),
            PathBuf::from(format!("run/bin/{bin_name}")),
            PathBuf::from(format!("target/release/{bin_name}")),
        ] {
            if candidate.exists() {
                return Ok(candidate);
            }
        }
        Err(format!(
            "could not locate '{bin_name}' binary; install it via the Launcher Editions tab or place '{bin_name}' in ~/.sawbyte/bin"
        ))
    }

    /// Ensures `sawbyte-play` is available locally; if not present on disk, automatically
    /// indexes and downloads the platform release archive from GitHub Releases into `~/.sawbyte/bin/`.
    pub fn ensure_sawbyte_play_binary(&self) -> Result<PathBuf, String> {
        if let Ok(existing) = Self::locate_sawbyte_play_binary() {
            return Ok(existing);
        }
        let bin_dir = self.root_dir.join("bin");
        fs::create_dir_all(&bin_dir).map_err(|e| e.to_string())?;
        let exe_ext = if cfg!(windows) { ".exe" } else { "" };
        let bin_name = format!("sawbyte-play{exe_ext}");
        let target_bin = bin_dir.join(&bin_name);

        let artifact = if cfg!(target_os = "windows") {
            "sawbyte-play-windows-x64.zip"
        } else if cfg!(target_os = "macos") {
            "sawbyte-play-macos-arm64.tar.gz"
        } else {
            "sawbyte-play-linux-x64.tar.gz"
        };

        let archive_path = bin_dir.join(artifact);
        let latest_url = format!(
            "https://github.com/5ivesaw/Sawbyte/releases/latest/download/{artifact}"
        );
        let fallback_url = format!(
            "https://github.com/5ivesaw/Sawbyte/releases/download/v0.3.3/{artifact}"
        );

        let dl_ok = Command::new("curl")
            .args(["-fsSL", "-o", archive_path.to_str().unwrap_or(""), &latest_url])
            .status()
            .is_ok_and(|s| s.success())
            || Command::new("curl")
                .args(["-fsSL", "-o", archive_path.to_str().unwrap_or(""), &fallback_url])
                .status()
                .is_ok_and(|s| s.success());

        if !dl_ok || !archive_path.exists() {
            return Err(format!(
                "could not locate or download '{bin_name}' from GitHub Releases ({latest_url})"
            ));
        }

        if artifact.ends_with(".zip") {
            #[cfg(windows)]
            {
                let ps_cmd = format!(
                    "Expand-Archive -Path '{}' -DestinationPath '{}' -Force",
                    archive_path.display(),
                    bin_dir.display()
                );
                let _ = Command::new("powershell")
                    .args(["-NoProfile", "-Command", &ps_cmd])
                    .status();
            }
            #[cfg(not(windows))]
            {
                let _ = Command::new("unzip")
                    .args([
                        "-o",
                        "-q",
                        archive_path.to_str().unwrap_or(""),
                        "-d",
                        bin_dir.to_str().unwrap_or(""),
                    ])
                    .status();
            }
        } else {
            let _ = Command::new("tar")
                .args([
                    "-xzf",
                    archive_path.to_str().unwrap_or(""),
                    "-C",
                    bin_dir.to_str().unwrap_or(""),
                ])
                .status();
        }
        let _ = fs::remove_file(&archive_path);

        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            if target_bin.exists() {
                let _ = fs::set_permissions(&target_bin, fs::Permissions::from_mode(0o755));
            }
        }

        if target_bin.exists() {
            Ok(target_bin)
        } else {
            Err(format!(
                "downloaded {artifact}, but '{bin_name}' was not found after extraction"
            ))
        }
    }

    pub fn ensure_block_profile(&self, requested: &Path) -> PathBuf {
        if requested.exists() {
            return requested.to_path_buf();
        }
        for cand in [
            self.root_dir.join("block-profile.json"),
            self.root_dir.join("bin").join("block-profile.json"),
            PathBuf::from("block-profile.json"),
            PathBuf::from("run/reports/block-profile.json"),
        ] {
            if cand.exists() {
                return cand;
            }
        }
        if let Ok(exe) = std::env::current_exe() {
            if let Some(parent) = exe.parent() {
                let sib = parent.join("block-profile.json");
                if sib.exists() {
                    return sib;
                }
            }
        }
        let fallback = self.root_dir.join("block-profile.json");
        let default_json = "{\n  \"minecraft\": \"26.3\",\n  \"air\": [0],\n  \"solid\": [1],\n  \"bottom_slabs\": [],\n  \"top_slabs\": [],\n  \"stairs\": [],\n  \"fences\": [],\n  \"carpets\": [],\n  \"climbable\": [],\n  \"water\": [],\n  \"lava\": [],\n  \"cobwebs\": [],\n  \"soul_sand\": [],\n  \"ice\": [],\n  \"hardness\": {\n    \"1\": 1.5\n  }\n}\n";
        let _ = fs::create_dir_all(&self.root_dir);
        let _ = fs::write(&fallback, default_json.as_bytes());
        fallback
    }

    /// Spawns the real `sawbyte-play` headless client process connected to `server`, persists its PID
    /// and log file on disk, and returns the spawned `Child`.
    pub fn spawn_live_process(
        &mut self,
        instance_name: &str,
        server_override: Option<&str>,
        account_name: &str,
        blocks_profile_path: &Path,
        extra_args: &[String],
    ) -> Result<(Child, PathBuf), String> {
        let (inst_mode, server, rpc_bind) = {
            let inst = self
                .instances
                .get(instance_name)
                .ok_or_else(|| format!("instance '{instance_name}' not found"))?;
            (
                inst.mode,
                server_override.unwrap_or(&inst.server).to_owned(),
                inst.rpc_bind.clone(),
            )
        };
        if inst_mode == RuntimeMode::FullClientBridge {
            return self.spawn_fabric_process(
                instance_name,
                server_override,
                account_name,
                extra_args,
            );
        }
        if inst_mode == RuntimeMode::HorizonRustEngine {
            if let Ok(res) = self.spawn_horizon_process(
                instance_name,
                server_override,
                account_name,
                extra_args,
            ) {
                return Ok(res);
            }
        }
        let (acc_token_file, acc_username) = {
            let acc = self
                .accounts
                .get(account_name)
                .ok_or_else(|| format!("account '{account_name}' not found"))?;
            (acc.token_file.clone(), acc.username.clone())
        };

        let bin = self.ensure_sawbyte_play_binary()?;
        let dir = self.instance_dir(instance_name);
        fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
        let log_path = dir.join("session.log");
        let log_file = fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(&log_path)
            .map_err(|e| e.to_string())?;
        let err_file = log_file.try_clone().map_err(|e| e.to_string())?;

        let mut cmd = Command::new(&bin);
        cmd.arg("live").arg(&server);
        if let Some(tok) = &acc_token_file {
            cmd.arg("--account").arg(tok);
        } else {
            cmd.arg("--offline").arg(&acc_username);
        }
        let resolved_profile = self.ensure_block_profile(blocks_profile_path);
        cmd.arg(&resolved_profile).arg("--listen").arg(&rpc_bind);
        for arg in extra_args {
            cmd.arg(arg);
        }
        cmd.stdout(Stdio::from(log_file))
            .stderr(Stdio::from(err_file));

        self.acquire_account_lock(account_name, instance_name)?;
        let mut child = cmd.spawn().map_err(|e| {
            let _ = self.release_account_lock(account_name);
            format!("failed to spawn {}: {e}", bin.display())
        })?;
        let pid = child.id();
        let pid_path = dir.join("session.pid");
        let persisted = fs::write(&pid_path, format!("{pid}\n"));
        if let Err(error) = persisted {
            let _ = child.kill();
            let _ = child.wait();
            let _ = self.release_account_lock(account_name);
            return Err(format!("failed to persist session PID: {error}"));
        }
        if let Some(inst_mut) = self.instances.get_mut(instance_name) {
            inst_mut.last_pid = Some(pid);
        }
        if let Err(error) = self.save() {
            let _ = child.kill();
            let _ = child.wait();
            let _ = fs::remove_file(pid_path);
            if let Some(inst_mut) = self.instances.get_mut(instance_name) {
                inst_mut.last_pid = None;
            }
            let _ = self.release_account_lock(account_name);
            return Err(error);
        }
        Ok((child, log_path))
    }

    /// Spawns the Fabric Minecraft client process targeting the instance's directory and `mods/`,
    /// automatically injecting the `sawbyte-bridge` framework mod if missing.

    /// Locates or spawns the full `horizon-rust/horizon` 26.3 engine (`horizon-client` / `horizon-ctl`),
    /// supporting `--offscreen` (Headless/Half-Headless) and full `wgpu` graphical modes, singleplayer
    /// server-thread worlds (`--world`), and multiplayer servers (`connect <server> <account>`).
    pub fn spawn_horizon_process(
        &mut self,
        instance_name: &str,
        server_override: Option<&str>,
        account_name: &str,
        extra_args: &[String],
    ) -> Result<(Child, PathBuf), String> {
        let (target, is_offscreen) = {
            let inst = self
                .instances
                .get(instance_name)
                .ok_or_else(|| format!("instance '{instance_name}' not found"))?;
            (
                server_override.unwrap_or(&inst.server).to_owned(),
                inst.settings.get("offscreen").map(String::as_str) == Some("true"),
            )
        };
        let dir = self.instance_dir(instance_name);
        fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
        let log_path = dir.join("session.log");
        let log_file = fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(&log_path)
            .map_err(|e| e.to_string())?;
        let err_file = log_file.try_clone().map_err(|e| e.to_string())?;

        let exe_ext = if cfg!(windows) { ".exe" } else { "" };
        let ctl_name = format!("sawbyte-ctl{exe_ext}");
        let client_name = format!("sawbyte-client{exe_ext}");

        let mut ctl_bin = None;
        for cand in [
            self.root_dir.join("bin").join(&ctl_name),
            self.root_dir.join("horizon").join("target").join("release").join(&ctl_name),
            self.root_dir.join("horizon").join("target").join("fast").join(&ctl_name),
            PathBuf::from(format!("horizon/target/release/{ctl_name}")),
            PathBuf::from(format!("horizon/target/fast/{ctl_name}")),
        ] {
            if cand.exists() {
                ctl_bin = Some(cand);
                break;
            }
        }
        let Some(bin) = ctl_bin else {
            return Err(format!("horizon-ctl binary not found yet; run './replace_all.sh --with-horizon' to fetch and build horizon-rust/horizon ({client_name} + {ctl_name})"));
        };

        let sock_path = dir.join("sawbyte-control.sock");
        let mut cmd = Command::new(&bin);
        cmd.arg("--socket").arg(&sock_path).arg("launch");
        if target == "singleplayer" || target.starts_with("singleplayer:") {
            let world_name = target.split_once(':').map(|x| x.1).unwrap_or("Sawbyte_26_3_World");
            cmd.arg("--world").arg(world_name);
        }
        if is_offscreen {
            cmd.arg("--offscreen");
        }
        for arg in extra_args {
            cmd.arg(arg);
        }
        cmd.stdout(Stdio::from(log_file)).stderr(Stdio::from(err_file));
        let child = cmd.spawn().map_err(|e| format!("failed to spawn horizon-ctl: {e}"))?;
        let pid = child.id();
        let _ = fs::write(dir.join("session.pid"), format!("{pid}\n"));
        if let Some(inst_mut) = self.instances.get_mut(instance_name) {
            inst_mut.last_pid = Some(pid);
        }
        let _ = self.save();
        let _ = account_name;
        Ok((child, log_path))
    }

    pub fn spawn_fabric_process(
        &mut self,
        instance_name: &str,
        server_override: Option<&str>,
        account_name: &str,
        extra_args: &[String],
    ) -> Result<(Child, PathBuf), String> {
        let _inst = self
            .instances
            .get(instance_name)
            .ok_or_else(|| format!("instance '{instance_name}' not found"))?;
        if !self.accounts.contains_key(account_name) {
            return Err(format!("account '{account_name}' not found"));
        }
        let dir = self.instance_dir(instance_name);
        fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
        let log_path = dir.join("session.log");
        let log_file = fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(&log_path)
            .map_err(|e| e.to_string())?;
        let err_file = log_file.try_clone().map_err(|e| e.to_string())?;

        // Automatically inject Sawbyte Bridge Mod into mods/ if not already present
        let _ = self.inject_bridge_mod(instance_name);

        let java_bin = std::env::var("JAVA_HOME")
            .map(|j| format!("{j}/bin/java"))
            .unwrap_or_else(|_| "java".to_string());

        let mut cmd = Command::new(&java_bin);
        cmd.current_dir(&dir);

        let mut jar_found = None;
        for jar in [
            "fabric-loader.jar",
            "fabric.jar",
            "client.jar",
            "minecraft.jar",
        ] {
            let p = dir.join(jar);
            if p.exists() {
                jar_found = Some(p);
                break;
            }
        }

        if jar_found.is_none() {
            let installer_jar = self.root_dir.join("editions").join("fabric-installer.jar");
            if !installer_jar.exists() {
                let _ = fs::create_dir_all(self.root_dir.join("editions"));
                let fabric_url = "https://maven.fabricmc.net/net/fabricmc/fabric-installer/1.0.1/fabric-installer-1.0.1.jar";
                let _ = Command::new("curl")
                    .args(["-fsSL", "-o", installer_jar.to_str().unwrap_or(""), fabric_url])
                    .status();
            }
            if installer_jar.exists() {
                let _ = Command::new(&java_bin)
                    .args([
                        "-jar",
                        installer_jar.to_str().unwrap_or(""),
                        "client",
                        "-dir",
                        dir.to_str().unwrap_or(""),
                        "-noprofile",
                    ])
                    .stdout(Stdio::null())
                    .stderr(Stdio::null())
                    .status();
                for jar in ["fabric-loader.jar", "fabric.jar", "client.jar"] {
                    let p = dir.join(jar);
                    if p.exists() {
                        jar_found = Some(p);
                        break;
                    }
                }
            }
        }

        if let Some(jar) = jar_found {
            cmd.arg("-jar").arg(&jar);
            cmd.arg(format!("--gameDir={}", dir.display()));
        } else {
            // Fabric standard runner arguments
            cmd.arg(format!("-Dfabric.gameDir={}", dir.display()));
            cmd.arg(format!(
                "-Dfabric.modsFolder={}",
                self.mods_dir(instance_name).display()
            ));
            cmd.arg("-version");
        }

        if let Some(srv) = server_override {
            cmd.arg("--server").arg(srv);
        }
        for arg in extra_args {
            cmd.arg(arg);
        }
        cmd.stdout(Stdio::from(log_file))
            .stderr(Stdio::from(err_file));

        self.acquire_account_lock(account_name, instance_name)?;
        let child = cmd.spawn().map_err(|e| {
            let _ = self.release_account_lock(account_name);
            format!(
                "failed to launch Fabric client ({java_bin}): {e}. Ensure Java is installed and in PATH."
            )
        })?;
        let pid = child.id();
        let pid_path = dir.join("session.pid");
        let _ = fs::write(pid_path, format!("{pid}\n"));
        if let Some(inst_mut) = self.instances.get_mut(instance_name) {
            inst_mut.last_pid = Some(pid);
        }
        let _ = self.save();
        Ok((child, log_path))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn launcher_rejects_invalid_usernames_and_unsafe_paths() {
        let temp_root =
            std::env::temp_dir().join(format!("sawbyte-launcher-path-test-{}", std::process::id()));
        let _ = fs::remove_dir_all(&temp_root);
        let mut l = PersistentLauncher::load_from(temp_root.clone()).unwrap();

        assert!(l
            .add_account("../traversal", "00000000000030008000000000000001", None)
            .is_err());
        assert!(l
            .add_account("user with space", "00000000000030008000000000000001", None)
            .is_err());
        assert!(l.add_account("valid_user", "not-hex-uuid", None).is_err());
        assert!(l
            .add_account("valid_user", "00000000000030008000000000000001", None)
            .is_ok());

        assert!(l
            .acquire_account_lock("../traversal", "default-native")
            .is_err());

        let profiles = built_in_version_profiles();
        let p26_2 = profiles
            .iter()
            .find(|p| p.profile_id == "sawbyte-26.2")
            .unwrap();
        assert!(!p26_2.bridge_supported);

        let _ = fs::remove_dir_all(&temp_root);
    }

    #[test]
    fn launcher_persists_instances_settings_snapshots_and_locks_across_reloads() {
        let temp_root =
            std::env::temp_dir().join(format!("sawbyte-launcher-test-{}", std::process::id()));
        let _ = fs::remove_dir_all(&temp_root);

        {
            let mut l1 = PersistentLauncher::load_from(temp_root.clone()).unwrap();
            let mut competing = PersistentLauncher::load_from(temp_root.clone()).unwrap();
            l1.create_instance("bot-alpha", "sawbyte-26.3", RuntimeMode::NativeStandalone)
                .unwrap();
            l1.set_setting("bot-alpha", "fov", "95").unwrap();
            let snap_id = l1.backup_instance("bot-alpha").unwrap();
            assert_eq!(snap_id, 1);
            l1.set_setting("bot-alpha", "fov", "110").unwrap();
            l1.acquire_account_lock("SawbyteNative", "bot-alpha")
                .unwrap();
            assert!(competing
                .acquire_account_lock("SawbyteNative", "bot-alpha")
                .is_err());
        }

        // Reload from disk in a fresh struct instance and verify nothing was lost
        {
            let mut l2 = PersistentLauncher::load_from(temp_root.clone()).unwrap();
            let inst = l2.instances.get("bot-alpha").unwrap();
            assert_eq!(inst.settings.get("fov").map(String::as_str), Some("110"));
            assert_eq!(inst.snapshot_count, 1);
            assert!(l2
                .acquire_account_lock("SawbyteNative", "default-native")
                .is_err());

            l2.rollback_instance("bot-alpha", 1).unwrap();
            let inst_after = l2.instances.get("bot-alpha").unwrap();
            assert_eq!(
                inst_after.settings.get("fov").map(String::as_str),
                Some("95")
            );
            let options_on_disk =
                fs::read_to_string(l2.instance_dir("bot-alpha").join("options.txt")).unwrap();
            assert!(options_on_disk.contains("fov:95"));
            l2.release_account_lock("SawbyteNative").unwrap();
        }

        let _ = fs::remove_dir_all(&temp_root);
    }
}

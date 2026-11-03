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
            profile_id: "sawbyte-26.2".into(),
            minecraft_version: "26.2".into(),
            protocol_version: 776,
            native_verified_parity: false,
            bridge_supported: true,
        },
    ]
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RuntimeMode {
    NativeStandalone,
    FullClientBridge,
}

impl RuntimeMode {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::NativeStandalone => "native",
            Self::FullClientBridge => "bridge",
        }
    }

    pub fn parse(s: &str) -> Result<Self, String> {
        match s {
            "native" => Ok(Self::NativeStandalone),
            "bridge" => Ok(Self::FullClientBridge),
            _ => Err(format!(
                "unknown runtime mode '{s}'; expected 'native' or 'bridge'"
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
        let dir = self.instance_dir(name);
        fs::create_dir_all(dir.join("snapshots")).map_err(|e| e.to_string())?;
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

    pub fn add_account(
        &mut self,
        username: &str,
        uuid_hex: &str,
        token_file: Option<&str>,
    ) -> Result<(), String> {
        if username.is_empty() || username.len() > 16 {
            return Err("invalid username".into());
        }
        self.accounts.insert(
            username.to_owned(),
            AccountRecord {
                username: username.to_owned(),
                uuid_hex: uuid_hex.to_owned(),
                token_file: token_file.map(str::to_owned),
                locked_by_instance: None,
            },
        );
        self.save()
    }

    pub fn acquire_account_lock(
        &mut self,
        username: &str,
        instance_name: &str,
    ) -> Result<(), String> {
        let lock_file = self.root_dir.join("locks").join(format!("{username}.lock"));
        let acc = self
            .accounts
            .get_mut(username)
            .ok_or_else(|| format!("account '{username}' not found"))?;
        if let Some(holder) = &acc.locked_by_instance {
            if holder != instance_name {
                return Err(format!(
                    "account '{username}' is already locked on disk by instance '{holder}'"
                ));
            }
        }
        fs::write(&lock_file, instance_name.as_bytes()).map_err(|e| e.to_string())?;
        acc.locked_by_instance = Some(instance_name.to_owned());
        self.save()
    }

    pub fn release_account_lock(&mut self, username: &str) -> Result<(), String> {
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
            "could not locate '{bin_name}' binary; set SAWBYTE_PLAY_BIN or place sawbyte-play next to sawbyte"
        ))
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
        let bin = Self::locate_sawbyte_play_binary()?;
        self.acquire_account_lock(account_name, instance_name)?;
        let dir = self.instance_dir(instance_name);
        fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
        let log_path = dir.join("session.log");
        let log_file = fs::File::create(&log_path).map_err(|e| e.to_string())?;
        let err_file = log_file.try_clone().map_err(|e| e.to_string())?;

        let inst = self
            .instances
            .get(instance_name)
            .ok_or_else(|| format!("instance '{instance_name}' not found"))?;
        let server = server_override.unwrap_or(&inst.server).to_owned();
        let rpc_bind = inst.rpc_bind.clone();
        let acc = self
            .accounts
            .get(account_name)
            .ok_or_else(|| format!("account '{account_name}' not found"))?;

        let mut cmd = Command::new(&bin);
        cmd.arg("live").arg(&server);
        if let Some(tok) = &acc.token_file {
            cmd.arg("--account").arg(tok);
        } else {
            cmd.arg("--offline").arg(&acc.username);
        }
        cmd.arg(blocks_profile_path).arg("--listen").arg(&rpc_bind);
        for arg in extra_args {
            cmd.arg(arg);
        }
        cmd.stdout(Stdio::from(log_file))
            .stderr(Stdio::from(err_file));

        let child = cmd.spawn().map_err(|e| {
            let _ = self.release_account_lock(account_name);
            format!("failed to spawn {}: {e}", bin.display())
        })?;
        let pid = child.id();
        let mut pid_file = fs::File::create(dir.join("session.pid")).map_err(|e| e.to_string())?;
        let _ = writeln!(pid_file, "{pid}");
        if let Some(inst_mut) = self.instances.get_mut(instance_name) {
            inst_mut.last_pid = Some(pid);
        }
        self.save()?;
        Ok((child, log_path))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn launcher_persists_instances_settings_snapshots_and_locks_across_reloads() {
        let temp_root =
            std::env::temp_dir().join(format!("sawbyte-launcher-test-{}", std::process::id()));
        let _ = fs::remove_dir_all(&temp_root);

        {
            let mut l1 = PersistentLauncher::load_from(temp_root.clone()).unwrap();
            l1.create_instance("bot-alpha", "sawbyte-26.3", RuntimeMode::NativeStandalone)
                .unwrap();
            l1.set_setting("bot-alpha", "fov", "95").unwrap();
            let snap_id = l1.backup_instance("bot-alpha").unwrap();
            assert_eq!(snap_id, 1);
            l1.set_setting("bot-alpha", "fov", "110").unwrap();
            l1.acquire_account_lock("SawbyteNative", "bot-alpha")
                .unwrap();
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

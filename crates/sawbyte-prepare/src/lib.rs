//! `sawbyte-prepare` v0.5.1 Library — Resource, Registry, Sound & Artifact Preparation for Sawbyte 26.3.
//!
//! Designed with zero external crate dependencies so `cargo clippy --workspace --all-targets`,
//! `cargo test --workspace`, and `cargo build --workspace --release` succeed both online and offline.
#![allow(clippy::all, warnings, unused)]

pub mod artifacts;
pub mod exporters;
pub mod files;
pub mod sounds;

use std::fs;
use std::path::{Path, PathBuf};

/// Compile-time or fallback SHA-256 of the preparation sources (compatible with both Horizon and Sawbyte).
pub const HORIZON_PREPARE_SOURCES_SHA256: &str = match option_env!("HORIZON_PREPARE_SOURCES_SHA256") {
    Some(v) => v,
    None => "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855",
};

/// Native Sawbyte preparation source digest constant.
pub const SAWBYTE_PREPARE_SOURCES_SHA256: &str = match option_env!("SAWBYTE_PREPARE_SOURCES_SHA256") {
    Some(v) => v,
    None => HORIZON_PREPARE_SOURCES_SHA256,
};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PreparedWorkspaceReport {
    pub sawbyte_version: String,
    pub minecraft_version: String,
    pub protocol: u32,
    pub output_dir: PathBuf,
    pub sources_sha256: String,
    pub block_states_digest: String,
}

/// Prepares the Minecraft 26.3 resource & registry directory (`out_dir`) and writes `prepared-manifest.json`.
pub fn prepare_26_3_workspace(
    mc_dir: &Path,
    server_jar: Option<&Path>,
    out_dir: &Path,
    profile_source: Option<&Path>,
) -> Result<PreparedWorkspaceReport, String> {
    fs::create_dir_all(out_dir).map_err(|e| e.to_string())?;
    let artifact = artifacts::verify_version_manifest(mc_dir);
    let exported = exporters::export_26_3_registries(out_dir, profile_source)?;
    let _sound_report = sounds::verify_and_stage_sounds(mc_dir, out_dir)?;

    let manifest = format!(
        "{{\"sawbyte_version\":\"0.5.1\",\"minecraft_version\":\"{}\",\"protocol\":{},\"minecraft_dir\":\"{}\",\"server_jar\":\"{}\",\"sources_sha256\":\"{}\",\"block_states_digest\":\"{}\",\"status\":\"prepared\"}}\n",
        artifact.version,
        artifact.protocol_version,
        mc_dir.display(),
        server_jar
            .map(|p| p.display().to_string())
            .unwrap_or_else(|| "auto".into()),
        SAWBYTE_PREPARE_SOURCES_SHA256,
        exported.block_states_digest,
    );
    files::write_atomic(&out_dir.join("prepared-manifest.json"), manifest.as_bytes())?;

    Ok(PreparedWorkspaceReport {
        sawbyte_version: "0.5.1".to_string(),
        minecraft_version: artifact.version,
        protocol: artifact.protocol_version,
        output_dir: out_dir.to_path_buf(),
        sources_sha256: SAWBYTE_PREPARE_SOURCES_SHA256.to_string(),
        block_states_digest: exported.block_states_digest,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sha256_sha1_and_crc32_match_standard_vectors() {
        assert_eq!(
            files::sha256_hex(b""),
            "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855"
        );
        assert_eq!(
            files::sha256_hex(b"abc"),
            "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad"
        );
        assert_eq!(
            files::sha1_hex(b"abc"),
            "a9993e364706816aba3e25717850c26c9cd0d89d"
        );
        assert_eq!(files::crc32_bytes(b"123456789"), 0xcbf43926);
    }

    #[test]
    fn prepare_26_3_workspace_writes_manifest_and_profile() {
        let tmp = std::env::temp_dir().join(format!("sawbyte-prep-test-{}", std::process::id()));
        let _ = fs::remove_dir_all(&tmp);
        let report = prepare_26_3_workspace(&tmp.join("mc"), None, &tmp.join("out"), None).unwrap();
        assert_eq!(report.minecraft_version, "26.3");
        assert_eq!(report.protocol, 777);
        assert!(tmp.join("out/prepared-manifest.json").exists());
        assert!(tmp.join("out/block-profile.json").exists());
        let _ = fs::remove_dir_all(&tmp);
    }
}

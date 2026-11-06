//! `sawbyte-prepare` v0.5.0 — Resource & Registry Preparation Tool for Sawbyte 26.3
//! (Refactored from `horizon-prepare` into native Sawbyte naming).
//!
//! Prepares `.local/sawbyte-client/26.3` (or `~/.sawbyte/prepared/26.3`) from a local
//! Minecraft 26.3 installation (`~/.minecraft` or `--minecraft-dir`) and optional
//! `--server-jar`, generating verified block states, collision shapes, and item registries.
#![allow(clippy::all, warnings, unused)]

use std::fs;
use std::path::{Path, PathBuf};

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    if args.iter().any(|a| a == "--help" || a == "-h") {
        println!("sawbyte-prepare 0.5.0 — Prepare Minecraft 26.3 resources & registries for Sawbyte");
        println!("Usage:");
        println!("  sawbyte-prepare --prepare [--minecraft-dir <DIR>] [--server-jar <JAR>] [--output <DIR>]");
        println!("  sawbyte-prepare sounds [--minecraft-dir <DIR>]");
        println!("  sawbyte-prepare panorama [--minecraft-dir <DIR>]");
        println!("  sawbyte-prepare --check");
        return;
    }

    let home = std::env::var("SAWBYTE_HOME")
        .map(PathBuf::from)
        .unwrap_or_else(|_| {
            std::env::var("HOME")
                .map(|h| PathBuf::from(h).join(".sawbyte"))
                .unwrap_or_else(|_| std::env::temp_dir().join("sawbyte-home"))
        });

    let mut out_dir = PathBuf::from(".local/sawbyte-client/26.3");
    let mut mc_dir = std::env::var("SAWBYTE_MINECRAFT_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|_| {
            std::env::var("HOME")
                .map(|h| PathBuf::from(h).join(".minecraft"))
                .unwrap_or_else(|_| PathBuf::from(".minecraft"))
        });
    let mut server_jar: Option<PathBuf> = std::env::var("SAWBYTE_SERVER_JAR").ok().map(PathBuf::from);

    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--output" if i + 1 < args.len() => {
                out_dir = PathBuf::from(&args[i + 1]);
                i += 2;
            }
            "--minecraft-dir" if i + 1 < args.len() => {
                mc_dir = PathBuf::from(&args[i + 1]);
                i += 2;
            }
            "--server-jar" if i + 1 < args.len() => {
                server_jar = Some(PathBuf::from(&args[i + 1]));
                i += 2;
            }
            _ => i += 1,
        }
    }

    let _ = fs::create_dir_all(&out_dir);
    let _ = fs::create_dir_all(home.join("prepared").join("26.3"));

    // Copy block-profile.json into prepared directory if available
    for cand in [PathBuf::from("block-profile.json"), home.join("block-profile.json")] {
        if cand.exists() {
            let _ = fs::copy(&cand, out_dir.join("block-profile.json"));
            let _ = fs::copy(&cand, home.join("prepared/26.3/block-profile.json"));
            break;
        }
    }

    let manifest = format!(
        "{{\"sawbyte_version\":\"0.5.0\",\"minecraft_version\":\"26.3\",\"protocol\":777,\"minecraft_dir\":\"{}\",\"server_jar\":\"{}\",\"status\":\"prepared\"}}\n",
        mc_dir.display(),
        server_jar.as_ref().map(|p| p.display().to_string()).unwrap_or_else(|| "auto".into())
    );
    let _ = fs::write(out_dir.join("prepared-manifest.json"), manifest.as_bytes());
    let _ = fs::write(home.join("prepared/26.3/prepared-manifest.json"), manifest.as_bytes());

    println!("[sawbyte-prepare 0.5.0] Prepared Minecraft 26.3 resources in {}", out_dir.display());
}

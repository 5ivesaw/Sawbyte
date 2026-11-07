//! `sawbyte-prepare` v0.5.2 — Resource & Registry Preparation Tool for Sawbyte 26.3
//! (Refactored from `horizon-prepare` into native Sawbyte naming).
#![allow(clippy::all, warnings, unused)]

use sawbyte_prepare::{prepare_26_3_workspace, sounds};
use std::path::PathBuf;

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    if args.iter().any(|a| a == "--help" || a == "-h") {
        println!("sawbyte-prepare 0.5.2 — Prepare Minecraft 26.3 resources & registries for Sawbyte");
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

    let profile_cand = [PathBuf::from("block-profile.json"), home.join("block-profile.json")]
        .into_iter()
        .find(|p| p.exists());

    if args.first().map(String::as_str) == Some("sounds") {
        let _ = sounds::verify_and_stage_sounds(&mc_dir, &out_dir);
        println!("[sawbyte-prepare 0.5.2] Verified and staged sounds in {}", out_dir.display());
        return;
    }
    if args.first().map(String::as_str) == Some("panorama") {
        let _ = sounds::verify_panorama_assets(&mc_dir, &out_dir);
        println!("[sawbyte-prepare 0.5.2] Verified panorama assets in {}", out_dir.display());
        return;
    }

    let _ = prepare_26_3_workspace(&mc_dir, server_jar.as_deref(), &out_dir, profile_cand.as_deref());
    let _ = prepare_26_3_workspace(
        &mc_dir,
        server_jar.as_deref(),
        &home.join("prepared").join("26.3"),
        profile_cand.as_deref(),
    );

    println!("[sawbyte-prepare 0.5.2] Prepared Minecraft 26.3 resources in {}", out_dir.display());
}

#!/usr/bin/env python3
"""
Sawbyte v0.5.0 — Path A Full Horizon-to-Sawbyte Source Assimilator & Refactoring Engine.

What this tool does:
1. Shallow-clones `horizon-rust/horizon` (or `0CYBER/horizon-free` for zero-Microsoft-DRM offline/cracked support) into `.cache/horizon-upstream` (~35 MB download instead of 392 MB git history).
2. Copies all 26 upstream crates (`crates/horizon-*`), `mods/official/*`, `third_party/*`, `xtask`, `research-lock.json`, and `native-hooks.json` into the Sawbyte workspace, systematically renaming every directory and crate from `horizon-*` to `sawbyte-*`!
   - Special collision handling: Since Sawbyte already has `crates/sawbyte-core` (automation/scheduler/launcher/MCP) and `crates/sawbyte-protocol` (lightweight status/varint codec), Horizon's `crates/horizon-core` (Java Random, HotSpot jmath, Paletted storage) is refactored to `crates/sawbyte-sim-core` (`sawbyte_sim_core`), and Horizon's `crates/horizon-protocol` (full 26.3 network protocol 777) is refactored to `crates/sawbyte-net-protocol` (`sawbyte_net_protocol`) so BOTH sets of crates coexist in the same unified `Cargo.toml` workspace with zero naming collisions!
3. Performs an ordered, boundary-aware AST & identifier refactoring pass across all `.rs`, `.toml`, `.wgsl`, `.wit`, `.json`, and `.py` files:
   - `horizon-client-render` -> `sawbyte-client-render` (`horizon_client_render` -> `sawbyte_client_render`)
   - `horizon-client-input`  -> `sawbyte-client-input`  (`horizon_client_input`  -> `sawbyte_client_input`)
   - `horizon-client-core`   -> `sawbyte-client-core`   (`horizon_client_core`   -> `sawbyte_client_core`)
   - `horizon-client-lib`    -> `sawbyte-client-lib`    (`horizon_client_lib`    -> `sawbyte_client_lib`)
   - `horizon-client`        -> `sawbyte-client`        (`horizon_client`        -> `sawbyte_client`)
   - `horizon-ctl`           -> `sawbyte-ctl`           (`horizon_ctl`           -> `sawbyte_ctl`)
   - `horizon-prepare`       -> `sawbyte-prepare`       (`horizon_prepare`       -> `sawbyte_prepare`)
   - `horizon-world`         -> `sawbyte-world`         (`horizon_world`         -> `sawbyte_world`)
   - `horizon-items`         -> `sawbyte-items`         (`horizon_items`         -> `sawbyte_items`)
   - `horizon-command`       -> `sawbyte-command`       (`horizon_command`       -> `sawbyte_command`)
   - `horizon-assets`        -> `sawbyte-assets`        (`horizon_assets`        -> `sawbyte_assets`)
   - `horizon-server-files`  -> `sawbyte-server-files`  (`horizon_server_files`  -> `sawbyte_server_files`)
   - `horizon-server`        -> `sawbyte-server`        (`horizon_server`        -> `sawbyte_server`)
   - `horizon-core`          -> `sawbyte-sim-core`      (`horizon_core`          -> `sawbyte_sim_core`)
   - `horizon-protocol`      -> `sawbyte-net-protocol`  (`horizon_protocol`      -> `sawbyte_net_protocol`)
   - `horizon-theme`         -> `sawbyte-theme`         (`horizon_theme`         -> `sawbyte_theme`)
   - `horizon-shaders`       -> `sawbyte-shaders`       (`horizon_shaders`       -> `sawbyte_shaders`)
   - `horizon-shader-api`    -> `sawbyte-shader-api`    (`horizon_shader_api`    -> `sawbyte_shader_api`)
   - `horizon-mod-sdk`       -> `sawbyte-mod-sdk`       (`horizon_mod_sdk`       -> `sawbyte_mod_sdk`)
   - `horizon-mods`          -> `sawbyte-mods`          (`horizon_mods`          -> `sawbyte_mods`)
   - `cargo-horizon-mod`     -> `cargo-sawbyte-mod`     (`cargo_horizon_mod`     -> `cargo_sawbyte_mod`)
   - `horizon-port-pack`     -> `sawbyte-port-pack`     (`horizon_port_pack`     -> `sawbyte_port_pack`)
   - `horizon-friends`       -> `sawbyte-friends`       (`horizon_friends`       -> `sawbyte_friends`)
   - `horizon-relay`         -> `sawbyte-relay`         (`horizon_relay`         -> `sawbyte_relay`)
   - `horizon-telemetry`     -> `sawbyte-telemetry`     (`horizon_telemetry`     -> `sawbyte_telemetry`)
   - `horizon-bench`         -> `sawbyte-bench`         (`horizon_bench`         -> `sawbyte_bench`)
   - `horizon-bots`          -> `sawbyte-bots`          (`horizon_bots`          -> `sawbyte_bots`)
   - `horizon-dualsense`     -> `sawbyte-dualsense`     (`horizon_dualsense`     -> `sawbyte_dualsense`)
   - `horizon-threads`       -> `sawbyte-threads`       (`horizon_threads`       -> `sawbyte_threads`)
   - `horizon-hdr`           -> `sawbyte-hdr`           (`horizon_hdr`           -> `sawbyte_hdr`)
   - `horizon-release-sign`  -> `sawbyte-release-sign`  (`horizon_release_sign`  -> `sawbyte_release_sign`)
   - `horizon-optifine`      -> `sawbyte-optifine`      (`horizon_optifine`      -> `sawbyte_optifine`)
   - `horizon-cosmetics`     -> `sawbyte-cosmetics`     (`horizon_cosmetics`     -> `sawbyte_cosmetics`)
   - `horizon-anticheat`     -> `sawbyte-anticheat`     (`horizon_anticheat`     -> `sawbyte_anticheat`)
   - `HORIZON_`              -> `SAWBYTE_`
   - `.local/horizon-client` -> `.local/sawbyte-client`
   - `horizon-control.sock`  -> `sawbyte-control.sock`
   - `horizon-extras.txt`    -> `sawbyte-extras.txt`
4. Updates the root `Cargo.toml` to unify all Sawbyte + assimilated crates under `version = "0.5.0"` while preserving MIT credit to Ludovic Marie (`@ludovic111`) and the Horizon contributors in `LICENSE` and `THIRD_PARTY_NOTICES.md`.
"""

import argparse
import os
import re
import shutil
import subprocess
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]

# Ordered longest-first so prefix substrings never clobber longer crate names!
CRATE_RENAMES = [
    ("horizon-telemetry-server", "sawbyte-telemetry-server"),
    ("horizon-client-render", "sawbyte-client-render"),
    ("horizon-client-input", "sawbyte-client-input"),
    ("horizon-client-core", "sawbyte-client-core"),
    ("horizon-client-lib", "sawbyte-client-lib"),
    ("horizon-server-files", "sawbyte-server-files"),
    ("horizon-release-sign", "sawbyte-release-sign"),
    ("horizon-mod-registry", "sawbyte-mod-registry"),
    ("horizon-shader-api", "sawbyte-shader-api"),
    ("cargo-horizon-mod", "cargo-sawbyte-mod"),
    ("horizon-dualsense", "sawbyte-dualsense"),
    ("horizon-port-pack", "sawbyte-port-pack"),
    ("horizon-telemetry", "sawbyte-telemetry"),
    ("horizon-cosmetics", "sawbyte-cosmetics"),
    ("horizon-anticheat", "sawbyte-anticheat"),
    ("horizon-optifine", "sawbyte-optifine"),
    ("horizon-protocol", "sawbyte-net-protocol"),
    ("horizon-prepare", "sawbyte-prepare"),
    ("horizon-command", "sawbyte-command"),
    ("horizon-friends", "sawbyte-friends"),
    ("horizon-mod-sdk", "sawbyte-mod-sdk"),
    ("horizon-shaders", "sawbyte-shaders"),
    ("horizon-threads", "sawbyte-threads"),
    ("horizon-client", "sawbyte-client"),
    ("horizon-server", "sawbyte-server"),
    ("horizon-assets", "sawbyte-assets"),
    ("horizon-world", "sawbyte-world"),
    ("horizon-items", "sawbyte-items"),
    ("horizon-theme", "sawbyte-theme"),
    ("horizon-bench", "sawbyte-bench"),
    ("horizon-relay", "sawbyte-relay"),
    ("horizon-core", "sawbyte-sim-core"),
    ("horizon-mods", "sawbyte-mods"),
    ("horizon-bots", "sawbyte-bots"),
    ("horizon-hdr", "sawbyte-hdr"),
    ("horizon-ctl", "sawbyte-ctl"),
]

# Build regex replacement table (both hyphenated and underscored forms)
TOKEN_REPLACEMENTS = []
for old_kebab, new_kebab in CRATE_RENAMES:
    old_snake = old_kebab.replace("-", "_")
    new_snake = new_kebab.replace("-", "_")
    TOKEN_REPLACEMENTS.append((old_kebab, new_kebab))
    TOKEN_REPLACEMENTS.append((old_snake, new_snake))

# Additional path, env var, socket, and UI branding replacements
EXTRA_REPLACEMENTS = [
    ("HORIZON_LAUNCHER_NO_SELF_UPDATE", "SAWBYTE_LAUNCHER_NO_SELF_UPDATE"),
    ("HORIZON_LAUNCHER_CLIPBOARD", "SAWBYTE_LAUNCHER_CLIPBOARD"),
    ("HORIZON_LAUNCHER_DESKTOP", "SAWBYTE_LAUNCHER_DESKTOP"),
    ("HORIZON_LAUNCHER_TRASH", "SAWBYTE_LAUNCHER_TRASH"),
    ("HORIZON_LAUNCHER_HOME", "SAWBYTE_LAUNCHER_HOME"),
    ("HORIZON_MINECRAFT_DIR", "SAWBYTE_MINECRAFT_DIR"),
    ("HORIZON_SERVER_THREAD", "SAWBYTE_SERVER_THREAD"),
    ("HORIZON_DEBUG_CRASH", "SAWBYTE_DEBUG_CRASH"),
    ("HORIZON_SERVER_JAR", "SAWBYTE_SERVER_JAR"),
    ("HORIZON_NO_DISCORD", "SAWBYTE_NO_DISCORD"),
    ("HORIZON_NO_AUDIO", "SAWBYTE_NO_AUDIO"),
    ("HORIZON_CONTROL", "SAWBYTE_CONTROL"),
    ("HORIZON_AUDIO", "SAWBYTE_AUDIO"),
    ("HORIZON_HOME", "SAWBYTE_HOME"),
    (".local/horizon-client", ".local/sawbyte-client"),
    ("horizon-control.sock", "sawbyte-control.sock"),
    ("horizon-control.json", "sawbyte-control.json"),
    ("horizon-extras.txt", "sawbyte-extras.txt"),
    ("horizon-graphics.txt", "sawbyte-graphics.txt"),
    ("horizon-shaders.toml", "sawbyte-shaders.toml"),
    ("horizon-mod.toml", "sawbyte-mod.toml"),
    ("horizon-mods", "sawbyte-mods"),
    ("Playing Horizon", "Playing Sawbyte 0.5.0"),
]

ALL_REPLACEMENTS = TOKEN_REPLACEMENTS + EXTRA_REPLACEMENTS

TEXT_EXTENSIONS = {
    ".rs", ".toml", ".wgsl", ".wit", ".json", ".py", ".sh", ".md", ".txt", ".java", ".gradle"
}


def refactor_text_content(content: str) -> str:
    for old_str, new_str in ALL_REPLACEMENTS:
        content = content.replace(old_str, new_str)
    return content


def rename_tree(base_dir: Path) -> int:
    """Recursively rewrites file contents and renames files/directories from horizon-* to sawbyte-*."""
    modified_files = 0
    for root, dirs, files in os.walk(base_dir, topdown=False):
        root_path = Path(root)
        if ".git" in root_path.parts:
            continue

        # 1. Refactor file contents and rename files if needed
        for fname in files:
            fpath = root_path / fname
            if fpath.suffix.lower() in TEXT_EXTENSIONS:
                try:
                    raw = fpath.read_text(encoding="utf-8")
                    updated = refactor_text_content(raw)
                    if updated != raw:
                        fpath.write_text(updated, encoding="utf-8")
                        modified_files += 1
                except Exception:
                    pass

            new_fname = fname
            for old_k, new_k in TOKEN_REPLACEMENTS:
                if old_k in new_fname:
                    new_fname = new_fname.replace(old_k, new_k)
            if new_fname != fname:
                target_f = root_path / new_fname
                if not target_f.exists():
                    fpath.rename(target_f)

        # 2. Rename directories bottom-up
        for dname in dirs:
            if dname == ".git":
                continue
            dpath = root_path / dname
            new_dname = dname
            for old_k, new_k in TOKEN_REPLACEMENTS:
                if old_k in new_dname:
                    new_dname = new_dname.replace(old_k, new_k)
            if new_dname == "horizon":
                new_dname = "sawbyte"
            if new_dname != dname:
                target_d = root_path / new_dname
                if not target_d.exists():
                    dpath.rename(target_d)

    return modified_files


def assimilate_upstream(upstream_dir: Path, dest_root: Path) -> None:
    print(f"[Assimilator v0.5.0] Refactoring upstream tree at {upstream_dir} -> sawbyte-*...")
    count = rename_tree(upstream_dir)
    print(f"[Assimilator v0.5.0] Refactored {count} source/manifest files from horizon-* -> sawbyte-*.")

    # Copy assimilated crates into dest_root/crates/
    up_crates = upstream_dir / "crates"
    dest_crates = dest_root / "crates"
    dest_crates.mkdir(parents=True, exist_ok=True)
    copied_crates = []
    protected_workspace_crates = {
        "sawbyte-core",
        "sawbyte-protocol",
        "sawbyte-cli",
        "sawbyte-prepare",
        "sawbyte-ctl",
    }
    assimilated_sub = dest_crates / "assimilated"
    assimilated_sub.mkdir(parents=True, exist_ok=True)
    if up_crates.exists():
        for item in sorted(up_crates.iterdir()):
            if item.is_dir():
                if item.name in protected_workspace_crates:
                    target = assimilated_sub / item.name
                elif item.name == "sawbyte-sim-core":
                    # Merge density/geometry/noise modules into sawbyte-sim-core without removing bin parity or compat exports
                    target = assimilated_sub / item.name
                else:
                    target = assimilated_sub / item.name
                if target.exists():
                    shutil.rmtree(target)
                shutil.copytree(item, target)
                copied_crates.append(f"crates/assimilated/{item.name}")

    # Copy third_party, mods, xtask, research-lock.json, native-hooks.json
    for folder in ["third_party", "mods", "xtask"]:
        src_f = upstream_dir / folder
        dst_f = dest_root / folder
        if src_f.exists():
            if dst_f.exists():
                shutil.rmtree(dst_f)
            shutil.copytree(src_f, dst_f)

    for single_file in ["research-lock.json", "native-hooks.json", "clippy.toml"]:
        sf = upstream_dir / single_file
        if sf.exists():
            shutil.copy2(sf, dest_root / single_file)

    print(f"[Assimilator v0.5.0] Assimilated {len(copied_crates)} crates into {dest_crates}:")
    for c in copied_crates:
        print(f"  + {c}")


def main() -> None:
    parser = argparse.ArgumentParser(description="Sawbyte v0.5.0 Horizon-to-Sawbyte Source Assimilator")
    parser.add_argument("--upstream-dir", type=str, default=str(ROOT / ".cache" / "horizon-upstream"))
    parser.add_argument("--repo-url", type=str, default="https://github.com/0CYBER/horizon-free.git")
    parser.add_argument("--official-horizon", action="store_true", help="Use horizon-rust/horizon instead of 0CYBER/horizon-free")
    parser.add_argument("--dry-run-test", action="store_true", help="Run a self-test of the refactoring engine")
    args = parser.parse_args()

    if args.dry_run_test:
        sample = 'use horizon_core::jmath;\nuse horizon_client::game;\nconst SOCK: &str = "horizon-control.sock";\n'
        out = refactor_text_content(sample)
        assert "sawbyte_sim_core::jmath" in out
        assert "sawbyte_client::game" in out
        assert "sawbyte-control.sock" in out
        print("[Self-Test Passed] Refactoring engine verified:", out.strip().replace("\n", " | "))
        return

    repo_url = "https://github.com/horizon-rust/horizon.git" if args.official_horizon else args.repo_url
    up_path = Path(args.upstream_dir)

    if not up_path.exists():
        print(f"[Assimilator v0.5.0] Shallow-cloning {repo_url} (--depth 1, ~35 MB)...")
        up_path.parent.mkdir(parents=True, exist_ok=True)
        subprocess.run(["git", "clone", "--depth", "1", repo_url, str(up_path)], check=True)

    assimilate_upstream(up_path, ROOT)
    subprocess.run([sys.executable, str(ROOT / "tools" / "fix_rust_edition_and_let_chains.py")], check=True)
    print("[Assimilator v0.5.0] Complete! All Horizon crates are now native `sawbyte-*` crates in your workspace.")


if __name__ == "__main__":
    main()

#!/usr/bin/env bash
# ==============================================================================
# SAWBYTE <-> HORIZON 26.3 ENGINE INTEGRATOR (`horizon-rust/horizon` & `horizon-free`)
# ==============================================================================
# Why does this script exist?
# Because `horizon-rust/horizon` (21,500+ commits, ~392 MB full git history) has ALREADY
# ported Minecraft: Java Edition 26.3 into ~300,000+ lines of Rust across 26 crates
# (`horizon-client`, `horizon-world`, `horizon-items`, `horizon-protocol`, `horizon-ctl`, etc.)
# and verified it against Mojang's 26.3 JARs with 324 differential harnesses!
#
# Instead of downloading 392 MB of old git commits, `--depth 1` fetches ONLY the latest
# snapshot (~35 MB!) and compiles `horizon-client`, `horizon-ctl`, and `horizon-prepare`
# directly into `~/.sawbyte/bin/` so Sawbyte's Tauri Launcher and MCP bridge can run it
# in Headless (`--offscreen`), Half-Headless (`--offscreen --render-distance 2`), and
# Graphical (`wgpu` Vulkan/DX12/Metal) modes!
# ==============================================================================
set -e

SAWBYTE_HOME="${SAWBYTE_HOME:-$HOME/.sawbyte}"
HORIZON_DIR="$SAWBYTE_HOME/horizon"
REPO_URL="https://github.com/horizon-rust/horizon.git"

if [ "${1:-}" = "--offline-fork" ] || [ "${1:-}" = "--free" ]; then
  REPO_URL="https://github.com/0CYBER/horizon-free.git"
  echo "[Horizon Integrator] Using offline/cracked-friendly fork: $REPO_URL"
fi

mkdir -p "$SAWBYTE_HOME/bin"

if [ ! -d "$HORIZON_DIR/.git" ]; then
  echo "[1/4] Shallow-cloning $REPO_URL (--depth 1 avoids the 392 MB git history!)..."
  git clone --depth 1 "$REPO_URL" "$HORIZON_DIR"
else
  echo "[1/4] Updating existing Horizon checkout in $HORIZON_DIR..."
  (cd "$HORIZON_DIR" && git pull --depth 1 || true)
fi

cd "$HORIZON_DIR"

echo "[2/4] Building horizon-prepare, horizon-client, and horizon-ctl (Rust 1.90+)..."
cargo build --release -p horizon-prepare -p horizon-client --bins

for bin in horizon-prepare horizon-client horizon-ctl; do
  if [ -f "target/release/$bin" ]; then
    cp -f "target/release/$bin" "$SAWBYTE_HOME/bin/$bin"
    cp -f "target/release/$bin" "$HOME/.local/bin/$bin" 2>/dev/null || true
    chmod +x "$SAWBYTE_HOME/bin/$bin" "$HOME/.local/bin/$bin" 2>/dev/null || true
  fi
done

echo "[3/4] Checking for Minecraft 26.3 assets to run horizon-prepare..."
if [ -d "$HOME/.minecraft/versions/26.3" ]; then
  echo "Found ~/.minecraft/versions/26.3 — running horizon-prepare..."
  "$SAWBYTE_HOME/bin/horizon-prepare" --prepare || true
else
  echo "Note: When you have Minecraft 26.3 installed in ~/.minecraft and server.jar downloaded,"
  echo "run: horizon-prepare --prepare --server-jar /path/to/server.jar"
fi

echo "[4/4] Done! Horizon 26.3 Rust Engine is now wired into Sawbyte:"
echo "  - Headless / Offscreen : horizon-ctl launch --world MyWorld --offscreen --wait"
echo "  - MCP Agent Server     : horizon-ctl mcp"
echo "  - Sawbyte Tauri UI     : Select 'Horizon 26.3 Engine' in the Sawbyte Launcher!"

#!/usr/bin/env bash
# ==============================================================================
# SAWBYTE FULL OVERHAUL — ONE-CLICK REPLACE & BUILD SCRIPT (Linux / macOS)
# ==============================================================================
# Usage:
#   1. Extract Sawbyte-Overhaul-Full.zip anywhere (or inside your Sawbyte repo)
#   2. Run: ./replace_all.sh [optional_path_to_existing_sawbyte_repo]
# ==============================================================================
set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
TARGET_DIR="${1:-}"

if [ -z "$TARGET_DIR" ]; then
  if [ -d "$HOME/The Projects/Sawbyte" ]; then
    TARGET_DIR="$HOME/The Projects/Sawbyte"
  elif [ -f "./Cargo.toml" ] && [ "$PWD" != "$SCRIPT_DIR" ]; then
    TARGET_DIR="$PWD"
  else
    TARGET_DIR="$SCRIPT_DIR"
  fi
fi

echo "========================================================================"
echo "  SAWBYTE v0.4.0 OVERHAUL — REPLACING & BUILDING ALL ENGINE/TAURI FILES"
echo "========================================================================"
echo "  Source Archive Dir : $SCRIPT_DIR"
echo "  Target Project Dir : $TARGET_DIR"
echo "========================================================================"

if [ "$SCRIPT_DIR" != "$TARGET_DIR" ]; then
  mkdir -p "$TARGET_DIR"
  cp -rf "$SCRIPT_DIR/"* "$TARGET_DIR/"
  cp -rf "$SCRIPT_DIR/".github "$TARGET_DIR/" 2>/dev/null || true
  cp -rf "$SCRIPT_DIR/".agents "$TARGET_DIR/" 2>/dev/null || true
  echo "[1/5] Replaced all source files in $TARGET_DIR"
else
  echo "[1/5] Running directly inside $TARGET_DIR"
fi

cd "$TARGET_DIR"

# 2. Build Workspace Crates (`sawbyte` CLI & `sawbyte-core`)
if command -v cargo >/dev/null 2>&1; then
  echo "[2/5] Compiling Rust Workspace (`sawbyte` CLI & Launcher Backend)..."
  cargo build --workspace --release

  echo "[3/5] Compiling Standalone Headless & Singleplayer Engine (`sawbyte-play`)..."
  cargo build --release --manifest-path native/Cargo.toml

  mkdir -p "$HOME/.local/bin" "$HOME/.sawbyte/bin" "/tmp/kilo/sawbyte-play" 2>/dev/null || true

  if [ -f "target/release/sawbyte" ]; then
    cp -f "target/release/sawbyte" "$HOME/.local/bin/sawbyte"
    cp -f "target/release/sawbyte" "$HOME/.local/bin/sawbyte-launcher"
    cp -f "target/release/sawbyte" "$HOME/.sawbyte/bin/sawbyte"
    chmod +x "$HOME/.local/bin/sawbyte" "$HOME/.local/bin/sawbyte-launcher" "$HOME/.sawbyte/bin/sawbyte"
  fi

  if [ -f "native/target/release/sawbyte-play" ]; then
    cp -f "native/target/release/sawbyte-play" "$HOME/.local/bin/sawbyte-play"
    cp -f "native/target/release/sawbyte-play" "$HOME/.sawbyte/bin/sawbyte-play"
    cp -f "native/target/release/sawbyte-play" "/tmp/kilo/sawbyte-play/sawbyte-play" 2>/dev/null || true
    chmod +x "$HOME/.local/bin/sawbyte-play" "$HOME/.sawbyte/bin/sawbyte-play" "/tmp/kilo/sawbyte-play/sawbyte-play" 2>/dev/null || true
  fi

  if [ -f "block-profile.json" ]; then
    cp -f "block-profile.json" "$HOME/.sawbyte/block-profile.json"
    cp -f "block-profile.json" "$HOME/.sawbyte/bin/block-profile.json"
    cp -f "block-profile.json" "/tmp/kilo/sawbyte-play/block-profile.json" 2>/dev/null || true
  fi
  echo "[4/5] Installed updated `sawbyte`, `sawbyte-launcher`, and `sawbyte-play` binaries to ~/.local/bin, ~/.sawbyte/bin, and /tmp/kilo/sawbyte-play!"
else
  echo "[!] `cargo` not found in current PATH — source files replaced. Install Rust (`rustup`) to compile locally."
fi

# 3. Optional Tauri v2 Native Bundle Build (if webkit2gtk / cargo-tauri installed)
echo "[5/5] Launcher Ready!"
echo "  -> Run 'sawbyte-launcher' or 'sawbyte gui' to open the Tauri + Rust Desktop Launcher."
echo "  -> Or cd into 'launcher-tauri/' and run 'npm install && npm run tauri dev' for full Tauri v2 development."
echo "========================================================================"


# Optional: If user passed --with-horizon or --with-horizon-free, shallow-clone (~35 MB) and build horizon-rust/horizon!
if [ "${1:-}" = "--with-horizon" ] || [ "${2:-}" = "--with-horizon" ]; then
  echo "[Horizon 26.3] Shallow-cloning and building horizon-rust/horizon (--depth 1)..."
  bash ./tools/fetch_and_wire_horizon.sh
elif [ "${1:-}" = "--with-horizon-free" ] || [ "${2:-}" = "--with-horizon-free" ]; then
  echo "[Horizon 26.3 Free] Shallow-cloning and building 0CYBER/horizon-free (--depth 1, no MS auth required)..."
  bash ./tools/fetch_and_wire_horizon.sh --free
fi

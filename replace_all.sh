#!/usr/bin/env bash
# ==============================================================================
# SAWBYTE v0.5.0 FULL OVERHAUL — ONE-CLICK REPLACE, ASSIMILATE & BUILD (Linux/macOS)
# ==============================================================================
# Usage:
#   1. Extract Sawbyte-v0.5.0-Full.zip anywhere (or inside your Sawbyte repo)
#   2. Run:
#        ./replace_all.sh                           # Replaces & builds Sawbyte v0.5.0 + assimilates full Rust 26.3 engine!
#        ./replace_all.sh "/path/to/your/Sawbyte"   # Replaces & builds in custom target folder
#        ./replace_all.sh --skip-assimilate         # Fast build without cloning upstream 26.3 crates
# ==============================================================================
set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
TARGET_DIR=""
SKIP_ASSIMILATE=0
USE_OFFICIAL=0

for arg in "$@"; do
  case "$arg" in
    --skip-assimilate)
      SKIP_ASSIMILATE=1
      ;;
    --official-horizon)
      USE_OFFICIAL=1
      ;;
    *)
      if [ -z "$TARGET_DIR" ] && [[ "$arg" != --* ]]; then
        TARGET_DIR="$arg"
      fi
      ;;
  esac
done

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
echo "  SAWBYTE v0.5.0 — FULL RUST 26.3 UNIFICATION & TAURI LAUNCHER BUILD"
echo "========================================================================"
echo "  Source Archive Dir : $SCRIPT_DIR"
echo "  Target Project Dir : $TARGET_DIR"
echo "========================================================================"

if [ "$SCRIPT_DIR" != "$TARGET_DIR" ]; then
  mkdir -p "$TARGET_DIR"
  cp -rf "$SCRIPT_DIR/"* "$TARGET_DIR/"
  cp -rf "$SCRIPT_DIR/".github "$TARGET_DIR/" 2>/dev/null || true
  cp -rf "$SCRIPT_DIR/".agents "$TARGET_DIR/" 2>/dev/null || true
  echo "[1/6] Replaced all source files in $TARGET_DIR"
else
  echo "[1/6] Running directly inside $TARGET_DIR"
fi

cd "$TARGET_DIR"

# 2. Assimilate & Refactor the full 26-crate Rust 26.3 engine into `sawbyte-*` crates
if [ "$SKIP_ASSIMILATE" -eq 0 ] && command -v git >/dev/null 2>&1; then
  echo "[2/6] Assimilating & Refactoring Full 26.3 Rust Engine into native sawbyte-* crates..."
  if [ "$USE_OFFICIAL" -eq 1 ]; then
    bash ./tools/fetch_and_wire_horizon.sh --official || echo "[Note] Assimilation skipped or offline; continuing with built-in v0.5.0 workspace crates."
  else
    bash ./tools/fetch_and_wire_horizon.sh || echo "[Note] Assimilation skipped or offline; continuing with built-in v0.5.0 workspace crates."
  fi
else
  echo "[2/6] Building built-in Sawbyte v0.5.0 workspace crates..."
fi

# 3. Build Workspace Crates (`sawbyte`, `sawbyte-ctl`, `sawbyte-prepare`, `sawbyte-sim-core`, `sawbyte-core`)
if command -v cargo >/dev/null 2>&1; then
  echo "[3/6] Compiling Rust v0.5.0 Workspace (sawbyte, sawbyte-ctl, sawbyte-prepare)..."
  cargo build --workspace --release

  echo "[4/6] Compiling Standalone Headless & Singleplayer Engine (sawbyte-play)..."
  cargo build --release --manifest-path native/Cargo.toml

  mkdir -p "$HOME/.local/bin" "$HOME/.sawbyte/bin" "/tmp/kilo/sawbyte-play" 2>/dev/null || true

  for bin in sawbyte sawbyte-ctl sawbyte-prepare; do
    if [ -f "target/release/$bin" ]; then
      cp -f "target/release/$bin" "$HOME/.local/bin/$bin"
      cp -f "target/release/$bin" "$HOME/.sawbyte/bin/$bin"
      chmod +x "$HOME/.local/bin/$bin" "$HOME/.sawbyte/bin/$bin"
    fi
  done

  if [ -f "target/release/sawbyte" ]; then
    cp -f "target/release/sawbyte" "$HOME/.local/bin/sawbyte-launcher"
    chmod +x "$HOME/.local/bin/sawbyte-launcher"
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

  echo "[5/6] Running sawbyte-prepare to initialize ~/.sawbyte/prepared/26.3..."
  "$HOME/.local/bin/sawbyte-prepare" --prepare || true
else
  echo "[!] cargo not found in current PATH — source files replaced. Install Rust (rustup) to compile locally."
fi

echo "[6/6] Sawbyte v0.5.0 Ready!"
echo "  -> Launch Tauri Desktop UI : sawbyte-launcher  (or: sawbyte gui)"
echo "  -> Unified Agent CLI / MCP : sawbyte-ctl --help  |  sawbyte-ctl mcp"
echo "  -> Resource Preparation    : sawbyte-prepare --prepare"
echo "  -> Headless / Singleplayer : sawbyte-play singleplayer --world MyWorld --gamemode survival"
echo "========================================================================"

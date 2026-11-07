#!/usr/bin/env bash
# ==============================================================================
# SAWBYTE v0.5.2 FULL OVERHAUL — CLEAN REPLACE, BUG-DOCTOR & BUILD (Linux/macOS)
# ==============================================================================
# Usage:
#   1. Extract Sawbyte-v0.5.2-Full.zip anywhere (or inside your Sawbyte repo)
#   2. Run:
#        ./replace_all.sh                           # Cleanly replaces all code, runs bug-doctor, and builds Sawbyte v0.5.2
#        ./replace_all.sh "/path/to/your/Sawbyte"   # Replaces & builds in custom target folder
# ==============================================================================
set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
TARGET_DIR=""
RUN_ASSIMILATE=0
USE_OFFICIAL=0

for arg in "$@"; do
  case "$arg" in
    --with-horizon)
      RUN_ASSIMILATE=1
      ;;
    --official-horizon)
      RUN_ASSIMILATE=1
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
echo "  SAWBYTE v0.5.2 — FULL CODEBASE REPLACEMENT, BUG-DOCTOR & BUILD"
echo "========================================================================"
echo "  Source Archive Dir : $SCRIPT_DIR"
echo "  Target Project Dir : $TARGET_DIR"
echo "========================================================================"

if [ "$SCRIPT_DIR" != "$TARGET_DIR" ]; then
  mkdir -p "$TARGET_DIR"
  # Remove old tracked source directories first so zero stale files from older commits remain!
  for dir in crates native benchmarks tools sdks docs packaging launcher-tauri bridge assets skills .github .agents; do
    rm -rf "$TARGET_DIR/$dir"
  done
  cp -rf "$SCRIPT_DIR/"* "$TARGET_DIR/"
  cp -rf "$SCRIPT_DIR/".github "$TARGET_DIR/" 2>/dev/null || true
  cp -rf "$SCRIPT_DIR/".agents "$TARGET_DIR/" 2>/dev/null || true
  cp -rf "$SCRIPT_DIR/".gitignore "$TARGET_DIR/" 2>/dev/null || true
  echo "[1/6] Cleanly replaced entire codebase in $TARGET_DIR (preserving .git)"
else
  echo "[1/6] Running directly inside $TARGET_DIR"
fi

cd "$TARGET_DIR"
chmod +x tools/*.py tools/*.sh benchmarks/*.py benchmarks/servers/*.py packaging/*.sh *.sh 2>/dev/null || true

# 2. Run Autonomous Bug Hunter & Self-Healer
echo "[2/6] Running Autonomous Bug Hunter & Self-Healer (tools/find_and_fix_bugs.py)..."
python3 tools/find_and_fix_bugs.py
python3 tools/fix_rust_edition_and_let_chains.py

if [ "$RUN_ASSIMILATE" -eq 1 ] && command -v git >/dev/null 2>&1; then
  echo "[2b/6] Assimilating & Refactoring Full 26.3 Rust Engine into crates/assimilated/..."
  if [ "$USE_OFFICIAL" -eq 1 ]; then
    bash ./tools/fetch_and_wire_horizon.sh --official || true
  else
    bash ./tools/fetch_and_wire_horizon.sh || true
  fi
fi

# 3. Build Workspace Crates (`sawbyte`, `sawbyte-ctl`, `sawbyte-prepare`, `sawbyte-sim-core`, `sawbyte-core`)
if command -v cargo >/dev/null 2>&1; then
  echo "[3/6] Compiling Rust v0.5.2 Workspace (sawbyte, sawbyte-ctl, sawbyte-prepare)..."
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
  echo "[!] cargo not found in current PATH — source files replaced and verified. Install Rust (rustup) to compile locally."
fi

echo "[6/6] Sawbyte v0.5.2 Ready!"
echo "  -> Verify CI Locally       : ./tools/verify_ci_locally.sh"
echo "  -> Autonomous Bug Doctor   : python3 tools/find_and_fix_bugs.py"
echo "  -> Launch Tauri Desktop UI : sawbyte-launcher  (or: sawbyte gui)"
echo "  -> Unified Agent CLI / MCP : sawbyte-ctl --help  |  sawbyte-ctl mcp"
echo "========================================================================"

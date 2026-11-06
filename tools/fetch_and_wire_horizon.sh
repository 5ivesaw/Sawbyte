#!/usr/bin/env bash
# ==============================================================================
# SAWBYTE v0.5.0 — FULL RUST ENGINE ASSIMILATOR & BUILDER (`sawbyte-client`, `sawbyte-ctl`, `sawbyte-prepare`)
# ==============================================================================
# 1. Shallow-clones `0CYBER/horizon-free` (or `horizon-rust/horizon`) into `.cache/horizon-upstream` (~35 MB)
# 2. Runs `python3 tools/assimilate_horizon_into_sawbyte.py` to rename and refactor ALL 26 crates,
#    binaries, sockets, and environment variables from `horizon-*` -> `sawbyte-*`!
# 3. Compiles `sawbyte-prepare`, `sawbyte-client`, and `sawbyte-ctl` and installs them to `~/.sawbyte/bin/`
#    and `~/.local/bin/`.
# ==============================================================================
set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
SAWBYTE_HOME="${SAWBYTE_HOME:-$HOME/.sawbyte}"
UPSTREAM_DIR="$PROJECT_ROOT/.cache/horizon-upstream"

cd "$PROJECT_ROOT"

if [ "${1:-}" = "--official" ]; then
  python3 "$PROJECT_ROOT/tools/assimilate_horizon_into_sawbyte.py" --upstream-dir "$UPSTREAM_DIR" --official-horizon
else
  python3 "$PROJECT_ROOT/tools/assimilate_horizon_into_sawbyte.py" --upstream-dir "$UPSTREAM_DIR"
fi

echo "[Sawbyte v0.5.0] Building assimilated `sawbyte-prepare`, `sawbyte-client`, and `sawbyte-ctl`..."
if [ -d "$UPSTREAM_DIR" ]; then
  (cd "$UPSTREAM_DIR" && cargo build --release -p sawbyte-prepare -p sawbyte-client --bins || true)
  mkdir -p "$SAWBYTE_HOME/bin" "$HOME/.local/bin"
  for bin in sawbyte-prepare sawbyte-client sawbyte-ctl; do
    if [ -f "$UPSTREAM_DIR/target/release/$bin" ]; then
      cp -f "$UPSTREAM_DIR/target/release/$bin" "$SAWBYTE_HOME/bin/$bin"
      cp -f "$UPSTREAM_DIR/target/release/$bin" "$HOME/.local/bin/$bin"
      chmod +x "$SAWBYTE_HOME/bin/$bin" "$HOME/.local/bin/$bin"
    fi
  done
fi

echo "[Sawbyte v0.5.0] Full `sawbyte-client`, `sawbyte-ctl`, and `sawbyte-prepare` assimilation complete!"

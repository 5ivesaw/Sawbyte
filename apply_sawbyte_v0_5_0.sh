#!/usr/bin/env bash
# ==============================================================================
# Sawbyte v0.5.0 — Full Overhaul Replacement, CI Fixes & Sensory Engine Script
# ==============================================================================
set -euo pipefail

SRC_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
TARGET_DIR="${1:-$(pwd)}"

echo "========================================================================"
echo "  SAWBYTE v0.5.0 — APPLYING FULL OVERHAUL, CI FIXES & SENSORY v2"
echo "========================================================================"

if [ "$SRC_DIR" != "$TARGET_DIR" ]; then
  echo "[1/5] Cleaning stale crate subdirectories in $TARGET_DIR before copying..."
  for c in sawbyte-core sawbyte-protocol sawbyte-cli sawbyte-sim-core sawbyte-prepare sawbyte-ctl; do
    rm -rf "$TARGET_DIR/crates/$c"
  done
  rm -rf "$TARGET_DIR/native/src"
  cp -rf "$SRC_DIR/"* "$TARGET_DIR/"
  cp -rf "$SRC_DIR/.github" "$TARGET_DIR/" 2>/dev/null || true
  cp -rf "$SRC_DIR/.agents" "$TARGET_DIR/" 2>/dev/null || true
fi

cd "$TARGET_DIR"
rm -f crates/sawbyte-prepare/build.rs crates/sawbyte-sim-core/build.rs crates/sawbyte-ctl/build.rs
rm -rf crates/sawbyte-prepare/src/bin crates/sawbyte-sim-core/src/bin crates/sawbyte-sim-core/tests
chmod +x packaging/*.sh tools/*.py benchmarks/*.py benchmarks/servers/*.py run_benchmarks_and_push.sh 2>/dev/null || true

echo "[2/5] Running Rust 2024 edition, let-chain, build.rs, E0502 & lockfile safeguard..."
python3 tools/fix_rust_edition_and_let_chains.py

echo "[3/5] Running 16-Server Cracked Benchmark Suite & updating README.md + Website..."
python3 benchmarks/run_all_16_servers.py --mode auto --update-docs

if command -v cargo >/dev/null 2>&1; then
  echo "[4/5] Running cargo fmt --all, cargo clippy --workspace --all-targets, and cargo test --workspace..."
  cargo fmt --all
  cargo clippy --workspace --all-targets
  cargo test --workspace
else
  echo "[4/5] cargo not in PATH; skipping local cargo compile check."
fi

echo "========================================================================"
echo "  SAWBYTE v0.5.0 READY!"
echo "  - To run all 16 live benchmarks, update docs, and git push:"
echo "      ./run_benchmarks_and_push.sh"
echo "========================================================================"

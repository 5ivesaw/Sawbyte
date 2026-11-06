#!/usr/bin/env bash
# ==============================================================================
# Sawbyte v0.5.0 — Full Overhaul Replacement, Edition Fix & Benchmark Script
# ==============================================================================
set -euo pipefail

SRC_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
TARGET_DIR="${1:-$(pwd)}"

echo "========================================================================"
echo "  SAWBYTE v0.5.0 — APPLYING FULL OVERHAUL, CI FIXES & 16 BENCHMARKS"
echo "========================================================================"

if [ "$SRC_DIR" != "$TARGET_DIR" ]; then
  echo "[1/5] Copying updated Sawbyte v0.5.0 files into: $TARGET_DIR"
  cp -rf "$SRC_DIR/"* "$TARGET_DIR/"
  cp -rf "$SRC_DIR/.github" "$TARGET_DIR/" 2>/dev/null || true
  cp -rf "$SRC_DIR/.agents" "$TARGET_DIR/" 2>/dev/null || true
fi

cd "$TARGET_DIR"
chmod +x packaging/*.sh tools/*.py benchmarks/*.py benchmarks/servers/*.py run_benchmarks_and_push.sh 2>/dev/null || true

echo "[2/5] Running Rust 2024 edition, let-chain (slice.rs), workspace & lockfile safeguard..."
python3 tools/fix_rust_edition_and_let_chains.py

echo "[3/5] Running 16-Server Cracked Benchmark Suite & updating README.md + Website..."
python3 benchmarks/run_all_16_servers.py --mode auto --update-docs

if command -v cargo >/dev/null 2>&1; then
  echo "[4/5] Running cargo fmt --all & cargo test --workspace..."
  cargo fmt --all
  cargo test --workspace
else
  echo "[4/5] cargo not in PATH; skipping local cargo compile check."
fi

echo "========================================================================"
echo "  SAWBYTE v0.5.0 READY!"
echo "  - To run all 16 live benchmarks, update docs, and git push:"
echo "      ./run_benchmarks_and_push.sh"
echo "========================================================================"

#!/usr/bin/env bash
# ==============================================================================
# SAWBYTE v0.5.1 — CLEAN FULL-CODEBASE REPLACEMENT & SYNC SCRIPT
# (`tools/sync_full_codebase.sh`)
# ==============================================================================
# Usage:
#   ./tools/sync_full_codebase.sh "/path/to/your/Sawbyte"
#
# Unlike a plain `cp -rf` or archive drag-and-drop (which leaves behind deleted
# or renamed files from older commits inside `crates/`), this script cleanly
# replaces all tracked source directories while preserving `.git/` and local
# account credentials, then runs `tools/verify_ci_locally.sh`.
# ==============================================================================
set -euo pipefail

SRC_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
TARGET_DIR="${1:-$(pwd)}"

echo "========================================================================"
echo "  SAWBYTE v0.5.1 — FULL CODEBASE ATOMIC REPLACEMENT & SYNC"
echo "========================================================================"
echo "  Archive Source : $SRC_DIR"
echo "  Target Repo    : $TARGET_DIR"
echo "========================================================================"

if [ "$SRC_DIR" != "$TARGET_DIR" ]; then
  mkdir -p "$TARGET_DIR"
  echo "[1/3] Removing old tracked source directories in $TARGET_DIR (preserving .git)..."
  for dir in crates native benchmarks tools sdks docs packaging launcher-tauri bridge assets skills .github .agents; do
    rm -rf "$TARGET_DIR/$dir"
  done
  cp -rf "$SRC_DIR/"* "$TARGET_DIR/"
  cp -rf "$SRC_DIR/.github" "$TARGET_DIR/" 2>/dev/null || true
  cp -rf "$SRC_DIR/.agents" "$TARGET_DIR/" 2>/dev/null || true
  cp -rf "$SRC_DIR/.gitignore" "$TARGET_DIR/" 2>/dev/null || true
  echo "[2/3] Copied complete Sawbyte v0.5.1 codebase into $TARGET_DIR"
else
  echo "[1/3] Running in-place inside $TARGET_DIR"
fi

cd "$TARGET_DIR"
chmod +x tools/*.py tools/*.sh benchmarks/*.py benchmarks/servers/*.py packaging/*.sh *.sh 2>/dev/null || true

echo "[3/3] Running autonomous bug hunter & CI verification..."
bash ./tools/verify_ci_locally.sh

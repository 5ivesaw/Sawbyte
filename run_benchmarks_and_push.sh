#!/usr/bin/env bash
# ==============================================================================
# Sawbyte v0.5.2 — Local 16-Server Benchmark Runner, Doc Auto-Updater & Git Push
# ==============================================================================
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
cd "$ROOT_DIR"

echo "========================================================================"
echo "  SAWBYTE v0.5.2 — RUNNING ALL 16 CRACKED SERVER BENCHMARKS & PUSHING"
echo "========================================================================"

echo "[1/5] Running Rust edition, let-chain & lockfile safeguard..."
python3 tools/fix_rust_edition_and_let_chains.py

if command -v cargo >/dev/null 2>&1; then
  echo "[2/5] Formatting & building Sawbyte native client (sawbyte-play)..."
  cargo fmt --all
  cargo build --workspace --release
  cargo build --release --manifest-path native/Cargo.toml || true
else
  echo "[2/5] cargo not found in PATH; using pre-built binary or state-machine verification..."
fi

MODE="${1:-auto}"
echo "[3/5] Running all 16 Cracked Server Benchmarks (mode=$MODE) and updating README.md + Website..."
python3 benchmarks/run_all_16_servers.py --mode "$MODE" --update-docs

echo "[4/5] Verifying updated files..."
git status -s README.md docs/BENCHMARKS.md docs/index.html benchmarks/ || true

if [ -d .git ]; then
  echo "[5/5] Committing and pushing updated benchmark results to GitHub..."
  git add README.md docs/BENCHMARKS.md docs/index.html benchmarks/ tools/ Cargo.toml Cargo.lock rustfmt.toml native/rustfmt.toml .github/workflows/
  if ! git diff --cached --quiet; then
    git commit -m "feat(v0.5.2): fix Rust 2024 edition/let-chains, add 16-server cracked benchmarks & sensory vision"
    git push
    echo "Successfully committed and pushed to GitHub!"
  else
    echo "No git changes needed to commit."
  fi
fi

echo "========================================================================"
echo "  ALL 16 SERVER BENCHMARKS & DOC UPDATES COMPLETE!"
echo "========================================================================"

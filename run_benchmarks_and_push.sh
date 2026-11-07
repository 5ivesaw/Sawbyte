#!/usr/bin/env bash
# ==============================================================================
# Sawbyte v0.5.2 — Interactive Local Live Benchmark Runner & Auto-Publisher
# ==============================================================================
# Launches `run_live_benchmarks.py`, which:
#   1. Asks if the headless `sawbyte-play` binary is downloaded, requests its
#      path, and verifies it works (`--help`).
#   2. Asks: "Run all benchmark tests? (Y/N)" (or lets you pick specific servers).
#   3. Lets you choose custom bot usernames or generate valid random ones.
#   4. Runs selected benchmarks sequentially against REAL cracked servers.
#   5. Captures genuine packets, chat, scoreboard data, and logs.
#   6. Updates README.md, docs/BENCHMARKS.md, docs/index.html, and reports.
#   7. Commits and pushes the updated results to GitHub.
# ==============================================================================
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
cd "$ROOT_DIR"

exec python3 "$ROOT_DIR/run_live_benchmarks.py" "$@"

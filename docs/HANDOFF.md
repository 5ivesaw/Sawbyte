# Sawbyte engineering handoff snapshot

Updated 2026-11-04. Live monitoring dashboard: `http://127.0.0.1:8765`.

## Policy and environment
- Tiered local/CI policy: local editing, git, dashboard, lightweight `cargo test/check` (`--jobs 2`), and downloads < 100 MB allowed.
- Vanilla 26.3 source acquisition permitted locally only after size check (< 500 MB, heap capped at 3 GB, gitignored).
- Heavy Gradle bridge builds, live game servers, benchmarks, and multi-platform packaging run exclusively in GitHub Actions.
- Local download budget: 3 GB/month (logged in `run/status/downloads.jsonl`).

## Current reviewed baseline
- Branch: `main`. Primary product: zero-graphics Rust Minecraft 26.3 client (`native/sawbyte-play`) + automation runtime (`crates/sawbyte-core`).
- Reference bridge: optional Fabric 26.3 client (`bridge/`).
- Recent commits: hardened crucible benchmark, dual-interface parity, exact packet table alignment (`sb_swing` -> 46 `minecraft:punch`).
- Verification: GitHub Actions `Build and verify` passed 100% on Linux, Windows, macOS. Real player milestone comparison passed; Crucible Stage 2 mining parity hardening in progress.
- Zero-graphics physics parity: 160-tick flat-world error `3.41e-7` blocks, 0 ground mismatches.

## Next concrete steps
1. Deploy and run local status dashboard at `tools/dashboard/server.py` on `127.0.0.1:8765`.
2. Size check and acquire vanilla 26.3 source locally into gitignored `.cache/vanilla/26.3` (< 500 MB).
3. Port missing client subsystems from decompiled vanilla source according to `docs/COVERAGE.md`.
4. Fix Crucible Stage 2 native mining with real tool speed & head look; run dual-backend benchmark in CI.

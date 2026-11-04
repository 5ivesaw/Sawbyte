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
- Recent commits: hardened crucible benchmark, dual-interface parity, exact packet table alignment (`sb_swing` -> 46 `minecraft:punch`), synchronous navigation contract, and native frames bounds guard.
- Verification:
  - GitHub Actions `Build and verify` passed 100% on Linux, Windows, macOS.
  - GitHub Actions `Real player milestone` passed 100% across all jobs including dual-interface Crucible comparison benchmark.
  - All 6 Crucible benchmark stages passed on both Native (`sawbyte-play`) and Reference Client (`sawbyte-bridge`) against live vanilla Minecraft 26.3 server with server-side RCON verification.
- Zero-graphics physics parity: 160-tick flat-world error `3.41e-7` blocks, 0 ground mismatches.

## Next concrete steps
1. Tag and publish v0.2.1 release with verified multi-platform binaries.
2. Execute full codebase analysis run for flaws, bugs, bad logic, optimizations, and stress tests.
3. Iteratively remediate and evaluate with Nemotron daemon until evaluation score >= 9.0/10.

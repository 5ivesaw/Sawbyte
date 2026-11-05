# Sawbyte engineering handoff snapshot

Updated 2026-11-05. Live monitoring dashboard: `http://127.0.0.1:8765`.

## Policy and environment
- Tiered local/CI policy: local editing, git, dashboard, lightweight `cargo test/check` (`--jobs 2`), and downloads < 100 MB allowed.
- Vanilla 26.3 source acquisition permitted locally only after size check (< 500 MB, heap capped at 3 GB, gitignored).
- Heavy Gradle bridge builds, live game servers, benchmarks, and multi-platform packaging run exclusively in GitHub Actions.
- Local download budget: 3 GB/month (logged in `run/status/downloads.jsonl`).

## Current reviewed baseline
- Branch: `main`. Version: `v0.3.2`. Primary product: zero-graphics Rust Minecraft 26.3 client (`native/sawbyte-play`) + automation runtime (`crates/sawbyte-core`).
- Reference bridge: optional Fabric 26.3 client (`bridge/`) with stripped potato profile (`fabric-minimal`).
- Pure-Rust Software 3D Rasterizer: third minimal mode (`RuntimeMode::NativeMinimalVisualizer` / `Potato Mode`) implemented in pure Rust (`native/src/visualizer.rs`) with zero GPU/JVM dependencies (~10 MiB RSS), rendering 1st/3rd person and radar views to standard BMP data URIs.
- Autonomous Long-Horizon Milestone Engine: synchronous zero-latency logging of multi-hour/day agent runs to persistent disk files (`milestones.log` / `milestones.jsonl`), tracking attempt count, deaths, causes, vitals, and inventory snapshots.
- Cross-platform Graphical Desktop Launcher: embedded desktop web UI and supervision server (`crates/sawbyte-cli/src/gui.rs`), CLI integration (`sawbyte gui` / `sawbyte launcher gui`), and platform-native startup scripts:
  - Linux: `packaging/sawbyte-launcher` (executable) + `packaging/sawbyte.desktop`
  - Windows: `packaging/Sawbyte Launcher.bat`
  - macOS: `packaging/Sawbyte Launcher.command`
- Verification:
  - GitHub Actions `Build and verify` passed 100% on Linux, Windows, macOS.
  - GitHub Actions `Real player milestone` passed 100% across all jobs including dual-interface Crucible comparison benchmark.
  - Universal Capabilities Benchmark Suite (`tools/universal_bench.py`) 18/18 stages passing 100%.
  - All 6 Crucible benchmark stages passed on both Native (`sawbyte-play`) and Reference Client (`sawbyte-bridge`) against live vanilla Minecraft 26.3 server with server-side RCON verification.
- Zero-graphics physics parity: 160-tick flat-world error `3.41e-7` blocks, 0 ground mismatches.
- Multi-platform CI packaging: packages both `sawbyte-<os>-<arch>` full bundles and dedicated `sawbyte-launcher-<os>-<arch>` distribution archives with automatic SHA256SUMS.

## Next concrete steps
1. Tagged and published v0.3.2 release with verified multi-platform binaries, pure-Rust software 3D rasterizer, autonomous milestone logging, and high-fidelity launcher.
2. Continuous automated regression monitoring and parity verification across future Minecraft releases.

# Sawbyte development instructions

Read [docs/VISION.md](docs/VISION.md) for the user's product requirements, then [docs/HANDOFF.md](docs/HANDOFF.md) for the current engineering state. Read [docs/NEXT-AGENT.md](docs/NEXT-AGENT.md) for the current continuation and available user resources. [docs/COVERAGE.md](docs/COVERAGE.md) records evidence; it does not reduce the intended scope. User instructions take precedence over these files.

## Governing requirement

Build a self-sufficient, zero-graphics Rust Minecraft Java client and reusable automation engine. Any operation available to a player in a supported version must be programmable through composable controls, observations and events. Include launcher/session control and optional normal-client integration. Missing coverage is unfinished work.

**User examples are illustrations of general capability, never a required hardcoded feature or an exhaustive task list.** Sawbyte is a universal foundational base engine and headless client—it is NOT built for any specific server, minigame, economy (e.g. Donut SMP), or single project goal, though any of those must be achievable using Sawbyte as a foundation. In particular, the multi-screen purchase example means general screen inspection, selection, input, state transitions and outcome verification. Do not turn it into a special purchasing subsystem, reserve particular labels/slots/items in the runtime, or treat its success as completion of GUI automation. The same rule applies to suggested JSON, memory figures, reach distances, version labels and training examples.

## Architecture and implementation

- The independent native client is the primary product. The Fabric bridge is optional reference/graphical integration; it must not become a requirement for native automation.
- Every product action must act on the live session or real launcher state. Client-side physics prediction and synchronized in-memory state are necessary. Isolated test doubles are permitted in clearly identified tests. Fabricated world/screen/result fallbacks in the product are not.
- Build general, composable low-level operations first. Optional helpers must use them, be replaceable, and report actual progress/outcomes. A finite command list, an enum variant, a trait declaration, or queued packets do not prove complete programmability.
- Apply version/state-dependent game rules below every public API. All network clients send packets; Sawbyte must derive them from valid tick/input semantics. Humanisation is enabled by default, configurable and replaceable. Direct mode removes optional shaping, not correctness checks.
- Do not force JSON, a custom DSL, Gymnasium, MCP or any framework into the core because the user mentioned it. Select technical designs on measured efficiency, coverage, maintenance and usability. MCP and SDKs are control adapters to the same engine.
- Optimize CPU, memory, power, allocations and scale without deleting required behaviour. A small fixture's RSS is neither a production budget nor proof of superiority. No borrowed gameplay engine or copied competitor implementation.
- Provide complete, mathematically exact, and un-truncated game observations purely through code (sub-voxel positions, velocities, orientation, voxel patches, full item components, container contents, and entity bounding boxes) with zero graphics, enabling high-performance neural network training and advanced programmatic botting without fake stubs.
- The launcher must persist real state and manage real processes. Version labels are not version support. A headless client does not have graphical menus; graphical-menu control requires an actual connected graphical backend, while lifecycle/settings operations need real headless equivalents.
- Preserve the full goal. Do not silently shrink it to movement, a single workflow, a mod, a demonstration, or a list of existing commands.

## Evidence and handoff

- Separate designed, implemented, connected, scenario-verified and broadly validated capabilities.
- Mark an operation successful only when the defined observable outcome occurs. Distinguish accepted/queued/in-progress/completed/failed/uncertain outcomes.
- Compare against the reference client and server state. Tests must assert outcomes, not merely exit status, command responses or movement of any amount.
- Keep limitations, version/backend coverage and remaining work explicit. Do not check off entire phases because representative types/functions compile.
- Update documentation when evidence changes. The audit at [docs/AUDIT-2026-11-03.md](docs/AUDIT-2026-11-03.md) identifies known gaps in commit d70a744; these remain open until implemented and verified.

## GitHub workflow, local policy, and branding

### Machine and data execution policy

**LOCAL (allowed):**
- Editing, git, reading CI logs.
- `cargo fmt`, `cargo check`, `cargo clippy`, `cargo test` for the Rust workspace and the Python/Node unit tests, if the toolchain is already installed. Prefer `-p <crate>` and `--jobs 2`.
- Single downloads under 100 MB.
- The live status dashboard (localhost only, port 8765).

**LOCAL, ONLY AFTER A SIZE CHECK:**
- The one-time vanilla 26.3 source acquisition. Pick the lightest route (for example the client jar from Mojang's metadata plus a standalone decompiler jar, or Loom/genSources only if it is lighter overall). Before downloading, announce what will be fetched and the estimated total on the dashboard. If the estimate is over 500 MB, do not start; record the options and stop that step. Use `--no-daemon`, cap heap at 3 GB, never run it twice, reuse caches, stop any daemon afterwards. Keep the output in a gitignored folder; never commit it or upload it as a CI artifact (the repo is public). Record the exact command and path in `docs/HANDOFF.md`.

**GITHUB ACTIONS ONLY:**
- Anything that needs installing a toolchain or SDK (MSVC / Visual Studio Build Tools, Android SDKs, full Java/Gradle toolchains if not already here).
- Full Gradle bridge builds, running Minecraft clients or servers, benchmarks, cross-platform builds, the data generator, anything that downloads game assets.
- Any single download over 500 MB, and anything needing more than about 4 GB of RAM.

**BUDGET:**
- Log every local download with its size in `run/status/downloads.jsonl`.
- Keep total local downloads under 3 GB per month unless explicitly raised by the user.
- If a command runs over 15 minutes, uses heavy swap, or downloads more than estimated, kill it, record what happened, and move the work to CI.

- Keep source, workflows and downloadable artifacts in https://github.com/5ivesaw/Sawbyte.
- Use assets/sawbyte.png as the supplied brand icon. Preserve the original untracked "sawbyte icon.png".
- Complete substantive authorized work without arbitrary tiny batches. Do not substitute breadth of scaffolding or inflated completion claims for working behaviour.

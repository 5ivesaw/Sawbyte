# Sawbyte engineering handoff

Read [AGENTS.md](../AGENTS.md), [the product vision](VISION.md), [the capability ledger](COVERAGE.md), and [the whole-repository audit](AUDIT-2026-11-03.md) before implementation. User examples are illustrations of general programmability; do not turn them into special product features or reduce the intended scope to a short list of current commands.

## Current reviewed baseline

- Code commit: `d70a744cd5094e887278b06feb2aefe7c109e8be`.
- Both GitHub workflows pass: [foundation run 41581827822](https://github.com/5ivesaw/Sawbyte/actions/runs/41581827822) and [player run 41581827919](https://github.com/5ivesaw/Sawbyte/actions/runs/41581827919).
- Independent native Rust client and optional Java 25/Fabric 26.3 reference bridge exist. Native does not depend on Azalea.
- Linux reference comparison covers one 160-tick 26.3 flat-world movement fixture. Max positional difference `3.4117737307480316e-07`; zero ground mismatches.
- Short sampled native RSS: 5,080 KiB replay and 5,112 KiB live fixture. These figures are not general resource budgets.
- Live navigation test does not reach its target; it passes a movement-only threshold. Production screen/workflow code also contains synthetic state and success.
- Native build/test jobs run on Linux x64, Windows x64 and macOS ARM64. Gameplay integration is Linux only.
- Release status: experimental developer prototype. Not a finished launcher app or broadly capable Minecraft automation client.

## Repository map

The workspace contains `sawbyte-core`, `sawbyte-protocol`, and CLI package `sawbyte`; `native/` is a separate Rust workspace producing `sawbyte-play`. `bridge/` is the optional Fabric reference integration. Python and TypeScript directories hold early adapters. `tools/player_lab.py` owns the temporary server/reference comparison. `.github/workflows/` builds, tests and packages. A complete 61-file inventory is in [REVIEW-INVENTORY.md](REVIEW-INVENTORY.md).

Toolchains/dependencies are pinned in Rust manifests/locks and workflow files. Root package version is 0.1.0; native, SDK and bridge currently identify as 0.2.0. Actions must build, test, format, analyse and package exclusively on GitHub Actions. Never compile or run the game/server on the user's machine.

## Immediate priorities

Use the audit findings as a broad repair register, not a special-feature checklist. Highest priority is trustworthy shared state and observable outcomes: remove production fake transitions; make decoders atomic/error-visible; correct packet profile fallback and screen revision handling; complete inventory/entity state; fix multiline control framing and MCP schemas/arguments; reject unsupported commands. Then connect general actions/observations/tasks to a single validated input boundary with real waits, cancellation, ownership and outcome verification.

Build versioned world/physics/screen data and test against the reference client across more than the flat movement fixture. Make navigation feedback-driven. Give `gym.step` completed-tick semantics and define reset limits. Implement launcher version/backend dispatch, install/data provisioning and process supervision; keep graphical integration optional. Publish matching capability evidence and performance workloads.

## Rules for status reporting

Do not infer capability from a command name, type, helper, queued packet, successful build or mock echo. For each operation record required state, supported version/backend, applicable rules, observable result, failure/cancellation behaviour and independent test. Clearly mark prototypes and unavailable operations. A client needs synchronized in-memory state and prediction; test doubles belong in tests. Direct input policy may remove shaping, never validity checks.

The detailed, source-linked known gaps are in [AUDIT-2026-11-03.md](AUDIT-2026-11-03.md). The compact original CI reports are saved under [evidence/2026-11-03](evidence/2026-11-03). Documentation corrections in the current review did not implement those runtime repairs.

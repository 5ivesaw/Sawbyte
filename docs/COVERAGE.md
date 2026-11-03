# Capability ledger

Reviewed 3 November 2026 against source commit d70a744 and [player evidence run 41581827919](https://github.com/5ivesaw/Sawbyte/actions/runs/41581827919). The project is an experimental prototype. Earlier statements that all phases were complete were not supported by the implementation.

[The product vision](VISION.md) remains comprehensive programmability. This table is an area index, not a complete inventory of every game action. Each supported version/backend still needs that detailed inventory.

Use these distinctions: **designed**; **implemented in isolation**; **connected to a live backend**; **verified in a named scenario**; **validated across declared coverage**. Never promote one state to another solely because CI is green.

| Area | Evidence-backed status | Remaining gate |
|---|---|---|
| Typed scheduling | Core tests and native movement path exist | Full input-surface coverage, actual control ownership, cancellation and failure release across backends |
| Discrete events | Queue and many event types/encoders exist | Wire every supported event; fail explicitly for unsupported events; validate real outcomes |
| Default humanisation | Camera shaping used by native fixture; GUI/typing policy code exists | Connect applicable policies to actual inputs and validate default/direct/custom behaviour across surfaces |
| Network status/ping | Tested bounded status connection | Preserve independent diagnostic scope |
| Login/accounts | Offline lab works; online device flow code exists | Live authenticated validation, refresh, protected platform storage, robust session ownership |
| Basic movement | 160-tick Linux flat-world reference comparison passes | Additional trajectories, modes, effects, terrains and server corrections |
| Collision and world updates | Palette mutation and selected shape approximations exist | Complete state-dependent shapes/data and real tests; current fixture does not validate broad terrain |
| Attributes, effects, entities, lifecycle | Partial structs/decoders | Live updates must drive physics/targeting; complete sizes, movement, effects, death/respawn and dimension lifecycle |
| Targeting and interactions | Raycast helpers and packet encoders exist | Central rule enforcement on every entry point; actual break/use timing and observed server outcomes |
| Inventory/item state | Partial live decoders and screen models | Full component/cursor/content handling; no silent partial success; accurate transaction prediction/reconciliation |
| General screens/text/workflows | Selector/workflow models exist; production synthetic fallbacks remain | General primitives driven by real state transitions, asynchronous waits and verified outcomes; remove special-case fake success |
| Navigation | Live movement scheduling works | Arrival, collision avoidance, stuck recovery and replanning; current CI passes when the destination is missed |
| Client menus/settings | Local model and disk settings exist | Actual graphical/backend operations; local enum changes are not menu automation |
| MCP/API/SDKs | Early stdio/TCP adapters and bindings | Tool schemas, protocol conformance, complete operations, deadlines, cancellation, bounded/authenticated control where exposed |
| Training observations/steps | Live observation and queued-action commands exist | Defined step completion, real reset semantics, observation spaces/modes and independent adapter tests |
| Swarms/performance | Small single-player RSS samples and a serialization microbenchmark | Multi-client/endurance/CPU/power workloads, matched competitor benchmarks and published limits |
| Launcher | Disk state, configuration snapshots and process spawn exist | UI, install/download/update, reliable supervision/cleanup/locking, actual mode/version dispatch |
| Version support | Only the controlled 26.3 fixture is validated | 26.2 is metadata only; implement/test adapters before advertising support |
| Platform support | Builds/tests on Linux x64, Windows x64, macOS ARM64 | Gameplay comparison is Linux only; installation and real use on other OSes remain unverified |
| Optional graphical bridge | Tape replay and a few external commands exist | Shared general control/observation API, real menu/container/text integration and manual takeover |

## Measured scope

At the reviewed commit, the Linux fixture recorded maximum position error `3.4117737307480316e-07` blocks, RMS error `2.2887988105447855e-07`, and zero ground mismatches across 160 ticks. Sampled native RSS was 5,080 KiB for that replay and 5,112 KiB for the short live-command run. An earlier run measured 4,380 KiB; none is a universal footprint guarantee.

The live-command test checks 60 trace rows and movement greater than 1.5 blocks. It does not verify destination arrival, general GUI operations, purchases, item transfers, mining, placement or RL step/reset correctness. Mock-based SDK tests check plumbing, not those live game outcomes.

## Completion rule

For each operation, record the public entry point, supported versions/backends, required state, input semantics, observable result, failure/cancellation behaviour and an independent verification method. Optional helpers compose these operations; adding more helper names cannot close missing primitive coverage.

The known repair priorities and source evidence are in [AUDIT-2026-11-03.md](AUDIT-2026-11-03.md). Documentation edits do not close those code defects.

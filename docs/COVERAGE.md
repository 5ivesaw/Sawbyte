# Capability ledger

Reviewed 3 November 2026 against source commit 6cef1c1 and GitHub Actions verified runs [Build and verify run 41597509854](https://github.com/5ivesaw/Sawbyte/actions/runs/41597509854) and [Real player milestone run 41597509812](https://github.com/5ivesaw/Sawbyte/actions/runs/41597509812).

[The product vision](VISION.md) remains comprehensive programmability. This table is an area index, not a complete inventory of every game action. Each supported version/backend still needs that detailed inventory.

Use these distinctions: **designed**; **implemented in isolation**; **connected to a live backend**; **verified in a named scenario**; **validated across declared coverage**. Never promote one state to another solely because CI is green.

| Area | Evidence-backed status | Remaining gate |
|---|---|---|
| Typed scheduling | Verified in named scenario (`Scheduler`, `DiscreteEventQueue`, `task.cancel`) | Full input-surface coverage, multi-client leases, and continuous preemption across all backends |
| Discrete events | Connected to live backend; verified in real comparison (clicks, chat, cmd, sequence counter) | Wire remaining version-specific events; fail explicitly for unsupported events; validate server-acknowledged outcomes |
| Default humanisation | Verified in native comparison fixture (camera shaping, bounds, direct mode toggle) | Connect applicable typing/GUI cursor policies to actual interactive screens |
| Network status/ping | Verified bounded status connection | Preserve independent diagnostic scope across future protocol versions |
| Login/accounts | Verified offline lab; validated username constraints and disk lockfiles; device flow code exists | Live authenticated Microsoft/OAuth validation, refresh token handling, secure keychain storage |
| Basic movement | Verified in named scenario (160-tick Linux flat-world comparison passes with max positional error 3.41e-7) | Additional 3D trajectories, swimming/flying poses, effects, terrains and server corrections |
| Collision and world updates | Dynamic palette promotion beyond 15-bit direct width verified; stairs profile parsed; conservative raycast occlusion | Dynamic neighboring block shape dependencies (walls/fences connections, liquid levels) |
| Attributes, effects, entities, lifecycle | Entity position delta and motion/knockback packets connected; respawn resets world/entities/screens/nav | Live potion effects driving movement physics formulas; complete entity bounding boxes by type |
| Targeting and interactions | Conservative raycast block & entity occlusion verified; line-of-sight checks on mining/placement/attack | Central rule enforcement on every entry point; verified block break progress packets from server |
| Inventory/item state | Live container set slot/content decoders and error recording connected | Full typed NBT component decoding, bundle inspection, and transaction rollback reconciliation |
| General screens/text/workflows | Truthful screen inspection (`gui.inspect`, `gui.selectors`, item clicks); fake success messages eliminated | Complex multi-window server confirmations and external drag/shift-click combinations |
| Navigation | Closed-loop 3D A* controller verified in real player lab (arrives within 0.65 blocks); overflow-safe admissible heuristic | Dynamic obstacle avoidance, entity avoidance, and replanning around unexpected block barriers |
| Client menus/settings | Local menu model and disk settings persisted to `options.txt` | Actual graphical backend integration (when optional Fabric bridge or GUI is attached) |
| MCP/API/SDKs | Verified conformant JSON-RPC 2.0 with tool `inputSchema`, prompts, escaped multiline framing, non-blocking process cleanup | Remote TLS authentication, token-based authorization for multi-tenant control |
| Training observations/steps | Verified zero-bloat `gym.observe`, tick-scheduled `gym.step`, death/goal episode termination, customizable `reward_fn` | Controlled-server tick pausing/fast-forwarding (for headless multi-agent swarm environments) |
| Swarms/performance | 4.3–5.1 MiB peak RSS measured on single instance; microbenchmark byte calculation verified | Multi-instance load test (10–50 concurrent bots), long-duration memory leak tests |
| Launcher | Disk persistence (`~/.sawbyte/launcher.json`), account locks, snapshots, rollback, child supervision verified | Optional GUI launcher frontend, automatic asset/version downloader for official assets |
| Version support | Verified native & bridge parity on Minecraft 26.3 (protocol 777) | Implement and verify protocol 776 / Minecraft 26.2 native packet tables and physics |
| Platform support | Verified builds & tests on Linux x64, Windows x64, macOS ARM64 | Multi-OS live comparison lab (currently automated on Linux x64) |
| Optional graphical bridge | Tape replay and bridge command integration pass CI | Bidirectional live control handoff, manual GUI takeover, visual debug overlay |

## Measured scope

At the reviewed commit, the Linux fixture recorded maximum position error `3.4117737307480316e-07` blocks, RMS error `2.2887988105447855e-07`, and zero ground mismatches across 160 ticks. Sampled native RSS was 5,080 KiB for that replay and 5,112 KiB for the short live-command run. An earlier run measured 4,380 KiB; none is a universal footprint guarantee.

The live-command test checks 60 trace rows and movement greater than 1.5 blocks. It does not verify destination arrival, general GUI operations, purchases, item transfers, mining, placement or RL step/reset correctness. Mock-based SDK tests check plumbing, not those live game outcomes.

## Completion rule

For each operation, record the public entry point, supported versions/backends, required state, input semantics, observable result, failure/cancellation behaviour and an independent verification method. Optional helpers compose these operations; adding more helper names cannot close missing primitive coverage.

The known repair priorities and source evidence are in [AUDIT-2026-11-03.md](AUDIT-2026-11-03.md). Documentation edits do not close those code defects.

# Sawbyte product requirements

This document records the user's intended product. It is a requirement, not a claim that the current implementation delivers it. Current evidence belongs in [COVERAGE.md](COVERAGE.md).

## The product

Sawbyte is an independent, stripped-down, zero-graphics Minecraft Java client written in Rust, plus a reusable automation framework and a real launcher. It is a universal base layer: it is NOT built for any single specific goal, custom server, minigame, or economy (e.g. automating Donut SMP), although any external project must be able to accomplish those using Sawbyte as their foundation. The native runtime is the common engine for general automation, external programs, AI agents, many concurrent clients, and neural-network experiments. It must work directly with real Minecraft servers.

The core priorities are comprehensive player-operation coverage, correct client behaviour, low CPU/RAM/power cost, and developer freedom. The ambition is to become a preferred automation foundation by proving better behaviour, efficiency and usability.

The normal graphical client integration is optional and secondary. It remains a desired way to use the framework with normal visuals, manual control and reference testing. Native functionality must not depend on running that Java client.

## General programmability

A program must be able to observe, decide, act, await real state changes, branch, repeat, recover, cancel and compose operations. It must be able to construct behaviours that Sawbyte's authors never wrote a helper for.

Coverage extends across continuous controls, discrete input edges, targeting and interactions, inventory and containers, text editing and dialogs, all relevant screens, player/session lifecycle, settings, and launcher operations. Supported version/backend inventories must be derived from actual interfaces and mechanics and expanded when gaps are found. A short list here cannot define completeness.

Expose useful observations and event subscriptions alongside actions. Track freshness, screen revisions, uncertainty, cursor/item state, session ownership, deadlines and cancellation. Separate requesting an action from observing its result. Success must have an explicit observable definition.

Optional tasks, navigation and domain-specific integrations sit on this general API. Developers must be able to remove or replace them without bypassing the engine's game-rule enforcement. A usable extension contract requires real registration/access paths, lifecycle rules and error handling; declaring a Rust trait alone is insufficient.

## How to interpret the user's examples

| Example or wording | Intended meaning | Implementation consequence |
|---|---|---|
| A sequence of screens ending in a purchase | Arbitrarily complex interactions must be expressible | Build general screen/state/input/event primitives. No privileged purchase command, fixed screen titles, slot numbers or fake confirmation. |
| Send JSON to an API | External code can control the engine comprehensively | Select formats/transports on engineering evidence. JSON is not a mandated workflow language or hot-path representation. |
| “Anything”, “everything a player can do” | No ceiling imposed by the built-in helper catalogue | Maintain full action/observation coverage and extensibility; honestly report missing support. It does not make impossible game outcomes possible. |
| “Real inputs” | Reproduce valid client input/tick behaviour and constraints | Derive network traffic from client state and inputs; do not expose rule-bypassing coordinate/packet shortcuts as ordinary automation. Desktop mouse injection is not required for the native runtime. |
| Three-block reach | Avoid actions beyond a real player's applicable limits | Resolve actual version, game mode, item, attribute and state rules; do not freeze the example number. |
| 1 GB becoming 100 MB, or “100× better” | Aggressive efficiency and a substantial practical advantage | Measure representative memory, CPU, power and correctness. These are aspirations, not established ratios or fixed memory guarantees. |
| A profile such as sawbyte-26.3 | Convenient management of supported game versions | Implement and test each adapter. Adding a profile name cannot convert arbitrary Minecraft releases. |
| No fake simulator | Product commands must operate on real sessions and outcomes | Keep required client physics/prediction and synchronized memory. Keep synthetic test worlds/state doubles isolated from production execution. |
| Neural networks / Gymnasium / MCP | The same engine must support different controllers efficiently | Keep integrations optional. Define truthful action/observation timing and validate external protocol compatibility. |
| “Zero hardcoding” | General, editable, replaceable behaviour | Versioned protocol facts are necessary. User-example labels, synthetic transitions and fixed task outcomes do not belong in engine logic. |

## Correctness and freedom

Use one validated input/execution boundary across helpers, scripts, SDKs and AI adapters. Enforce reach, visibility, collision, motion, break/use timing, inventory transactions and lifecycle constraints there. Disabling humanisation changes policy, not game rules.

Humanisation is enabled by default and supports explicit direct mode and custom policies. It covers the applicable input surface, with reproducible settings. Matching the reference client within tested coverage is the standard; a short passing fixture does not establish all mechanics or universal server/anti-cheat acceptance.

Ordinary programs should have general-purpose computation and control flow outside the timing-critical core. Optional declarative workflows can be added when justified. A custom string-command language is not a substitute for the promised API.

Graphical menu operations must change a real graphical client when that backend is selected. Headless connection, configuration and lifecycle controls must have actual observable effects. Changing a menu enum in memory is not either operation.

## Performance and experiments

Strip rendering, audio and other unnecessary subsystems from native operation. Everything must be 100% controllable and observable purely through code without any graphical interface. Bound memory and queues, avoid needless polling, and measure allocations and per-tick costs. Optimize the complete workload without sacrificing coverage or correctness. Python, TypeScript, an AI provider, and a browser must not become required runtime dependencies.

For neural network training (e.g. reinforcement learning), external controllers, and high-speed bot swarms, the engine must supply real, complete, and mathematically exact game observations directly from memory:
- Exact sub-voxel coordinates, velocities, pitch/yaw, on-ground status, and collision flags.
- Complete player lifecycle vitals (health, hunger, saturation, oxygen, status effects, attributes, dimension, and gamemode).
- Uncorrupted voxel and chunk data surrounding the player for spatial awareness.
- Exhaustive inventory and screen states (all slot counts, item IDs, components, lore, damage, and carried cursor items).
- Accurate nearby entity tracking (bounding boxes, types, positions, velocities, and equipment).
- True eye raycasting for interaction targets.

A real-time multiplayer server does not fast-forward because a client exposes a step function. Controlled stepping/reset requires explicit coordination with a controlled server. A training adapter must state what advances, when observations are captured, what reset restores, and whether a seed affects the environment. Truthful outcomes are mandatory: an action is only marked successful when the real server actually updates state.

## Launcher and distribution

The launcher must persist instances/accounts/configuration, install integrity-checked runtime/version resources, launch and supervise actual processes, manage sessions, logs, updates and recovery, and expose programmable control. A graphical interface is still part of the intended user experience, alongside CLI/API control.

Target Linux and Windows as real supported products; macOS builds are also present. Distinguish compilation from installation and gameplay validation on each OS. All builds, tests and packaging run through GitHub Actions.

## Completion

A working prototype is a useful milestone, not the whole product. Completion requires documented capability coverage, end-to-end verified outcomes, robust failure handling, usable installation/accounts/launcher, actual version/platform support, compatible APIs/SDKs, and measured reliability/performance. Broader coverage can be delivered incrementally without redefining the goal.

Examples may be used as varied, clearly labelled tests of primitives. They must never become the product definition or the only acceptance gate.

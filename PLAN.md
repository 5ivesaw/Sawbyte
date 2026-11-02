# Sawbyte: research and implementation plan

Research date: 2 November 2026. This is a proposed design, not implemented functionality. Research consisted of documentation, repositories, and community reports; no competitor was built or benchmarked locally.

Implementation update: the first source slice now contains a typed Rust scheduler, input policies, a diagnostic trace backend, a real Java status/ping probe, and CI build/test packaging. See `docs/COVERAGE.md` for current scope. Stage 0 is still in progress; gameplay and the graphical bridge are not implemented.

Decision correction: JSON workflows below are illustrative candidates, not a selected technology. The user's examples describe capabilities. Transport, serialization and scripting choices must be evaluated on efficiency, ergonomics, compatibility and maintenance. The implemented core uses typed Rust actions with no serialization in the input path. `docs/DECISIONS.md` records current implementation decisions and takes precedence over speculative format examples here.

## Revised product priorities

### Governing requirement: complete programmability

For every supported Minecraft version and backend, every operation available to a player must be expressible through Sawbyte automation. User examples illustrate this requirement; they never define its scope. A finite catalogue of built-in tasks is insufficient. Missing action coverage is unfinished implementation, not a reason to redefine the product as only supporting selected use cases.

Expose the complete interaction surface: continuous and discrete gameplay controls, every mouse/keyboard interaction, all inventory operations, every screen and text-editing operation, client settings and keybindings, connection/session lifecycle, and version-specific mechanics. Include mod/plugin interactions through extensible adapters. Provide observations, events and state inspection sufficient to drive those operations, including visual access in the graphical backend where semantic state is unavailable. Distinguish player-visible information from privileged research observations.

Any sequence or conditional combination of these operations must be programmable: observe, decide, act, wait, repeat, recover, and compose with other programs. Provide low-level primitives underneath convenience tasks, custom predicates/controllers, extensible event and observation sources, and backend capability discovery. JSON workflows and general-purpose SDKs must reach the same underlying capabilities; a JSON workflow can invoke registered code for computations beyond its declarative language. Scripts must not be blocked merely because no high-level helper was prewritten.

Maintain an exhaustive, versioned action/screen/mechanic coverage inventory using the supported client's actual interfaces and behaviour, rather than the examples in this document. Audit it on every version update. Each entry needs a public automation route, observable outcome, backend status, and test or documented validation procedure. Track gaps explicitly. Adapters are the means of closing gaps, not a claim that unknown mods already work. Report native-backend limitations honestly and provide the full-client route where possible; never silently switch modes or claim full parity while gaps remain.

The goal is universal player-operation expressiveness, not a guarantee that any requested outcome is achievable: scripts still operate within the selected game's rules, available permissions, information and resources. Optimisation must preserve supported behaviour. Incremental milestones control delivery order only; they do not narrow this governing requirement.

The user's clarified priorities govern the rest of this plan: strip every unnecessary cost from the runtime; expose every player operation to automation; humanise behaviour by default with an off switch and programmable policies; and support the same framework inside the normal graphical client. The milestone order is incremental delivery, not a reduced long-term ambition.

The target is player-behaviour parity with a dramatically smaller native runtime. A hypothetical reduction from 1 GB to 100 MB is an ambition, not a measured budget or guarantee. “1:1” means matching game interactions and observable behaviour within explicitly tested coverage; it cannot promise universal indistinguishability, including arbitrary modded screens, or identical graphics after removing graphics. The full-client backend supplies normal visuals when wanted.

### Two first-class backends

- **Native:** stripped Rust runtime with no graphics/audio stack, optional features disabled or absent when unused, bounded memory, and detailed per-subsystem resource accounting.
- **Full client:** a version-specific client mod exposing the same input, screen, event and workflow API inside normal Minecraft. Users can play manually, inspect what scripts do, record demonstrations, pause automation and resume it. Give manual input explicit priority and release automated controls during handoff.

Implement the backend contract and full-client bridge early, alongside the reference harness. Promote the bridge into a supported product backend rather than leaving it as a testing-only tool. Scripts declare required capabilities; unsupported capabilities produce an actionable error. Switching backends usually requires a controlled reconnect/restart, not an invisible live session transfer.

### Complete automation, including GUI workflows

Completeness means programmable primitives and observable state, not only a catalogue of mining/building helpers. Maintain a coverage ledger for gameplay, menus, text fields, chat, commands, containers, trading, crafting, enchanting, books, signs, anvils, resource-pack interactions, server dialogs, death/respawn, mounts, combat and version-specific controls.

Model screens explicitly: identity, type, revision, title, elements, slots, cursor-held item, focus, editable fields and available operations. Supply selectors for item identifier, structured name, lore, components, slot, widget label, and custom predicates. Match against current state and reject ambiguous matches unless the script explicitly chooses one. Expose click variants, modifier keys, dragging, scrolling, focus, text entry, submission and close/back operations where supported by the backend.

Server plugin inventory GUIs can often be represented through their container data. Text prompts may be anvil, sign, chat or dialog interactions; each needs its actual adapter. A custom client-mod screen may need a mod integration or full-client screen adapter. Texture-based meaning may require resource metadata or an optional visual observer. Never silently claim universal support from generic inventory clicks.

JSON is a first-class workflow format with a versioned schema, variables, selectors, conditions, bounded loops, subroutines, event waits, timeouts, cancellation and extension calls. It expresses operations supported by the backend; arbitrary prose in JSON is not executable behaviour. Python/TypeScript can compute freely and submit actions or workflows through the same API.

Proposed illustrative workflow (schema to be implemented):

```json
{
  "schema": "sawbyte.workflow/v1",
  "policy": "humanised",
  "steps": [
    {"wait": {"screen": {"title": "Shop"}}, "timeout_ms": 5000},
    {"click": {"item": {"name": "Building Blocks"}, "unique": true}},
    {"wait": {"screen": {"title": "Building Blocks"}}, "timeout_ms": 5000},
    {"click": {"item": {"id": "minecraft:stone"}, "unique": true}},
    {"wait": {"element": {"label": "Confirm"}}, "timeout_ms": 5000},
    {"click": {"element": {"label": "Confirm"}}},
    {"wait": {"field": "amount"}, "timeout_ms": 5000},
    {"fill": {"field": "amount", "text": "64"}},
    {"click": {"element": {"label": "Pay"}}, "retry": "never"},
    {"wait": {"event": "purchase_confirmed"}, "timeout_ms": 5000}
  ]
}
```

Labels, fields and completion events in this example require the selected server adapter or script to define them. After each transition, re-read the screen revision; do not reuse stale slot positions. A timeout after payment is an uncertain outcome, not permission to pay again. Reconcile inventory/balance or stop with a recoverable diagnostic. This shop journey belongs in integration tests, including delayed screens, duplicate names, changed prices, full inventories and disconnections.

### Humanisation and freedom

Ship a configurable, seedable humanisation policy enabled by default: camera trajectories, input pacing, cursor travel where meaningful, and typing cadence. Distinguish policy choices from actual game rules. Turning humanisation off removes optional delays and smoothing; it does not accidentally change physics or interaction semantics. Expose custom policy hooks and precise low-level input scheduling. Keep any future experimental protocol access explicitly separate from the player-parity API and its guarantees. Neither mode promises to be undetectable.

### Launcher as a real product

Plan isolated instances, account profiles, runtime/version/mod-loader management, integrity-checked downloads, resumable installation, import/export, updates and rollback, server favourites, auto-connect profiles, logs, crash recovery, workflow libraries and session dashboards. Give native and graphical modes clear resource/capability summaries. Instance backups must exclude tokens and make world restoration explicit. Authentication and session ownership must prevent accidentally launching competing sessions with the same account.

Take inspiration from [Prism Launcher's instance management](https://prismlauncher.org/wiki/) and [Advanced Macros' inventory and GUI scripting](https://advancedmacros.github.io/docs/). These establish useful expectations; they do not prove that Sawbyte has surpassed them.

### Differentiation to build and prove

Combine a single workflow across native and graphical clients, a screen/state inspector that generates reliable selectors, record-to-workflow tooling with editable waits and assertions, per-action explanations, reproducible failure traces, and a published mechanics/GUI compatibility ledger. Add a resource profiler showing exactly what each enabled subsystem costs. These are proposed advantages to validate with users, not established uniqueness claims.

Expand Stage 0 with the backend capability contract; Stage 1 with the full-client bridge and resource accounting; Stage 2 with the GUI state machine, JSON workflows and shop fixture; Stage 3 with humanisation verification, manual handoff and cross-backend workflow tests. Begin launcher instance management early, then broaden distribution features alongside validated runtime support. Review competing projects and user pain points at each milestone rather than treating this document as a fixed exhaustive feature list.

## Product decision

Build a lightweight Minecraft Java Edition client and automation platform in Rust. Provide a terminal interface, optional 2D world inspector, stable scripting API, version manager, and tools for experiments. Use a real Minecraft server to run the world initially. Sawbyte controls players and maintains the client-side state needed to act correctly. Reimplementing world generation, redstone, mobs, and every server mechanic would be a separate, much larger project.

The central promise should be **efficient automation through verified player controls**. Being faster, smaller, or more correct than alternatives is a goal to measure, not a claim we can make yet. AI-generated implementation still needs independent behavioural tests.

## Existing projects and what they teach us

| Project | Evidence and relevance | Sawbyte decision |
|---|---|---|
| [Mineflayer](https://github.com/PrismarineJS/mineflayer) | JavaScript ecosystem with movement, inventory, crafting, building and many plugins; its README lists 1.8 through 26.1. | Use as an API/usability and performance comparison. Its breadth is a serious standard to work toward. |
| [Azalea](https://github.com/azalea-rs/azalea) | Rust client libraries, swarm support and pathfinding. README targets 26.2, explicitly excludes simultaneous native multi-version support, and documents missing physics features. | Audit individual crates before deciding on reuse. Keep dependencies behind our own interfaces. |
| [HeadlessMC](https://headlesshq.github.io/headlessmc/launch/) | Runs the real client headlessly by suppressing graphics calls. [Specifics](https://headlesshq.github.io/headlessmc/specifics/) adds client control. | Useful reference backend and CI test harness; graphics suppression alone does not establish minimum memory or CPU usage. |
| [Minecraft Console Client](https://github.com/MCCTeam/Minecraft-Console-Client) | Existing terminal client with a 2.0 rewrite underway. | Study terminal workflows and diagnostics; a terminal interface alone is not differentiation. |
| [Baritone](https://github.com/cabaletta/baritone) | Established pathfinder with version-specific client integrations. | Study navigation behaviours and failure recovery. Review its license before incorporating implementation. |
| [MineRL](https://github.com/minerllabs/minerl) | Minecraft environments with action/observation loops and several API generations. | Borrow the experiment interface concept; keep training optional. |
| [Craftium](https://github.com/mikelma/craftium) | Minecraft-like single/multi-agent research environments. | Useful training-system reference, but changing engines would change the compatibility goal. |
| [Rapidbot](https://github.com/creator1116/rapidbot) | Claims Rust, 26.3 support, vanilla-exact behaviour, and a human input layer. | Closest conceptual competitor found. Claims remain unverified; audit tests, provenance, and coverage before reuse. |

Mineflayer already exposes control states and physics; describing it as simply having no input model would be inaccurate. See its [API](https://github.com/PrismarineJS/mineflayer/blob/master/docs/api.md). Sawbyte must demonstrate stronger invariants rather than rely on that caricature.

### Community findings

- An [admincraft headless Go client discussion](https://www.reddit.com/r/admincraft/comments/1w1hf5j/lightweight_fast_headless_golang_minecraft_client/) describes demand for a small binary, remote API, and automated server testing. Its author reports resource numbers, but these are self-reported and cannot be compared directly to our future workload. The comments also reveal missing command/inventory workflows: small size alone is not enough.
- A [headless-client question](https://www.reddit.com/r/admincraft/comments/z01d18) points to MCC and Mineflayer, showing this is an established category.
- A [Minecraft companion discussion](https://www.reddit.com/r/aigamedev/comments/1wonyp6/the_llm_never_picks_the_actions_how_i_split/) describes separating dialogue from action execution. Design inference: keep planning/model calls outside the timing-critical client.

These are qualitative examples, not a representative user survey or validated performance evidence.

## Language and architecture

Choose Rust for the runtime, protocol adapter, player simulation, scheduler, command line and optional terminal map. It permits explicit memory management without a garbage collector and compact native distribution. It does not automatically make the result faster. Java/Kotlin is the better fit for the small reference-client test mod, because that runs inside Minecraft. Python is the first external SDK for neural-network work; TypeScript follows for general automation. No model, Python environment, or browser should be required to run the basic client.

Use a modular Rust workspace, initially a modest set of modules rather than dozens of microservices:

`launcher → selected runtime/version adapter → world cache + input executor`

`CLI / Python / TypeScript / skills → common API → scheduler → input executor`

`events + observations + traces ← runtime`

The launcher manages downloads, profiles, authentication, runtime selection, updates and logs. The runtime connects and plays. Scripts request actions; only the executor can cause gameplay changes. Start with one process per session for isolation; add a pooled multi-agent worker after profiling demonstrates a benefit.

## What “real inputs” means

All online clients eventually send packets. Sawbyte should take key states, mouse movement, button presses and inventory interactions as input and derive valid game actions from them. This is an internal emulation of the game's input pipeline, not physically moving the user's desktop mouse. A real-client reference backend should inject the same actions into the actual game for comparison.

Enforce the following below every public scripting/skill API:

- Eye position, target raycast, occlusion, collision, and the applicable reach rules.
- Version-, mode-, item-, attribute- and state-dependent interaction rules. Do not hardcode the example “three blocks” as a universal rule.
- Movement acceleration, jumping, stepping, sneaking, sprinting, water, knockback, and eventually vehicles and flight.
- Correct hold/release semantics, breaking duration, attack/use behaviour, selected slots, inventory transaction ordering, and server corrections.
- One owner of an agent's controls at a time, with cancellation releasing held keys.
- No public raw-packet or direct-position escape hatch in the normal gameplay API.

Legal game inputs are distinct from human-like reaction speed or aiming. The default humanisation profile shapes camera movement and timing, can be disabled, and can be replaced by user code. Artificial fatigue and random delays do not define basic correctness. Record the selected policy with each run.

Observation rules also matter: knowing every loaded block is more information than a person sees. Offer explicit `structured` observations and a later `visible` observation profile with occlusion restrictions. A small raycast image is useful but must not be advertised as visually identical to Minecraft.

## Versions and distribution

Use one Sawbyte application version and separate Minecraft compatibility profiles. Example proposed commands:

```text
sawbyte install 26.3
sawbyte connect localhost:25565 --minecraft 26.3
sawbyte run examples/gather.py --profile laboratory
sawbyte inspect session-1
```

A `sawbyte-26.3` profile is a reasonable user-facing label. It is not an automatic converter that turns every Minecraft release into a correct lightweight client. Version adapters include protocol, registries, collision data, physics and interaction differences. Generated data can reduce work; it cannot replace behavioural implementations.

Target Java 26.3 first, which has an [official release page](https://www.minecraft.net/en-us/article/minecraft-java-edition-26-3). Add a second version only after the first is useful, to prove the abstraction. Bedrock, arbitrary mods, historical releases, and snapshots are future compatibility projects. Protocol translation may permit connection without reproducing the target version's physics; label it accordingly.

Publish a compatibility matrix with separate statuses for connecting, movement, interactions, inventories, entities, and advanced mechanics. Unsupported actions return explicit errors. Each release pins tested adapter versions and supports rollback.

## Automation and scripting

Expose a versioned local JSON-RPC API with event streaming over a local socket or authenticated loopback connection. Use generated schemas and SDK types. Begin with Python plus CLI; add TypeScript when the core contract stabilizes. Delay embedded Lua/WASM until a concrete need justifies another runtime.

Three layers:

1. Inputs: press/release keys, move camera, mouse buttons, hotbar and inventory interaction.
2. Tasks: navigate, mine, place, craft, transfer, equip, follow, eat. Tasks compile into inputs and wait for confirmed results.
3. Workflows: gather materials, build a structure, tend a farm, perform server regression tests, run experiments.

Every task needs an ID, deadline, cancellation, progress events, clear failure reason and bounded retries. Commands need sequence numbers and duplicate detection so retries cannot accidentally repeat actions. Navigation needs budgets, stuck detection and replanning; it must share the same movement rules as execution. Inventory changes need server reconciliation.

Provide task composition, event subscriptions, schedules, checkpoints and replayable examples. Treat “automate anything” as extensibility: document capability coverage rather than promise every mechanic on day one. Models can select tasks through an optional integration later; the runtime has no dependency on an AI provider.

## Worlds, training and multiple agents

First support a separately running vanilla server. A user can join the same server with normal Minecraft to watch Sawbyte. A later lab manager can provision server profiles and test worlds on the chosen runtime host. Building on GitHub does not require hosting actual gameplay there.

Add Gymnasium-style reset/step/observation/reward adapters after basic play works. Real-time servers do not become deterministic or fast-forwardable just because the client has a `step` method. Exact synchronized stepping and resets require a controlled server-side harness. Distinguish real-time sessions, recorded replays, and controlled experiments.

Record versions, seed where available, world snapshot identifier, action policy, observation policy, random seeds, input sequence, server updates and outcomes. Replays of an external multiplayer world cannot guarantee identical results without reproducing server state and timing.

Support both agents sharing one world and isolated environments; shared worlds are cheaper but agents influence each other. Bound chunk caches and worker queues. Share immutable registries; only share world data across sessions if visibility and consistency semantics are preserved. Budget model inference separately from game-client work.

## Performance and proof

Skip texture/audio loading, mesh construction and rendering in the native runtime. Keep the optional map at a low refresh rate and off by default in batch runs. Use bounded caches, compact chunk storage, incremental observations, and budgeted pathfinding. Preserve physics/tick correctness before optimizing it.

Benchmark 1, 4, 16 and eventually 64 agents on matched versions, worlds, view distances, actions and hardware. Measure memory, CPU, startup, network traffic, tick deadlines, task success, server corrections and total server-plus-client cost. Compare Mineflayer and a real headless client; add other competitors when they support the same scenario. Publish scripts, raw results and variance. GitHub-hosted performance results are indicative because machines vary; stable dedicated remote hardware is preferable for later claims.

No promised “10× faster” or fixed RAM figure before measurement. Four useful concurrent agents is an initial workload target, with a resource budget set after the baseline.

## Correctness and diagnostics

Create a reference-client test mod and identical scenario fixtures. Feed input sequences into both implementations and compare trajectories, target selection, inventory results, and timing. Server acceptance alone is insufficient: servers may accept actions the normal client would not produce.

Include boundaries around reach, line of sight, corners, slabs/stairs, water, jump/sprint transitions, collision, tool changes, interrupted mining, inventory acknowledgement, death/respawn, dimensions, disconnects and corrections. Expand the suite with every supported mechanic. Define tolerances and explain them; avoid blanket “vanilla exact” claims.

Add structured logs, per-task traces, a tick timeline, an optional 2D inspector, replay export and a diagnostic bundle that removes credentials. Run prolonged sessions for memory growth and fuzz untrusted protocol decoding. Unknown world state should pause dependent actions rather than guess.

## Authentication, extensions and repository hygiene

Use browser/device-code sign-in, protected token storage and redacted logs; never ask scripts to handle account passwords. Multiple simultaneous online players generally need distinct valid sessions/accounts; local controlled test environments should be explicitly separate.

Keep remote control local by default, require authentication for exposed endpoints, and make filesystem/network capabilities explicit for future extensions. Ordinary Python scripts are trusted programs, not a sandbox.

Distribute Sawbyte's own code and artifacts. Do not bundle Minecraft jars/assets or copied decompiled game source. Audit dependency licenses and extracted-data provenance before publication; generation does not itself settle redistribution rights. Minecraft's [EULA](https://www.minecraft.net/en-us/eula) permits independently created tools under its terms and restricts redistribution of its material. Include clear unofficial branding, notices and a dependency inventory. Tentative project license: MIT, subject to the actual dependency audit.

## GitHub-only build and validation workflow

Local work is editing/reviewing repository files. GitHub Actions handles compilation, tests, integration servers, packaging and releases. No local toolchain installation or compilation.

- Pull requests: formatting, static checks, unit/contract tests, protocol fixtures and dependency review.
- Integration: controlled ephemeral server plus Sawbyte; reference-client jobs where supported. Keep authentication secrets away from untrusted pull requests.
- Scheduled/manual: extended mechanics, fuzzing, endurance and benchmark jobs with timeouts and cleanup.
- Releases: Linux x86-64 first; Windows and macOS once tested. Upload binaries, checksums, dependency inventory, compatibility report and installation instructions. Build Python wheels only when native bindings become necessary; the initial remote SDK can remain pure Python.
- Pin toolchains and dependencies, use minimal workflow permissions, cache safely, cancel superseded jobs and retain failure traces. Do not put game binaries or credentials in public artifacts.

## Delivery sequence and exit criteria

| Stage | Deliverable | Exit criterion |
|---|---|---|
| 0: Evidence prototype | GitHub repository, CI, dependency audit, 26.3 reference harness and minimal native connection | CI can reproduce a join/move/stop trace; decide Azalea reuse versus narrow custom adapter from evidence. |
| 1: Useful client | Authentication, chunks, basic input execution, terminal status, reconnect handling | Repeatable walk/jump/target tests match reference within documented tolerances; no stuck controls after cancellation. |
| 2: Useful automation | Inventory, mining, placement, navigation, Python SDK, task scheduler | Gather a small material set and place a simple structure through inputs; failures are explainable. |
| 3: Measured reliability | Traces, regression suite, endurance runs and benchmark reports | Four-agent workload completes reliably with published resource results and no unbounded memory growth. |
| 4: Experiments | Controlled server harness, resets, dataset export, observation policies | Repeatable documented experiment; shared and isolated agent modes clearly distinguished. |
| 5: Compatibility/product polish | Second version adapter, installer/update manager, TypeScript SDK, optional map | Compatibility matrix passes; unsupported mechanics fail clearly; download-and-run experience works on supported platforms. |

Avoid building a universal launcher, plugin marketplace, complete standalone server, or elaborate GUI before proving Stage 2. These can consume the project while leaving its central promise untested.

## Open engineering questions to resolve in Stage 0

Audit how much Azalea can serve the 26.3 adapter without deep forks; inspect Rapidbot's tests and implementation provenance; confirm practical headless reference-client operation for 26.3; establish authentication application requirements; identify distributable fixture/data boundaries; measure server overhead and settle initial observation semantics. These are research gates, not decisions the nontechnical user must make.

The next implementation task is Stage 0. This plan does not create a GitHub repository, publish anything, or claim that a build or test has run.

# Sawbyte: research context and development plan

[The user's full product requirement](docs/VISION.md) is the authority for scope. The example about moving through screens is a metaphor for general programmability; it does not define a dedicated feature. JSON and other named formats are options only. See the [whole-repository audit](docs/AUDIT-2026-11-03.md) and [capability ledger](docs/COVERAGE.md) for the actual state.

## Research context

The earlier research reviewed Mineflayer, Azalea, HeadlessMC, Minecraft Console Client, Baritone, MineRL and Craftium as comparison points. Their existence establishes a substantial field, not that Sawbyte has surpassed them. The prior plan also captured community discussions about lightweight automation, client workflows and training interfaces. Those posts and project pages are useful leads, not independent performance or correctness evidence. Revisit current project versions and licenses before making comparative claims or selecting dependencies.

The research suggested retaining an independent native Rust player, treating the normal client as a useful optional reference/integration, building for real server state, and separating external planning from timing-critical execution. These remain sound architectural directions. It also proposed an illustrative purchase workflow; that was incorrectly treated as a feature target. That interpretation has been corrected throughout the project documentation.

## Product and technical direction

- Native zero-graphics Rust Minecraft Java client is primary. Optional graphical integration remains desired.
- First expose comprehensive, validated actions, observations, events, task lifecycle and outcome reporting. This general layer must support behaviours the project authors have never prewritten.
- Apply real version/state-dependent game rules across every entry point. Humanised defaults are configurable; direct mode does not bypass game rules.
- Use live synchronized client state and prediction as a real client requires. Do not invent production screens, world changes, confirmations or success. Keep test doubles inside tests.
- Choose APIs, transports, scripting and data formats from measured performance, usability and coverage. Python/TypeScript/MCP/Gymnasium integrations are optional adapters.
- Build a genuinely useful launcher that installs verified runtime/version data, runs and supervises processes, manages accounts safely and eventually offers a graphical interface.
- Measure full workloads. Small fixtures are useful baselines, not memory targets or proof of superiority.
- Build, test and package only through GitHub Actions.

## Evidence and completion gates

| Gate | Required result |
|---|---|
| Reliable client foundation | Bounded protocol/state handling, lifecycle transitions, complete supported-version data, ordinary input/prediction compared with the normal client on varied scenarios. |
| General automation core | Every advertised operation appears in a capability inventory, uses common rule validation, exposes sufficient state/events and reports completed/failed/uncertain outcomes. Real waits, cancellation, recovery and extension access work. |
| Player-operation coverage | Independently tested movement/modes, targeting, interaction timing, inventory/components/cursor, screen families, text, chat, lifecycle and world changes for each claimed adapter. No special illustrative workflow stands in for primitives. |
| Training and concurrency | Defined tick/action/observation semantics, real reset or explicitly limited real-time episodes, reproducible seeding where supported, useful observations, cancellation, and measured multi-client scaling. |
| Launcher and platforms | Clean install, data/profile provisioning, account flow, mode/version dispatch, supervised processes, updates/recovery and packages tested on supported OSes. |
| Efficiency and reliability | Matched reproducible CPU/memory/power/scale measurements, sustained sessions, failure recovery and comparisons with honest workload boundaries. |
| Integrations | MCP protocol checks, built and installed SDK packages, optional graphical client behaviour and complete documentation generated from tested capability lists. |

## Work order & 5-Batch Execution Blueprint

1. **Batch 1: Commit & Verify Groundwork on GitHub Actions (`A01`, `A06`, `A08`, `A10`–`A13`, `A15`, `A18`, `A21`, `A22`, `A37`, `A50`)**
   - Stage, commit, and push the Step 1 runtime repairs already in the working tree:
     - Authoritative server `state_id` preservation (`ScreenState::set_slot_authoritative`, `ScreenState::apply_server_content` in `crates/sawbyte-core/src/screen.rs`).
     - NBT list and small array parsing (`Reader::tag` in `native/src/wire.rs`).
     - 26.3 protocol 777 packet table alignment, structured item & component decoding (`decode_slot_item`: `max_damage`, `damage`, `custom_name`, `item_name`, `lore`), cursor item decoding, 24 standard 26.3 menu mappings, and removal of hardcoded chat strings in `native/src/inventory.rs`.
     - Removal of `shop_fixture_mode` and all synthetic screen/widget/event transitions, closed-loop feedback-driven 3D A* navigation (`step_live_navigation`), reach/line-of-sight/break-tick enforcement on `interact.mine`/`place`/`attack`, and full voxel/inventory/screen/entity JSON observations in `native/src/main.rs`.
     - Closed-loop navigation arrival assertion (`<= 0.65` blocks from `(3.5, 2.5)`) in `tools/player_lab.py`.
   - Verify both `Build and verify` and `Real player milestone` pass green on GitHub Actions.
2. **Batch 2: Control Framing, Standards-Compliant MCP JSON-RPC & Safe Launcher Persistence (`A27`–`A30`, `A38`, `A40`–`A42`, `A48`)**
   - Escape/unescape multiline TCP control replies across `native/src/main.rs`, `crates/sawbyte-core/src/mcp.rs`, and `sdks/python/sawbyte/__init__.py`, and treat EOF (`0` bytes read) as an explicit connection-closed error (`A27`).
   - Add `inputSchema` to all MCP `tools/list` definitions, support `params.arguments`, add `prompts/get`, reject unknown tools/resources with explicit errors, and remove mutating default fallback scripts (`A28`, `A29`, `A30`).
   - Validate usernames (`1..=16` ASCII alphanumeric/`_`) and 32-hex UUIDs in `crates/sawbyte-core/src/launcher.rs` to prevent `../` lockfile path traversal, enforce disk lock checks, and verify `sawbyte-26.3` native mode before spawning (`A38`, `A40`, `A41`, `A42`).
   - Fix `benchmark_transports` byte count in `crates/sawbyte-core/src/mcp.rs` (`A48`).
3. **Batch 3: Dynamic Palette Promotion, Conservative Raycast Occlusion, Entity Motion & Respawn Cleanup (`A09`, `A14`, `A16`, `A19`, `A20`)**
   - Compute dynamic direct palette bit width in `Palette::set` (`native/src/world.rs`) and test >16 palette entries and state IDs above 15 bits (`A19`).
   - Treat unloaded chunks or unknown block states as solid unit-cube occluders in `raycast_entity` (`native/src/raycast.rs`) so entity targeting never clips through unloaded walls (`A09`).
   - Implement `apply_move_entity_pos` and `apply_set_entity_motion` in `native/src/inventory.rs` and apply server knockback velocity to the local player in `native/src/main.rs` (`A14`, `A18`).
   - Reset chunks, entities, open screens, and active navigation on `respawn` in `native/src/main.rs` (`A20`).
   - Populate `stairs` in `tools/player_lab.py` from the generated 26.3 `blocks.json` report (`A16`).
4. **Batch 4: Admissible Overflow-Safe Pathfinding, Task Cancellation & Bounded SDK/Gym Lifecycle (`A23`, `A26`, `A33`, `A35`, `A36`)**
   - Make `find_path` and `heuristic` in `crates/sawbyte-core/src/tasks.rs` overflow-safe (`i64` diffs and `checked_add`) and admissible (`(dx + dz) * 10 + dy * 6`) (`A23`).
   - Add `task.cancel` in `native/src/main.rs` to stop active navigation and release all scheduled buttons/events (`A26`).
   - Add timeout kill fallback in `SawbyteMcpClient.close`, death termination check (`health <= 0.0`), and configurable `reward_fn` in `SawbyteEnv` (`sdks/python/sawbyte/__init__.py`) (`A33`, `A35`, `A36`).
5. **Batch 5: Documentation & Coverage Ledger Sync (`docs/COVERAGE.md`, `docs/HANDOFF.md`)**
   - Update `docs/COVERAGE.md` and `docs/HANDOFF.md` with the verified commit hash, CI run links, and repaired vs. still-open audit items.

This order is a dependency guide, not a limit on the amount of work per pass. Update status only when evidence meets a gate. All builds, tests, and benchmarks run exclusively in GitHub Actions.

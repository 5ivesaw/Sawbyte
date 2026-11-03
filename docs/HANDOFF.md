# Sawbyte — Agent Handoff & Complete Technical Reference

Last updated: 3 November 2026 (`main` branch, Stage 0 complete).

Read this document first if you are picking up development in a new session. It contains the non-negotiable project rules, repository map, verified Minecraft Java 26.3 protocol/physics invariants, CI workflow mechanics, and the exact roadmap so you do not have to re-research them.

---

## 1. Non-Negotiable Rules (`AGENTS.md`)

1. **NO LOCAL BUILDS OR TOOLCHAINS**:
   - **Never** run `cargo`, `rustc`, `gradle`, Java compilers, local Minecraft servers, or resource-heavy test harnesses on the user's machine unless explicitly requested.
   - Local work is strictly editing/reviewing files and using lightweight `git` / `gh` commands to push and inspect GitHub Actions runs (`https://github.com/5ivesaw/Sawbyte`).
2. **Main Product vs. Reference Bridge**:
   - Sawbyte's primary product is the **standalone native Rust client** (`native/`, `sawbyte-play`) and shared automation framework (`crates/*`).
   - It is **not** a renamed Azalea client (zero Azalea dependencies) and **not** primarily a Minecraft mod.
   - The normal-client Fabric 26.3 mod (`bridge/`) is the optional full-client integration and parity-reference backend.
3. **Complete Programmability (Governing Requirement)**:
   - Every player operation in a supported version/backend must have an automation route with composable low-level primitives beneath helpers.
   - User examples (e.g., complex multi-screen shop GUIs, JSON workflows) illustrate expressiveness requirements; they never limit scope or force a poor technical format choice.
4. **Humanisation by Default + Freedom**:
   - Ship configurable humanisation enabled by default (`Humanised`), with an explicit off switch (`Direct`) and custom policy hooks (`InputPolicy`).
5. **Strict Engineering Honesty**:
   - Only claim what is measured and verified in CI. Document scope limits and unsupported mechanics explicitly in `docs/COVERAGE.md` and `docs/PLAYER.md`.
6. **Branding**:
   - Use `assets/sawbyte.png` (supplied by the user) for repository and platform artifact branding (`.ico` embedded in Windows `.exe`s, `.icns` in macOS `.app` bundles, `.desktop` + `.png` in Linux packages).
   - Note: an untracked `"sawbyte icon.png"` copy exists in the local working tree root; leave it untouched and uncommitted.

---

## 2. Repository Structure & File Map

```text
Sawbyte/
├── AGENTS.md                        # Mandatory workflow and product rules
├── PLAN.md                          # Full product architecture, research, and 6-stage delivery plan
├── README.md                        # Public overview, status badges, CLI usage, and verified metrics
├── Cargo.toml / Cargo.lock          # Foundation Rust workspace (zero external dependencies)
├── rust-toolchain.toml              # Pinned Rust 1.85.0 (rustfmt, clippy)
├── assets/sawbyte.png               # Canonical user-supplied Sawbyte icon
├── crates/
│   ├── sawbyte-core/                # Zero-dep core: Action, Button, Capability, InputFrame,
│   │   ├── src/lib.rs               #   InputPolicy (Humanised, Direct), Backend trait, Scheduler
│   │   └── src/tape.rs              #   Binary .sbif ("SBIF\x01") input tape reader/writer & 160-tick fixture
│   ├── sawbyte-protocol/            # Zero-dep TCP status/ping client, VarInt, bounded framing
│   │   └── src/lib.rs
│   └── sawbyte-cli/                 # Foundation `sawbyte` CLI (demo, tape, status, capabilities)
│       ├── build.rs                 # Windows .ico resource embedding via packaging/windows_resource.rs
│       ├── src/main.rs
│       └── tests/cli.rs
├── native/                          # Separate Rust workspace for standalone 26.3 player (`sawbyte-play`)
│   ├── Cargo.toml / Cargo.lock      # Uses standard crypto/compression/HTTP/NBT crates (no Azalea)
│   ├── rust-toolchain.toml          # Pinned Rust 1.85.0
│   ├── build.rs                     # Windows .ico resource embedding
│   └── src/
│       ├── main.rs                  # 26.3 TCP loop, login/config/play state machine, trace CSV output
│       ├── auth.rs                  # Offline UUIDv3 ("OfflinePlayer:<name>"), AES-128-CFB8, RSA/SHA-1
│       │                            #   serverId hash, Mojang session join, MS device-code OAuth flow
│       ├── wire.rs                  # Bounded packet framing (2 MiB limit), zlib threshold, network NBT
│       ├── world.rs                 # 26.3 chunk section palette decoding, heightmap skip, 256-chunk bound
│       └── player.rs                # 26.3 f32 look/movement vectors, f64 AABB collision, gravity/drag
├── bridge/                          # Optional Fabric 26.3 mod (`sawbyte-bridge`) for normal client
│   ├── build.gradle                 # Gradle 9.4.1, Fabric Loom 1.15-SNAPSHOT, Java 25, Minecraft 26.3
│   ├── gradle.properties
│   ├── settings.gradle
│   └── src/main/
│       ├── java/dev/fivesaw/sawbyte/SawbyteBridge.java
│       └── resources/fabric.mod.json
├── packaging/
│   ├── sawbyte.desktop              # Linux desktop entry template
│   └── windows_resource.rs          # Pure-Rust COFF .res generator + linker arg for Windows .ico
├── tools/
│   ├── build_icons.py               # Generates sawbyte.ico and sawbyte.icns from assets/sawbyte.png
│   └── player_lab.py                # CI parity lab: downloads 26.3 server, extracts block IDs, runs
│                                    #   server + sawbyte-play + HeadlessMC Fabric client, compares CSVs
├── docs/
│   ├── COVERAGE.md                  # Honest capability ledger
│   ├── DECISIONS.md                 # Architectural decisions and Stage 0 -> Stage 1 transition notes
│   ├── PLAYER.md                    # Standalone player usage, scope limits, and comparison rules
│   └── HANDOFF.md                   # This file
└── .github/
    ├── actions/branding/action.yml  # Composite action generating icons on Linux/Windows/macOS
    └── workflows/
        ├── ci.yml                   # "Build and verify" workflow (foundation workspace)
        └── player.yml               # "Real player milestone" workflow (native + bridge + 26.3 lab)
```

---

## 3. What Has Been Built & Verified (Stage 0 Complete)

Both GitHub Actions workflows are **green** on `main`:

1. **`Build and verify` (`.github/workflows/ci.yml`)**:
   - Builds, tests, clippy-checks, and fmt-checks the zero-dependency foundation workspace (`sawbyte-core`, `sawbyte-protocol`, `sawbyte-cli`) on `ubuntu-24.04`, `windows-2022`, and `macos-14` with `--locked --offline`.
   - Produces downloadable OS bundles (`sawbyte-linux-x64`, `sawbyte-windows-x64`, `sawbyte-macos-arm64`) with platform icons and `SHA256SUMS`.
2. **`Real player milestone` (`.github/workflows/player.yml`)**:
   - Builds and unit-tests `native/` (`sawbyte-play`) across Linux x86-64, Windows x86-64, and macOS ARM64.
   - Builds `bridge/` (`sawbyte-bridge-0.1.0.jar`) with Java 25 and Gradle 9.4.1 against Minecraft 26.3.
   - Runs `tools/player_lab.py` on Ubuntu 24.04 with `xvfb-run`:
     - Downloads the official Minecraft `26.3` server jar from Mojang's version manifest and runs `--reports` to generate `blocks.json` (`block-profile.json`).
     - Launches an offline, loopback-only flat-world 26.3 server (`127.0.0.1:25565`).
     - Runs `sawbyte-play` (`SawbyteNative`) through the 160-tick humanised movement fixture (`movement.sbif`: 20 ticks wait, 40 ticks walk forward, 40 ticks walk + 90° humanised yaw turn at 12°/tick cap, 1 tick jump + walk, 29 ticks walk, 30 ticks stop).
     - Launches the real Minecraft 26.3 client with Fabric Loader `0.18.4` and `sawbyte-bridge` under HeadlessMC `2.7.0` (`-Xms512M -Xmx2G`) to execute the exact same `movement.sbif`.
     - Compares all 160 ticks in `native.csv` vs `reference.csv`.
   - **Verified comparison results** (`player-comparison-evidence` artifact, Run `41570156922`):
     - `max_position_error_blocks`: `3.4117737307480316e-07` (gate tolerance: `0.001` blocks)
     - `rms_position_error_blocks`: `2.2887988105447855e-07`
     - `ground_state_mismatches`: `0`
     - `sampled_peak_rss_kib` (`sawbyte-play`): `4,380 KiB` (~4.28 MiB)

---

## 4. Critical Minecraft Java 26.3 Technical Findings (Do Not Lose)

Minecraft Java **26.3** (protocol version **`777`**, Java 25) has several wire-protocol and physics specifics that were verified against the 26.3 server and client during Stage 0:

### 4.1 Protocol `777` Packet IDs (`native/src/main.rs`)

- **Handshaking (Serverbound)**:
  - `0x00` (`0`): Intention (`protocol = 777`, `host`, `port`, `next_state = 1` for Status or `2` for Login).
- **Login**:
  - Serverbound: `0` = Hello (`name`, `uuid`), `1` = Key (encryption response), `2` = Custom Query Answer, `3` = Login Acknowledged, `4` = Cookie Response.
  - Clientbound: `0` = Disconnect, `1` = Hello (encryption request), `2` = Login Finished (`uuid`, `name`, properties), `3` = Login Compression (`threshold`), `4` = Custom Query, `5` = Cookie Request.
- **Configuration**:
  - Clientbound:
    - `0` = Cookie Request -> reply Serverbound `1` (Cookie Response).
    - `2` = Disconnect (network NBT text component).
    - `3` = Finish Configuration -> reply Serverbound `3` (Finish Configuration) and transition to Play.
    - `4` = Keep Alive (`i64`) -> reply Serverbound `4` (`i64`).
    - `5` = Ping (`i32`) -> reply Serverbound `5` (`i32`).
    - `7` = Registry Data (`registry_id: String`, entries `Vec<(entry_id: String, Optional<NBT>)>`). Used to extract `minecraft:dimension_type` (`min_y`, `height`).
    - `9` = Resource Pack Push (`uuid: 16 bytes`, ...) -> reply Serverbound `6` (`uuid` + VarInt `1` = Declined/Accepted response code as appropriate; `1` in our offline lab).
    - `15` = Select Known Packs -> reply Serverbound `7` with `[0]` (0 known packs, forcing the server to send full NBT in packet `7` Registry Data so we don't hardcode registry assumptions).
    - `20` = Code of Conduct.
  - Serverbound:
    - `0` = Client Information (`locale`, `view_distance`, `chat_visibility`, `chat_colors`, `model_customisation`, `main_hand`, `text_filtering`, `allow_listing`, `particle_status`).
- **Play**:
  - Clientbound:
    - `8` = Block Update (`ClientboundBlockUpdatePacket`).
    - `11` = Chunk Batch Finished -> reply Serverbound `11` (`f32` desired chunks/tick = `20.0`).
    - `21` = Cookie Request -> reply Serverbound `21`.
    - `32` = Disconnect (network NBT component).
    - `38` = Forget Level Chunk (`z: i32`, `x: i32` — note `z` is read before `x`!).
    - `45` = Keep Alive (`u64`) -> reply Serverbound `28` (`u64`).
    - `46` = Level Chunk With Light (`x: i32`, `z: i32`, heightmaps binary format, byte-array `data` containing sections `non_empty_blocks: u16` + block state `PalettedContainer` + biome `PalettedContainer`).
    - `50` = Login (`player_id: i32`, `hardcore: bool`, `levels: Vec<String>`, `max_players: VarInt`, `chunk_radius: VarInt`, `simulation_distance: VarInt`, `reduced_debug_info: bool`, `show_death_screen: bool`, `do_limited_crafting: bool`, `dimension_type: VarInt`, `dimension: String`, `seed: u64`, `game_mode: u8`, ...).
    - `62` = Ping (`i32`) -> reply Serverbound `45` (`i32`).
    - `69` = Player Combat Kill (death).
    - `73` = Player Position (`teleport_id: VarInt`, `pos: [f64; 3]`, `velocity: [f64; 3]`, `yaw: f32`, `pitch: f32`, `relatives_flags: i32`).
    - `82` = Resource Pack Push -> reply Serverbound `50`.
    - `84` = Respawn.
    - `86` = Section Blocks Update (`ClientboundSectionBlocksUpdatePacket`).
    - `120` = Start Configuration.
  - Serverbound:
    - `0` = Accept Teleportation (`ServerboundAcceptTeleportationPacket`): **In 26.3, this packet contains `(teleport_id: VarInt, x: f64, y: f64, z: f64, yaw: f32, pitch: f32)`**—not just `teleport_id`!
    - `11` = Chunk Batch Received (`f32`).
    - `13` = Client Tick End (`ServerboundClientTickEndPacket`, empty payload).
    - `28` = Keep Alive (`u64`).
    - `31` = Move Player PosRot (`x: f64`, `y: f64`, `z: f64`, `yaw: f32`, `pitch: f32`, `flags: u8` where bit 0 = `on_ground`, bit 1 = `horizontal_collision`).
    - `43` = Player Input (`ServerboundPlayerInputPacket`, `u8` bitmask: Forward=1, Backward=2, Left=4, Right=8, Jump=16, Sneak=32, Sprint=64).
    - `44` = Player Loaded (`ServerboundPlayerLoadedPacket`, empty payload — sent once when spawn chunk is loaded).
    - `45` = Pong (`i32`).
    - `50` = Resource Pack Response.

### 4.2 The 26.3 One-Position-Packet-Per-Tick Rule (Crucial!)

In Minecraft 26.3:
- `ServerGamePacketListenerImpl.handleAcceptTeleportPacket` and `handleMovePlayer` both enforce a per-tick position packet guard (`receivedPositionThisTick`), which is only reset when the client sends `ServerboundClientTickEndPacket` (`packet 13`).
- Because `ServerboundAcceptTeleportationPacket` (`packet 0`) in 26.3 already carries `(id, x, y, z, yRot, xRot)` and updates the player's server-side position, the client **must NOT** send an immediate `ServerboundMovePlayerPosRot` (`packet 31`) inside the packet `73` handler. Doing so causes the next position packet before `ClientTickEnd` to disconnect the client with `multiplayer.disconnect.invalid_player_movement` (`Invalid move player packet received`).
- Instead, packet `73` sends **only** `packet 0` (`AcceptTeleportation`), and regular tick movement sends `packet 31` (`MovePlayerPosRot`) followed by `packet 13` (`ClientTickEnd`) once per 50 ms client tick.

### 4.3 Chunk Section Palette Packing (`native/src/world.rs`)

- In 26.3 `LevelChunkWithLight` (`packet 46`), heightmaps are serialized as a `VarInt` count of `(type: VarInt, longs: Vec<u64>)` (not NBT!).
- Each chunk section has `non_empty_blocks: u16`, then the block-state `PalettedContainer`:
  - `bits == 0`: single-value palette (`VarInt` state ID, no data array).
  - `1..=8`: indirect palette (`bits` clamped to `4..=8` for block states), followed by `VarInt` palette length and `VarInt` state IDs, then a tightly calculated number of 64-bit integers (`ceil(4096 / floor(64 / bits))`) with **no VarInt long-array length prefix** in 26.3!
  - `> 8`: direct global palette (`bits = ceil(log2(state_count))`), followed directly by the packed `u64` words.
- Biomes follow the same rule for `64` entries (`bits == 0`, `1..=3` indirect, `> 3` direct).

### 4.4 Player Physics & Precision (`native/src/player.rs` & `bridge/SawbyteBridge.java`)

- Look angles (`yaw`, `pitch`) update in `f32`.
- In `SawbyteBridge.java`, input frame yaw/pitch deltas are divided by `0.15` before calling `player.turn(yaw / 0.15, pitch / 0.15)` because `Entity.turn(double, double)` multiplies by `0.15f`.
- Forward/strafe impulse for ordinary walking is `0.98f32`, normalized if length > 1.0, scaled by `0.1f32` on ground (`0.6` slipperiness * `0.91` = `0.546` friction; `0.1 * (0.21600002 / 0.6^3) = 0.1f32`) or `0.02f32` in air.
- Trig uses Minecraft's 65536-entry `f32` sine table (`Mth.sin` / `Mth.cos` lookup table reproduced in `player.rs:sin()`).
- Jump sets `velocity[1] = max(0.42f32 as f64, velocity[1])`.
- Gravity subtracts `0.08` from `Y` velocity, then multiplies `Y` by `0.98`, and `X`/`Z` by `0.546` (ground) or `0.91` (air).
- Velocity components with `norm_sq < 9e-6` are zeroed at the start of the tick.

### 4.5 HeadlessMC 26.3 JVM Heap Quirk (`tools/player_lab.py`)

- Mojang's `26.3.json` manifest injects `-Xms2G` into default JVM args. When launching via HeadlessMC `--jvm`, always specify **both** `-Xms` and `-Xmx` (e.g., `-Xms512M -Xmx2G`), otherwise passing `-Xmx1G` alone crashes JVM initialization (`Initial heap size set to a larger value than the maximum heap size`).

---

## 5. How to Trigger & Inspect Builds/Tests (GitHub Actions Only)

After editing files locally, commit and push to GitHub, then monitor or download artifacts via `gh`:

```bash
# Check recent workflow runs
gh run list --limit 6

# Watch a run until completion
gh run watch <run-id> --exit-status

# Inspect failed job logs if needed
gh run view <run-id> --log-failed

# Download comparison evidence artifact to a scratch directory
gh run download <run-id> -n player-comparison-evidence -D /tmp/sawbyte-evidence
```

---

## 6. Complete Master Checklist of Everything Remaining (Stage 1 to Full Product)

See `PLAN.md`, `docs/DECISIONS.md`, and `docs/COVERAGE.md` for full context. Every item below must be completed and verified in GitHub Actions for the project to be finished and usable:

### Phase 1: Complete World, Physics & Movement Engine (`native/` & `bridge/`)
- [ ] **1.1 Dynamic World Updates**: Handle `BlockUpdate` (`packet 8`), `SectionBlocksUpdate` (`packet 86`), chunk unload/reload, and multi-block state changes in loaded chunks without stopping the session.
- [ ] **1.2 Full Block Collision Shapes**: Extract and support all 26.3 block collision shapes from official data/reports (slabs, stairs, fences, walls, panes, doors, trapdoors, gates, carpets, snow layers, beds, chests, anvils, honey/slime/soul sand, berry bushes, cobwebs, powdered snow).
- [ ] **1.3 Extended Movement Modes**:
  - [ ] Sprinting (impulse, FOV/attribute modifier, block/collision sprint-cancel rules, food/hunger sprint gates).
  - [ ] Sneaking / crouching (eye height, bounding-box height change, `0.3` speed multiplier, edge-clamping so the player cannot walk off block edges).
  - [ ] Stepping (`maxUpStep` = `0.6` blocks across stairs/slabs/low ledges).
  - [ ] Climbables (ladders, vines, scaffolding, twisting/weeping vines).
  - [ ] Fluids & Swimming (water flow vectors, buoyancy, swim sprint, lava movement & drag, bubble columns).
  - [ ] Gliding (Elytra pitch/yaw lift & drag math) and Creative/Spectator flight.
  - [ ] Vehicles & Mounts (boats, minecarts, horses, pigs, striders — passenger position & steering packets).
- [ ] **1.4 Server Velocity, Attributes & Status Effects**:
  - [ ] Handle `SetEntityMotion` (knockback/velocity), `Explode` knockback, and mid-session `PlayerPosition` corrections (including relative flags and rotated velocity flag `256`).
  - [ ] Track living entity attributes (`generic.movement_speed`, `generic.step_height`, `generic.jump_strength`, `player.block_interaction_range`, `player.entity_interaction_range`, `player.block_break_speed`).
  - [ ] Track status effects affecting motion/mining (`Speed`, `Slowness`, `Jump Boost`, `Slow Falling`, `Levitation`, `Dolphin's Grace`, `Haste`, `Mining Fatigue`, `Blindness`).
- [ ] **1.5 Entity Tracking & World State**:
  - [ ] Track nearby entities (`AddEntity`, `RemoveEntities`, `MoveEntityPos/Rot`, `TeleportEntity`, `SetEntityData`, `SetEquipment`) with bounding boxes for collision and targeting.
  - [ ] Track health, hunger, saturation, oxygen/air supply, experience, game mode transitions (`GameEvent`), respawn (`Respawn`), and dimension changes.

### Phase 2: Complete Low-Level Player Inputs, Targeting & Interactions
- [ ] **2.1 Ordered Sub-Tick Event Contract (`sawbyte-core`)**:
  - [ ] Extend the scheduler beyond continuous tick buttons (`InputFrame`) with an ordered discrete event queue per tick (mouse click edges, hotbar slot selection `0..8`, offhand swap `F`, drop item `Q`/`Ctrl+Q`, screen interactions, text edits).
- [ ] **2.2 Eye Raycasting & Reach Enforcement**:
  - [ ] Implement exact 26.3 eye-to-target voxel raycasting (`ClipContext` fluid/outline modes) and entity AABB raycasting.
  - [ ] Enforce version-, attribute-, and game-mode-accurate block & entity reach limits and occlusion rules before emitting interaction packets.
- [ ] **2.3 Block Mining & Placement**:
  - [ ] Multi-tick block breaking progress (`StartDestroyBlock`, `StopDestroyBlock`, `AbortDestroyBlock`) with tool speed, enchantments (`Efficiency`, `Aqua Affinity`), underwater/airborne penalties, and sequence numbers (`ack`).
  - [ ] Block placement & block use (`UseItemOn`) with accurate hit-vector coordinates, inside-block flag, hand selection, and client-side placement prediction/reconciliation.
- [ ] **2.4 Item Use & Entity Combat/Interaction**:
  - [ ] Item use (`UseItem` — eating, drinking, bows, crossbows, shields, ender pearls, buckets, fishing rods) and `ReleaseUseItem`.
  - [ ] Entity attack (`Interact` attack mode, attack cooldown reset, critical-hit conditions, sweep rules) and entity interact (`Interact` / `InteractAt` for villagers, mounts, item frames, armor stands).

### Phase 3: Full Inventory, GUI, Screen & Text Programmability
- [ ] **3.1 Complete Inventory & Container State Machine**:
  - [ ] Track player inventory (hotbar, main storage, armor, offhand, crafting grid) and open container windows (`OpenScreen`, `ContainerSetContent`, `ContainerSetSlot`, `ContainerSetData`, `ContainerClose`, `SetCursorItem`).
  - [ ] Parse 26.3 item stacks and structured data components (item ID, count, custom name, lore lines, enchantments, durability, custom model data, bundle contents).
  - [ ] Enforce `state_id` (revision) tracking and implement all `ContainerClick` modes (`PICKUP`, `QUICK_MOVE` / shift-click, `SWAP` hotbar key, `CLONE` middle-click, `THROW`, `QUICK_CRAFT` drag-split, `PICKUP_ALL` double-click).
- [ ] **3.2 Specialised Vanilla Screens & Widgets**:
  - [ ] Recipe book (`PlaceRecipe`), Crafting Table, Furnace/Smoker/Blast Furnace, Anvil (`RenameItem` + slot clicks), Enchanting Table (`ContainerButtonClick`), Brewing Stand, Villager Trading (`SelectTrade`), Stonecutter, Loom, Cartography Table, Grindstone, Smithing Table, Beacon (`SetBeacon`), Lectern (`TakeBook` / page turn).
- [ ] **3.3 Text Input, Chat, Commands, Signs, Books & Server Dialogs**:
  - [ ] Chat message sending (`Chat`), signed/unsigned chat handling, command execution (`ChatCommand` / `ChatCommandSigned`), tab-completion (`CommandSuggestions`), Sign editing (`SignUpdate`), Book editing (`EditBook`), and 26.3 server dialogs / custom payload screens (`CustomClickAction` / `CodeOfConduct` / resource-pack prompts).
- [ ] **3.4 GUI Selector Engine & Complex Workflow Execution**:
  - [ ] Query/selector primitives by item ID, display name, lore substring/regex, data components, slot index/region, widget label, and custom predicates (failing explicitly on ambiguous matches unless disambiguated).
  - [ ] End-to-end acceptance test for complex plugin GUIs (e.g., multi-screen server shop: wait for screen title -> find category item -> wait for next screen revision -> select item -> confirm -> enter amount in anvil/sign/chat prompt -> click pay with `retry: never` and post-timeout reconciliation).
- [ ] **3.5 Full-Client Bridge Parity (`bridge/`)**:
  - [ ] Upgrade `sawbyte-bridge` from a tape replay mod to a full interactive backend supporting the exact same screen, inventory, observation, and input API inside graphical Minecraft, with manual player override/handoff (pausing automation and releasing controls when the user touches mouse/keyboard).

### Phase 4: Humanisation Engine & Policy Framework
- [ ] **4.1 Camera & Aiming Policies**:
  - [ ] Seedable, configurable camera trajectories (acceleration/deceleration profiles, target acquisition overshoot/settle, micro-adjustments) alongside `Direct` mode and custom `InputPolicy` trait hooks.
- [ ] **4.2 GUI Cursor, Click Pacing & Typing Cadence**:
  - [ ] Virtual cursor travel timing between slots/buttons in screens, bounded reaction delays after screen opens/revisions, click cadence limits, and humanised typing speed for text fields/chat/signs.

### Phase 5: External Control API, SDKs, Tasks & Diagnostics
- [ ] **5.1 Transport Benchmark & Local RPC Server**:
  - [ ] Benchmark candidate local IPC/RPC transports (Unix domain socket / named pipe / loopback TCP with framed binary vs JSON-RPC) for latency, memory, and throughput.
  - [ ] Implement the versioned local control plane with authentication, capability discovery, event streaming, and per-task cancellation/deadlines.
- [ ] **5.2 High-Level Composable Tasks (Built Strictly on Low-Level Inputs)**:
  - [ ] Budgeted A* / jump-aware pathfinder and movement follower using the exact same input executor (no teleporting or packet spoofing).
  - [ ] Composable tasks for `navigate`, `mine`, `place`, `craft`, `smelt`, `trade`, `equip`, `transfer`, `eat`, `follow`, and `interact_screen` with progress events, stuck detection, and explainable failure diagnostics.
- [ ] **5.3 Python & TypeScript SDKs + Workflow Runner**:
  - [ ] Ship the zero-bloat Python SDK (first) and TypeScript SDK (second), plus a workflow runner supporting variables, loops, conditionals, waits, timeouts, and registered code extensions.
- [ ] **5.4 Inspector, Recorder & Resource Profiler**:
  - [ ] Record-to-workflow tool (recording manual play in the full-client bridge into editable workflows with selectors and waits).
  - [ ] Screen/state inspector, optional lightweight 2D terminal/top-down map, replay trace exporter, and per-subsystem memory/CPU profiler.

### Phase 6: Authentication, Multi-Agent Harness, Benchmarks & Standalone Launcher
- [ ] **6.1 Complete Authentication & Account Store**:
  - [ ] Live validation of Microsoft device-code login, automatic token refresh before expiry, OS keychain / Windows DPAPI / Unix `0600` credential protection, redacted logs, and session lockfile protection against duplicate concurrent logins on the same account.
- [ ] **6.2 Multi-Agent Swarm, Endurance & Competitor Benchmarks**:
  - [ ] Run 1, 4, 16, and 64 concurrent agents in shared and isolated worlds; verify zero unbounded memory growth over prolonged sessions and fuzz untrusted protocol decoders.
  - [ ] Benchmark memory, CPU, startup time, and task completion against Mineflayer and HeadlessMC on matched scenarios.
- [ ] **6.3 Training & Experiment Adapters**:
  - [ ] Gymnasium-style `reset` / `step` / `observation` (`structured` vs `visible` occlusion-aware) / `reward` adapters with a controlled server-side tick/reset harness.
- [ ] **6.4 Standalone Launcher & Multi-Version Profiles**:
  - [ ] Build the Sawbyte launcher (CLI + UI) managing isolated instances, account profiles, integrity-checked downloads, version adapters (e.g., `sawbyte-26.3` plus a second Minecraft version to prove the version-adapter boundary), mod-loader integration for full-client mode, rollback, and platform installers/release assets in GitHub Actions.

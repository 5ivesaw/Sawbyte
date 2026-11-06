---
name: sawbyte
description: >-
  Complete operational manual, Model Context Protocol (MCP) tool reference,
  and long-horizon autonomous gameplay guide for AI agents interacting with
  the Sawbyte zero-graphics Minecraft 26.3 native client and automation engine.
---

# Sawbyte AI Agent & MCP Automation Guide

This skill guides AI agents (such as Claude, Gemini, Cursor, Antigravity, or OpenAI models) on how to autonomously connect to, control, inspect, and beat Minecraft Java Edition 26.3 using **Sawbyte**.

Sawbyte is an independent, zero-graphics native Rust Minecraft client (`sawbyte-play`) and deterministic automation runtime (`sawbyte`). It operates with zero GPU, zero JVM, and consumes only **~8.2 MiB RSS**, making it ideal for continuous, multi-hour and multi-day long-horizon autonomous tasks.

---

## 1. Connection & Operational Modes

Sawbyte offers two primary communication channels for AI agents:
1. **Model Context Protocol (MCP)**: Standard JSON-RPC 2.0 over `stdio`, providing structured tool calls and real-time state resources.
2. **Live TCP Control Socket**: High-speed, non-blocking socket interface for raw command execution and Gymnasium stepping (default port: `25580`).

### Starting the Headless Engine with MCP

```bash
# Launch Sawbyte in standard MCP stdio mode
sawbyte mcp --stdio

# Or start a live headless bot listening on control socket 127.0.0.1:25580
sawbyte-play live 127.0.0.1:25565 --offline AgentName block-profile.json --listen 127.0.0.1:25580
```

---

## 2. Model Context Protocol (MCP) Tool Reference

The Sawbyte MCP server exposes 10 native tools:

### `agent_milestone_logger`
**Purpose**: Synchronous, zero-latency persistence of long-horizon task progress and death tracking directly to disk (`milestones.log` and `milestones.jsonl`).
- **Arguments**:
  - `action`: `"save"` (record new milestone), `"list"` (retrieve historical milestones), or `"deaths"` (retrieve death records).
  - `name` *(optional)*: Unique milestone name (e.g. `"Crafted Wooden Pickaxe"`, `"Entered Nether"`).
  - `notes` *(optional)*: Freeform context (e.g. `"Biome: Plains, found iron vein at y=16"`).
- **Execution Effect**: Automatically captures sub-tick player coordinates $(X, Y, Z)$, yaw, pitch, health, food level, experience level, total attempts, and a serialized snapshot of all inventory slots. Writes synchronously without introducing delays to the game loop.

### `player_input`
**Purpose**: Emit fine-grained sub-tick inputs and smooth humanised or direct camera rotations.
- **Arguments**:
  - `keys`: Array of directional inputs: `["forward", "back", "left", "right", "jump", "sneak", "sprint"]`.
  - `yaw`: Delta horizontal rotation in degrees (positive = right, negative = left).
  - `pitch`: Delta vertical rotation in degrees (clamped to $-90^\circ$ to $+90^\circ$).
  - `policy`: `"humanised"` (default, smoothed angular acceleration) or `"direct"` (instant snap).

### `player_interact`
**Purpose**: Perform mathematically verified block raycasting, mining, placement, item interaction, and entity attacks.
- **Arguments**:
  - `action`: `"mine"`, `"place"`, `"use_item"`, `"attack"`, `"raycast"`.
  - `target_pos` *(optional)*: `[x, y, z]` integer block coordinates.
  - `entity_id` *(optional)*: Target entity network ID for combat.
  - `slot` *(optional)*: Hotbar slot index (`0..=8`).
- **Rules Enforced**: Verifies line-of-sight raycasts from eye position, respects max reach (configurable via `player.reach`), and prevents placing blocks intersecting the player's bounding box.

### `gui_screen_automation`
**Purpose**: Inspect server-opened containers (chests, crafting tables, furnaces, villagers) and execute atomic slot transactions.
- **Arguments**:
  - `action`: `"inspect"` (return open window slot metadata), `"click"` (execute slot click), `"close"` (close screen).
  - `slot`: Container slot index (`0..=N`).
  - `button`: `0` (left click), `1` (right click).
  - `mode`: `"pickup"`, `"quick_move"` (shift-click), `"swap"` (number keys), `"clone"` (middle click), `"throw"` (drop key).
- **State Verification**: Enforces server `state_id` transaction verification. Discards stale clicks if the server revision changes before confirmation.

### `task_and_navigation`
**Purpose**: Compute and execute 3D A* pathfinding over loaded chunk voxels.
- **Arguments**:
  - `goal`: `[x, y, z]` target coordinates.
  - `sprint`: Boolean indicating whether to sprint-jump over obstacles.
  - `preempt`: If true, aborts any currently executing movement path.

### `world_inspector_and_profiler`
**Purpose**: Inspect surrounding voxels, block palettes, and entity positions without rendering graphics.
- **Arguments**:
  - `bounds`: `[min_x, min_y, min_z, max_x, max_y, max_z]`.
  - `format`: `"json"` (full block state IDs and bounding boxes) or `"ascii"` (top-down slice map).

### `gym_step_and_observe`
**Purpose**: Execute sequential tick stepping and retrieve un-truncated observation vectors for RL models.
- **Output**: Returns exact player coordinates, velocities, on-ground status, health, hunger, loaded entity AABBs, and surrounding $7 \times 7 \times 7$ voxel patch.

### `launcher_control`
**Purpose**: Manage client instances, snapshot backups, and launch native/potato/bridge processes.

### `client_menu_and_settings`
**Purpose**: Configure persistent options (`fov`, `renderDistance`, keybindings) on disk.

### `code_execute`
**Purpose**: Execute multi-line deterministic SawbyteScript sequences.

---

## 3. Real-Time MCP Resources

Agents can subscribe to or query live engine resources:
- `sawbyte://agent/milestones`: Live JSON stream of autonomous attempt history, death records, and milestone markers.
- `sawbyte://world/state`: High-frequency player vitals, coordinates, and loaded entity table.
- `sawbyte://gui/current_screen`: Live container slot layout and title.

---

## 4. Live TCP Scripting Command Reference

When connected to `sawbyte-play` via TCP socket (`127.0.0.1:25580`):

| Command Line | Description | Return Format |
|:---|:---|:---|
| `inspect.status` | Query position, yaw/pitch, health, food, tick counter | `pos=(X,Y,Z) yaw=Y pitch=P health=H food=F tick=T` |
| `inspect.chat` | Retrieve recent server chat messages | JSON array of chat strings |
| `inspect.events` | Query discrete subtick event logs | Array of emitted event identifiers |
| `inspect.milestones` | List all recorded milestones in current session | Formatted list or JSON array |
| `inspect.deaths` | List all death events, attempt numbers, and causes | Formatted list of death records |
| `inspect.frame [w] [h] [mode]` | Render pure-Rust software 3D raymarched frame | `frame:data:image/bmp;base64,...` |
| `inspect.visualizer_state` | Query telemetry and nearby entity distances | JSON telemetry and waypoint object |
| `milestone.save <name> [notes...]` | Record milestone to `milestones.log` and `jsonl` | `milestone_recorded name="..." attempt=A ...` |
| `inventory.find <keyword>` | Search inventory for matching item by name or lore | `{"slot": S, "id": "...", "count": C}` |
| `player.hotbar <slot>` | Switch held hotbar slot (`0..=8`) | `selected_hotbar: S` |
| `player.reach <blocks>` | Adjust max reach boundary (default: 4.5 blocks) | `player.reach set to B` |
| `player.look <yaw> <pitch>` | Adjust camera look angles | `player.look yaw=Y pitch=P` |
| `interact.use_item` | Right-click held item (consume food, activate item) | `interact.use_item hand=MainHand` |
| `gui.inspect` | Inspect open container slots | `{"title": "...", "slots": [...]}` |
| `gui.click <slot> [button] [mode]` | Click container slot | `gui.click slot=S state_id=I` |
| `task.navigate <x> <y> <z>` | Begin 3D A* navigation to coordinates | `live 3D A* path active (N waypoints)` |
| `task.cancel` | Abort active pathfinding and clear inputs | `cancelled active navigation` |

---

## 5. Autonomous Long-Horizon Agent Playbook ("Beat Minecraft")

To instruct an agent to complete long-horizon goals (such as obtaining diamond or defeating the Ender Dragon) without human intervention or cheating:

### Phase 1: World Entry & Baseline Checkpoint
1. Query `inspect.status` to confirm spawn position and ground state.
2. Call `milestone.save "Spawn Point Reached" "World loaded successfully"`.
3. If on a cracked server with an authentication challenge, inspect `inspect.chat` and emit `chat.send "/register <pass> <pass>"`.

### Phase 2: Tech-Tree Progression (Wood -> Stone -> Iron)
1. **Harvest Wood**:
   - Query `world_inspector_and_profiler` for nearby logs (`minecraft:oak_log`).
   - Navigate to tree with `task.navigate <x> <y> <z>`.
   - Aim raycast at log and execute `interact.mine <x> <y> <z>`.
   - Call `milestone.save "Harvested Log" "Collected 4 oak logs"`.
2. **Crafting Grid Operations**:
   - Convert logs to planks via 2x2 grid (`gui_screen_automation` or `craft.recipe`).
   - Assemble Crafting Table and Wooden Pickaxe.
   - Record milestone: `milestone.save "Wooden Pickaxe Crafted"`.
3. **Stone & Iron Age**:
   - Mine 3 Cobblestone voxels to craft a Stone Pickaxe.
   - Locate and mine raw iron ore voxels (`y=16..64`).
   - Craft a Smelting Furnace (`8 cobblestone`) and smelt raw iron into iron ingots.
   - Record milestone: `milestone.save "Iron Tools Acquired"`.

### Phase 3: Survival, Threat Avoidance & Health Management
1. **Hunger Maintenance**:
   - Monitor `inspect.status`. When `food < 18`, search inventory for food with `inventory.find "cooked"` or `"apple"`.
   - Select slot (`player.hotbar <slot>`) and hold `interact.use_item` for 32 ticks.
2. **Hostile Mob Combat & Shield Parrying**:
   - When hostile entities appear in `inspect.visualizer_state`, check distance.
   - If distance $< 3.5$ blocks: aim camera with `player.look` and execute `player_interact action="attack"`.
   - If projectile detected: raise offhand shield (`interact.use_item`) to mitigate damage.
3. **Night Survival**:
   - If health $< 10.0$ or night approaches, navigate to an enclosed voxel enclave using `task.navigate` or dig into a 3-block shelter.

### Phase 4: Death Recovery & Attempt Handling
- If lethal damage occurs, the client automatically increments `death_count` and `attempt` counter, logs the exact cause and coordinates to `milestones.log`, and respawns.
- The agent loop must read `inspect.deaths`, evaluate the failure cause, adjust pathfinding heuristics, and continue without restarting the overall process.

---

## 6. Golden Rules for AI Agents

1. **Zero Artificial Sleeps**: Never introduce arbitrary client-side sleep delays. Always synchronize using `wait.ticks <N>` or wait for server packet acknowledgments.
2. **Verify Container Revisions**: Never blind-click inventory slots. Always inspect `gui.inspect` to verify server `state_id` before transaction emission.
3. **Line of Sight Required**: You cannot mine blocks or attack mobs through walls unless unobstructed line-of-sight raycasts confirm visual intersection.
4. **Always Log Milestones**: Call `milestone.save` at every notable achievement. This ensures complete audit trails and guarantees resume capability if disconnected.

---

## 7. "Seeing the Game with Words v2" & `SVPE-1` Encoding Reference

Sawbyte v0.5.0 includes a 10-section text-and-code visual perception engine (`inspect.sensory` / `see`) and the **Sawbyte Voxel Perception Encoding (`SVPE-1`)** format (`docs/SENSORY_ENCODING.md`):

### Live Perception & Memory Commands
| Command | Purpose |
|:---|:---|
| `inspect.sensory` / `see` | Full 10-section report: Vitals, 360° Radar, 46-Slot Inventory, Open GUI, Hologram NPCs, 24x11 First-Person Retinal Viewport, `SVPE-1` RLE3D & HD25D streams, Through-Block X-Ray, Humanised ESP, 64m Horizon, Small-Map Inspector & Persistent Memory |
| `inspect.sensory.json` | Complete JSON perception snapshot with `svpe1_rle3d`, `svpe1_hd25d`, `svpe1_depth24x11`, and entity `visibility_tier` |
| `inspect.sensory.delta` / `svpe.delta` | Compact `SVPE1:DELTA` differential tick frame (~92% smaller for 20 TPS loops) |
| `inspect.viewport` / `see.viewport` | 24x11 first-person retinal ASCII camera frustum (`FOV 72°x44°`) + `SVPE1:DEPTH24x11` depth buffer |
| `inspect.svpe [rx ry rz]` | Encode local 3D volume as lossless `SVPE1:RLE3D` stream |
| `inspect.svpe.decode <str>` | Decode & verify an `SVPE1:RLE3D` stream |
| `inspect.svpe.height [radius]` | Encode 2.5D surface elevation deltas & hazards as `SVPE1:HD25D` (`=` level, `A..Z` `+1..+26`, `a..z` `-1..-26`, `~` water, `!` lava, `_` void) |
| `inspect.xray` / `see.xray` | 16m through-block penetrating raycasts along Crosshair, UnderFeet, N/S/E/W revealing wall thickness, hidden rooms, and underground lava |
| `inspect.long_range` / `see.horizon` | 64m 8-sector horizon scanner detecting void gaps, distant islands, bridges, and peaks |
| `inspect.small_map` / `see.small_map` | Automatic full-arena mapper for small maps (BedWars, SkyWars, Duels, SkyBlock) |
| `humanisation.perception [on\|off\|status]` | Toggle or inspect `HumanisedPerceptionPolicy` (locks vanilla 3.0m/4.5m reach and downgrades occluded through-wall ESP to `0.18` confidence supplementary sector hints) |
| `memory.status` / `memory.save` / `memory.load` / `memory.landmarks` / `memory.mark` | Persistent cross-run world memory (`SVMEM-1`, saved in `~/.sawbyte/memory/<world_id>.svmem.json`) |

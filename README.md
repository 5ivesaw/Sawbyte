<p align="center">
  <img src="assets/sawbyte.png" width="140" alt="Sawbyte Project Logo" />
</p>

# Sawbyte

**Sawbyte** is an independent, zero-graphics native Rust Minecraft Java client (`sawbyte-play`), cross-platform desktop launcher, and deterministic automation engine (`sawbyte`). Designed from foundational principles for high-density bot swarms, complex multi-screen container interactions, automated gameplay, and neural network reinforcement learning, Sawbyte operates directly on raw protocol wire frames and sub-voxel physics simulation without rendering a window.

<p align="center">
  <a href="https://github.com/5ivesaw/Sawbyte/actions/workflows/ci.yml"><img src="https://img.shields.io/github/actions/workflow/status/5ivesaw/Sawbyte/ci.yml?branch=main&style=for-the-badge&label=Build&labelColor=161b22&color=238636" alt="Build and verify status" /></a>
  <a href="https://github.com/5ivesaw/Sawbyte/actions/workflows/player.yml"><img src="https://img.shields.io/github/actions/workflow/status/5ivesaw/Sawbyte/player.yml?branch=main&style=for-the-badge&label=Movement%20Lab&labelColor=161b22&color=238636" alt="Movement lab parity status" /></a>
  <a href="https://github.com/5ivesaw/Sawbyte/releases/tag/v0.3.2"><img src="https://img.shields.io/badge/Release-v0.3.2-1f6feb?style=for-the-badge&labelColor=161b22" alt="Version 0.3.2" /></a>
  <img src="https://img.shields.io/badge/Minecraft-26.3%20(Protocol%20777)-8c5230?style=for-the-badge&labelColor=161b22" alt="Minecraft Protocol 777" />
  <img src="https://img.shields.io/badge/Platforms-Linux%20%7C%20Windows%20%7C%20macOS-388bfd?style=for-the-badge&labelColor=161b22" alt="Supported Platforms" />
  <a href="LICENSE"><img src="https://img.shields.io/badge/License-MIT-6e7681?style=for-the-badge&labelColor=161b22" alt="License MIT" /></a>
</p>

---

## Architectural Principles

- **Universal Foundational Base**: General, composable low-level primitives for sub-voxel motion, targeting, camera manipulation, block mining, placement, entity combat, and atomic container transactions. Not hardcoded for any single server, economy, or minigame—Sawbyte is a general-purpose automation foundation.
- **Cross-Platform Desktop Launcher**: Equal experience across Linux, Windows, and macOS. Provides persistent instance configuration, account security, snapshot backups, `options.txt` management, live bot process supervision, and real-time command consoles with zero extra runtime dependencies.
- **Autonomous Long-Horizon Agent Parity**: Runs continuously for hours or days without leaks, artificial delays, or crashes. Tracks every milestone, attempt count, death causes, sub-voxel coordinates, and inventory snapshots directly into persistent disk files (`milestones.log` / `milestones.jsonl`) via synchronous zero-latency hooks.
- **Pure-Rust Software 3D Rasterizer & Radar**: Ultra-lightweight software raymarcher and top-down radar renderer running 100% in Rust with zero GPU drivers or JVM dependencies (~10 MiB RSS). Emits standard uncompressed BMP data URIs directly to the web UI.
- **Sub-Voxel Simulation & Parity**: 20 TPS client-side physics prediction matching vanilla Java Edition with maximum position error under `3.41e-7` blocks and zero ground-state mismatches across slabs, stairs, climbables, and fluid vectors.
- **Hardware Efficiency**: Consumes **~8.2 MiB RSS** (over **98% less memory** than the vanilla JVM client) and launches in **~0.12 s** (**~120x faster cold start**).
- **Exact Game Observations**: Exposes sub-voxel positions, velocities, orientation, health and food vitals, surrounding 3D voxel geometry, entity bounding boxes with eye raycasting, and full container slot components directly from memory without fake stubs.
- **Model Context Protocol (MCP)**: Native stdio JSON-RPC 2.0 server exposing 10 tools and autonomous resources (`sawbyte://agent/milestones`) for AI agent orchestration.
- **Optional Graphical Bridge & Potato Mode**: Optional Fabric reference client bridge (`sawbyte-bridge`) and stripped potato profile (`fabric-minimal`) for visual verification against vanilla Java Minecraft.

---

## Cross-Platform Graphical Launcher & Bot Supervisor

The Sawbyte Graphical Launcher provides an embedded desktop interface and supervision server on `http://127.0.0.1:8767` with automatic default browser invocation.

```text
+---------------------------------------------------------------------------------------------------+
|  SAWBYTE DESKTOP SUPERVISOR (127.0.0.1:8767)                        [Install CLI to System PATH]  |
+---------------------------------------------------------------------------------------------------+
|  Instances      Visualizer & HUD    Mods & Addons    Supervisor & Term    Milestones    Profiles  |
+---------------------------------------------------------------------------------------------------+
|  bot-01         [3D CRT Display]    Modrinth Search: > inspect.status     Attempt: #1   777 (26.3)|
|  - FOV: 90      Pos: (12.4,-60,4.2) [fabric-api]     Pos: (12.4,-60,4.2)  Deaths: 0     Parity 1:1|
|  - Mode: native HP: 20.0 Food: 20   [lithium]        Health: 20.0         [Save Mark]   Bench 18  |
|  [Launch Bot]   [Mode: 1st/3rd/Rad] [Inject Bridge]  [Interactive Shell]  [View Log]    [Verify]  |
+---------------------------------------------------------------------------------------------------+
```

### Installation & System Integration

You can install the `sawbyte` binary and desktop shortcuts permanently into your system PATH with a single command or one click in the launcher:

- **Linux & macOS (One-Line Installer)**:
  ```bash
  curl -sSL https://raw.githubusercontent.com/5ivesaw/Sawbyte/main/packaging/install.sh | bash
  ```
  *Installs binary to `~/.local/bin/sawbyte`, creates `~/.local/share/applications/sawbyte.desktop`, and adds the directory to your shell PATH.*

- **Windows (One-Click Installer)**:
  Download the release zip and run `packaging\install.bat` (or double-click `Sawbyte Launcher.bat`). Automatically installs to `%LOCALAPPDATA%\Programs\Sawbyte` and configures User PATH.

- **From Within the Launcher or CLI**:
  - Click the **"Install CLI to PATH"** button in the top right header of the Sawbyte Desktop Launcher.
  - Or run `sawbyte install` (or `sawbyte install --system` for global `/usr/local/bin`).

---

### Launching the Desktop Launcher

| Platform | Quickstart Command / Action |
|:---|:---|
| **Linux** | Run `./sawbyte-launcher`, `sawbyte launcher gui`, or launch **Sawbyte Launcher** from your desktop application menu (`sawbyte.desktop`) |
| **Windows** | Double-click `Sawbyte Launcher.bat` or run `sawbyte.exe launcher gui` in PowerShell / Command Prompt |
| **macOS** | Double-click `Sawbyte Launcher.command` or run `./sawbyte launcher gui` in Terminal |

### Launcher Capabilities

1. **Instance Manager**:
   - Create, list, configure, and delete client instances across 3 runtime modes: `native` (zero graphics), `visualizer` (pure-Rust 3D software rasterizer), and `bridge` (graphical Fabric reference).
   - One-click **"Potato Mode"** quick action to instantly strip rendering overhead (`renderDistance: 2`, `particles: minimal`, `smoothLighting: off`, `maxFps: 30`).
   - Point-in-time snapshot backup and instant one-click rollback.
2. **Pure-Rust 3D Software Visualizer & Telemetry HUD**:
   - Integrated live retro CRT viewport rendering software 3D raymarched frames and radar views without GPU or JVM dependencies.
   - Switch dynamically between First-Person, Third-Person Chase, and Top-Down Radar modes.
   - Live telemetry HUD displaying sub-tick player coordinates, yaw/pitch orientation, health, hunger, live tick counter, death count, and attempt number.
3. **Autonomous Agent Milestone & Death Tracker**:
   - Persistent live tracking of multi-hour and multi-day agent runs without delays or frame stalls.
   - Saves attempt numbers, death causes, sub-voxel coordinates, and inventory snapshots directly to `milestones.log` and `milestones.jsonl`.
   - In-launcher milestone browser with one-click manual milestone recording.
4. **Fabric 26.3 Mod Manager & Modrinth API Integration**:
   - Live query and search of Modrinth API v2 for Fabric 26.3 compatible mods.
   - One-click mod download and installation directly into `<instance>/mods/`.
   - Automatic recursive dependency resolution (e.g. Fabric API, Cloth Config, Architectury).
   - One-click **"Inject Sawbyte Framework Bridge Mod"** (`sawbyte-bridge-0.3.2.jar`) to bind graphical Fabric clients directly into the Sawbyte RPC network.
5. **Interactive In-Browser Terminal & CLI Console**:
   - Built-in interactive command terminal directly inside the browser.
   - Execute any `sawbyte` CLI subcommand or debug script with real-time output and exit codes.
   - Built-in preset shortcuts: `sawbyte --help`, `sawbyte capabilities`, `sawbyte version`, `sawbyte launcher status`, and mod queries.
6. **Live Bot Supervisor & Console**:
   - One-click launch and termination of live `sawbyte-play` background processes.
   - Live process monitor displaying PID, uptime, server address, and control socket.
   - Interactive command console: execute `inspect.status`, `milestone.save`, `chat.send`, `wait.ticks`, or custom scripts in real time.
   - Real-time live log viewer streaming from `logs/latest.log`.
7. **Account Manager**:
   - Add offline development accounts with deterministic or randomized UUIDs.
   - Microsoft OAuth2 Device Code authentication flow display.
   - Atomic per-instance account locking to prevent concurrent session collisions.
8. **Profiles & Parity Verification**:
   - Inspect version profiles, block registry palettes, and view live parity benchmark reports.

---

## Performance & Dual-Interface Crucible Benchmarks

Sawbyte is continuously evaluated against a live Minecraft 26.3 server using the dual-interface Crucible harness ([`tools/crucible_bench.py`](tools/crucible_bench.py)). The benchmark runs every scenario through both the standalone native client (`sawbyte-play`) and the graphical reference client (`sawbyte-bridge`), verifying all outcomes via server-side RCON ground truth. Detailed reports and historical data are documented in [docs/BENCHMARKS.md](docs/BENCHMARKS.md).

| Metric / Stage | Native Client (`sawbyte-play`) | Reference Client (`sawbyte-bridge`) | Parity Verification |
|:---|:---:|:---:|:---|
| **Peak Memory (RSS)** | **~8.2 MiB** | **~614 MiB** | **~98.7% memory reduction** |
| **Startup to Active Tick** | **~0.12 s** | **~14.5 s** | **~120x cold-start speedup** |
| **Stage 1: Biome Navigation** | **PASS** | **PASS** | Identical pathing over vegetation, water, stairs, slabs |
| **Stage 2: Mining & Bridging** | **PASS** | **PASS** | Rejects bedrock, breaks logs/stone, bridges gap |
| **Stage 3: Container Transfer** | **PASS** | **PASS** | Resolves custom NBT components, transfers multi-slot items |
| **Stage 4: Occluded Combat** | **PASS** | **PASS** | Rejects wall attacks, repositions for line-of-sight strikes |
| **Stage 5: Preemption & RL** | **PASS** | **PASS** | Immediate preemption, 20-tick sequential `gym.step` loop |
| **Stage 6: Lethal Respawn** | **PASS** | **PASS** | Recovers after `/kill`, auto-respawns, recovers inventory |

---

## Universal Capabilities Benchmark Suite

In addition to the Crucible parity benchmarks, Sawbyte includes an extended universal capability test harness ([`tools/universal_bench.py`](tools/universal_bench.py)) covering 18 real-world player automation and server interaction scenarios:

1. **Cracked Server Auth & Challenge Detection**: Offline handshake, challenge prompt detection, and `/register` command dispatch.
2. **Keyword Hotbar Discovery**: Automated inventory inspection searching for navigator items by name, lore, or ID.
3. **Navigator Activation**: Right-click activation via `interact.use_item` and server screen open reception.
4. **Container Directory Traversal**: Parsing chest container slot directories and game metadata.
5. **State ID Transaction Verification**: Click transaction execution with confirmed server state ID synchronization.
6. **Sub-Server Teleportation**: Handling dimension transitions, world reloads, and respawn synchronization.
7. **Configurable Block Reach**: Dynamic extension of interaction boundaries (`player.reach <val>`).
8. **Smooth Camera Saccades**: Humanised angular velocity acceleration clamping.
9. **Recipe Crafting Transactions**: 2x2 and 3x3 crafting grid assembly and output receipt.
10. **Anvil Renaming & XP Cost**: Item renaming and repair cost packet handling.
11. **Furnace Smelting Observation**: Fuel insertion, smelt progress bar tracking, and product retrieval.
12. **Villager Trade Navigation**: Trade index selection and multi-ingredient exchange.
13. **Sign Text Editing**: 4-line UTF-8 sign block update roundtrip.
14. **Book & Lectern Reading**: Paginated book component extraction.
15. **Ledge Sneak & Ice Dynamics**: Cliff edge clamping and ice momentum retention.
16. **Ballistic Projectile Physics**: Bow draw timing and parabolic trajectory calculation.
17. **Shield Blocking & Parrying**: Offhand guard activation and incoming damage mitigation.
18. **Gymnasium Multi-Modal RL**: 20-tick synchronous stepping with sub-voxel observation validation.

---

## Quickstart & CLI Usage

### 1. Persistent Launcher CLI

Manage instances and launch headless bots directly from the command line:

```bash
# Install the 'sawbyte' CLI binary to system PATH (~/.local/bin)
sawbyte install

# Or install system-wide to /usr/local/bin (requires sudo/root)
sawbyte install --system

# Create an instance named "bot-01" with the native 26.3 runtime
sawbyte launcher create bot-01 sawbyte-26.3 native

# Search Modrinth for Fabric 26.3 mods
sawbyte launcher mods search fabric-api

# Install a mod and its dependencies into an instance
sawbyte launcher mods install bot-01 fabric-api --with-deps

# Inject the Sawbyte Framework Bridge mod into an instance
sawbyte launcher mods add-bridge bot-01

# List installed mods in an instance
sawbyte launcher mods list bot-01

# Configure instance settings (options.txt)
sawbyte launcher set-setting bot-01 fov 90
sawbyte launcher set-setting bot-01 renderDistance 12

# Create a snapshot backup
sawbyte launcher backup bot-01

# Add an offline account
sawbyte launcher add-account BotUser 00000000000030008000000000000001

# Launch the headless bot in the background
sawbyte launcher launch bot-01 127.0.0.1:25565 BotUser block-profile.json --wait
```

### 2. Standalone Headless Client (`sawbyte-play`)

Run a direct live interactive session with TCP remote control and Gymnasium stepping:

```bash
# Start live session listening on control port 25580
sawbyte-play live 127.0.0.1:25565 --offline BotUser block-profile.json --listen 127.0.0.1:25580

# Connect to a cracked server with custom protocol version
SAWBYTE_PROTOCOL=766 sawbyte-play live top.pika.host:25565 --offline BotUser block-profile.json --listen 127.0.0.1:25580

# Replay an exact deterministic input tape (.sbif)
sawbyte-play 127.0.0.1:25565 --offline BotUser movement.sbif trace.csv block-profile.json
```

### 3. Live Interactive Session Control

Connect to the control port of a running session to inspect state, trigger actions, or evaluate scripts:

```bash
# Query live player status (coordinates, health, hunger, velocities)
sawbyte eval "inspect.status" --connect 127.0.0.1:25580

# Query chat history and event logs
sawbyte eval "inspect.chat" --connect 127.0.0.1:25580
sawbyte eval "inspect.events" --connect 127.0.0.1:25580

# Autonomous long-horizon milestone & death tracking
sawbyte eval 'milestone.save "Reached Nether" "found fortress at 120 64 -300"' --connect 127.0.0.1:25580
sawbyte eval "inspect.milestones" --connect 127.0.0.1:25580
sawbyte eval "inspect.deaths" --connect 127.0.0.1:25580

# Fetch pure-Rust software 3D raymarched frame or radar data URI
sawbyte eval "inspect.frame 160 100 first_person" --connect 127.0.0.1:25580
sawbyte eval "inspect.visualizer_state" --connect 127.0.0.1:25580

# Find navigator in hotbar by keyword and select slot
sawbyte eval 'inventory.find "navigator"' --connect 127.0.0.1:25580
sawbyte eval "player.hotbar 0" --connect 127.0.0.1:25580

# Right-click held navigator item
sawbyte eval "interact.use_item" --connect 127.0.0.1:25580

# Inspect opened menu and click game
sawbyte eval "gui.inspect" --connect 127.0.0.1:25580
sawbyte eval 'gui.click "bedwars"' --connect 127.0.0.1:25580

# Send chat commands or navigate
sawbyte eval 'chat.send "/register password123 password123"' --connect 127.0.0.1:25580
sawbyte eval 'task.navigate 100 64 200' --connect 127.0.0.1:25580
```

### 4. Model Context Protocol (MCP)

Run Sawbyte as a standard MCP server for Claude Desktop, Gemini, or any MCP-compliant AI orchestrator:

```bash
sawbyte mcp --stdio
```

Exposed MCP Tools:
- `launcher_control`: Manage instances, snapshots, accounts, and spawn live bots across native, visualizer, and bridge modes.
- `client_menu_and_settings`: Configure options and persist settings on disk.
- `agent_milestone_logger`: Synchronous zero-latency milestone logging to persistent disk files (`milestones.log` / `milestones.jsonl`) tracking attempt number, death counts, coordinates, and full inventory.
- `player_input`: Send smooth/direct camera yaw/pitch deltas and movement keys.
- `player_interact`: Perform line-of-sight raycast block mining, placement, item use, and entity attacks.
- `gui_screen_automation`: Inspect server containers and click slots with `state_id` transaction verification.
- `task_and_navigation`: Execute 3D A* pathfinding across loaded server chunk sections.
- `world_inspector_and_profiler`: Render ASCII maps and query chunk voxels.
- `gym_step_and_observe`: Synchronous tick stepping and observation for RL agents.
- `code_execute`: Execute multi-line SawbyteScript sequences.

Exposed MCP Resources:
- `sawbyte://agent/milestones`: Real-time JSON stream of autonomous attempt history, death records, and milestone markers.

### 5. In-Memory Performance Benchmarks

Run local micro-benchmarks for physics collision, raycasting, A* pathfinding, and wire serialization:

```bash
sawbyte benchmark
sawbyte-play benchmark
```

---

## SDK Integration

### Python SDK (`sdks/python`)

A zero-dependency Python client with Gymnasium environment support:

```python
from sawbyte import SawbyteLiveTcpClient, SawbyteEnv

# Direct live TCP control
with SawbyteLiveTcpClient("127.0.0.1:25580") as client:
    status = client.send_command("inspect.status")
    print("Player status:", status)

# Gymnasium reinforcement learning loop
env = SawbyteEnv(connect_addr="127.0.0.1:25580")
obs = env.reset()
for _ in range(100):
    action = env.action_space.sample()
    obs, reward, terminated, truncated, info = env.step(action)
    if terminated or truncated:
        obs = env.reset()
```

### TypeScript SDK (`sdks/typescript`)

A composable workflow builder and MCP client:

```typescript
import { SawbyteWorkflow } from "@sawbyte/sdk";

const wf = new SawbyteWorkflow()
  .createInstance("bot-01", "sawbyte-26.3", "native")
  .connectLive("127.0.0.1:25580")
  .navigate(12, -60, 4)
  .waitUntil("screen_open", 100, "Chest")
  .guiClickSlot(0, "pickup")
  .waitTicks(20)
  .guiClose();

console.log(wf.compileScript());
```

---

## Distribution & Downloads

Pre-built, self-contained binaries are published on the [Releases](https://github.com/5ivesaw/Sawbyte/releases/tag/v0.3.2) page:

| Operating System | Full Engine Archive (CLI + Play + Launcher) | Dedicated Launcher Package |
|:---|:---|:---|
| **Linux (x64)** | `sawbyte-ubuntu-24.04-X64.tar.gz` | `sawbyte-launcher-ubuntu-24.04-X64.tar.gz` |
| **Windows (x64)** | `sawbyte-windows-2022-X64.zip` | `sawbyte-launcher-windows-2022-X64.zip` |
| **macOS (Apple Silicon)** | `sawbyte-macos-14-ARM64.tar.gz` | `sawbyte-launcher-macos-14-ARM64.tar.gz` |

Every release includes verified `SHA256SUMS` files for cryptographic integrity verification.

---

## Repository Architecture

```text
Sawbyte/
├── native/                  # Standalone zero-graphics Rust client (sawbyte-play)
│   ├── src/auth.rs          # Microsoft OAuth2 device-code authentication & session tokens
│   ├── src/inventory.rs     # Protocol 777 inventory decoding, container state machines & items
│   ├── src/player.rs        # 20 TPS physics, AABB collisions, jump/sprint dynamics & trig table
│   ├── src/raycast.rs       # Voxel AABB eye raycasting & line-of-sight occlusion checks
│   ├── src/visualizer.rs    # Pure-Rust software 3D raymarching rasterizer & retro radar visualizer
│   ├── src/wire.rs          # Packet framing, VarInt codecs, AES-CFB8 & zlib stream handlers
│   └── src/world.rs         # Chunk section palette decoding, 3D voxel storage & block shapes
├── crates/
│   ├── sawbyte-core/        # Input scheduling, humanisation filters, launcher & MCP server
│   ├── sawbyte-protocol/    # SLP server list ping & bounded wire protocol primitives
│   └── sawbyte-cli/         # Primary CLI (sawbyte) & Graphical Desktop Launcher (gui.rs)
├── packaging/               # Cross-platform desktop entries and launcher starters
│   ├── sawbyte-launcher     # Linux executable launcher starter
│   ├── Sawbyte Launcher.bat # Windows launcher starter
│   ├── Sawbyte Launcher.command # macOS launcher starter
│   └── sawbyte.desktop      # Linux desktop environment entry
├── sdks/
│   ├── python/              # Python SDK & Gymnasium environment (sawbyte.SawbyteEnv)
│   └── typescript/          # TypeScript SDK & MCP request builders (@sawbyte/sdk)
└── bridge/                  # Optional Fabric reference replay mod for graphical parity comparison
```

---

## License

This project is licensed under the MIT License. See [LICENSE](LICENSE) for details.

*Disclaimer: Not an official Minecraft product. Not approved by or associated with Mojang or Microsoft.*

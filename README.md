<p align="center">
  <img src="assets/sawbyte.png" width="148" alt="Sawbyte Logo" />
</p>

# Sawbyte

**Sawbyte** is an independent, zero-graphics native Rust Minecraft Java client (`sawbyte-play`), cross-platform desktop launcher, and deterministic automation runtime (`sawbyte`). Designed from the ground up for high-density bot swarms, complex multi-screen container interactions, automated gameplay, and neural network reinforcement learning (RL), it operates directly on raw protocol wire frames and sub-voxel physics simulation without rendering a window.

<p align="center">
  <a href="https://github.com/5ivesaw/Sawbyte/actions/workflows/ci.yml"><img src="https://img.shields.io/github/actions/workflow/status/5ivesaw/Sawbyte/ci.yml?branch=main&style=for-the-badge&label=Build&labelColor=1a1a1a&color=2B6E5B" alt="Build and verify" /></a>
  <a href="https://github.com/5ivesaw/Sawbyte/actions/workflows/player.yml"><img src="https://img.shields.io/github/actions/workflow/status/5ivesaw/Sawbyte/player.yml?branch=main&style=for-the-badge&label=Movement%20Lab&labelColor=1a1a1a&color=2ea043" alt="Movement lab" /></a>
  <img src="https://img.shields.io/badge/Release-v0.3.0-58a6ff?style=for-the-badge&labelColor=1a1a1a" alt="Version 0.3.0" />
  <img src="https://img.shields.io/badge/Minecraft-26.3%20(Protocol%20777)-8C5230?style=for-the-badge&labelColor=1a1a1a" alt="Protocol 777" />
  <img src="https://img.shields.io/badge/Platforms-Linux%20%7C%20Windows%20%7C%20macOS-36528C?style=for-the-badge&labelColor=1a1a1a" alt="Platforms" />
  <img src="https://img.shields.io/badge/License-MIT-gray?style=for-the-badge&labelColor=1a1a1a" alt="License" />
</p>

---

## Key Highlights

- **Universal Foundational Base**: General, composable low-level primitives for movement, targeting, continuous camera controls, block mining, placement, entity combat, and atomic container transactions. Not hardcoded for any single server or minigame—use Sawbyte as a high-performance foundation for any custom automation.
- **Cross-Platform Desktop Launcher (`v0.3.0`)**: 100% identical graphical launcher and live bot supervisor across Linux, Windows, and macOS. Manage instances, accounts, snapshot backups, `options.txt` settings, live bot processes, and real-time command consoles with zero extra runtime dependencies.
- **Mathematically Exact Physics**: 20 TPS client-side prediction matching vanilla Java Edition with maximum position error under `3.41e-7` blocks and 0 ground-state mismatches across slabs, stairs, climbables, and fluids.
- **Uncompromised Efficiency**: Consumes **~8.2 MiB RSS** (over **98% less memory** than the vanilla JVM client) and launches in **~0.12 s** (**~120x faster cold start**).
- **Rich RL & AI Observations**: Supplies exact sub-voxel state, velocities, orientation, health/hunger vitals, surrounding 3D voxel geometry, entity bounding boxes with eye raycasting, and complete container slot components without fake stubs.
- **Model Context Protocol (MCP)**: Native stdio JSON-RPC 2.0 server exposing 9 powerful tools for autonomous AI agent control.
- **Optional Graphical Bridge**: Includes an optional Fabric reference client bridge (`sawbyte-bridge`) for visual replay and parity verification against vanilla Java Minecraft.

---

## 🚀 Graphical Desktop Launcher & Bot Supervisor

The Sawbyte Graphical Launcher provides a modern, responsive web dashboard on `http://127.0.0.1:8767` with automatic native browser integration.

### Launching the Desktop GUI

| Platform | Quickstart Command / Action |
|:---|:---|
| **Linux** | Run `./sawbyte-launcher`, `sawbyte launcher gui`, or open **Sawbyte Launcher** from your desktop app menu (`sawbyte.desktop`) |
| **Windows** | Double-click `Sawbyte Launcher.bat` or run `sawbyte.exe launcher gui` in PowerShell/CMD |
| **macOS** | Double-click `Sawbyte Launcher.command` or run `./sawbyte launcher gui` in Terminal |

### GUI Features

1. **Instance Manager**:
   - Create, list, configure, and delete headless client instances.
   - Edit Minecraft settings (`options.txt`, `fov`, `renderDistance`, `gamma`, keybinds).
   - Take point-in-time snapshot backups and rollback with one click.
2. **Account Manager**:
   - Add offline development accounts with deterministic or randomized UUIDs.
   - Microsoft OAuth2 Device Code authentication flow display.
   - Safe per-instance account locking to prevent concurrent session collisions.
3. **Live Bot Supervisor & Console**:
   - One-click launch and termination of live `sawbyte-play` background processes.
   - Live process table displaying PID, runtime uptime, target server, and control endpoints.
   - Interactive command console: execute `inspect.status`, `chat.send`, `wait.ticks`, or custom scripts in real time.
   - Real-time live log viewer streaming from `logs/latest.log`.
4. **Profiles & Benchmarks**:
   - Inspect version profiles, block registry palettes, and view live Crucible parity benchmark reports.

---

## 🏆 Performance & Crucible Dual-Interface Benchmarks

Sawbyte is tested against a live Minecraft 26.3 server using the dual-interface Crucible harness ([`tools/crucible_bench.py`](tools/crucible_bench.py)). The benchmark runs every scenario through both the standalone native client (`sawbyte-play`) and the graphical reference client (`sawbyte-bridge`), verifying all outcomes via server-side RCON ground truth. Detailed reports and historical data are documented in [docs/BENCHMARKS.md](docs/BENCHMARKS.md).

| Metric / Stage | Native Client (`sawbyte-play`) | Reference Client (`sawbyte-bridge`) | Parity Verification |
|:---|:---:|:---:|:---|
| **Peak Memory (RSS)** | **~8.2 MiB** | **~614 MiB** | **~98.7% memory reduction** |
| **Startup to Active Tick** | **~0.12 s** | **~14.5 s** | **~120x cold-start speedup** |
| **Stage 1: Biome Navigation** | **PASS** | **PASS** | 100% identical pathing over vegetation, water, stairs, slabs |
| **Stage 2: Mining & Bridging** | **PASS** | **PASS** | Rejects bedrock, breaks logs/stone, bridges gap |
| **Stage 3: Container Transfer** | **PASS** | **PASS** | Resolves custom NBT components, transfers multi-slot items |
| **Stage 4: Occluded Combat** | **PASS** | **PASS** | Rejects wall attacks, repositions for line-of-sight strikes |
| **Stage 5: Preemption & RL** | **PASS** | **PASS** | Immediate preemption, 20-tick sequential `gym.step` loop |
| **Stage 6: Lethal Respawn** | **PASS** | **PASS** | Recovers after `/kill`, auto-respawns, recovers inventory |

---

## Quickstart & CLI Usage

### 1. Persistent Launcher CLI

Manage instances and launch headless bots directly from the command line:

```bash
# Create an instance named "bot-01" with the native 26.3 runtime
sawbyte launcher create bot-01 sawbyte-26.3 native

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

# Send chat commands or navigate
sawbyte eval 'chat.send "/help"' --connect 127.0.0.1:25580
sawbyte eval 'task.navigate 100 64 200' --connect 127.0.0.1:25580
```

### 4. Model Context Protocol (MCP)

Run Sawbyte as a standard MCP server for Claude Desktop, Gemini, or any MCP-compliant AI orchestrator:

```bash
sawbyte mcp --stdio
```

Exposed MCP Tools:
- `launcher_control`: Manage instances, snapshots, accounts, and spawn live bots.
- `client_menu_and_settings`: Configure options and persist settings on disk.
- `player_input`: Send smooth/direct camera yaw/pitch deltas and movement keys.
- `player_interact`: Perform line-of-sight raycast block mining, placement, and entity attacks.
- `gui_screen_automation`: Inspect server containers and click slots with `state_id` transaction verification.
- `task_and_navigation`: Execute 3D A* pathfinding across loaded server chunk sections.
- `world_inspector_and_profiler`: Render ASCII maps and query chunk voxels.
- `gym_step_and_observe`: Synchronous tick stepping and observation for RL agents.
- `code_execute`: Execute multi-line SawbyteScript sequences.

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

Pre-built, self-contained binaries are available on the [Releases](https://github.com/5ivesaw/Sawbyte/releases) page for every release:

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

<p align="center">
  <img src="assets/sawbyte.png" width="148" alt="Sawbyte Logo" />
</p>

# Sawbyte

> **Development status:** Sawbyte is an experimental client. The built-in block profile currently contains only two block states. Supply a suitable 26.3 profile for live worlds. Gameplay, automation and benchmark coverage remain incomplete.

**Sawbyte** is a zero-graphics native Rust Minecraft client (`sawbyte-play`) and deterministic, tick-synchronized automation and reinforcement learning runtime (`sawbyte`). Designed for high-performance programmatic botting, automated multi-step interaction, and neural network training, it operates directly on raw protocol wire frames and sub-voxel physics simulation without rendering a window.

<p align="center">
  <a href="https://github.com/5ivesaw/Sawbyte/actions/workflows/ci.yml"><img src="https://img.shields.io/github/actions/workflow/status/5ivesaw/Sawbyte/ci.yml?branch=main&style=for-the-badge&label=Build&labelColor=1a1a1a&color=2B6E5B" alt="Build and verify" /></a>
  <a href="https://github.com/5ivesaw/Sawbyte/actions/workflows/player.yml"><img src="https://img.shields.io/github/actions/workflow/status/5ivesaw/Sawbyte/player.yml?branch=main&style=for-the-badge&label=Movement%20Lab&labelColor=1a1a1a&color=C47A2C" alt="Movement lab" /></a>
  <img src="https://img.shields.io/badge/Minecraft-26.3%20(Protocol%20777)-8C5230?style=for-the-badge&labelColor=1a1a1a" alt="Protocol 777" />
  <img src="https://img.shields.io/badge/Rust-2024%20%2F%202021-36528C?style=for-the-badge&labelColor=1a1a1a" alt="Rust" />
</p>

---

## Core Capabilities

- **Zero-Graphics Native Client (`sawbyte-play`)**: Connects directly to vanilla Minecraft Java servers over raw TCP with full protocol 777 support (Minecraft 26.3). Features AES-CFB8 stream cipher encryption, zlib compression framing, chunk section palette unpacking, dynamic block updates, entity motion tracking, and authoritative container state machines.
- **Bit-Exact 20 TPS Physics Simulation**: Replicates client-side movement and collision resolution with zero drift (`0.0` coordinate divergence against the reference vanilla client). Implements single-precision (`f32`) trigonometric yaw lookup tables, sequential `[Y, X, Z]` AABB collision resolution, and complete block bounding boxes (stairs, slabs, fences, walls, climbables, and fluids).
- **Deterministic Tick-Synchronized Control**: Exposes composable control primitives that wait for real server ticks and event outcomes rather than guessing with sleeps or arbitrary delays. Movement submissions cleanly preempt prior trajectories, preventing queue desynchronization.
- **Raycast & Interaction Verification**: Verifies line of sight and reach distances for block mining, block placement, and entity attacks. Block mining respects true hardness values and tool speeds across discrete ticks, while combat checks for blocking voxel geometry.
- **Reinforcement Learning & MCP Interface**: Native Gymnasium stepping (`gym.step`, `gym.observe`) returning un-truncated mathematical observations (sub-voxel positions, velocity vectors, orientation, voxel collision patches, inventory contents, and tracked entity bounding boxes) alongside a Model Context Protocol (`sawbyte mcp`) tool server.
- **Embedded Out-of-the-Box Profiles**: Runs immediately without external configuration or profile files via an embedded protocol 777 (Minecraft 26.3) block state table.

---

## Performance & Hard Engine Benchmarks

Sawbyte executes all 3D world voxel geometry, collision resolution, entity tracking, and player kinematics purely in native Rust without a graphical engine. Benchmarked directly via `sawbyte benchmark`:

| Subsystem / Workload | Workload Scale | Throughput | Mean Latency | Real-Time Multiplier |
| :--- | :--- | :--- | :--- | :--- |
| **3D Minecraft 26.3 Physics & Collision** | 100,000 continuous ticks | **1,145,916 TPS** | **872.7 ns / tick** | **57,295.8x** |
| **Sub-Voxel Spatial Raycasting & Occlusion** | 100,000 spherical rays (32m) | **137,318 rays / s** | **7.28 µs / ray** | *Sub-millisecond DDA* |
| **3D A\* Voxel Navigation & Pathfinding** | 500 heuristic obstacle queries | **168 paths / s** | **5.95 ms / path** | *Budgeted 2,048 nodes* |
| **Binary SBIF Transport Serialization** | 16,000 input frames (100 batches) | **37,300,000 frames / s** | **26.8 ns / frame** | **22.0x faster than JSON** |
| **JSON-RPC Transport Framing** | 16,000 input frames (100 batches) | **1,700,000 frames / s** | **589.6 ns / frame** | *Standard text bridge* |

> *Benchmark conducted on x86_64 Linux, release profile with LTO. Run `sawbyte benchmark` to reproduce.*

---

## Quickstart & CLI Usage

### 1. Launching the Headless Client

Start an interactive session connected to a Minecraft Java 26.3 server (using the embedded profile out-of-the-box):

```bash
# Direct execution (embedded 26.3 profile out-of-the-box):
sawbyte-play live 127.0.0.1:25565 --offline BotUser --listen 127.0.0.1:25580

# Or managed via the persistent multi-instance launcher:
sawbyte launcher create bot-01 sawbyte-26.3 native
sawbyte launcher launch bot-01 127.0.0.1:25565 BotUser --wait
```

To run a headless scripted routine or reinforcement learning evaluation:

```bash
sawbyte-play live 127.0.0.1:25565 --offline BotUser \
  --script "task.navigate 10 -60 12\nwait.ticks 20\ngym.observe" \
  --ticks 500 \
  --trace trace.csv
```

### 2. Live Interactive Control

Send commands directly over TCP to the active `sawbyte-play` session:

```bash
# 3D A* navigation across loaded server chunks
sawbyte eval "task.navigate 10 -60 15"

# Tick-driven block mining with hardness-adjusted duration
sawbyte eval "interact.mine 10 -60 14"

# Container screen inspection, synchronization, and slot clicks
sawbyte eval "wait.until screen_open 40 \"Chest\""
sawbyte eval "gui.click_slot 0 pickup"
sawbyte eval "gui.close"

# Reinforcement learning step (advances exactly 1 tick and observes post-tick state)
sawbyte eval "gym.step 1 0 false false false 0 0"
```

### 3. Model Context Protocol (MCP)

Sawbyte provides a standard Model Context Protocol (MCP) JSON-RPC 2.0 server over stdio for AI agent orchestration:

```bash
sawbyte mcp --stdio
```

Available MCP tools include `launcher_control`, `client_menu_and_settings`, `player_input`, `player_interact`, `gui_screen_automation`, `task_and_navigation`, `world_inspector_and_profiler`, `gym_step_and_observe`, and `code_execute`.

### 4. Running Hard Benchmarks

Execute the full suite of in-memory physics simulation, spatial raycasting, A* navigation, and transport serialization benchmarks:

```bash
sawbyte benchmark
```

---

## SDK Integration

### Python SDK (`sdks/python`)

```python
from sawbyte import SawbyteEnv, SawbyteLiveClient, WorkflowBuilder

# Gymnasium Reinforcement Learning Loop
env = SawbyteEnv(endpoint="127.0.0.1:25580")
obs, info = env.reset()

for _ in range(100):
    # Action: [forward, strafe, jump, sneak, sprint, yaw_delta, pitch_delta]
    action = [1, 0, False, False, False, 0.0, 0.0]
    obs, reward, terminated, truncated, info = env.step(action)
    if terminated or truncated:
        obs, info = env.reset()

env.close()

# Declarative Workflow Builder
wf = (
    WorkflowBuilder()
    .navigate(12, -60, 4)
    .wait_until("screen_open", 100, "Chest")
    .gui_click_slot(0, "pickup")
    .wait_ticks(20)
    .gui_close()
)

client = SawbyteLiveClient("127.0.0.1:25580")
client.eval(wf.compile_script())
client.close()
```

### TypeScript SDK (`sdks/typescript`)

```typescript
import { SawbyteWorkflow, createMcpToolCallRequest } from "@sawbyte/sdk";

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

## Repository Architecture

```text
Sawbyte/
├── native/                  # Standalone zero-graphics Rust client (sawbyte-play & sawbyte_play lib)
│   ├── src/auth.rs          # Microsoft OAuth2 device-code authentication & session tokens
│   ├── src/inventory.rs     # Protocol 777 inventory decoding, container state machines & items
│   ├── src/player.rs        # 20 TPS physics, AABB collisions, jump/sprint dynamics & trig table
│   ├── src/raycast.rs       # Voxel AABB eye raycasting & line-of-sight occlusion checks
│   ├── src/wire.rs          # Packet framing, VarInt codecs, AES-CFB8 & zlib stream handlers
│   └── src/world.rs         # Chunk section palette decoding, 3D voxel storage & block shapes
├── crates/
│   ├── sawbyte-core/        # Input scheduling, humanisation filters, launcher & MCP server
│   ├── sawbyte-protocol/    # SLP server list ping & bounded wire protocol primitives
│   └── sawbyte-cli/         # Primary CLI (sawbyte) for launcher, diagnostics & evaluation
├── sdks/
│   ├── python/              # Python SDK & Gymnasium environment (sawbyte.SawbyteEnv)
│   └── typescript/          # TypeScript SDK & MCP request builders (@sawbyte/sdk)
└── bridge/                  # Optional Fabric reference replay mod for graphical parity comparison
```

---

## License

This project is licensed under the MIT License. See [LICENSE](LICENSE) for details.

*Disclaimer: Not an official Minecraft product. Not approved by or associated with Mojang or Microsoft.*

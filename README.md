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

## Current capabilities and limits

Sawbyte includes a standalone Rust client, command-line session control, an optional reference-client bridge, and early Python, TypeScript and MCP integrations. The release goal is broad player-equivalent automation. Many mechanics, state decoders and launcher features still need implementation or independent validation.

A Linux CI fixture compared 160 ticks of walking, turning, jumping and stopping on a flat 26.3 world against the normal client. An earlier saved result reported maximum position error of approximately 0.000000341 blocks and no ground-state mismatches. This is evidence for that fixture only. See [coverage](docs/COVERAGE.md) and the [current handoff](docs/NEXT-AGENT.md) for scope and remaining work.

## Performance measurements

The `sawbyte benchmark` command measures isolated in-memory operations. It does not measure full-client performance, network load, heavy real-server gameplay, or multi-client capacity. Earlier short CI fixtures sampled about 5 MiB RSS for one native process, without a representative world or full launcher/controller cost. Reliable end-to-end CPU and memory figures are still pending.

---

## Quickstart & CLI Usage

### 1. Launching the Headless Client

Start an experimental interactive session connected to a Minecraft Java 26.3 server with a separately supplied version profile:

```bash
# The built-in profile is a minimal fixture. Supply a compatible block profile.
sawbyte-play live 127.0.0.1:25565 --offline BotUser block-profile.json --listen 127.0.0.1:25580

# Or managed via the persistent multi-instance launcher:
sawbyte launcher create bot-01 sawbyte-26.3 native
sawbyte launcher add-account BotUser 00000000000030008000000000000001
sawbyte launcher launch bot-01 127.0.0.1:25565 BotUser "$PWD/block-profile.json" --wait
```

### 2. Live Interactive Control

Connect to the local control port of a running session. The server address and Sawbyte control address are different endpoints.

```bash
sawbyte eval "inspect.status" --connect 127.0.0.1:25580
sawbyte eval "inspect.chat" --connect 127.0.0.1:25580
sawbyte eval "inspect.events" --connect 127.0.0.1:25580
```

Action and training interfaces are experimental. Queued actions and observed state changes require separate verification.

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

The Python adapter is experimental. Its training environment does not yet reset the server world, and some convenience methods remain incomplete. A basic status query uses the direct TCP client:

```python
from sawbyte import SawbyteLiveTcpClient

with SawbyteLiveTcpClient("127.0.0.1:25580") as client:
    print(client.send_command("inspect.status"))
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

*Disclaimer: Not an official Minecraft product. Not approved by or associated with Mojang or Microslop(Microsoft).*

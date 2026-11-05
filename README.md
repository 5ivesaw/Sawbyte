<p align="center">
  <img src="assets/sawbyte.png" width="120" alt="Sawbyte" />
</p>

<h1 align="center">Sawbyte</h1>

<p align="center">
  Zero-graphics native Rust Minecraft 26.3 client, desktop launcher, and automation engine.
</p>

<p align="center">
  <a href="https://github.com/5ivesaw/Sawbyte/actions/workflows/ci.yml"><img src="https://img.shields.io/github/actions/workflow/status/5ivesaw/Sawbyte/ci.yml?branch=main&style=flat-square&label=build&labelColor=161b22&color=238636" alt="Build" /></a>
  <a href="https://github.com/5ivesaw/Sawbyte/actions/workflows/player.yml"><img src="https://img.shields.io/github/actions/workflow/status/5ivesaw/Sawbyte/player.yml?branch=main&style=flat-square&label=parity%20lab&labelColor=161b22&color=238636" alt="Parity" /></a>
  <a href="https://github.com/5ivesaw/Sawbyte/releases/tag/v0.3.3"><img src="https://img.shields.io/badge/v0.3.3-1f6feb?style=flat-square&label=release&labelColor=161b22" alt="v0.3.3" /></a>
  <img src="https://img.shields.io/badge/26.3%20·%20Protocol%20777-8c5230?style=flat-square&label=minecraft&labelColor=161b22" alt="MC 26.3" />
  <a href="LICENSE"><img src="https://img.shields.io/badge/MIT-6e7681?style=flat-square&label=license&labelColor=161b22" alt="MIT" /></a>
</p>

<p align="center">
  <a href="https://5ivesaw.github.io/Sawbyte/"><b>Portal</b></a> · <a href="https://github.com/5ivesaw/Sawbyte/releases/tag/v0.3.3"><b>Downloads</b></a> · <a href="docs/VISION.md"><b>Vision</b></a> · <a href=".agents/skills/sawbyte/SKILL.md"><b>AI Skill</b></a>
</p>

---

### At a Glance

| Metric | Value |
|:---|:---|
| Memory | **~8.2 MiB RSS** (98.7% less than vanilla JVM) |
| Cold Start | **~0.12 s** (120x faster) |
| Physics Parity | < 3.41e-7 blocks max error, zero ground mismatch |
| AI Benchmark | 28/28 passed (MineRL, MineDojo, Voyager, Systems) |
| Crucible | 6/6 stages, dual-backend (native + bridge) |

---

### What It Does

- **Headless Native Client** -- Connects to real Minecraft 26.3 servers over raw protocol, runs sub-voxel physics at 20 TPS, no window needed.
- **Desktop Launcher** -- Electron-based cross-platform GUI for managing instances, accounts, mods, snapshots, and live bot supervision.
- **Automation Engine** -- Composable low-level operations for movement, mining, placement, combat, container transactions, and GUI automation.
- **AI Agent Interface** -- MCP server (10 tools), Gymnasium environment, Python and TypeScript SDKs, persistent milestone logging.
- **Software 3D Visualizer** -- Pure-Rust raymarcher and radar renderer, zero GPU dependencies.
- **Fabric Bridge** -- Optional graphical mod for visual verification against the vanilla Java client.

---

### Install

**One-line installer:**

```bash
# Linux / macOS
curl -sSL https://raw.githubusercontent.com/5ivesaw/Sawbyte/main/packaging/install.sh | bash

# Windows -- download the release zip and run install.bat
```

**Or download from [Releases](https://github.com/5ivesaw/Sawbyte/releases/tag/v0.3.3):**

| Platform | Engine | Launcher |
|:---|:---|:---|
| Linux x64 | [sawbyte-ubuntu-24.04-X64.tar.gz](https://github.com/5ivesaw/Sawbyte/releases/download/v0.3.3/sawbyte-ubuntu-24.04-X64.tar.gz) | [sawbyte-launcher-ubuntu-24.04-X64.tar.gz](https://github.com/5ivesaw/Sawbyte/releases/download/v0.3.3/sawbyte-launcher-ubuntu-24.04-X64.tar.gz) |
| Windows x64 | [sawbyte-windows-2022-X64.zip](https://github.com/5ivesaw/Sawbyte/releases/download/v0.3.3/sawbyte-windows-2022-X64.zip) | [sawbyte-launcher-windows-2022-X64.zip](https://github.com/5ivesaw/Sawbyte/releases/download/v0.3.3/sawbyte-launcher-windows-2022-X64.zip) |
| macOS ARM64 | [sawbyte-macos-14-ARM64.tar.gz](https://github.com/5ivesaw/Sawbyte/releases/download/v0.3.3/sawbyte-macos-14-ARM64.tar.gz) | [sawbyte-launcher-macos-14-ARM64.tar.gz](https://github.com/5ivesaw/Sawbyte/releases/download/v0.3.3/sawbyte-launcher-macos-14-ARM64.tar.gz) |

---

### Quick Start

```bash
# Launch the desktop GUI
./sawbyte-launcher            # Linux
Sawbyte Launcher.bat          # Windows
open Sawbyte Launcher.command # macOS

# Or use the CLI directly
sawbyte launcher create bot-01 sawbyte-26.3 native
sawbyte launcher launch bot-01 127.0.0.1:25565 BotUser block-profile.json --wait

# Standalone headless client with remote control
sawbyte-play live 127.0.0.1:25565 --offline BotUser block-profile.json --listen 127.0.0.1:25580

# Query live state
sawbyte eval "inspect.status" --connect 127.0.0.1:25580

# MCP server for AI agents
sawbyte mcp --stdio
```

---

### SDKs

**Python** -- zero-dependency client with Gymnasium support:

```python
from sawbyte import SawbyteLiveTcpClient, SawbyteEnv

with SawbyteLiveTcpClient("127.0.0.1:25580") as client:
    print(client.send_command("inspect.status"))

env = SawbyteEnv(connect_addr="127.0.0.1:25580")
obs = env.reset()
for _ in range(100):
    obs, reward, done, trunc, info = env.step(env.action_space.sample())
```

**TypeScript** -- composable workflow builder:

```typescript
import { SawbyteWorkflow } from "@sawbyte/sdk";

const wf = new SawbyteWorkflow()
  .connectLive("127.0.0.1:25580")
  .navigate(12, -60, 4)
  .waitUntil("screen_open", 100, "Chest")
  .guiClickSlot(0, "pickup");
```

---

### Repository Structure

```
Sawbyte/
  native/           Standalone zero-graphics Rust client (sawbyte-play)
  crates/            Core library, protocol codec, CLI + desktop launcher
  launcher/          Electron desktop launcher (Vue 3 + Vuetify)
  bridge/            Optional Fabric graphical bridge mod
  sdks/python/       Python SDK + Gymnasium environment
  sdks/typescript/   TypeScript SDK + MCP client
  packaging/         Cross-platform installers and desktop entries
  docs/              GitHub Pages portal + benchmark reports
  tools/             Crucible, Universal, and AI Agent benchmark harnesses
```

---

### License

MIT. See [LICENSE](LICENSE).

*Not an official Minecraft product. Not approved by or associated with Mojang or Microsoft.*

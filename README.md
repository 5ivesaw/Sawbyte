<p align="center">
  <img src="assets/sawbyte.png" width="148" alt="Sawbyte icon" />
</p>

<p align="center">
  <img src="assets/cream-hero.svg" width="100%" alt="Sawbyte Cream Hero Banner" />
</p>

<p align="center">
  <a href="https://github.com/5ivesaw/Sawbyte/actions/workflows/ci.yml"><img src="https://img.shields.io/github/actions/workflow/status/5ivesaw/Sawbyte/ci.yml?branch=main&style=for-the-badge&label=Build%20%26%20Verify&labelColor=F3ECE0&color=2B6E5B" alt="Build and verify" /></a>
  <a href="https://github.com/5ivesaw/Sawbyte/actions/workflows/player.yml"><img src="https://img.shields.io/github/actions/workflow/status/5ivesaw/Sawbyte/player.yml?branch=main&style=for-the-badge&label=26.3%20Parity%20Lab&labelColor=F3ECE0&color=C47A2C" alt="Real player milestone" /></a>
  <a href="LICENSE"><img src="https://img.shields.io/badge/License-MIT-8C5230?style=for-the-badge&labelColor=F3ECE0" alt="MIT license" /></a>
  <a href="docs/COVERAGE.md"><img src="https://img.shields.io/badge/MCP_2.0-Agent_Ready-1F1D1A?style=for-the-badge&labelColor=F3ECE0" alt="MCP Ready" /></a>
</p>

<p align="center">
  <a href="#download--quick-start"><strong>Downloads &amp; CLI</strong></a> ·
  <a href="#model-context-protocol-mcp--ai-agent-control"><strong>MCP &amp; AI Agents</strong></a> ·
  <a href="#complete-programmability--gui-workflows"><strong>GUI &amp; Workflows</strong></a> ·
  <a href="#python--typescript-sdks"><strong>Python &amp; TS SDKs</strong></a> ·
  <a href="PLAN.md"><strong>Architecture Plan</strong></a> ·
  <a href="docs/COVERAGE.md"><strong>Capability Ledger</strong></a> ·
  <a href="docs/HANDOFF.md"><strong>Technical Reference</strong></a>
</p>

---

**Sawbyte** is an independently written, aggressively stripped-down **Rust Minecraft Java client, launcher, and full-stack automation platform** with an optional **normal-client Fabric bridge**. Every player operation—from out-of-game launcher instances, main menus, and client settings to in-game movement, raycast mining/building, complex multi-screen shop GUIs, and dynamic code execution—is programmable through a shared input and event pipeline, with configurable humanisation enabled by default.

<p align="center">
  <img src="assets/cream-metrics.svg" width="100%" alt="Verified 26.3 CI Parity and Resource Metrics" />
</p>

<p align="center">
  <img src="assets/cream-architecture.svg" width="100%" alt="Sawbyte Unified Architecture" />
</p>

---

## What Is Built & Verified

- **Standalone Native 26.3 Runtime (`native/`, `sawbyte-play`)**:
  - Independently written Rust 26.3 (`protocol 777`) login, configuration, AES-128-CFB8 encryption, zlib framing, chunk palette storage, dynamic block updates (`BlockUpdate` packet `8` & `SectionBlocksUpdate` packet `86`), and collision/motion physics. Zero Azalea or external game-engine dependencies.
  - Verified against the real Minecraft 26.3 client (`sawbyte-bridge` under HeadlessMC) on a live vanilla 26.3 server in GitHub Actions: **`max_position_error_blocks = 3.41e-7`**, **`rms_position_error_blocks = 2.29e-7`**, **`0` ground mismatches**, and **`4,380 KiB` (~4.28 MiB) sampled peak RSS** on the 160-tick fixture.
  - Extended collision & movement engine covering slabs, stairs, fences, carpets, climbables (ladders/vines/scaffolding), water/lava fluids, cobwebs, ice slipperiness, sprinting, sneaking with ground edge-clamping, `0.6`-block stepping, player attributes, status effects, eye-to-target voxel/entity raycasting with occlusion, and serverbound interaction packets.
- **Full-Client Graphical Bridge (`bridge/`, `sawbyte-bridge`)**:
  - Fabric 26.3 client mod sharing the exact same binary `.sbif` input recordings and supporting external runtime commands (`hotbar`, `respawn`, `disconnect`, and manual player priority/handoff `handoff.pause` / `handoff.resume`).
- **Complete GUI, Container, Menu & Settings Programmability (`crates/sawbyte-core`)**:
  - Revision-checked (`state_id`) container & screen state machine supporting all 7 `ContainerClick` modes (`Pickup`, `QuickMove`, `Swap`, `Clone`, `Throw`, `QuickCraft`, `PickupAll`), 26.3 item components, widgets, editable text fields, anvils, signs, books, villager trades, and server dialogs.
  - Disambiguated `ItemSelector` engine (matching ID, exact/substring display name, lore, components, count, and slot range; rejecting ambiguous matches unless explicitly disambiguated).
  - Multi-screen workflow engine with variables, conditionals, bounded loops, event/screen waits, and `RetryPolicy::Never` protection so payment/irrevocable clicks are never repeated after a timeout.
  - Out-of-game client menu & settings controller (`MainMenu`, `Singleplayer`, `Multiplayer`, `DirectConnect`, `Options`, `VideoSettings`, `AudioSettings`, `Keybinds`, `PauseMenu`, `options.txt` export).
- **Model Context Protocol (MCP) Server & `SawbyteScript` VM (`sawbyte mcp`)**:
  - Built-in JSON-RPC 2.0 MCP server over `stdio` exposing **10 agent tools**, **4 live resources**, and prompts so any AI agent can control the launcher, client menus, settings, player inputs, world interactions, GUI workflows, 3D A* pathfinding, multi-agent swarms, and dynamic script execution.
- **Humanisation Enabled by Default + Freedom**:
  - Bounded (`Humanised`) and seedable smooth acceleration/deceleration (`SmoothHumanisedCamera`) aiming policies, Fitts'-law-inspired `GuiTimingPolicy` virtual cursor travel and post-revision reaction delays, and `TypingPolicy` character cadence—plus an instant `Direct` bypass and custom `InputPolicy` trait hooks.

> **Scope & Honesty Note**: The live vanilla 26.3 server comparison in CI currently certifies the 160-tick controlled flat-world fixture; extended collision shapes, GUI workflows, 3D A* navigation, launcher profiles (`sawbyte-26.3` and `sawbyte-26.2`), and MCP tools are verified by deterministic unit and integration suites in CI. Online Microsoft sign-in requires a live account and registered client ID. See [docs/COVERAGE.md](docs/COVERAGE.md) and [docs/PLAYER.md](docs/PLAYER.md).

---

## Model Context Protocol (MCP) & AI Agent Control

Any MCP-compatible AI client (Claude Desktop, Cursor, Antigravity, custom LLM agents) can attach directly to `sawbyte mcp --stdio` to control **the launcher, out-of-game menus, client settings, in-game actions, GUI workflows, and code execution**:

```json
{
  "mcpServers": {
    "sawbyte": {
      "command": "sawbyte",
      "args": ["mcp", "--stdio"]
    }
  }
}
```

| MCP Tool | Capabilities Exposed to AI Agents |
|---|---|
| `launcher_control` | Create, list, backup (token-free), rollback, and lock isolated instances (`sawbyte-26.3`, `sawbyte-26.2`) & accounts |
| `client_menu_and_settings` | Navigate `main_menu`, `singleplayer`, `multiplayer`, `direct_connect:<ip>`, `options`, `keybinds`, and edit `options.txt` settings |
| `player_input` | Submit continuous movement buttons, humanised/smooth/direct camera look deltas, mouse clicks, hotbar selection, or manual handoff |
| `player_interact` | Execute raycast-verified block mining, block placement, item use, and entity combat/interaction |
| `gui_screen_automation` | Inspect open screens, run `ItemSelector` queries, click slots/widgets with `state_id` checks, fill text fields, or run the multi-screen shop workflow |
| `task_and_navigation` | Run budgeted 3D A* jump-aware pathfinding and composable tasks (`navigate`, `mine`, `place`, `craft`, `smelt`, `trade`, `equip`, `eat`) |
| `code_execute` | Execute multi-line `SawbyteScript` automation programs combining launcher, menu, setting, gameplay, GUI, and swarm commands |
| `workflow_recorder` | Export recorded manual or scripted actions into reusable workflows |
| `world_inspector_and_profiler` | Render the live 2D ASCII top-down world map, generate screen selectors, and inspect per-subsystem memory usage |
| `swarm_and_gym` | Spawn and step 1–64 concurrent agents in shared or isolated worlds |

---

## Complete Programmability & GUI Workflows

Execute a full-stack script from the command line or through MCP:

```bash
sawbyte eval "
  launcher.create_instance survival-bot sawbyte-26.3 native
  launcher.backup survival-bot
  menu.goto options
  settings.set fov 90
  menu.goto direct_connect:127.0.0.1:25565
  player.select_hotbar 2
  interact.place 1 -60 1 2
  task.navigate 4 -60 4
  gui.run_shop_workflow minecraft:stone 64
  swarm.spawn 4 shared
  inspect.map
  inspect.profile
"
```

---

## Python & TypeScript SDKs

### Python SDK & Gymnasium RL Environment (`sdks/python/`)

```python
from sawbyte import SawbyteEnv, SawbyteMcpClient, WorkflowBuilder

wf = (
    WorkflowBuilder(policy="humanised")
    .create_instance("agent-1", "sawbyte-26.3", "native")
    .goto_menu("options")
    .set_setting("fov", "90")
    .navigate(4, -60, 4)
    .run_shop_purchase("minecraft:stone", amount=64)
)

with SawbyteMcpClient("sawbyte") as mcp:
    print(mcp.execute_workflow(wf))

env = SawbyteEnv(observation_mode="visible")
obs, info = env.reset(seed=42)
obs, reward, terminated, truncated, step_info = env.step({"forward": True, "yaw_delta": 6.0})
```

### TypeScript SDK (`sdks/typescript/`)

```typescript
import { SawbyteWorkflow, createMcpToolCallRequest } from "./sdks/typescript/index";

const workflow = new SawbyteWorkflow()
  .createInstance("ts-agent", "sawbyte-26.3", "native")
  .gotoMenu("multiplayer")
  .setSetting("renderDistance", 4)
  .navigate(4, -60, 4)
  .runShopPurchase("minecraft:stone", 64);

const rpcPayload = createMcpToolCallRequest(1, "code_execute", workflow.compileScript());
```

---

## Download & Quick Start

Open **Actions → Build and verify** (or **Real player milestone**) → select the latest run → download the artifact for your OS (`linux-x64`, `windows-x64`, or `macos-arm64`). Every archive includes `SHA256SUMS` and platform icons generated from `assets/sawbyte.png`.

```text
sawbyte --help
sawbyte capabilities
sawbyte launcher list
sawbyte workflow shop-demo
sawbyte inspect
sawbyte benchmark
sawbyte mcp --stdio
sawbyte status 127.0.0.1:25565
sawbyte tape movement.sbif
sawbyte-play 127.0.0.1:25565 --offline SawbyteNative movement.sbif native.csv block-profile.json
```

---

## Repository Structure

| Component | Description |
|---|---|
| [`crates/sawbyte-core`](crates/sawbyte-core) | Zero-dependency Rust core: scheduler, humanisation policies, discrete sub-tick events, screen selector engine, 3D A* pathfinder, workflow runner, launcher/menu state, and MCP server |
| [`crates/sawbyte-protocol`](crates/sawbyte-protocol) | Bounded Minecraft Java TCP status/ping client and VarInt framing |
| [`crates/sawbyte-cli`](crates/sawbyte-cli) | Unified `sawbyte` CLI (`mcp`, `launcher`, `workflow`, `inspect`, `eval`, `benchmark`, `demo`, `tape`, `status`) |
| [`native/`](native) | Standalone `sawbyte-play` Rust 26.3 client (chunk palettes, dynamic block updates, extended collision/motion physics, raycasting, inventory/container packets, OAuth device login) |
| [`bridge/`](bridge) | Optional Fabric 26.3 mod (`sawbyte-bridge`) for normal graphical Minecraft replay, commands, and manual handoff |
| [`sdks/python`](sdks/python) & [`sdks/typescript`](sdks/typescript) | Zero-dependency Python SDK (with Gymnasium `SawbyteEnv`) and TypeScript SDK |

All compilation, testing, formatting, icon generation, and live 26.3 server/client parity comparisons execute in GitHub Actions. See [docs/DECISIONS.md](docs/DECISIONS.md) and [docs/HANDOFF.md](docs/HANDOFF.md).

*Not an official Minecraft product. Not approved by or associated with Mojang or Microsoft.*

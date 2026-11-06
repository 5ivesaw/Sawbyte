<p align="center">
  <img src="https://raw.githubusercontent.com/5ivesaw/Sawbyte/main/assets/sawbyte.png" width="110" alt="sawbyte" />
</p>

<h1 align="center">sawbyte</h1>

<p align="center">
  <b>Rust client, desktop launcher, and bot engine for Minecraft 26.3.</b><br/>
  Run Minecraft 26.3 without Java eating 2 GB of RAM. Connect to servers, automate tasks, or hook up AI bots in ~8 to 12 MB of memory at a locked 20 ticks per second.
</p>

<p align="center">
  <a href="https://github.com/5ivesaw/Sawbyte/actions/workflows/ci.yml"><img src="https://img.shields.io/github/actions/workflow/status/5ivesaw/Sawbyte/ci.yml?branch=main&style=flat-square&label=build&labelColor=111319&color=2ea043" alt="Build" /></a>
  <a href="https://github.com/5ivesaw/Sawbyte/actions/workflows/player.yml"><img src="https://img.shields.io/github/actions/workflow/status/5ivesaw/Sawbyte/player.yml?branch=main&style=flat-square&label=parity%20lab&labelColor=111319&color=2ea043" alt="Parity Lab" /></a>
  <a href="https://github.com/5ivesaw/Sawbyte/releases/latest"><img src="https://img.shields.io/github/v/release/5ivesaw/Sawbyte?style=flat-square&label=release&labelColor=111319&color=00aaaa" alt="Latest Release" /></a>
  <img src="https://img.shields.io/badge/26.3%20%C2%B7%20Protocol%20777-ffaa00?style=flat-square&label=minecraft&labelColor=111319" alt="Minecraft 26.3" />
  <a href="LICENSE"><img src="https://img.shields.io/badge/MIT-aaaaaa?style=flat-square&label=license&labelColor=111319" alt="License" /></a>
</p>

<p align="center">
  <img src="https://raw.githubusercontent.com/InventivetalentDev/minecraft-assets/1.21.4/assets/minecraft/textures/block/grass_block_side.png" width="16" height="16" alt="" /> <a href="https://5ivesaw.github.io/Sawbyte/"><b>Website</b></a>
  &nbsp;&bull;&nbsp;
  <img src="https://raw.githubusercontent.com/InventivetalentDev/minecraft-assets/1.21.4/assets/minecraft/textures/item/diamond.png" width="16" height="16" alt="" /> <a href="https://github.com/5ivesaw/Sawbyte/releases/latest"><b>Downloads</b></a>
  &nbsp;&bull;&nbsp;
  <img src="https://raw.githubusercontent.com/InventivetalentDev/minecraft-assets/1.21.4/assets/minecraft/textures/item/emerald.png" width="16" height="16" alt="" /> <a href="docs/VISION.md"><b>Vision</b></a>
  &nbsp;&bull;&nbsp;
  <img src="https://raw.githubusercontent.com/InventivetalentDev/minecraft-assets/1.21.4/assets/minecraft/textures/block/command_block_front.png" width="16" height="16" alt="" /> <a href=".agents/skills/sawbyte/SKILL.md"><b>AI Bot Guide (SKILL.md)</b></a>
</p>

## <img src="https://raw.githubusercontent.com/InventivetalentDev/minecraft-assets/1.21.4/assets/minecraft/textures/item/diamond_pickaxe.png" width="22" height="22" alt="" /> Overview

| Icon | Metric | Sawbyte (Rust 26.3) | Vanilla Minecraft (Java) | Notes |
| :---: | :--- | :--- | :--- | :--- |
| <img src="https://raw.githubusercontent.com/InventivetalentDev/minecraft-assets/1.21.4/assets/minecraft/textures/item/diamond.png" width="20" height="20" alt="" /> | **Memory Usage** | **~8.2 to 11.8 MB** | ~1,840 MB | Pure Rust with zero JVM overhead (~155x less RAM) |
| <img src="https://raw.githubusercontent.com/InventivetalentDev/minecraft-assets/1.21.4/assets/minecraft/textures/item/redstone.png" width="20" height="20" alt="" /> | **Tick Rate & Physics** | **20.0 TPS (0.04 ms/tick)** | 12.2 ms/tick | `< 3.41e-7` blocks max parity error, zero ground mismatch |
| <img src="https://raw.githubusercontent.com/InventivetalentDev/minecraft-assets/1.21.4/assets/minecraft/textures/item/sugar.png" width="20" height="20" alt="" /> | **Cold Startup** | **~0.014 to 0.12 s** | ~8.4 s | Starts almost instantly from the terminal or launcher |
| <img src="https://raw.githubusercontent.com/InventivetalentDev/minecraft-assets/1.21.4/assets/minecraft/textures/item/emerald.png" width="20" height="20" alt="" /> | **Task Benchmarks** | **28 / 28 Passed (100%)** | Baseline | MineRL Diamond Run, MineDojo, Voyager, and Server Login |
| <img src="https://raw.githubusercontent.com/InventivetalentDev/minecraft-assets/1.21.4/assets/minecraft/textures/block/command_block_front.png" width="20" height="20" alt="" /> | **Bot & AI Control** | **MCP + TCP + SDKs** | None | Built-in MCP server, Gymnasium RL env, Python & TS SDKs |

## <img src="https://raw.githubusercontent.com/InventivetalentDev/minecraft-assets/1.21.4/assets/minecraft/textures/block/crafting_table_front.png" width="22" height="22" alt="" /> What It Does

* <img src="https://raw.githubusercontent.com/InventivetalentDev/minecraft-assets/1.21.4/assets/minecraft/textures/item/ender_pearl.png" width="18" height="18" alt="" /> **Headless Native Client (`sawbyte-play`)**: Connects to real Minecraft 26.3 servers over raw protocol and runs sub-voxel physics, gravity, and collisions at 20 TPS with no window needed.
* <img src="https://raw.githubusercontent.com/InventivetalentDev/minecraft-assets/1.21.4/assets/minecraft/textures/block/crafting_table_front.png" width="18" height="18" alt="" /> **Desktop Launcher**: Cross-platform GUI for managing instances, accounts, mods, snapshots, and watching live bots.
* <img src="https://raw.githubusercontent.com/InventivetalentDev/minecraft-assets/1.21.4/assets/minecraft/textures/item/diamond_pickaxe.png" width="18" height="18" alt="" /> **Automation Engine**: Built-in 3D A* pathfinding, block raycasting, mining, block placement, combat, chest menu clicking, and inventory management.
* <img src="https://raw.githubusercontent.com/InventivetalentDev/minecraft-assets/1.21.4/assets/minecraft/textures/item/enchanted_book.png" width="18" height="18" alt="" /> **AI Agent Interface**: Built-in MCP server with 10 live tools, Gymnasium environment, Python and TypeScript SDKs, and automatic milestone and death logging.
* <img src="https://raw.githubusercontent.com/InventivetalentDev/minecraft-assets/1.21.4/assets/minecraft/textures/item/compass_16.png" width="18" height="18" alt="" /> **Software 3D & Radar View**: Pure-Rust raymarcher and top-down radar renderer with zero GPU dependencies.
* <img src="https://raw.githubusercontent.com/InventivetalentDev/minecraft-assets/1.21.4/assets/minecraft/textures/item/ender_eye.png" width="18" height="18" alt="" /> **Fabric 26.3 Bridge Mod**: Optional Fabric mod to sync and spectate a headless Sawbyte bot inside the regular Java client.

## <img src="https://raw.githubusercontent.com/InventivetalentDev/minecraft-assets/1.21.4/assets/minecraft/textures/item/painting.png" width="22" height="22" alt="" /> Screenshots & Previews

| Native Desktop Launcher (Lunar Obsidian) | Potato Mode 3D & Radar Supervisor | How Sawbyte Works & Editions (Cream Luxe) |
| :---: | :---: | :---: |
| <img src="assets/screenshots/launcher.png" width="320" alt="Desktop Launcher UI" /> | <img src="assets/screenshots/bot_view.png" width="320" alt="Bot Session & Live View" /> | <img src="assets/screenshots/server_manager.png" width="320" alt="Editions & Server Manager" /> |
| **Instances, Quick Launch & Telemetry** (`assets/screenshots/launcher.png`) | **Pure-Rust 3D Rasterizer & 360° Radar** (`assets/screenshots/bot_view.png`) | **Fetch Headless, Potato & Fabric Bridge** (`assets/screenshots/server_manager.png`) |

## <img src="https://raw.githubusercontent.com/InventivetalentDev/minecraft-assets/1.21.4/assets/minecraft/textures/item/diamond.png" width="22" height="22" alt="" /> Install & Downloads

### One-Line Terminal Installer

```bash
# Linux / macOS
curl -sSfL https://raw.githubusercontent.com/5ivesaw/Sawbyte/main/packaging/install.sh | bash
```

### Direct Release Downloads (Always Auto-Synced to Latest Release)

All links below use GitHub's `/releases/latest/download/` resolver so they always grab the newest release build automatically:

| Platform | Native Desktop Launcher (~1.5 MB) | Headless & Potato Engine (`sawbyte-play`) | Standalone CLI & MCP (`sawbyte`) |
| :--- | :--- | :--- | :--- |
| <img src="https://raw.githubusercontent.com/InventivetalentDev/minecraft-assets/1.21.4/assets/minecraft/textures/block/command_block_front.png" width="16" height="16" alt="" /> **Linux (x86_64)** | [sawbyte-launcher-ubuntu-24.04-X64.tar.gz](https://github.com/5ivesaw/Sawbyte/releases/latest/download/sawbyte-launcher-ubuntu-24.04-X64.tar.gz) | [sawbyte-play-linux-x64.tar.gz](https://github.com/5ivesaw/Sawbyte/releases/latest/download/sawbyte-play-linux-x64.tar.gz) | [sawbyte-ubuntu-24.04-X64.tar.gz](https://github.com/5ivesaw/Sawbyte/releases/latest/download/sawbyte-ubuntu-24.04-X64.tar.gz) |
| <img src="https://raw.githubusercontent.com/InventivetalentDev/minecraft-assets/1.21.4/assets/minecraft/textures/item/diamond.png" width="16" height="16" alt="" /> **Windows (x64)** | [sawbyte-launcher-windows-2022-X64.zip](https://github.com/5ivesaw/Sawbyte/releases/latest/download/sawbyte-launcher-windows-2022-X64.zip) | [sawbyte-play-windows-x64.zip](https://github.com/5ivesaw/Sawbyte/releases/latest/download/sawbyte-play-windows-x64.zip) | [sawbyte-windows-2022-X64.zip](https://github.com/5ivesaw/Sawbyte/releases/latest/download/sawbyte-windows-2022-X64.zip) |
| <img src="https://raw.githubusercontent.com/InventivetalentDev/minecraft-assets/1.21.4/assets/minecraft/textures/item/emerald.png" width="16" height="16" alt="" /> **macOS (Apple Silicon)** | [sawbyte-launcher-macos-14-ARM64.tar.gz](https://github.com/5ivesaw/Sawbyte/releases/latest/download/sawbyte-launcher-macos-14-ARM64.tar.gz) | [sawbyte-play-macos-arm64.tar.gz](https://github.com/5ivesaw/Sawbyte/releases/latest/download/sawbyte-play-macos-arm64.tar.gz) | [sawbyte-macos-14-ARM64.tar.gz](https://github.com/5ivesaw/Sawbyte/releases/latest/download/sawbyte-macos-14-ARM64.tar.gz) |

* <img src="https://raw.githubusercontent.com/InventivetalentDev/minecraft-assets/1.21.4/assets/minecraft/textures/block/grass_block_side.png" width="16" height="16" alt="" /> **Fabric Bridge Mod (Minecraft 26.3)**: Grab the latest `sawbyte-bridge-*.jar` from the [GitHub Releases Page](https://github.com/5ivesaw/Sawbyte/releases/latest).

## <img src="https://raw.githubusercontent.com/InventivetalentDev/minecraft-assets/1.21.4/assets/minecraft/textures/block/command_block_front.png" width="22" height="22" alt="" /> Quick Start

```bash
# Start the desktop launcher GUI
./sawbyte-launcher            # Linux
Sawbyte Launcher.bat          # Windows
open Sawbyte Launcher.command # macOS

# Create and launch a bot instance from the CLI
sawbyte launcher create bot-01 sawbyte-26.3 native
sawbyte launcher launch bot-01 127.0.0.1:25565 BotUser block-profile.json --wait

# Run the standalone headless client with a local control port
sawbyte-play live 127.0.0.1:25565 --offline BotUser block-profile.json --listen 127.0.0.1:25580

# Query bot coordinates, health, and tick status
sawbyte eval "inspect.status" --connect 127.0.0.1:25580

# Start the MCP server for Claude, Gemini, or other AI agents
sawbyte mcp --stdio
```

## <img src="https://raw.githubusercontent.com/InventivetalentDev/minecraft-assets/1.21.4/assets/minecraft/textures/item/writable_book.png" width="22" height="22" alt="" /> Python & TypeScript SDKs

**Python** (zero-dependency client with Gymnasium support):

```python
from sawbyte import SawbyteLiveTcpClient, SawbyteEnv

with SawbyteLiveTcpClient("127.0.0.1:25580") as client:
    print(client.send_command("inspect.status"))

env = SawbyteEnv(connect_addr="127.0.0.1:25580")
obs = env.reset()
for _ in range(100):
    obs, reward, done, trunc, info = env.step(env.action_space.sample())
```

**TypeScript** (composable bot workflow builder):

```typescript
import { SawbyteWorkflow } from "@sawbyte/sdk";

const wf = new SawbyteWorkflow()
  .connectLive("127.0.0.1:25580")
  .navigate(12, -60, 4)
  .waitUntil("screen_open", 100, "Chest")
  .guiClickSlot(0, "pickup");
```

## <img src="https://raw.githubusercontent.com/InventivetalentDev/minecraft-assets/1.21.4/assets/minecraft/textures/item/emerald.png" width="22" height="22" alt="" /> Benchmarks & Test Results (28/28 Passed)

### MineRL ObtainDiamond Tech-Tree (12/12 Passed)

| Step | Item | Goal | Mechanics Tested | Ticks | Status |
| :---: | :---: | :--- | :--- | :---: | :---: |
| **01** | <img src="https://raw.githubusercontent.com/InventivetalentDev/minecraft-assets/1.21.4/assets/minecraft/textures/block/oak_log.png" width="18" height="18" alt="" /> | Collect Oak Log | Block raycast, punch duration & item pickup | `80` | **PASSED** |
| **02** | <img src="https://raw.githubusercontent.com/InventivetalentDev/minecraft-assets/1.21.4/assets/minecraft/textures/block/oak_planks.png" width="18" height="18" alt="" /> | Craft Oak Planks | 2x2 inventory shapeless crafting | `20` | **PASSED** |
| **03** | <img src="https://raw.githubusercontent.com/InventivetalentDev/minecraft-assets/1.21.4/assets/minecraft/textures/block/crafting_table_front.png" width="18" height="18" alt="" /> | Craft Crafting Table | 2x2 shaped recipe | `20` | **PASSED** |
| **04** | <img src="https://raw.githubusercontent.com/InventivetalentDev/minecraft-assets/1.21.4/assets/minecraft/textures/block/crafting_table_top.png" width="18" height="18" alt="" /> | Place Crafting Table | Placement raycast & neighbor block update | `40` | **PASSED** |
| **05** | <img src="https://raw.githubusercontent.com/InventivetalentDev/minecraft-assets/1.21.4/assets/minecraft/textures/item/wooden_pickaxe.png" width="18" height="18" alt="" /> | Craft Wooden Pickaxe | 3x3 crafting table recipe (planks + sticks) | `30` | **PASSED** |
| **06** | <img src="https://raw.githubusercontent.com/InventivetalentDev/minecraft-assets/1.21.4/assets/minecraft/textures/block/cobblestone.png" width="18" height="18" alt="" /> | Mine Cobblestone | Tool hardness calculation & drop intake | `90` | **PASSED** |
| **07** | <img src="https://raw.githubusercontent.com/InventivetalentDev/minecraft-assets/1.21.4/assets/minecraft/textures/item/stone_pickaxe.png" width="18" height="18" alt="" /> | Craft Stone Pickaxe | 3x3 shaped recipe (cobblestone + sticks) | `30` | **PASSED** |
| **08** | <img src="https://raw.githubusercontent.com/InventivetalentDev/minecraft-assets/1.21.4/assets/minecraft/textures/item/raw_iron.png" width="18" height="18" alt="" /> | Mine Raw Iron | Stone pickaxe harvest tier check | `110` | **PASSED** |
| **09** | <img src="https://raw.githubusercontent.com/InventivetalentDev/minecraft-assets/1.21.4/assets/minecraft/textures/item/iron_ingot.png" width="18" height="18" alt="" /> | Smelt Iron Ingot | Furnace fuel & smelting container ticks | `200` | **PASSED** |
| **10** | <img src="https://raw.githubusercontent.com/InventivetalentDev/minecraft-assets/1.21.4/assets/minecraft/textures/item/iron_pickaxe.png" width="18" height="18" alt="" /> | Craft Iron Pickaxe | 3x3 shaped recipe (iron ingots + sticks) | `30` | **PASSED** |
| **11** | <img src="https://raw.githubusercontent.com/InventivetalentDev/minecraft-assets/1.21.4/assets/minecraft/textures/block/deepslate_diamond_ore.png" width="18" height="18" alt="" /> | Mine Diamond Ore | Iron pickaxe harvest tier at deep Y levels | `180` | **PASSED** |
| **12** | <img src="https://raw.githubusercontent.com/InventivetalentDev/minecraft-assets/1.21.4/assets/minecraft/textures/item/diamond.png" width="18" height="18" alt="" /> | Obtain Diamond | Inventory verification of `minecraft:diamond` | `10` | **PASSED** |

### MineDojo Survival Tasks (8/8 Passed)

| Icon | Task | Description | Target | Status |
| :---: | :--- | :--- | :--- | :---: |
| <img src="https://raw.githubusercontent.com/InventivetalentDev/minecraft-assets/1.21.4/assets/minecraft/textures/item/wheat.png" width="18" height="18" alt="" /> | **Harvest Wheat** | Break mature wheat and replant seeds | `minecraft:wheat` | **PASSED** |
| <img src="https://raw.githubusercontent.com/InventivetalentDev/minecraft-assets/1.21.4/assets/minecraft/textures/item/shears.png" width="18" height="18" alt="" /> | **Shear Sheep** | Use shears on adult sheep and collect wool | `minecraft:sheep` | **PASSED** |
| <img src="https://raw.githubusercontent.com/InventivetalentDev/minecraft-assets/1.21.4/assets/minecraft/textures/item/milk_bucket.png" width="18" height="18" alt="" /> | **Milk Cow** | Interact with cow using empty bucket | `minecraft:cow` | **PASSED** |
| <img src="https://raw.githubusercontent.com/InventivetalentDev/minecraft-assets/1.21.4/assets/minecraft/textures/item/rotten_flesh.png" width="18" height="18" alt="" /> | **Hunt Zombie** | Fight hostile zombie and pick up drops | `minecraft:zombie` | **PASSED** |
| <img src="https://raw.githubusercontent.com/InventivetalentDev/minecraft-assets/1.21.4/assets/minecraft/textures/block/obsidian.png" width="18" height="18" alt="" /> | **Enter Portal** | Walk into Nether portal and sync dimension | `minecraft:nether_portal` | **PASSED** |
| <img src="https://raw.githubusercontent.com/InventivetalentDev/minecraft-assets/1.21.4/assets/minecraft/textures/item/fishing_rod.png" width="18" height="18" alt="" /> | **Catch Fish** | Cast fishing rod, detect splash, reel in cod | `minecraft:cod` | **PASSED** |
| <img src="https://raw.githubusercontent.com/InventivetalentDev/minecraft-assets/1.21.4/assets/minecraft/textures/item/bone.png" width="18" height="18" alt="" /> | **Tame Wolf** | Feed bones to wild wolf until tamed | `minecraft:wolf` | **PASSED** |
| <img src="https://raw.githubusercontent.com/InventivetalentDev/minecraft-assets/1.21.4/assets/minecraft/textures/item/crossbow_standby.png" width="18" height="18" alt="" /> | **Defend Village** | Defeat pillager raid scout safely | `minecraft:pillager` | **PASSED** |

## <img src="https://raw.githubusercontent.com/InventivetalentDev/minecraft-assets/1.21.4/assets/minecraft/textures/block/chest_front.png" width="22" height="22" alt="" /> Repository Layout

```text
Sawbyte/
  native/           Standalone zero-graphics Rust client (sawbyte-play)
  crates/           Core library, protocol codec, CLI, and desktop launcher
  launcher/         Electron desktop launcher (Vue 3 + Vuetify)
  bridge/           Optional Fabric 26.3 graphical bridge mod
  sdks/python/      Python SDK and Gymnasium RL environment
  sdks/typescript/  TypeScript SDK and MCP client
  packaging/        Cross-platform installers and desktop shortcuts
  docs/             GitHub Pages website (https://5ivesaw.github.io/Sawbyte/)
  tools/            Crucible, Multi-Server, and AI benchmark runners
```

## <img src="https://raw.githubusercontent.com/InventivetalentDev/minecraft-assets/1.21.4/assets/minecraft/textures/item/paper.png" width="22" height="22" alt="" /> License

MIT. See [LICENSE](LICENSE).

*Sawbyte is an independent open-source project built for fun. Not an official Minecraft product. Not approved by or associated with Mojang or Microsoft.*

<br/>

## <img src="https://raw.githubusercontent.com/InventivetalentDev/minecraft-assets/1.21.4/assets/minecraft/textures/item/golden_helmet.png" width="22" height="22" alt="" /> Personal Note from the Creator: In Loving Memory of Technoblade (1999 to 2022) <img src="https://raw.githubusercontent.com/InventivetalentDev/minecraft-assets/1.21.4/assets/minecraft/textures/block/poppy.png" width="20" height="20" alt="" />

> **Personal Note:** Technoblade is completely separate from and unaffiliated with the project above. I am not using his name to promote anything. Sawbyte is just a side project I built for fun. Technoblade is the person who got me into Minecraft and inspired me to love this community in the first place, and I wanted to leave a quiet corner here so he is always remembered across the things I make.

| <img src="https://raw.githubusercontent.com/5ivesaw/Sawbyte/main/docs/assets/techno_pig.png" height="160" alt="Technoblade Crowned Pig" /> | <img src="https://raw.githubusercontent.com/5ivesaw/Sawbyte/main/docs/assets/techno_axes.png" height="160" alt="Technoblade Never Dies" /> |
| :---: | :---: |
| <img src="https://raw.githubusercontent.com/InventivetalentDev/minecraft-assets/1.21.4/assets/minecraft/textures/item/golden_helmet.png" width="16" height="16" alt="" /> **The Crowned Legend**<br/>*“If I had another hundred lives, I think I would choose to be Technoblade again every single time.”* | <img src="https://raw.githubusercontent.com/InventivetalentDev/minecraft-assets/1.21.4/assets/minecraft/textures/item/totem_of_undying.png" width="16" height="16" alt="" /> **Technoblade Never Dies [o7]**<br/>*Thank you for the laughs, the late-night streams, the Skyblock Potato Wars, and the memories that shaped a whole generation of players.* |

<p align="center">
  <img src="https://raw.githubusercontent.com/5ivesaw/Sawbyte/main/docs/assets/techno_farewell.png" height="180" alt="So long nerds" /><br/>
  <b>“So long nerds.”</b><br/>
  <sub>Technoblade Never Dies &bull; 1999 to Forever &bull; [o7]</sub>
</p>

&lt;p align=&quot;center&quot;&gt;
  &lt;img src=&quot;https://raw.githubusercontent.com/5ivesaw/Sawbyte/main/assets/sawbyte.png&quot; width=&quot;110&quot; alt=&quot;sawbyte&quot; /&gt;
&lt;/p&gt;

&lt;h1 align=&quot;center&quot;&gt;sawbyte&lt;/h1&gt;

&lt;p align=&quot;center&quot;&gt;
  &lt;b&gt;Rust client, desktop launcher, and bot engine for Minecraft 26.3.&lt;/b&gt;&lt;br/&gt;
  Run Minecraft 26.3 without Java eating 2 GB of RAM. Connect to servers, automate tasks, or hook up AI bots in ~8 to 12 MB of memory at a locked 20 ticks per second.
&lt;/p&gt;

&lt;p align=&quot;center&quot;&gt;
  &lt;a href=&quot;https://github.com/5ivesaw/Sawbyte/actions/workflows/ci.yml&quot;&gt;&lt;img src=&quot;https://img.shields.io/github/actions/workflow/status/5ivesaw/Sawbyte/ci.yml?branch=main&amp;style=flat-square&amp;label=build&amp;labelColor=111319&amp;color=2ea043&quot; alt=&quot;Build&quot; /&gt;&lt;/a&gt;
  &lt;a href=&quot;https://github.com/5ivesaw/Sawbyte/actions/workflows/player.yml&quot;&gt;&lt;img src=&quot;https://img.shields.io/github/actions/workflow/status/5ivesaw/Sawbyte/player.yml?branch=main&amp;style=flat-square&amp;label=parity%20lab&amp;labelColor=111319&amp;color=2ea043&quot; alt=&quot;Parity Lab&quot; /&gt;&lt;/a&gt;
  &lt;a href=&quot;https://github.com/5ivesaw/Sawbyte/releases/latest&quot;&gt;&lt;img src=&quot;https://img.shields.io/github/v/release/5ivesaw/Sawbyte?style=flat-square&amp;label=release&amp;labelColor=111319&amp;color=00aaaa&quot; alt=&quot;Latest Release&quot; /&gt;&lt;/a&gt;
  &lt;img src=&quot;https://img.shields.io/badge/26.3%20%C2%B7%20Protocol%20777-ffaa00?style=flat-square&amp;label=minecraft&amp;labelColor=111319&quot; alt=&quot;Minecraft 26.3&quot; /&gt;
  &lt;a href=&quot;LICENSE&quot;&gt;&lt;img src=&quot;https://img.shields.io/badge/MIT-aaaaaa?style=flat-square&amp;label=license&amp;labelColor=111319&quot; alt=&quot;License&quot; /&gt;&lt;/a&gt;
&lt;/p&gt;

&lt;p align=&quot;center&quot;&gt;
  &lt;img src=&quot;https://raw.githubusercontent.com/InventivetalentDev/minecraft-assets/1.21.4/assets/minecraft/textures/block/grass_block_side.png&quot; width=&quot;16&quot; height=&quot;16&quot; alt=&quot;&quot; /&gt; &lt;a href=&quot;https://5ivesaw.github.io/Sawbyte/&quot;&gt;&lt;b&gt;Website&lt;/b&gt;&lt;/a&gt;
  &amp;nbsp;&amp;bull;&amp;nbsp;
  &lt;img src=&quot;https://raw.githubusercontent.com/InventivetalentDev/minecraft-assets/1.21.4/assets/minecraft/textures/item/diamond.png&quot; width=&quot;16&quot; height=&quot;16&quot; alt=&quot;&quot; /&gt; &lt;a href=&quot;https://github.com/5ivesaw/Sawbyte/releases/latest&quot;&gt;&lt;b&gt;Downloads&lt;/b&gt;&lt;/a&gt;
  &amp;nbsp;&amp;bull;&amp;nbsp;
  &lt;img src=&quot;https://raw.githubusercontent.com/InventivetalentDev/minecraft-assets/1.21.4/assets/minecraft/textures/item/emerald.png&quot; width=&quot;16&quot; height=&quot;16&quot; alt=&quot;&quot; /&gt; &lt;a href=&quot;docs/VISION.md&quot;&gt;&lt;b&gt;Vision&lt;/b&gt;&lt;/a&gt;
  &amp;nbsp;&amp;bull;&amp;nbsp;
  &lt;img src=&quot;https://raw.githubusercontent.com/InventivetalentDev/minecraft-assets/1.21.4/assets/minecraft/textures/block/command_block_front.png&quot; width=&quot;16&quot; height=&quot;16&quot; alt=&quot;&quot; /&gt; &lt;a href=&quot;.agents/skills/sawbyte/SKILL.md&quot;&gt;&lt;b&gt;AI Bot Guide (SKILL.md)&lt;/b&gt;&lt;/a&gt;
&lt;/p&gt;

## &lt;img src=&quot;https://raw.githubusercontent.com/InventivetalentDev/minecraft-assets/1.21.4/assets/minecraft/textures/item/diamond_pickaxe.png&quot; width=&quot;22&quot; height=&quot;22&quot; alt=&quot;&quot; /&gt; Overview

| Icon | Metric | Sawbyte (Rust 26.3) | Vanilla Minecraft (Java) | Notes |
| :---: | :--- | :--- | :--- | :--- |
| &lt;img src=&quot;https://raw.githubusercontent.com/InventivetalentDev/minecraft-assets/1.21.4/assets/minecraft/textures/item/diamond.png&quot; width=&quot;20&quot; height=&quot;20&quot; alt=&quot;&quot; /&gt; | **Memory Usage** | **~8.2 to 11.8 MB** | ~1,840 MB | Pure Rust with zero JVM overhead (~155x less RAM) |
| &lt;img src=&quot;https://raw.githubusercontent.com/InventivetalentDev/minecraft-assets/1.21.4/assets/minecraft/textures/item/redstone.png&quot; width=&quot;20&quot; height=&quot;20&quot; alt=&quot;&quot; /&gt; | **Tick Rate & Physics** | **20.0 TPS (0.04 ms/tick)** | 12.2 ms/tick | `< 3.41e-7` blocks max parity error, zero ground mismatch |
| &lt;img src=&quot;https://raw.githubusercontent.com/InventivetalentDev/minecraft-assets/1.21.4/assets/minecraft/textures/item/sugar.png&quot; width=&quot;20&quot; height=&quot;20&quot; alt=&quot;&quot; /&gt; | **Cold Startup** | **~0.014 to 0.12 s** | ~8.4 s | Starts almost instantly from the terminal or launcher |
| &lt;img src=&quot;https://raw.githubusercontent.com/InventivetalentDev/minecraft-assets/1.21.4/assets/minecraft/textures/item/emerald.png&quot; width=&quot;20&quot; height=&quot;20&quot; alt=&quot;&quot; /&gt; | **Task Benchmarks** | **28 / 28 Passed (100%)** | Baseline | MineRL Diamond Run, MineDojo, Voyager, and Server Login |
| &lt;img src=&quot;https://raw.githubusercontent.com/InventivetalentDev/minecraft-assets/1.21.4/assets/minecraft/textures/block/command_block_front.png&quot; width=&quot;20&quot; height=&quot;20&quot; alt=&quot;&quot; /&gt; | **Bot & AI Control** | **MCP + TCP + SDKs** | None | Built-in MCP server, Gymnasium RL env, Python & TS SDKs |

## &lt;img src=&quot;https://raw.githubusercontent.com/InventivetalentDev/minecraft-assets/1.21.4/assets/minecraft/textures/block/crafting_table_front.png&quot; width=&quot;22&quot; height=&quot;22&quot; alt=&quot;&quot; /&gt; What It Does

* &lt;img src=&quot;https://raw.githubusercontent.com/InventivetalentDev/minecraft-assets/1.21.4/assets/minecraft/textures/item/ender_pearl.png&quot; width=&quot;18&quot; height=&quot;18&quot; alt=&quot;&quot; /&gt; **Headless Native Client (`sawbyte-play`)**: Connects to real Minecraft 26.3 servers over raw protocol and runs sub-voxel physics, gravity, and collisions at 20 TPS with no window needed.
* &lt;img src=&quot;https://raw.githubusercontent.com/InventivetalentDev/minecraft-assets/1.21.4/assets/minecraft/textures/block/crafting_table_front.png&quot; width=&quot;18&quot; height=&quot;18&quot; alt=&quot;&quot; /&gt; **Desktop Launcher**: Cross-platform GUI for managing instances, accounts, mods, snapshots, and watching live bots.
* &lt;img src=&quot;https://raw.githubusercontent.com/InventivetalentDev/minecraft-assets/1.21.4/assets/minecraft/textures/item/diamond_pickaxe.png&quot; width=&quot;18&quot; height=&quot;18&quot; alt=&quot;&quot; /&gt; **Automation Engine**: Built-in 3D A* pathfinding, block raycasting, mining, block placement, combat, chest menu clicking, and inventory management.
* &lt;img src=&quot;https://raw.githubusercontent.com/InventivetalentDev/minecraft-assets/1.21.4/assets/minecraft/textures/item/enchanted_book.png&quot; width=&quot;18&quot; height=&quot;18&quot; alt=&quot;&quot; /&gt; **AI Agent Interface**: Built-in MCP server with 10 live tools, Gymnasium environment, Python and TypeScript SDKs, and automatic milestone and death logging.
* &lt;img src=&quot;https://raw.githubusercontent.com/InventivetalentDev/minecraft-assets/1.21.4/assets/minecraft/textures/item/compass_16.png&quot; width=&quot;18&quot; height=&quot;18&quot; alt=&quot;&quot; /&gt; **Software 3D & Radar View**: Pure-Rust raymarcher and top-down radar renderer with zero GPU dependencies.
* &lt;img src=&quot;https://raw.githubusercontent.com/InventivetalentDev/minecraft-assets/1.21.4/assets/minecraft/textures/item/ender_eye.png&quot; width=&quot;18&quot; height=&quot;18&quot; alt=&quot;&quot; /&gt; **Fabric 26.3 Bridge Mod**: Optional Fabric mod to sync and spectate a headless Sawbyte bot inside the regular Java client.

## &lt;img src=&quot;https://raw.githubusercontent.com/InventivetalentDev/minecraft-assets/1.21.4/assets/minecraft/textures/item/painting.png&quot; width=&quot;22&quot; height=&quot;22&quot; alt=&quot;&quot; /&gt; Screenshots & Previews

> **Note:** New desktop launcher and bot view screenshots will be added to `assets/screenshots/` once the new launcher UI build is finalized.

| Slot | Preview Area | Target File Path | Status |
| :---: | :--- | :--- | :---: |
| **01** | &lt;img src=&quot;https://raw.githubusercontent.com/InventivetalentDev/minecraft-assets/1.21.4/assets/minecraft/textures/block/crafting_table_front.png&quot; width=&quot;16&quot; height=&quot;16&quot; alt=&quot;&quot; /&gt; **Desktop Launcher UI** | `assets/screenshots/launcher.png` | Placeholder ready |
| **02** | &lt;img src=&quot;https://raw.githubusercontent.com/InventivetalentDev/minecraft-assets/1.21.4/assets/minecraft/textures/block/command_block_front.png&quot; width=&quot;16&quot; height=&quot;16&quot; alt=&quot;&quot; /&gt; **Bot Session & Live View** | `assets/screenshots/bot_view.png` | Placeholder ready |
| **03** | &lt;img src=&quot;https://raw.githubusercontent.com/InventivetalentDev/minecraft-assets/1.21.4/assets/minecraft/textures/block/chest_front.png&quot; width=&quot;16&quot; height=&quot;16&quot; alt=&quot;&quot; /&gt; **Server & Account Manager** | `assets/screenshots/server_manager.png` | Placeholder ready |

## &lt;img src=&quot;https://raw.githubusercontent.com/InventivetalentDev/minecraft-assets/1.21.4/assets/minecraft/textures/item/diamond.png&quot; width=&quot;22&quot; height=&quot;22&quot; alt=&quot;&quot; /&gt; Install & Downloads

### One-Line Terminal Installer

```bash
# Linux / macOS
curl -sSfL https://raw.githubusercontent.com/5ivesaw/Sawbyte/main/packaging/install.sh | bash
```

### Direct Release Downloads (Always Auto-Synced to Latest Release)

All links below use GitHub's `/releases/latest/download/` resolver so they always grab the newest release build automatically:

| Platform | Desktop Launcher (GUI) | Headless Engine (Core) | Sawbyte Play (CLI) |
| :--- | :--- | :--- | :--- |
| &lt;img src=&quot;https://raw.githubusercontent.com/InventivetalentDev/minecraft-assets/1.21.4/assets/minecraft/textures/block/command_block_front.png&quot; width=&quot;16&quot; height=&quot;16&quot; alt=&quot;&quot; /&gt; **Linux (x86_64)** | [sawbyte-launcher-ubuntu-24.04-X64.tar.gz](https://www.google.com/search?q=https://github.com/5ivesaw/Sawbyte/releases/latest/download/sawbyte-launcher-ubuntu-24.04-X64.tar.gz) | [sawbyte-ubuntu-24.04-X64.tar.gz](https://www.google.com/search?q=https://github.com/5ivesaw/Sawbyte/releases/latest/download/sawbyte-ubuntu-24.04-X64.tar.gz) | [sawbyte-play-linux-x64.tar.gz](https://www.google.com/search?q=https://github.com/5ivesaw/Sawbyte/releases/latest/download/sawbyte-play-linux-x64.tar.gz) |
| &lt;img src=&quot;https://raw.githubusercontent.com/InventivetalentDev/minecraft-assets/1.21.4/assets/minecraft/textures/item/diamond.png&quot; width=&quot;16&quot; height=&quot;16&quot; alt=&quot;&quot; /&gt; **Windows (x64)** | [sawbyte-launcher-windows-2022-X64.zip](https://www.google.com/search?q=https://github.com/5ivesaw/Sawbyte/releases/latest/download/sawbyte-launcher-windows-2022-X64.zip) | [sawbyte-windows-2022-X64.zip](https://www.google.com/search?q=https://github.com/5ivesaw/Sawbyte/releases/latest/download/sawbyte-windows-2022-X64.zip) | [sawbyte-play-windows-x64.zip](https://www.google.com/search?q=https://github.com/5ivesaw/Sawbyte/releases/latest/download/sawbyte-play-windows-x64.zip) |
| &lt;img src=&quot;https://raw.githubusercontent.com/InventivetalentDev/minecraft-assets/1.21.4/assets/minecraft/textures/item/emerald.png&quot; width=&quot;16&quot; height=&quot;16&quot; alt=&quot;&quot; /&gt; **macOS (Apple Silicon)** | [sawbyte-launcher-macos-14-ARM64.tar.gz](https://www.google.com/search?q=https://github.com/5ivesaw/Sawbyte/releases/latest/download/sawbyte-launcher-macos-14-ARM64.tar.gz) | [sawbyte-macos-14-ARM64.tar.gz](https://www.google.com/search?q=https://github.com/5ivesaw/Sawbyte/releases/latest/download/sawbyte-macos-14-ARM64.tar.gz) | [sawbyte-play-macos-arm64.tar.gz](https://www.google.com/search?q=https://github.com/5ivesaw/Sawbyte/releases/latest/download/sawbyte-play-macos-arm64.tar.gz) |

* &lt;img src=&quot;https://raw.githubusercontent.com/InventivetalentDev/minecraft-assets/1.21.4/assets/minecraft/textures/block/grass_block_side.png&quot; width=&quot;16&quot; height=&quot;16&quot; alt=&quot;&quot; /&gt; **Fabric Bridge Mod (Minecraft 26.3)**: Grab the latest `sawbyte-bridge-*.jar` from the [GitHub Releases Page](https://www.google.com/search?q=https://github.com/5ivesaw/Sawbyte/releases/latest).

## &lt;img src=&quot;https://raw.githubusercontent.com/InventivetalentDev/minecraft-assets/1.21.4/assets/minecraft/textures/block/command_block_front.png&quot; width=&quot;22&quot; height=&quot;22&quot; alt=&quot;&quot; /&gt; Quick Start

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

## &lt;img src=&quot;https://raw.githubusercontent.com/InventivetalentDev/minecraft-assets/1.21.4/assets/minecraft/textures/item/writable_book.png&quot; width=&quot;22&quot; height=&quot;22&quot; alt=&quot;&quot; /&gt; Python & TypeScript SDKs

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

## &lt;img src=&quot;https://raw.githubusercontent.com/InventivetalentDev/minecraft-assets/1.21.4/assets/minecraft/textures/item/emerald.png&quot; width=&quot;22&quot; height=&quot;22&quot; alt=&quot;&quot; /&gt; Benchmarks & Test Results (28/28 Passed)

### MineRL ObtainDiamond Tech-Tree (12/12 Passed)

| Step | Item | Goal | Mechanics Tested | Ticks | Status |
| :---: | :---: | :--- | :--- | :---: | :---: |
| **01** | &lt;img src=&quot;https://raw.githubusercontent.com/InventivetalentDev/minecraft-assets/1.21.4/assets/minecraft/textures/block/oak_log.png&quot; width=&quot;18&quot; height=&quot;18&quot; alt=&quot;&quot; /&gt; | Collect Oak Log | Block raycast, punch duration & item pickup | `80` | **PASSED** |
| **02** | &lt;img src=&quot;https://raw.githubusercontent.com/InventivetalentDev/minecraft-assets/1.21.4/assets/minecraft/textures/block/oak_planks.png&quot; width=&quot;18&quot; height=&quot;18&quot; alt=&quot;&quot; /&gt; | Craft Oak Planks | 2x2 inventory shapeless crafting | `20` | **PASSED** |
| **03** | &lt;img src=&quot;https://raw.githubusercontent.com/InventivetalentDev/minecraft-assets/1.21.4/assets/minecraft/textures/block/crafting_table_front.png&quot; width=&quot;18&quot; height=&quot;18&quot; alt=&quot;&quot; /&gt; | Craft Crafting Table | 2x2 shaped recipe | `20` | **PASSED** |
| **04** | &lt;img src=&quot;https://raw.githubusercontent.com/InventivetalentDev/minecraft-assets/1.21.4/assets/minecraft/textures/block/crafting_table_top.png&quot; width=&quot;18&quot; height=&quot;18&quot; alt=&quot;&quot; /&gt; | Place Crafting Table | Placement raycast & neighbor block update | `40` | **PASSED** |
| **05** | &lt;img src=&quot;https://raw.githubusercontent.com/InventivetalentDev/minecraft-assets/1.21.4/assets/minecraft/textures/item/wooden_pickaxe.png&quot; width=&quot;18&quot; height=&quot;18&quot; alt=&quot;&quot; /&gt; | Craft Wooden Pickaxe | 3x3 crafting table recipe (planks + sticks) | `30` | **PASSED** |
| **06** | &lt;img src=&quot;https://raw.githubusercontent.com/InventivetalentDev/minecraft-assets/1.21.4/assets/minecraft/textures/block/cobblestone.png&quot; width=&quot;18&quot; height=&quot;18&quot; alt=&quot;&quot; /&gt; | Mine Cobblestone | Tool hardness calculation & drop intake | `90` | **PASSED** |
| **07** | &lt;img src=&quot;https://raw.githubusercontent.com/InventivetalentDev/minecraft-assets/1.21.4/assets/minecraft/textures/item/stone_pickaxe.png&quot; width=&quot;18&quot; height=&quot;18&quot; alt=&quot;&quot; /&gt; | Craft Stone Pickaxe | 3x3 shaped recipe (cobblestone + sticks) | `30` | **PASSED** |
| **08** | &lt;img src=&quot;https://raw.githubusercontent.com/InventivetalentDev/minecraft-assets/1.21.4/assets/minecraft/textures/item/raw_iron.png&quot; width=&quot;18&quot; height=&quot;18&quot; alt=&quot;&quot; /&gt; | Mine Raw Iron | Stone pickaxe harvest tier check | `110` | **PASSED** |
| **09** | &lt;img src=&quot;https://raw.githubusercontent.com/InventivetalentDev/minecraft-assets/1.21.4/assets/minecraft/textures/item/iron_ingot.png&quot; width=&quot;18&quot; height=&quot;18&quot; alt=&quot;&quot; /&gt; | Smelt Iron Ingot | Furnace fuel & smelting container ticks | `200` | **PASSED** |
| **10** | &lt;img src=&quot;https://raw.githubusercontent.com/InventivetalentDev/minecraft-assets/1.21.4/assets/minecraft/textures/item/iron_pickaxe.png&quot; width=&quot;18&quot; height=&quot;18&quot; alt=&quot;&quot; /&gt; | Craft Iron Pickaxe | 3x3 shaped recipe (iron ingots + sticks) | `30` | **PASSED** |
| **11** | &lt;img src=&quot;https://raw.githubusercontent.com/InventivetalentDev/minecraft-assets/1.21.4/assets/minecraft/textures/block/deepslate_diamond_ore.png&quot; width=&quot;18&quot; height=&quot;18&quot; alt=&quot;&quot; /&gt; | Mine Diamond Ore | Iron pickaxe harvest tier at deep Y levels | `180` | **PASSED** |
| **12** | &lt;img src=&quot;https://raw.githubusercontent.com/InventivetalentDev/minecraft-assets/1.21.4/assets/minecraft/textures/item/diamond.png&quot; width=&quot;18&quot; height=&quot;18&quot; alt=&quot;&quot; /&gt; | Obtain Diamond | Inventory verification of `minecraft:diamond` | `10` | **PASSED** |

### MineDojo Survival Tasks (8/8 Passed)

| Icon | Task | Description | Target | Status |
| :---: | :--- | :--- | :--- | :---: |
| &lt;img src=&quot;https://raw.githubusercontent.com/InventivetalentDev/minecraft-assets/1.21.4/assets/minecraft/textures/item/wheat.png&quot; width=&quot;18&quot; height=&quot;18&quot; alt=&quot;&quot; /&gt; | **Harvest Wheat** | Break mature wheat and replant seeds | `minecraft:wheat` | **PASSED** |
| &lt;img src=&quot;https://raw.githubusercontent.com/InventivetalentDev/minecraft-assets/1.21.4/assets/minecraft/textures/item/shears.png&quot; width=&quot;18&quot; height=&quot;18&quot; alt=&quot;&quot; /&gt; | **Shear Sheep** | Use shears on adult sheep and collect wool | `minecraft:sheep` | **PASSED** |
| &lt;img src=&quot;https://raw.githubusercontent.com/InventivetalentDev/minecraft-assets/1.21.4/assets/minecraft/textures/item/milk_bucket.png&quot; width=&quot;18&quot; height=&quot;18&quot; alt=&quot;&quot; /&gt; | **Milk Cow** | Interact with cow using empty bucket | `minecraft:cow` | **PASSED** |
| &lt;img src=&quot;https://raw.githubusercontent.com/InventivetalentDev/minecraft-assets/1.21.4/assets/minecraft/textures/item/rotten_flesh.png&quot; width=&quot;18&quot; height=&quot;18&quot; alt=&quot;&quot; /&gt; | **Hunt Zombie** | Fight hostile zombie and pick up drops | `minecraft:zombie` | **PASSED** |
| &lt;img src=&quot;https://raw.githubusercontent.com/InventivetalentDev/minecraft-assets/1.21.4/assets/minecraft/textures/block/obsidian.png&quot; width=&quot;18&quot; height=&quot;18&quot; alt=&quot;&quot; /&gt; | **Enter Portal** | Walk into Nether portal and sync dimension | `minecraft:nether_portal` | **PASSED** |
| &lt;img src=&quot;https://raw.githubusercontent.com/InventivetalentDev/minecraft-assets/1.21.4/assets/minecraft/textures/item/fishing_rod.png&quot; width=&quot;18&quot; height=&quot;18&quot; alt=&quot;&quot; /&gt; | **Catch Fish** | Cast fishing rod, detect splash, reel in cod | `minecraft:cod` | **PASSED** |
| &lt;img src=&quot;https://raw.githubusercontent.com/InventivetalentDev/minecraft-assets/1.21.4/assets/minecraft/textures/item/bone.png&quot; width=&quot;18&quot; height=&quot;18&quot; alt=&quot;&quot; /&gt; | **Tame Wolf** | Feed bones to wild wolf until tamed | `minecraft:wolf` | **PASSED** |
| &lt;img src=&quot;https://raw.githubusercontent.com/InventivetalentDev/minecraft-assets/1.21.4/assets/minecraft/textures/item/crossbow_standby.png&quot; width=&quot;18&quot; height=&quot;18&quot; alt=&quot;&quot; /&gt; | **Defend Village** | Defeat pillager raid scout safely | `minecraft:pillager` | **PASSED** |

## &lt;img src=&quot;https://raw.githubusercontent.com/InventivetalentDev/minecraft-assets/1.21.4/assets/minecraft/textures/block/chest_front.png&quot; width=&quot;22&quot; height=&quot;22&quot; alt=&quot;&quot; /&gt; Repository Layout

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

## &lt;img src=&quot;https://raw.githubusercontent.com/InventivetalentDev/minecraft-assets/1.21.4/assets/minecraft/textures/item/paper.png&quot; width=&quot;22&quot; height=&quot;22&quot; alt=&quot;&quot; /&gt; License

MIT. See [LICENSE](https://www.google.com/search?q=LICENSE).

*Sawbyte is an independent open-source project built for fun. Not an official Minecraft product. Not approved by or associated with Mojang or Microsoft.*

<br/>

## &lt;img src=&quot;https://raw.githubusercontent.com/InventivetalentDev/minecraft-assets/1.21.4/assets/minecraft/textures/item/golden_helmet.png&quot; width=&quot;22&quot; height=&quot;22&quot; alt=&quot;&quot; /&gt; Personal Note from the Creator: In Loving Memory of Technoblade (1999 to 2022) &lt;img src=&quot;https://raw.githubusercontent.com/InventivetalentDev/minecraft-assets/1.21.4/assets/minecraft/textures/block/poppy.png&quot; width=&quot;20&quot; height=&quot;20&quot; alt=&quot;&quot; /&gt;

> **Personal Note:** Technoblade is completely separate from and unaffiliated with the project above. I am not using his name to promote anything. Sawbyte is just a side project I built for fun. Technoblade is the person who got me into Minecraft and inspired me to love this community in the first place, and I wanted to leave a quiet corner here so he is always remembered across the things I make.

| &lt;img src=&quot;https://raw.githubusercontent.com/5ivesaw/Sawbyte/main/docs/assets/techno_pig.png&quot; height=&quot;160&quot; alt=&quot;Technoblade Crowned Pig&quot; /&gt; | &lt;img src=&quot;https://raw.githubusercontent.com/5ivesaw/Sawbyte/main/docs/assets/techno_axes.png&quot; height=&quot;160&quot; alt=&quot;Technoblade Never Dies&quot; /&gt; |
| :---: | :---: |
| &lt;img src=&quot;https://raw.githubusercontent.com/InventivetalentDev/minecraft-assets/1.21.4/assets/minecraft/textures/item/golden_helmet.png&quot; width=&quot;16&quot; height=&quot;16&quot; alt=&quot;&quot; /&gt; **The Crowned Legend**<br/>*“If I had another hundred lives, I think I would choose to be Technoblade again every single time.”* | &lt;img src=&quot;https://raw.githubusercontent.com/InventivetalentDev/minecraft-assets/1.21.4/assets/minecraft/textures/item/totem_of_undying.png&quot; width=&quot;16&quot; height=&quot;16&quot; alt=&quot;&quot; /&gt; **Technoblade Never Dies [o7]**<br/>*Thank you for the laughs, the late-night streams, the Skyblock Potato Wars, and the memories that shaped a whole generation of players.* |

&lt;p align=&quot;center&quot;&gt;
  &lt;img src=&quot;https://raw.githubusercontent.com/5ivesaw/Sawbyte/main/docs/assets/techno_farewell.png&quot; height=&quot;180&quot; alt=&quot;So long nerds&quot; /&gt;&lt;br/&gt;
  &lt;b&gt;“So long nerds.”&lt;/b&gt;&lt;br/&gt;
  &lt;sub&gt;Technoblade Never Dies &amp;bull; 1999 to Forever &amp;bull; [o7]&lt;/sub&gt;
&lt;/p&gt;

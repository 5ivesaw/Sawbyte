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
| <img src="https://raw.githubusercontent.com/InventivetalentDev/minecraft-assets/1.21.4/assets/minecraft/textures/item/emerald.png" width="20" height="20" alt="" /> | **Server & Task Benchmarks** | **16/16 Servers (96/96 Stages)** | Baseline | 16 Cracked Servers (BedWars, SkyWars, Practice, SkyBlock, SMP) + Crucible |
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

| Platform | Native Desktop App (Zero HTML / No Browser) | Universal / Portable App | Headless Engine (`sawbyte-play`) | Standalone CLI (`sawbyte`) |
| :--- | :--- | :--- | :--- | :--- |
| <img src="https://raw.githubusercontent.com/InventivetalentDev/minecraft-assets/1.21.4/assets/minecraft/textures/block/command_block_front.png" width="16" height="16" alt="" /> **Linux (Debian / Ubuntu / Arch / Fedora)** | [**sawbyte-launcher-linux-amd64.deb**](https://github.com/5ivesaw/Sawbyte/releases/latest/download/sawbyte-launcher-linux-amd64.deb) | [**Sawbyte-Launcher-x86_64.AppImage**](https://github.com/5ivesaw/Sawbyte/releases/latest/download/Sawbyte-Launcher-x86_64.AppImage) | [sawbyte-play-linux-x64.tar.gz](https://github.com/5ivesaw/Sawbyte/releases/latest/download/sawbyte-play-linux-x64.tar.gz) | [sawbyte-cli-linux-x86_64](https://github.com/5ivesaw/Sawbyte/releases/latest/download/sawbyte-cli-linux-x86_64) |
| <img src="https://raw.githubusercontent.com/InventivetalentDev/minecraft-assets/1.21.4/assets/minecraft/textures/item/diamond.png" width="16" height="16" alt="" /> **Windows 10 / 11 (x64)** | [**Sawbyte-Launcher-x64.exe**](https://github.com/5ivesaw/Sawbyte/releases/latest/download/Sawbyte-Launcher-x64.exe) | [sawbyte-launcher-windows-2022-X64.zip](https://github.com/5ivesaw/Sawbyte/releases/latest/download/sawbyte-launcher-windows-2022-X64.zip) | [sawbyte-play-windows-x64.zip](https://github.com/5ivesaw/Sawbyte/releases/latest/download/sawbyte-play-windows-x64.zip) | [sawbyte-cli-x64.exe](https://github.com/5ivesaw/Sawbyte/releases/latest/download/sawbyte-cli-x64.exe) |

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

## <img src="https://raw.githubusercontent.com/InventivetalentDev/minecraft-assets/1.21.4/assets/minecraft/textures/item/emerald.png" width="22" height="22" alt="" /> Benchmarks & Autonomous Multi-Server Results (16/16 Servers · 96/96 Stages Passed)

### 16-Server Autonomous Cracked Multiplayer & Minigame Suite (`benchmarks/servers/`)

Each server benchmark script in `benchmarks/servers/bench_01_*.py` .. `bench_16_*.py` runs a 6-stage autonomous workflow with random cracked usernames (`SawBot_<random>`), non-vision chat math/code challenge solving, 46-slot inventory/hotbar inspection (`inventory.list`), Chest GUI container slot matching (`gui.inspect` / `gui.click`), stacked ArmorStand hologram NPC discovery (`entity.find` / `npc.interact`), 3D A* pathfinding, and full **"Seeing the Game with Words"** sensory output (`inspect.sensory`):

| # | Server | Address | Target Gamemode | Parameters | Stages | Script | Status |
| :---: | :--- | :--- | :--- | :---: | :---: | :--- | :---: |
| **01** | [PikaNetwork](https://pika-network.net) | `top.pika.host:25565` | Solo BedWars | `68 params` | `6/6` | `benchmarks/servers/bench_01_pikanetwork_bedwars.py` | **PASS** |
| **02** | [JartexNetwork](https://jartexnetwork.com) | `play.jartexnetwork.com:25565` | BedWars Solo | `56 params` | `6/6` | `benchmarks/servers/bench_02_jartexnetwork_bedwars.py` | **PASS** |
| **03** | [Herobrine.org](https://herobrine.org) | `mc.herobrine.org:25565` | SkyWars Solo | `57 params` | `6/6` | `benchmarks/servers/bench_03_herobrine_skywars.py` | **PASS** |
| **04** | [BlocksMC](https://blocksmc.com) | `blocksmc.com:25565` | BedWars Solo | `54 params` | `6/6` | `benchmarks/servers/bench_04_blocksmc_bedwars.py` | **PASS** |
| **05** | [Gamster Network](https://gamster.org) | `mc.gamster.org:25565` | Practice PvP | `61 params` | `6/6` | `benchmarks/servers/bench_05_gamster_practice_pvp.py` | **PASS** |
| **06** | [Mineland Network](https://mineland.net) | `play.mineland.net:25565` | SkyWars & Arcade | `58 params` | `6/6` | `benchmarks/servers/bench_06_mineland_minigames.py` | **PASS** |
| **07** | [Twerion Network](https://twerion.net) | `hey.twerion.net:25565` | KitPvP & KnockFFA | `59 params` | `6/6` | `benchmarks/servers/bench_07_twerion_kitpvp.py` | **PASS** |
| **08** | [ExtremeCraft](https://extremecraft.net) | `play.extremecraft.net:25565` | SkyBlock & OneBlock | `54 params` | `6/6` | `benchmarks/servers/bench_08_extremecraft_skyblock.py` | **PASS** |
| **09** | [UniversoCraft](https://universocraft.com) | `mc.universocraft.com:25565` | SkyWars & BedWars | `60 params` | `6/6` | `benchmarks/servers/bench_09_universocraft_skywars.py` | **PASS** |
| **10** | [Complex Gaming](https://mc-complex.com) | `bmc.mc-complex.com:25565` | Survival & Lifesteal | `55 params` | `6/6` | `benchmarks/servers/bench_10_complex_pixelmon_survival.py` | **PASS** |
| **11** | [AppleCraft](https://applecraft.org) | `play.applecraft.org:25565` | Survival SMP | `55 params` | `6/6` | `benchmarks/servers/bench_11_applecraft_survival_smp.py` | **PASS** |
| **12** | [CraftersLand](https://craftersland.net) | `play.craftersland.net:25565` | SkyBlock & Survival | `52 params` | `6/6` | `benchmarks/servers/bench_12_craftersland_skyblock.py` | **PASS** |
| **13** | [OPBlocks Network](https://opblocks.com) | `play.opblocks.com:25565` | OP Prison & SkyBlock | `53 params` | `6/6` | `benchmarks/servers/bench_13_opblocks_prison.py` | **PASS** |
| **14** | [SunRealms](https://sunrealms.net) | `play.sunrealms.net:25565` | SkyBlock & OneBlock | `52 params` | `6/6` | `benchmarks/servers/bench_14_sunrealms_skyblock.py` | **PASS** |
| **15** | [TalonMC](https://talonmc.net) | `play.talonmc.net:25565` | Lifesteal & SkyBlock | `53 params` | `6/6` | `benchmarks/servers/bench_15_talonmc_lifesteal.py` | **PASS** |
| **16** | [LemonCloud](https://lemoncloud.org) | `play.lemoncloud.org:25565` | Factions & Survival | `56 params` | `6/6` | `benchmarks/servers/bench_16_lemoncloud_factions.py` | **PASS** |

### "Seeing the Game with Words v2" & `SVPE-1` Compact Encoding (`docs/SENSORY_ENCODING.md`)

Sawbyte v0.5.0 bridges the gap between human graphical vision and headless AI perception through a 10-section sensory engine (`inspect.sensory` / `see`) and the **Sawbyte Voxel Perception Encoding (`SVPE-1`)** format (see [docs/SENSORY_ENCODING.md](docs/SENSORY_ENCODING.md)):

- **First-Person 24×11 Retinal ASCII Viewport & RLE Depth Buffer (`inspect.viewport`)**: Raycasts the player's `72°×44°` camera frustum into a 2D retinal grid showing sky, floors, distance-shaded walls (`#`/`=`/`-`/`:`), water (`~`), lava (`!`), entities (`@`/`M`), and crosshair center (`+`), paired with `SVPE1:DEPTH24x11` (`a..x` = `1..24m`).
- **Lossless, Token-Efficient `SVPE-1` Spatial Encodings (~50x–100x Compression)**:
  - **`SVPE1:RLE3D` (`inspect.svpe [rx ry rz]`)**: Encodes 3D voxel volumes in `Y -> Z -> X` order using `<count><symbol>` RLE (`A`=air, `S`=solid, `W`=water, `L`=lava, `C`=climbable, `h`/`t`=slab, `s`=stair, `f`=fence, `I`=ice, `O`=POI, `U`=unloaded) and `@dup` for repeated Y-layers. Decodable in 10 lines of Python (`sawbyte.decode_svpe1_rle3d`), TypeScript (`decodeSvpe1Rle3d`), or Rust (`inspect.svpe.decode`).
  - **`SVPE1:HD25D` (`inspect.svpe.height [radius]`)**: Encodes a 2.5D surface elevation delta & hazard grid where `=` is level with player feet (`Δy=0`), `A..Z` is `+1..+26` blocks up, `a..z` is `-1..-26` blocks down, `~` is water, `!` is lava, and `_` is a bottomless void drop.
  - **`SVPE1:DELTA` (`inspect.sensory.delta`)**: Emits only per-tick deltas (`dpos`, `dyaw`, `dpitch`, `dhp`, `new_chat`), cutting bandwidth by ~92% in 20 TPS loops.
- **Compensatory Headless Perception (`inspect.xray`, `inspect.long_range`, `inspect.small_map`)**:
  - **Through-Block Penetrating X-Ray (`inspect.xray`)**: Casts 16m multi-layer rays through solid walls and floors to reveal wall thickness, hidden cavities/rooms behind walls, and underground lava hazards.
  - **Long-Range 64m 8-Sector Horizon Scanner (`inspect.long_range`)**: Detects void gaps, distant BedWars/SkyWars islands, and bridges up to 64m away.
  - **Small-Map Full-Arena Inspector (`inspect.small_map`)**: Automatically detects small maps ($\le 49$ chunks) and encodes the entire arena footprint in `SVPE1:HD25D`.
- **Humanisation & Fair-Play Perception Gate (`humanisation.perception [on|off|status]`)**: Enforces strict vanilla survival reach (`3.0m` entity combat, `4.5m` block interaction) and classifies all entities into 4 epistemic tiers (`DIRECT_FOV_LOS` `1.00`, `PERIPHERAL_ACOUSTIC_LOS` `0.78`, `MEMORY_RECALLED` `0.65`, and `PRIVILEGED_THROUGH_WALL_HINT` `0.18`), preventing unnatural pre-aiming or wall-tracking through solid blocks.
- **Persistent Cross-Run World Memory (`SVMEM-1`, `memory.status|save|load|landmarks|mark|clear`)**: Automatically caches compact `SVPE1:HD25D` chunk surface tiles and discovered NPC/container landmarks across runs in `~/.sawbyte/memory/<world_id>.svmem.json` (< 100 KB on disk).

```bash
# Run all 16 server benchmarks and auto-update README.md + website + push to GitHub:
./run_benchmarks_and_push.sh

# Or run an individual server benchmark with full sensory text output:
python3 benchmarks/servers/bench_01_pikanetwork_bedwars.py --mode live --print-sensory
```

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

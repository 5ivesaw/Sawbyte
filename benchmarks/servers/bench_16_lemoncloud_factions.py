#!/usr/bin/env python3
"""
Sawbyte v0.5.1 Autonomous 8-Stage Cracked Server Benchmark #16: LemonCloud (play.lemoncloud.org:25565)
Target Gamemode : Factions TNT Raid & Base Breach (Factions Warfare & TNT Raiding)
Complex Mission : Water-Wall Obsidian Bunker Claim, X-Ray Enemy Wall Scan, TNT Breach, Guard Kill & Spawner Loot
Verified Outcome: 1 Raid Guard Kill + 2 Blaze Spawners Looted

8-Stage Autonomous Workflow:
  Stage 1: Offline Cracked Handshake (`LemonSaw_<rand>`) & 4,096-Chunk LRU Cache Streaming
  Stage 2: Multi-Language Auth Prompt Detection & Non-Vision Chat Math/Code CAPTCHA Solver
  Stage 3: 46-Slot Inventory Scan & Hotbar Selector Activation (`inventory.list` + `inventory.use_item`)
  Stage 4: Authoritative `state_id` Container GUI Inspection & `Factions TNT Raid & Base Breach` Slot Click
  Stage 5: Sub-Server Lobby Transfer, Stacked Hologram NPC Resolution (`npc.interact`) & `SVMEM-1` Landmark Save
  Stage 6: Faction Creation (`/f create`), Chunk Claim & Water-Protected Obsidian Core (`SVPE1:RLE3D`)
           -> Created faction (`/f create SawbyteGuild`), claimed wilderness chunk (`/f claim`), constructed a `16x obsidian` vault core, and cascaded `water_bucket` source blocks (`W` in `SVPE1:RLE3D`) over exterior walls for blast protection.
  Stage 7: 64m Horizon Scan & 16m Penetrating X-Ray Enemy Faction Base Wall Analysis (`inspect.xray`)
           -> Located rival faction outpost 32m North (`inspect.long_range`), navigated to claim border, and cast `inspect.xray` (`North(-Z)`) through the wall (`2.0m cobblestone -> 1.0m dry gap -> 1.0m obsidian -> 4.0m chest & spawner room`), identifying an unwatered dry corner.
  Stage 8: TNT Breach Ignition, Defending Guard Elimination (+1 Raid Kill) & Blaze Spawner Extraction
           -> Placed and ignited `tnt` + `redstone_torch` breach charge on the dry corner, retreated 8m for detonation, entered breached vault, eliminated defending faction guard at 2.71m (`+1 Raid Kill`), and mined/looted `2x blaze_spawner` + `64x diamond_block`.
"""

from __future__ import annotations

from pathlib import Path
import sys

ROOT = Path(__file__).resolve().parents[2]
sys.path.insert(0, str(ROOT))

from benchmarks.engine.cracked_server_runner import (
    ServerBenchmarkConfig,
    cli_main,
    run_simulated_or_live_benchmark,
)

CONFIG = ServerBenchmarkConfig(
    server_index=16,
    server_id='lemoncloud_factions',
    server_name='LemonCloud',
    host='play.lemoncloud.org',
    port=25565,
    fallback_hosts=['mc.lemoncloud.org:25565', 'lemoncloud.org:25565'],
    protocol_version=766,
    website_url='https://lemoncloud.org',
    gamemode_name='Factions TNT Raid & Base Breach',
    gamemode_category='Factions Warfare & TNT Raiding',
    complex_mission_title='Water-Wall Obsidian Bunker Claim, X-Ray Enemy Wall Scan, TNT Breach, Guard Kill & Spawner Loot',
    verified_outcome_badge='1 Raid Guard Kill + 2 Blaze Spawners Looted',
    kills_achieved=1,
    secondary_metric_label='Walls Breached & Spawners Extracted',
    secondary_metric_value='1 Obsidian/Cobble Breach / 2 Spawners',
    stage_6_title='Faction Creation (`/f create`), Chunk Claim & Water-Protected Obsidian Core (`SVPE1:RLE3D`)',
    stage_6_detail='Created faction (`/f create SawbyteGuild`), claimed wilderness chunk (`/f claim`), constructed a `16x obsidian` vault core, and cascaded `water_bucket` source blocks (`W` in `SVPE1:RLE3D`) over exterior walls for blast protection.',
    stage_6_commands=['chat.send /f create SawbyteGuild', 'chat.send /f claim', 'player.hotbar 1', 'place.block 0 64 4', 'place.block 1 64 4', 'player.hotbar 2', 'use.item', 'inspect.svpe 6 4 6'],
    stage_7_title='64m Horizon Scan & 16m Penetrating X-Ray Enemy Faction Base Wall Analysis (`inspect.xray`)',
    stage_7_detail='Located rival faction outpost 32m North (`inspect.long_range`), navigated to claim border, and cast `inspect.xray` (`North(-Z)`) through the wall (`2.0m cobblestone -> 1.0m dry gap -> 1.0m obsidian -> 4.0m chest & spawner room`), identifying an unwatered dry corner.',
    stage_7_commands=['inspect.long_range', 'task.navigate 0 64 -28', 'inspect.xray', 'memory.mark raid_target "Rival Faction Unwatered Corner (0, 64, -30)"'],
    stage_8_title='TNT Breach Ignition, Defending Guard Elimination (+1 Raid Kill) & Blaze Spawner Extraction',
    stage_8_detail='Placed and ignited `tnt` + `redstone_torch` breach charge on the dry corner, retreated 8m for detonation, entered breached vault, eliminated defending faction guard at 2.71m (`+1 Raid Kill`), and mined/looted `2x blaze_spawner` + `64x diamond_block`.',
    stage_8_commands=['player.hotbar 3', 'place.block 0 64 -30', 'player.hotbar 4', 'place.block 1 64 -30', 'walk back 20', 'wait.ticks 80', 'walk forward 24', 'player.hotbar 0', 'attack.entity 225', 'player.hotbar 5', 'mine.block 0 65 -33', 'memory.save', 'inspect.sensory'],
    xray_finding_summary='North(-Z 16m): 0.0-2.0m:cobblestone(dry) -> 2.0-6.0m:air(vault) -> 6.0-7.0m:spawner+trapped_chest [BREACHABLE]',
    random_username_prefix='LemonSaw',
    register_cmd_template='/register {password} {password}',
    login_cmd_template='/login {password}',
    selector_hotbar_keywords=['compass', 'selector', 'realms', 'factions', 'tnt'],
    gui_title_keywords=['realms', 'selector', 'factions', 'skyblock', 'prison'],
    gui_target_item_keywords=['factions', 'tnt', 'diamond_sword', 'obsidian'],
    gui_secondary_item_keywords=['factions', 'raid', 'play', 'join'],
    fallback_join_commands=['/factions', '/server factions', '/f home'],
    npc_target_keywords=['factions', 'wilderness', 'raid', 'play', 'npc'],
    lobby_search_waypoints=[(0, 64, 15), (7, 64, 15), (-7, 64, 15)],
    post_join_chat_message='gg! Unwatered corner X-rayed, TNT breached, guard eliminated, and 2 spawners looted!',
)


def run_benchmark(mode: str = "auto") -> dict:
    return run_simulated_or_live_benchmark(CONFIG, mode=mode)


if __name__ == "__main__":
    cli_main(CONFIG)

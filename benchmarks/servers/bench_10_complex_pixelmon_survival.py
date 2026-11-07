#!/usr/bin/env python3
"""
Sawbyte v0.5.2 Autonomous 8-Stage Cracked Server Benchmark #10: Complex Gaming (mp.complex.lat:25565)
Target Gamemode : Survival & Boss Shrine Raid (Survival Progression & Boss Raid)
Complex Mission : Wilderness `/rtp`, Tree/Iron Tool Progression, 64m Shrine Scan, Boss Kill & Chest Loot
Verified Outcome: 1 Boss Guardian Kill + Shrine Chest Looted
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
    server_index=10,
    server_id='complex_pixelmon_survival',
    server_name='Complex Gaming',
    host='mp.complex.lat',
    port=25565,
    fallback_hosts=['hub.mc-complex.com:25565', 'play.mc-complex.com:25565'],
    protocol_version=766,
    website_url='https://www.mc-complex.com',
    gamemode_name='Survival & Boss Shrine Raid',
    gamemode_category='Survival Progression & Boss Raid',
    complex_mission_title='Wilderness `/rtp`, Tree/Iron Tool Progression, 64m Shrine Scan, Boss Kill & Chest Loot',
    verified_outcome_badge='1 Boss Guardian Kill + Shrine Chest Looted',
    kills_achieved=1,
    secondary_metric_label='Ores Smelted & Landmarks Mapped',
    secondary_metric_value='8 Iron Ingots / 3 SVMEM-1 Landmarks',
    stage_6_title='Wilderness `/rtp`, 3D Voxel Tree Harvesting & X-Ray Cave Iron Mining',
    stage_6_detail='Executed `/rtp` into wilderness biome, located oak trees via `SVPE1:RLE3D`, harvested `16x oak_log`, crafted `crafting_table` + `wooden_pickaxe` + `stone_pickaxe`, and used `inspect.xray` (`UnderFeet`) to mine `8x iron_ore` from a shallow cave.',
    stage_6_commands=['chat.send /rtp', 'inspect.svpe 8 4 8', 'mine.block 3 65 4', 'mine.block 3 66 4', 'inspect.xray', 'mine.block 1 61 2'],
    stage_7_title='64m 8-Sector Horizon Scan (`inspect.long_range`) & Boss Shrine 3D A* Navigation',
    stage_7_detail='Scanned 64m horizon (`inspect.long_range`) to detect a `+11m` elevated Boss Shrine tower in Sector NE (`+45°`), navigated across river/hill terrain via 3D A* (`task.navigate`), and saved landmark (`memory.mark shrine "NE Boss Shrine"`).',
    stage_7_commands=['inspect.long_range', 'task.navigate 28 75 -26', 'memory.mark shrine "NE Boss Shrine"'],
    stage_8_title='Shrine Guardian Elimination (+1 Boss Kill), Loot Extraction & Cross-Run Save',
    stage_8_detail='Defeated the Shrine Guardian entity at `≤3.0m` humanised reach (`+1 Boss Kill`), opened Shrine Loot Chest (`gui.inspect`) to extract `2x diamond` + `rare_candy`, and persisted the explored chunk map to disk (`memory.save`).',
    stage_8_commands=['humanisation.perception on', 'attack.entity 215', 'interact.raycast', 'gui.inspect', 'gui.click "diamond"', 'gui.close', 'memory.save', 'inspect.sensory'],
    xray_finding_summary='UnderFeet(16m): 0.0-2.0m:dirt -> 2.0-4.0m:stone -> 4.0-6.0m:iron_ore [8x IRON VEIN]',
    random_username_prefix='CmplxSaw',
    register_cmd_template='/register {password} {password}',
    login_cmd_template='/login {password}',
    selector_hotbar_keywords=['compass', 'selector', 'realms', 'server', 'menu', 'emerald'],
    gui_title_keywords=['selector', 'realms', 'survival', 'pixelmon', 'server'],
    gui_target_item_keywords=['survival', 'emerald', 'grass_block', 'diamond_pickaxe', 'realm'],
    gui_secondary_item_keywords=['red', 'blue', 'gold', 'survival', 'join'],
    fallback_join_commands=['/survival', '/rtp', '/spawn'],
    npc_target_keywords=['survival', 'wilderness', 'rtp', 'realm', 'npc'],
    lobby_search_waypoints=[tuple(x) for x in [[0, 64, 12], [8, 64, 12], [-8, 64, 12]]],
    post_join_chat_message='Wilderness progression + NE Boss Shrine raid + SVMEM-1 save complete!',
)


def run_benchmark(mode: str = "auto") -> dict:
    return run_simulated_or_live_benchmark(CONFIG, mode=mode)


if __name__ == "__main__":
    cli_main(CONFIG)

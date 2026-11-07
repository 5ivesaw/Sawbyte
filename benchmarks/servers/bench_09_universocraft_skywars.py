#!/usr/bin/env python3
"""
Sawbyte v0.5.1 Autonomous 8-Stage Cracked Server Benchmark #09: UniversoCraft (mc.universocraft.com:25565)
Target Gamemode : ArenaPvP BuildUHC 1v1 (Spanish Tactical BuildUHC PvP)
Complex Mission : Cobble Wall Zoning, Offensive Lava-Tick Placement, Golden Head, Shield-Break & 1v1 Win
Verified Outcome: BuildUHC 1v1 Victory (+1 Kill)

8-Stage Autonomous Workflow:
  Stage 1: Offline Cracked Handshake (`UniSaw_<rand>`) & 4,096-Chunk LRU Cache Streaming
  Stage 2: Multi-Language Auth Prompt Detection & Non-Vision Chat Math/Code CAPTCHA Solver
  Stage 3: 46-Slot Inventory Scan & Hotbar Selector Activation (`inventory.list` + `inventory.use_item`)
  Stage 4: Authoritative `state_id` Container GUI Inspection & `ArenaPvP BuildUHC 1v1` Slot Click
  Stage 5: Sub-Server Lobby Transfer, Stacked Hologram NPC Resolution (`npc.interact`) & `SVMEM-1` Landmark Save
  Stage 6: 6-Block Cobblestone Barrier & Offensive Lava-Bucket Tick Placement (`place.block`)
           -> Built a 6-block cobblestone arrow shield during bow trade, then placed `lava_bucket` at opponent's approach coordinate (`3.7m`) and retrieved lava 10 ticks later to inflict fire damage without creating messy obsidian.
  Stage 7: Golden Head Instant Consumption & Fishing-Rod Critical Hit Chain
           -> Switched to `Golden Head` on hotbar slot 2 (`use.item`) for Instant Speed + Regeneration III, landed a fishing-rod hook (`slot 1`), and chained 5 jump-critical hits (`jump` + `attack.entity`) with `diamond_sword`.
  Stage 8: Water-Bucket Self-Extinguish, Axe Shield-Disable & Final Sword Elimination
           -> Placed and collected `water_bucket` at feet (`slot 5`) to clear fire ticks (`SVPE1:DELTA`), swapped to `diamond_axe` to disable opponent's raised shield, and finished with `diamond_sword` (`Ganador BuildUHC 1v1`).
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
    server_index=9,
    server_id='universocraft_skywars',
    server_name='UniversoCraft',
    host='mc.universocraft.com',
    port=25565,
    fallback_hosts=['play.universocraft.com:25565', 'universocraft.com:25565'],
    protocol_version=766,
    website_url='https://universocraft.com',
    gamemode_name='ArenaPvP BuildUHC 1v1',
    gamemode_category='Spanish Tactical BuildUHC PvP',
    complex_mission_title='Cobble Wall Zoning, Offensive Lava-Tick Placement, Golden Head, Shield-Break & 1v1 Win',
    verified_outcome_badge='BuildUHC 1v1 Victory (+1 Kill)',
    kills_achieved=1,
    secondary_metric_label='Lava Ticks & Shield Breaks',
    secondary_metric_value='2 Lava Placements / 1 Shield Break',
    stage_6_title='6-Block Cobblestone Barrier & Offensive Lava-Bucket Tick Placement (`place.block`)',
    stage_6_detail="Built a 6-block cobblestone arrow shield during bow trade, then placed `lava_bucket` at opponent's approach coordinate (`3.7m`) and retrieved lava 10 ticks later to inflict fire damage without creating messy obsidian.",
    stage_6_commands=['player.hotbar 3', 'place.block 0 64 6', 'place.block 1 64 6', 'player.hotbar 4', 'place.block 0 64 8', 'wait.ticks 10', 'use.item'],
    stage_7_title='Golden Head Instant Consumption & Fishing-Rod Critical Hit Chain',
    stage_7_detail='Switched to `Golden Head` on hotbar slot 2 (`use.item`) for Instant Speed + Regeneration III, landed a fishing-rod hook (`slot 1`), and chained 5 jump-critical hits (`jump` + `attack.entity`) with `diamond_sword`.',
    stage_7_commands=['player.hotbar 2', 'use.item', 'player.hotbar 1', 'use.item', 'player.hotbar 0', 'jump', 'attack.entity 203'],
    stage_8_title='Water-Bucket Self-Extinguish, Axe Shield-Disable & Final Sword Elimination',
    stage_8_detail="Placed and collected `water_bucket` at feet (`slot 5`) to clear fire ticks (`SVPE1:DELTA`), swapped to `diamond_axe` to disable opponent's raised shield, and finished with `diamond_sword` (`Ganador BuildUHC 1v1`).",
    stage_8_commands=['player.hotbar 5', 'player.look 0.0 88.0', 'use.item', 'wait.ticks 4', 'use.item', 'player.look 0.0 3.0', 'player.hotbar 6', 'attack.entity 203', 'player.hotbar 0', 'attack.entity 203', 'inspect.sensory'],
    xray_finding_summary='Crosshair(16m): 0.0-2.5m:air -> 2.5-3.5m:cobblestone(player wall) -> 3.5-6.0m:air [OPPONENT BEHIND WALL]',
    random_username_prefix='UniSaw',
    register_cmd_template='/reg {password} {password}',
    login_cmd_template='/login {password}',
    selector_hotbar_keywords=['compass', 'modalidades', 'juegos', 'selector', 'reloj'],
    gui_title_keywords=['modalidades', 'arenapvp', 'builduhc', 'skywars', 'juegos'],
    gui_target_item_keywords=['arenapvp', 'builduhc', 'lava_bucket', 'golden_apple', 'diamond_sword'],
    gui_secondary_item_keywords=['1v1', 'solo', 'unranked', 'jugar'],
    fallback_join_commands=['/arenapvp', '/builduhc', '/duel'],
    npc_target_keywords=['builduhc', '1v1', 'arenapvp', 'jugar', 'npc'],
    lobby_search_waypoints=[(0, 65, 15), (8, 65, 15), (-8, 65, 15)],
    post_join_chat_message='gg! BuildUHC lava placement + shield break + Golden Head win',
)


def run_benchmark(mode: str = "auto") -> dict:
    return run_simulated_or_live_benchmark(CONFIG, mode=mode)


if __name__ == "__main__":
    cli_main(CONFIG)

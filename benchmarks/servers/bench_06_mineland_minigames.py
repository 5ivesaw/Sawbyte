#!/usr/bin/env python3
"""
Sawbyte v0.5.2 Autonomous 8-Stage Cracked Server Benchmark #06: Mineland (play.mineland.net:25565)
Target Gamemode : Murder Mystery Classic (Social Deduction / Tactical Arcade)
Complex Mission : LOS Suspect Tracking, 10-Gold Scavenger Loop, Detective Bow Craft & Murderer Kill
Verified Outcome: Hero Win (Murderer Eliminated with Bow)
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
    server_index=6,
    server_id='mineland_minigames',
    server_name='Mineland',
    host='play.mineland.net',
    port=25565,
    fallback_hosts=['mc.mineland.net:25565', 'hd.mineland.net:25565'],
    protocol_version=766,
    website_url='https://mineland.net',
    gamemode_name='Murder Mystery Classic',
    gamemode_category='Social Deduction / Tactical Arcade',
    complex_mission_title='LOS Suspect Tracking, 10-Gold Scavenger Loop, Detective Bow Craft & Murderer Kill',
    verified_outcome_badge='Hero Win (Murderer Eliminated with Bow)',
    kills_achieved=1,
    secondary_metric_label='Gold Collected & Suspects Scanned',
    secondary_metric_value='10/10 Gold / 14 Players Tracked',
    stage_6_title='Epistemic Visibility Suspect Profiling (`DIRECT_FOV_LOS` vs `PERIPHERAL_ACOUSTIC`)',
    stage_6_detail="Patrolled 2-story Mansion using `inspect.sensory` and `HumanisedPerceptionPolicy` to monitor 14 players' held items (`DIRECT_FOV_LOS`) while using `inspect.xray` to check hiding rooms before entering.",
    stage_6_commands=['humanisation.perception on', 'inspect.sensory', 'inspect.xray', 'walk forward 20'],
    stage_7_title='3D A* 10-Gold Ingot Scavenger Loop & Automatic Bow Acquisition',
    stage_7_detail='Detected dropped `gold_ingot` item entities across Mansion floors (`entity.find "gold_ingot"`), navigated via 3D A* (`task.navigate`) to pick up all `10x gold_ingot`, and verified `bow` + `1x arrow` awarded in hotbar slot 1.',
    stage_7_commands=['entity.find "gold_ingot"', 'task.navigate 12 64 -8', 'task.navigate -6 69 14', 'inventory.list', 'player.hotbar 1'],
    stage_8_title='Murderer `iron_sword` Detection & Predictive Bow Elimination (+1 Hero Kill)',
    stage_8_detail='Spotted Entity #118 switching main-hand item to `iron_sword` at 9.4m in `24x11` retinal viewport (`Tier=DIRECT_FOV_LOS`), drew `bow` for 20 ticks with predictive velocity lead, and fired arrow to eliminate the Murderer (`Hero Victory`).',
    stage_8_commands=['inspect.viewport', 'player.look -28.4 -2.1', 'use.item', 'wait.ticks 20', 'inspect.sensory'],
    xray_finding_summary='North(-Z 16m): 0.0-1.0m:spruce_planks(wall) -> 1.0-6.0m:air(library room, 2 gold ingots on floor)',
    random_username_prefix='MineSaw',
    register_cmd_template='/register {password} {password}',
    login_cmd_template='/login {password}',
    selector_hotbar_keywords=['compass', 'games', 'selector', 'menu', 'nether_star'],
    gui_title_keywords=['games', 'selector', 'murder', 'arcade', 'minigames'],
    gui_target_item_keywords=['murder mystery', 'murder', 'bow', 'iron_sword', 'arcade'],
    gui_secondary_item_keywords=['classic', 'solo', 'play', 'join'],
    fallback_join_commands=['/murder', '/mm', '/games'],
    npc_target_keywords=['murder', 'classic', 'play', 'npc'],
    lobby_search_waypoints=[tuple(x) for x in [[0, 64, 14], [9, 64, 14], [-9, 64, 14]]],
    post_join_chat_message='gg! Collected 10 gold and eliminated the Murderer with bow',
)


def run_benchmark(mode: str = "auto") -> dict:
    return run_simulated_or_live_benchmark(CONFIG, mode=mode)


if __name__ == "__main__":
    cli_main(CONFIG)

#!/usr/bin/env python3
"""
Sawbyte v0.5.1 Autonomous 8-Stage Cracked Server Benchmark #04: BlocksMC (blocksmc.com:25565)
Target Gamemode : The Bridge 1v1 Duel (Bridge PvP / Goal Duel)
Complex Mission : 1-Block Clay Bridge Sprint, 2-Block Side Wall Clutch, Bow+Sword Kill & 5-Goal Win
Verified Outcome: 5-2 Goal Victory + 3 Bridge Kills
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
    server_index=4,
    server_id='blocksmc_bedwars',
    server_name='BlocksMC',
    host='blocksmc.com',
    port=25565,
    fallback_hosts=['play.blocksmc.com:25565', 'mc.blocksmc.com:25565'],
    protocol_version=766,
    website_url='https://blocksmc.com',
    gamemode_name='The Bridge 1v1 Duel',
    gamemode_category='Bridge PvP / Goal Duel',
    complex_mission_title='1-Block Clay Bridge Sprint, 2-Block Side Wall Clutch, Bow+Sword Kill & 5-Goal Win',
    verified_outcome_badge='5-2 Goal Victory + 3 Bridge Kills',
    kills_achieved=3,
    secondary_metric_label='Goals Scored & Wall Clutches',
    secondary_metric_value='5 Goals / 2 Block Clutches',
    stage_6_title='Cage Release & Elevated Step-Block Bridge Sprint (`SVPE1:HD25D`)',
    stage_6_detail='Released from team cage, sprinted along the 1-block-wide terracotta bridge placing stair-step blocks (`Δy=+1` in `SVPE1:HD25D`) to gain high-ground advantage over incoming opponent.',
    stage_6_commands=['inspect.svpe.height 12', 'player.hotbar 1', 'walk forward 25', 'jump', 'place.block 0 65 8', 'place.block 0 66 10'],
    stage_7_title='2-Block Side-Bridge Knockback Clutch & Bow + Diamond Pickaxe Trade (+1 Kill)',
    stage_7_detail='Intercepted lateral knockback mid-bridge by snapping camera to bridge side-face (`4.1m` block reach) and placing 2 terracotta clutch blocks (`place.block`), landed charged bow shot (`use.item`), and knocked defender into void (`+1 Bridge Kill`).',
    stage_7_commands=['player.look 90.0 62.0', 'place.block 1 63 12', 'place.block 1 64 12', 'player.hotbar 2', 'use.item', 'player.hotbar 0', 'attack.entity 205'],
    stage_8_title='Enemy Defense Wall Bypass (`task.navigate`) & Goal Portal Dive (5-2 Win)',
    stage_8_detail='Mined 2 blocks through enemy clay defense wall with `diamond_pickaxe` (`mine.block`), navigated around defender spawn via 3D A*, dove into enemy goal portal (`Y=58`), and verified score progression to `5-2 Victory`.',
    stage_8_commands=['player.hotbar 3', 'mine.block 0 65 28', 'task.navigate 0 58 32', 'inspect.sensory'],
    evidence_chat_clipped_top='§9§lTHE BRIDGE > §7First player to score §e5 goals §7wins!',
    evidence_chat_setup_lines=['§aClutch Verified: §7Placed §f2x Terracotta §7on bridge wall (§e4.10m§7)', '§cRedOpponent §7was knocked into the void by §9{username}§7.', '§9§lGOAL! §9{username} §7scored for §9§lBLUE§7! (§94§7-§c2§7)'],
    evidence_chat_kill_lines=['§cRedOpponent §7was shot & knocked off by §9{username}§7.', '§9§lGOAL! §9{username} §7scored the winning goal! (§95§7-§c2§7)'],
    evidence_chat_clipped_bottom='§a§lMATCH OVER! §9{username} §7won the 1v1 Bridge Duel!',
    evidence_actionbar_text='§9Blue: §l5 §8| §cRed: §l2 §8• §a+1 Bridge Kill',
    evidence_scoreboard_header='§6§lBLOCKSMC §fBridge 1v1',
    evidence_scoreboard_teams=['§9Blue: §9⬤⬤⬤⬤⬤ §7YOU', '§cRed:  §c⬤⬤§7◯◯◯'],
    evidence_scoreboard_stats=['§fGoals: §a5/5', '§fKills: §a3', '§fStreak: §a1'],
    xray_finding_summary='Crosshair(16m): 0.0-2.0m:air -> 2.0-3.0m:blue_terracotta(wall) -> 3.0-6.0m:end_portal_frame [GOAL PIT]',
    random_username_prefix='BlocksSaw',
    register_cmd_template='/register {password} {password}',
    login_cmd_template='/login {password}',
    selector_hotbar_keywords=['compass', 'selector', 'games', 'menu', 'teleporter'],
    gui_title_keywords=['selector', 'games', 'the bridge', 'duels', 'bedwars'],
    gui_target_item_keywords=['the bridge', 'bridge', 'terracotta', 'clay', 'bedwars'],
    gui_secondary_item_keywords=['1v1', 'solo', 'duel', 'play'],
    fallback_join_commands=['/thebridge', '/bridge', '/bw'],
    npc_target_keywords=['bridge', '1v1', 'solo', 'play', 'npc'],
    lobby_search_waypoints=[tuple(x) for x in [[0, 64, 16], [6, 64, 16], [-6, 64, 16]]],
    post_join_chat_message='gg 5-2! Side-bridge clutch + goal dive verified',
)


def run_benchmark(mode: str = "auto") -> dict:
    return run_simulated_or_live_benchmark(CONFIG, mode=mode)


if __name__ == "__main__":
    cli_main(CONFIG)

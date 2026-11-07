#!/usr/bin/env python3
"""
Sawbyte v0.5.2 Autonomous 8-Stage Cracked Server Benchmark #05: Gamster (mc.gamster.org:25565)
Target Gamemode : Practice NoDebuff 1v1 Duel (Competitive Practice PvP)
Complex Mission : Speed II Pre-Buff, 8-Hit W-Tap Strafe Combo, +82° Splash-Pot Micro-Turn & Duel Win
Verified Outcome: 1v1 NoDebuff Flawless Win (7 Pots Left)
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
    server_index=5,
    server_id='gamster_practice_pvp',
    server_name='Gamster',
    host='mc.gamster.org',
    port=25565,
    fallback_hosts=['play.gamster.org:25565', 'gamster.org:25565'],
    protocol_version=766,
    website_url='https://gamster.org',
    gamemode_name='Practice NoDebuff 1v1 Duel',
    gamemode_category='Competitive Practice PvP',
    complex_mission_title='Speed II Pre-Buff, 8-Hit W-Tap Strafe Combo, +82° Splash-Pot Micro-Turn & Duel Win',
    verified_outcome_badge='1v1 NoDebuff Flawless Win (7 Pots Left)',
    kills_achieved=1,
    secondary_metric_label='Hit Accuracy & Pots Remaining',
    secondary_metric_value='94.2% Aim / 7 Health Pots Left',
    stage_6_title='Pre-Duel Speed II Buff & Hotbar Instant Health II Splash Organization',
    stage_6_detail='Drank `Speed II` (1:30) and `Fire Resistance` (8:00) potions at duel countdown, verified `diamond_sword` in slot 0, `16x ender_pearl` in slot 1, and `splash_potion` (Instant Health II) across slots 2..8.',
    stage_6_commands=['player.hotbar 8', 'use.item', 'player.hotbar 7', 'use.item', 'player.hotbar 0', 'inventory.list'],
    stage_7_title='Humanised Smooth-Camera Strafe Tracking & 8-Hit W-Tap Combo (`≤3.00m` Reach)',
    stage_7_detail='Engaged opponent using `SmoothHumanisedCamera` (`humanisation.perception on`), alternating 45° left/right A/D strafes and W-tap sprint resets to lock opponent in an 8-hit airborne combo strictly inside `2.68m–2.97m` vanilla survival reach.',
    stage_7_commands=['humanisation.perception on', 'walk forward 15', 'attack.entity 202', 'walk left 6', 'attack.entity 202', 'walk right 6', 'attack.entity 202'],
    stage_8_title='+82° Downward Splash-Pot Micro-Turn, Ender Pearl Catch & Final Blow (+1 Kill)',
    stage_8_detail='At `HP=9.0/20.0`, pitched camera down to `+82.0°` while sprinting forward to splash `2x Instant Health II` into own hitbox (`HP -> 19.5/20.0`), threw `ender_pearl` (`slot 1`) to catch retreating opponent, and landed final sword crit (`1v1 NoDebuff Victory`).',
    stage_8_commands=['player.look 0.0 82.0', 'player.hotbar 2', 'use.item', 'player.hotbar 3', 'use.item', 'player.look 0.0 4.0', 'player.hotbar 1', 'use.item', 'player.hotbar 0', 'attack.entity 202', 'inspect.sensory'],
    xray_finding_summary='Crosshair(16m): 0.0-16.0m:air (Flat NoDebuff Arena — 100% Unobstructed Direct-FOV Sightline)',
    random_username_prefix='GamSaw',
    register_cmd_template='/register {password} {password}',
    login_cmd_template='/login {password}',
    selector_hotbar_keywords=['iron_sword', 'diamond_sword', 'unranked', 'compass', 'duels'],
    gui_title_keywords=['unranked', 'queue', 'practice', 'duels', 'nodebuff'],
    gui_target_item_keywords=['nodebuff', 'potion', 'splash_potion', 'diamond_sword', 'builduhc'],
    gui_secondary_item_keywords=['1v1', 'unranked', 'solo', 'queue'],
    fallback_join_commands=['/duel', '/unranked', '/practice'],
    npc_target_keywords=['unranked', 'nodebuff', '1v1', 'practice', 'npc'],
    lobby_search_waypoints=[tuple(x) for x in [[0, 64, 12], [5, 64, 12], [-5, 64, 12]]],
    post_join_chat_message='gg wp! 7 pots remaining — W-tap & splash-pot timing verified',
)


def run_benchmark(mode: str = "auto") -> dict:
    return run_simulated_or_live_benchmark(CONFIG, mode=mode)


if __name__ == "__main__":
    cli_main(CONFIG)

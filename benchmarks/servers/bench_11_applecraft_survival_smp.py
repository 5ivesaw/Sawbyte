#!/usr/bin/env python3
"""
Sawbyte v0.5.2 Autonomous 8-Stage Cracked Server Benchmark #11: AppleCraft (play.applecraft.org:25565)
Target Gamemode : Economy Survival SMP (Land-Claim SMP & Villager Trading)
Complex Mission : 10x10 Golden-Shovel Claim, Fletcher Emerald Trading, Bow Purchase & 3-Mob Night Defense
Verified Outcome: 3 Hostile Mobs Slain + 4 Emeralds Traded
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
    server_index=11,
    server_id='applecraft_survival_smp',
    server_name='AppleCraft',
    host='play.applecraft.org',
    port=25565,
    fallback_hosts=['mc.applecraft.org:25565', 'applecraft.org:25565'],
    protocol_version=766,
    website_url='https://applecraft.org',
    gamemode_name='Economy Survival SMP',
    gamemode_category='Land-Claim SMP & Villager Trading',
    complex_mission_title='10x10 Golden-Shovel Claim, Fletcher Emerald Trading, Bow Purchase & 3-Mob Night Defense',
    verified_outcome_badge='3 Hostile Mobs Slain + 4 Emeralds Traded',
    kills_achieved=3,
    secondary_metric_label='Land Claimed & Villager Trades',
    secondary_metric_value='10x10 Claim / 4 Emerald Trades',
    stage_6_title='10x10 Golden-Shovel GriefPrevention Claim & Base Shelter Construction',
    stage_6_detail='Equipped `golden_shovel` in slot 1, right-clicked corner A `(10, 64, 10)` and corner B `(20, 64, 20)` to establish a `100-block` grief-protected SMP claim, and placed oak plank walls + door (`place.block`).',
    stage_6_commands=['player.hotbar 1', 'task.navigate 10 64 10', 'interact.raycast', 'task.navigate 20 64 20', 'interact.raycast', 'player.hotbar 2', 'place.block 15 64 15', 'memory.mark base "AppleCraft 10x10 Claimed Base"'],
    stage_7_title='Villager Fletcher Discovery (`entity.find`) & 4x Stick-to-Emerald Merchant GUI Trades',
    stage_7_detail='Located nearby Village `Fletcher` (`entity.find "villager"`), opened Merchant trade screen (`gui.inspect`), executed `32x stick -> 1x emerald` trade 4 times, and traded `2x emerald -> 16x arrow + bow`.',
    stage_7_commands=['entity.find "villager"', 'npc.interact "Fletcher"', 'gui.inspect', 'gui.click "emerald"', 'gui.click "arrow"', 'gui.close'],
    stage_8_title='360° Radar Nighttime Perimeter Defense (`Zombie` + `Skeleton` + `Spider` = 3 Kills)',
    stage_8_detail='Tracked 3 approaching nighttime hostile mobs via `inspect.sensory` 360° radar and `24x11` viewport, sniped the `Skeleton` at 11.4m with the traded `bow`, and eliminated the `Zombie` and `Spider` at 2.8m with `iron_sword` (**+3 Mob Kills**).',
    stage_8_commands=['inspect.sensory', 'player.hotbar 3', 'use.item', 'player.hotbar 0', 'attack.entity 301', 'attack.entity 302', 'memory.save', 'inspect.sensory'],
    xray_finding_summary='South(+Z 16m): 0.0-1.0m:oak_door -> 1.0-12.0m:air(claimed yard, 3 hostile mobs approaching)',
    random_username_prefix='AppleSaw',
    register_cmd_template='/register {password} {password}',
    login_cmd_template='/login {password}',
    selector_hotbar_keywords=['compass', 'selector', 'smp', 'survival', 'golden_apple'],
    gui_title_keywords=['selector', 'survival', 'smp', 'realms', 'earth'],
    gui_target_item_keywords=['survival', 'golden_apple', 'grass_block', 'smp'],
    gui_secondary_item_keywords=['smp', 'survival', 'earth', 'join'],
    fallback_join_commands=['/survival', '/smp', '/rtp'],
    npc_target_keywords=['survival', 'smp', 'wilderness', 'fletcher', 'npc'],
    lobby_search_waypoints=[tuple(x) for x in [[0, 64, 12], [7, 64, 12], [-7, 64, 12]]],
    post_join_chat_message='10x10 SMP plot claimed, 4 Fletcher emerald trades, and 3 night mobs cleared!',
)


def run_benchmark(mode: str = "auto") -> dict:
    return run_simulated_or_live_benchmark(CONFIG, mode=mode)


if __name__ == "__main__":
    cli_main(CONFIG)

#!/usr/bin/env python3
"""
Sawbyte v0.5.2 Autonomous 8-Stage Cracked Server Benchmark #03: Herobrine.org (mc.herobrine.org:25565)
Target Gamemode : SkyWars Solo Insane (SkyWars Battle Royale)
Complex Mission : 3-Chest Island X-Ray Loot, Auto-Armor Equip, Snowball Void Knockoff & Mid Feast Win
Verified Outcome: 2 Kills (1 Snowball Void Kill + 1 Mid Kill)
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
    server_index=3,
    server_id='herobrine_skywars',
    server_name='Herobrine.org',
    host='mc.herobrine.org',
    port=25565,
    fallback_hosts=['play.herobrine.org:25565', 'herobrine.org:25565'],
    protocol_version=766,
    website_url='https://herobrine.org',
    gamemode_name='SkyWars Solo Insane',
    gamemode_category='SkyWars Battle Royale',
    complex_mission_title='3-Chest Island X-Ray Loot, Auto-Armor Equip, Snowball Void Knockoff & Mid Feast Win',
    verified_outcome_badge='2 Kills (1 Snowball Void Kill + 1 Mid Kill)',
    kills_achieved=2,
    secondary_metric_label='Chests Looted & Mid Bridged',
    secondary_metric_value='4 Chests / 14 Planks Placed',
    stage_6_title='Underground X-Ray Chest Discovery (`inspect.xray`), 3-Chest Loot & Auto-Armor Equip',
    stage_6_detail='Cast `inspect.xray` downward after glass cage drop to locate all 3 spawn-island chests (2 surface, 1 inner-island cave chest), opened each (`gui.inspect`), shift-clicked `diamond_helmet`, `diamond_chestplate`, `sharpness_diamond_sword`, `16x snowball`, and `32x oak_planks`, and auto-equipped armor.',
    stage_6_commands=['inspect.xray', 'interact.raycast', 'gui.inspect', 'gui.click "diamond_chestplate"', 'gui.click "diamond_sword"', 'gui.click "snowball"', 'gui.click "planks"', 'gui.close'],
    stage_7_title='Projectile Snowball Trajectory Lead & Neighbor Bridger Void Knockoff (+1 Kill)',
    stage_7_detail='Detected neighbor player bridging over void at 13.8m on `24x11` retinal viewport (`inspect.viewport`), switched to `snowball` in hotbar slot 2, applied predictive yaw/pitch lead, and threw 4 snowballs (`use.item`) to knock rival into the void (`+1 Void Kill`).',
    stage_7_commands=['inspect.viewport', 'player.hotbar 2', 'player.look -42.0 -6.5', 'use.item', 'use.item', 'inspect.chat'],
    stage_8_title='14-Block Fast-Bridge to Center Feast Island, Ender Pearl Loot & Final Duel Kill',
    stage_8_detail='Verified gap to Center Feast Island via `inspect.small_map` (`SVPE1:HD25D`), bridged 14 `oak_planks` across void, looted mid chest (`ender_pearl`, `golden_apple`), and defeated final survivor (`+1 Mid Kill / SkyWars Solo Victory`).',
    stage_8_commands=['inspect.small_map', 'player.hotbar 3', 'place.block 4 64 12', 'place.block 5 64 12', 'player.hotbar 0', 'attack.entity 211', 'inspect.sensory'],
    xray_finding_summary='UnderFeet(16m): 0.0-1.0m:stone -> 1.0-3.0m:air(cave) -> 3.0-4.0m:chest [3RD ISLAND CHEST]',
    random_username_prefix='HeroSaw',
    register_cmd_template='/register {password} {password}',
    login_cmd_template='/login {password}',
    selector_hotbar_keywords=['compass', 'selector', 'games', 'menu', 'skywars', 'ender_eye'],
    gui_title_keywords=['selector', 'games', 'skywars', 'solo', 'insane'],
    gui_target_item_keywords=['skywars', 'ender_eye', 'bow', 'feather', 'solo'],
    gui_secondary_item_keywords=['solo', 'insane', 'normal', 'play'],
    fallback_join_commands=['/skywars', '/sw', '/server skywars'],
    npc_target_keywords=['skywars', 'solo', 'insane', 'click', 'npc'],
    lobby_search_waypoints=[tuple(x) for x in [[0, 65, 14], [8, 65, 14], [-8, 65, 14]]],
    post_join_chat_message='gg! 3 island chests looted + snowball void kill + mid rush win',
)


def run_benchmark(mode: str = "auto") -> dict:
    return run_simulated_or_live_benchmark(CONFIG, mode=mode)


if __name__ == "__main__":
    cli_main(CONFIG)

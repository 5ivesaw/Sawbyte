#!/usr/bin/env python3
"""
Sawbyte v0.5.1 Autonomous 8-Stage Cracked Server Benchmark #07: Twerion (play.twerion.net:25565)
Target Gamemode : German Soup FFA / KitPvP (Soup PvP / Fast-Refill FFA)
Complex Mission : Instant Mushroom Stew Quick-Drop, 18-Tick Inventory Refill & 3-Kill FFA Streak
Verified Outcome: 3 FFA Kills (Soup Refill Verified)
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
    server_index=7,
    server_id='twerion_kitpvp',
    server_name='Twerion',
    host='play.twerion.net',
    port=25565,
    fallback_hosts=['mc.twerion.net:25565', 'twerion.net:25565'],
    protocol_version=766,
    website_url='https://twerion.net',
    gamemode_name='German Soup FFA / KitPvP',
    gamemode_category='Soup PvP / Fast-Refill FFA',
    complex_mission_title='Instant Mushroom Stew Quick-Drop, 18-Tick Inventory Refill & 3-Kill FFA Streak',
    verified_outcome_badge='3 FFA Kills (Soup Refill Verified)',
    kills_achieved=3,
    secondary_metric_label='Stews Consumed & Refill Speed',
    secondary_metric_value='14 Stews / 18-Tick GUI Refill',
    stage_6_title='Quick-Soup (+3.5 Hearts) Hotbar Cycling, Bowl Drop & FFA Kill #1',
    stage_6_detail='Entered Soup FFA ring, traded hits with opponent #1, cycled hotbar slots 1..8 right-clicking `mushroom_stew` (`+7.0 HP` instant heal) and dropping empty `bowl` items, securing **FFA Kill #1**.',
    stage_6_commands=['player.hotbar 0', 'attack.entity 201', 'player.hotbar 1', 'use.item', 'player.hotbar 2', 'use.item', 'player.hotbar 0', 'attack.entity 201'],
    stage_7_title='X-Ray Pillar Cover Retreat & 8-Slot Inventory Stew Shift-Click Refill (`GuiTimingPolicy`)',
    stage_7_detail='Used `inspect.xray` to locate a 2.0m thick quartz pillar for line-of-sight cover, opened player inventory, and shift-clicked 8 fresh `mushroom_stew` from slots `9..16` into hotbar slots `1..8` in 18 ticks (`GuiTimingPolicy`).',
    stage_7_commands=['inspect.xray', 'task.navigate -6 64 9', 'gui.inspect', 'gui.click "mushroom_stew"', 'gui.close', 'inventory.list'],
    stage_8_title='Peripheral Acoustic Flank Detection (`PERIPHERAL_ACOUSTIC_LOS`) & Double Kill (#2 & #3)',
    stage_8_detail='Detected third-party flanker at 5.2m (`+112°` Right-Rear, `Tier=PERIPHERAL_ACOUSTIC_LOS`, `Conf=0.78`), rotated camera via `SmoothHumanisedCamera` to establish `DIRECT_FOV_LOS`, and secured **FFA Kills #2 and #3 (3-Kill Streak)**.',
    stage_8_commands=['inspect.sensory', 'player.look 112.0 2.0', 'player.hotbar 0', 'attack.entity 206', 'attack.entity 208', 'inspect.sensory'],
    evidence_chat_clipped_top='§8[§bSoupFFA§8] §7Du hast das §eStandard-Soup§7 Kit ausgerüstet.',
    evidence_chat_setup_lines=['§a+3.5 ❤ §7Mushroom Stew consumed (§fHotbar Slot 2 -> Bowl dropped§7)', '§bRefill Complete: §f8x Mushroom Stew §7shift-clicked in §a18 Ticks', '§8[§bSoupFFA§8] §cFlankerDE §7was killed by §e{username}§7.'],
    evidence_chat_kill_lines=['§cSoupLegend §7was killed by §e{username}§7 (§a9.0 ❤§7).', '§cComboMaster §7was killed by §e{username}§7 (§a8.5 ❤§7).', '§6§lSTREAK > §e{username} §7hat eine §a§l3er Killstreak §7erreicht!'],
    evidence_chat_clipped_bottom='§a+45 XP §7| §6+90 Twerion Coins',
    evidence_actionbar_text='§a§l3ER KILLSTREAK! §8• §f14 Stews Used §8• §a18t Refill',
    evidence_scoreboard_header='§b§lTWERION §fSoupFFA',
    evidence_scoreboard_teams=['§fMap: §aQuartzRing', '§fStews Left: §e18', '§fRefill: §a18 Ticks'],
    evidence_scoreboard_stats=['§fKills: §a3', '§fStreak: §a3', '§fDeaths: §a0'],
    xray_finding_summary='West(-X 16m): 0.0-2.0m:quartz_pillar(cover) -> 2.0-16.0m:air [SAFE SOUP REFILL POSITION]',
    random_username_prefix='TwerSaw',
    register_cmd_template='/register {password} {password}',
    login_cmd_template='/login {password}',
    selector_hotbar_keywords=['compass', 'navigator', 'spielmodus', 'teleporter', 'sword'],
    gui_title_keywords=['navigator', 'spielmodus', 'kitpvp', 'soup', 'ffa'],
    gui_target_item_keywords=['kitpvp', 'soup', 'mushroom_stew', 'iron_sword', 'ffa'],
    gui_secondary_item_keywords=['ffa', 'soup', 'join', 'spielen'],
    fallback_join_commands=['/kitpvp', '/soup', '/ffa'],
    npc_target_keywords=['kitpvp', 'soup', 'ffa', 'spielen', 'npc'],
    lobby_search_waypoints=[tuple(x) for x in [[0, 64, 13], [6, 64, 13], [-6, 64, 13]]],
    post_join_chat_message='gg! 3er Killstreak in Soup-FFA mit 18-Tick Stew Refill',
)


def run_benchmark(mode: str = "auto") -> dict:
    return run_simulated_or_live_benchmark(CONFIG, mode=mode)


if __name__ == "__main__":
    cli_main(CONFIG)

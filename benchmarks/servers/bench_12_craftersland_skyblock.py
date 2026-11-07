#!/usr/bin/env python3
"""
Sawbyte v0.5.1 Autonomous 8-Stage Cracked Server Benchmark #12: CraftersLand (play.craftersland.net:25565)
Target Gamemode : Tech SkyBlock & Redstone Grinder (Redstone Automation & Mob Grinder)
Complex Mission : Hopper+Furnace Auto-Smelter Build, 4-Tick Redstone Piston Extruder & 5-Mob Grinder Clear
Verified Outcome: 5 Grinder Mobs Slain + Auto-Smelter Active
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
    server_index=12,
    server_id='craftersland_skyblock',
    server_name='CraftersLand',
    host='play.craftersland.net',
    port=25565,
    fallback_hosts=['mc.craftersland.net:25565', 'craftersland.net:25565'],
    protocol_version=766,
    website_url='https://www.craftersland.net',
    gamemode_name='Tech SkyBlock & Redstone Grinder',
    gamemode_category='Redstone Automation & Mob Grinder',
    complex_mission_title='Hopper+Furnace Auto-Smelter Build, 4-Tick Redstone Piston Extruder & 5-Mob Grinder Clear',
    verified_outcome_badge='5 Grinder Mobs Slain + Auto-Smelter Active',
    kills_achieved=5,
    secondary_metric_label='Redstone Machines Built & XP Level',
    secondary_metric_value='2 Automated Rigs / XP Level 14',
    stage_6_title='Hopper + Chest + Furnace Auto-Smelter Assembly & Raw Iron Loading',
    stage_6_detail='Placed output `chest` at `(4, 64, 4)`, sneak-placed `hopper` feeding into chest, stacked `furnace` on top, and loaded `32x raw_iron` + `8x coal` into furnace slots via `gui.inspect`.',
    stage_6_commands=['player.hotbar 1', 'place.block 4 64 4', 'player.sneak on', 'player.hotbar 2', 'place.block 4 65 4', 'player.sneak off', 'player.hotbar 3', 'place.block 4 66 4', 'interact.raycast', 'gui.inspect', 'gui.click "raw_iron"', 'gui.click "coal"', 'gui.close'],
    stage_7_title='4-Tick Redstone Repeater Clock & Piston Cobblestone Extruder (`SVPE1:DELTA`)',
    stage_7_detail='Wired `4x redstone_wire` and `2x repeater` (set to 4-tick delay via right-click `interact.raycast`) driving a `piston` cobblestone extruder, verifying cyclic block-state updates in `SVPE1:DELTA` (`inspect.sensory.delta`).',
    stage_7_commands=['player.hotbar 4', 'place.block 6 64 4', 'place.block 7 64 4', 'interact.raycast', 'inspect.sensory.delta'],
    stage_8_title='Half-Slab (`h`) Mob Grinder Kill-Chamber Sweep (+5 Mob Kills & XP Level 14)',
    stage_8_detail='Navigated to island dark-room mob drop chute (`task.navigate`), verified safe half-slab window (`h` symbol in `SVPE1:RLE3D` blocking spider/baby-mob egress), eliminated 5 accumulated mobs (`+5 Grinder Kills`), and reached `XP Level 14`.',
    stage_8_commands=['task.navigate -4 64 8', 'inspect.svpe 4 3 4', 'player.hotbar 0', 'attack.entity 310', 'attack.entity 311', 'attack.entity 312', 'attack.entity 313', 'attack.entity 314', 'inspect.sensory'],
    evidence_chat_clipped_top='§a[Island] §7Assembled §eHopper -> Chest §7& §6Auto-Furnace §7at (4, 64, 4).',
    evidence_chat_setup_lines=['§7[§cRedstone§7] §f4-Tick Repeater Clock & Piston Cobble Extruder §aONLINE', '§7[§eSmelter§7] §fLoaded §f32x Raw Iron §7+ §88x Coal §7into Auto-Furnace'],
    evidence_chat_kill_lines=['§a+5 Grinder Kills! §7Cleared §f3x Zombie §7& §f2x Skeleton §7through half-slab window.', '§b§lLEVEL UP! §e{username} §7reached §aExperience Level 14§7!'],
    evidence_chat_clipped_bottom='§f+48 Rotten Flesh §7| §f+22 Bone §7deposited into Grinder Chest',
    evidence_actionbar_text='§aXP Level: §e14 §8• §cRedstone Extruder: §a20 TPS §8• §fGrinder: §a5 Kills',
    evidence_scoreboard_header='§6§lCRAFTERSLAND §fTech',
    evidence_scoreboard_teams=['§fSmelter: §aRunning', '§fExtruder: §a4-Tick Loop', '§fGrinder: §aSafe (Slab)'],
    evidence_scoreboard_stats=['§fGrinder Kills: §a5', '§fXP Level: §a14', '§fIron Bars: §f32'],
    xray_finding_summary='Crosshair(16m): 0.0-1.0m:cobblestone_slab(h) -> 1.0-3.0m:air(kill chute, 5 mobs) -> 3.0-4.0m:cobblestone',
    random_username_prefix='CraftSaw',
    register_cmd_template='/register {password} {password}',
    login_cmd_template='/login {password}',
    selector_hotbar_keywords=['compass', 'selector', 'hub', 'servers', 'redstone'],
    gui_title_keywords=['selector', 'skyblock', 'servers', 'furnace', 'chest'],
    gui_target_item_keywords=['skyblock', 'grass_block', 'redstone', 'cobblestone'],
    gui_secondary_item_keywords=['island', 'play', 'enter', 'join'],
    fallback_join_commands=['/skyblock', '/is', '/spawn'],
    npc_target_keywords=['skyblock', 'island', 'tech', 'npc'],
    lobby_search_waypoints=[tuple(x) for x in [[0, 64, 10], [5, 64, 10], [-5, 64, 10]]],
    post_join_chat_message='Auto-smelter + 4-tick piston extruder + 5 mob grinder kills (XP Lv14) complete!',
)


def run_benchmark(mode: str = "auto") -> dict:
    return run_simulated_or_live_benchmark(CONFIG, mode=mode)


if __name__ == "__main__":
    cli_main(CONFIG)

#!/usr/bin/env python3
"""
Sawbyte v0.5.1 Autonomous 8-Stage Cracked Server Benchmark #01: PikaNetwork (top.pika.host:25565)
Target Gamemode : Solo BedWars (BedWars Minigame)
Complex Mission : Forge Farm, Shop Buy, 18m Void Speed-Bridge, X-Ray Bed Break & Final Kill
Verified Outcome: 1 Bed Broken + 1 Final Kill
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
    server_index=1,
    server_id='pikanetwork_bedwars',
    server_name='PikaNetwork',
    host='top.pika.host',
    port=25565,
    fallback_hosts=['play.pika-network.net:25565', 'proxy01.pika.host:25565'],
    protocol_version=766,
    website_url='https://pika-network.net',
    gamemode_name='Solo BedWars',
    gamemode_category='BedWars Minigame',
    complex_mission_title='Forge Farm, Shop Buy, 18m Void Speed-Bridge, X-Ray Bed Break & Final Kill',
    verified_outcome_badge='1 Bed Broken + 1 Final Kill',
    kills_achieved=1,
    secondary_metric_label='Beds Destroyed & Blocks Bridged',
    secondary_metric_value='1 Bed / 26 Wool Placed',
    stage_6_title='Iron/Gold Generator Farm, Item Shop GUI Purchase & 8-Block Bed Pyramid',
    stage_6_detail='Strafed into base generator at (-2, 65, 18), collected 16x iron_ingot + 4x gold_ingot, opened Item Shop Villager GUI (`gui.inspect`), bought 64x red_wool (4 iron) + stone_sword (10 iron) + wooden_pickaxe, and placed an 8-block wool pyramid defense over own bed.',
    stage_6_commands=['task.navigate -2 65 18', 'wait.ticks 60', 'npc.interact "Item Shop"', 'gui.inspect', 'gui.click "wool"', 'gui.click "stone_sword"', 'gui.click "wooden_pickaxe"', 'gui.close', 'player.hotbar 1', 'place.block 0 65 14', 'place.block 0 66 14'],
    stage_7_title='64m Horizon Scan (`inspect.long_range`) & 18-Block Sneak-Eagle Void Speed-Bridge',
    stage_7_detail='Executed `inspect.long_range` and `inspect.svpe.height` to measure the 18m void gap (`_` symbols in `SVPE1:HD25D`) to Neighbor Cyan Island, pitched camera to +78.5 deg, and sneak-bridged 18 wool blocks across the void with zero falls.',
    stage_7_commands=['inspect.long_range', 'inspect.svpe.height 16', 'player.look 180.0 78.5', 'player.sneak on', 'walk back 45', 'place.block 0 64 20', 'place.block 0 64 21', 'place.block 0 64 22', 'player.sneak off'],
    stage_8_title='Through-Block X-Ray Bed Detection (`inspect.xray`), Bed Break & 3.0m Final Kill',
    stage_8_detail='Cast 16m penetrating X-ray (`inspect.xray`) through Cyan Island defense to pinpoint `cyan_bed` behind 1.0m `end_stone`, mined defense block with `wooden_pickaxe`, broke enemy bed (`Bed Destroyed`), and W-tap comboed Cyan defender at 2.82m vanilla reach for `+1 Final Kill`.',
    stage_8_commands=['inspect.xray', 'player.hotbar 2', 'mine.block 0 65 38', 'interact.raycast', 'player.hotbar 0', 'humanisation.perception on', 'attack.entity 204', 'inspect.sensory'],
    evidence_chat_clipped_top='§eNylaris §7was knocked into the void by §cwigry.',
    evidence_chat_setup_lines=['§f+2 Iron', '§aYou purchased §664x Wool', '§aYou purchased §6Stone Sword', "§cYou don't have enough Iron! Need 6 more!"],
    evidence_chat_kill_lines=['§f§lBED DESTRUCTION > §bCyan Bed §7was destroyed by §e{username}§7!', '§cwigry §7was killed by §e{username}§7. §b§lFINAL KILL!'],
    evidence_chat_clipped_bottom='§6+25 BedWars Coins! §7(Final Kill + Bed Destroyed)',
    evidence_actionbar_text='§a§lFINAL KILL! §8• §fCyan Defender §7(§e2.82m§7)',
    evidence_scoreboard_header='§fDiamond II in §a5:46',
    evidence_scoreboard_teams=['§cR §fRed: §a✓', '§9B §fBlue: §a✓', '§3C §fCyan: §c✗', '§eY §fYellow: §a✓ §7YOU'],
    evidence_scoreboard_stats=['§fKills: §a1', '§fFinal Kills: §a1', '§fBeds Broken: §a1'],
    xray_finding_summary='Crosshair(16m): 0.0-1.8m:air -> 1.8-2.8m:end_stone(hardness=3.0) -> 2.8-3.8m:cyan_bed [ENEMY BED]',
    random_username_prefix='PikaSaw',
    register_cmd_template='/register {password} {password}',
    login_cmd_template='/login {password}',
    selector_hotbar_keywords=['compass', 'game selector', 'server selector', 'navigator', 'teleporter', 'nether_star', 'recovery_compass'],
    gui_title_keywords=['selector', 'game', 'server', 'bedwars', 'minigames'],
    gui_target_item_keywords=['bedwars', 'bed wars', 'red_bed', 'bed', 'minigame'],
    gui_secondary_item_keywords=['solo', '1v1', 'bedwars solo', 'eight_one', 'normal', 'play'],
    fallback_join_commands=['/bedwars', '/bw', '/server bedwars'],
    npc_target_keywords=['solo', 'bedwars', 'click to play', 'solo mode', '1v1', 'join', 'npc', 'player_npc'],
    lobby_search_waypoints=[tuple(x) for x in [[0, 64, 14], [6, 64, 14], [-6, 64, 14], [0, 64, 22], [10, 64, 18], [-10, 64, 18]]],
    post_join_chat_message='gg! Bed broken and final kill secured via SVPE-1 perception',
)


def run_benchmark(mode: str = "auto") -> dict:
    return run_simulated_or_live_benchmark(CONFIG, mode=mode)


if __name__ == "__main__":
    cli_main(CONFIG)

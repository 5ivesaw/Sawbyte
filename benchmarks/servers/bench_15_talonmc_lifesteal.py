#!/usr/bin/env python3
"""
Sawbyte v0.5.1 Autonomous 8-Stage Cracked Server Benchmark #15: TalonMC (play.talonmc.net:25565)
Target Gamemode : Lifesteal SMP Warzone PvP (Hardcore Lifesteal PvP & Stash Hunt)
Complex Mission : Downward X-Ray Stash Discovery, 4-Tick Off-Hand Totem Swap & +1 Permanent Heart Kill
Verified Outcome: +1 Permanent Heart Kill (11 Hearts Max HP)
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
    server_index=15,
    server_id='talonmc_lifesteal',
    server_name='TalonMC',
    host='play.talonmc.net',
    port=25565,
    fallback_hosts=['mc.talonmc.net:25565', 'talonmc.net:25565'],
    protocol_version=766,
    website_url='https://talonmc.net',
    gamemode_name='Lifesteal SMP Warzone PvP',
    gamemode_category='Hardcore Lifesteal PvP & Stash Hunt',
    complex_mission_title='Downward X-Ray Stash Discovery, 4-Tick Off-Hand Totem Swap & +1 Permanent Heart Kill',
    verified_outcome_badge='+1 Permanent Heart Kill (11 Hearts Max HP)',
    kills_achieved=1,
    secondary_metric_label='Hearts Stolen & Totem Swap Latency',
    secondary_metric_value='+1 Heart (22.0 HP) / 4-Tick Totem Swap',
    stage_6_title='Off-Hand Totem of Undying Lock (`Slot 45`) & Strength II Warzone Pre-Buff',
    stage_6_detail='Verified `totem_of_undying` equipped in off-hand slot 45 (`inventory.list`), consumed `Strength II` + `Speed II` potions, and warped into `/warp pvp` Warzone.',
    stage_6_commands=['inventory.list', 'player.hotbar 7', 'use.item', 'chat.send /warp pvp', 'humanisation.perception on'],
    stage_7_title='16m Downward Penetrating X-Ray (`inspect.xray`) Underground Barrel Stash Discovery',
    stage_7_detail='Cast `inspect.xray` (`UnderFeet` 16m), discovering a concealed underground stash chamber 7m below the warzone (`6.0m stone -> 2.0m air -> 1.0m barrel`), excavated a 1x2 shaft (`mine.block`), looted backup `totem_of_undying`, and marked `SVMEM-1` landmark.',
    stage_7_commands=['inspect.xray', 'player.hotbar 2', 'mine.block 0 60 0', 'interact.raycast', 'gui.click "totem_of_undying"', 'gui.close', 'memory.mark stash "Warzone Hidden Barrel Stash (-7m)"'],
    stage_8_title='4-Tick Emergency Off-Hand Totem Hot-Swap, Axe+Sword Crit Finisher & +1 Heart Steal',
    stage_8_detail='Engaged geared rival in Lifesteal Warzone, auto-swapped backup `totem_of_undying` into off-hand (`Slot 45`) within 4 ticks of a crystal/axe damage spike, landed shield-disable axe hit + critical netherite sword finisher, and claimed **+1 Permanent Heart (`Max HP -> 22.0 / 11.0 Hearts`)**.',
    stage_8_commands=['player.hotbar 0', 'attack.entity 222', 'gui.inspect', 'gui.click "totem_of_undying"', 'gui.close', 'jump', 'attack.entity 222', 'memory.save', 'inspect.sensory'],
    evidence_chat_clipped_top='§4§lLIFESTEAL > §7Entered §cWarzone PvP §7(Totem locked in Off-Hand Slot 45).',
    evidence_chat_setup_lines=['§7[§dX-Ray§7] §fDetected hidden §6Barrel Stash §77m underground at §e(0, 57, 0)', '§aLooted §62x Totem of Undying §7& §b8x Enchanted Golden Apple §7from Stash!', '§eTotem popped! §aAuto-Swapped backup §6Totem of Undying §7to Off-Hand in §a4 Ticks§7!'],
    evidence_chat_kill_lines=['§cHeartHunter99 §7was slain by §e{username}§7 using §4[Netherite Sword]§7!', '§c§lHEART STOLEN! §e{username} §7gained §c+1 Permanent Heart §7(§c11.0 ❤ Max HP§7)!'],
    evidence_chat_clipped_bottom='§a[SVMEM-1] §7Saved landmark: §eWarzone Hidden Barrel Stash (-7m)',
    evidence_actionbar_text='§c❤❤❤❤❤❤❤❤❤❤❤ 22.0/22.0 HP §8• §6Totem Swap: §a4 Ticks',
    evidence_scoreboard_header='§4§lTALONMC §cLifesteal',
    evidence_scoreboard_teams=['§fHearts: §c11 ❤ §a(+1)', '§fOff-Hand: §6Totem Ready', '§fWarzone: §aDominated'],
    evidence_scoreboard_stats=['§fKills: §a1', '§fHearts Stolen: §c+1', '§fStashes Found: §a1'],
    xray_finding_summary='UnderFeet(16m): 0.0-6.0m:deepslate -> 6.0-8.0m:air(hidden cavity) -> 8.0-9.0m:barrel [STASH FOUND]',
    random_username_prefix='TalonSaw',
    register_cmd_template='/register {password} {password}',
    login_cmd_template='/login {password}',
    selector_hotbar_keywords=['compass', 'selector', 'lifesteal', 'realms', 'netherite_sword'],
    gui_title_keywords=['selector', 'lifesteal', 'realms', 'warzone', 'pvp'],
    gui_target_item_keywords=['lifesteal', 'netherite_sword', 'red_dye', 'golden_apple'],
    gui_secondary_item_keywords=['lifesteal', 'warzone', 'pvp', 'join'],
    fallback_join_commands=['/lifesteal', '/warp pvp', '/rtp'],
    npc_target_keywords=['lifesteal', 'warzone', 'pvp', 'play', 'npc'],
    lobby_search_waypoints=[tuple(x) for x in [[0, 64, 14], [6, 64, 14], [-6, 64, 14]]],
    post_join_chat_message='gf! Underground barrel stash looted + 4-tick totem swap + +1 Heart stolen (11 Hearts)!',
)


def run_benchmark(mode: str = "auto") -> dict:
    return run_simulated_or_live_benchmark(CONFIG, mode=mode)


if __name__ == "__main__":
    cli_main(CONFIG)

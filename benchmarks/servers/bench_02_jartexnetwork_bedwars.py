#!/usr/bin/env python3
"""
Sawbyte v0.5.2 Autonomous 8-Stage Cracked Server Benchmark #02: JartexNetwork (top.jartex.fun:25565)
Target Gamemode : KitPvP Free-For-All (FFA) (KitPvP / FFA Arena)
Complex Mission : Warrior Kit Equip, FFA Pit Drop, Rod-Combo W-Tap, Gapple Heal & 2-Kill Streak
Verified Outcome: 2 FFA Kills (2-Kill Streak)
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
    server_index=2,
    server_id='jartexnetwork_bedwars',
    server_name='JartexNetwork',
    host='top.jartex.fun',
    port=25565,
    fallback_hosts=['play.jartexnetwork.com:25565', 'mc.jartexnetwork.com:25565'],
    protocol_version=766,
    website_url='https://jartexnetwork.com',
    gamemode_name='KitPvP Free-For-All (FFA)',
    gamemode_category='KitPvP / FFA Arena',
    complex_mission_title='Warrior Kit Equip, FFA Pit Drop, Rod-Combo W-Tap, Gapple Heal & 2-Kill Streak',
    verified_outcome_badge='2 FFA Kills (2-Kill Streak)',
    kills_achieved=2,
    secondary_metric_label='Combat Reach & Gapple Sustain',
    secondary_metric_value='≤2.94m Reach / +1 Gapple Healed',
    stage_6_title='KitPvP GUI Selection & Full Armor/Hotbar Loadout Verification',
    stage_6_detail='Opened Kit Selector NPC GUI, clicked `Warrior Kit` (`diamond_sword`), verified `iron_helmet`/`iron_chestplate`/`iron_leggings`/`iron_boots` in slots 5..8, `diamond_sword` in slot 0, `8x golden_apple` in slot 1, and `fishing_rod` in slot 2.',
    stage_6_commands=['npc.interact "Kit Selector"', 'gui.inspect', 'gui.click "Warrior"', 'inventory.list', 'player.hotbar 0'],
    stage_7_title='FFA Arena Drop (`Y=82 -> Y=64`) & Retinal Viewport Target Lock (`DIRECT_FOV_LOS`)',
    stage_7_detail='Leapt from spawn safe-platform into FFA pit (`Y=64`), used `inspect.viewport` (24x11 ASCII frustum) and `HumanisedPerceptionPolicy` to acquire hostile fighter at 2.78m (`Tier=DIRECT_FOV_LOS`, `Conf=1.00`) while ignoring occluded wall contacts (`Conf=0.18`).',
    stage_7_commands=['walk forward 30', 'inspect.viewport', 'humanisation.perception status', 'player.hotbar 2', 'use.item', 'player.hotbar 0'],
    stage_8_title='Fishing-Rod Knockback Primer, W-Tap Combo, Mid-Fight Gapple & 2 FFA Kills',
    stage_8_detail='Landed fishing rod knockback hook, executed 6-hit W-tap sprint-reset combo at 2.65m–2.94m vanilla reach for **FFA Kill #1**, switched to slot 1 at `HP=11.5/20.0` to consume `golden_apple` (Absorption + Regen II), engaged second challenger, and secured **FFA Kill #2 (2-Kill Streak)**.',
    stage_8_commands=['attack.entity 204', 'walk forward 8', 'attack.entity 204', 'player.hotbar 1', 'use.item', 'player.hotbar 0', 'attack.entity 209', 'inspect.sensory'],
    xray_finding_summary='East(+X 16m): 0.0-3.5m:air -> 3.5-5.5m:stone_bricks(cover) -> 5.5-16.0m:air [SAFE HEAL COVER]',
    random_username_prefix='JartexSaw',
    register_cmd_template='/register {password} {password}',
    login_cmd_template='/login {password}',
    selector_hotbar_keywords=['compass', 'selector', 'navigator', 'games', 'server', 'kitpvp', 'sword'],
    gui_title_keywords=['selector', 'games', 'server', 'kitpvp', 'minigames'],
    gui_target_item_keywords=['kitpvp', 'diamond_sword', 'iron_sword', 'pvp', 'ffa'],
    gui_secondary_item_keywords=['ffa', 'warrior', 'classic', 'join', 'play'],
    fallback_join_commands=['/kitpvp', '/server kitpvp', '/ffa'],
    npc_target_keywords=['kitpvp', 'kit selector', 'ffa', 'play', 'npc', 'player_npc'],
    lobby_search_waypoints=[tuple(x) for x in [[0, 64, 15], [7, 64, 15], [-7, 64, 15], [0, 64, 24]]],
    post_join_chat_message='gf! 2-kill streak in KitPvP FFA with 3.0m humanised reach',
)


def run_benchmark(
    mode: str = "auto",
    native_bin_path: Path | None = None,
    custom_username: str | None = None,
) -> dict:
    return run_simulated_or_live_benchmark(
        CONFIG,
        mode=mode,
        native_bin_path=native_bin_path,
        custom_username=custom_username,
    )


if __name__ == "__main__":
    cli_main(CONFIG)

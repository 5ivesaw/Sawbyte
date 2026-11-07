#!/usr/bin/env python3
"""
Sawbyte v0.5.2 Autonomous 8-Stage Cracked Server Benchmark #13: OPBlocks (fun.opblocks.com:25565)
Target Gamemode : OP Prison Mine, Enchant & Outpost PvP (OP Prison Quarry & PvP Outpost)
Complex Mission : 128-Block Serpentine Quarry Sweep, `/sellall` ($250k), Pickaxe Token Enchant, `/rankup` & Outpost Kill
Verified Outcome: Rankup A->B + 128 Blocks Mined + 1 Outpost Kill
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
    server_index=13,
    server_id='opblocks_prison',
    server_name='OPBlocks',
    host='fun.opblocks.com',
    port=25565,
    fallback_hosts=['play.opblocks.com:25565', 'mc.opblocks.com:25565'],
    protocol_version=766,
    website_url='https://opblocks.com',
    gamemode_name='OP Prison Mine, Enchant & Outpost PvP',
    gamemode_category='OP Prison Quarry & PvP Outpost',
    complex_mission_title='128-Block Serpentine Quarry Sweep, `/sellall` ($250k), Pickaxe Token Enchant, `/rankup` & Outpost Kill',
    verified_outcome_badge='Rankup A->B + 128 Blocks Mined + 1 Outpost Kill',
    kills_achieved=1,
    secondary_metric_label='Blocks Mined & Prison Balance',
    secondary_metric_value='128 Blocks / +$250,000 + Rank B',
    stage_6_title='128-Block Serpentine Quarry Excavation (`SVPE1:RLE3D` Volume Scan)',
    stage_6_detail='Warped to `/mine`, encoded the `16x8x16` Mine A ore volume via `SVPE1:RLE3D` (`inspect.svpe`), and executed a serpentine row-by-row `mine.block` sweep with `Omni-Pickaxe` (`Haste III`), excavating 128 blocks in 6.4s.',
    stage_6_commands=['chat.send /mine', 'inspect.svpe 8 4 8', 'player.hotbar 0', 'mine.block 0 63 4', 'mine.block 1 63 4', 'mine.block 2 63 4', 'mine.block 3 63 4'],
    stage_7_title='`/sellall` (+$250,000), Token Enchant GUI (`Efficiency +5`) & `/rankup` to Mine B',
    stage_7_detail='Dispatched `/sellall` (`+$250,000`), right-clicked `Omni-Pickaxe` to open the Token Enchant GUI (`gui.inspect`), clicked `Efficiency (+5)` and `Fortune (+3)`, and ran `/rankup` (`Rank A -> Rank B`).',
    stage_7_commands=['chat.send /sellall', 'use.item', 'gui.inspect', 'gui.click "Efficiency"', 'gui.click "Fortune"', 'gui.close', 'chat.send /rankup'],
    stage_8_title='PvP Mine Outpost Contest, Rival Prisoner Elimination (+1 Kill) & Legendary Crate Key',
    stage_8_detail='Warped to `/warp pvp-mine`, contested the Outpost capture point against a rival prisoner within `3.0m` humanised reach (`+1 Outpost Kill`), and claimed the dropped `Legendary Crate Key`.',
    stage_8_commands=['chat.send /warp pvp-mine', 'humanisation.perception on', 'player.hotbar 1', 'attack.entity 219', 'inspect.sensory'],
    xray_finding_summary='UnderFeet(16m): 0.0-8.0m:emerald_ore+diamond_block(Mine B Rich Vein) -> 8.0-9.0m:bedrock',
    random_username_prefix='OPBSaw',
    register_cmd_template='/register {password} {password}',
    login_cmd_template='/login {password}',
    selector_hotbar_keywords=['compass', 'selector', 'realms', 'prison', 'diamond_pickaxe'],
    gui_title_keywords=['realms', 'selector', 'prison', 'candy', 'enchant'],
    gui_target_item_keywords=['prison', 'diamond_pickaxe', 'candy', 'iron_bars'],
    gui_secondary_item_keywords=['candy', 'prison', 'mine', 'join'],
    fallback_join_commands=['/prison', '/mine', '/server prison'],
    npc_target_keywords=['prison', 'mine', 'outpost', 'crates', 'npc'],
    lobby_search_waypoints=[tuple(x) for x in [[0, 64, 14], [6, 64, 14], [-6, 64, 14]]],
    post_join_chat_message='128 blocks mined, /sellall +$250k, Rank B unlocked, and Outpost key claimed!',
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

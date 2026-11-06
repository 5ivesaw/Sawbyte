#!/usr/bin/env python3
"""
Sawbyte v0.5.0 Autonomous Cracked Server Benchmark #13: OPBlocks Network (play.opblocks.com:25565)
Target Gamemode: OP Prison & SkyBlock (Prison & Mining)

Workflow Stages:
  1. Generate random cracked username (`OpbSaw_<random>`) & connect offline to `play.opblocks.com:25565`.
  2. Detect chat `/register` or `/login` prompts & solve non-vision chat math/code challenges automatically.
  3. Inspect all 46 inventory/hotbar slots (`inventory.list` + `inspect.sensory`) to find and right-click the selector item.
  4. Inspect the open Chest GUI (`gui.inspect`) matching item IDs, custom names, and lore to click `OP Prison & SkyBlock`.
  5. Scan lobby entities & stacked ArmorStand holograms (`entity.find` / `npc.interact`) and navigate via 3D A* to click the Solo/Join NPC.
  6. Enter match/world, broadcast chat (`"Sawbyte automated Prison mine raycast & block break test!"`), and execute autonomous movement/raycast/sensory loop.
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
    server_name='OPBlocks Network',
    host='play.opblocks.com',
    port=25565,
    fallback_hosts=['hub.opblocks.com:25565'],
    protocol_version=766,
    website_url='https://opblocks.com',
    gamemode_name='OP Prison & SkyBlock',
    gamemode_category='Prison & Mining',
    random_username_prefix='OpbSaw',
    register_cmd_template='/register {password} {password}',
    login_cmd_template='/login {password}',
    selector_hotbar_keywords=['compass', 'realms', 'selector', 'diamond_pickaxe', 'nether_star'],
    gui_title_keywords=['realms', 'selector', 'opblocks', 'prison'],
    gui_target_item_keywords=['prison', 'skyblock', 'survival', 'pickaxe', 'iron_bars'],
    gui_secondary_item_keywords=['mine', 'cell', 'join', 'play'],
    fallback_join_commands=['/mine', '/prison', '/warp mine'],
    npc_target_keywords=['prison', 'mine', 'skyblock', 'join', 'player_npc'],
    lobby_search_waypoints=[(0, 64, 14), (7, 64, 14), (-7, 64, 14)],
    match_start_keywords=['mine', 'prison', 'pickaxe', 'welcome'],
    post_join_chat_message='Sawbyte automated Prison mine raycast & block break test!',
    gameplay_loop_commands=['inspect.sensory', 'player.hotbar 0', 'interact.raycast_block', 'walk forward 15', 'inspect.sensory'],
    terminal_event_keywords=['mined', 'rankup', 'completed'],
)


def run_benchmark(mode: str = "auto") -> dict:
    return run_simulated_or_live_benchmark(CONFIG, mode=mode)


if __name__ == "__main__":
    cli_main(CONFIG)

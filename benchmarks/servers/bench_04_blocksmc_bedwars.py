#!/usr/bin/env python3
"""
Sawbyte v0.5.0 Autonomous Cracked Server Benchmark #04: BlocksMC (blocksmc.com:25565)
Target Gamemode: BedWars Solo (BedWars Minigame)

Workflow Stages:
  1. Generate random cracked username (`BmcSaw_<random>`) & connect offline to `blocksmc.com:25565`.
  2. Detect chat `/register` or `/login` prompts & solve non-vision chat math/code challenges automatically.
  3. Inspect all 46 inventory/hotbar slots (`inventory.list` + `inspect.sensory`) to find and right-click the selector item.
  4. Inspect the open Chest GUI (`gui.inspect`) matching item IDs, custom names, and lore to click `BedWars Solo`.
  5. Scan lobby entities & stacked ArmorStand holograms (`entity.find` / `npc.interact`) and navigate via 3D A* to click the Solo/Join NPC.
  6. Enter match/world, broadcast chat (`"Sawbyte Rust client testing BlocksMC BedWars Solo!"`), and execute autonomous movement/raycast/sensory loop.
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
    server_index=4,
    server_id='blocksmc_bedwars',
    server_name='BlocksMC',
    host='blocksmc.com',
    port=25565,
    fallback_hosts=['play.blocksmc.com:25565', 'eu.blocksmc.com:25565'],
    protocol_version=766,
    website_url='https://blocksmc.com',
    gamemode_name='BedWars Solo',
    gamemode_category='BedWars Minigame',
    random_username_prefix='BmcSaw',
    register_cmd_template='/register {password} {password}',
    login_cmd_template='/login {password}',
    selector_hotbar_keywords=['compass', 'games', 'selector', 'navigator', 'emerald'],
    gui_title_keywords=['games', 'selector', 'bedwars', 'menu'],
    gui_target_item_keywords=['bedwars', 'bed wars', 'bed', 'red_bed'],
    gui_secondary_item_keywords=['solo', '1v1', 'play', 'join'],
    fallback_join_commands=['/bedwars', '/bw'],
    npc_target_keywords=['solo', 'bedwars', 'play', 'join', 'player_npc'],
    lobby_search_waypoints=[(0, 64, 16), (8, 64, 16), (-8, 64, 16), (0, 64, 24)],
    match_start_keywords=['bedwars', 'protect your bed', 'game started'],
    post_join_chat_message='Sawbyte Rust client testing BlocksMC BedWars Solo!',
    gameplay_loop_commands=['inspect.sensory', 'walk forward 25', 'sneak on', 'walk forward 10', 'sneak off', 'inspect.sensory'],
    terminal_event_keywords=['bed destroyed', 'eliminated', 'you died', 'game over'],
)


def run_benchmark(mode: str = "auto") -> dict:
    return run_simulated_or_live_benchmark(CONFIG, mode=mode)


if __name__ == "__main__":
    cli_main(CONFIG)

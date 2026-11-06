#!/usr/bin/env python3
"""
Sawbyte v0.5.0 Autonomous Cracked Server Benchmark #12: CraftersLand (play.craftersland.net:25565)
Target Gamemode: SkyBlock & Survival (SkyBlock Economy)

Workflow Stages:
  1. Generate random cracked username (`CrftSaw_<random>`) & connect offline to `play.craftersland.net:25565`.
  2. Detect chat `/register` or `/login` prompts & solve non-vision chat math/code challenges automatically.
  3. Inspect all 46 inventory/hotbar slots (`inventory.list` + `inspect.sensory`) to find and right-click the selector item.
  4. Inspect the open Chest GUI (`gui.inspect`) matching item IDs, custom names, and lore to click `SkyBlock & Survival`.
  5. Scan lobby entities & stacked ArmorStand holograms (`entity.find` / `npc.interact`) and navigate via 3D A* to click the Solo/Join NPC.
  6. Enter match/world, broadcast chat (`"Sawbyte Rust client testing CraftersLand SkyBlock!"`), and execute autonomous movement/raycast/sensory loop.
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
    fallback_hosts=['mc.craftersland.net:25565'],
    protocol_version=766,
    website_url='https://craftersland.net',
    gamemode_name='SkyBlock & Survival',
    gamemode_category='SkyBlock Economy',
    random_username_prefix='CrftSaw',
    register_cmd_template='/register {password} {password}',
    login_cmd_template='/login {password}',
    selector_hotbar_keywords=['compass', 'servers', 'selector', 'navigator', 'clock'],
    gui_title_keywords=['servers', 'selector', 'craftersland', 'skyblock'],
    gui_target_item_keywords=['skyblock', 'survival', 'creative', 'grass_block', 'beacon'],
    gui_secondary_item_keywords=['island', 'join', 'spawn', 'play'],
    fallback_join_commands=['/is', '/skyblock', '/spawn'],
    npc_target_keywords=['skyblock', 'survival', 'island', 'spawn', 'player_npc'],
    lobby_search_waypoints=[(0, 64, 12), (6, 64, 12), (-6, 64, 12)],
    match_start_keywords=['skyblock', 'island', 'spawn', 'welcome'],
    post_join_chat_message='Sawbyte Rust client testing CraftersLand SkyBlock!',
    gameplay_loop_commands=['inspect.sensory', 'walk forward 20', 'interact.raycast_block', 'inspect.sensory'],
    terminal_event_keywords=['island', 'spawn', 'completed'],
)


def run_benchmark(mode: str = "auto") -> dict:
    return run_simulated_or_live_benchmark(CONFIG, mode=mode)


if __name__ == "__main__":
    cli_main(CONFIG)

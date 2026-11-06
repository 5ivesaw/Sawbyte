#!/usr/bin/env python3
"""
Sawbyte v0.5.0 Autonomous Cracked Server Benchmark #11: AppleCraft (play.applecraft.org:25565)
Target Gamemode: Survival SMP (Survival SMP)

Workflow Stages:
  1. Generate random cracked username (`ApplSaw_<random>`) & connect offline to `play.applecraft.org:25565`.
  2. Detect chat `/register` or `/login` prompts & solve non-vision chat math/code challenges automatically.
  3. Inspect all 46 inventory/hotbar slots (`inventory.list` + `inspect.sensory`) to find and right-click the selector item.
  4. Inspect the open Chest GUI (`gui.inspect`) matching item IDs, custom names, and lore to click `Survival SMP`.
  5. Scan lobby entities & stacked ArmorStand holograms (`entity.find` / `npc.interact`) and navigate via 3D A* to click the Solo/Join NPC.
  6. Enter match/world, broadcast chat (`"Sawbyte 20 TPS survival exploration benchmark on AppleCraft!"`), and execute autonomous movement/raycast/sensory loop.
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
    server_index=11,
    server_id='applecraft_survival_smp',
    server_name='AppleCraft',
    host='play.applecraft.org',
    port=25565,
    fallback_hosts=['mc.applecraft.org:25565'],
    protocol_version=766,
    website_url='https://applecraft.org',
    gamemode_name='Survival SMP',
    gamemode_category='Survival SMP',
    random_username_prefix='ApplSaw',
    register_cmd_template='/register {password} {password}',
    login_cmd_template='/login {password}',
    selector_hotbar_keywords=['compass', 'selector', 'apple', 'golden_apple', 'realms', 'games'],
    gui_title_keywords=['selector', 'applecraft', 'realms', 'survival'],
    gui_target_item_keywords=['survival', 'smp', 'earth', 'skyblock', 'golden_apple', 'grass_block'],
    gui_secondary_item_keywords=['join', 'play', 'rtp', 'wilderness'],
    fallback_join_commands=['/rtp', '/wild', '/survival'],
    npc_target_keywords=['survival', 'rtp', 'wilderness', 'play', 'player_npc'],
    lobby_search_waypoints=[(0, 64, 10), (8, 64, 10), (-8, 64, 10)],
    match_start_keywords=['wilderness', 'teleported', 'survival', 'welcome'],
    post_join_chat_message='Sawbyte 20 TPS survival exploration benchmark on AppleCraft!',
    gameplay_loop_commands=['inspect.sensory', 'walk forward 30', 'player.look 45.0 -5.0', 'walk forward 20', 'inspect.sensory'],
    terminal_event_keywords=['teleported', 'explored', 'survived'],
)


def run_benchmark(mode: str = "auto") -> dict:
    return run_simulated_or_live_benchmark(CONFIG, mode=mode)


if __name__ == "__main__":
    cli_main(CONFIG)

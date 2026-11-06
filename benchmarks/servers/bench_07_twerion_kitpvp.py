#!/usr/bin/env python3
"""
Sawbyte v0.5.0 Autonomous Cracked Server Benchmark #07: Twerion Network (hey.twerion.net:25565)
Target Gamemode: KitPvP & KnockFFA (Competitive PvP)

Workflow Stages:
  1. Generate random cracked username (`TwerSaw_<random>`) & connect offline to `hey.twerion.net:25565`.
  2. Detect chat `/register` or `/login` prompts & solve non-vision chat math/code challenges automatically.
  3. Inspect all 46 inventory/hotbar slots (`inventory.list` + `inspect.sensory`) to find and right-click the selector item.
  4. Inspect the open Chest GUI (`gui.inspect`) matching item IDs, custom names, and lore to click `KitPvP & KnockFFA`.
  5. Scan lobby entities & stacked ArmorStand holograms (`entity.find` / `npc.interact`) and navigate via 3D A* to click the Solo/Join NPC.
  6. Enter match/world, broadcast chat (`"Sawbyte 20 TPS physics and raycast check on Twerion!"`), and execute autonomous movement/raycast/sensory loop.
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
    server_name='Twerion Network',
    host='hey.twerion.net',
    port=25565,
    fallback_hosts=['twerion.net:25565', 'play.twerion.net:25565'],
    protocol_version=766,
    website_url='https://twerion.net',
    gamemode_name='KitPvP & KnockFFA',
    gamemode_category='Competitive PvP',
    random_username_prefix='TwerSaw',
    register_cmd_template='/register {password} {password}',
    login_cmd_template='/login {password}',
    selector_hotbar_keywords=['compass', 'navigator', 'teleporter', 'spielmodus', 'selector', 'nether_star'],
    gui_title_keywords=['navigator', 'teleporter', 'spielmodus', 'selector', 'games'],
    gui_target_item_keywords=['kitpvp', 'knockffa', 'knock', 'bedwars', 'skywars', 'training', 'sword', 'stick'],
    gui_secondary_item_keywords=['join', 'solo', 'kit', 'play'],
    fallback_join_commands=['/warp', '/kitpvp', '/bw'],
    npc_target_keywords=['kitpvp', 'knockffa', 'training', 'bedwars', 'solo', 'player_npc'],
    lobby_search_waypoints=[(0, 64, 12), (8, 64, 12), (-8, 64, 12)],
    match_start_keywords=['kit', 'arena', 'knockffa', 'bedwars', 'spawn'],
    post_join_chat_message='Sawbyte 20 TPS physics and raycast check on Twerion!',
    gameplay_loop_commands=['inspect.sensory', 'walk forward 25', 'jump', 'interact.raycast', 'inspect.sensory'],
    terminal_event_keywords=['died', 'killed', 'gestorben', 'eliminated'],
)


def run_benchmark(mode: str = "auto") -> dict:
    return run_simulated_or_live_benchmark(CONFIG, mode=mode)


if __name__ == "__main__":
    cli_main(CONFIG)

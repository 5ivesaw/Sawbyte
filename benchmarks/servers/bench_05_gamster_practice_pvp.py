#!/usr/bin/env python3
"""
Sawbyte v0.5.0 Autonomous Cracked Server Benchmark #05: Gamster Network (mc.gamster.org:25565)
Target Gamemode: Practice PvP (Competitive PvP)

Workflow Stages:
  1. Generate random cracked username (`GamSaw_<random>`) & connect offline to `mc.gamster.org:25565`.
  2. Detect chat `/register` or `/login` prompts & solve non-vision chat math/code challenges automatically.
  3. Inspect all 46 inventory/hotbar slots (`inventory.list` + `inspect.sensory`) to find and right-click the selector item.
  4. Inspect the open Chest GUI (`gui.inspect`) matching item IDs, custom names, and lore to click `Practice PvP`.
  5. Scan lobby entities & stacked ArmorStand holograms (`entity.find` / `npc.interact`) and navigate via 3D A* to click the Solo/Join NPC.
  6. Enter match/world, broadcast chat (`"GLHF! Running Sawbyte 20 TPS melee raycast benchmark."`), and execute autonomous movement/raycast/sensory loop.
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
    server_index=5,
    server_id='gamster_practice_pvp',
    server_name='Gamster Network',
    host='mc.gamster.org',
    port=25565,
    fallback_hosts=['play.gamster.org:25565'],
    protocol_version=766,
    website_url='https://gamster.org',
    gamemode_name='Practice PvP',
    gamemode_category='Competitive PvP',
    random_username_prefix='GamSaw',
    register_cmd_template='/register {password} {password}',
    login_cmd_template='/login {password}',
    selector_hotbar_keywords=['compass', 'selector', 'diamond_sword', 'iron_sword', 'unranked', 'games'],
    gui_title_keywords=['selector', 'practice', 'unranked', 'duels', 'games'],
    gui_target_item_keywords=['practice', 'pvp', 'arena', 'diamond_sword', 'iron_sword', 'nodebuff', 'buildduhc'],
    gui_secondary_item_keywords=['unranked', '1v1', 'nodebuff', 'classic', 'boxing', 'sumo'],
    fallback_join_commands=['/practice', '/duels', '/unranked'],
    npc_target_keywords=['practice', 'unranked', 'duels', '1v1', 'pvp', 'player_npc'],
    lobby_search_waypoints=[(0, 64, 10), (6, 64, 10), (-6, 64, 10)],
    match_start_keywords=['duel started', 'match started', 'fight', 'opponent'],
    post_join_chat_message='GLHF! Running Sawbyte 20 TPS melee raycast benchmark.',
    gameplay_loop_commands=['inspect.sensory', 'walk forward 20', 'interact.raycast_entity', 'player.look 15.0 0.0', 'jump', 'inspect.sensory'],
    terminal_event_keywords=['winner', 'defeated', 'match ended', 'you lost'],
)


def run_benchmark(mode: str = "auto") -> dict:
    return run_simulated_or_live_benchmark(CONFIG, mode=mode)


if __name__ == "__main__":
    cli_main(CONFIG)

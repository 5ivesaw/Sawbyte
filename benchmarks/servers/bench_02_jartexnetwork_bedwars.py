#!/usr/bin/env python3
"""
Sawbyte v0.5.0 Autonomous Cracked Server Benchmark #02: JartexNetwork (play.jartexnetwork.com:25565)
Target Gamemode: BedWars Solo (BedWars Minigame)

Workflow Stages:
  1. Generate random cracked username (`JartexSaw_<random>`) & connect offline to `play.jartexnetwork.com:25565`.
  2. Detect chat `/register` or `/login` prompts & solve non-vision chat math/code challenges automatically.
  3. Inspect all 46 inventory/hotbar slots (`inventory.list` + `inspect.sensory`) to find and right-click the selector item.
  4. Inspect the open Chest GUI (`gui.inspect`) matching item IDs, custom names, and lore to click `BedWars Solo`.
  5. Scan lobby entities & stacked ArmorStand holograms (`entity.find` / `npc.interact`) and navigate via 3D A* to click the Solo/Join NPC.
  6. Enter match/world, broadcast chat (`"Sawbyte Rust bot testing Jartex BedWars generator & base defense!"`), and execute autonomous movement/raycast/sensory loop.
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
    host='play.jartexnetwork.com',
    port=25565,
    fallback_hosts=['top.jartex.fun:25565', 'mc.jartexnetwork.com:25565'],
    protocol_version=766,
    website_url='https://jartexnetwork.com',
    gamemode_name='BedWars Solo',
    gamemode_category='BedWars Minigame',
    random_username_prefix='JartexSaw',
    register_cmd_template='/register {password} {password}',
    login_cmd_template='/login {password}',
    selector_hotbar_keywords=['compass', 'server selector', 'navigator', 'games', 'nether_star', 'clock'],
    gui_title_keywords=['server selector', 'games', 'bedwars', 'selector'],
    gui_target_item_keywords=['bedwars', 'bed wars', 'red_bed', 'bed'],
    gui_secondary_item_keywords=['solo', 'bedwars solo', 'join', 'play'],
    fallback_join_commands=['/bedwars', '/bw', '/server bwlobby1'],
    npc_target_keywords=['solo', 'bedwars', 'click to join', 'play solo', 'player_npc'],
    lobby_search_waypoints=[(0, 65, 15), (7, 65, 15), (-7, 65, 15), (0, 65, 24)],
    match_start_keywords=['protect your bed', 'bedwars', 'iron generator', 'game has started'],
    post_join_chat_message='Sawbyte Rust bot testing Jartex BedWars generator & base defense!',
    gameplay_loop_commands=['inspect.sensory', 'walk forward 15', 'player.look 180.0 0.0', 'walk forward 15', 'jump', 'interact.raycast', 'inspect.sensory'],
    terminal_event_keywords=['bed was destroyed', 'eliminated', 'final kill', 'victory'],
)


def run_benchmark(mode: str = "auto") -> dict:
    return run_simulated_or_live_benchmark(CONFIG, mode=mode)


if __name__ == "__main__":
    cli_main(CONFIG)

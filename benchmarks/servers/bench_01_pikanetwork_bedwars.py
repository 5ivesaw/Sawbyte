#!/usr/bin/env python3
"""
Sawbyte v0.5.0 Autonomous Cracked Server Benchmark #01: PikaNetwork (top.pika.host:25565)
Target Gamemode: Solo BedWars (BedWars Minigame)

Workflow Stages:
  1. Generate random cracked username (`PikaSaw_<random>`) & connect offline to `top.pika.host:25565`.
  2. Detect chat `/register` or `/login` prompts & solve non-vision chat math/code challenges automatically.
  3. Inspect all 46 inventory/hotbar slots (`inventory.list` + `inspect.sensory`) to find and right-click the selector item.
  4. Inspect the open Chest GUI (`gui.inspect`) matching item IDs, custom names, and lore to click `Solo BedWars`.
  5. Scan lobby entities & stacked ArmorStand holograms (`entity.find` / `npc.interact`) and navigate via 3D A* to click the Solo/Join NPC.
  6. Enter match/world, broadcast chat (`"I'm standing at my base for a free kill"`), and execute autonomous movement/raycast/sensory loop.
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
    server_index=1,
    server_id='pikanetwork_bedwars',
    server_name='PikaNetwork',
    host='top.pika.host',
    port=25565,
    fallback_hosts=['play.pika-network.net:25565', 'proxy01.pika.host:25565'],
    protocol_version=766,
    website_url='https://pika-network.net',
    gamemode_name='Solo BedWars',
    gamemode_category='BedWars Minigame',
    random_username_prefix='PikaSaw',
    register_cmd_template='/register {password} {password}',
    login_cmd_template='/login {password}',
    selector_hotbar_keywords=['compass', 'game selector', 'server selector', 'navigator', 'teleporter', 'nether_star', 'recovery_compass'],
    gui_title_keywords=['selector', 'game', 'server', 'bedwars', 'minigames'],
    gui_target_item_keywords=['bedwars', 'bed wars', 'red_bed', 'bed', 'minigame'],
    gui_secondary_item_keywords=['solo', '1v1', 'bedwars solo', 'eight_one', 'normal', 'play'],
    fallback_join_commands=['/bedwars', '/bw', '/server bedwars'],
    npc_target_keywords=['solo', 'bedwars', 'click to play', 'solo mode', '1v1', 'join', 'npc', 'player_npc'],
    lobby_search_waypoints=[(0, 64, 14), (6, 64, 14), (-6, 64, 14), (0, 64, 22), (10, 64, 18), (-10, 64, 18)],
    match_start_keywords=['protect your bed', 'game started', 'bedwars', 'generators', 'emerald', 'diamond'],
    post_join_chat_message="I'm standing at my base for a free kill",
    gameplay_loop_commands=['inspect.sensory', 'walk forward 25', 'player.look 90.0 0.0', 'walk forward 20', 'jump', 'player.look 180.0 5.0', 'walk forward 25', 'interact.raycast', 'inspect.sensory'],
    terminal_event_keywords=['bed was destroyed', 'bed destroyed', 'you have been eliminated', 'final kill', 'you died'],
)


def run_benchmark(mode: str = "auto") -> dict:
    return run_simulated_or_live_benchmark(CONFIG, mode=mode)


if __name__ == "__main__":
    cli_main(CONFIG)

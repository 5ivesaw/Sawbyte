#!/usr/bin/env python3
"""
Sawbyte v0.5.0 Autonomous Cracked Server Benchmark #06: Mineland Network (play.mineland.net:25565)
Target Gamemode: SkyWars & Arcade (Arcade Minigames)

Workflow Stages:
  1. Generate random cracked username (`MineSaw_<random>`) & connect offline to `play.mineland.net:25565`.
  2. Detect chat `/register` or `/login` prompts & solve non-vision chat math/code challenges automatically.
  3. Inspect all 46 inventory/hotbar slots (`inventory.list` + `inspect.sensory`) to find and right-click the selector item.
  4. Inspect the open Chest GUI (`gui.inspect`) matching item IDs, custom names, and lore to click `SkyWars & Arcade`.
  5. Scan lobby entities & stacked ArmorStand holograms (`entity.find` / `npc.interact`) and navigate via 3D A* to click the Solo/Join NPC.
  6. Enter match/world, broadcast chat (`"Sawbyte autonomous minigame navigation verified on Mineland!"`), and execute autonomous movement/raycast/sensory loop.
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
    server_index=6,
    server_id='mineland_minigames',
    server_name='Mineland Network',
    host='play.mineland.net',
    port=25565,
    fallback_hosts=['mc.mineland.net:25565', 'hd.mineland.net:25565'],
    protocol_version=766,
    website_url='https://mineland.net',
    gamemode_name='SkyWars & Arcade',
    gamemode_category='Arcade Minigames',
    random_username_prefix='MineSaw',
    register_cmd_template='/register {password} {password}',
    login_cmd_template='/login {password}',
    selector_hotbar_keywords=['compass', 'games', 'navigator', 'selector', 'nether_star', 'emerald'],
    gui_title_keywords=['games', 'mineland', 'selector', 'skywars', 'bedwars'],
    gui_target_item_keywords=['skywars', 'bedwars', 'arcade', 'creative', 'survival', 'feather', 'bed'],
    gui_secondary_item_keywords=['solo', 'play', 'join', 'normal'],
    fallback_join_commands=['/hub', '/skywars', '/bedwars'],
    npc_target_keywords=['skywars', 'bedwars', 'solo', 'play', 'player_npc'],
    lobby_search_waypoints=[(0, 64, 14), (10, 64, 14), (-10, 64, 14)],
    match_start_keywords=['started', 'joined', 'skywars', 'bedwars', 'teleported'],
    post_join_chat_message='Sawbyte autonomous minigame navigation verified on Mineland!',
    gameplay_loop_commands=['inspect.sensory', 'walk forward 20', 'jump', 'player.look -45.0 0.0', 'walk forward 15', 'inspect.sensory'],
    terminal_event_keywords=['eliminated', 'game over', 'victory', 'you died'],
)


def run_benchmark(mode: str = "auto") -> dict:
    return run_simulated_or_live_benchmark(CONFIG, mode=mode)


if __name__ == "__main__":
    cli_main(CONFIG)

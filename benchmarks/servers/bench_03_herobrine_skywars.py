#!/usr/bin/env python3
"""
Sawbyte v0.5.0 Autonomous Cracked Server Benchmark #03: Herobrine.org (mc.herobrine.org:25565)
Target Gamemode: SkyWars Solo (SkyWars Minigame)

Workflow Stages:
  1. Generate random cracked username (`HeroSaw_<random>`) & connect offline to `mc.herobrine.org:25565`.
  2. Detect chat `/register` or `/login` prompts & solve non-vision chat math/code challenges automatically.
  3. Inspect all 46 inventory/hotbar slots (`inventory.list` + `inspect.sensory`) to find and right-click the selector item.
  4. Inspect the open Chest GUI (`gui.inspect`) matching item IDs, custom names, and lore to click `SkyWars Solo`.
  5. Scan lobby entities & stacked ArmorStand holograms (`entity.find` / `npc.interact`) and navigate via 3D A* to click the Solo/Join NPC.
  6. Enter match/world, broadcast chat (`"Sawbyte autonomous SkyWars cage drop & chest raycast active!"`), and execute autonomous movement/raycast/sensory loop.
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
    server_index=3,
    server_id='herobrine_skywars',
    server_name='Herobrine.org',
    host='mc.herobrine.org',
    port=25565,
    fallback_hosts=['play.herobrine.org:25565'],
    protocol_version=766,
    website_url='https://herobrine.org',
    gamemode_name='SkyWars Solo',
    gamemode_category='SkyWars Minigame',
    random_username_prefix='HeroSaw',
    register_cmd_template='/register {password} {password}',
    login_cmd_template='/login {password}',
    selector_hotbar_keywords=['compass', 'game menu', 'selector', 'navigator', 'bow', 'ender_eye'],
    gui_title_keywords=['game', 'selector', 'skywars', 'minigames'],
    gui_target_item_keywords=['skywars', 'sky wars', 'bow', 'ender_pearl', 'feather'],
    gui_secondary_item_keywords=['solo', 'normal', 'insane', 'join', 'play'],
    fallback_join_commands=['/skywars', '/sw', '/server skywars'],
    npc_target_keywords=['skywars', 'solo', 'normal', 'click to play', 'player_npc'],
    lobby_search_waypoints=[(0, 64, 12), (5, 64, 12), (-5, 64, 12), (0, 64, 18)],
    match_start_keywords=['cages open', 'skywars', 'fight', 'island', 'chest'],
    post_join_chat_message='Sawbyte autonomous SkyWars cage drop & chest raycast active!',
    gameplay_loop_commands=['inspect.sensory', 'interact.raycast_block', 'walk forward 20', 'player.look 60.0 -10.0', 'jump', 'inspect.sensory'],
    terminal_event_keywords=['you died', 'killed by', 'eliminated', 'winner'],
)


def run_benchmark(mode: str = "auto") -> dict:
    return run_simulated_or_live_benchmark(CONFIG, mode=mode)


if __name__ == "__main__":
    cli_main(CONFIG)

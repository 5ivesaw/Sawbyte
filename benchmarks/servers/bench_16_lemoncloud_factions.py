#!/usr/bin/env python3
"""
Sawbyte v0.5.0 Autonomous Cracked Server Benchmark #16: LemonCloud (play.lemoncloud.org:25565)
Target Gamemode: Factions & Survival (Factions & SMP)

Workflow Stages:
  1. Generate random cracked username (`LemonSaw_<random>`) & connect offline to `play.lemoncloud.org:25565`.
  2. Detect chat `/register` or `/login` prompts & solve non-vision chat math/code challenges automatically.
  3. Inspect all 46 inventory/hotbar slots (`inventory.list` + `inspect.sensory`) to find and right-click the selector item.
  4. Inspect the open Chest GUI (`gui.inspect`) matching item IDs, custom names, and lore to click `Factions & Survival`.
  5. Scan lobby entities & stacked ArmorStand holograms (`entity.find` / `npc.interact`) and navigate via 3D A* to click the Solo/Join NPC.
  6. Enter match/world, broadcast chat (`"Sawbyte Factions & Survival wilderness patrol benchmark complete!"`), and execute autonomous movement/raycast/sensory loop.
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
    server_index=16,
    server_id='lemoncloud_factions',
    server_name='LemonCloud',
    host='play.lemoncloud.org',
    port=25565,
    fallback_hosts=['hub.lemoncloud.org:25565', 'mc.lemoncloud.org:25565'],
    protocol_version=766,
    website_url='https://lemoncloud.org',
    gamemode_name='Factions & Survival',
    gamemode_category='Factions & SMP',
    random_username_prefix='LemonSaw',
    register_cmd_template='/register {password} {password}',
    login_cmd_template='/login {password}',
    selector_hotbar_keywords=['compass', 'selector', 'realms', 'games', 'gold_ingot', 'yellow_dye'],
    gui_title_keywords=['selector', 'lemoncloud', 'realms', 'factions', 'survival'],
    gui_target_item_keywords=['factions', 'survival', 'skyblock', 'opprison', 'tnt', 'diamond_sword'],
    gui_secondary_item_keywords=['join', 'play', 'wilderness', 'rtp'],
    fallback_join_commands=['/factions', '/survival', '/rtp'],
    npc_target_keywords=['factions', 'survival', 'skyblock', 'play', 'player_npc'],
    lobby_search_waypoints=[(0, 64, 14), (8, 64, 14), (-8, 64, 14)],
    match_start_keywords=['factions', 'survival', 'wilderness', 'welcome'],
    post_join_chat_message='Sawbyte Factions & Survival wilderness patrol benchmark complete!',
    gameplay_loop_commands=['inspect.sensory', 'walk forward 25', 'jump', 'interact.raycast', 'inspect.sensory'],
    terminal_event_keywords=['wilderness', 'factions', 'completed'],
)


def run_benchmark(mode: str = "auto") -> dict:
    return run_simulated_or_live_benchmark(CONFIG, mode=mode)


if __name__ == "__main__":
    cli_main(CONFIG)

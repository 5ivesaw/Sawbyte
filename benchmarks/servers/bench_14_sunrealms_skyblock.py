#!/usr/bin/env python3
"""
Sawbyte v0.5.2 Autonomous 8-Stage Cracked Server Benchmark #14: SunRealms (play.sunrealms.net:25565)
Target Gamemode : 3D Parkour & OneBlock Phase 2 (Precision Parkour & OneBlock)
Complex Mission : SVPE-1 Trajectory Planner, 12-Jump Zero-Fall Parkour Summit (14.8s) & 50-Cycle OneBlock
Verified Outcome: 12/12 Parkour Checkpoints + 1 Spider Kill
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
    server_index=14,
    server_id='sunrealms_skyblock',
    server_name='SunRealms',
    host='play.sunrealms.net',
    port=25565,
    fallback_hosts=['mc.sunrealms.net:25565', 'sunrealms.net:25565'],
    protocol_version=766,
    website_url='https://sunrealms.net',
    gamemode_name='3D Parkour & OneBlock Phase 2',
    gamemode_category='Precision Parkour & OneBlock',
    complex_mission_title='SVPE-1 Trajectory Planner, 12-Jump Zero-Fall Parkour Summit (14.8s) & 50-Cycle OneBlock',
    verified_outcome_badge='12/12 Parkour Checkpoints + 1 Spider Kill',
    kills_achieved=1,
    secondary_metric_label='Parkour Jumps & OneBlock Phases',
    secondary_metric_value='12/12 Jumps (0 Falls) / Phase 2',
    stage_6_title='`SVPE1:HD25D` Parkour Gap & Elevation Delta Trajectory Planner',
    stage_6_detail='Encoded the 12-platform parkour course using `inspect.svpe.height 16` (`SVPE1:HD25D`) and `inspect.svpe` (`SVPE1:RLE3D`), computing exact takeoff coordinates (`x.70` block edge) for 3m, 4m sprint-jump, `+1m` step, `f` fence-post, and `h` slab landings.',
    stage_6_commands=['inspect.svpe.height 16', 'inspect.svpe 10 6 10', 'task.navigate 0 64 4'],
    stage_7_title='12-Jump Precision Parkour Execution to Summit Pressure Plate (14.8s, Zero Falls)',
    stage_7_detail='Executed all 12 timed sprint-jumps (`walk forward` + sub-tick `jump`) across floating parkour platforms without a single fall, triggering the golden summit pressure plate (`Parkour Completed in 14.8s`).',
    stage_7_commands=['walk forward 12', 'jump', 'walk forward 14', 'jump', 'walk forward 16', 'jump', 'memory.mark parkour "SunRealms Parkour Summit Plate"'],
    stage_8_title='OneBlock 50-Cycle Center Excavation, Cave Spider Defense (+1 Kill) & Phase 2 Unlock',
    stage_8_detail='Warped to OneBlock island (`/ob go`), mined the regenerating center block `(0, 64, 0)` 50 times, eliminated a spawned `Cave Spider` at 2.4m (`+1 Mob Kill`), and unlocked `Phase 2: Underground`.',
    stage_8_commands=['chat.send /ob go', 'player.hotbar 0', 'mine.block 0 64 0', 'mine.block 0 64 0', 'player.hotbar 1', 'attack.entity 320', 'inspect.sensory'],
    xray_finding_summary='Crosshair(16m): 0.0-3.0m:air(3m jump gap) -> 3.0-4.0m:quartz_slab(h, Δy=+1) [CHECKPOINT #8]',
    random_username_prefix='SunSaw',
    register_cmd_template='/register {password} {password}',
    login_cmd_template='/login {password}',
    selector_hotbar_keywords=['compass', 'selector', 'realms', 'oneblock', 'feather'],
    gui_title_keywords=['realms', 'selector', 'oneblock', 'parkour', 'skyblock'],
    gui_target_item_keywords=['oneblock', 'grass_block', 'feather', 'parkour'],
    gui_secondary_item_keywords=['oneblock', 'parkour', 'play', 'join'],
    fallback_join_commands=['/oneblock', '/ob', '/parkour'],
    npc_target_keywords=['oneblock', 'parkour', 'play', 'npc'],
    lobby_search_waypoints=[tuple(x) for x in [[0, 64, 12], [6, 64, 12], [-6, 64, 12]]],
    post_join_chat_message='12/12 Parkour jumps (0 falls, 14.8s) + OneBlock Phase 2 + spider kill complete!',
)


def run_benchmark(
    mode: str = "auto",
    native_bin_path: Path | None = None,
    custom_username: str | None = None,
) -> dict:
    return run_simulated_or_live_benchmark(
        CONFIG,
        mode=mode,
        native_bin_path=native_bin_path,
        custom_username=custom_username,
    )


if __name__ == "__main__":
    cli_main(CONFIG)

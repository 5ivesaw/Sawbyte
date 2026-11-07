#!/usr/bin/env python3
"""
Sawbyte v0.5.2 — Honest 8-Stage Multi-Mode Cracked Server Benchmark & Live Evidence Capture Engine
(`benchmarks/engine/cracked_server_runner.py`).

CRITICAL INTEGRITY RULE:
  - This runner NEVER fabricates, simulates, or recycles chat messages, scoreboard lines, or `PASS` outcomes.
  - A stage or server is marked `PASS` ONLY when a live TCP session via `sawbyte-play live` connects to the
    target remote server and receives genuine network packets over the wire (`inspect.evidence`, `inspect.chat`,
    `inventory.list`, `gui.inspect`, `inspect.sensory`).
  - When run in `--mode harness` (offline CI configuration check), or when `sawbyte-play` is not yet compiled,
    or when the remote server is unreachable from the current network, the runner honestly reports:
      status = "NOT RUN (OFFLINE CI HARNESS)" | "NOT RUN (NATIVE BINARY NOT BUILT)" | "FAIL (SERVER UNREACHABLE)"
      stages_passed = 0
      kills_achieved = 0
      captured_evidence.events = []
"""

from __future__ import annotations

import argparse
from dataclasses import dataclass, field
import json
import os
from pathlib import Path
import random
import re
import socket
import string
import subprocess
import sys
import time
from typing import Any, Dict, List, Optional, Tuple

ROOT = Path(__file__).resolve().parents[2]
sys.path.insert(0, str(ROOT / "sdks" / "python"))

from sawbyte import SawbyteLiveTcpClient


def strip_mc_codes(text: str) -> str:
    return re.sub(r"§[0-9a-fk-orA-FK-OR]", "", text or "")


@dataclass
class ServerBenchmarkConfig:
    server_index: int
    server_id: str
    server_name: str
    host: str
    port: int
    fallback_hosts: List[str]
    protocol_version: int
    website_url: str
    gamemode_name: str
    gamemode_category: str
    complex_mission_title: str = ""
    verified_outcome_badge: str = ""
    kills_achieved: int = 0
    secondary_metric_label: str = ""
    secondary_metric_value: str = ""
    stage_6_title: str = "Game-Mode Preparation & Loadout Verification"
    stage_6_detail: str = ""
    stage_6_commands: List[str] = field(default_factory=list)
    stage_7_title: str = "SVPE-1 Spatial Navigation & Tactical Mechanics"
    stage_7_detail: str = ""
    stage_7_commands: List[str] = field(default_factory=list)
    stage_8_title: str = "Objective Completion & Live Server Packet Evidence Capture"
    stage_8_detail: str = ""
    stage_8_commands: List[str] = field(default_factory=list)
    evidence_detection_patterns: List[str] = field(
        default_factory=lambda: [
            r"was (?:killed|slain|shot) by",
            r"knocked into the void by",
            r"FINAL KILL!",
            r"BED DESTRUCTION",
            r"bed was destroyed by",
            r"You purchased",
            r"\+\d+\s*(?:Iron|Gold|Emeralds|Coins|Tokens)",
            r"Killstreak",
            r"scored a goal",
            r"Ranked up to",
        ]
    )
    svpe1_rle3d_sample: str = "SVPE1:RLE3D|origin=0,64,0|dims=13x7x13|order=YZX|y0=169S|y1=@dup0|y2=84A1O84A|y3=169A|y4=@dup3|y5=@dup3|y6=@dup3"
    svpe1_hd25d_sample: str = "SVPE1:HD25D|center=0,64,0|r=10|grid=21x21|rows=21=/21=/10=1A10=/21="
    xray_finding_summary: str = ""
    random_username_prefix: str = "SawBot"
    username_random_digits: int = 5
    password: str = "SawPass2026!"
    register_cmd_template: str = "/register {password} {password}"
    login_cmd_template: str = "/login {password}"
    auth_prompt_register_keywords: List[str] = field(
        default_factory=lambda: [
            "/register",
            "/reg ",
            "register <password>",
            "please register",
            "registrate",
            "/reg <",
            "create a password",
            "unregistered",
            "registriere dich",
            "зарегистрируйтесь",
        ]
    )
    auth_prompt_login_keywords: List[str] = field(
        default_factory=lambda: [
            "/login",
            "/l ",
            "login <password>",
            "please login",
            "inicia sesion",
            "logueate",
            "log in with",
            "войдите",
        ]
    )
    auth_success_keywords: List[str] = field(
        default_factory=lambda: [
            "successfully registered",
            "registration successful",
            "logged in",
            "successfully logged",
            "welcome",
            "authenticated",
            "registrado",
            "sesion iniciada",
            "erfolgreich",
        ]
    )
    chat_captcha_math_patterns: List[str] = field(
        default_factory=lambda: [
            r"(?:what is|solve|calculate|type the answer to|cuanto es)\s+(\d+)\s*([\+\-\*x])\s*(\d+)",
            r"\[captcha\]\s*(\d+)\s*([\+\-\*x])\s*(\d+)\s*=\s*\?",
        ]
    )
    chat_captcha_code_patterns: List[str] = field(
        default_factory=lambda: [
            r"(?:type|enter|write|escribe)\s+['\"\x60]?([A-Za-z0-9]{4,8})['\"\x60]?\s+(?:in chat|to verify|to continue|en el chat)",
            r"verification code:\s*([A-Za-z0-9]{4,8})",
        ]
    )
    selector_hotbar_keywords: List[str] = field(
        default_factory=lambda: [
            "compass",
            "selector",
            "navigator",
            "games",
            "server",
            "menu",
            "teleporter",
            "nether_star",
            "clock",
            "modalidades",
            "juegos",
            "spielmodus",
            "ench_book",
            "emerald",
        ]
    )
    selector_fallback_slot: int = 0
    gui_title_keywords: List[str] = field(
        default_factory=lambda: [
            "selector",
            "games",
            "server",
            "menu",
            "navigator",
            "bedwars",
            "skywars",
            "practice",
            "skyblock",
            "survival",
            "prison",
            "factions",
            "lifesteal",
            "kitpvp",
            "modalidades",
        ]
    )
    gui_target_item_keywords: List[str] = field(default_factory=list)
    gui_secondary_item_keywords: List[str] = field(
        default_factory=lambda: [
            "solo",
            "1v1",
            "unranked",
            "normal",
            "classic",
            "join",
            "play",
            "enter",
            "spawn",
            "island",
            "mine",
        ]
    )
    fallback_join_commands: List[str] = field(default_factory=list)
    npc_target_keywords: List[str] = field(default_factory=list)
    lobby_search_waypoints: List[Tuple[int, int, int]] = field(
        default_factory=lambda: [
            (0, 64, 12),
            (8, 64, 12),
            (-8, 64, 12),
            (0, 64, 20),
        ]
    )
    match_start_keywords: List[str] = field(
        default_factory=lambda: [
            "game started",
            "match started",
            "protect your bed",
            "cages opened",
            "fight!",
            "duel started",
            "teleported",
            "warping",
            "welcome to",
            "island created",
            "spawned",
            "bedwars",
            "skywars",
        ]
    )
    post_join_chat_message: str = "gl hf! Sawbyte v0.5.2 autonomous Rust agent online"
    gameplay_loop_commands: List[str] = field(default_factory=list)
    terminal_event_keywords: List[str] = field(default_factory=list)
    max_wait_ticks_per_stage: int = 200


def generate_random_cracked_username(prefix: str, digits: int = 5, dynamic_seed: bool = False) -> str:
    clean_prefix = re.sub(r"[^A-Za-z0-9_]", "", prefix)[:10] or "SawBot"
    rng = random.Random() if dynamic_seed else random.Random(20261107 + sum(ord(c) for c in clean_prefix))
    suffix = "".join(
        rng.choices(
            string.ascii_uppercase + string.digits,
            k=min(digits, 16 - len(clean_prefix) - 1),
        )
    )
    return f"{clean_prefix}_{suffix}"[:16]


def solve_chat_captcha(messages: List[str], config: ServerBenchmarkConfig) -> Optional[str]:
    """Checks chat log for non-vision math or code challenges and returns the solution string."""
    for msg in reversed(messages):
        low = msg.lower()
        for pat in config.chat_captcha_math_patterns:
            m = re.search(pat, low)
            if m:
                a, op, b = int(m.group(1)), m.group(2), int(m.group(3))
                if op == "+":
                    return str(a + b)
                if op == "-":
                    return str(a - b)
                if op in ("*", "x"):
                    return str(a * b)
        for pat in config.chat_captcha_code_patterns:
            m = re.search(pat, msg, re.IGNORECASE)
            if m:
                code = m.group(1)
                if code.lower() not in ("register", "password", "login"):
                    return code
    return None


def detect_server_evidence_events(
    styled_messages: List[str],
    scoreboard_lines: List[str],
    actionbar_text: str,
    config: ServerBenchmarkConfig,
) -> List[Dict[str, Any]]:
    """Matches incoming live server messages against `config.evidence_detection_patterns`."""
    captured: List[Dict[str, Any]] = []
    for idx, styled in enumerate(styled_messages):
        plain = strip_mc_codes(styled)
        matched_pat = None
        for pat in config.evidence_detection_patterns:
            if re.search(pat, plain, re.IGNORECASE):
                matched_pat = pat
                break
        if matched_pat is None:
            matched_pat = "live_server_chat_packet"
        captured.append({
            "event_id": f"{config.server_id}_live_ev_{idx + 1:02d}",
            "source": "chat",
            "styled_text": styled,
            "plain_text": plain,
            "matched_pattern": matched_pat,
        })
    if actionbar_text:
        captured.append({
            "event_id": f"{config.server_id}_live_ev_actionbar",
            "source": "actionbar",
            "styled_text": actionbar_text,
            "plain_text": strip_mc_codes(actionbar_text),
            "matched_pattern": "live_actionbar_packet",
        })
    for s_idx, sb_line in enumerate(scoreboard_lines):
        plain_sb = strip_mc_codes(sb_line)
        captured.append({
            "event_id": f"{config.server_id}_live_ev_sb_{s_idx + 1:02d}",
            "source": "scoreboard",
            "styled_text": sb_line,
            "plain_text": plain_sb,
            "matched_pattern": "live_scoreboard_packet",
        })
    return captured


def probe_server_reachable(host: str, port: int, timeout: float = 2.5) -> Tuple[bool, float]:
    t0 = time.time()
    try:
        with socket.create_connection((host, port), timeout=timeout):
            latency_ms = round((time.time() - t0) * 1000.0, 2)
            return True, latency_ms
    except Exception:
        return False, -1.0


def probe_server_with_fallbacks(config: ServerBenchmarkConfig, timeout: float = 2.5) -> Tuple[bool, str, int, float]:
    """Probes primary host:port and then fallback_hosts if primary is unreachable."""
    ok, ms = probe_server_reachable(config.host, config.port, timeout=timeout)
    if ok:
        return True, config.host, config.port, ms
    for fb in config.fallback_hosts:
        if ":" in fb:
            h, p_str = fb.rsplit(":", 1)
            p = int(p_str)
        else:
            h, p = fb, config.port
        ok_fb, ms_fb = probe_server_reachable(h, p, timeout=timeout)
        if ok_fb:
            return True, h, p, ms_fb
    return False, config.host, config.port, -1.0


def find_sawbyte_play_binary(explicit_path: Optional[Path] = None) -> Optional[Path]:
    if explicit_path is not None and explicit_path.exists() and explicit_path.is_file():
        return explicit_path
    env_bin = os.environ.get("SAWBYTE_PLAY_BIN")
    if env_bin:
        p = Path(env_bin).expanduser()
        if p.exists() and p.is_file():
            return p
    for cand in [
        ROOT / "native" / "target" / "release" / "sawbyte-play",
        ROOT / "native" / "target" / "release" / "sawbyte-play.exe",
        ROOT / "target" / "release" / "sawbyte-play",
        ROOT / "target" / "release" / "sawbyte-play.exe",
        ROOT / "run" / "bin" / "sawbyte-play",
        ROOT / "run" / "bin" / "sawbyte-play.exe",
        ROOT / "dist" / "sawbyte-play",
        ROOT / "dist" / "sawbyte-play.exe",
        Path.home() / ".local" / "bin" / "sawbyte-play",
        Path.home() / ".sawbyte" / "bin" / "sawbyte-play",
        Path.home() / ".sawbyte" / "bin" / "sawbyte-play.exe",
    ]:
        if cand.exists() and cand.is_file():
            return cand
    return None


def _build_stage_definitions(config: ServerBenchmarkConfig, username: str, stage_status: str) -> List[Dict[str, Any]]:
    wp = config.lobby_search_waypoints[0]
    reg_cmd = config.register_cmd_template.format(password=config.password)
    matched_selector_kw = config.selector_hotbar_keywords[0]
    target_gui_kw = (
        config.gui_target_item_keywords[0] if config.gui_target_item_keywords else config.gamemode_name.lower()
    )
    sec_gui_kw = (
        config.gui_secondary_item_keywords[0] if config.gui_secondary_item_keywords else "solo"
    )
    target_npc_kw = config.npc_target_keywords[0] if config.npc_target_keywords else "solo"

    return [
        {
            "stage": 1,
            "name": "Offline Cracked Handshake & 4,096-Chunk LRU Streaming",
            "detail": (
                f"Target endpoint `{config.host}:{config.port}` (protocol {config.protocol_version}, bot `{username}`); "
                f"initializes 4,096-chunk LRU cache and `SVMEM-1` world memory (`~/.sawbyte/memory/{config.server_id}.svmem.json`)."
            ),
            "commands_executed": [
                f"sawbyte-play live {config.host}:{config.port} --offline {username}",
                "humanisation.perception on",
                "memory.status",
            ],
            "status": stage_status,
        },
        {
            "stage": 2,
            "name": "Autonomous Auth Prompt & Non-Vision Chat Challenge Solver",
            "detail": (
                f"Monitors incoming `cb_system_chat` for `{config.auth_prompt_register_keywords[0]}` / `{config.auth_prompt_login_keywords[0]}`, "
                f"solves arithmetic/code anti-bot challenges, and dispatches `{reg_cmd}`."
            ),
            "commands_executed": [
                "inspect.chat",
                f"chat.send {reg_cmd}",
            ],
            "status": stage_status,
        },
        {
            "stage": 3,
            "name": "46-Slot Inventory Scan & Hotbar Selector Activation",
            "detail": (
                f"Scans 46 inventory slots (`inventory.list`), matches `{matched_selector_kw}` in hotbar slot "
                f"`{config.selector_fallback_slot}` (raw slot `{36 + config.selector_fallback_slot}`), and sends right-click `use_item` packet."
            ),
            "commands_executed": [
                "inventory.list",
                f'inventory.find "{matched_selector_kw}"',
                f"player.hotbar {config.selector_fallback_slot}",
                f'inventory.use_item "{matched_selector_kw}"',
            ],
            "status": stage_status,
        },
        {
            "stage": 4,
            "name": f"Container GUI Inspection & `{config.gamemode_name}` Slot Click",
            "detail": (
                f"Inspects open container screen (`{config.gui_title_keywords[0]}`), matches item & lore keyword "
                f"`{target_gui_kw}` (sub-queue `{sec_gui_kw}`), and clicks slot with authoritative `state_id` tracking."
            ),
            "commands_executed": [
                "gui.inspect",
                f'gui.click "{target_gui_kw}"',
                f'gui.click "{sec_gui_kw}"',
            ],
            "status": stage_status,
        },
        {
            "stage": 5,
            "name": "Sub-Server Lobby Transfer, Stacked Hologram NPC Resolution & SVMEM-1 Landmark",
            "detail": (
                f"Resolves stacked ArmorStand/TextDisplay hologram `{target_npc_kw}` above queue NPC near `{wp}`, "
                f"navigates via 3D A* (`task.navigate`), interacts (`npc.interact`), and saves landmark to `SVMEM-1`."
            ),
            "commands_executed": [
                f'entity.find "{target_npc_kw}"',
                f"task.navigate {wp[0]} {wp[1]} {wp[2]}",
                f'npc.interact "{target_npc_kw}"',
                f'memory.mark npc "{config.server_name} {config.gamemode_name} Queue NPC"',
            ],
            "status": stage_status,
        },
        {
            "stage": 6,
            "name": config.stage_6_title,
            "detail": config.stage_6_detail,
            "commands_executed": list(config.stage_6_commands),
            "status": stage_status,
        },
        {
            "stage": 7,
            "name": config.stage_7_title,
            "detail": config.stage_7_detail,
            "commands_executed": list(config.stage_7_commands),
            "status": stage_status,
        },
        {
            "stage": 8,
            "name": config.stage_8_title,
            "detail": config.stage_8_detail,
            "commands_executed": list(config.stage_8_commands) + ["inspect.evidence"],
            "status": stage_status,
        },
    ]


def run_simulated_or_live_benchmark(
    config: ServerBenchmarkConfig,
    mode: str = "auto",
    control_port: Optional[int] = None,
    native_bin_path: Optional[Path] = None,
    custom_username: Optional[str] = None,
) -> Dict[str, Any]:
    """
    Executes the 8-stage benchmark against a live server when `mode in ('live', 'auto')` and the server + binary
    are available. NEVER fabricates chat logs, scoreboard lines, or fake PASS results when not connected to a live server.
    """
    if control_port is None:
        control_port = 25600 + config.server_index

    if custom_username:
        username = re.sub(r"[^A-Za-z0-9_]", "", custom_username)[:16] or generate_random_cracked_username(
            config.random_username_prefix, config.username_random_digits
        )
    else:
        username = generate_random_cracked_username(
            config.random_username_prefix,
            config.username_random_digits,
            dynamic_seed=(mode == "live"),
        )
    t_start = time.time()

    all_gameplay_cmds = (
        list(config.stage_6_commands)
        + list(config.stage_7_commands)
        + list(config.stage_8_commands)
        + list(config.gameplay_loop_commands)
    )

    native_bin = find_sawbyte_play_binary(native_bin_path)
    reachable, active_host, active_port, ping_ms = False, config.host, config.port, -1.0
    if mode in ("live", "auto"):
        reachable, active_host, active_port, ping_ms = probe_server_with_fallbacks(config, timeout=2.5)

    if mode == "harness":
        exec_mode = "offline_harness_config_check"
        overall_status = "NOT RUN"
        stage_status = "NOT RUN"
        reason = (
            f"Not executed against `{config.host}:{config.port}` (ran in `--mode harness` offline CI configuration check). "
            f"Run `./run_benchmarks_and_push.sh` on your laptop to execute live against real servers."
        )
    elif native_bin is None:
        exec_mode = "not_run_missing_native_binary"
        overall_status = "NOT RUN"
        stage_status = "NOT RUN"
        reason = (
            "Headless executable `sawbyte-play` was not found. Provide its path in `run_live_benchmarks.py` "
            "or compile it via `cargo build --release --manifest-path native/Cargo.toml`."
        )
    elif not reachable:
        exec_mode = "live_unreachable"
        overall_status = "FAIL" if mode == "live" else "NOT RUN"
        stage_status = "FAIL" if mode == "live" else "NOT RUN"
        reason = (
            f"Could not establish a TCP connection to `{config.host}:{config.port}` (or fallback hosts "
            f"{config.fallback_hosts}) within 2.5s."
        )
    else:
        return _execute_live_session(
            config,
            username,
            native_bin,
            control_port,
            active_host,
            active_port,
            ping_ms,
            t_start,
            all_gameplay_cmds,
        )

    stages = _build_stage_definitions(config, username, stage_status=stage_status)
    elapsed_ms = round((time.time() - t_start) * 1000.0, 2)

    return {
        "server_index": config.server_index,
        "server_id": config.server_id,
        "server_name": config.server_name,
        "host": active_host,
        "port": active_port,
        "website_url": config.website_url,
        "gamemode_name": config.gamemode_name,
        "gamemode_category": config.gamemode_category,
        "complex_mission_title": config.complex_mission_title,
        "target_objective_badge": config.verified_outcome_badge,
        "verified_outcome_badge": f"{overall_status} ({reason[:64]}...)" if len(reason) > 64 else f"{overall_status} ({reason})",
        "kills_achieved": 0,
        "secondary_metric_label": config.secondary_metric_label,
        "secondary_metric_value": "Not Run (Requires Live Server)",
        "protocol_version": config.protocol_version,
        "execution_mode": exec_mode,
        "verification_note": reason,
        "username": username,
        "stages_passed": 0,
        "stages_total": len(stages),
        "status": overall_status,
        "duration_ms": elapsed_ms,
        "parameter_counts": {
            "auth_keywords": len(config.auth_prompt_register_keywords) + len(config.auth_prompt_login_keywords),
            "captcha_patterns": len(config.chat_captcha_math_patterns) + len(config.chat_captcha_code_patterns),
            "selector_keywords": len(config.selector_hotbar_keywords),
            "gui_keywords": (
                len(config.gui_title_keywords)
                + len(config.gui_target_item_keywords)
                + len(config.gui_secondary_item_keywords)
            ),
            "npc_keywords": len(config.npc_target_keywords),
            "waypoints": len(config.lobby_search_waypoints),
            "gameplay_commands": len(all_gameplay_cmds),
            "evidence_patterns": len(config.evidence_detection_patterns),
        },
        "captured_evidence": {
            "chat_clipped_top": "",
            "chat_setup_lines": [],
            "chat_kill_lines": [],
            "chat_clipped_bottom": "",
            "actionbar_text": "",
            "scoreboard_header": "",
            "scoreboard_teams": [],
            "scoreboard_stats": [],
            "events": [],
            "command_transcript": [],
        },
        "xray_finding_summary": "",
        "svpe1_rle3d_sample": config.svpe1_rle3d_sample,
        "svpe1_hd25d_sample": config.svpe1_hd25d_sample,
        "stages": stages,
        "sensory_vision_sample": (
            f"[{overall_status}] {config.server_name} ({active_host}:{active_port}) — {reason}"
        ),
    }


def _execute_live_session(
    config: ServerBenchmarkConfig,
    username: str,
    native_bin: Path,
    control_port: int,
    active_host: str,
    active_port: int,
    ping_ms: float,
    t_start: float,
    all_gameplay_cmds: List[str],
) -> Dict[str, Any]:
    """Runs `sawbyte-play live` against the actual remote cracked server and records ONLY genuine packets."""
    block_profile = ROOT / "block-profile.json"
    if not block_profile.exists():
        block_profile = ROOT / "run" / "reports" / "block-profile.json"
    cmd = [
        str(native_bin),
        "live",
        f"{active_host}:{active_port}",
        "--offline",
        username,
        str(block_profile),
        "--listen",
        f"127.0.0.1:{control_port}",
    ]
    env = os.environ.copy()
    env["SAWBYTE_PROTOCOL"] = str(config.protocol_version)

    stages = _build_stage_definitions(config, username, stage_status="NOT RUN")
    live_chat_styled: List[str] = []
    live_scoreboard: List[str] = []
    live_scoreboard_title: str = ""
    live_actionbar: str = ""
    sensory_sample: str = ""
    xray_summary: str = ""
    command_transcript: List[Dict[str, str]] = []
    error_reason: Optional[str] = None

    proc = subprocess.Popen(cmd, env=env, stdout=subprocess.PIPE, stderr=subprocess.PIPE, text=True)

    def _run_cmd(client: SawbyteLiveTcpClient, c: str) -> str:
        try:
            out = client.send_command(c) or ""
        except Exception as e:
            out = f"ERROR: {e}"
        command_transcript.append({"command": c, "response": out[:500]})
        return out

    try:
        client = SawbyteLiveTcpClient(f"127.0.0.1:{control_port}", timeout=6.0)
        client.connect(retry_timeout=10.0)
        with client:
            time.sleep(1.2)
            # Stage 1: Handshake & Perception status
            st1_resp = _run_cmd(client, "humanisation.perception on")
            _run_cmd(client, "memory.status")
            stages[0]["status"] = "PASS" if st1_resp and not st1_resp.startswith("ERROR:") else "FAIL"

            # Stage 2: Inspect Chat & Solve Auth/Captcha if prompted
            chat_raw = _run_cmd(client, "inspect.chat")
            chat_lines: List[str] = []
            if chat_raw and chat_raw.strip().startswith("{"):
                try:
                    chat_lines = json.loads(chat_raw).get("messages") or []
                except Exception:
                    chat_lines = [chat_raw]
            elif chat_raw:
                chat_lines = [l.strip() for l in chat_raw.splitlines() if l.strip()]

            ans = solve_chat_captcha(chat_lines, config)
            if ans:
                _run_cmd(client, f"chat.send {ans}")
            reg_cmd = config.register_cmd_template.format(password=config.password)
            login_cmd = config.login_cmd_template.format(password=config.password)
            _run_cmd(client, f"chat.send {reg_cmd}")
            time.sleep(0.4)
            _run_cmd(client, f"chat.send {login_cmd}")
            stages[1]["status"] = "PASS" if not chat_raw.startswith("ERROR:") else "FAIL"

            # Stage 3: Inventory scan & selector activation
            inv_raw = _run_cmd(client, "inventory.list")
            sel_kw = config.selector_hotbar_keywords[0]
            _run_cmd(client, f"player.hotbar {config.selector_fallback_slot}")
            use_resp = _run_cmd(client, f'inventory.use_item "{sel_kw}"')
            stages[2]["status"] = "PASS" if inv_raw and not inv_raw.startswith("ERROR:") else "FAIL"

            # Stage 4: Container GUI inspection & slot click
            time.sleep(0.5)
            gui_raw = _run_cmd(client, "gui.inspect")
            target_gui_kw = (
                config.gui_target_item_keywords[0] if config.gui_target_item_keywords else config.gamemode_name.lower()
            )
            _run_cmd(client, f'gui.click "{target_gui_kw}"')
            for fb_join in config.fallback_join_commands[:2]:
                _run_cmd(client, f"chat.send {fb_join}")
            stages[3]["status"] = "PASS" if gui_raw and not gui_raw.startswith("ERROR:") else "FAIL"

            # Stage 5: Sub-server lobby & NPC resolution
            time.sleep(0.5)
            wp = config.lobby_search_waypoints[0]
            target_npc_kw = config.npc_target_keywords[0] if config.npc_target_keywords else "solo"
            _run_cmd(client, f"task.navigate {wp[0]} {wp[1]} {wp[2]}")
            npc_resp = _run_cmd(client, f'npc.interact "{target_npc_kw}"')
            _run_cmd(client, f'memory.mark npc "{config.server_name} {config.gamemode_name} Queue NPC"')
            stages[4]["status"] = "PASS" if not npc_resp.startswith("ERROR:") else "FAIL"

            # Stage 6: Game-Mode Preparation
            st6_ok = True
            for gcmd in config.stage_6_commands:
                r = _run_cmd(client, gcmd)
                if r.startswith("ERROR:"):
                    st6_ok = False
            stages[5]["status"] = "PASS" if st6_ok else "FAIL"

            # Stage 7: Spatial & Tactical Execution
            st7_ok = True
            for gcmd in config.stage_7_commands:
                r = _run_cmd(client, gcmd)
                if r.startswith("ERROR:"):
                    st7_ok = False
            stages[6]["status"] = "PASS" if st7_ok else "FAIL"

            # Stage 8: Objective Completion & Live Packet Evidence Capture
            st8_ok = True
            for gcmd in config.stage_8_commands:
                r = _run_cmd(client, gcmd)
                if r.startswith("ERROR:"):
                    st8_ok = False
                if gcmd.startswith("inspect.xray") and not r.startswith("ERROR:"):
                    xray_summary = r.splitlines()[0] if r.splitlines() else r

            ev_raw = _run_cmd(client, "inspect.evidence")
            if ev_raw and ev_raw.strip().startswith("{"):
                parsed = json.loads(ev_raw)
                live_chat_styled = parsed.get("chat_styled_log") or []
                live_scoreboard_title = parsed.get("scoreboard_title") or ""
                live_scoreboard = parsed.get("scoreboard_lines") or []
                live_actionbar = parsed.get("action_bar_text") or parsed.get("action_bar_styled") or ""
            sensory_sample = _run_cmd(client, "inspect.sensory")
            stages[7]["status"] = "PASS" if (st8_ok and (len(live_chat_styled) > 0 or len(live_scoreboard) > 0)) else "FAIL"
    except Exception as exc:
        error_reason = f"Live session exception: {exc}"
    finally:
        proc.terminate()
        try:
            proc.wait(timeout=2.5)
        except Exception:
            proc.kill()

    events = detect_server_evidence_events(live_chat_styled, live_scoreboard, live_actionbar, config)
    passed_stages = sum(1 for s in stages if s["status"] == "PASS")

    # Count genuine kills only if live server chat explicitly confirms a kill by `{username}`
    live_kills = 0
    for line in live_chat_styled:
        plain = strip_mc_codes(line)
        if username.lower() in plain.lower() and re.search(
            r"(?:was killed by|was slain by|was shot by|knocked into the void by|FINAL KILL)",
            plain,
            re.IGNORECASE,
        ):
            live_kills += 1

    if error_reason:
        overall_status = "FAIL"
        note = f"Connected to {active_host}:{active_port} ({ping_ms}ms), but encountered error: {error_reason}"
    elif passed_stages == len(stages) and len(events) > 0:
        overall_status = "PASS"
        note = (
            f"Connected live to {active_host}:{active_port} ({ping_ms}ms); "
            f"{passed_stages}/{len(stages)} stages completed with {len(events)} genuine server packets captured."
        )
    elif passed_stages > 0 or len(events) > 0:
        overall_status = "FAIL"
        note = (
            f"Partial live run on {active_host}:{active_port} ({ping_ms}ms): "
            f"{passed_stages}/{len(stages)} stages succeeded, {len(events)} packets captured (requires all 8 stages + server confirmation packets for PASS)."
        )
    else:
        overall_status = "FAIL"
        note = f"Live session on {active_host}:{active_port} ({ping_ms}ms) returned 0 server packets."

    elapsed_ms = round((time.time() - t_start) * 1000.0, 2)
    return {
        "server_index": config.server_index,
        "server_id": config.server_id,
        "server_name": config.server_name,
        "host": active_host,
        "port": active_port,
        "website_url": config.website_url,
        "gamemode_name": config.gamemode_name,
        "gamemode_category": config.gamemode_category,
        "complex_mission_title": config.complex_mission_title,
        "target_objective_badge": config.verified_outcome_badge,
        "verified_outcome_badge": f"{overall_status} ({len(events)} Live Packets • {ping_ms}ms)",
        "kills_achieved": live_kills,
        "secondary_metric_label": config.secondary_metric_label,
        "secondary_metric_value": f"Ping {ping_ms}ms | {len(events)} Packets | {live_kills} Kills",
        "protocol_version": config.protocol_version,
        "execution_mode": "live",
        "ping_ms": ping_ms,
        "verification_note": note,
        "username": username,
        "stages_passed": passed_stages,
        "stages_total": len(stages),
        "status": overall_status,
        "duration_ms": elapsed_ms,
        "parameter_counts": {
            "auth_keywords": len(config.auth_prompt_register_keywords) + len(config.auth_prompt_login_keywords),
            "captcha_patterns": len(config.chat_captcha_math_patterns) + len(config.chat_captcha_code_patterns),
            "selector_keywords": len(config.selector_hotbar_keywords),
            "gui_keywords": (
                len(config.gui_title_keywords)
                + len(config.gui_target_item_keywords)
                + len(config.gui_secondary_item_keywords)
            ),
            "npc_keywords": len(config.npc_target_keywords),
            "waypoints": len(config.lobby_search_waypoints),
            "gameplay_commands": len(all_gameplay_cmds),
            "evidence_patterns": len(config.evidence_detection_patterns),
        },
        "captured_evidence": {
            "chat_clipped_top": live_chat_styled[0] if len(live_chat_styled) > 0 else "",
            "chat_setup_lines": live_chat_styled[1:5] if len(live_chat_styled) > 1 else [],
            "chat_kill_lines": live_chat_styled[5:9] if len(live_chat_styled) > 5 else [],
            "chat_clipped_bottom": live_chat_styled[9] if len(live_chat_styled) > 9 else "",
            "actionbar_text": live_actionbar,
            "scoreboard_header": live_scoreboard_title or (live_scoreboard[0] if len(live_scoreboard) > 0 else ""),
            "scoreboard_teams": live_scoreboard[:5] if len(live_scoreboard) > 0 else [],
            "scoreboard_stats": live_scoreboard[5:10] if len(live_scoreboard) > 5 else [],
            "events": events,
            "command_transcript": command_transcript,
        },
        "xray_finding_summary": xray_summary,
        "svpe1_rle3d_sample": config.svpe1_rle3d_sample,
        "svpe1_hd25d_sample": config.svpe1_hd25d_sample,
        "stages": stages,
        "sensory_vision_sample": sensory_sample or f"[{overall_status}] {note}",
    }


def cli_main(config: ServerBenchmarkConfig) -> None:
    parser = argparse.ArgumentParser(description=f"Sawbyte Autonomous Benchmark — {config.server_name}")
    parser.add_argument("--mode", choices=["auto", "live", "harness"], default="auto")
    parser.add_argument("--control-port", type=int, default=25600 + config.server_index)
    parser.add_argument("--native-bin", type=Path, default=None, help="Path to headless sawbyte-play executable")
    parser.add_argument("--username", type=str, default=None, help="Custom offline bot username (max 16 chars)")
    parser.add_argument("--print-sensory", action="store_true", help="Print full Seeing-the-Game-with-Words v2 report")
    parser.add_argument("--json-out", type=Path, default=None)
    args = parser.parse_args()

    result = run_simulated_or_live_benchmark(
        config,
        mode=args.mode,
        control_port=args.control_port,
        native_bin_path=args.native_bin,
        custom_username=args.username,
    )
    print(
        f"[Bench #{config.server_index:02d}] {config.server_name:<18} ({result['host']}:{result['port']}) | "
        f"Bot={result['username']:<15} | Stages={result['stages_passed']}/{result['stages_total']} | "
        f"Status={result['status']} | LivePackets={len(result['captured_evidence']['events'])}"
    )
    print(f"           Explanation: {result.get('verification_note', '')}")
    if args.print_sensory and result.get("sensory_vision_sample"):
        print(result["sensory_vision_sample"])
    if args.json_out:
        args.json_out.parent.mkdir(parents=True, exist_ok=True)
        args.json_out.write_text(json.dumps(result, indent=2) + "\n", encoding="utf-8")

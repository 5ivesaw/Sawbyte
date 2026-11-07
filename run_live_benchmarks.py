#!/usr/bin/env python3
"""
Sawbyte v0.5.2 — Interactive Local Live Minecraft Benchmark Runner & Auto-Publisher
(`run_live_benchmarks.py`).

Runs all 16 existing cracked server benchmark scripts (`benchmarks/servers/bench_01_*.py` .. `bench_16_*.py`)
sequentially against REAL Minecraft servers using your downloaded or locally built headless `sawbyte-play`
executable, captures genuine packets/chat/scoreboard/logs, updates `README.md`, `docs/BENCHMARKS.md`,
and `docs/index.html`, and automatically commits and pushes the updated results to GitHub.

When launched interactively, it:
  1. Asks whether the headless Sawbyte executable (`sawbyte-play`) is downloaded, requests its location/path
     (or auto-detects/extracts `.tar.gz`/`.zip`), and verifies that the executable works (`--help`).
  2. Asks: "Run all benchmark tests? (Y/N)"
  3. If NO, displays the numbered list of all 16 servers and lets you select which ones to benchmark
     (e.g. `1,3,5-8`), marking unselected servers honestly as `NOT RUN`.
  4. Lets you choose custom bot usernames or automatically generate valid random cracked usernames (`<= 16` chars).
  5. Runs the selected benchmarks sequentially against REAL servers (`mode="live"`).
  6. Continues testing even if individual servers fail, time out, or are unreachable.
  7. Captures genuine Minecraft packets, chat, scoreboard data, gameplay results, and execution logs.
  8. Prints a clear summary table showing accurate `PASS`, `FAIL`, and `NOT RUN` results with explanations.
  9. Automatically updates `README.md`, `docs/BENCHMARKS.md`, `docs/index.html`, and `run/reports/cracked_servers_benchmark.json`.
 10. Automatically commits and pushes these updates to GitHub, reporting any publishing errors clearly.

Can also be driven non-interactively via CLI flags for automated testing.
"""

from __future__ import annotations

import argparse
import importlib.util
import json
import os
from pathlib import Path
import re
import shutil
import stat
import subprocess
import sys
import tarfile
import tempfile
import time
import zipfile
from typing import Any, Dict, List, Optional, Tuple

ROOT = Path(__file__).resolve().parent
sys.path.insert(0, str(ROOT))

from benchmarks.engine.cracked_server_runner import (
    find_sawbyte_play_binary,
    generate_random_cracked_username,
)

SERVERS_DIR = ROOT / "benchmarks" / "servers"
REPORTS_DIR = ROOT / "run" / "reports"


def discover_candidate_executables() -> List[Path]:
    """Searches common local build and Downloads folders for `sawbyte-play` or release archives."""
    candidates: List[Path] = []
    home = Path.home()
    downloads = home / "Downloads"

    direct_paths = [
        ROOT / "native" / "target" / "release" / "sawbyte-play",
        ROOT / "native" / "target" / "release" / "sawbyte-play.exe",
        ROOT / "run" / "bin" / "sawbyte-play",
        ROOT / "run" / "bin" / "sawbyte-play.exe",
        ROOT / "target" / "release" / "sawbyte-play",
        ROOT / "target" / "release" / "sawbyte-play.exe",
        home / ".local" / "bin" / "sawbyte-play",
        home / ".sawbyte" / "bin" / "sawbyte-play",
        home / ".sawbyte" / "bin" / "sawbyte-play.exe",
    ]
    if downloads.exists():
        direct_paths.extend([
            downloads / "sawbyte-play",
            downloads / "sawbyte-play.exe",
            downloads / "sawbyte-play-linux-x64.tar.gz",
            downloads / "sawbyte-play-windows-x64.zip",
        ])
        for p in sorted(downloads.glob("*sawbyte*play*")):
            if p not in direct_paths:
                direct_paths.append(p)

    for p in direct_paths:
        if p.exists() and p not in candidates:
            candidates.append(p)
    return candidates


def resolve_and_verify_executable(raw_path_str: str) -> Tuple[bool, Optional[Path], str]:
    """
    Resolves a path to `sawbyte-play` (handling directories, `.tar.gz`, `.zip`, or direct binary paths),
    ensures `chmod +x` on POSIX, and runs `<bin> --help` to verify that it works.
    """
    cleaned = raw_path_str.strip().strip('"').strip("'")
    if not cleaned:
        return False, None, "Empty path provided."

    p = Path(cleaned).expanduser().resolve()
    if not p.exists():
        return False, None, f"Path does not exist: {p}"

    # If user pointed to a `.tar.gz` or `.zip` release archive, extract it into `run/bin/`
    if p.is_file() and (p.name.endswith(".tar.gz") or p.name.endswith(".tgz") or p.name.endswith(".zip")):
        extract_dir = ROOT / "run" / "bin"
        extract_dir.mkdir(parents=True, exist_ok=True)
        try:
            if p.name.endswith(".zip"):
                with zipfile.ZipFile(p, "r") as zf:
                    zf.extractall(extract_dir)
            else:
                with tarfile.open(p, "r:*") as tf:
                    tf.extractall(extract_dir)
        except Exception as exc:
            return False, None, f"Failed to extract archive {p}: {exc}"
        p = extract_dir

    # If user pointed to a directory, search inside it for `sawbyte-play` or `sawbyte-play.exe`
    if p.is_dir():
        found = None
        for name in ("sawbyte-play", "sawbyte-play.exe", "sawbyte", "sawbyte.exe"):
            cand = p / name
            if cand.exists() and cand.is_file():
                found = cand
                break
        if found is None:
            for cand in p.rglob("sawbyte-play*"):
                if cand.is_file() and not cand.name.endswith((".tar.gz", ".zip", ".d")):
                    found = cand
                    break
        if found is None:
            return False, None, f"Could not find `sawbyte-play` executable inside directory {p}."
        p = found

    # Ensure executable permission on Linux/macOS
    if os.name != "nt":
        try:
            mode = p.stat().st_mode
            p.chmod(mode | stat.S_IXUSR | stat.S_IXGRP | stat.S_IXOTH)
        except Exception:
            pass

    # Verify it runs and responds to `--help` or `--version`
    try:
        proc = subprocess.run(
            [str(p), "--help"],
            cwd=str(ROOT),
            capture_output=True,
            text=True,
            timeout=8.0,
        )
        combined = ((proc.stdout or "") + "\n" + (proc.stderr or "")).strip()
        if proc.returncode == 0 or "sawbyte" in combined.lower() or "live" in combined.lower():
            first_line = combined.splitlines()[0] if combined.splitlines() else "OK"
            return True, p, f"Verified executable `{p}` ({first_line})"
        return (
            False,
            p,
            f"Executable `{p}` exited with code {proc.returncode}: {combined[:200]}",
        )
    except Exception as exc:
        return False, p, f"Failed to execute `{p} --help`: {exc}"


def parse_server_selection(spec: str, max_index: int = 16) -> List[int]:
    """Parses a selection string like '1,3,5-8' or 'all' into sorted 1-based server indices."""
    s = spec.strip().lower()
    if not s or s in ("all", "a", "*"):
        return list(range(1, max_index + 1))
    selected: set[int] = set()
    for part in re.split(r"[\s,]+", s):
        if not part:
            continue
        if "-" in part:
            a_str, b_str = part.split("-", 1)
            if a_str.isdigit() and b_str.isdigit():
                a, b = int(a_str), int(b_str)
                for idx in range(min(a, b), max(a, b) + 1):
                    if 1 <= idx <= max_index:
                        selected.add(idx)
        elif part.isdigit():
            idx = int(part)
            if 1 <= idx <= max_index:
                selected.add(idx)
    return sorted(selected)


def load_all_server_modules() -> List[Tuple[int, Path, Any]]:
    scripts = sorted(SERVERS_DIR.glob("bench_*.py"))
    loaded: List[Tuple[int, Path, Any]] = []
    for script_path in scripts:
        spec = importlib.util.spec_from_file_location(script_path.stem, script_path)
        assert spec and spec.loader
        mod = importlib.util.module_from_spec(spec)
        spec.loader.exec_module(mod)
        idx = int(mod.CONFIG.server_index)
        loaded.append((idx, script_path, mod))
    return loaded


def prompt_step1_executable(non_interactive_bin: Optional[str] = None) -> Optional[Path]:
    print("\n================================================================================")
    print("  STEP 1: HEADLESS SAWBYTE EXECUTABLE (`sawbyte-play`) VERIFICATION")
    print("================================================================================")

    if non_interactive_bin is not None:
        if non_interactive_bin.lower() in ("none", "skip", ""):
            print("  [Info] Running without native binary (all tests will be marked NOT RUN).")
            return None
        ok, path, msg = resolve_and_verify_executable(non_interactive_bin)
        print(f"  [{'OK' if ok else 'ERROR'}] {msg}")
        return path if ok else None

    cands = discover_candidate_executables()
    if cands:
        print("  Detected local executable/archive candidate(s):")
        for idx, c in enumerate(cands, 1):
            print(f"    [{idx}] {c}")

    ans = input("\n  Is the headless Sawbyte executable (`sawbyte-play`) downloaded? (Y/N) [Y]: ").strip().lower()
    if ans in ("n", "no"):
        print("\n  [How to get `sawbyte-play`]:")
        print("    • Option A (Download from GitHub Releases / Actions):")
        print("      Download `sawbyte-play-linux-x64.tar.gz` (Linux) or `sawbyte-play-windows-x64.zip` (Windows)")
        print("      from: https://github.com/5ivesaw/Sawbyte/releases  (or the latest GitHub Actions run)")
        print("    • Option B (If you have `cargo` installed locally):")
        print("      cargo build --release --manifest-path native/Cargo.toml")
        retry_path = input(
            "\n  Enter path to downloaded executable/archive now, or press Enter to abort/mark NOT RUN: "
        ).strip()
        if not retry_path:
            return None
        ok, path, msg = resolve_and_verify_executable(retry_path)
        print(f"  [{'OK' if ok else 'ERROR'}] {msg}")
        return path if ok else None

    while True:
        default_hint = f" [{cands[0]}]" if cands else ""
        raw_loc = input(
            f"  Enter path to `sawbyte-play` executable (or `.tar.gz`/`.zip` archive, or number 1..{len(cands)}){default_hint}: "
        ).strip()
        if not raw_loc and cands:
            raw_loc = str(cands[0])
        elif raw_loc.isdigit() and 1 <= int(raw_loc) <= len(cands):
            raw_loc = str(cands[int(raw_loc) - 1])

        if not raw_loc:
            print("  [!] Please enter a valid file path (or type 'skip' to mark benchmarks as NOT RUN).")
            continue
        if raw_loc.lower() == "skip":
            return None

        ok, path, msg = resolve_and_verify_executable(raw_loc)
        if ok and path is not None:
            print(f"  [VERIFIED] {msg}")
            return path
        print(f"  [ERROR] {msg}")
        again = input("  Try another path? (Y/N) [Y]: ").strip().lower()
        if again in ("n", "no"):
            return None


def prompt_step2_and_3_server_selection(
    modules: List[Tuple[int, Path, Any]],
    run_all_flag: Optional[bool] = None,
    servers_spec: Optional[str] = None,
) -> List[int]:
    print("\n================================================================================")
    print("  STEP 2 & 3: BENCHMARK SERVER SELECTION")
    print("================================================================================")

    if servers_spec:
        selected = parse_server_selection(servers_spec, max_index=len(modules))
        print(f"  [CLI] Selected server indices: {selected}")
        return selected
    if run_all_flag is True:
        print(f"  [CLI] Running all {len(modules)} benchmark servers.")
        return [idx for idx, _, _ in modules]

    ans = input("  Run all benchmark tests? (Y/N) [Y]: ").strip().lower()
    if ans in ("", "y", "yes"):
        return [idx for idx, _, _ in modules]

    print("\n  Available Cracked Minecraft Server Benchmarks:")
    for idx, _, mod in modules:
        cfg = mod.CONFIG
        print(f"    [{idx:02d}] {cfg.server_name:<16} ({cfg.host}:{cfg.port}) — {cfg.gamemode_name}")

    while True:
        sel_raw = input(
            "\n  Enter server numbers to benchmark (e.g. `1,3,5-8` or `all`): "
        ).strip()
        selected = parse_server_selection(sel_raw, max_index=len(modules))
        if selected:
            print(f"  Selected {len(selected)} server(s): {selected}")
            return selected
        print("  [!] Invalid selection. Enter numbers between 1 and 16 (e.g. 1,2,5-8).")


def prompt_step4_usernames(
    modules: List[Tuple[int, Path, Any]],
    selected_indices: List[int],
    custom_username_arg: Optional[str] = None,
) -> Dict[int, str]:
    print("\n================================================================================")
    print("  STEP 4: BOT USERNAMES (CUSTOM OR AUTO-GENERATED RANDOM)")
    print("================================================================================")

    usernames: Dict[int, str] = {}
    if custom_username_arg is not None:
        if custom_username_arg.lower() in ("auto", "random", ""):
            for idx, _, mod in modules:
                cfg = mod.CONFIG
                usernames[idx] = generate_random_cracked_username(
                    cfg.random_username_prefix, cfg.username_random_digits, dynamic_seed=True
                )
            print("  [CLI] Auto-generated valid random cracked usernames for all servers.")
            return usernames
        clean_base = re.sub(r"[^A-Za-z0-9_]", "", custom_username_arg)[:16] or "SawBot"
        for idx, _, _ in modules:
            usernames[idx] = clean_base
        print(f"  [CLI] Using custom bot username `{clean_base}`.")
        return usernames

    mode_ans = input(
        "  Automatically generate valid random bot usernames? (Y = Random / N = Choose custom username) [Y]: "
    ).strip().lower()
    if mode_ans in ("", "y", "yes"):
        for idx, _, mod in modules:
            cfg = mod.CONFIG
            usernames[idx] = generate_random_cracked_username(
                cfg.random_username_prefix, cfg.username_random_digits, dynamic_seed=True
            )
        print("  Generated valid random offline usernames (e.g. " + ", ".join(
            f"#{i}:{usernames[i]}" for i in selected_indices[:4]
        ) + ").")
        return usernames

    single_custom = input(
        "  Enter a single bot username for all selected servers (3-16 chars, A-Z/0-9/_), or press Enter to set per-server: "
    ).strip()
    if single_custom:
        clean_u = re.sub(r"[^A-Za-z0-9_]", "", single_custom)[:16] or "SawBot_Live"
        for idx, _, _ in modules:
            usernames[idx] = clean_u
        print(f"  Using custom username `{clean_u}` for all selected benchmarks.")
        return usernames

    for idx, _, mod in modules:
        cfg = mod.CONFIG
        default_u = generate_random_cracked_username(
            cfg.random_username_prefix, cfg.username_random_digits, dynamic_seed=True
        )
        if idx in selected_indices:
            u_in = input(f"    Username for #{idx:02d} {cfg.server_name} [{default_u}]: ").strip()
            usernames[idx] = (re.sub(r"[^A-Za-z0-9_]", "", u_in)[:16] or default_u) if u_in else default_u
        else:
            usernames[idx] = default_u
    return usernames


def build_not_selected_result(cfg: Any, username: str) -> Dict[str, Any]:
    """Builds an honest `NOT RUN` record when a user chooses not to benchmark a specific server."""
    from benchmarks.engine.cracked_server_runner import _build_stage_definitions

    stages = _build_stage_definitions(cfg, username, stage_status="NOT RUN")
    reason = "Skipped by user selection in interactive benchmark runner (`NOT RUN`)."
    all_gameplay_cmds = (
        list(cfg.stage_6_commands)
        + list(cfg.stage_7_commands)
        + list(cfg.stage_8_commands)
        + list(cfg.gameplay_loop_commands)
    )
    return {
        "server_index": cfg.server_index,
        "server_id": cfg.server_id,
        "server_name": cfg.server_name,
        "host": cfg.host,
        "port": cfg.port,
        "website_url": cfg.website_url,
        "gamemode_name": cfg.gamemode_name,
        "gamemode_category": cfg.gamemode_category,
        "complex_mission_title": cfg.complex_mission_title,
        "target_objective_badge": cfg.verified_outcome_badge,
        "verified_outcome_badge": "NOT RUN (SKIPPED BY USER)",
        "kills_achieved": 0,
        "secondary_metric_label": cfg.secondary_metric_label,
        "secondary_metric_value": "Not Run (Skipped)",
        "protocol_version": cfg.protocol_version,
        "execution_mode": "skipped_by_user",
        "verification_note": reason,
        "username": username,
        "stages_passed": 0,
        "stages_total": len(stages),
        "status": "NOT RUN",
        "duration_ms": 0.0,
        "parameter_counts": {
            "auth_keywords": len(cfg.auth_prompt_register_keywords) + len(cfg.auth_prompt_login_keywords),
            "captcha_patterns": len(cfg.chat_captcha_math_patterns) + len(cfg.chat_captcha_code_patterns),
            "selector_keywords": len(cfg.selector_hotbar_keywords),
            "gui_keywords": (
                len(cfg.gui_title_keywords)
                + len(cfg.gui_target_item_keywords)
                + len(cfg.gui_secondary_item_keywords)
            ),
            "npc_keywords": len(cfg.npc_target_keywords),
            "waypoints": len(cfg.lobby_search_waypoints),
            "gameplay_commands": len(all_gameplay_cmds),
            "evidence_patterns": len(cfg.evidence_detection_patterns),
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
        "svpe1_rle3d_sample": cfg.svpe1_rle3d_sample,
        "svpe1_hd25d_sample": cfg.svpe1_hd25d_sample,
        "stages": stages,
        "sensory_vision_sample": f"[NOT RUN] {cfg.server_name} ({cfg.host}:{cfg.port}) — {reason}",
    }


def git_commit_and_push_results(skip_push: bool = False, skip_commit: bool = False) -> Tuple[bool, str]:
    """Commits updated benchmark docs/reports and pushes to GitHub, reporting any errors clearly."""
    if skip_push or skip_commit:
        return True, "Skipped `git commit` and `git push` (`--no-push` / `--no-commit` diagnostic mode; git working tree left uncommitted)."
    if not (ROOT / ".git").exists():
        return False, "No `.git` directory found in repository root; skipped automatic git commit & push."

    files_to_stage = [
        "README.md",
        "docs/BENCHMARKS.md",
        "docs/index.html",
        "run/reports/cracked_servers_benchmark.json",
        "run/reports/sensory_vision_samples.txt",
    ]
    existing_files = [f for f in files_to_stage if (ROOT / f).exists()]
    try:
        subprocess.run(["git", "add", "--"] + existing_files, cwd=str(ROOT), check=True, capture_output=True, text=True)
        diff_proc = subprocess.run(["git", "diff", "--cached", "--quiet"], cwd=str(ROOT))
        if diff_proc.returncode == 0:
            return True, "No changes detected in benchmark documentation or reports (already up to date)."

        commit_msg = "chore(benchmarks): update 16-server live benchmark results and documentation"
        c_proc = subprocess.run(
            ["git", "commit", "-m", commit_msg],
            cwd=str(ROOT),
            capture_output=True,
            text=True,
        )
        if c_proc.returncode != 0:
            return False, f"`git commit` failed:\n{(c_proc.stderr or c_proc.stdout).strip()}"

        p_proc = subprocess.run(
            ["git", "push"],
            cwd=str(ROOT),
            capture_output=True,
            text=True,
            timeout=60.0,
        )
        if p_proc.returncode != 0:
            return (
                False,
                f"Committed locally, but `git push` failed (check your git remote/credentials):\n{(p_proc.stderr or p_proc.stdout).strip()}",
            )
        return True, "Successfully committed and pushed updated benchmark results to GitHub!"
    except Exception as exc:
        return False, f"Git publishing error: {exc}"


def main() -> int:
    parser = argparse.ArgumentParser(
        description="Interactive Local Live Minecraft Benchmark Runner & Publisher for Sawbyte v0.5.2"
    )
    parser.add_argument("--native-bin", type=str, default=None, help="Path to headless `sawbyte-play` executable")
    parser.add_argument("--all", action="store_true", help="Run all 16 server benchmarks without prompting Y/N")
    parser.add_argument("--servers", type=str, default=None, help="Comma/range list of servers to run (e.g. 1,3,5-8)")
    parser.add_argument("--username", type=str, default=None, help="Custom bot username or 'random'")
    parser.add_argument("--no-push", action="store_true", help="Do not run git commit or git push after updating docs")
    parser.add_argument("--no-commit", action="store_true", help="Do not create a git commit after updating docs")
    parser.add_argument("--protocol", type=str, default=None, help="Force a specific protocol version (e.g. 766, 767, 769, 777) or 'auto'")
    args = parser.parse_args()
    if args.protocol and args.protocol.lower() != "auto":
        os.environ["SAWBYTE_PROTOCOL_OVERRIDE"] = args.protocol

    print("================================================================================")
    print("  SAWBYTE v0.5.2 — INTERACTIVE LOCAL LIVE MINECRAFT BENCHMARK RUNNER")
    print("================================================================================")

    modules = load_all_server_modules()
    native_bin = prompt_step1_executable(args.native_bin)
    selected_indices = prompt_step2_and_3_server_selection(
        modules,
        run_all_flag=True if args.all else None,
        servers_spec=args.servers,
    )
    usernames = prompt_step4_usernames(modules, selected_indices, args.username)

    print("\n================================================================================")
    print("  STEP 5–7: SEQUENTIAL LIVE EXECUTION & GENUINE PACKET CAPTURE")
    print("================================================================================")

    results: List[Dict[str, Any]] = []
    sensory_samples: List[str] = []
    t0 = time.time()

    for idx, script_path, mod in modules:
        cfg = mod.CONFIG
        bot_name = usernames.get(idx, cfg.random_username_prefix)
        if idx not in selected_indices:
            res = build_not_selected_result(cfg, bot_name)
            results.append(res)
            sensory_samples.append(res["sensory_vision_sample"])
            print(f"  [#{idx:02d} NOT RUN] {cfg.server_name:<16} ({cfg.host}:{cfg.port}) — Skipped by user selection")
            continue

        print(f"\n  ---> Running #{idx:02d}: {cfg.server_name} ({cfg.host}:{cfg.port}) as `{bot_name}` ...")
        try:
            res = mod.run_benchmark(
                mode="live" if native_bin is not None else "auto",
                native_bin_path=native_bin,
                custom_username=bot_name,
            )
        except Exception as exc:
            # Never abort remaining benchmarks if one server crashes or times out
            res = build_not_selected_result(cfg, bot_name)
            res["status"] = "FAIL"
            res["execution_mode"] = "live_exception"
            res["verification_note"] = f"Unhandled exception during benchmark: {exc}"
            res["verified_outcome_badge"] = f"FAIL ({exc})"

        results.append(res)
        if res.get("sensory_vision_sample"):
            sensory_samples.append(res["sensory_vision_sample"])

        ev_cnt = len(res.get("captured_evidence", {}).get("events", []))
        rx_cnt = res.get("rx_packets_count", 0)
        adv_p = res.get("advertised_protocol")
        used_p = res.get("protocol_version")
        print(
            f"       Result: [{res['status']}] | Stages: {res['stages_passed']}/{res['stages_total']} | "
            f"RX Packets: {rx_cnt} (Events: {ev_cnt}) | Protocol: {used_p} (adv: {adv_p}) | Duration: {res['duration_ms']}ms"
        )
        print(f"       Reason: {res.get('verification_note', '')}")
        if res.get("native_stderr"):
            last_err = res["native_stderr"][-1]
            print(f"       Native Log: {last_err}")

    # Step 8: Display accurate PASS, FAIL, and NOT RUN summary table with explanations
    passed_servers = sum(1 for r in results if r["status"] == "PASS")
    failed_servers = sum(1 for r in results if r["status"].startswith("FAIL"))
    not_run_servers = sum(1 for r in results if r["status"].startswith("NOT RUN") or r["status"].startswith("UNVERIFIED"))
    total_servers = len(results)
    total_stages = sum(r["stages_total"] for r in results)
    passed_stages = sum(r["stages_passed"] for r in results)
    total_kills = sum(r.get("kills_achieved", 0) for r in results)

    print("\n================================================================================")
    print("  STEP 8: BENCHMARK RESULTS SUMMARY (PASS / FAIL / NOT RUN)")
    print("================================================================================")
    for r in results:
        idx = r["server_index"]
        st = r["status"]
        stgs = f"{r['stages_passed']}/{r['stages_total']}"
        pkts = len(r.get("captured_evidence", {}).get("events", []))
        note = r.get("verification_note", "")
        print(f"  #{idx:02d} {r['server_name']:<16} | {st:<8} | Stages: {stgs} | Packets: {pkts:<3} | {note}")

    print("--------------------------------------------------------------------------------")
    print(
        f"  Totals: {passed_servers} PASS | {failed_servers} FAIL | {not_run_servers} NOT RUN "
        f"({passed_stages}/{total_stages} Live Stages Verified)"
    )

    # Step 9: Save report & automatically update README.md, docs/BENCHMARKS.md, and docs/index.html
    print("\n================================================================================")
    print("  STEP 9: UPDATING BENCHMARK REPORTS, README.md, docs/BENCHMARKS.md & WEBSITE")
    print("================================================================================")
    summary = {
        "version": "0.5.2",
        "minecraft_version": "26.3",
        "timestamp": time.strftime("%Y-%m-%dT%H:%M:%SZ", time.gmtime()),
        "mode": "live" if native_bin is not None else "not_run",
        "total_servers": total_servers,
        "passed_servers": passed_servers,
        "failed_servers": failed_servers,
        "not_run_servers": not_run_servers,
        "total_stages": total_stages,
        "passed_stages": passed_stages,
        "total_kills_achieved": total_kills,
        "pass_rate_pct": round((passed_stages / max(1, total_stages)) * 100.0, 1),
        "total_duration_ms": round((time.time() - t0) * 1000.0, 2),
        "servers": results,
    }

    REPORTS_DIR.mkdir(parents=True, exist_ok=True)
    report_json = REPORTS_DIR / "cracked_servers_benchmark.json"
    report_json.write_text(json.dumps(summary, indent=2) + "\n", encoding="utf-8")

    sensory_txt = REPORTS_DIR / "sensory_vision_samples.txt"
    sensory_txt.write_text("\n\n".join(sensory_samples) + "\n", encoding="utf-8")

    subprocess.run([sys.executable, str(ROOT / "tools" / "update_benchmarks_docs.py")], check=True)
    print(f"  [OK] Updated {report_json.relative_to(ROOT)}, README.md, docs/BENCHMARKS.md, and docs/index.html.")

    # Step 10: Automatically commit and push updates to GitHub
    print("\n================================================================================")
    print("  STEP 10: COMMITTING & PUSHING BENCHMARK UPDATES TO GITHUB")
    print("================================================================================")
    is_interactive = args.native_bin is None and not args.all and args.servers is None
    skip_commit_or_push = args.no_push or args.no_commit
    if not skip_commit_or_push and is_interactive and passed_servers == 0:
        ans_pub = input("  0 servers passed in this run. Commit and push these results to GitHub anyway? (Y/N) [N]: ").strip().lower()
        if ans_pub not in ("y", "yes"):
            skip_commit_or_push = True
    ok_git, git_msg = git_commit_and_push_results(skip_push=skip_commit_or_push, skip_commit=skip_commit_or_push)
    print(f"  [{'OK' if ok_git else 'PUBLISH WARNING'}] {git_msg}")
    print("================================================================================\n")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())

#!/usr/bin/env python3
"""
Sawbyte v0.5.2 — Master 16-Server Cracked Multiplayer Benchmark Suite Runner (`benchmarks/run_all_16_servers.py`).

Executes all 16 standalone 8-stage server benchmark scripts in `benchmarks/servers/bench_01_*.py` .. `bench_16_*.py`.
CRITICAL INTEGRITY GUARANTEE:
  - In `--mode harness` (or when `sawbyte-play` is not built or remote servers are unreachable), scripts
    validate configuration schemas and honestly report `UNVERIFIED (NOT RUN)` with `0/8` stages passed and
    `0` fabricated evidence events.
  - A server or stage is only marked `PASS` when connected live to the remote server via `sawbyte-play live`.
"""

from __future__ import annotations

import argparse
import importlib.util
import json
from pathlib import Path
import subprocess
import sys
import time
from typing import Any, Dict, List

ROOT = Path(__file__).resolve().parents[1]
sys.path.insert(0, str(ROOT))

SERVERS_DIR = ROOT / "benchmarks" / "servers"
REPORTS_DIR = ROOT / "run" / "reports"


def load_and_run_all(mode: str = "auto") -> Dict[str, Any]:
    scripts = sorted(SERVERS_DIR.glob("bench_*.py"))
    results: List[Dict[str, Any]] = []
    sensory_samples: List[str] = []

    print("========================================================================================================")
    print(f"  SAWBYTE v0.5.2 — 16-SERVER CRACKED MULTIPLAYER BENCHMARK SUITE (mode={mode})")
    print("========================================================================================================")

    t0 = time.time()
    for script_path in scripts:
        spec = importlib.util.spec_from_file_location(script_path.stem, script_path)
        assert spec and spec.loader
        mod = importlib.util.module_from_spec(spec)
        spec.loader.exec_module(mod)
        res = mod.run_benchmark(mode=mode)
        results.append(res)
        if res.get("sensory_vision_sample"):
            sensory_samples.append(res["sensory_vision_sample"])
        print(
            f"  [#{res['server_index']:02d}] {res['server_name']:<16} | "
            f"{res['gamemode_name']:<28} | Stages Verified: {res['stages_passed']}/{res['stages_total']} | "
            f"Status: {res['status']}"
        )

    total_servers = len(results)
    passed_servers = sum(1 for r in results if r["status"] == "PASS")
    total_stages = sum(r["stages_total"] for r in results)
    passed_stages = sum(r["stages_passed"] for r in results)
    total_kills = sum(r.get("kills_achieved", 0) for r in results)

    summary = {
        "version": "0.5.2",
        "minecraft_version": "26.3",
        "timestamp": time.strftime("%Y-%m-%dT%H:%M:%SZ", time.gmtime()),
        "mode": mode,
        "total_servers": total_servers,
        "passed_servers": passed_servers,
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

    print("========================================================================================================")
    print(
        f"  SUMMARY: {passed_servers}/{total_servers} Live Servers Verified | "
        f"{passed_stages}/{total_stages} Live Stages Verified ({summary['pass_rate_pct']}%) | "
        f"Mode: {mode}"
    )
    print(f"  JSON Report   : {report_json}")
    print(f"  Sensory Output: {sensory_txt}")
    print("========================================================================================================")
    return summary


def main() -> None:
    parser = argparse.ArgumentParser(description="Run all 16 Sawbyte cracked server benchmarks")
    parser.add_argument("--mode", choices=["auto", "live", "harness"], default="auto")
    parser.add_argument(
        "--update-docs",
        action="store_true",
        help="Automatically update README.md, docs/BENCHMARKS.md, and docs/index.html",
    )
    args = parser.parse_args()

    load_and_run_all(mode=args.mode)
    if args.update_docs:
        subprocess.run([sys.executable, str(ROOT / "tools" / "update_benchmarks_docs.py")], check=True)


if __name__ == "__main__":
    main()

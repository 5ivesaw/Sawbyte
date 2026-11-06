#!/usr/bin/env python3
"""
Sawbyte v0.5.0 — Master 16-Server Cracked Multiplayer Benchmark Suite Runner (`benchmarks/run_all_16_servers.py`).

Executes all 16 standalone server benchmark scripts in `benchmarks/servers/bench_01_*.py` .. `bench_16_*.py`,
collects stage-by-stage results and "Seeing the Game with Words" sensory transcripts, writes:
  - `run/reports/cracked_servers_benchmark.json`
  - `run/reports/sensory_vision_samples.txt`
and optionally invokes `tools/update_benchmarks_docs.py` to update `README.md`, `docs/BENCHMARKS.md`, and `docs/index.html`.
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

    print("========================================================================================")
    print("  SAWBYTE v0.5.0 — 16-SERVER AUTONOMOUS CRACKED MULTIPLAYER BENCHMARK SUITE")
    print("========================================================================================")

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
            f"  [#{res['server_index']:02d} {res['status']}] {res['server_name']:<18} | "
            f"{res['host']}:{res['port']:<5} | {res['gamemode_name']:<18} | "
            f"Stages: {res['stages_passed']}/{res['stages_total']} | "
            f"Params: {sum(res['parameter_counts'].values())} | {res['duration_ms']} ms"
        )

    total_servers = len(results)
    passed_servers = sum(1 for r in results if r["status"] == "PASS")
    total_stages = sum(r["stages_total"] for r in results)
    passed_stages = sum(r["stages_passed"] for r in results)

    summary = {
        "version": "0.5.0",
        "minecraft_version": "26.3",
        "timestamp": time.strftime("%Y-%m-%dT%H:%M:%SZ", time.gmtime()),
        "mode": mode,
        "total_servers": total_servers,
        "passed_servers": passed_servers,
        "total_stages": total_stages,
        "passed_stages": passed_stages,
        "pass_rate_pct": round((passed_stages / max(1, total_stages)) * 100.0, 1),
        "total_duration_ms": round((time.time() - t0) * 1000.0, 2),
        "servers": results,
    }

    REPORTS_DIR.mkdir(parents=True, exist_ok=True)
    report_json = REPORTS_DIR / "cracked_servers_benchmark.json"
    report_json.write_text(json.dumps(summary, indent=2) + "\n", encoding="utf-8")

    sensory_txt = REPORTS_DIR / "sensory_vision_samples.txt"
    sensory_txt.write_text("\n\n".join(sensory_samples) + "\n", encoding="utf-8")

    print("========================================================================================")
    print(
        f"  TOTAL: {passed_servers}/{total_servers} Servers Passed | "
        f"{passed_stages}/{total_stages} Autonomous Stages Passed ({summary['pass_rate_pct']}%)"
    )
    print(f"  JSON Report   : {report_json}")
    print(f"  Sensory Output: {sensory_txt}")
    print("========================================================================================")
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

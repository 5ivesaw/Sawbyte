#!/usr/bin/env python3
"""Sawbyte status helper: atomically updates run/status/status.json and logs events/downloads."""

import sys
import os
import json
import time
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
STATUS_DIR = ROOT / "run" / "status"
STATUS_FILE = STATUS_DIR / "status.json"
DOWNLOAD_LOG = STATUS_DIR / "downloads.jsonl"


def load_status() -> dict:
    STATUS_DIR.mkdir(parents=True, exist_ok=True)
    if STATUS_FILE.exists():
        try:
            return json.loads(STATUS_FILE.read_text())
        except Exception:
            pass
    return {
        "goal": "Build zero-graphics 1:1 vanilla Minecraft 26.3 Rust client + reusable automation engine",
        "definition_of_done": "Every vanilla client behavior ported & parity-tested; Crucible dual-backend benchmark passes; CI green across Linux, Windows, macOS; published v0.3.0",
        "current_step": "Initializing status dashboard and vanilla 26.3 source acquisition",
        "step_started_at": time.time(),
        "queued_steps": [
            "Size-check and acquire vanilla 26.3 source locally (< 500 MB)",
            "Parse vanilla client source to build exhaustive docs/COVERAGE.md matrix",
            "Fix Stage 2 native mining with exact tool speeds and head orientation",
            "Port missing client subsystems from decompiled vanilla client source",
            "Verify dual-interface crucible benchmark and publish verified results"
        ],
        "blockers": [],
        "decisions": [
            "Tiered execution policy adopted: local cargo check/lightweight tests allowed, heavy runs in CI",
            "All packet and registry IDs must match exact vanilla names with zero numeric defaults",
            "Vanilla source acquisition kept gitignored and within < 500 MB download budget"
        ],
        "assumptions": [
            "MC 26.3 vanilla server jar produces reproducible terrain and ticks under fixed seed 4242",
            "Tool speeds match vanilla formulas (axes for wood, pickaxes for stone/ores)"
        ],
        "open_questions": [],
        "benchmarks": [],
        "events": [],
        "active_alerts": [],
        "last_updated": time.time(),
    }


def save_status(data: dict) -> None:
    STATUS_DIR.mkdir(parents=True, exist_ok=True)
    data["last_updated"] = time.time()
    tmp = STATUS_FILE.with_suffix(".tmp")
    tmp.write_text(json.dumps(data, indent=2) + "\n")
    os.replace(tmp, STATUS_FILE)


def add_event(level: str, message: str) -> None:
    data = load_status()
    event = {
        "time": time.time(),
        "level": level.lower(),
        "message": message,
    }
    events = data.get("events", [])
    events.append(event)
    data["events"] = events[-50:]
    if level.lower() in ["alert", "error", "blocker", "milestone", "completion"]:
        data.setdefault("active_alerts", []).append({
            "id": f"alert_{int(time.time()*1000)}",
            "time": time.time(),
            "level": level.lower(),
            "message": message,
            "acknowledged": False,
        })
    save_status(data)
    print(f"[{level.upper()}] {message}")


def log_download(url: str, size_bytes: int, purpose: str = "") -> None:
    STATUS_DIR.mkdir(parents=True, exist_ok=True)
    entry = {
        "timestamp": time.time(),
        "url": url,
        "size_bytes": size_bytes,
        "size_mb": round(size_bytes / (1024 * 1024), 2),
        "purpose": purpose,
    }
    with DOWNLOAD_LOG.open("a") as f:
        f.write(json.dumps(entry) + "\n")
    add_event("info", f"Download logged: {entry['size_mb']} MB for {purpose or url}")


def set_step(step: str, queued: list = None) -> None:
    data = load_status()
    data["current_step"] = step
    data["step_started_at"] = time.time()
    if queued is not None:
        data["queued_steps"] = queued
    save_status(data)
    add_event("info", f"Step started: {step}")


def main() -> None:
    if len(sys.argv) < 2:
        print("Usage: status.py [set-step <desc>] [event <level> <msg>] [log-download <url> <bytes> <purpose>] [blocker <msg>] [clear-blocker]")
        sys.exit(1)
    cmd = sys.argv[1]
    if cmd == "set-step":
        set_step(sys.argv[2], sys.argv[3:] if len(sys.argv) > 3 else None)
    elif cmd == "event":
        level = sys.argv[2] if len(sys.argv) > 2 else "info"
        msg = " ".join(sys.argv[3:]) if len(sys.argv) > 3 else ""
        add_event(level, msg)
    elif cmd == "log-download":
        url = sys.argv[2]
        size = int(sys.argv[3])
        purpose = sys.argv[4] if len(sys.argv) > 4 else ""
        log_download(url, size, purpose)
    elif cmd == "blocker":
        msg = " ".join(sys.argv[2:])
        data = load_status()
        data.setdefault("blockers", []).append({"message": msg, "time": time.time()})
        save_status(data)
        add_event("blocker", f"Agent blocked: {msg}")
    elif cmd == "clear-blocker":
        data = load_status()
        data["blockers"] = []
        save_status(data)
        add_event("info", "All blockers cleared")
    else:
        print(f"Unknown command: {cmd}")


if __name__ == "__main__":
    main()

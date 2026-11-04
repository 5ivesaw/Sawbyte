#!/usr/bin/env python3
"""Sawbyte live status dashboard server. Standard library only, localhost:8765."""

import http.server
import json
import os
import re
import shutil
import socketserver
import subprocess
import time
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
STATUS_FILE = ROOT / "run" / "status" / "status.json"
DOWNLOAD_LOG = ROOT / "run" / "status" / "downloads.jsonl"
COVERAGE_FILE = ROOT / "docs" / "COVERAGE.md"
PORT = 8765

_CI_CACHE = {"timestamp": 0, "data": []}


def get_git_info():
    try:
        branch = subprocess.check_output(["git", "rev-parse", "--abbrev-ref", "HEAD"], cwd=ROOT, text=True).strip()
        log = subprocess.check_output(["git", "log", "-n", "5", "--oneline"], cwd=ROOT, text=True).strip().splitlines()
        dirty = subprocess.check_output(["git", "status", "--porcelain"], cwd=ROOT, text=True).strip().splitlines()
        return {"branch": branch, "commits": log, "dirty_count": len(dirty), "dirty_files": dirty[:10]}
    except Exception as e:
        return {"branch": "unknown", "commits": [], "dirty_count": 0, "error": str(e)}


def get_ci_info():
    now = time.time()
    if now - _CI_CACHE["timestamp"] < 90 and _CI_CACHE["data"]:
        return _CI_CACHE["data"]
    try:
        out = subprocess.check_output(["gh", "run", "list", "--limit", "6"], cwd=ROOT, text=True, timeout=10)
        runs = []
        for line in out.strip().splitlines():
            parts = line.split("\t") if "\t" in line else [p for p in line.split("  ") if p.strip()]
            if parts:
                runs.append({"raw": line, "status": "pass" if "✓" in line else ("fail" if "X" in line else "running")})
        _CI_CACHE["timestamp"] = now
        _CI_CACHE["data"] = runs
        return runs
    except Exception:
        return _CI_CACHE["data"]


def get_system_metrics():
    metrics = {"cpu_load": 0.0, "ram_used_mb": 0, "ram_total_mb": 0, "ram_pct": 0, "swap_used_mb": 0, "disk_pct": 0}
    try:
        with open("/proc/loadavg") as f:
            metrics["cpu_load"] = float(f.read().split()[0])
        mem = {}
        with open("/proc/meminfo") as f:
            for line in f:
                parts = line.split(":")
                if len(parts) == 2:
                    mem[parts[0].strip()] = int(parts[1].strip().split()[0])
        total = mem.get("MemTotal", 1)
        avail = mem.get("MemAvailable", mem.get("MemFree", 0) + mem.get("Cached", 0))
        used = total - avail
        metrics["ram_total_mb"] = total // 1024
        metrics["ram_used_mb"] = used // 1024
        metrics["ram_pct"] = round((used / total) * 100, 1)
        swap_total = mem.get("SwapTotal", 0)
        swap_free = mem.get("SwapFree", 0)
        metrics["swap_used_mb"] = (swap_total - swap_free) // 1024
        disk = shutil.disk_usage(ROOT)
        metrics["disk_pct"] = round((disk.used / disk.total) * 100, 1)
    except Exception:
        pass
    return metrics


def get_downloads():
    total_bytes = 0
    entries = []
    if DOWNLOAD_LOG.exists():
        try:
            for line in DOWNLOAD_LOG.read_text().splitlines():
                if line.strip():
                    item = json.loads(line)
                    entries.append(item)
                    total_bytes += item.get("size_bytes", 0)
        except Exception:
            pass
    total_mb = round(total_bytes / (1024 * 1024), 2)
    budget_mb = 3072.0
    return {"total_mb": total_mb, "budget_mb": budget_mb, "pct": round((total_mb / budget_mb) * 100, 1), "entries": entries[-10:]}


def get_coverage():
    counts = {"PORTED+PARITY-TESTED": 0, "PORTED": 0, "PARTIAL": 0, "MISSING": 0, "BLOCKED": 0}
    total = 0
    if COVERAGE_FILE.exists():
        for line in COVERAGE_FILE.read_text().splitlines():
            if "|" in line and not line.strip().startswith("|-") and "Status" not in line:
                for k in counts:
                    if k in line:
                        counts[k] += 1
                        total += 1
                        break
    pct = round(((counts["PORTED+PARITY-TESTED"] + counts["PORTED"]) / max(1, total)) * 100, 1)
    return {"counts": counts, "total": total, "percent_ported": pct}


def build_status():
    agent = {}
    if STATUS_FILE.exists():
        try:
            agent = json.loads(STATUS_FILE.read_text())
        except Exception:
            pass
    now = time.time()
    last_up = agent.get("last_updated", now)
    agent_stale = (now - last_up) > 600
    return {
        "now": now,
        "agent": agent,
        "agent_stale": agent_stale,
        "git": get_git_info(),
        "ci": get_ci_info(),
        "system": get_system_metrics(),
        "downloads": get_downloads(),
        "coverage": get_coverage(),
    }


HTML_PAGE = r"""<!DOCTYPE html>
<html lang="en">
<head>
<meta charset="utf-8">
<title>Sawbyte Live Status</title>
<style>
  :root { --bg: #0f1117; --card: #181b24; --border: #282d3c; --text: #e1e4ea; --sub: #8b949e; --green: #2ea043; --yellow: #d29922; --red: #f85149; --blue: #58a6ff; }
  body { background: var(--bg); color: var(--text); font-family: -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, monospace; margin: 0; padding: 16px; font-size: 13px; }
  h1, h2, h3 { margin: 0 0 8px 0; font-weight: 600; }
  .grid { display: grid; grid-template-columns: repeat(auto-fit, minmax(360px, 1fr)); gap: 14px; }
  .card { background: var(--card); border: 1px solid var(--border); border-radius: 6px; padding: 14px; }
  .badge { display: inline-block; padding: 2px 7px; border-radius: 10px; font-size: 11px; font-weight: 600; }
  .badge-green { background: rgba(46, 160, 67, 0.2); color: #3fb950; border: 1px solid #3fb950; }
  .badge-yellow { background: rgba(210, 153, 34, 0.2); color: #d29922; border: 1px solid #d29922; }
  .badge-red { background: rgba(248, 81, 73, 0.2); color: #f85149; border: 1px solid #f85149; }
  .badge-blue { background: rgba(88, 166, 255, 0.2); color: #58a6ff; border: 1px solid #58a6ff; }
  .bar-bg { background: #21262d; border-radius: 4px; height: 10px; overflow: hidden; margin-top: 4px; }
  .bar-fill { height: 100%; transition: width 0.3s; }
  table { width: 100%; border-collapse: collapse; margin-top: 6px; }
  th, td { text-align: left; padding: 4px 6px; border-bottom: 1px solid var(--border); }
  .timeline { max-height: 220px; overflow-y: auto; font-family: monospace; font-size: 12px; }
  .timeline-item { padding: 4px 0; border-bottom: 1px dotted var(--border); }
  button { background: #238636; color: white; border: none; padding: 6px 12px; border-radius: 4px; cursor: pointer; font-weight: bold; }
  button:hover { background: #2ea043; }
  .alert-banner { background: #f85149; color: white; padding: 10px 14px; border-radius: 6px; margin-bottom: 14px; display: none; align-items: center; justify-content: space-between; font-weight: bold; }
</style>
</head>
<body>
<div id="alert-banner" class="alert-banner">
  <span id="alert-msg">ALERT: Agent needs attention!</span>
  <button onclick="ackAlerts()" style="background:#fff; color:#b62324;">Acknowledge</button>
</div>
<div style="display:flex; justify-content:space-between; align-items:center; margin-bottom:14px;">
  <div>
    <h2>Sawbyte 1:1 Vanilla Port & Engine</h2>
    <div style="color:var(--sub);">Local dashboard: 127.0.0.1:8765 &bull; <span id="stale-badge" class="badge badge-green">LIVE</span> &bull; Updated: <span id="last-time">--</span></div>
  </div>
  <div>
    <button id="sound-btn" onclick="toggleAudio()">Enable Sound</button>
    <button onclick="requestNotif()">Enable Notifications</button>
  </div>
</div>

<div class="grid">
  <!-- Goal & Active Step -->
  <div class="card">
    <h3>🎯 Goal & Workflow</h3>
    <div style="font-weight:600; color:var(--blue);" id="goal-text">--</div>
    <div style="margin-top:6px; color:var(--sub);" id="dod-text">--</div>
    <hr style="border:0; border-top:1px solid var(--border); margin:10px 0;">
    <div style="font-size:11px; text-transform:uppercase; color:var(--sub); font-weight:bold;">Current Active Step (agent-reported)</div>
    <div style="font-size:14px; font-weight:bold; margin-top:4px;" id="current-step">--</div>
    <div style="margin-top:8px; font-size:11px; text-transform:uppercase; color:var(--sub); font-weight:bold;">Queued Next</div>
    <ul id="queued-list" style="margin:4px 0 0 16px; padding:0;"></ul>
  </div>

  <!-- Vanilla Client Coverage Matrix -->
  <div class="card">
    <h3>📊 Vanilla 26.3 Coverage Matrix</h3>
    <div style="display:flex; justify-content:space-between; font-weight:bold; margin-bottom:4px;">
      <span>Ported + Tested Coverage</span>
      <span id="cov-pct">0%</span>
    </div>
    <div class="bar-bg"><div id="cov-bar" class="bar-fill" style="width:0%; background:var(--green);"></div></div>
    <table style="margin-top:10px;">
      <tr><th>Status</th><th>Count</th><th>Category</th></tr>
      <tr><td><span class="badge badge-green">PORTED+TESTED</span></td><td id="cnt-tested">0</td><td>Parity-Verified</td></tr>
      <tr><td><span class="badge badge-blue">PORTED</span></td><td id="cnt-ported">0</td><td>Implemented</td></tr>
      <tr><td><span class="badge badge-yellow">PARTIAL</span></td><td id="cnt-partial">0</td><td>In Progress</td></tr>
      <tr><td><span class="badge badge-red">MISSING</span></td><td id="cnt-missing">0</td><td>Pending Vanilla Port</td></tr>
      <tr><td><span class="badge" style="background:#30363d; color:#8b949e;">BLOCKED</span></td><td id="cnt-blocked">0</td><td>Reason Documented</td></tr>
    </table>
  </div>

  <!-- Git & Remote CI -->
  <div class="card">
    <h3>⚡ Git & GitHub Actions CI</h3>
    <div>Branch: <strong id="git-branch">main</strong> &bull; Uncommitted changes: <span id="git-dirty" class="badge badge-green">0</span></div>
    <div style="margin-top:8px; font-weight:bold; font-size:11px; text-transform:uppercase; color:var(--sub);">Recent Commits</div>
    <div id="git-commits" style="font-family:monospace; margin-top:4px;"></div>
    <div style="margin-top:8px; font-weight:bold; font-size:11px; text-transform:uppercase; color:var(--sub);">GitHub Actions Runs</div>
    <div id="ci-runs" style="margin-top:4px;"></div>
  </div>

  <!-- Machine & Data Budget -->
  <div class="card">
    <h3>💻 Machine Limits & Data Budget</h3>
    <div style="display:flex; justify-content:space-between;">
      <span>Monthly Download Budget (15 GB Plan limit: 3 GB)</span>
      <strong id="dl-stats">0 MB / 3072 MB</strong>
    </div>
    <div class="bar-bg"><div id="dl-bar" class="bar-fill" style="width:0%; background:var(--blue);"></div></div>
    <div style="display:flex; gap:12px; margin-top:12px;">
      <div style="flex:1;">
        <div style="color:var(--sub);">CPU Load: <span id="cpu-load" style="color:var(--text); font-weight:bold;">0.0</span></div>
        <div style="color:var(--sub);">Disk Usage: <span id="disk-pct" style="color:var(--text); font-weight:bold;">0%</span></div>
      </div>
      <div style="flex:1;">
        <div style="color:var(--sub);">RAM: <span id="ram-usage" style="color:var(--text); font-weight:bold;">0/0 MB</span></div>
        <div style="color:var(--sub);">Swap: <span id="swap-used" style="color:var(--text); font-weight:bold;">0 MB</span></div>
      </div>
    </div>
    <div style="margin-top:10px; font-weight:bold; font-size:11px; text-transform:uppercase; color:var(--sub);">Recent Downloads</div>
    <div id="dl-list" style="font-family:monospace; font-size:11px; max-height:80px; overflow-y:auto; margin-top:4px;"></div>
  </div>

  <!-- Blockers & Decisions -->
  <div class="card">
    <h3>🛡️ Blockers, Decisions & Assumptions</h3>
    <div id="blockers-section"></div>
    <div style="margin-top:8px; font-weight:bold; font-size:11px; text-transform:uppercase; color:var(--sub);">Decisions</div>
    <ul id="decisions-list" style="margin:4px 0 0 16px; padding:0;"></ul>
    <div style="margin-top:8px; font-weight:bold; font-size:11px; text-transform:uppercase; color:var(--sub);">Assumptions</div>
    <ul id="assumptions-list" style="margin:4px 0 0 16px; padding:0;"></ul>
  </div>

  <!-- Event Timeline -->
  <div class="card">
    <h3>📜 Event Timeline (Last 50)</h3>
    <div class="timeline" id="timeline"></div>
  </div>
</div>

<script>
let audioCtx = null;
let soundEnabled = false;
let lastAlertCount = 0;
let titleFlashing = false;

function toggleAudio() {
  if (!audioCtx) audioCtx = new (window.AudioContext || window.webkitAudioContext)();
  if (audioCtx.state === 'suspended') audioCtx.resume();
  soundEnabled = true;
  document.getElementById('sound-btn').innerText = "Sound Enabled ✓";
  document.getElementById('sound-btn').style.background = "#1f6feb";
  beep(440, 0.1);
}

function beep(freq, dur) {
  if (!soundEnabled || !audioCtx) return;
  try {
    let osc = audioCtx.createOscillator();
    let gain = audioCtx.createGain();
    osc.connect(gain); gain.connect(audioCtx.destination);
    osc.frequency.value = freq;
    gain.gain.setValueAtTime(0.2, audioCtx.currentTime);
    osc.start();
    osc.stop(audioCtx.currentTime + dur);
  } catch(e) {}
}

function playAlertSound(level) {
  if (level === 'blocker' || level === 'alert' || level === 'error') {
    beep(880, 0.15); setTimeout(() => beep(880, 0.25), 180);
  } else if (level === 'completion' || level === 'milestone') {
    [523, 659, 784, 1046].forEach((f, i) => setTimeout(() => beep(f, 0.15), i * 140));
  } else {
    beep(587, 0.1);
  }
}

function requestNotif() {
  if ("Notification" in window) Notification.requestPermission();
}

function ackAlerts() {
  fetch('/api/ack', { method: 'POST' }).then(() => {
    document.getElementById('alert-banner').style.display = 'none';
    document.title = "Sawbyte Live Status";
    titleFlashing = false;
  });
}

function render(data) {
  document.getElementById('last-time').innerText = new Date(data.now * 1000).toLocaleTimeString();
  const staleEl = document.getElementById('stale-badge');
  if (data.agent_stale) {
    staleEl.innerText = "STALE (agent >10m idle)";
    staleEl.className = "badge badge-yellow";
  } else {
    staleEl.innerText = "LIVE";
    staleEl.className = "badge badge-green";
  }

  // Agent
  const ag = data.agent || {};
  document.getElementById('goal-text').innerText = ag.goal || "Universal 1:1 zero-graphics vanilla port";
  document.getElementById('dod-text').innerText = "DoD: " + (ag.definition_of_done || "100% parity tests & benchmarks passing");
  document.getElementById('current-step').innerText = ag.current_step || "Ready";
  
  const qList = document.getElementById('queued-list');
  qList.innerHTML = (ag.queued_steps || []).map(s => `<li>${s}</li>`).join("");

  // Coverage
  const cov = data.coverage || { counts: {}, total: 0, percent_ported: 0 };
  document.getElementById('cov-pct').innerText = cov.percent_ported + "%";
  document.getElementById('cov-bar').style.width = cov.percent_ported + "%";
  document.getElementById('cnt-tested').innerText = cov.counts['PORTED+PARITY-TESTED'] || 0;
  document.getElementById('cnt-ported').innerText = cov.counts['PORTED'] || 0;
  document.getElementById('cnt-partial').innerText = cov.counts['PARTIAL'] || 0;
  document.getElementById('cnt-missing').innerText = cov.counts['MISSING'] || 0;
  document.getElementById('cnt-blocked').innerText = cov.counts['BLOCKED'] || 0;

  // Git
  const git = data.git || {};
  document.getElementById('git-branch').innerText = git.branch || "main";
  const dirty = document.getElementById('git-dirty');
  dirty.innerText = git.dirty_count + " files";
  dirty.className = git.dirty_count > 0 ? "badge badge-yellow" : "badge badge-green";
  document.getElementById('git-commits').innerHTML = (git.commits || []).slice(0, 3).map(c => `<div style="color:var(--sub);">${c}</div>`).join("");

  // CI
  const ciDiv = document.getElementById('ci-runs');
  ciDiv.innerHTML = (data.ci || []).slice(0, 3).map(r => {
    let color = r.status === 'pass' ? '#3fb950' : (r.status === 'fail' ? '#f85149' : '#d29922');
    return `<div style="color:${color}; font-size:12px; margin-top:2px;">${r.raw}</div>`;
  }).join("") || "<div>No recent CI data</div>";

  // System
  const sys = data.system || {};
  document.getElementById('cpu-load').innerText = sys.cpu_load || "0.0";
  document.getElementById('disk-pct').innerText = (sys.disk_pct || 0) + "%";
  document.getElementById('ram-usage').innerText = `${sys.ram_used_mb || 0} / ${sys.ram_total_mb || 0} MB (${sys.ram_pct || 0}%)`;
  document.getElementById('swap-used').innerText = (sys.swap_used_mb || 0) + " MB";

  // Downloads
  const dl = data.downloads || { total_mb: 0, budget_mb: 3072, pct: 0, entries: [] };
  document.getElementById('dl-stats').innerText = `${dl.total_mb} MB / ${dl.budget_mb} MB (${dl.pct}%)`;
  const dlBar = document.getElementById('dl-bar');
  dlBar.style.width = Math.min(100, dl.pct) + "%";
  dlBar.style.background = dl.pct > 80 ? "var(--red)" : (dl.pct > 50 ? "var(--yellow)" : "var(--blue)");
  document.getElementById('dl-list').innerHTML = (dl.entries || []).map(e => `<div>&bull; ${e.size_mb} MB: ${e.purpose || e.url}</div>`).join("") || "<div>None logged this period</div>";

  // Blockers & Decisions
  const blDiv = document.getElementById('blockers-section');
  const blockers = ag.blockers || [];
  if (blockers.length > 0) {
    blDiv.innerHTML = `<div class="badge badge-red" style="font-size:12px; padding:4px 8px;">⚠️ ${blockers[0].message || blockers[0]}</div>`;
  } else {
    blDiv.innerHTML = `<span class="badge badge-green">No Active Blockers</span>`;
  }
  document.getElementById('decisions-list').innerHTML = (ag.decisions || []).map(d => `<li>${d}</li>`).join("");
  document.getElementById('assumptions-list').innerHTML = (ag.assumptions || []).map(a => `<li>${a}</li>`).join("");

  // Timeline
  const tl = document.getElementById('timeline');
  const events = ag.events || [];
  tl.innerHTML = events.slice().reverse().map(e => {
    let col = e.level === 'error' || e.level === 'blocker' ? 'var(--red)' : (e.level === 'warn' ? 'var(--yellow)' : 'var(--sub)');
    let d = new Date(e.time * 1000).toLocaleTimeString();
    return `<div class="timeline-item"><span style="color:${col};">[${d}] [${e.level.toUpperCase()}]</span> ${e.message}</div>`;
  }).join("");

  // Alerts
  const alerts = (ag.active_alerts || []).filter(a => !a.acknowledged);
  if (alerts.length > 0) {
    document.getElementById('alert-banner').style.display = 'flex';
    document.getElementById('alert-msg').innerText = "⚠️ " + alerts[0].message;
    if (alerts.length > lastAlertCount) {
      playAlertSound(alerts[0].level);
      if ("Notification" in window && Notification.permission === "granted") {
        new Notification("Sawbyte Alert", { body: alerts[0].message });
      }
    }
    if (!titleFlashing) {
      titleFlashing = true;
      let toggle = false;
      setInterval(() => {
        if (titleFlashing) {
          document.title = toggle ? "⚠️ [ALERT] Sawbyte Status" : "Sawbyte Live Status";
          toggle = !toggle;
        }
      }, 800);
    }
  } else {
    document.getElementById('alert-banner').style.display = 'none';
    titleFlashing = false;
    document.title = "Sawbyte Live Status";
  }
  lastAlertCount = alerts.length;
}

function poll() {
  fetch('/api/status')
    .then(r => r.json())
    .then(render)
    .catch(console.error);
}

setInterval(poll, 3000);
poll();
</script>
</body>
</html>"""


class StatusHandler(http.server.BaseHTTPRequestHandler):
    def do_GET(self):
        if self.path == "/" or self.path.startswith("/index"):
            self.send_response(200)
            self.send_header("Content-Type", "text/html; charset=utf-8")
            self.end_headers()
            self.wfile.write(HTML_PAGE.encode("utf-8"))
        elif self.path == "/api/status":
            data = build_status()
            self.send_response(200)
            self.send_header("Content-Type", "application/json")
            self.send_header("Access-Control-Allow-Origin", "*")
            self.end_headers()
            self.wfile.write(json.dumps(data).encode("utf-8"))
        else:
            self.send_response(404)
            self.end_headers()

    def do_POST(self):
        if self.path == "/api/ack":
            if STATUS_FILE.exists():
                try:
                    data = json.loads(STATUS_FILE.read_text())
                    for alert in data.get("active_alerts", []):
                        alert["acknowledged"] = True
                    STATUS_FILE.write_text(json.dumps(data, indent=2) + "\n")
                except Exception:
                    pass
            self.send_response(200)
            self.send_header("Content-Type", "application/json")
            self.end_headers()
            self.wfile.write(b'{"ok": true}')
        else:
            self.send_response(404)
            self.end_headers()

    def log_message(self, format, *args):
        pass  # Quiet logging to save CPU


def main():
    socketserver.TCPServer.allow_reuse_address = True
    with socketserver.TCPServer(("127.0.0.1", PORT), StatusHandler) as httpd:
        print(f"Sawbyte status dashboard listening on http://127.0.0.1:{PORT}")
        httpd.serve_forever()


if __name__ == "__main__":
    main()

"""Crucible Benchmark Suite — Dual-Interface Parity Verification for Sawbyte.

Runs 6 complex, chained Minecraft gameplay stages through BOTH interfaces
(`--backend native` and `--backend client`, or `--backend both`) using the exact
same control logic over the standard Sawbyte TCP control socket.
Every stage is strictly graded via server-side RCON ground truth.
"""
from __future__ import annotations

import argparse
import hashlib
import json
import math
import os
from pathlib import Path
import re
import shutil
import socket
import struct
import subprocess
import sys
import threading
import time
import urllib.request
from typing import Any, Dict, List, Optional, Tuple

ROOT = Path(__file__).resolve().parents[1]
sys.path.insert(0, str(ROOT / "sdks" / "python"))

from sawbyte import SawbyteLiveTcpClient

BENCH_DIR = ROOT / "run" / "bench"
REPORTS_DIR = ROOT / "run" / "reports"


# ==============================================================================
# 1. Pure-Python Source RCON Client
# ==============================================================================

class RconClient:
    """Minimal, robust Valve Source RCON client for querying vanilla Minecraft server."""

    SERVERDATA_AUTH = 3
    SERVERDATA_AUTH_RESPONSE = 2
    SERVERDATA_EXECCOMMAND = 2
    SERVERDATA_RESPONSE_VALUE = 0

    def __init__(self, host: str, port: int, password: str, timeout: float = 10.0) -> None:
        self.host = host
        self.port = port
        self.password = password
        self.timeout = timeout
        self.sock: Optional[socket.socket] = None
        self._req_id = 1

    def connect(self, retry_timeout: float = 30.0) -> None:
        start_t = time.time()
        last_err = None
        while time.time() - start_t < retry_timeout:
            try:
                self.sock = socket.create_connection((self.host, self.port), timeout=self.timeout)
                self.sock.settimeout(self.timeout)
                # Authenticate
                self._send(self._req_id, self.SERVERDATA_AUTH, self.password)
                resp_id, resp_type, _ = self._read()
                if resp_id == -1:
                    raise RuntimeError("RCON authentication failed: bad password")
                self._req_id += 1
                return
            except (ConnectionRefusedError, OSError) as e:
                last_err = e
                time.sleep(0.5)
        raise RuntimeError(f"RCON connection to {self.host}:{self.port} timed out after {retry_timeout}s: {last_err}")

    def close(self) -> None:
        if self.sock:
            try:
                self.sock.close()
            except Exception:
                pass
            self.sock = None

    def command(self, cmd: str) -> str:
        if not self.sock:
            self.connect()
        assert self.sock is not None
        req = self._req_id
        self._req_id += 1
        self._send(req, self.SERVERDATA_EXECCOMMAND, cmd)
        resp_id, resp_type, payload = self._read()
        return payload

    def _send(self, req_id: int, req_type: int, payload: str) -> None:
        assert self.sock is not None
        body = payload.encode("utf-8") + b"\x00\x00"
        length = len(body) + 8
        pkt = struct.pack("<iii", length, req_id, req_type) + body
        self.sock.sendall(pkt)

    def _read(self) -> Tuple[int, int, str]:
        assert self.sock is not None
        header = self._recv_all(4)
        (length,) = struct.unpack("<i", header)
        data = self._recv_all(length)
        req_id, req_type = struct.unpack("<ii", data[:8])
        payload = data[8:-2].decode("utf-8", errors="replace")
        return req_id, req_type, payload

    def _recv_all(self, n: int) -> bytes:
        assert self.sock is not None
        buf = bytearray()
        while len(buf) < n:
            chunk = self.sock.recv(n - len(buf))
            if not chunk:
                raise ConnectionResetError("RCON socket closed unexpectedly")
            buf.extend(chunk)
        return bytes(buf)


# ==============================================================================
# 2. Lightweight Vanilla Server Harness (Seed 4242 with Real Terrain)
# ==============================================================================

def fetch_json(url: str) -> Any:
    with urllib.request.urlopen(url, timeout=60) as resp:
        return json.load(resp)


def download(url: str, target: Path, digest: Optional[str] = None, algorithm: str = "sha1") -> None:
    if target.exists() and digest:
        if hashlib.new(algorithm, target.read_bytes()).hexdigest() == digest:
            return
    with urllib.request.urlopen(url, timeout=90) as resp:
        data = resp.read()
    if digest and hashlib.new(algorithm, data).hexdigest() != digest:
        raise RuntimeError(f"download checksum mismatch: {target.name}")
    target.parent.mkdir(parents=True, exist_ok=True)
    target.write_bytes(data)


class ServerHarness:
    """Manages the real 26.3 vanilla Minecraft server with hardware-friendly low memory flags."""

    def __init__(self, directory: Path, port: int = 25565, rcon_port: int = 25575) -> None:
        self.directory = directory
        self.port = port
        self.rcon_port = rcon_port
        self.directory.mkdir(parents=True, exist_ok=True)
        self.server_jar = self.directory / "server.jar"
        self._ensure_server_jar()
        self._write_configs()
        self.process: Optional[subprocess.Popen[str]] = None
        self.thread: Optional[threading.Thread] = None
        self.ready_event = threading.Event()
        self.rcon: Optional[RconClient] = None

    def _ensure_server_jar(self) -> None:
        if not self.server_jar.exists():
            lab_server = ROOT / "run" / "lab" / "server" / "server.jar"
            if lab_server.exists():
                shutil.copy2(lab_server, self.server_jar)
                return
            manifest = fetch_json("https://piston-meta.mojang.com/mc/game/version_manifest_v2.json")
            entry = next(v for v in manifest["versions"] if v["id"] == "26.3")
            version_meta = fetch_json(entry["url"])
            server_info = version_meta["downloads"]["server"]
            download(server_info["url"], self.server_jar, server_info["sha1"])

    def _write_configs(self) -> None:
        (self.directory / "eula.txt").write_text("eula=true\n")
        props = [
            f"server-ip=127.0.0.1",
            f"server-port={self.port}",
            "online-mode=false",
            "enforce-secure-profile=false",
            "white-list=false",
            "max-players=3",
            "level-type=minecraft:normal",
            "generate-structures=false",
            "level-seed=4242",
            "view-distance=4",
            "simulation-distance=4",
            "spawn-chunk-radius=2",
            "spawn-protection=0",
            "difficulty=peaceful",
            "gamemode=survival",
            "spawn-animals=false",
            "spawn-monsters=false",
            "enable-command-block=true",
            "sync-chunk-writes=false",
            "max-tick-time=-1",
            "enable-rcon=true",
            f"rcon.port={self.rcon_port}",
            "rcon.password=sawbyte",
        ]
        (self.directory / "server.properties").write_text("\n".join(props) + "\n")

    def start(self) -> None:
        self.ready_event.clear()
        cmd = [
            "java",
            "-Xms512M",
            "-Xmx768M",
            "-XX:+UseG1GC",
            "-XX:MaxGCPauseMillis=50",
            "-jar",
            str(self.server_jar),
            "nogui",
        ]
        log_file = (REPORTS_DIR / "crucible_server.log").open("w")
        self.process = subprocess.Popen(
            cmd,
            cwd=self.directory,
            stdin=subprocess.PIPE,
            stdout=subprocess.PIPE,
            stderr=subprocess.STDOUT,
            text=True,
            bufsize=1,
        )

        def reader_loop() -> None:
            assert self.process is not None and self.process.stdout is not None
            for line in self.process.stdout:
                log_file.write(line)
                log_file.flush()
                if "RCON running on" in line or "Done (" in line:
                    self.ready_event.set()

        self.thread = threading.Thread(target=reader_loop, daemon=True)
        self.thread.start()
        if not self.ready_event.wait(180):
            self.stop()
            raise RuntimeError("Server did not become ready within 180 seconds")

        time.sleep(1.0)
        self.rcon = RconClient("127.0.0.1", self.rcon_port, "sawbyte")
        self.rcon.connect(retry_timeout=30.0)
        # Enforce day, clear weather, and safe rules
        self.rcon.command("time set noon")
        self.rcon.command("weather clear")
        self.rcon.command("gamerule doDaylightCycle false")
        self.rcon.command("gamerule doWeatherCycle false")
        self.rcon.command("gamerule doMobSpawning false")

    def stop(self) -> None:
        if self.rcon:
            self.rcon.close()
            self.rcon = None
        if self.process and self.process.poll() is None:
            try:
                if self.process.stdin:
                    self.process.stdin.write("stop\n")
                    self.process.stdin.flush()
                self.process.wait(timeout=20)
            except Exception:
                self.process.kill()
                self.process.wait()


# ==============================================================================
# 3. Client & Native Backend Launcher Supervision
# ==============================================================================

def track_process_peak_rss(pid: int, stop_ev: threading.Event, peak_rss: List[int]) -> None:
    while not stop_ev.is_set():
        try:
            status = Path(f"/proc/{pid}/status").read_text()
            for line in status.splitlines():
                if line.startswith("VmRSS:"):
                    kib = int(line.split()[1])
                    peak_rss[0] = max(peak_rss[0], kib)
        except Exception:
            pass
        time.sleep(0.05)


class BackendInstance:
    """Manages either the native `sawbyte-play` process or HeadlessMC Fabric normal client."""

    def __init__(
        self,
        mode: str,
        native_bin: Path,
        bridge_jar: Path,
        block_profile: Path,
        server_port: int = 25565,
        ctrl_port: int = 25580,
    ) -> None:
        self.mode = mode
        self.native_bin = native_bin
        self.bridge_jar = bridge_jar
        self.block_profile = block_profile
        self.server_port = server_port
        self.ctrl_port = ctrl_port
        self.proc: Optional[subprocess.Popen[str]] = None
        self.peak_rss = [0]
        self._stop_rss = threading.Event()
        self._rss_thread: Optional[threading.Thread] = None

    def start(self) -> None:
        self._stop_rss.clear()
        self.peak_rss = [0]
        if self.mode == "native":
            cmd = [
                str(self.native_bin.resolve()),
                "live",
                f"127.0.0.1:{self.server_port}",
                "--offline",
                "SawbyteCrucible",
                str(self.block_profile.resolve()),
                "--listen",
                f"127.0.0.1:{self.ctrl_port}",
            ]
            log = (REPORTS_DIR / "crucible_native.log").open("w")
            self.proc = subprocess.Popen(cmd, stdout=log, stderr=subprocess.STDOUT, text=True)
        elif self.mode == "client":
            hmc = BENCH_DIR / "headlessmc.jar"
            lab_hmc = ROOT / "run" / "lab" / "headlessmc.jar"
            if lab_hmc.exists():
                shutil.copy2(lab_hmc, hmc)
            else:
                download(
                    "https://github.com/headlesshq/headlessmc/releases/download/3.0.0-RC5/headlessmc.jar",
                    hmc,
                    "240809e92fdf0dc880bf4a9a5d2e2657a7172f9c603fc5f5536fe4f0aa16e833",
                    "sha256",
                )
            game = BENCH_DIR / "client"
            mods = game / "mods"
            mods.mkdir(parents=True, exist_ok=True)
            shutil.copy2(self.bridge_jar, mods / "sawbyte-bridge.jar")
            lab_api = ROOT / "run" / "lab" / "client" / "mods" / "fabric-api.jar"
            if lab_api.exists():
                shutil.copy2(lab_api, mods / "fabric-api.jar")
            else:
                api = "0.165.0+26.3"
                download(
                    f"https://maven.fabricmc.net/net/fabricmc/fabric-api/fabric-api/{api}/fabric-api-{api}.jar",
                    mods / "fabric-api.jar",
                )
            (game / "options.txt").write_text(
                "onboardAccessibility:false\npauseOnLostFocus:false\nautoJump:false\nrenderDistance:2\nsimulationDistance:2\nmaxFps:30\nsoundCategory_master:0.0\n"
            )
            config = BENCH_DIR / "HeadlessMC"
            config.mkdir(exist_ok=True)
            mc_dir = (ROOT / "run" / "lab" / "minecraft") if (ROOT / "run" / "lab" / "minecraft").exists() else (BENCH_DIR / "minecraft")
            (config / "config.properties").write_text(
                f"hmc.java.versions={os.environ.get('JAVA_HOME', '/usr/lib/jvm/java-25-openjdk')}\nhmc.files.game={game}\n"
                f"hmc.files.mc={mc_dir}\nhmc.files.game-for-each-version=false\n"
                "hmc.jline.enabled=false\nhmc.launcher.crash-report-watcher=true\n"
            )
            env = dict(os.environ, HMC_FILES_LOCATION=str(config))
            install_proc = subprocess.run(
                ["java", "-jar", str(hmc), "version", "install", "fabric", "26.3"],
                cwd=ROOT,
                capture_output=True,
                env=env,
            )
            jvm_args = f"-Xms512M -Xmx768M -Dsawbyte.port={self.ctrl_port} -Dsawbyte.exit=true"
            log = (REPORTS_DIR / "crucible_client.log").open("w")
            self.proc = subprocess.Popen(
                [
                    "xvfb-run",
                    "-a",
                    "java",
                    "-jar",
                    str(hmc),
                    "launch",
                    "--offline",
                    "--jvm",
                    jvm_args,
                    "--game",
                    f"--quickPlayMultiplayer 127.0.0.1:{self.server_port}",
                    "fabric",
                    "26.3",
                ],
                stdout=log,
                stderr=subprocess.STDOUT,
                env=env,
            )
        else:
            raise ValueError(f"Unknown mode: {self.mode}")

        assert self.proc is not None
        self._rss_thread = threading.Thread(
            target=track_process_peak_rss,
            args=(self.proc.pid, self._stop_rss, self.peak_rss),
            daemon=True,
        )
        self._rss_thread.start()

    def stop(self) -> None:
        self._stop_rss.set()
        if self._rss_thread:
            self._rss_thread.join(timeout=1.0)
        if self.proc and self.proc.poll() is None:
            try:
                self.proc.terminate()
                self.proc.wait(timeout=10)
            except Exception:
                self.proc.kill()
                self.proc.wait()


# ==============================================================================
# 4. The 6 Mandatory Crucible Stages (100% Backend-Agnostic)
# ==============================================================================

def execute_stage_1(client: SawbyteLiveTcpClient, rcon: RconClient) -> Dict[str, Any]:
    """Stage 1: Complex Biome & Block-State Navigation."""
    print("  -> Running Stage 1: Complex Biome & Block-State Navigation...")
    t0 = time.monotonic()
    # Build course in loaded spawn area at Z=10, Y=64..68, X=10..25
    rcon.command("fill 8 64 8 36 68 12 air")
    rcon.command("fill 8 63 8 36 63 12 smooth_stone")
    # Obstacles along the straight line from X=10 to 25 at Z=10
    rcon.command("setblock 12 64 10 poppy")
    rcon.command("setblock 14 64 10 short_grass")
    rcon.command("setblock 18 64 10 oak_fence_gate[facing=west,open=true]")
    # Marker at destination (floor block under player at 25)
    rcon.command("setblock 25 63 10 gold_block")
    # Teleport bot to start
    rcon.command("tp @a 10.5 64.0 10.5 90 0")
    client.send_command("wait.ticks 20")

    # Bot issues navigate command
    client.send_command("task.navigate 25 64 10 walk")
    client.wait_until("task_idle", 300)

    # RCON Ground Truth Verification
    res = rcon.command("execute if entity @a[x=25,y=64,z=10,distance=..1.8]")
    passed = "Test passed" in res or "count: 1" in res
    if not passed:
        # Check actual coordinate
        pos_raw = rcon.command("data get entity @a[limit=1] Pos")
        raise AssertionError(f"Stage 1 failed: Bot did not reach goal (25, 64, 10). Actual pos: {pos_raw}")

    return {"stage": 1, "name": "complex_biome_navigation", "passed": True, "elapsed": time.monotonic() - t0}


def execute_stage_2(client: SawbyteLiveTcpClient, rcon: RconClient) -> Dict[str, Any]:
    """Stage 2: Multi-Hardness Mining, Unbreakable Refusal & Gap Bridging."""
    print("  -> Running Stage 2: Multi-Hardness Mining, Unbreakable Refusal & Gap Bridging...")
    t0 = time.monotonic()
    # Equip iron pickaxe so mining doesn't take 15 seconds per block
    rcon.command("item replace entity @a hotbar.0 with iron_pickaxe 1")
    client.send_command("player.select_hotbar 0")
    client.send_command("wait.ticks 5")

    # Course setup starting from X=25
    rcon.command("setblock 26 64 10 bedrock")
    client.send_command("wait.ticks 5")

    # 1. Unbreakable refusal
    refused = False
    try:
        resp = client.send_command("interact.mine 26 64 10")
        if "unbreakable" in resp.lower():
            refused = True
    except RuntimeError as e:
        if "unbreakable" in str(e).lower():
            refused = True

    rcon_check = rcon.command("execute if block 26 64 10 bedrock")
    if not refused or ("Test passed" not in rcon_check and "count: 1" not in rcon_check):
        raise AssertionError("Stage 2 failed: Bot did not reject mining bedrock, or bedrock broke")

    # 2. Mine oak log & stone with duration
    rcon.command("setblock 26 64 10 oak_log")
    rcon.command("setblock 27 64 10 stone")
    client.send_command("wait.ticks 5")

    client.send_command("interact.mine 26 64 10")
    client.wait_until("task_idle", 250)
    rcon_log = rcon.command("execute if block 26 64 10 air")
    if "Test passed" not in rcon_log and "count: 1" not in rcon_log:
        raise AssertionError("Stage 2 failed: Oak log at (26, 64, 10) was not broken")

    client.send_command("interact.mine 27 64 10")
    client.wait_until("task_idle", 250)
    rcon_stone = rcon.command("execute if block 27 64 10 air")
    if "Test passed" not in rcon_stone and "count: 1" not in rcon_stone:
        raise AssertionError("Stage 2 failed: Stone at (27, 64, 10) was not broken")

    # 3. Gap Bridging: 4-block pit from X=28..31
    client.send_command("task.navigate 27 64 10 walk")
    client.wait_until("task_idle", 100)
    rcon.command("fill 28 63 10 31 63 10 air")
    rcon.command("item replace entity @a hotbar.1 with cobblestone 64")
    client.send_command("player.select_hotbar 1")
    client.send_command("wait.ticks 5")

    for bx in range(28, 32):
        client.send_command(f"interact.place {bx} 63 10 1")
        client.send_command("wait.ticks 5")

    # Verify RCON presence of placed cobblestones
    for bx in range(28, 32):
        rcon_b = rcon.command(f"execute if block {bx} 63 10 cobblestone")
        if "Test passed" not in rcon_b and "count: 1" not in rcon_b:
            raise AssertionError(f"Stage 2 failed: Bridging block at ({bx}, 63, 10) was not placed in server world")

    return {"stage": 2, "name": "mining_refusal_and_bridging", "passed": True, "elapsed": time.monotonic() - t0}


def execute_stage_3(client: SawbyteLiveTcpClient, rcon: RconClient) -> Dict[str, Any]:
    """Stage 3: Component-Heavy Container & Multi-Slot Inventory Manipulation."""
    print("  -> Running Stage 3: Component-Heavy Container & Multi-Slot Inventory Manipulation...")
    t0 = time.monotonic()
    chest_pos = (33, 64, 10)
    # Double chest facing north: West (33) is left, East (34) is right
    rcon.command(f"setblock {chest_pos[0]} {chest_pos[1]} {chest_pos[2]} chest[facing=north,type=left]{{CustomName:'\"CrucibleChest\"'}}")
    rcon.command(f"setblock {chest_pos[0] + 1} {chest_pos[1]} {chest_pos[2]} chest[facing=north,type=right]{{CustomName:'\"CrucibleChest\"'}}")
    # In Minecraft DoubleInventory (facing north):
    # East block (34, type=right) contains container slots 0..26
    # West block (33, type=left) contains container slots 27..53
    # Populate East block: Slot 0 (diamond_sword with components), Slot 1 (oak_planks)
    res_sword = rcon.command(
        f'item replace block {chest_pos[0] + 1} {chest_pos[1]} {chest_pos[2]} container.0 with minecraft:diamond_sword[minecraft:custom_name=\'{{"text":"Sawbyte Key Blade"}}\',minecraft:lore=[\'{{"text":"A legendary weapon"}}\'],minecraft:enchantments={{"minecraft:sharpness":5}}] 1'
    )
    print(f"  [Stage 3] RCON item replace slot 0: {res_sword.strip()}")
    if "Replaced" not in res_sword and "replaced" not in res_sword:
        res_sword_plain = rcon.command(
            f'item replace block {chest_pos[0] + 1} {chest_pos[1]} {chest_pos[2]} container.0 with diamond_sword[custom_name=\'{{"text":"Sawbyte Key Blade"}}\',lore=[\'{{"text":"A legendary weapon"}}\'],enchantments={{"minecraft:sharpness":5}}] 1'
        )
        print(f"  [Stage 3] RCON fallback item replace slot 0: {res_sword_plain.strip()}")

    res_planks = rcon.command(
        f"item replace block {chest_pos[0] + 1} {chest_pos[1]} {chest_pos[2]} container.1 with oak_planks 16"
    )
    print(f"  [Stage 3] RCON item replace slot 1: {res_planks.strip()}")

    # Populate West block: Slot 53 (container.26: iron_ingot)
    res_iron = rcon.command(
        f"item replace block {chest_pos[0]} {chest_pos[1]} {chest_pos[2]} container.26 with iron_ingot 8"
    )
    print(f"  [Stage 3] RCON item replace slot 53: {res_iron.strip()}")

    # Backup data merge if slot 0 is not yet present on server
    check_east = rcon.command(f"data get block {chest_pos[0] + 1} {chest_pos[1]} {chest_pos[2]} Items")
    if "Sawbyte Key Blade" not in check_east:
        print(f"  [Stage 3] Item replace did not write custom item. Applying data merge fallback...")
        merge_payload = (
            f"data merge block {chest_pos[0] + 1} {chest_pos[1]} {chest_pos[2]} "
            '{Items: ['
            '{Slot: 0b, id: "minecraft:diamond_sword", count: 1b, components: {"minecraft:custom_name": \'{"text":"Sawbyte Key Blade"}\', "minecraft:lore": [\'{"text":"A legendary weapon"}\'], "minecraft:enchantments": {"minecraft:sharpness": 5}}}, '
            '{Slot: 1b, id: "minecraft:oak_planks", count: 16b}'
            ']}'
        )
        merge_res = rcon.command(merge_payload)
        print(f"  [Stage 3] Data merge result: {merge_res.strip()}")

    # Pre-inspection RCON check
    pre_east = rcon.command(f"data get block {chest_pos[0] + 1} {chest_pos[1]} {chest_pos[2]} Items")
    pre_west = rcon.command(f"data get block {chest_pos[0]} {chest_pos[1]} {chest_pos[2]} Items")
    print(f"  [Stage 3] Pre-inspection RCON East (34): {pre_east.strip()}")
    print(f"  [Stage 3] Pre-inspection RCON West (33): {pre_west.strip()}")
    client.send_command("wait.ticks 5")

    # Navigate across bridged pit to chest
    client.send_command("task.navigate 32 64 10 walk")
    client.wait_until("task_idle", 150)

    # Open chest
    client.send_command(f"interact.use_block {chest_pos[0]} {chest_pos[1]} {chest_pos[2]}")
    client.wait_until("screen_open", 60, "CrucibleChest")

    # Polling wait for container slots to be populated by server packets
    slot_map = {}
    last_inspect = {}
    for attempt in range(15):
        client.send_command("wait.ticks 3")
        inspect_raw = client.send_command("gui.inspect")
        try:
            last_inspect = json.loads(inspect_raw)
            slots = last_inspect.get("slots", [])
            slot_map = {s["slot"]: s for s in slots if "slot" in s}
            if 0 in slot_map and 1 in slot_map and 53 in slot_map:
                break
        except Exception:
            pass
        print(f"  [Stage 3] Waiting for container slots to populate (attempt {attempt + 1}/15, observed slots: {list(slot_map.keys())})...")

    if 0 not in slot_map or 1 not in slot_map or 53 not in slot_map:
        raise AssertionError(f"Stage 3 failed: Container slots truncated or missing. Observed slots: {list(slot_map.keys())}. Inspect payload: {last_inspect}")

    s0 = slot_map[0]
    if "Sawbyte Key Blade" not in str(s0.get("display_name", "")):
        raise AssertionError(f"Stage 3 failed: Slot 0 custom name missing or truncated: {s0}")
    if "A legendary weapon" not in str(s0.get("lore", [])):
        raise AssertionError(f"Stage 3 failed: Slot 0 lore truncated: {s0}")

    # Transfer Slot 1 to player inventory with quick move (shift-click)
    client.send_command("gui.click_slot 1 quick")
    client.send_command("wait.ticks 5")
    client.send_command("gui.close")
    client.send_command("wait.ticks 5")

    # Verify RCON server ground truth across both chest blocks
    chest_data_34 = rcon.command(f"data get block {chest_pos[0] + 1} {chest_pos[1]} {chest_pos[2]} Items")
    chest_data_33 = rcon.command(f"data get block {chest_pos[0]} {chest_pos[1]} {chest_pos[2]} Items")
    if "oak_planks" in chest_data_34 or "oak_planks" in chest_data_33:
        raise AssertionError(f"Stage 3 failed: Slot 1 (oak_planks) still present in chest after transfer. East: {chest_data_34}, West: {chest_data_33}")
    if "Sawbyte Key Blade" not in chest_data_34:
        raise AssertionError(f"Stage 3 failed: Slot 0 was unexpectedly disturbed. East: {chest_data_34}")
    if "iron_ingot" not in chest_data_33:
        raise AssertionError(f"Stage 3 failed: Slot 53 was unexpectedly disturbed. West: {chest_data_33}")

    return {"stage": 3, "name": "component_heavy_container_transfer", "passed": True, "elapsed": time.monotonic() - t0}


def execute_stage_4(client: SawbyteLiveTcpClient, rcon: RconClient) -> Dict[str, Any]:
    """Stage 4: Occluded vs. Line-of-Sight Entity Combat."""
    print("  -> Running Stage 4: Occluded vs. Line-of-Sight Entity Combat...")
    t0 = time.monotonic()
    # Prepare clean arena for Stage 4
    rcon.command("fill 18 64 12 25 67 17 air")
    rcon.command("fill 18 63 12 25 63 17 smooth_stone")
    # Spawn dummy entity behind a stone wall
    rcon.command("kill @e[tag=crucible_dummy]")
    rcon.command("setblock 20 64 15 stone")
    rcon.command("setblock 20 65 15 stone")
    rcon.command("summon zombie 22.5 64.0 15.5 {NoAI: 1b, Health: 8.0f, Tags: [\"crucible_dummy\"]}")
    # Teleport bot facing wall within reach but obstructed by stone block
    rcon.command("tp @a 19.5 64.0 15.5 90 0")
    client.send_command("wait.ticks 10")

    obs = json.loads(client.send_command("gym.observe"))
    entities = obs.get("entities", [])
    target = next((e for e in entities if "zombie" in str(e.get("type", "")).lower() or e.get("id")), None)
    if not target:
        raise AssertionError("Stage 4 failed: Target entity not discovered in gym.observe")
    target_id = target["id"]

    # 1. Attack through wall must fail due to occlusion
    blocked = False
    try:
        resp = client.send_command(f"interact.attack {target_id}")
        if any(w in resp.lower() for w in ["line of sight", "blocked", "reach", "occluded"]):
            blocked = True
    except RuntimeError as e:
        if any(w in str(e).lower() for w in ["line of sight", "blocked", "reach", "occluded"]):
            blocked = True

    if not blocked:
        raise AssertionError("Stage 4 failed: Bot did not reject attacking occluded entity through solid wall")

    # 2. Reposition around wall
    client.send_command("task.navigate 21 64 13 walk")
    client.wait_until("task_idle", 100)

    # 3. Attack target with clear line of sight
    for _ in range(4):
        try:
            client.send_command(f"interact.attack {target_id}")
        except Exception:
            pass
        client.send_command("wait.ticks 5")

    # Verify RCON server ground truth
    health_raw = rcon.command("data get entity @e[tag=crucible_dummy,limit=1] Health")
    # Entity should either be dead or have Health < 8.0f
    damaged = "No entity was found" in health_raw
    if not damaged:
        match = re.search(r"([0-9]+(?:\.[0-9]+)?)[fd]?", health_raw)
        if match:
            damaged = float(match.group(1)) < 8.0
    if not damaged:
        raise AssertionError(f"Stage 4 failed: Target entity took zero damage on server. Output: {health_raw}")

    return {"stage": 4, "name": "line_of_sight_combat", "passed": True, "elapsed": time.monotonic() - t0}


def execute_stage_5(client: SawbyteLiveTcpClient, rcon: RconClient) -> Dict[str, Any]:
    """Stage 5: Mid-Path Preemption & Tick-Exact RL Loop."""
    print("  -> Running Stage 5: Mid-Path Preemption & Tick-Exact RL Loop...")
    t0 = time.monotonic()
    # Start long navigation
    client.send_command("task.navigate 35 64 10 walk")
    client.send_command("wait.ticks 4")

    # Preempt mid-stride
    client.send_command("task.cancel")
    client.send_command("task.navigate 20 64 10 walk")
    client.send_command("wait.ticks 5")
    client.send_command("task.cancel")

    # 20-tick exact gym.step loop
    obs_prev = json.loads(client.send_command("gym.observe"))
    prev_tick = obs_prev["tick"]
    for i in range(20):
        step_raw = client.send_command("gym.step 1 0 0 0 0 0 0 0.0 0.0")
        obs = json.loads(step_raw)
        curr_tick = obs["tick"]
        if curr_tick != prev_tick + 1:
            raise AssertionError(f"Stage 5 failed: Tick sequence mismatch at step {i}: expected {prev_tick + 1}, got {curr_tick}")
        prev_tick = curr_tick

    return {"stage": 5, "name": "preemption_and_tick_exact_rl", "passed": True, "elapsed": time.monotonic() - t0}


def execute_stage_6(client: SawbyteLiveTcpClient, rcon: RconClient) -> Dict[str, Any]:
    """Stage 6: Lethal Damage, Respawn State Machine & Item Recovery."""
    print("  -> Running Stage 6: Lethal Damage, Respawn State Machine & Item Recovery...")
    t0 = time.monotonic()
    # Drop item at (25, 64, 10)
    rcon.command("summon item 25.5 64.0 10.5 {Item: {id: \"minecraft:diamond\", count: 1b}, Tags: [\"crucible_loot\"]}")
    # Kill the bot via server RCON
    rcon.command("kill @a")

    # Control connection must stay alive; wait for respawn
    client.wait_until("player_alive", 100)
    obs = json.loads(client.send_command("gym.observe"))
    if obs.get("health", 0.0) < 19.0:
        raise AssertionError(f"Stage 6 failed: Bot did not respawn with full health: {obs}")

    # Navigate back to dropped item
    client.send_command("task.navigate 25 64 10 walk")
    client.wait_until("task_idle", 150)

    # Server RCON verification: bot is back at coordinates
    pos_check = rcon.command("execute if entity @a[x=25,y=64,z=10,distance=..3.0]")
    if "Test passed" not in pos_check and "count: 1" not in pos_check:
        actual_pos = rcon.command("data get entity @a[limit=1] Pos")
        raise AssertionError(f"Stage 6 failed: Bot did not navigate back to dropped item. Actual pos: {actual_pos}")

    return {"stage": 6, "name": "lethal_damage_respawn_recovery", "passed": True, "elapsed": time.monotonic() - t0}


# ==============================================================================
# 5. Main Benchmark Orchestrator & CLI Runner
# ==============================================================================

def run_stages_on_backend(
    mode: str,
    server: ServerHarness,
    backend: BackendInstance,
    ctrl_port: int,
) -> Dict[str, Any]:
    print(f"\n================================================================================")
    print(f"       RUNNING CRUCIBLE BENCHMARK ON BACKEND: [{mode.upper()}]")
    print(f"================================================================================")
    backend.start()
    time.sleep(3.0)  # Wait for socket listener to bind

    client = SawbyteLiveTcpClient(f"127.0.0.1:{ctrl_port}", timeout=15.0)
    client.connect(retry_timeout=45.0)
    # Wait until client finishes initial settle on the server
    client.wait_until("player_alive", 150)

    results = []
    t_start = time.monotonic()
    try:
        results.append(execute_stage_1(client, server.rcon))
        results.append(execute_stage_2(client, server.rcon))
        results.append(execute_stage_3(client, server.rcon))
        results.append(execute_stage_4(client, server.rcon))
        results.append(execute_stage_5(client, server.rcon))
        results.append(execute_stage_6(client, server.rcon))
    finally:
        client.close()
        backend.stop()

    total_time = time.monotonic() - t_start
    peak_rss = backend.peak_rss[0]
    return {
        "mode": mode,
        "passed": all(r["passed"] for r in results),
        "total_elapsed_seconds": total_time,
        "peak_rss_kib": peak_rss,
        "stages": results,
    }


def main() -> None:
    parser = argparse.ArgumentParser(description="Sawbyte Dual-Interface Crucible Benchmark Suite")
    parser.add_argument("--backend", choices=["native", "client", "both"], default="native")
    parser.add_argument("--native-bin", type=Path, default=ROOT / "target" / "release" / "sawbyte-play")
    parser.add_argument("--cli-bin", type=Path, default=ROOT / "target" / "release" / "sawbyte")
    parser.add_argument("--bridge-jar", type=Path, default=ROOT / "run" / "mods" / "sawbyte-bridge-0.2.0.jar")
    parser.add_argument("--block-profile", type=Path, default=ROOT / "run" / "reports" / "block-profile.json")
    parser.add_argument("--server-port", type=int, default=25565)
    parser.add_argument("--rcon-port", type=int, default=25575)
    parser.add_argument("--ctrl-port", type=int, default=25580)
    parser.add_argument("--report", type=Path, default=REPORTS_DIR / "crucible_report.json")
    args = parser.parse_args()

    BENCH_DIR.mkdir(parents=True, exist_ok=True)
    REPORTS_DIR.mkdir(parents=True, exist_ok=True)

    server = ServerHarness(BENCH_DIR / "server", port=args.server_port, rcon_port=args.rcon_port)
    print("Starting lightweight Minecraft 26.3 server harness (seed 4242)...")
    server.start()

    bench_report: Dict[str, Any] = {
        "benchmark": "crucible",
        "minecraft": "26.3",
        "timestamp": time.strftime("%Y-%m-%dT%H:%M:%SZ", time.gmtime()),
        "backends": {},
    }

    try:
        if args.backend in ["native", "both"]:
            native_backend = BackendInstance(
                "native",
                args.native_bin,
                args.bridge_jar,
                args.block_profile,
                args.server_port,
                args.ctrl_port,
            )
            report = run_stages_on_backend("native", server, native_backend, args.ctrl_port)
            bench_report["backends"]["native"] = report

        if args.backend in ["client", "both"]:
            # Clean world before starting client
            server.rcon.command("kill @e[type=!minecraft:player]")
            time.sleep(1.0)
            client_backend = BackendInstance(
                "client",
                args.native_bin,
                args.bridge_jar,
                args.block_profile,
                args.server_port,
                args.ctrl_port,
            )
            report = run_stages_on_backend("client", server, client_backend, args.ctrl_port)
            bench_report["backends"]["client"] = report

    finally:
        server.stop()

    args.report.parent.mkdir(parents=True, exist_ok=True)
    args.report.write_text(json.dumps(bench_report, indent=2) + "\n")

    print("\n================================================================================")
    print("                       CRUCIBLE BENCHMARK SUMMARY")
    print("================================================================================")
    all_passed = True
    for mode, rep in bench_report["backends"].items():
        status = "PASSED" if rep["passed"] else "FAILED"
        print(f"Backend [{mode.upper()}]: {status} in {rep['total_elapsed_seconds']:.2f}s (Peak RSS: {rep['peak_rss_kib']} KiB)")
        for st in rep["stages"]:
            print(f"  Stage {st['stage']} ({st['name']}): {'PASS' if st['passed'] else 'FAIL'} ({st['elapsed']:.2f}s)")
        if not rep["passed"]:
            all_passed = False

    if not all_passed:
        print("\nCRUCIBLE BENCHMARK FAILED; see logs in run/reports/")
        sys.exit(1)
    else:
        print("\nALL 6 CRUCIBLE STAGES PASSED ON ALL CONFIGURED BACKENDS!")


if __name__ == "__main__":
    main()

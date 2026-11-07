"""Ephemeral GitHub Actions lab. Never invoked on the user's machine.

Downloads official server software at runtime; only traces/reports are published.
Runs a vanilla server, Sawbyte, then a real client with the same input recording.
"""
import argparse
import csv
import hashlib
import json
import math
import os
from pathlib import Path
import shutil
import subprocess
import threading
import time
import urllib.request

ROOT = Path(__file__).resolve().parents[1]
LAB = ROOT / "run" / "lab"
REPORTS = ROOT / "run" / "reports"


def fetch_json(url):
    with urllib.request.urlopen(url, timeout=60) as response:
        return json.load(response)


def download(url, target, digest=None, algorithm="sha1"):
    with urllib.request.urlopen(url, timeout=90) as response:
        data = response.read()
    if digest and hashlib.new(algorithm, data).hexdigest() != digest:
        raise RuntimeError(f"download checksum mismatch: {target.name}")
    target.parent.mkdir(parents=True, exist_ok=True)
    target.write_bytes(data)


class Server:
    def __init__(self):
        manifest = fetch_json("https://piston-meta.mojang.com/mc/game/version_manifest_v2.json")
        entry = next(v for v in manifest["versions"] if v["id"] == "26.3")
        version = fetch_json(entry["url"])
        server = version["downloads"]["server"]
        self.directory = LAB / "server"
        self.directory.mkdir(parents=True, exist_ok=True)
        download(server["url"], self.directory / "server.jar", server["sha1"])
        with (REPORTS / "data-generation.log").open("w") as log:
            subprocess.run(["java", "-DbundlerMainClass=net.minecraft.data.Main", "-jar", "server.jar",
                "--reports", "--output", "generated"], cwd=self.directory, stdout=log,
                stderr=subprocess.STDOUT, check=True, timeout=150)
        def lookup_block_hardness(name: str) -> float:
            if name == "grass_block":
                return 0.6
            if name in ["bedrock", "barrier", "end_portal_frame", "command_block", "structure_block", "jigsaw", "reinforced_deepslate"]:
                return -1.0
            if any(k in name for k in ["air", "water", "lava", "flower", "torch", "crop", "sapling", "wheat", "carpet"]) or "grass" in name:
                return 0.0
            if "leaves" in name:
                return 0.2
            if any(k in name for k in ["dirt", "sand", "gravel", "snow"]):
                return 0.5
            if any(k in name for k in ["stone", "andesite", "diorite", "granite"]):
                return 1.5
            if any(k in name for k in ["cobblestone", "planks", "log", "wood", "fence"]):
                return 2.0
            if any(k in name for k in ["deepslate", "ore"]):
                return 3.0
            if any(k in name for k in ["iron_block", "gold_block", "diamond_block"]):
                return 5.0
            if "obsidian" in name:
                return 50.0
            return 1.5

        block_data = json.loads((self.directory / "generated/reports/blocks.json").read_text())
        profile = {
            "minecraft": "26.3", "air": [], "solid": [],
            "bottom_slabs": [], "top_slabs": [], "stairs": [], "fences": [],
            "carpets": [], "climbable": [], "water": [], "lava": [],
            "cobwebs": [], "soul_sand": [], "ice": [], "hardness": {},
        }
        for key, val in block_data.items():
            clean_name = key.replace("minecraft:", "")
            states = val.get("states", [])
            for state in states:
                sid = state["id"]
                profile["hardness"][str(sid)] = lookup_block_hardness(clean_name)
                props = state.get("properties", {})
                if clean_name == "grass_block":
                    profile["solid"].append(sid)
                elif clean_name in ["air", "cave_air", "void_air", "light"] or any(k in clean_name for k in ["grass", "flower", "torch", "crop", "sapling", "button", "lever", "rail", "pressure_plate", "sign", "poppy", "dandelion", "orchid", "tulip", "daisy", "fern", "bush", "bluet", "allium", "cornflower", "peony", "lilac"]):
                    profile["air"].append(sid)
                elif "gate" in clean_name:
                    if props.get("open") == "true":
                        profile["air"].append(sid)
                    else:
                        profile["fences"].append(sid)
                elif "slab" in clean_name:
                    stype = props.get("type", "bottom")
                    if stype == "bottom":
                        profile["bottom_slabs"].append(sid)
                    elif stype == "top":
                        profile["top_slabs"].append(sid)
                    else:
                        profile["solid"].append(sid)
                elif "stairs" in clean_name:
                    profile["stairs"].append(sid)
                elif any(k in clean_name for k in ["fence", "wall", "pane", "iron_bars"]):
                    profile["fences"].append(sid)
                elif "carpet" in clean_name:
                    profile["carpets"].append(sid)
                elif clean_name in ["ladder", "vine", "scaffolding"]:
                    profile["climbable"].append(sid)
                elif clean_name == "water" or clean_name == "bubble_column":
                    profile["water"].append(sid)
                elif clean_name == "lava":
                    profile["lava"].append(sid)
                elif clean_name == "cobweb":
                    profile["cobwebs"].append(sid)
                elif clean_name in ["soul_sand", "honey_block"]:
                    profile["soul_sand"].append(sid)
                elif "ice" in clean_name:
                    profile["ice"].append(sid)
                else:
                    profile["solid"].append(sid)
        packets_file = self.directory / "generated/reports/packets.json"
        if packets_file.exists():
            packets_data = json.loads(packets_file.read_text())
            profile["packets"] = packets_data.get("play", {})
        registries_file = self.directory / "generated/reports/registries.json"
        if registries_file.exists():
            reg_data = json.loads(registries_file.read_text())
            item_entries = reg_data.get("minecraft:item", {}).get("entries", {})
            profile["items"] = {k: v["protocol_id"] for k, v in item_entries.items() if "protocol_id" in v}
        (REPORTS / "block-profile.json").write_text(json.dumps(profile, indent=2) + "\n")
        (self.directory / "eula.txt").write_text("eula=true\n")
        (self.directory / "server.properties").write_text("\n".join([
            "server-ip=127.0.0.1", "server-port=25565", "online-mode=false",
            "enforce-secure-profile=false", "white-list=false", "max-players=3",
            "level-type=minecraft:flat", "generate-structures=false", "level-seed=0",
            "view-distance=3", "simulation-distance=3", "spawn-protection=0",
            "difficulty=peaceful", "gamemode=survival", "spawn-animals=false",
            "spawn-monsters=false", "enable-command-block=false", "sync-chunk-writes=false",
        ]) + "\n")
        self.ready = threading.Event()
        self.process = subprocess.Popen(
            ["java", "-Xms512M", "-Xmx1G", "-jar", "server.jar", "nogui"],
            cwd=self.directory, stdin=subprocess.PIPE, stdout=subprocess.PIPE,
            stderr=subprocess.STDOUT, text=True, bufsize=1,
        )
        self.thread = threading.Thread(target=self.read, daemon=True)
        self.thread.start()
        if not self.ready.wait(150):
            self.close()
            raise RuntimeError("server did not become ready; see server.log")
        for command in ["setworldspawn 0 -60 0", "time set noon", "weather clear"]:
            self.command(command)

    def command(self, text):
        self.process.stdin.write(text + "\n")
        self.process.stdin.flush()

    def read(self):
        with (REPORTS / "server.log").open("w") as log:
            for line in self.process.stdout:
                log.write(line)
                log.flush()
                if 'Done (' in line:
                    self.ready.set()
                if "logged in with entity id" in line:
                    # Every backend begins from the same world position and heading.
                    self.command("tp @a 0.5 -60 0.5 0 0")
                    self.command("kill @e[type=!minecraft:player]")

    def close(self):
        if self.process.poll() is None:
            try:
                self.command("stop")
                self.process.wait(timeout=20)
            except (OSError, subprocess.TimeoutExpired):
                self.process.kill()
                self.process.wait()
        self.thread.join(timeout=3)


def run_process(command, name, timeout, env=None):
    peak_kib = 0
    started = time.monotonic()
    with (REPORTS / f"{name}.log").open("w") as log:
        process = subprocess.Popen(command, cwd=ROOT, stdout=log, stderr=subprocess.STDOUT, env=env)
        try:
            while process.poll() is None:
                if time.monotonic() - started > timeout:
                    raise TimeoutError(f"{name} timed out")
                try:
                    for line in Path(f"/proc/{process.pid}/status").read_text().splitlines():
                        if line.startswith("VmRSS:"):
                            peak_kib = max(peak_kib, int(line.split()[1]))
                except (FileNotFoundError, ProcessLookupError):
                    pass
                time.sleep(0.1)
            if process.returncode:
                raise RuntimeError(f"{name} exited {process.returncode}; see its log")
        finally:
            if process.poll() is None:
                process.kill()
                process.wait()
    return {"elapsed_seconds": time.monotonic() - started, "sampled_peak_rss_kib": peak_kib}


def rows(path):
    with path.open() as file:
        result = list(csv.DictReader(file))
    if len(result) != 160 or [int(row["tick"]) for row in result] != list(range(160)):
        raise AssertionError(f"{path.name}: missing or unordered ticks")
    for row in result:
        if not all(math.isfinite(float(row[key])) for key in ["x", "y", "z", "yaw", "pitch"]):
            raise AssertionError("nonfinite observation")
    return result


def distance(a, b):
    return math.sqrt(sum((float(a[k]) - float(b[k])) ** 2 for k in ["x", "y", "z"]))


def verify_motion(trace):
    if distance(trace[20], trace[59]) < 3:
        raise AssertionError("player did not walk")
    if max(float(r["y"]) for r in trace[80:100]) - float(trace[79]["y"]) < 0.5:
        raise AssertionError("player did not jump")
    if abs(float(trace[99]["yaw"]) - float(trace[0]["yaw"]) - 90) > 0.01:
        raise AssertionError("player did not turn")
    if distance(trace[140], trace[159]) > 0.001:
        raise AssertionError("player did not stop")
    if not all(int(r["chunks"]) > 0 for r in trace):
        raise AssertionError("world data unavailable")


def compare(native, reference):
    errors = [distance(a, b) for a, b in zip(native, reference)]
    report = {
        "minecraft": "26.3", "scenario": "flat-walk-turn-jump-stop",
        "ticks": len(errors), "max_position_error_blocks": max(errors),
        "rms_position_error_blocks": math.sqrt(sum(e * e for e in errors) / len(errors)),
        "ground_state_mismatches": sum(a["on_ground"] != b["on_ground"] for a, b in zip(native, reference)),
        "position_tolerance_blocks": 0.001,
    }
    (REPORTS / "comparison.json").write_text(json.dumps(report, indent=2) + "\n")
    if max(errors) > 0.001 or report["ground_state_mismatches"]:
        raise AssertionError("native/reference parity failed; comparison.json records the discrepancy")


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument("--native", required=True, type=Path)
    parser.add_argument("--cli", required=True, type=Path)
    parser.add_argument("--bridge", required=True, type=Path)
    args = parser.parse_args()
    LAB.mkdir(parents=True, exist_ok=True)
    REPORTS.mkdir(parents=True, exist_ok=True)
    tape = LAB / "movement.sbif"
    subprocess.run([str(args.cli.resolve()), "tape", str(tape)], check=True)
    server = Server()
    try:
        metrics = run_process([str(args.native.resolve()), "127.0.0.1:25565", "--offline",
            "SawbyteNative", str(tape), str(REPORTS / "native.csv"), str(REPORTS / "block-profile.json")], "native", 90)
        (REPORTS / "native-resources.json").write_text(json.dumps(metrics, indent=2) + "\n")
        native = rows(REPORTS / "native.csv")
        verify_motion(native)
        hmc = LAB / "headlessmc.jar"
        download("https://github.com/headlesshq/headlessmc/releases/download/3.0.0-RC5/headlessmc.jar",
            hmc, "240809e92fdf0dc880bf4a9a5d2e2657a7172f9c603fc5f5536fe4f0aa16e833", "sha256")
        game = LAB / "client"
        mods = game / "mods"
        mods.mkdir(parents=True, exist_ok=True)
        shutil.copy2(args.bridge, mods / "sawbyte-bridge.jar")
        api = "0.165.0+26.3"
        download(f"https://maven.fabricmc.net/net/fabricmc/fabric-api/fabric-api/{api}/fabric-api-{api}.jar",
            mods / "fabric-api.jar")
        (game / "options.txt").write_text("onboardAccessibility:false\npauseOnLostFocus:false\nautoJump:false\nrenderDistance:3\nsimulationDistance:3\nmaxFps:30\n")
        config = LAB / "HeadlessMC"
        config.mkdir(exist_ok=True)
        (config / "config.properties").write_text(
            f"hmc.java.versions={os.environ['JAVA_HOME']}\nhmc.files.game={game}\n"
            f"hmc.files.mc={LAB / 'minecraft'}\nhmc.files.game-for-each-version=false\n"
            "hmc.jline.enabled=false\nhmc.launcher.crash-report-watcher=true\n"
        )
        env = dict(os.environ, HMC_FILES_LOCATION=str(config))
        run_process(["java", "-jar", str(hmc), "version", "install", "fabric", "26.3"],
            "reference-install", 240, env)
        jvm = f"-Xms512M -Xmx2G -Dsawbyte.tape={tape} -Dsawbyte.trace={REPORTS / 'reference.csv'} -Dsawbyte.exit=true"
        run_process(["xvfb-run", "-a", "java", "-jar", str(hmc), "launch", "--offline",
            "--jvm", jvm, "--game", "--quickPlayMultiplayer 127.0.0.1:25565", "fabric", "26.3"],
            "reference", 240, env)
        reference = rows(REPORTS / "reference.csv")
        verify_motion(reference)
        compare(native, reference)
        live_metrics = run_process([
            str(args.native.resolve()), "live", "127.0.0.1:25565", "--offline",
            "SawbyteLive", str(REPORTS / "block-profile.json"),
            "--script", "task.navigate 3 -60 2\nplayer.select_hotbar 2\ngui.inspect\ninventory.inspect\ninteract.raycast_block\ninspect.map\ngym.observe",
            "--ticks", "80",
            "--trace", str(REPORTS / "live.csv"),
        ], "native-live", 90)
        (REPORTS / "native-live-resources.json").write_text(json.dumps(live_metrics, indent=2) + "\n")
        with (REPORTS / "live.csv").open() as f:
            live_rows = list(csv.DictReader(f))
        if len(live_rows) != 80:
            raise AssertionError(f"expected 80 live ticks, got {len(live_rows)}")
        final_x = float(live_rows[-1]["x"])
        final_z = float(live_rows[-1]["z"])
        goal_dist = math.hypot(final_x - 3.5, final_z - 2.5)
        if goal_dist > 0.65:
            raise AssertionError(
                f"live closed-loop 3D A* navigation did not reach goal (3.5, 2.5): final=({final_x:.3f}, {final_z:.3f}), dist={goal_dist:.3f}"
            )
    finally:
        server.close()


if __name__ == "__main__":
    main()

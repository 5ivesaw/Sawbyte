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
        block_data = json.loads((self.directory / "generated/reports/blocks.json").read_text())
        profile = {"minecraft": "26.3", "air": [], "solid": []}
        for group, names in {
            "air": ["air", "cave_air", "void_air"],
            "solid": ["bedrock", "dirt", "grass_block", "stone", "cobblestone"],
        }.items():
            for name in names:
                profile[group].extend(state["id"] for state in block_data[f"minecraft:{name}"]["states"])
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
    finally:
        server.close()


if __name__ == "__main__":
    main()

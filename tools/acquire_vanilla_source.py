#!/usr/bin/env python3
"""Acquires and decompiles vanilla 26.3 client source under strict machine and data limits.
Size estimate: ~41.4 MB total (< 500 MB limit). Heap capped at 2.5 GB (< 3 GB limit).
Logs downloads to run/status/downloads.jsonl and outputs to gitignored .cache/vanilla/26.3/src.
"""

import hashlib
import json
import os
import subprocess
import sys
import time
import urllib.request
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
sys.path.insert(0, str(ROOT / "tools" / "dashboard"))
import status

CACHE_DIR = ROOT / ".cache" / "vanilla" / "26.3"
CLIENT_JAR = CACHE_DIR / "client.jar"
VINEFLOWER_JAR = CACHE_DIR / "vineflower.jar"
SRC_DIR = CACHE_DIR / "src"

VINEFLOWER_URL = "https://github.com/Vineflower/vineflower/releases/download/1.11.1/vineflower-1.11.1.jar"
VINEFLOWER_EXPECTED_SIZE = 1847869

CLIENT_URL = "https://piston-data.mojang.com/v1/objects/e877b6a07acd633fb3bb475002175cec036e7b87/client.jar"
CLIENT_EXPECTED_SIZE = 41483720
CLIENT_SHA1 = "e877b6a07acd633fb3bb475002175cec036e7b87"


def download_file(url: str, dest: Path, purpose: str, expected_size: int = None, expected_sha1: str = None) -> None:
    if dest.exists() and dest.stat().st_size > 0:
        print(f"Already downloaded: {dest.name} ({dest.stat().st_size} bytes)")
        return
    print(f"Downloading {url} to {dest}...")
    dest.parent.mkdir(parents=True, exist_ok=True)
    
    # Measure and stream download
    start = time.time()
    req = urllib.request.Request(url, headers={"User-Agent": "SawbyteSourceFetcher/1.0"})
    with urllib.request.urlopen(req, timeout=90) as resp, open(dest, "wb") as out_f:
        downloaded = 0
        sha = hashlib.sha1()
        while True:
            chunk = resp.read(65536)
            if not chunk:
                break
            out_f.write(chunk)
            sha.update(chunk)
            downloaded += len(chunk)
    elapsed = time.time() - start
    print(f"Downloaded {downloaded} bytes in {elapsed:.2f}s ({round(downloaded / (1024 * 1024), 2)} MB)")
    if expected_sha1 and sha.hexdigest() != expected_sha1:
        dest.unlink(missing_ok=True)
        raise RuntimeError(f"SHA1 mismatch for {dest.name}: got {sha.hexdigest()}, expected {expected_sha1}")
    status.log_download(url, downloaded, purpose)


def decompile() -> None:
    SRC_DIR.mkdir(parents=True, exist_ok=True)
    print("Running Vineflower decompiler with 2.5 GB heap cap...")
    cmd = [
        "java",
        "-Xms512M",
        "-Xmx2500M",
        "-jar",
        str(VINEFLOWER_JAR),
        "-dgs=1",  # decompile generic signatures
        "-hdc=0",  # hide default constructors
        "-rbr=0",  # hide bridge methods
        "-asc=1",  # encode non-ASCII characters
        str(CLIENT_JAR),
        str(SRC_DIR),
    ]
    t0 = time.time()
    subprocess.run(cmd, check=True, timeout=900)
    print(f"Decompilation finished in {time.time() - t0:.2f}s.")
    status.add_event("info", f"Vanilla 26.3 client source decompiled into {SRC_DIR}")


def main() -> None:
    status.set_step("Acquiring and decompiling vanilla 26.3 client source into .cache/vanilla/26.3/src")
    download_file(VINEFLOWER_URL, VINEFLOWER_JAR, "Vineflower standalone decompiler 1.11.1", VINEFLOWER_EXPECTED_SIZE)
    download_file(CLIENT_URL, CLIENT_JAR, "Mojang vanilla 26.3 client jar", CLIENT_EXPECTED_SIZE, CLIENT_SHA1)
    decompile()
    print("Vanilla 26.3 source acquisition complete!")


if __name__ == "__main__":
    main()

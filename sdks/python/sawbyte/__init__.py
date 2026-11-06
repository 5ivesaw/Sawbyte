"""Sawbyte Python SDK — Live TCP Session Client, MCP JSON-RPC 2.0 Client, Workflow Builder, and Gymnasium RL Adapter."""

from __future__ import annotations
from dataclasses import dataclass
import json
import math
import socket
import subprocess
import time
from typing import Any, Callable, Dict, List, Optional, Tuple


@dataclass
class ItemSelector:
    id: Optional[str] = None
    name_exact: Optional[str] = None
    name_contains: Optional[str] = None
    lore_contains: Optional[str] = None
    min_count: Optional[int] = None


class WorkflowBuilder:
    """Declarative & programmable workflow builder compiling to SawbyteScript commands."""

    def __init__(self, policy: str = "humanised") -> None:
        self.policy = policy
        self.statements: List[str] = []

    def create_instance(self, name: str, profile: str = "sawbyte-26.3", mode: str = "native") -> "WorkflowBuilder":
        self.statements.append(f"launcher.create_instance {name} {profile} {mode}")
        return self

    def backup_instance(self, name: str) -> "WorkflowBuilder":
        self.statements.append(f"launcher.backup {name}")
        return self

    def connect_live(self, endpoint: str) -> "WorkflowBuilder":
        self.statements.append(f"launcher.connect_live {endpoint}")
        return self

    def goto_menu(self, target: str) -> "WorkflowBuilder":
        self.statements.append(f"menu.goto {target}")
        return self

    def set_setting(self, key: str, value: str) -> "WorkflowBuilder":
        self.statements.append(f"settings.set {key} {value}")
        return self

    def select_hotbar(self, slot: int) -> "WorkflowBuilder":
        self.statements.append(f"player.select_hotbar {slot}")
        return self

    def mine_block(self, x: int, y: int, z: int) -> "WorkflowBuilder":
        self.statements.append(f"interact.mine {x} {y} {z}")
        return self

    def place_block(self, x: int, y: int, z: int, slot: int = 0) -> "WorkflowBuilder":
        self.statements.append(f"interact.place {x} {y} {z} {slot}")
        return self

    def navigate(self, x: int, y: int, z: int) -> "WorkflowBuilder":
        self.statements.append(f"task.navigate {x} {y} {z}")
        return self

    def wait_ticks(self, ticks: int) -> "WorkflowBuilder":
        self.statements.append(f"wait.ticks {ticks}")
        return self

    def wait_until(self, condition: str, timeout_ticks: int = 100, filter_arg: Optional[str] = None) -> "WorkflowBuilder":
        if filter_arg:
            self.statements.append(f"wait.until {condition} {timeout_ticks} \"{filter_arg}\"")
        else:
            self.statements.append(f"wait.until {condition} {timeout_ticks}")
        return self

    def gui_click_slot(self, slot: int, mode: str = "pickup") -> "WorkflowBuilder":
        self.statements.append(f"gui.click_slot {slot} {mode}")
        return self

    def gui_close(self) -> "WorkflowBuilder":
        self.statements.append("gui.close")
        return self

    def chat(self, message: str) -> "WorkflowBuilder":
        self.statements.append(f"chat.send {message}")
        return self

    def command(self, cmd: str) -> "WorkflowBuilder":
        self.statements.append(f"cmd.run {cmd}")
        return self

    def observe_gym(self) -> "WorkflowBuilder":
        self.statements.append("gym.observe")
        return self

    def cancel_task(self) -> "WorkflowBuilder":
        self.statements.append("task.cancel")
        return self

    def observe_sensory(self) -> "WorkflowBuilder":
        self.statements.append("inspect.sensory")
        return self

    def observe_svpe(self, rx: int = 8, ry: int = 4, rz: int = 8) -> "WorkflowBuilder":
        self.statements.append(f"inspect.svpe {rx} {ry} {rz}")
        return self

    def observe_xray(self) -> "WorkflowBuilder":
        self.statements.append("inspect.xray")
        return self

    def observe_viewport(self) -> "WorkflowBuilder":
        self.statements.append("inspect.viewport")
        return self

    def save_memory(self) -> "WorkflowBuilder":
        self.statements.append("memory.save")
        return self

    def compile_script(self) -> str:
        return "\n".join(self.statements)


class SawbyteLiveTcpClient:
    """Direct low-latency TCP client for a running `sawbyte-play live --listen <host:port>` instance."""

    def __init__(self, endpoint: str = "127.0.0.1:25580", timeout: float = 30.0) -> None:
        host, port_str = endpoint.rsplit(":", 1)
        self.host = host
        self.port = int(port_str)
        self.timeout = timeout
        self._sock: Optional[socket.socket] = None
        self._reader: Any = None

    def __enter__(self) -> "SawbyteLiveTcpClient":
        self.connect()
        return self

    def __exit__(self, exc_type: Any, exc: Any, tb: Any) -> None:
        self.close()

    def connect(self, retry_timeout: float = 30.0) -> None:
        start_t = time.time()
        last_err = None
        while time.time() - start_t < retry_timeout:
            try:
                self._sock = socket.create_connection((self.host, self.port), timeout=self.timeout)
                self._sock.setsockopt(socket.IPPROTO_TCP, socket.TCP_NODELAY, 1)
                self._reader = self._sock.makefile("r", encoding="utf-8")
                return
            except (ConnectionRefusedError, OSError) as e:
                last_err = e
                time.sleep(0.5)
        raise RuntimeError(f"Failed to connect to {self.host}:{self.port} within {retry_timeout}s: {last_err}")

    def close(self) -> None:
        if self._reader is not None:
            self._reader.close()
            self._reader = None
        if self._sock is not None:
            self._sock.close()
            self._sock = None

    def send_command(self, line: str, timeout: Optional[float] = None) -> str:
        if self._sock is None or self._reader is None:
            self.connect()
        assert self._sock is not None and self._reader is not None
        old_timeout = self._sock.gettimeout()
        if timeout is not None:
            self._sock.settimeout(timeout)
        try:
            self._sock.sendall((line.strip() + "\n").encode("utf-8"))
            reply = self._reader.readline()
            if not reply:
                raise RuntimeError("sawbyte-play live TCP socket closed")
            text = reply.rstrip("\r\n")
            if not text.lstrip().startswith(("{", "[")):
                text = text.replace("\\n", "\n")
            if text.startswith("error: "):
                raise RuntimeError(text)
            return text
        finally:
            if timeout is not None:
                self._sock.settimeout(old_timeout)

    def wait_ticks(self, ticks: int) -> str:
        req_timeout = max(self.timeout, (ticks * 0.05) + 10.0)
        return self.send_command(f"wait.ticks {ticks}", timeout=req_timeout)

    def wait_until(self, condition: str, timeout_ticks: int = 100, filter_arg: Optional[str] = None) -> str:
        req_timeout = max(self.timeout, (timeout_ticks * 0.05) + 10.0)
        cmd = f"wait.until {condition} {timeout_ticks} \"{filter_arg}\"" if filter_arg else f"wait.until {condition} {timeout_ticks}"
        return self.send_command(cmd, timeout=req_timeout)

    def gui_click_slot(self, slot: int, mode: str = "pickup") -> str:
        return self.send_command(f"gui.click_slot {slot} {mode}")

    def gui_close(self) -> str:
        return self.send_command("gui.close")

    def inspect_sensory(self) -> str:
        return self.send_command("inspect.sensory")

    def inspect_sensory_json(self) -> Dict[str, Any]:
        return json.loads(self.send_command("inspect.sensory.json"))

    def inspect_sensory_delta(self) -> str:
        return self.send_command("inspect.sensory.delta")

    def inspect_svpe(self, rx: int = 8, ry: int = 4, rz: int = 8) -> str:
        return self.send_command(f"inspect.svpe {rx} {ry} {rz}")

    def inspect_svpe_height(self, radius: int = 12) -> str:
        return self.send_command(f"inspect.svpe.height {radius}")

    def inspect_viewport(self) -> str:
        return self.send_command("inspect.viewport")

    def inspect_xray(self) -> str:
        return self.send_command("inspect.xray")

    def inspect_long_range(self) -> str:
        return self.send_command("inspect.long_range")

    def inspect_small_map(self) -> str:
        return self.send_command("inspect.small_map")

    def memory_status(self) -> str:
        return self.send_command("memory.status")

    def memory_save(self, path: Optional[str] = None) -> str:
        return self.send_command(f"memory.save {path}" if path else "memory.save")


class SawbyteMcpClient:
    """Stdio JSON-RPC 2.0 MCP client driving `sawbyte mcp --stdio` or `sawbyte-play live --mcp-stdio`."""

    def __init__(self, binary_path: str = "sawbyte", live_endpoint: Optional[str] = None) -> None:
        self.binary_path = binary_path
        self.live_endpoint = live_endpoint
        self._proc: Optional[subprocess.Popen[str]] = None
        self._req_id = 0

    def __enter__(self) -> "SawbyteMcpClient":
        self.start()
        return self

    def __exit__(self, exc_type: Any, exc: Any, tb: Any) -> None:
        self.close()

    def start(self) -> Dict[str, Any]:
        cmd = [self.binary_path, "mcp", "--stdio"]
        if self.live_endpoint:
            cmd.extend(["--connect", self.live_endpoint])
        self._proc = subprocess.Popen(
            cmd,
            stdin=subprocess.PIPE,
            stdout=subprocess.PIPE,
            stderr=subprocess.PIPE,
            text=True,
            bufsize=1,
        )
        return self.request("initialize", {})

    def close(self) -> None:
        if self._proc is not None:
            if self._proc.stdin:
                try:
                    self._proc.stdin.close()
                except Exception:
                    pass
            if self._proc.stdout:
                try:
                    self._proc.stdout.close()
                except Exception:
                    pass
            if self._proc.stderr:
                try:
                    self._proc.stderr.close()
                except Exception:
                    pass
            try:
                self._proc.wait(timeout=5)
            except subprocess.TimeoutExpired:
                self._proc.kill()
                self._proc.wait()
            self._proc = None

    def request(self, method: str, params: Optional[Dict[str, Any]] = None) -> Dict[str, Any]:
        if self._proc is None or self._proc.stdin is None or self._proc.stdout is None:
            raise RuntimeError("MCP client is not started")
        self._req_id += 1
        payload = {"jsonrpc": "2.0", "id": self._req_id, "method": method}
        if params is not None:
            payload["params"] = params
        self._proc.stdin.write(json.dumps(payload) + "\n")
        self._proc.stdin.flush()
        line = self._proc.stdout.readline()
        if not line:
            raise RuntimeError("MCP server closed stdout unexpectedly")
        return json.loads(line)

    def list_tools(self) -> List[Dict[str, Any]]:
        return self.request("tools/list")["result"]["tools"]

    def call_tool(self, name: str, script: str = "") -> str:
        resp = self.request("tools/call", {"name": name, "script": script})
        result = resp["result"]
        if result.get("isError"):
            raise RuntimeError(result["content"][0]["text"])
        return result["content"][0]["text"]

    def execute_workflow(self, workflow: WorkflowBuilder) -> str:
        return self.call_tool("code_execute", workflow.compile_script())


class SawbyteEnv:
    """Gymnasium-compatible RL environment driving a live `sawbyte-play live --listen <host:port>` session."""

    def __init__(
        self,
        live_endpoint: str = "127.0.0.1:25580",
        goal_pos: Tuple[float, float, float] = (4.5, -60.0, 4.5),
        max_steps: int = 160,
        reward_fn: Optional[Callable[[Dict[str, Any], bool, float], float]] = None,
        reset_provider: Optional[Callable[[Optional[int]], None]] = None,
    ) -> None:
        self.client = SawbyteLiveTcpClient(live_endpoint)
        self.goal_pos = goal_pos
        self.max_steps = max_steps
        self.reward_fn = reward_fn
        self.reset_provider = reset_provider
        self.step_count = 0

    def reset(self, seed: Optional[int] = None) -> Tuple[Dict[str, Any], Dict[str, Any]]:
        if self.reset_provider is None:
            raise RuntimeError("reset requires a controlled-server reset provider")
        self.reset_provider(seed)
        self.step_count = 0
        obs = self.observe()
        return obs, {"seed": seed}

    def observe(self) -> Dict[str, Any]:
        raw = self.client.send_command("gym.observe")
        return json.loads(raw)

    def step(self, action: Dict[str, Any]) -> Tuple[Dict[str, Any], float, bool, bool, Dict[str, Any]]:
        b = lambda k: 1 if action.get(k, False) else 0
        yaw_d = float(action.get("yaw_delta", 0.0))
        pitch_d = float(action.get("pitch_delta", 0.0))
        cmd = (
            f"gym.step {b('forward')} {b('back')} {b('left')} {b('right')} "
            f"{b('jump')} {b('sneak')} {b('sprint')} {yaw_d:.4f} {pitch_d:.4f}"
        )
        resp = self.client.send_command(cmd)
        try:
            obs = json.loads(resp)
        except json.JSONDecodeError as error:
            raise RuntimeError("training step did not return an observation") from error
        if not isinstance(obs, dict) or "pos" not in obs:
            raise RuntimeError("training step returned an invalid observation")
        self.step_count += 1
        pos = obs.get("pos", [0.0, 0.0, 0.0])
        dist = math.sqrt(sum(( float(a) - float(g) ) ** 2 for a, g in zip(pos, self.goal_pos)))
        dead = float(obs.get("health", 20.0)) <= 0.0
        reached = dist < 0.75
        terminated = dead or reached
        truncated = self.step_count >= self.max_steps
        if self.reward_fn is not None:
            reward = self.reward_fn(obs, terminated, dist)
        else:
            if dead:
                reward = -50.0
            elif reached:
                reward = 10.0
            else:
                reward = -0.05 * dist
        return obs, reward, terminated, truncated, {"distance_to_goal": dist, "health": obs.get("health", 20.0), "dead": dead}

    def close(self) -> None:
        self.client.close()



def rle_decode_svpe1_tokens(rle_str: str) -> List[str]:
    """Decodes an SVPE-1 `<count><char>` RLE token stream into a flat list of single-character symbols."""
    out: List[str] = []
    count_acc = 0
    for ch in rle_str:
        if ch.isdigit():
            count_acc = count_acc * 10 + int(ch)
        else:
            n = count_acc if count_acc > 0 else 1
            out.extend([ch] * n)
            count_acc = 0
    return out


def decode_svpe1_rle3d(encoded: str) -> Dict[str, Any]:
    """Losslessly decodes an `SVPE1:RLE3D|origin=(ox,oy,oz)|dims=(dx,dy,dz)|...` string into a 3D volume dict."""
    parts = encoded.strip().split("|")
    if len(parts) < 6 or parts[0] != "SVPE1:RLE3D":
        raise ValueError(f"Invalid SVPE1:RLE3D header: {encoded[:48]}")
    origin = tuple(int(x) for x in parts[1].split("=", 1)[1].strip("()").split(","))
    dims = tuple(int(x) for x in parts[2].split("=", 1)[1].strip("()").split(","))
    dx, dy, dz = dims
    slice_len = dx * dz
    body = "|".join(parts[5:])
    raw_layers = body.split("/")
    if len(raw_layers) != dy:
        raise ValueError(f"Expected {dy} Y-layers, got {len(raw_layers)}")
    layers_2d: List[List[List[str]]] = []
    prev_flat: List[str] = []
    for layer_s in raw_layers:
        flat = list(prev_flat) if layer_s == "@dup" else rle_decode_svpe1_tokens(layer_s)
        if len(flat) != slice_len:
            raise ValueError(f"Decoded slice length {len(flat)} != {slice_len}")
        prev_flat = flat
        rows = [flat[z * dx : (z + 1) * dx] for z in range(dz)]
        layers_2d.append(rows)
    return {
        "origin": origin,
        "dims": dims,
        "layers_yzx": layers_2d,
    }


def decode_svpe1_hd25d(encoded: str) -> Dict[str, Any]:
    """Decodes an `SVPE1:HD25D|center=(px,py,pz)|radius=R|...` 2.5D height-delta map into integer height offsets."""
    parts = encoded.strip().split("|")
    if len(parts) < 5 or parts[0] != "SVPE1:HD25D":
        raise ValueError(f"Invalid SVPE1:HD25D header: {encoded[:48]}")
    center = tuple(int(x) for x in parts[1].split("=", 1)[1].strip("()").split(","))
    radius = int(parts[2].split("=", 1)[1])
    rows_rle = parts[4].split("/")
    grid_symbols: List[List[str]] = []
    grid_deltas: List[List[Optional[int]]] = []
    for row_s in rows_rle:
        syms = rle_decode_svpe1_tokens(row_s)
        grid_symbols.append(syms)
        deltas: List[Optional[int]] = []
        for s in syms:
            if s == "=":
                deltas.append(0)
            elif "A" <= s <= "Z":
                deltas.append(1 + (ord(s) - ord("A")))
            elif "a" <= s <= "z":
                deltas.append(-(1 + (ord(s) - ord("a"))))
            else:
                deltas.append(None)
        grid_deltas.append(deltas)
    return {
        "center": center,
        "radius": radius,
        "symbols": grid_symbols,
        "height_deltas": grid_deltas,
    }

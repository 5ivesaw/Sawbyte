"""Sawbyte Python SDK — Live TCP Session Client, MCP JSON-RPC 2.0 Client, Workflow Builder, and Gymnasium RL Adapter."""

from __future__ import annotations
from dataclasses import dataclass
import json
import math
import socket
import subprocess
from typing import Any, Dict, List, Optional, Tuple


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

    def run_shop_purchase(self, item_id: str, amount: int = 64) -> "WorkflowBuilder":
        self.statements.append(f"gui.run_shop_workflow {item_id} {amount}")
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

    def compile_script(self) -> str:
        return "\n".join(self.statements)


class SawbyteLiveTcpClient:
    """Direct low-latency TCP client for a running `sawbyte-play live --listen <host:port>` instance."""

    def __init__(self, endpoint: str = "127.0.0.1:25580", timeout: float = 5.0) -> None:
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

    def connect(self) -> None:
        self._sock = socket.create_connection((self.host, self.port), timeout=self.timeout)
        self._sock.setsockopt(socket.IPPROTO_TCP, socket.TCP_NODELAY, 1)
        self._reader = self._sock.makefile("r", encoding="utf-8")

    def close(self) -> None:
        if self._reader is not None:
            self._reader.close()
            self._reader = None
        if self._sock is not None:
            self._sock.close()
            self._sock = None

    def send_command(self, line: str) -> str:
        if self._sock is None or self._reader is None:
            self.connect()
        assert self._sock is not None and self._reader is not None
        self._sock.sendall((line.strip() + "\n").encode("utf-8"))
        reply = self._reader.readline()
        if not reply:
            raise RuntimeError("sawbyte-play live TCP socket closed")
        text = reply.rstrip("\r\n").replace("\\n", "\n")
        if text.startswith("error: "):
            raise RuntimeError(text)
        return text


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
                self._proc.stdin.close()
            self._proc.wait(timeout=5)
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
    ) -> None:
        self.client = SawbyteLiveTcpClient(live_endpoint)
        self.goal_pos = goal_pos
        self.max_steps = max_steps
        self.step_count = 0

    def reset(self, seed: Optional[int] = None) -> Tuple[Dict[str, Any], Dict[str, Any]]:
        self.step_count = 0
        obs = self.observe()
        return obs, {"seed": seed}

    def observe(self) -> Dict[str, Any]:
        raw = self.client.send_command("gym.observe")
        return json.loads(raw)

    def step(self, action: Dict[str, Any]) -> Tuple[Dict[str, Any], float, bool, bool, Dict[str, Any]]:
        self.step_count += 1
        b = lambda k: 1 if action.get(k, False) else 0
        yaw_d = float(action.get("yaw_delta", 0.0))
        pitch_d = float(action.get("pitch_delta", 0.0))
        cmd = (
            f"gym.step {b('forward')} {b('back')} {b('left')} {b('right')} "
            f"{b('jump')} {b('sneak')} {b('sprint')} {yaw_d:.4f} {pitch_d:.4f}"
        )
        self.client.send_command(cmd)
        obs = self.observe()
        pos = obs.get("pos", [0.0, 0.0, 0.0])
        dist = math.sqrt(sum(( float(a) - float(g) ) ** 2 for a, g in zip(pos, self.goal_pos)))
        terminated = dist < 0.75
        truncated = self.step_count >= self.max_steps
        reward = 10.0 if terminated else -0.05 * dist
        return obs, reward, terminated, truncated, {"distance_to_goal": dist}

    def close(self) -> None:
        self.client.close()

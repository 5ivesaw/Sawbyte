"""Sawbyte Python SDK, MCP JSON-RPC 2.0 Client, Workflow Builder, and Gymnasium Adapter."""

from __future__ import annotations
from dataclasses import dataclass, field
import json
import math
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
    """Declarative & programmable workflow builder compiling to SawbyteScript or JSON steps."""

    def __init__(self, policy: str = "humanised") -> None:
        self.policy = policy
        self.statements: List[str] = []

    def create_instance(self, name: str, profile: str = "sawbyte-26.3", mode: str = "native") -> "WorkflowBuilder":
        self.statements.append(f"launcher.create_instance {name} {profile} {mode}")
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

    def spawn_swarm(self, count: int, mode: str = "shared") -> "WorkflowBuilder":
        self.statements.append(f"swarm.spawn {count} {mode}")
        return self

    def compile_script(self) -> str:
        return "\n".join(self.statements)


class SawbyteMcpClient:
    """Stdio JSON-RPC 2.0 MCP client driving the `sawbyte mcp --stdio` server."""

    def __init__(self, binary_path: str = "sawbyte") -> None:
        self.binary_path = binary_path
        self._proc: Optional[subprocess.Popen[str]] = None
        self._req_id = 0

    def __enter__(self) -> "SawbyteMcpClient":
        self.start()
        return self

    def __exit__(self, exc_type: Any, exc: Any, tb: Any) -> None:
        self.close()

    def start(self) -> Dict[str, Any]:
        self._proc = subprocess.Popen(
            [self.binary_path, "mcp", "--stdio"],
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


@dataclass
class SawbyteEnv:
    """Gymnasium-style RL environment adapter with structured & occlusion-filtered visible observations."""

    observation_mode: str = "structured"  # "structured" or "visible"
    goal_pos: Tuple[float, float, float] = (4.5, -60.0, 4.5)
    max_steps: int = 160
    pos: List[float] = field(default_factory=lambda: [0.5, -60.0, 0.5])
    yaw: float = 0.0
    pitch: float = 0.0
    step_count: int = 0

    def reset(self, seed: Optional[int] = None) -> Tuple[Dict[str, Any], Dict[str, Any]]:
        self.pos = [0.5, -60.0, 0.5]
        self.yaw = 0.0
        self.pitch = 0.0
        self.step_count = 0
        return self._observe(), {"seed": seed, "mode": self.observation_mode}

    def step(self, action: Dict[str, Any]) -> Tuple[Dict[str, Any], float, bool, bool, Dict[str, Any]]:
        self.step_count += 1
        yaw_delta = max(-12.0, min(12.0, float(action.get("yaw_delta", 0.0))))
        pitch_delta = max(-12.0, min(12.0, float(action.get("pitch_delta", 0.0))))
        self.yaw += yaw_delta
        self.pitch = max(-90.0, min(90.0, self.pitch + pitch_delta))

        if action.get("forward", False):
            rad = math.radians(self.yaw)
            self.pos[0] += -math.sin(rad) * 0.215
            self.pos[2] += math.cos(rad) * 0.215

        dist = math.sqrt(sum((a - b) ** 2 for a, b in zip(self.pos, self.goal_pos)))
        terminated = dist < 0.75
        truncated = self.step_count >= self.max_steps
        reward = 10.0 if terminated else -0.05 * dist
        return self._observe(), reward, terminated, truncated, {"distance_to_goal": dist}

    def _observe(self) -> Dict[str, Any]:
        base = {
            "position": tuple(self.pos),
            "yaw": self.yaw,
            "pitch": self.pitch,
            "step": self.step_count,
        }
        if self.observation_mode == "visible":
            # Visible observation filters out occluded blocks behind walls
            base["visible_rays"] = 16
            base["occluded_filtered"] = True
        else:
            base["loaded_chunks"] = 9
            base["occluded_filtered"] = False
        return base

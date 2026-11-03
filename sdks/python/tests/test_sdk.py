import json
import os
from pathlib import Path
import socket
import sys
import threading
import unittest

sys.path.insert(0, str(Path(__file__).resolve().parents[1]))
from sawbyte import SawbyteEnv, SawbyteLiveTcpClient, SawbyteMcpClient, WorkflowBuilder


def _start_mock_live_server():
    srv = socket.socket(socket.AF_INET, socket.SOCK_STREAM)
    srv.bind(("127.0.0.1", 0))
    srv.listen(4)
    host, port = srv.getsockname()

    def _run():
        for _ in range(3):
            try:
                conn, _ = srv.accept()
            except OSError:
                break
            with conn, conn.makefile("r", encoding="utf-8") as reader:
                for line in reader:
                    cmd = line.strip()
                    if cmd == "gym.observe":
                        reply = json.dumps({
                            "tick": 12,
                            "pos": [1.5, -60.0, 1.5],
                            "vel": [0.0, 0.0, 0.0],
                            "yaw": 0.0,
                            "pitch": 0.0,
                            "ground": True,
                            "health": 20.0,
                            "food": 20,
                            "raycast_dist": 3.2,
                            "voxel_patch_len": 147,
                            "solid_voxels": 49,
                        })
                    elif cmd.startswith("gym.step"):
                        reply = "gym.step scheduled"
                    else:
                        reply = f"live_executed:{cmd}"
                    conn.sendall((reply + "\n").encode("utf-8"))
        srv.close()

    t = threading.Thread(target=_run, daemon=True)
    t.start()
    return f"{host}:{port}", srv


class TestSawbytePythonSdk(unittest.TestCase):
    def test_live_tcp_client_and_gym_environment(self):
        endpoint, srv = _start_mock_live_server()
        try:
            env = SawbyteEnv(live_endpoint=endpoint, goal_pos=(4.5, -60.0, 4.5))
            obs, info = env.reset(seed=42)
            self.assertEqual(info["seed"], 42)
            self.assertEqual(obs["voxel_patch_len"], 147)
            obs2, reward, terminated, truncated, step_info = env.step({"forward": True, "sprint": True, "yaw_delta": 5.0})
            self.assertIn("distance_to_goal", step_info)
            self.assertFalse(truncated)
            env.close()
        finally:
            srv.close()

    def test_workflow_builder_and_mcp_client_if_binary_available(self):
        endpoint, srv = _start_mock_live_server()
        try:
            wf = (
                WorkflowBuilder()
                .create_instance("py-inst", "sawbyte-26.3", "native")
                .goto_menu("options")
                .set_setting("fov", "92")
                .connect_live(endpoint)
                .navigate(3, -60, 3)
                .run_shop_purchase("minecraft:stone", 64)
                .observe_gym()
            )
            script = wf.compile_script()
            self.assertIn("gui.run_shop_workflow minecraft:stone 64", script)

            binary = os.environ.get("SAWBYTE_BIN")
            if binary:
                bin_path = Path(binary)
                if not bin_path.exists() and Path(binary + ".exe").exists():
                    bin_path = Path(binary + ".exe")
                if bin_path.exists():
                    with SawbyteMcpClient(str(bin_path), live_endpoint=endpoint) as client:
                        tools = client.list_tools()
                        self.assertGreaterEqual(len(tools), 8)
                        out = client.execute_workflow(wf)
                        self.assertIn("live_executed:gui.run_shop_workflow minecraft:stone 64", out)
        finally:
            srv.close()


if __name__ == "__main__":
    unittest.main()

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
    srv.listen(16)
    host, port = srv.getsockname()

    def _run():
        while True:
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
                        reply = json.dumps({"tick": 13, "pos": [1.5, -60.0, 1.5], "health": 20})
                    elif cmd == "inspect.events":
                        reply = json.dumps({"events": ["first\nsecond"]})
                    else:
                        reply = f"live_executed:{cmd}"
                    conn.sendall((reply + "\n").encode("utf-8"))
        srv.close()

    t = threading.Thread(target=_run, daemon=True)
    t.start()
    return f"{host}:{port}", srv


class TestSawbytePythonSdk(unittest.TestCase):
    def test_training_reset_requires_real_provider(self):
        env = SawbyteEnv()
        with self.assertRaisesRegex(RuntimeError, "reset provider"):
            env.reset(seed=42)

    def test_training_step_rejects_queued_response(self):
        class QueuedOnlyClient:
            def send_command(self, _command):
                return "gym.step scheduled"

        env = SawbyteEnv()
        env.client = QueuedOnlyClient()
        with self.assertRaisesRegex(RuntimeError, "did not return an observation"):
            env.step({})
        self.assertEqual(env.step_count, 0)

    def test_live_helpers_send_commands(self):
        endpoint, srv = _start_mock_live_server()
        try:
            with SawbyteLiveTcpClient(endpoint) as client:
                self.assertEqual(client.wait_ticks(3), "live_executed:wait.ticks 3")
                self.assertEqual(client.gui_click_slot(2), "live_executed:gui.click_slot 2 pickup")
                self.assertEqual(client.gui_close(), "live_executed:gui.close")
                self.assertEqual(json.loads(client.send_command("inspect.events"))["events"], ["first\nsecond"])
        finally:
            srv.close()

    def test_live_tcp_client_and_gym_environment(self):
        endpoint, srv = _start_mock_live_server()
        try:
            seeds = []
            env = SawbyteEnv(live_endpoint=endpoint, goal_pos=(4.5, -60.0, 4.5), reset_provider=seeds.append)
            obs, info = env.reset(seed=42)
            self.assertEqual(seeds, [42])
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
                .wait_until("screen_open", 100, "Chest")
                .gui_click_slot(0, "pickup")
                .wait_ticks(10)
                .cancel_task()
                .observe_gym()
            )
            script = wf.compile_script()
            self.assertIn("wait.until screen_open 100 \"Chest\"", script)
            self.assertIn("gui.click_slot 0 pickup", script)
            self.assertIn("wait.ticks 10", script)
            self.assertIn("task.cancel", script)

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
                        self.assertIn("live_executed:wait.ticks 10", out)
        finally:
            srv.close()


if __name__ == "__main__":
    unittest.main()

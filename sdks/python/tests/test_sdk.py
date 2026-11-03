import os
from pathlib import Path
import sys
import unittest

sys.path.insert(0, str(Path(__file__).resolve().parents[1]))
from sawbyte import SawbyteEnv, SawbyteMcpClient, WorkflowBuilder


class TestSawbytePythonSdk(unittest.TestCase):
    def test_gym_environment_structured_and_visible_modes(self):
        env = SawbyteEnv(observation_mode="visible", goal_pos=(0.5, -60.0, 1.0))
        obs, info = env.reset(seed=42)
        self.assertTrue(obs["occluded_filtered"])
        self.assertEqual(info["seed"], 42)
        obs2, reward, terminated, truncated, step_info = env.step({"forward": True, "yaw_delta": 0.0})
        self.assertIn("distance_to_goal", step_info)
        self.assertFalse(truncated)

    def test_workflow_builder_and_mcp_client_if_binary_available(self):
        wf = (
            WorkflowBuilder()
            .create_instance("py-inst", "sawbyte-26.3", "native")
            .goto_menu("options")
            .set_setting("fov", "92")
            .navigate(3, -60, 3)
            .run_shop_purchase("minecraft:stone", 64)
            .spawn_swarm(4, "shared")
        )
        script = wf.compile_script()
        self.assertIn("gui.run_shop_workflow minecraft:stone 64", script)

        binary = os.environ.get("SAWBYTE_BIN")
        if binary:
            bin_path = Path(binary)
            if not bin_path.exists() and Path(binary + ".exe").exists():
                bin_path = Path(binary + ".exe")
            if bin_path.exists():
                with SawbyteMcpClient(str(bin_path)) as client:
                    tools = client.list_tools()
                    self.assertGreaterEqual(len(tools), 10)
                    out = client.execute_workflow(wf)
                    self.assertIn("shop workflow completed", out)


if __name__ == "__main__":
    unittest.main()

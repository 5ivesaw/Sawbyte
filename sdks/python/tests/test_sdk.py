import json
import os
from pathlib import Path
import socket
import sys
import threading
import unittest

sys.path.insert(0, str(Path(__file__).resolve().parents[1]))
from sawbyte import SawbyteEnv, SawbyteLiveTcpClient, SawbyteMcpClient, WorkflowBuilder, decode_svpe1_hd25d, decode_svpe1_rle3d


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



    def test_svpe1_decoders_roundtrip(self):
        sample_rle = "SVPE1:RLE3D|origin=(-1,63,-1)|dims=(3,2,3)|order=YZX|legend={A:air,S:solid}|9S/4AS4A"
        vol = decode_svpe1_rle3d(sample_rle)
        self.assertEqual(vol["origin"], (-1, 63, -1))
        self.assertEqual(vol["dims"], (3, 2, 3))
        self.assertEqual(vol["layers_yzx"][0][1][1], "S")
        self.assertEqual(vol["layers_yzx"][1][1][1], "S")
        self.assertEqual(vol["layers_yzx"][1][0][0], "A")

        sample_hd = "SVPE1:HD25D|center=(0,63,0)|radius=1|legend={=:level,A..Z:up,a..z:down}|3=/a=A/3_"
        hd = decode_svpe1_hd25d(sample_hd)
        self.assertEqual(hd["center"], (0, 63, 0))
        self.assertEqual(hd["height_deltas"][0], [0, 0, 0])
        self.assertEqual(hd["height_deltas"][1], [-1, 0, 1])
        self.assertEqual(hd["height_deltas"][2], [None, None, None])



    def test_autonomous_bug_doctor_detects_and_heals_stale_files_env_macros_and_deps(self) -> None:
        import importlib.util
        import tempfile
        root_dir = Path(__file__).resolve().parents[3]
        doctor_path = root_dir / "tools" / "find_and_fix_bugs.py"
        spec = importlib.util.spec_from_file_location("find_and_fix_bugs", doctor_path)
        assert spec and spec.loader
        doctor = importlib.util.module_from_spec(spec)
        spec.loader.exec_module(doctor)

        with tempfile.TemporaryDirectory() as tmp:
            troot = Path(tmp)
            prep_dir = troot / "crates" / "sawbyte-prepare" / "src"
            prep_dir.mkdir(parents=True, exist_ok=True)
            (troot / "crates" / "sawbyte-prepare" / "Cargo.toml").write_text(
                '[package]\nname = "sawbyte-prepare"\nversion = "0.5.1"\n\n[dependencies]\nsawbyte-core = { path = "../sawbyte-core" }\n',
                encoding="utf-8",
            )
            # 1. Create a stale build.rs and a stale non-canonical file
            (troot / "crates" / "sawbyte-prepare" / "build.rs").write_text("use sha2::Sha256;\n", encoding="utf-8")
            (prep_dir / "stale_orphan.rs").write_text("use crc32fast::Hasher;\n", encoding="utf-8")
            # 2. Create lib.rs with bare env! macro and external zip + crc32fast + sha2 + regex + sha1 imports
            (prep_dir / "lib.rs").write_text(
                'use sha2::Sha256;\nuse regex::Regex;\nuse sha1::Sha1;\nuse zip::ZipArchive;\nuse crc32fast::Hasher;\n'
                'pub const H: &str = env!("HORIZON_PREPARE_SOURCES_SHA256");\n'
                'pub fn call_live(t: &mut TcpLiveSessionTransport) { let _ = t.send_live_command("see"); }\n',
                encoding="utf-8",
            )
            rep = doctor.BugReport()
            doctor.step1_clean_stale_targets_and_orphans(troot, rep)
            doctor.step2_rewrite_nonstandard_env_macros_and_let_chains(troot, rep)
            doctor.step3_scan_and_heal_external_crate_dependencies(troot, rep)

            assert not (troot / "crates" / "sawbyte-prepare" / "build.rs").exists()
            assert not (prep_dir / "stale_orphan.rs").exists()
            healed_lib = (prep_dir / "lib.rs").read_text(encoding="utf-8")
            assert 'option_env!("HORIZON_PREPARE_SOURCES_SHA256")' in healed_lib
            assert "LiveSessionTransport" in healed_lib
            healed_toml = (troot / "crates" / "sawbyte-prepare" / "Cargo.toml").read_text(encoding="utf-8")
            for dep in ("sha2 =", "regex =", "sha1 =", "zip =", "crc32fast ="):
                assert dep in healed_toml, f"Missing auto-injected {dep} in {healed_toml}"



    def test_protocol_tables_match_minecraft_data_766_to_775(self) -> None:
        import json, re
        root = Path(__file__).resolve().parents[3]
        fixture = json.loads((root / "native" / "tests" / "fixtures" / "mcdata_ids.json").read_text(encoding="utf-8"))
        release_rs = (root / "native" / "src" / "release_ids.rs").read_text(encoding="utf-8")
        inv_rs = (root / "native" / "src" / "inventory.rs").read_text(encoding="utf-8")

        # Parse release_ids.rs match arms for 766..=774
        arm_pattern = re.compile(r"([0-9\s\|]+)=>\s*\{([^}]+)\}", re.MULTILINE)
        parsed_tables = {}
        for m in arm_pattern.finditer(release_rs):
            protos = [p.strip() for p in m.group(1).split("|") if p.strip().isdigit()]
            body = m.group(2)
            tbl = {}
            for line in body.splitlines():
                fm = re.search(r"table\.([a-z0-9_]+)\s*=\s*(-?\d+);", line)
                if fm:
                    tbl[fm.group(1)] = int(fm.group(2))
            for p in protos:
                parsed_tables[p] = tbl

        # Parse 775 from inventory.rs
        m775 = re.search(r"if proto == 775 \{([^}]+)\}", inv_rs, re.MULTILINE)
        self.assertIsNotNone(m775, "Missing 775 branch in inventory.rs")
        tbl_775 = {}
        for line in m775.group(1).splitlines():
            fm = re.search(r"table\.([a-z0-9_]+)\s*=\s*(-?\d+);", line)
            if fm:
                tbl_775[fm.group(1)] = int(fm.group(2))
        parsed_tables["775"] = tbl_775

        for proto_num in range(766, 776):
            p = str(proto_num)
            self.assertIn(p, parsed_tables, f"Protocol {p} missing from Rust tables")
            rust_tbl = parsed_tables[p]
            expected_cb = fixture[p]["cb"]
            expected_sb = fixture[p]["sb"]
            for k, exp_v in expected_cb.items():
                self.assertEqual(rust_tbl.get(k), exp_v, f"Protocol {p} {k}: rust={rust_tbl.get(k)} != mcdata={exp_v}")
            for k, exp_v in expected_sb.items():
                self.assertEqual(rust_tbl.get(k), exp_v, f"Protocol {p} {k}: rust={rust_tbl.get(k)} != mcdata={exp_v}")


if __name__ == "__main__":
    unittest.main()

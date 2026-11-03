use std::io::{BufRead, BufReader, Write};
use std::net::TcpListener;
use std::process::{Command, Stdio};

fn run_with_home(args: &[&str], home: &std::path::Path) -> std::process::Output {
    Command::new(env!("CARGO_BIN_EXE_sawbyte"))
        .env("SAWBYTE_HOME", home)
        .args(args)
        .output()
        .unwrap()
}

#[test]
fn cli_reports_scope_and_rejects_unknown_commands() {
    let temp_home = std::env::temp_dir().join(format!("sawbyte-cli-cap-{}", std::process::id()));
    let output = run_with_home(&["capabilities"], &temp_home);
    assert!(output.status.success());
    let text = String::from_utf8(output.stdout).unwrap();
    assert!(text.contains("sawbyte-play live"));
    assert!(text.contains("Persistent disk launcher"));
    assert!(!run_with_home(&["connect"], &temp_home).status.success());
    assert!(!run_with_home(&["demo", "--unknown"], &temp_home)
        .status
        .success());
    let _ = std::fs::remove_dir_all(&temp_home);
}

#[test]
fn demos_exercise_different_policies_and_release_forward() {
    let temp_home = std::env::temp_dir().join(format!("sawbyte-cli-demo-{}", std::process::id()));
    let human = run_with_home(&["demo"], &temp_home);
    let direct = run_with_home(&["demo", "--direct"], &temp_home);
    assert!(human.status.success() && direct.status.success());
    let human = String::from_utf8(human.stdout).unwrap();
    let direct = String::from_utf8(direct.stdout).unwrap();
    assert!(human.contains("tick=00 forward=true yaw=12.000"));
    assert!(direct.contains("tick=00 forward=true yaw=90.000"));
    assert!(human.contains("tick=04 forward=false"));
    assert_eq!(human.lines().count(), 10);
    let _ = std::fs::remove_dir_all(&temp_home);
}

#[test]
fn persistent_disk_launcher_and_live_tcp_mcp_bridge_work_across_processes() {
    let temp_home = std::env::temp_dir().join(format!("sawbyte-cli-e2e-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&temp_home);

    // 1. Create instance in Process 1, set setting, backup snapshot
    assert!(run_with_home(
        &["launcher", "create", "rl-bot-1", "sawbyte-26.3", "native"],
        &temp_home
    )
    .status
    .success());
    assert!(run_with_home(
        &["launcher", "set-setting", "rl-bot-1", "fov", "92"],
        &temp_home
    )
    .status
    .success());
    assert!(
        run_with_home(&["launcher", "backup", "rl-bot-1"], &temp_home)
            .status
            .success()
    );

    // 2. Verify in Process 2 (`launcher list`) that disk state persisted after Process 1 exited
    let list_out = run_with_home(&["launcher", "list"], &temp_home);
    assert!(list_out.status.success());
    let list_text = String::from_utf8(list_out.stdout).unwrap();
    assert!(list_text.contains("rl-bot-1"));
    assert!(list_text.contains("snapshots=1"));

    // 3. Start a loopback TCP listener representing a live `sawbyte-play live --listen` session
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let addr = listener.local_addr().unwrap().to_string();
    let srv = std::thread::spawn(move || {
        for stream in listener.incoming().take(3).flatten() {
            let mut writer = stream.try_clone().unwrap();
            let reader = BufReader::new(stream);
            for line in reader.lines().map_while(Result::ok) {
                writeln!(writer, "live_session_ok:{line}").unwrap();
                writer.flush().unwrap();
            }
        }
    });

    // 4. Test `sawbyte eval --connect <addr>` forwarding to the live TCP session
    let eval = run_with_home(
        &[
            "eval",
            "menu.goto options\nsettings.set fov 100\ntask.navigate 3 -60 4\ngym.observe",
            "--connect",
            &addr,
        ],
        &temp_home,
    );
    assert!(eval.status.success());
    let eval_out = String::from_utf8(eval.stdout).unwrap();
    assert!(eval_out.contains("persisted setting fov=100"));
    assert!(eval_out.contains("live_session_ok:task.navigate 3 -60 4"));
    assert!(eval_out.contains("live_session_ok:gym.observe"));

    // 5. Verify MCP JSON-RPC over stdio connected to the live TCP session
    let mut child = Command::new(env!("CARGO_BIN_EXE_sawbyte"))
        .env("SAWBYTE_HOME", &temp_home)
        .args(["mcp", "--stdio", "--connect", &addr])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .unwrap();
    child
        .stdin
        .as_mut()
        .unwrap()
        .write_all(
            b"{\"jsonrpc\":\"2.0\",\"id\":1,\"method\":\"initialize\"}\n\
              {\"jsonrpc\":\"2.0\",\"id\":2,\"method\":\"tools/call\",\"params\":{\"name\":\"code_execute\",\"script\":\"gui.run_shop_workflow minecraft:stone 64\"}}\n",
        )
        .unwrap();
    let mcp_out = child.wait_with_output().unwrap();
    assert!(mcp_out.status.success());
    let mcp_text = String::from_utf8(mcp_out.stdout).unwrap();
    assert!(mcp_text.contains("sawbyte-mcp"));
    assert!(mcp_text.contains("live_session_ok:gui.run_shop_workflow minecraft:stone 64"));

    let _ = srv.join();
    let _ = std::fs::remove_dir_all(&temp_home);
}

use std::io::Write;
use std::process::{Command, Stdio};

fn run(args: &[&str]) -> std::process::Output {
    Command::new(env!("CARGO_BIN_EXE_sawbyte"))
        .args(args)
        .output()
        .unwrap()
}

#[test]
fn cli_reports_scope_and_rejects_unknown_commands() {
    let output = run(&["capabilities"]);
    assert!(output.status.success());
    let text = String::from_utf8(output.stdout).unwrap();
    assert!(text.contains("experimental sawbyte-play"));
    assert!(text.contains("No Minecraft version is certified for gameplay yet"));
    assert!(!run(&["connect"]).status.success());
    assert!(!run(&["demo", "--unknown"]).status.success());
}

#[test]
fn demos_exercise_different_policies_and_release_forward() {
    let human = run(&["demo"]);
    let direct = run(&["demo", "--direct"]);
    assert!(human.status.success() && direct.status.success());
    let human = String::from_utf8(human.stdout).unwrap();
    let direct = String::from_utf8(direct.stdout).unwrap();
    assert!(human.contains("tick=00 forward=true yaw=12.000"));
    assert!(direct.contains("tick=00 forward=true yaw=90.000"));
    assert!(human.contains("tick=04 forward=false"));
    assert_eq!(human.lines().count(), 10);
}

#[test]
fn launcher_workflow_inspect_eval_and_mcp_stdio_work_end_to_end() {
    let launcher = run(&["launcher", "list"]);
    assert!(launcher.status.success());
    let l_out = String::from_utf8(launcher.stdout).unwrap();
    assert!(l_out.contains("sawbyte-26.3"));
    assert!(l_out.contains("sawbyte-26.2"));

    let wf = run(&["workflow", "shop-demo"]);
    assert!(wf.status.success());
    assert!(String::from_utf8(wf.stdout)
        .unwrap()
        .contains("shop workflow completed"));

    let inspect = run(&["inspect"]);
    assert!(inspect.status.success());
    assert!(String::from_utf8(inspect.stdout)
        .unwrap()
        .contains("Sawbyte 2D Inspector"));

    let eval = run(&[
        "eval",
        "menu.goto options\nsettings.set fov 100\ntask.navigate 3 -60 4\nswarm.spawn 4 isolated",
    ]);
    assert!(eval.status.success());
    let eval_out = String::from_utf8(eval.stdout).unwrap();
    assert!(eval_out.contains("updated client setting fov=100"));
    assert!(eval_out.contains("spawned and stepped 4 agents"));

    // Verify MCP JSON-RPC over stdio
    let mut child = Command::new(env!("CARGO_BIN_EXE_sawbyte"))
        .args(["mcp", "--stdio"])
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
              {\"jsonrpc\":\"2.0\",\"id\":2,\"method\":\"tools/call\",\"params\":{\"name\":\"code_execute\",\"script\":\"menu.goto multiplayer\"}}\n",
        )
        .unwrap();
    let mcp_out = child.wait_with_output().unwrap();
    assert!(mcp_out.status.success());
    let mcp_text = String::from_utf8(mcp_out.stdout).unwrap();
    assert!(mcp_text.contains("sawbyte-mcp"));
    assert!(mcp_text.contains("navigated client menu -> multiplayer"));
}

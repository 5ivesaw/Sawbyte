use std::process::Command;

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

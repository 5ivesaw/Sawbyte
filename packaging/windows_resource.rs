// Shared by the CLI and native player; compilation happens on the Windows CI runner.
#[cfg(windows)]
fn embed_icon(root: &std::path::Path) {
    use std::{env, fs, process::Command};
    let icon = root.join("packaging/generated/sawbyte.ico");
    println!("cargo:rerun-if-changed={}", icon.display());
    let ok = icon.is_file();
    assert!(ok, "Generate platform icons in CI before building");
    let out = std::path::PathBuf::from(env::var_os("OUT_DIR").expect("Cargo OUT_DIR"));
    let script = out.join("sawbyte.rc");
    let resource = out.join("sawbyte.res");
    let path = icon.display().to_string().replace('\\', "/");
    let line = format!("1 ICON \"{path}\"\n");
    fs::write(&script, line).expect("write icon resource script");
    let status = Command::new("rc.exe")
        .arg("/nologo")
        .arg("/fo")
        .arg(&resource)
        .arg(&script)
        .status()
        .expect("Windows SDK resource compiler must be available in CI");
    assert!(status.success(), "icon resource compilation failed");
    println!("cargo:rustc-link-arg={}", resource.display());
}

use std::process::Command;

#[test]
fn version_flag() {
    let bin = env!("CARGO_BIN_EXE_yamlext");
    let out = Command::new(bin).arg("--version").output().unwrap();
    assert!(out.status.success(), "exit status: {}", out.status);
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(
        stdout.contains(env!("CARGO_PKG_VERSION")),
        "expected version {} in output: {stdout}",
        env!("CARGO_PKG_VERSION")
    );
}

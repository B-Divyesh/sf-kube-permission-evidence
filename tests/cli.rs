use std::path::PathBuf;
use std::process::Command;

fn fixture(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("examples").join(name)
}

#[test]
fn documented_offline_report_sign_and_verify_flow_works() {
    let temp = tempfile::tempdir().unwrap();
    let key = temp.path().join("audit.key");
    let output = temp.path().join("evidence");

    assert!(Command::new(env!("CARGO_BIN_EXE_kpe"))
        .args(["keygen", "--output"]).arg(&key).status().unwrap().success());

    let report = Command::new(env!("CARGO_BIN_EXE_kpe"))
        .args(["report", "--snapshot"]).arg(fixture("rbac-snapshot.json"))
        .args(["--subject", "user:alice@example.com", "--as-group", "platform-engineers", "--matrix"])
        .arg(fixture("matrix.json"))
        .arg("--output").arg(&output)
        .arg("--signing-key").arg(&key)
        .output().unwrap();
    assert!(report.status.success(), "{}", String::from_utf8_lossy(&report.stderr));
    assert!(output.with_extension("md").exists());
    assert!(output.with_extension("json").exists());

    let verify = Command::new(env!("CARGO_BIN_EXE_kpe"))
        .arg("verify").arg(output.with_extension("json")).output().unwrap();
    assert!(verify.status.success(), "{}", String::from_utf8_lossy(&verify.stderr));
    assert!(String::from_utf8_lossy(&verify.stdout).contains("VALID"));
}

#[test]
fn fail_on_denied_uses_documented_exit_code() {
    let output = Command::new(env!("CARGO_BIN_EXE_kpe"))
        .args(["report", "--snapshot"]).arg(fixture("rbac-snapshot.json"))
        .args(["--subject", "user:alice@example.com", "--as-group", "platform-engineers", "--matrix"])
        .arg(fixture("matrix.json"))
        .args(["--json", "--fail-on", "denied"])
        .output().unwrap();
    assert_eq!(output.status.code(), Some(3));
    let report: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(report.pointer("/summary/denied").unwrap(), 2);
}

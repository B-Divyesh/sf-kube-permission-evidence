use std::fs;
use std::path::PathBuf;
use std::process::Command;

fn fixture(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("examples")
        .join(name)
}

fn create_signed_packet(directory: &std::path::Path, stem: &str) -> (PathBuf, PathBuf) {
    let key = directory.join(format!("{stem}.key"));
    let public_key = directory.join(format!("{stem}.pub"));
    let output = directory.join(stem);
    let keygen = Command::new(env!("CARGO_BIN_EXE_kpe"))
        .args(["keygen", "--output"])
        .arg(&key)
        .arg("--public-key-output")
        .arg(&public_key)
        .output()
        .unwrap();
    assert!(
        keygen.status.success(),
        "{}",
        String::from_utf8_lossy(&keygen.stderr)
    );
    let report = Command::new(env!("CARGO_BIN_EXE_kpe"))
        .args(["report", "--snapshot"])
        .arg(fixture("rbac-snapshot.json"))
        .args([
            "--subject",
            "user:alice@example.com",
            "--as-group",
            "platform-engineers",
            "--matrix",
        ])
        .arg(fixture("matrix.json"))
        .arg("--output")
        .arg(&output)
        .arg("--signing-key")
        .arg(&key)
        .output()
        .unwrap();
    assert!(
        report.status.success(),
        "{}",
        String::from_utf8_lossy(&report.stderr)
    );
    (output.with_extension("json"), public_key)
}

#[test]
fn documented_offline_report_sign_and_verify_flow_works() {
    let temp = tempfile::tempdir().unwrap();
    let key = temp.path().join("audit.key");
    let output = temp.path().join("evidence");

    assert!(
        Command::new(env!("CARGO_BIN_EXE_kpe"))
            .args(["keygen", "--output"])
            .arg(&key)
            .status()
            .unwrap()
            .success()
    );

    let report = Command::new(env!("CARGO_BIN_EXE_kpe"))
        .args(["report", "--snapshot"])
        .arg(fixture("rbac-snapshot.json"))
        .args([
            "--subject",
            "user:alice@example.com",
            "--as-group",
            "platform-engineers",
            "--matrix",
        ])
        .arg(fixture("matrix.json"))
        .arg("--output")
        .arg(&output)
        .arg("--signing-key")
        .arg(&key)
        .output()
        .unwrap();
    assert!(
        report.status.success(),
        "{}",
        String::from_utf8_lossy(&report.stderr)
    );
    assert!(output.with_extension("md").exists());
    assert!(output.with_extension("json").exists());

    let verify = Command::new(env!("CARGO_BIN_EXE_kpe"))
        .arg("verify")
        .arg(output.with_extension("json"))
        .output()
        .unwrap();
    assert!(
        verify.status.success(),
        "{}",
        String::from_utf8_lossy(&verify.stderr)
    );
    assert!(String::from_utf8_lossy(&verify.stdout).contains("VALID"));
}

#[test]
fn fail_on_denied_uses_documented_exit_code() {
    let output = Command::new(env!("CARGO_BIN_EXE_kpe"))
        .args(["report", "--snapshot"])
        .arg(fixture("rbac-snapshot.json"))
        .args([
            "--subject",
            "user:alice@example.com",
            "--as-group",
            "platform-engineers",
            "--matrix",
        ])
        .arg(fixture("matrix.json"))
        .args(["--json", "--fail-on", "denied"])
        .output()
        .unwrap();
    assert_eq!(output.status.code(), Some(3));
    let report: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(report.pointer("/summary/denied").unwrap(), 2);
}

#[test]
fn release_binary_denies_named_top_level_create() {
    let temp = tempfile::tempdir().unwrap();
    let snapshot = temp.path().join("snapshot.json");
    let matrix = temp.path().join("matrix.json");
    fs::write(
        &snapshot,
        r#"{
          "schemaVersion":"kpe.snapshot/v1","collectedAt":"2026-08-28T00:00:00Z",
          "collectorVersion":"test","context":"regression","serverVersion":"v1.33.4",
          "roles":[],"roleBindings":[],
          "clusterRoles":[{"kind":"ClusterRole","metadata":{"name":"named-create"},
            "rules":[{"apiGroups":[""],"resources":["pods"],"verbs":["create"],"resourceNames":["approved"]}]}],
          "clusterRoleBindings":[{"kind":"ClusterRoleBinding","metadata":{"name":"named-create"},
            "subjects":[{"kind":"User","name":"alice"}],
            "roleRef":{"kind":"ClusterRole","name":"named-create","apiGroup":"rbac.authorization.k8s.io"}}]
        }"#,
    )
    .unwrap();
    fs::write(
        &matrix,
        r#"{"checks":[{"verb":"create","resource":"pods","namespace":"payments","resourceName":"approved"}]}"#,
    )
    .unwrap();

    let output = Command::new(env!("CARGO_BIN_EXE_kpe"))
        .args(["report", "--snapshot"])
        .arg(snapshot)
        .args(["--subject", "user:alice", "--matrix"])
        .arg(matrix)
        .arg("--json")
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let report: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(report.pointer("/checks/0/allowed"), Some(&false.into()));
    assert_eq!(report.pointer("/checks/0/grants/0"), None);
    assert_eq!(report.pointer("/summary/denied"), Some(&1.into()));
}

#[test]
fn release_binary_allows_kubernetes_wildcard_subresource_rule() {
    let temp = tempfile::tempdir().unwrap();
    let snapshot = temp.path().join("snapshot.json");
    let matrix = temp.path().join("matrix.json");
    fs::write(
        &snapshot,
        r#"{
          "schemaVersion":"kpe.snapshot/v1","collectedAt":"2026-08-28T00:00:00Z",
          "collectorVersion":"test","context":"regression","serverVersion":"v1.33.4",
          "roles":[],"roleBindings":[],
          "clusterRoles":[{"kind":"ClusterRole","metadata":{"name":"scale-any-resource"},
            "rules":[{"apiGroups":["apps"],"resources":["*/scale"],"verbs":["update"]}]}],
          "clusterRoleBindings":[{"kind":"ClusterRoleBinding","metadata":{"name":"alice-scale"},
            "subjects":[{"kind":"User","name":"alice"}],
            "roleRef":{"kind":"ClusterRole","name":"scale-any-resource","apiGroup":"rbac.authorization.k8s.io"}}]
        }"#,
    )
    .unwrap();
    fs::write(
        &matrix,
        r#"{"checks":[
          {"verb":"update","apiGroup":"apps","resource":"deployments","subresource":"scale","namespace":"payments"},
          {"verb":"update","apiGroup":"apps","resource":"deployments","subresource":"status","namespace":"payments"}
        ]}"#,
    )
    .unwrap();

    let output = Command::new(env!("CARGO_BIN_EXE_kpe"))
        .args(["report", "--snapshot"])
        .arg(snapshot)
        .args(["--subject", "user:alice", "--matrix"])
        .arg(matrix)
        .arg("--json")
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let report: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(report.pointer("/checks/0/allowed"), Some(&true.into()));
    assert_eq!(
        report.pointer("/checks/0/grants/0/rule/resources/0"),
        Some(&"*/scale".into())
    );
    assert_eq!(report.pointer("/checks/1/allowed"), Some(&false.into()));
    assert_eq!(report.pointer("/summary/allowed"), Some(&1.into()));
    assert_eq!(report.pointer("/summary/denied"), Some(&1.into()));
}

#[test]
fn signed_packet_rejects_unknown_fields_at_every_signed_boundary() {
    let temp = tempfile::tempdir().unwrap();
    let (packet, _) = create_signed_packet(temp.path(), "evidence");
    let original: serde_json::Value = serde_json::from_slice(&fs::read(&packet).unwrap()).unwrap();

    let mutations = [
        ("report", "/"),
        ("check", "/checks/0"),
        ("grant", "/checks/0/grants/0"),
        ("signature", "/signature"),
    ];
    for (name, pointer) in mutations {
        let mut changed = original.clone();
        let object = if pointer == "/" {
            changed.as_object_mut().unwrap()
        } else {
            changed
                .pointer_mut(pointer)
                .unwrap()
                .as_object_mut()
                .unwrap()
        };
        object.insert("displayVerdict".into(), "ALLOWED".into());
        let path = temp.path().join(format!("unknown-{name}.json"));
        fs::write(&path, serde_json::to_vec_pretty(&changed).unwrap()).unwrap();
        let verify = Command::new(env!("CARGO_BIN_EXE_kpe"))
            .arg("verify")
            .arg(path)
            .output()
            .unwrap();
        assert!(!verify.status.success(), "accepted unknown {name} field");
        assert!(
            String::from_utf8_lossy(&verify.stderr).contains("unknown field"),
            "{}",
            String::from_utf8_lossy(&verify.stderr)
        );
    }
}

#[test]
fn trusted_signer_must_match_the_packet_key_or_fingerprint() {
    let temp = tempfile::tempdir().unwrap();
    let (packet, trusted_public_key) = create_signed_packet(temp.path(), "trusted");
    let (_, other_public_key) = create_signed_packet(temp.path(), "other");

    let valid = Command::new(env!("CARGO_BIN_EXE_kpe"))
        .arg("verify")
        .arg(&packet)
        .arg("--trusted-public-key")
        .arg(&trusted_public_key)
        .arg("--json")
        .output()
        .unwrap();
    assert!(valid.status.success());
    let verified: serde_json::Value = serde_json::from_slice(&valid.stdout).unwrap();
    let fingerprint = verified["signerFingerprint"].as_str().unwrap();
    assert!(fingerprint.starts_with("SHA256:"));

    let wrong_key = Command::new(env!("CARGO_BIN_EXE_kpe"))
        .arg("verify")
        .arg(&packet)
        .arg("--trusted-public-key")
        .arg(&other_public_key)
        .output()
        .unwrap();
    assert!(!wrong_key.status.success());
    assert!(String::from_utf8_lossy(&wrong_key.stderr).contains("does not match"));

    let valid_fingerprint = Command::new(env!("CARGO_BIN_EXE_kpe"))
        .arg("verify")
        .arg(&packet)
        .arg("--trusted-fingerprint")
        .arg(fingerprint)
        .output()
        .unwrap();
    assert!(valid_fingerprint.status.success());

    let wrong_fingerprint = Command::new(env!("CARGO_BIN_EXE_kpe"))
        .arg("verify")
        .arg(&packet)
        .arg("--trusted-fingerprint")
        .arg(format!("SHA256:{}", "0".repeat(64)))
        .output()
        .unwrap();
    assert!(!wrong_fingerprint.status.success());
    assert!(String::from_utf8_lossy(&wrong_fingerprint.stderr).contains("mismatch"));
}

#[test]
fn misspelled_matrix_fields_fail_before_evaluation() {
    let temp = tempfile::tempdir().unwrap();
    let matrix = temp.path().join("typo-matrix.json");
    fs::write(
        &matrix,
        r#"{"checks":[{"verb":"get","apiGruop":"apps","resource":"deployments","namespace":"payments"}]}"#,
    )
    .unwrap();
    let result = Command::new(env!("CARGO_BIN_EXE_kpe"))
        .args(["report", "--snapshot"])
        .arg(fixture("rbac-snapshot.json"))
        .args(["--subject", "user:alice@example.com", "--matrix"])
        .arg(matrix)
        .arg("--json")
        .output()
        .unwrap();
    assert!(!result.status.success());
    assert!(String::from_utf8_lossy(&result.stderr).contains("unknown field `apiGruop`"));
    assert!(result.stdout.is_empty());
}

#[test]
fn demo_runs_bundled_sample_without_cluster_setup() {
    let temp = tempfile::tempdir().unwrap();
    let output_directory = temp.path().join("demo-output");
    let result = Command::new(env!("CARGO_BIN_EXE_kpe"))
        .arg("demo")
        .arg("--output")
        .arg(&output_directory)
        .env("PATH", "")
        .output()
        .unwrap();
    assert!(
        result.status.success(),
        "{}",
        String::from_utf8_lossy(&result.stderr)
    );
    let stdout = String::from_utf8_lossy(&result.stdout);
    assert!(stdout.contains("no cluster was contacted"));
    assert!(stdout.contains("4 checks — 2 allowed, 2 denied, 0 uncertain"));
    let evidence: serde_json::Value =
        serde_json::from_slice(&fs::read(output_directory.join("evidence.json")).unwrap()).unwrap();
    assert_eq!(evidence.pointer("/summary/total"), Some(&4.into()));
    assert_eq!(
        evidence.pointer("/checks/0/grants/0/bindingName"),
        Some(&"alice-secrets".into())
    );
    assert!(output_directory.join("evidence.md").exists());
    assert!(output_directory.join("matrix.json").exists());
    assert!(output_directory.join("rbac-snapshot.json").exists());
}

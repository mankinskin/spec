use std::process::Command;

use spec_api::SpecStore;
use tempfile::tempdir;

#[test]
fn init_supports_toon_output() {
    let dir = tempdir().expect("temp dir");
    let index_root = dir.path().join(".workflow-tools").join("spec");

    let out = Command::new(env!("CARGO_BIN_EXE_spec"))
        .arg("--toon")
        .arg("--index-root")
        .arg(&index_root)
        .arg("init")
        .output()
        .expect("spec binary should spawn");

    assert!(
        out.status.success(),
        "spec --toon init failed ({})\nstdout: {}\nstderr: {}",
        out.status,
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr),
    );

    let rendered = String::from_utf8(out.stdout).expect("toon output should be utf-8");
    let parsed: serde_json::Value =
        toon_format::decode_default(&rendered).expect("toon output should decode");

    assert_eq!(parsed["command"], "init");
    assert_eq!(parsed["status"], "ok");
    assert_eq!(parsed["message"], "workspace initialized");
}

#[test]
fn create_with_dot_workspace_reads_back_from_canonical_store() {
    let dir = tempdir().expect("temp dir");
    let workspace = dir.path().join("workspace");
    std::fs::create_dir_all(&workspace).expect("create workspace");
    let title = "Dot-selected CLI spec";
    let output = Command::new(env!("CARGO_BIN_EXE_spec"))
        .current_dir(&workspace)
        .args([
            "--json",
            "--workspace",
            ".",
            "create",
            "--title",
            title,
            "--slug",
            "selector/dot-cli",
            "--component",
            "selector",
        ])
        .output()
        .expect("run Spec CLI create");
    assert!(
        output.status.success(),
        "Spec CLI create failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let response: serde_json::Value =
        serde_json::from_slice(&output.stdout).expect("parse Spec CLI output");
    let id = response["id"]
        .as_str()
        .expect("create response includes spec id");
    let store_root = workspace.join(".workflow-tools").join("spec");
    let store = SpecStore::open(&store_root).expect("open canonical Spec store");
    let spec = store.get(id).expect("read created spec");

    assert_eq!(
        spec.extra.get("title").and_then(|value| value.as_str()),
        Some(title)
    );
    assert_eq!(store.entity_store().index_root, store_root,);
    assert!(!workspace.join(".spec").exists());
    assert!(!dir.path().join(".workflow-tools").join("spec").exists());
}

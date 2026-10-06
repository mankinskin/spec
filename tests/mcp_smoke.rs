//! Smoke tests for spec-mcp tools.
//!
//! Exercises the key tool cycle (create → get → update → list → search →
//! sections → tree → health → refs_validate → delete) via `SpecServer`
//! methods directly, without going through the JSON-RPC transport.

use std::{
    collections::BTreeMap,
    process::Command,
};

use rmcp::handler::server::wrapper::Parameters;
use spec_api::{SpecManifest, SpecStore};
use spec::mcp::server::*;

#[path = "mcp_smoke/support.rs"]
mod support;

use support::{
    extract_json,
    make_sandbox,
};

fn run_git(
    repo_root: &std::path::Path,
    args: &[&str],
) {
    let status = Command::new("git")
        .current_dir(repo_root)
        .args(args)
        .status()
        .expect("git command");
    assert!(status.success(), "git {args:?} failed: {status}");
}

// ── tests ────────────────────────────────────────────────────────────────────

#[tokio::test]
async fn explicit_dot_workspace_creates_and_reads_back_in_current_workspace() {
    const CHILD_ENV: &str = "SPEC_DOT_SELECTOR_TEST_CHILD";
    if std::env::var_os(CHILD_ENV).is_none() {
        let tmp = tempfile::tempdir().unwrap();
        let selected = tmp.path().join("selected");
        std::fs::create_dir_all(&selected).unwrap();
        let output = Command::new(std::env::current_exe().unwrap())
            .args([
                "--exact",
                "explicit_dot_workspace_creates_and_reads_back_in_current_workspace",
                "--nocapture",
            ])
            .env(CHILD_ENV, "1")
            .current_dir(&selected)
            .output()
            .unwrap();
        assert!(
            output.status.success(),
            "child regression failed: {} {}",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        );
        assert!(!tmp.path().join(".workflow-tools").exists());
        return;
    }

    let selected = std::env::current_dir().unwrap();
    let sibling = selected.parent().unwrap().join("ambient");
    std::fs::create_dir_all(&sibling).unwrap();
    let server = SpecServer::new(sibling.clone());
    let result = server
        .spec_create(Parameters(CreateSpecInput {
            workspace: ".".to_string(),
            title: "Dot selector read-back".to_string(),
            slug: "selector/dot".to_string(),
            component: "selector".to_string(),
            parent: None,
            scope: None,
            body: Some("# Selected workspace\n".to_string()),
            fields: BTreeMap::new(),
        }))
        .await
        .unwrap();
    let result = extract_json(result);
    let id = result["id"].as_str().unwrap();
    let store_root = memory_kernel::workspace::canonical_store_root(&selected, ".spec");
    assert!(store_root.join("specs").join(id).join("spec.toml").is_file());
    let store = SpecStore::open_or_init(&store_root).unwrap();
    let (manifest, body) = store.get_full(id).unwrap();
    assert_eq!(manifest.title(), Some("Dot selector read-back"));
    assert_eq!(body, "# Selected workspace\n");
    assert_eq!(result["scope"]["requested_workspace"], selected.to_string_lossy().as_ref());
    assert!(!sibling.join(".workflow-tools").exists());
}

#[tokio::test]
async fn move_targets_reject_ambient_workspace_aliases_before_store_access() {
    let temp = tempfile::tempdir().unwrap();
    let missing_store = temp.path().join("not-created-store");
    let server = SpecServer::new(missing_store.clone());

    for selector in ["", "  ", "default", ".."] {
        let input = SpecMoveInput {
            workspace: None,
            id: Some("spec-does-not-exist".to_string()),
            ids: None,
            to_workspace_root: selector.to_string(),
        };
        assert!(server.spec_move_preflight(Parameters(input)).await.is_err());

        let input = SpecMoveInput {
            workspace: None,
            id: Some("spec-does-not-exist".to_string()),
            ids: None,
            to_workspace_root: selector.to_string(),
        };
        assert!(server.spec_move_apply(Parameters(input)).await.is_err());
        assert!(!missing_store.exists());
    }
}

#[tokio::test]
async fn spec_mutations_reject_ambient_workspace_selectors_before_init() {
    let temp = tempfile::tempdir().unwrap();
    let store_root = temp.path().join(".workflow-tools/spec");
    let server = SpecServer::new(store_root.clone());

    for workspace in [None, Some(""), Some("  "), Some("default"), Some("..")] {
        let update = server
            .spec_update(Parameters(UpdateSpecInput {
                workspace: workspace.map(str::to_string),
                id: "missing-spec".to_string(),
                fields: Some(vec!["title=blocked".to_string()]),
                to_state: None,
                body: None,
                force_body: false,
                field_map: None,
            }))
            .await;
        assert!(update.is_err());

        let scan = server
            .spec_scan(Parameters(ScanInput {
                workspace: workspace.map(str::to_string),
                force: false,
            }))
            .await;
        assert!(scan.is_err());
        assert!(!store_root.exists());
    }
}

#[tokio::test]
async fn explicit_child_workspace_does_not_use_parent_spec_store() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let parent = tmp.path().join("parent-workspace");
    let child = parent.join("child-workspace");
    std::fs::create_dir_all(&child).expect("create child workspace");
    let server = SpecServer::new(parent.clone());

    let parent_store_root =
        memory_kernel::workspace::canonical_store_root(&parent, ".spec");
    let mut parent_store =
        SpecStore::open_or_init(&parent_store_root).expect("open parent store");
    let first_id = parent_store
        .create(
            &SpecManifest::new("parent/only-spec", "Parent Spec", "parent"),
            "parent body",
            None,
        )
        .expect("seed parent store");
    let second_id = parent_store
        .create(
            &SpecManifest::new("parent/second-spec", "Second Parent Spec", "parent"),
            "second parent body",
            None,
        )
        .expect("seed second parent spec");
    drop(parent_store);

    let second_manifest_path = parent_store_root
        .join("specs")
        .join(second_id.to_string())
        .join("spec.toml");
    let second_manifest =
        std::fs::read_to_string(&second_manifest_path).expect("read second manifest");
    let duplicate_manifest = second_manifest.replace(
        "parent/second-spec",
        "parent/only-spec",
    );
    assert_ne!(second_manifest, duplicate_manifest);
    std::fs::write(&second_manifest_path, duplicate_manifest)
        .expect("write duplicate slug manifest");

    let listed = server
        .spec_list(Parameters(ListSpecsInput {
            workspace: Some(child.display().to_string()),
            where_clauses: vec![],
            limit: None,
        }))
        .await
        .expect("list child specs");
    let listed = extract_json(listed);

    assert_eq!(listed["count"], 0);
    assert!(parent_store_root.join("specs").join(first_id.to_string()).is_dir());
    assert!(child.join(".workflow-tools/spec/entities.db").is_file());
    assert_eq!(
        listed["scope"]["requested_workspace"],
        child.display().to_string()
    );
    assert_eq!(
        listed["scope"]["active_index_root"],
        memory_kernel::workspace::canonical_store_root(&child, ".spec")
            .to_string_lossy()
            .replace('\\', "/")
    );
    assert_eq!(listed["scope"]["store_initialized"], true);
    assert_eq!(listed["scope"]["ancestor_lookup_used"], false);

    let health = server
        .spec_health(Parameters(HealthInput {
            workspace: Some(child.display().to_string()),
            id: None,
            all: true,
        }))
        .await
        .expect("check child workspace health");
    let health = extract_json(health);

    assert_eq!(health["status"], "ok");
    assert_eq!(health["specs_checked"], 0);
    assert_eq!(health["scope"]["store_initialized"], false);
}

#[tokio::test]
async fn spec_health_errors_include_workspace_resolution_context() {
    let (_tmp, server) = make_sandbox();

    let error = server
        .spec_health(Parameters(HealthInput {
            workspace: None,
            id: None,
            all: false,
        }))
        .await
        .expect_err("health requires a spec ID or all=true");
    let context = error.data.expect("workspace resolution context");

    assert_eq!(context["requested_workspace"], "default");
    assert_eq!(context["store_existed_before_call"], true);
    assert_eq!(context["store_initialized"], false);
    assert_eq!(context["ancestor_lookup_used"], false);
}

#[tokio::test]
async fn explicit_spec_store_path_remains_an_override() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let workspace = tmp.path().join("workspace");
    let store_root = memory_kernel::workspace::canonical_store_root(
        &workspace,
        ".spec",
    );
    let mut store = SpecStore::open_or_init(&store_root).expect("open store");
    store
        .create(
            &SpecManifest::new("explicit/store", "Explicit Store", "test"),
            "body",
            None,
        )
        .expect("seed explicit store");
    drop(store);

    let server = SpecServer::new(tmp.path().to_path_buf());
    let listed = server
        .spec_list(Parameters(ListSpecsInput {
            workspace: Some(store_root.display().to_string()),
            where_clauses: vec![],
            limit: None,
        }))
        .await
        .expect("list explicit store");
    let listed = extract_json(listed);

    assert_eq!(listed["count"], 1);
    assert_eq!(
        listed["scope"]["selection_reason"],
        "explicit_store_path"
    );
    assert_eq!(
        listed["scope"]["active_index_root"],
        store_root.to_string_lossy().replace('\\', "/")
    );
}

#[tokio::test]
async fn invalid_workspace_error_names_expected_store_path() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let requested_workspace = tmp.path().join("missing-workspace");
    let expected_store = memory_kernel::workspace::canonical_store_root(
        &requested_workspace,
        ".spec",
    );
    let server = SpecServer::new(tmp.path().to_path_buf());

    let error = server
        .spec_list(Parameters(ListSpecsInput {
            workspace: Some(requested_workspace.display().to_string()),
            where_clauses: vec![],
            limit: None,
        }))
        .await
        .expect_err("missing workspace must be rejected");
    let error_data = error.data.expect("workspace diagnostic data");

    assert!(error.message.contains("expected store location"));
    assert_eq!(
        error_data["requested_workspace"],
        requested_workspace.display().to_string()
    );
    assert_eq!(
        error_data["expected_store_location"],
        expected_store.to_string_lossy().to_string()
    );
}

/// Full CRUD lifecycle: create → get → get(full) → update → list → delete.
#[tokio::test]
async fn spec_crud_lifecycle() {
    let (_tmp, server) = make_sandbox();

    // 1. Create a spec
    let result = server
        .spec_create(Parameters(CreateSpecInput {
            workspace: _tmp.path().display().to_string(),
            title: "Test Spec".to_string(),
            slug: "test-component/test-spec".to_string(),
            component: "test-component".to_string(),
            parent: None,
            scope: Some("public".to_string()),
            body: Some("# Test\n\nThis is a test spec.".to_string()),
            fields: BTreeMap::new(),
        }))
        .await
        .expect("spec_create");
    let json = extract_json(result);
    assert_eq!(json["status"], "ok");
    assert_eq!(json["slug"], "test-component/test-spec");
    let spec_id = json["id"].as_str().expect("id").to_string();

    // 2. Get by slug
    let result = server
        .spec_get(Parameters(GetSpecInput {
            workspace: None,
            id: "test-component/test-spec".to_string(),
            full: false,
        }))
        .await
        .expect("spec_get");
    let json = extract_json(result);
    assert_eq!(json["status"], "ok");
    assert_eq!(json["spec"]["id"], spec_id);

    // 3. Get full (with body)
    let result = server
        .spec_get(Parameters(GetSpecInput {
            workspace: None,
            id: spec_id.clone(),
            full: true,
        }))
        .await
        .expect("spec_get full");
    let json = extract_json(result);
    assert_eq!(json["status"], "ok");
    assert!(json["body"].as_str().unwrap().contains("# Test"));
    assert!(json["sections"].is_array());

    // 4. Update fields
    let result = server
        .spec_update(Parameters(UpdateSpecInput {
            workspace: Some(_tmp.path().display().to_string()),
            id: spec_id.clone(),
            fields: Some(vec!["title=Updated Title".to_string()]),
            to_state: Some("reviewed".to_string()),
            body: None,
            force_body: false,
            field_map: None,
        }))
        .await
        .expect("spec_update");
    let json = extract_json(result);
    assert_eq!(json["status"], "ok");
    assert_eq!(json["changed_fields"]["title"], "Updated Title");
    assert!(json.get("fields").is_none());

    // 5. Update body
    let result = server
        .spec_update(Parameters(UpdateSpecInput {
            workspace: Some(_tmp.path().display().to_string()),
            id: spec_id.clone(),
            fields: None,
            to_state: None,
            body: Some("# Updated body".to_string()),
            force_body: false,
            field_map: None,
        }))
        .await
        .expect("spec_update body");
    let json = extract_json(result);
    assert_eq!(json["status"], "ok");
    assert_eq!(json["body_updated"], true);

    // 6. List all
    let result = server
        .spec_list(Parameters(ListSpecsInput {
            workspace: None,
            where_clauses: vec![],
            limit: None,
        }))
        .await
        .expect("spec_list");
    let json = extract_json(result);
    assert_eq!(json["status"], "ok");
    assert_eq!(json["count"], 1);

    // 7. List with filter
    let result = server
        .spec_list(Parameters(ListSpecsInput {
            workspace: None,
            where_clauses: vec!["component=test-component".to_string()],
            limit: None,
        }))
        .await
        .expect("spec_list filtered");
    let json = extract_json(result);
    assert_eq!(json["count"], 1);

    // 8. Delete
    let result = server
        .spec_delete(Parameters(SpecRefInput {
            workspace: Some(_tmp.path().display().to_string()),
            id: spec_id.clone(),
        }))
        .await
        .expect("spec_delete");
    let json = extract_json(result);
    assert_eq!(json["status"], "ok");
}

#[tokio::test]
async fn spec_update_accepts_sparse_payload_and_returns_minimal_response() {
    let (_tmp, server) = make_sandbox();

    let created = extract_json(
        server
            .spec_create(Parameters(CreateSpecInput {
                workspace: _tmp.path().display().to_string(),
                title: "Sparse Update".to_string(),
                slug: "tests/sparse-update".to_string(),
                component: "tests".to_string(),
                parent: None,
                scope: None,
                body: None,
                fields: BTreeMap::new(),
            }))
            .await
            .expect("create"),
    );
    let spec_id = created["id"].as_str().unwrap().to_string();

    let result = server
        .spec_update(Parameters(UpdateSpecInput {
            workspace: Some(_tmp.path().display().to_string()),
            id: spec_id,
            fields: None,
            to_state: Some("reviewed".to_string()),
            body: None,
            force_body: false,
            field_map: None,
        }))
        .await
        .expect("sparse state-only update");
    let json = extract_json(result);

    assert_eq!(json["status"], "ok");
    assert_eq!(json["state_transition"]["to"], "reviewed");
    assert!(json.get("changed_fields").is_none());
    assert!(json.get("fields").is_none());
}

#[tokio::test]
async fn spec_move_preflight_returns_supported_plan() {
    let (_tmp, server) = make_sandbox();
    run_git(_tmp.path(), &["init"]);
    let target_workspace = _tmp.path().join("target");
    std::fs::create_dir_all(&target_workspace).unwrap();
    spec_api::SpecStore::init(&target_workspace).unwrap();

    let created = extract_json(
        server
            .spec_create(Parameters(CreateSpecInput {
                workspace: _tmp.path().display().to_string(),
                title: "Movable Spec".to_string(),
                slug: "tests/movable-spec".to_string(),
                component: "tests".to_string(),
                parent: None,
                scope: None,
                body: Some("body".to_string()),
                fields: BTreeMap::new(),
            }))
            .await
            .expect("create"),
    );
    let spec_id = created["id"].as_str().unwrap().to_string();

    let result = server
        .spec_move_preflight(Parameters(SpecMoveInput {
            workspace: Some(_tmp.path().display().to_string()),
            id: Some(spec_id),
            ids: None,
            to_workspace_root: target_workspace.display().to_string(),
        }))
        .await
        .expect("move preflight");
    let json = extract_json(result);

    assert_eq!(json["status"], "ok");
    assert_eq!(json["mode"], "preflight");
    assert_eq!(json["supported"], true);
}

/// Section operations: add → list → get → delete.
#[tokio::test]
async fn spec_section_lifecycle() {
    let (_tmp, server) = make_sandbox();

    // Create a spec first
    let result = server
        .spec_create(Parameters(CreateSpecInput {
            workspace: _tmp.path().display().to_string(),
            title: "Section Test".to_string(),
            slug: "sections/test".to_string(),
            component: "sections".to_string(),
            parent: None,
            scope: None,
            body: None,
            fields: BTreeMap::new(),
        }))
        .await
        .expect("create");
    let json = extract_json(result);
    let spec_id = json["id"].as_str().unwrap().to_string();

    // Add section
    let result = server
        .spec_section_add(Parameters(SectionAddInput {
            workspace: Some(_tmp.path().display().to_string()),
            id: spec_id.clone(),
            name: "design".to_string(),
            content: "## Design\n\nKey design notes.".to_string(),
        }))
        .await
        .expect("section_add");
    let json = extract_json(result);
    assert_eq!(json["status"], "ok");

    // List sections
    let result = server
        .spec_section_list(Parameters(SpecRefInput {
            workspace: None,
            id: spec_id.clone(),
        }))
        .await
        .expect("section_list");
    let json = extract_json(result);
    assert_eq!(json["count"], 1);
    let sections = json["sections"].as_array().unwrap();
    assert!(
        sections
            .iter()
            .any(|s| s.as_str().unwrap().contains("design"))
    );

    // Get section
    let result = server
        .spec_section_get(Parameters(SectionRefInput {
            workspace: None,
            id: spec_id.clone(),
            name: "design".to_string(),
        }))
        .await
        .expect("section_get");
    let json = extract_json(result);
    assert_eq!(json["status"], "ok");
    assert!(
        json["content"]
            .as_str()
            .unwrap()
            .contains("Key design notes")
    );

    // Delete section
    let result = server
        .spec_section_delete(Parameters(SectionRefInput {
            workspace: Some(_tmp.path().display().to_string()),
            id: spec_id.clone(),
            name: "design".to_string(),
        }))
        .await
        .expect("section_delete");
    let json = extract_json(result);
    assert_eq!(json["status"], "ok");

    // List again — empty
    let result = server
        .spec_section_list(Parameters(SpecRefInput {
            workspace: None,
            id: spec_id.clone(),
        }))
        .await
        .expect("section_list empty");
    let json = extract_json(result);
    assert_eq!(json["count"], 0);
}

/// Tree and health tools.
#[tokio::test]
async fn spec_tree_and_health() {
    let (_tmp, server) = make_sandbox();

    // Create parent
    let result = server
        .spec_create(Parameters(CreateSpecInput {
            workspace: _tmp.path().display().to_string(),
            title: "Parent Spec".to_string(),
            slug: "tree/parent".to_string(),
            component: "tree".to_string(),
            parent: None,
            scope: None,
            body: None,
            fields: BTreeMap::new(),
        }))
        .await
        .expect("create parent");
    let parent_id = extract_json(result)["id"].as_str().unwrap().to_string();

    // Create child
    let result = server
        .spec_create(Parameters(CreateSpecInput {
            workspace: _tmp.path().display().to_string(),
            title: "Child Spec".to_string(),
            slug: "tree/parent/child".to_string(),
            component: "tree".to_string(),
            parent: Some(parent_id.clone()),
            scope: None,
            body: None,
            fields: BTreeMap::new(),
        }))
        .await
        .expect("create child");
    let _child_id = extract_json(result)["id"].as_str().unwrap().to_string();

    // Tree from parent
    let result = server
        .spec_tree(Parameters(TreeInput {
            workspace: None,
            id: Some(parent_id.clone()),
        }))
        .await
        .expect("spec_tree");
    let json = extract_json(result);
    assert_eq!(json["status"], "ok");
    assert_eq!(json["root"]["slug"], "tree/parent");

    // Tree all roots
    let result = server
        .spec_tree(Parameters(TreeInput {
            workspace: None,
            id: None,
        }))
        .await
        .expect("spec_tree roots");
    let json = extract_json(result);
    assert_eq!(json["status"], "ok");
    assert!(!json["roots"].as_array().unwrap().is_empty());

    // Health all
    let result = server
        .spec_health(Parameters(HealthInput {
            workspace: None,
            id: None,
            all: true,
        }))
        .await
        .expect("spec_health");
    let json = extract_json(result);
    assert_eq!(json["status"], "ok");
    assert!(json["specs_checked"].as_u64().unwrap() >= 2);

    // Health single
    let result = server
        .spec_health(Parameters(HealthInput {
            workspace: None,
            id: Some(parent_id.clone()),
            all: false,
        }))
        .await
        .expect("spec_health single");
    let json = extract_json(result);
    assert_eq!(json["specs_checked"], 1);
}

/// Scan and add-root tools.
#[tokio::test]
async fn spec_scan_and_add_root() {
    let (tmp, server) = make_sandbox();

    // Scan (non-force)
    let result = server
        .spec_scan(Parameters(ScanInput {
            workspace: Some(tmp.path().display().to_string()),
            force: false,
        }))
        .await
        .expect("spec_scan");
    let json = extract_json(result);
    assert_eq!(json["status"], "ok");

    // Add a new root
    let new_root = tmp.path().join("extra-specs");
    std::fs::create_dir_all(&new_root).expect("mkdir");
    let result = server
        .spec_add_root(Parameters(AddRootInput {
            workspace: Some(tmp.path().display().to_string()),
            path: new_root.to_str().unwrap().to_string(),
            label: Some("extra".to_string()),
        }))
        .await
        .expect("spec_add_root");
    let json = extract_json(result);
    assert_eq!(json["status"], "ok");
    assert_eq!(json["label"], "extra");
}

/// Refs validate tool (no refs = valid).
#[tokio::test]
async fn spec_refs_validate_empty() {
    let (_tmp, server) = make_sandbox();

    let result = server
        .spec_create(Parameters(CreateSpecInput {
            workspace: _tmp.path().display().to_string(),
            title: "Ref Test".to_string(),
            slug: "refs/test".to_string(),
            component: "refs".to_string(),
            parent: None,
            scope: None,
            body: None,
            fields: BTreeMap::new(),
        }))
        .await
        .expect("create");
    let spec_id = extract_json(result)["id"].as_str().unwrap().to_string();

    let result = server
        .spec_refs_validate(Parameters(RefsValidateInput {
            workspace: None,
            id: spec_id,
            workspace_root: ".".to_string(),
        }))
        .await
        .expect("refs_validate");
    let json = extract_json(result);
    assert_eq!(json["status"], "ok");
    assert_eq!(json["valid"], true);
    assert_eq!(json["count"], 0);
}

/// Search tool.
#[tokio::test]
async fn spec_search_tool() {
    let (_tmp, server) = make_sandbox();

    // Create a spec with searchable content
    server
        .spec_create(Parameters(CreateSpecInput {
            workspace: _tmp.path().display().to_string(),
            title: "Searchable Alpha".to_string(),
            slug: "search/alpha".to_string(),
            component: "search".to_string(),
            parent: None,
            scope: None,
            body: Some("This spec covers the alpha module.".to_string()),
            fields: BTreeMap::new(),
        }))
        .await
        .expect("create");

    // Search (may not find it immediately if index is async, but should not error)
    let result = server
        .spec_search(Parameters(SearchSpecsInput {
            workspace: None,
            query: "alpha".to_string(),
            limit: 10,
        }))
        .await
        .expect("spec_search");
    let json = extract_json(result);
    assert_eq!(json["status"], "ok");
    // Search results may or may not contain the new spec depending on
    // indexing timing, but the tool should succeed.
}

/// Workspace validation: omitted aliases produce the canonical error shape.
#[tokio::test]
async fn spec_workspace_validation_error() {
    let (_tmp, server) = make_sandbox();

    // Test 'default' rejection
    let result = server
        .spec_create(Parameters(CreateSpecInput {
            workspace: "default".to_string(),
            title: "Test".to_string(),
            slug: "test/spec".to_string(),
            component: "test".to_string(),
            parent: None,
            scope: None,
            body: None,
            fields: BTreeMap::new(),
        }))
        .await;

    let err = result.expect_err("should fail with 'default'");
    let err_msg = err.to_string();
    assert!(
        err_msg.contains("invalid workspace selector"),
        "error should mention 'invalid workspace selector': {err_msg}"
    );
    assert!(
        err_msg.contains("entity creation requires an explicit workspace path"),
        "error should state the requirement: {err_msg}"
    );
    assert!(
        err_msg.contains("'default'"),
        "error should list 'default' as rejected: {err_msg}"
    );

    // Test empty string rejection
    let result = server
        .spec_create(Parameters(CreateSpecInput {
            workspace: "".to_string(),
            title: "Test".to_string(),
            slug: "test/spec".to_string(),
            component: "test".to_string(),
            parent: None,
            scope: None,
            body: None,
            fields: BTreeMap::new(),
        }))
        .await;

    let err = result.expect_err("should fail with empty string");
    let err_msg = err.to_string();
    assert!(
        err_msg.contains("invalid workspace selector"),
        "error should mention 'invalid workspace selector': {err_msg}"
    );

}

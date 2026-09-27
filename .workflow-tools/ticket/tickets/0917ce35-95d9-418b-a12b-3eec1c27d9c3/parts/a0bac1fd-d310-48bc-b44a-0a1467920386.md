## Summary

When the requested workspace has no local `.workflow-tools/spec` store, spec-mcp can resolve the request to an unrelated existing spec store instead of initializing the requested workspace. In the affected case, calls issued for `C:/Users/linus/git/resume-latex` opened a different store containing duplicate `root/generated-artifacts` records and failed with `duplicate slug: root/generated-artifacts`, even though `resume-latex/.workflow-tools/spec` does not exist.

## Evidence

Relevant code:

- `src/mcp/server.rs`, `SpecServer::resolve_workspace_root`: non-default workspace values are passed to `memory_kernel::workspace::resolve_store_root_from(Path::new(workspace), ".spec")`; the returned path is accepted when it looks like a `.spec` store.
- `src/mcp/server.rs`, `SpecServer::with_store`: calls `SpecStore::open_or_init(&index_root)` and then `store.scan(false)`.
- `crates/spec-api/src/store.rs`, `SpecStore::open_or_init`: initialization/opening is idempotent, but it cannot correct a wrong `index_root`.
- `crates/spec-api/src/store.rs`, `rebuild_slug_index`: scans all indexed manifests and returns `DuplicateSlug` when the selected store contains duplicate slugs.

The server process also derives its default `index_root` from the process current directory in `src/bin/spec-mcp.rs`; a long-lived MCP server therefore needs explicit, deterministic workspace resolution rather than relying on whichever directory the server was started from.

## Reproduction

1. Start/use the already-running spec-mcp server from a workspace that has no `.workflow-tools/spec` store, such as `C:/Users/linus/git/resume-latex`.
2. Call a spec tool with `workspace` set to that repository root, or use the default scope when the server was started from another repository.
3. Observe that the operation accesses an unrelated existing store or reports its data error, for example `duplicate slug: root/generated-artifacts`.
4. Confirm that `C:/Users/linus/git/resume-latex/.workflow-tools/spec` was not initialized.

## Expected behavior

A workspace-root request must resolve only to the deterministic store path for that workspace. If the local store is absent, the first store operation should initialize it there. It must never silently redirect to an unrelated ancestor, process-cwd store, or other existing store.

If a supplied path is ambiguous or cannot be resolved safely, return a clear invalid-workspace error containing the requested path and expected store location.

## Acceptance criteria

- Add regression tests for a workspace with no local spec store and for a workspace whose ancestor or process cwd has another spec store.
- Verify that the requested workspace gets its own initialized store.
- Verify that no unrelated store is opened or scanned.
- Preserve valid explicit `.spec` store and existing-workspace behavior.
- Return a deterministic diagnostic for invalid/ambiguous workspace paths.
- Verify the MCP `spec_list`/`spec_health` flow against a clean workspace with no pre-existing store.

## Scope

This ticket is limited to spec-mcp workspace/store resolution and initialization. It does not delete or reconcile records in any existing spec store.


## Generalized isolated regression case

The regression must not depend on VS Code, a real repository, or a particular machine path. Use `tempfile::tempdir()` and create two independent workspace directories:

```text
<tmp>/parent-workspace/
  .workflow-tools/spec/        # existing store, seeded with two records
<tmp>/parent-workspace/child-workspace/
  .workflow-tools/spec/        # must not exist before the call
```

Seed only the parent store. To reproduce the currently observed error deterministically, place two valid spec manifests in the parent store with different UUID directories but the same slug `root/generated-artifacts`. The child remains completely store-free.

The test then constructs the server with the normal parent/default index root and invokes a spec operation using the explicit child workspace path. The current implementation performs:

```rust
resolve_store_root_from(Path::new(child_workspace), ".spec")
```

Because `resolve_store_root_from` walks ancestors, it returns the parent's existing `.workflow-tools/spec` store. `with_store` opens that store, scans it, rebuilds the slug index, and returns `DuplicateSlug("root/generated-artifacts")`. The child store is never initialized.

The regression assertion should cover all of these facts:

```rust
assert_eq!(error, "duplicate slug: root/generated-artifacts");
assert!(!child_workspace.join(".workflow-tools/spec").exists());
assert_eq!(resolved_store, parent_workspace.join(".workflow-tools/spec")); // current buggy behavior
```

The fixed test should instead assert:

```rust
assert_eq!(resolved_store, child_workspace.join(".workflow-tools/spec"));
assert!(child_workspace.join(".workflow-tools/spec/entities.db").is_file());
assert!(parent_store_was_not_opened_or_scanned);
```

Add companion cases for the path environments that can affect the result:

1. explicit workspace path pointing at a store-free child while an ancestor has a store;
2. explicit `.spec` store path, which must remain an intentional override;
3. server started with a parent CWD/default store while an explicit child workspace is supplied;
4. no local or ancestor store, which must initialize the canonical child path;
5. nested path inside a child workspace, which must not escape the explicit workspace boundary;
6. canonical and legacy layouts present together, verifying the selected path and diagnostic are explicit.

The test should use `canonical_store_root()` for expected paths and should never depend on `resume-latex`, VS Code, the current user profile, or a checked-in store.

## Additional observability requirements

The current error loses the most important context: `with_store` resolves and opens a store before the failing scan, while `spec_err` returns only `duplicate slug: ...`. Improve diagnostics so every MCP operation, including failures, makes the path decision visible:

- include `requested_workspace`, `resolved_workspace_root`, `active_index_root`, and `store_created`/`store_initialized` in successful result scope metadata;
- include the same fields in errors raised after resolution, especially scan/open/index failures;
- log one startup line containing the requested server root and resolved active store root;
- log one per-call line containing the requested workspace, resolved store root, whether the store existed before the call, and whether ancestor discovery was used;
- expose a lightweight `spec_store_info`/`spec_context` operation or equivalent response metadata so callers can inspect the active store without causing a list/scan failure;
- when ancestor walking is intentionally used, report the candidate path chain and the selection reason; when an explicit workspace is supplied, report that resolution was bounded and did not inspect ancestors;
- distinguish `store absent -> initialized locally` from `existing store opened` in machine-readable output.

A failure such as the current one should therefore read approximately: `requested workspace=<child>; resolved store=<parent> via ancestor lookup; child store absent; scan failed: duplicate slug root/generated-artifacts`, rather than exposing only the slug error.

## Test ownership

Prefer a unit test at the resolver boundary in `memory-kernel` for bounded-vs-ancestor semantics, plus an MCP integration test covering `SpecServer::with_store` and the serialized error/context payload. This keeps the path bug reproducible without requiring a running MCP process and still verifies the user-visible behavior.
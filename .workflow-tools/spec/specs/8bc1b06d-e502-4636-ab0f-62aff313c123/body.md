<!-- aligned-structure:v2 -->
# Spec-MCP Workspace Resolution Boundary

## Target Code Location

- [spec/src/mcp/server.rs](spec/src/mcp/server.rs) — `SpecServer::resolve_workspace_root` and `SpecServer::with_store`
- [spec/src/mcp/server/query.rs](spec/src/mcp/server/query.rs) — `spec_list` and `spec_health` MCP flows
- [memory-kernel/src/workspace.rs](memory-kernel/src/workspace.rs) — bounded explicit-workspace resolution and diagnostics
- [memory-kernel/src/workspace_tests.rs](memory-kernel/src/workspace_tests.rs) — resolver boundary regression tests
- [spec/tests/mcp_smoke.rs](spec/tests/mcp_smoke.rs) — serialized MCP integration coverage

## Naming Conventions

- Resolver behavior is named `explicit_workspace_resolution`.
- Store metadata uses `requested_workspace`, `resolved_workspace_root`, `active_index_root`, `store_initialized`, and `ancestor_lookup_used`.
- Regression tests use `tempfile::tempdir()` and canonical `.workflow-tools/spec` paths.

## Requester Input

> Ticket 0917ce35: spec-mcp resolves missing workspace to the wrong existing store.

## Reading Order

1. [workflow-tools/ticket/.agents/instructions/ticket/workflow.instructions.md](../ticket/.agents/instructions/ticket/workflow.instructions.md) — ticket execution and traceability rules
2. [spec/src/mcp/server.rs](spec/src/mcp/server.rs) — MCP resolution and store lifecycle
3. [memory-kernel/src/workspace.rs](memory-kernel/src/workspace.rs) — bounded and ancestor resolver implementations
4. [memory-kernel/src/workspace_tests.rs](memory-kernel/src/workspace_tests.rs) — resolver regression tests
5. [spec/tests/mcp_smoke.rs](spec/tests/mcp_smoke.rs) — MCP regression harness
6. [../.workflow-tools/ticket/tickets/0917ce35-95d9-418b-a12b-3eec1c27d9c3/ticket.toml](../.workflow-tools/ticket/tickets/0917ce35-95d9-418b-a12b-3eec1c27d9c3/ticket.toml) — owning ticket metadata

## Responsibility

When an MCP caller supplies a workspace root, spec-mcp resolves the spec store deterministically within that workspace boundary. Missing local stores initialize locally. Ancestor discovery remains available only for intentional ambient/default resolution or explicit store-path compatibility cases.

Dependents can rely on explicit workspace requests never opening or scanning an unrelated ancestor or process-CWD store.

## Interfaces And Dependencies

- `SpecServer::resolve_workspace_root` selects the active store before `SpecStore::open_or_init` and `scan`.
- `memory_kernel` owns path normalization, canonical store roots, and layout diagnostics.
- Existing explicit `.spec` and canonical `.workflow-tools/spec` store paths remain valid overrides.
- MCP success and post-resolution failure responses expose the path decision and initialization mode.

## Behavior

1. An explicit workspace with no local store resolves to `<workspace>/.workflow-tools/spec` and initializes that path on the first operation.
2. An explicit workspace does not walk ancestors or inspect the process current directory.
3. An explicit store path remains an intentional direct-store override.
4. Default/ambient resolution preserves valid existing-workspace behavior.
5. Successful and failed MCP operations expose requested workspace, resolved workspace root, active index root, prior store existence, initialization result, ancestor-lookup status, candidate chain, and selection reason.
6. Startup emits the requested server root, resolved store root, candidate chain, ancestor lookup status, and selection reason.
7. Invalid workspace inputs return a deterministic diagnostic containing the requested path and expected store location.
8. Canonical storage wins when canonical and legacy layouts coexist, with a diagnostic describing both candidates.

## Boundaries And Failure Cases

- A parent store seeded with duplicate slugs must not be opened or scanned for an explicit store-free child workspace.
- Nested paths inside an explicit store resolve to that store without escaping the explicit boundary.
- Existing valid explicit stores and existing ambient stores remain compatible.
- This spec does not delete, repair, or reconcile records in any existing store.

## Provider/Consumer Contract

- `SpecServer` consumes bounded resolver semantics from `memory-kernel`.
- MCP callers consume deterministic scope metadata and actionable resolution errors from `SpecServer`.

## Examples

Given `<tmp>/parent-workspace/.workflow-tools/spec` with duplicate `parent/only-spec` records and a store-free `<tmp>/parent-workspace/child-workspace`, `spec_list` and `spec_health` calls with the child workspace initialize and use `child-workspace/.workflow-tools/spec`. The response does not contain the parent's duplicate-slug error, and the parent store is not scanned.

## Evidence

Validation passed:

- `cargo test workspace::tests` from `workflow-tools/memory-kernel` — 50 passed.
- `cargo test -p spec --features mcp --test mcp_smoke` from `workflow-tools` — 13 passed.
- Focused MCP tests passed for child-workspace isolation, resolution context on post-resolution errors, explicit store overrides, and invalid workspace diagnostics.
- `cargo check -p spec --features mcp --bin spec-mcp` from `workflow-tools` — passed.
- `cargo test -p spec-api open_or_init --lib` from `workflow-tools` — passed.

`cargo fmt --check --manifest-path spec/Cargo.toml` reports broad formatting differences across existing untouched files as well as files in the current scope; no repository-wide formatting was applied.

Ticket evidence: [0917ce35](../.workflow-tools/ticket/tickets/0917ce35-95d9-418b-a12b-3eec1c27d9c3/ticket.toml).

Current positions: bounded resolver, startup/call diagnostics, and MCP regression coverage are implemented and passing the listed guards.

Governing rule: the repository's spec-system policy introduces the implemented contract when dependents need this behavior.

## Scope

This contract covers only spec-mcp workspace/store resolution, initialization, diagnostics, and focused regression coverage. It excludes unrelated spec-store data cleanup, broad store migration, and UI changes.

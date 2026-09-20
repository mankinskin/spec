<!-- aligned-structure:v2 -->

# Specification Query And Link Resolution CLI

## Target Code Location

[workflow-tools/spec/src/cli/args.rs](../../../workflow-tools/spec/src/cli/args.rs) declares CLI arguments; [workflow-tools/spec/src/cli/commands/crud.rs](../../../workflow-tools/spec/src/cli/commands/crud.rs) implements `cmd_get`; [workflow-tools/spec/src/cli/commands/refs.rs](../../../workflow-tools/spec/src/cli/commands/refs.rs) implements code-reference output; [workflow-tools/spec/src/cli/commands/query.rs](../../../workflow-tools/spec/src/cli/commands/query.rs) exposes `spec health`; [workflow-tools/spec/src/cli/commands/validate_links.rs](../../../workflow-tools/spec/src/cli/commands/validate_links.rs) resolves current ticket links.

## Naming Conventions

Use `TypedTarget` for the shared `spec`, `code`, `ticket`, `document`, `component`, and `criterion` target enum; use `spec dump <id>` for the complete projection, `spec links <id>` for resolved links, and `spec health` for diagnostic findings. This child owns `query-spec-dump`, `query-resolved-links`, and `query-health-findings`; all are read-only.

## Requester Input

> The cli should be able to output the data about each spec id and also resolve and list all of its links from the toml file.

## Reading Order

1. [55d8f2eb Specification Store Contract](../55d8f2eb-70f1-4b90-8c8f-e50d5e311d48/body.md) - persisted manifest and edges provider; its separate `spec migrate`/`spec_migrate_*` operation owns explicit mutation.
2. [ad0685f5 Directed Contract Edge](../ad0685f5-cb35-4c61-b1dc-f69232521e25/body.md) - structured edge provider.
3. [workflow-tools/spec/src/cli/commands/crud.rs](../../../workflow-tools/spec/src/cli/commands/crud.rs) - current get output.
4. [workflow-tools/spec/src/cli/commands/refs.rs](../../../workflow-tools/spec/src/cli/commands/refs.rs) - current code-ref-only output.

## Responsibility

If implemented, a CLI or MCP caller can obtain the same complete structured projection for one spec id and resolved TOML-backed spec, code, ticket, and document links without scraping Markdown; both transports consume a shared API projection rather than duplicate resolution.

## Interfaces And Dependencies

`spec dump <id> --json` accepts a storage UUID selector and returns explicit
`format_version`, storage `id`, immutable `component_id`, renameable `slug`,
timestamps, canonical typed tables, code refs, provider-owned criterion
artifacts, template binding provenance, evidence, observations, provider-owned
edges, sections, and body. `spec links <id> --json` returns source field,
normalized `TypedTarget`, resolution, and failure detail. A target parses only
as `<kind>/v1/<workspace_slug>/<repo_relative_ref>[#<locator>]`; the enum covers
`spec`, `code`, `ticket`, `document`, `component`, and `criterion`. `spec health --json` returns diagnostic
findings with stable severity and category/policy; it does not globally fail
merely because findings exist.

The `workflow-tools/spec` parent is the component composition boundary for
`spec-api`, `spec-cli`, and `spec-mcp`; those concrete child component specs
are specified-but-not-built. This CLI contract does not claim that an MCP
surface already exists.

## Behavior

- `query-spec-dump`: emit all persisted v2 data for an unambiguous storage id, including explicit `format_version`, canonical typed tables, `component_id`, provider-owned criterion artifacts, template bindings, structured edges, and `code_refs`; never infer v2 from fields or alias a slug as identity.
- `query-resolved-links`: parse every TOML-backed target through shared `TypedTarget`, enumerate source field, normalized target, resolution, and failure detail, and compare kinds explicitly without inferring body-only links as structured data. Invalid syntax is a request error; recognized but unsupported kind/version returns `unsupported`.
- `query-health-findings`: emit the structured diagnostic report, preserving distinct `violation` and `migration_notice` categories for hook policy evaluation.

## Boundaries And Failure Cases

The commands are read-only and do not claim body parity. Health findings are
diagnostic results, not a global CLI rejection; configured PostToolUse policy is
the only blocking decision. Unknown/ambiguous id, unparseable field, missing
store, dangling target, and cross-workspace target return a typed resolution
failure while preserving the source record. Invalid `TypedTarget` syntax is a
request error; an otherwise valid recognized unsupported kind or version is an
`unsupported` resolution, not a parser error.
This child never exposes or aliases `spec migrate`, `spec_migrate_*`, `spec move`,
scan, or open as a mutation path.

## Provider/Consumer Contract

Consumes [55d8f2eb Specification Store Contract](../55d8f2eb-70f1-4b90-8c8f-e50d5e311d48/body.md) `store-persists-artifacts` and [ad0685f5 Directed Contract Edge](../ad0685f5-cb35-4c61-b1dc-f69232521e25/body.md) `edge-persisted-typed-model`; provides query evidence to reviewers and MCP callers.

## Examples

`spec dump f1b8f01a --json` returns the parent component spec fields and its hierarchy. `spec links f1b8f01a --json` reports `{ source_field: "code_refs[0]", target: "code/v1/default/workflow-tools/spec/crates/spec-api/src/store.rs#SpecStore::health_all", resolution: "resolved" }`; a dangling document target has `resolution = "missing"`, and `document/v2/default/docs/guide.md` is `unsupported`. `spec health --json` returns a `migration_notice` separately from a `violation`, leaving blocking to the PostToolUse hook.

## Evidence

Position: `partial`; `get --full` emits manifest/body and `refs` emits only code refs. Planned command tests cover every link kind, resolved/missing targets, and matching MCP projection.

## Scope

Owns read-only query projection and resolution, not persistence, health policy,
or the separate migration interface.
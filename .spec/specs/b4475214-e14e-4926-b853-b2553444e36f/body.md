<!-- aligned-structure:v2 -->

# Specification Health Check

## Target Code Location

[workflow-tools/spec/crates/spec-api/src/manifest.rs](../../../workflow-tools/spec/crates/spec-api/src/manifest.rs) owns `SpecManifest::health_issues`; [workflow-tools/spec/crates/spec-api/src/store.rs](../../../workflow-tools/spec/crates/spec-api/src/store.rs) owns `SpecStore::health` and `SpecStore::health_all`; [workflow-tools/spec/crates/spec-api/src/store/hierarchy.rs](../../../workflow-tools/spec/crates/spec-api/src/store/hierarchy.rs) traverses parents; [workflow-tools/spec/src/cli/commands/query.rs](../../../workflow-tools/spec/src/cli/commands/query.rs) exposes `spec health`.

## Naming Conventions

Use `health-` criterion ids and stable finding categories/policies such as
`violation`, `migration_notice`, `link_parity`, `missing_parent`, `orphan`,
`parent_cycle`, `missing_examples`, `criterion_prefix_registry`,
`component_identity`, `composition_criteria`, `contract_edge`, and
`template_binding`, `manifest_version`, and `migration_mapping`. Every
finding carries stable `severity`, `category`, and policy metadata. Root-local
`.spec/health-policy.toml` has `policy_version = 1`, severity `info`, `warning`,
or `error`, maps `migration_notice` to `warning` and `violation` to `error`, and
defines allowlists as `v1:<spec-id>:<category>:<normalized-detail-fingerprint>`
with rationale plus expiry/review metadata. This child
owns `health-validates-references`, `health-allows-unvalidated-criteria`,
`health-no-fulfillment-gate`, `health-hierarchy-integrity`, `health-link-parity`, and `health-examples-section`.

## Requester Input

> `spec health` must parse markdown links from `body.md` and confirm they match the structured links in the toml, failing on drift.

## Reading Order

1. [55d8f2eb Specification Store Contract](../55d8f2eb-70f1-4b90-8c8f-e50d5e311d48/body.md) - persisted artifacts and parent navigation provider.
2. [ad0685f5 Directed Contract Edge](../ad0685f5-cb35-4c61-b1dc-f69232521e25/body.md) - structured-edge provider.
3. [89360ad7 Validation Store Evidence Integration](../89360ad7-d638-49e7-85ba-21839fa99851/body.md) - enforcement consumer.
4. [workflow-tools/spec/crates/spec-api/src/manifest.rs](../../../workflow-tools/spec/crates/spec-api/src/manifest.rs) - existing manifest health.
5. [workflow-tools/spec/crates/spec-api/src/store.rs](../../../workflow-tools/spec/crates/spec-api/src/store.rs) - aggregate health.

## Responsibility

If implemented, `spec health` diagnostically returns every structured finding
needed to trust the persisted contract and its authored Markdown navigation,
without asserting fulfillment or globally failing merely because findings exist.

## Interfaces And Dependencies

Health consumes `SpecManifest`, `body.md`, hierarchy records, parent-owned
ordinary composition criteria, structured provider edges, criterion artifacts, template
bindings, and structured links; it returns `SpecHealthReport` findings through
the CLI. Each finding includes a stable severity, category/policy, and detail;
at minimum `violation` denotes a contract breach and `migration_notice` denotes
distinguishable migration guidance. Health JSON exits zero for findings; the
PostToolUse hook is the policy-controlled blocking decision.

## Behavior

- `health-validates-references`: check required fields, globally unique immutable `component_id`, root membership, ownership, uniqueness, and artifact references.
- `health-allows-unvalidated-criteria` and `health-no-fulfillment-gate`: accept absent validation and never require satisfied evidence.
- `health-hierarchy-integrity`: reject missing parents, orphans, and parent cycles.
- `health-composition-criteria`: independently validate each parent's expected child `component_id` set, child shape, and required component relationships.
- `health-manifest-version-and-migration`: require explicit `format_version = 2` and canonical typed tables for v2 records; issue migration guidance for legacy records and reject a migration whose reviewed legacy-UUID-to-`component_id` mapping is missing, duplicate, or inconsistent. Never infer v2 or a component id from fields.
- `health-provider-consumer-edges`: independently validate provider-owned edge endpoint ids, provider ownership, criterion ownership, and duplicate claims; never infer an edge from hierarchy.
- `health-template-bindings`: independently validate template id/version, parameters, artifact owner identity and provenance, deterministic expansion, and collisions.
- `health-link-parity`: parse Markdown links only under `## Target Code Location`, `## Reading Order`, and `## Provider/Consumer Contract`; normalize recognized targets to `{kind, repo_relative_ref, optional_locator}` and compare kind explicitly against structured TOML links. Ordinary prose links are navigation-only.
- `health-examples-section`: require each spec to have a non-empty `## Examples` section.
- `health-parent-navigation`: verify handwritten root Reading Order and Component Relationship Map content without generating or rewriting `body.md`.
- `health-criterion-prefix-registry`: require exactly one committed registry entry per component, unique ids and prefixes, matching criterion ids, and no orphan entries across all registered scan roots.
- `health-diagnostic-result`: return structured findings, including stable severity and category/policy, without globally rejecting the command solely because findings exist; migration notices remain distinguishable from violations.
- `health-policy`: load only `.spec/health-policy.toml` with `policy_version = 1`, validate severity and category mappings, and accept an exemption only when its versioned identity, rationale, and expiry/review metadata match a normalized finding detail fingerprint.
- `health-template-and-annotation-integrity`: report unresolved template
	version/bindings/generated ids and invalid source annotations without
	pretending that either model is implemented today.

## Boundaries And Failure Cases

Health reports diagnostic structural findings, not fulfillment and not a global
write decision. Invalid Markdown, unknown
link target, unrepresented TOML link, duplicate structured `{relation,target}`,
duplicate or mutated `component_id`, failed composition criterion, invalid edge,
or invalid template binding, v2 version/table mismatch, or invalid migration mapping
tuple, missing examples, missing parent, orphan, cycle, or prefix-registry drift
must be a `violation` finding. Migration guidance must be a `migration_notice`
finding. Repeated navigation links normalize once; different relations to the
same target are valid. Current hierarchy traversal and health code do not
implement these checks.

## Provider/Consumer Contract

Consumes [55d8f2eb Specification Store Contract](../55d8f2eb-70f1-4b90-8c8f-e50d5e311d48/body.md) `store-persists-artifacts` and [ad0685f5 Directed Contract Edge](../ad0685f5-cb35-4c61-b1dc-f69232521e25/body.md) `edge-persisted-typed-model`; provides all health criteria to [89360ad7 Validation Store Evidence Integration](../89360ad7-d638-49e7-85ba-21839fa99851/body.md) for hook enforcement.

## Examples

If `body.md` links `[55d8f2eb Specification Store Contract](../55d8f2eb-70f1-4b90-8c8f-e50d5e311d48/body.md)` but the TOML omits its structured provider edge, `spec health` returns a `violation` with category `link_parity`; if a child has no `## Examples` content, it returns a `violation` with category `missing_examples`. A registry-transition advisory returns a `migration_notice`, not a violation.

## Evidence

Position: `partial`; existing `health_issues` checks field presence and generic dangling `depends_on`, while `health_all` aggregates reports. Planned tests cover TOML/body drift, missing and extra links, every hierarchy defect, policy severity/allowlist validation, and empty Examples; command: `./target/debug/spec.exe --workspace . health --all`. Current baseline is exactly three unrelated `9f0b9e30` findings.

The command remains diagnostic and may exit successfully with findings. The
specified PostToolUse hook, not `spec health`, applies configured blocking
policy and its versioned `(spec_id, issue)` allowlist after relevant
`.spec/specs/` writes. It blocks only findings selected by that policy;
`migration_notice` findings are distinguishable from `violation` findings.

## Scope

Owns health findings and validation behavior; it does not write manifests, render catalog files, or create tickets.

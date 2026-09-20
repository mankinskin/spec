<!-- aligned-structure:v2 -->

# Specification Store Contract

## Target Code Location

[workflow-tools/spec/crates/spec-api/src/store.rs](../../../workflow-tools/spec/crates/spec-api/src/store.rs) owns `SpecStore`; [workflow-tools/spec/crates/spec-api/src/store/hierarchy.rs](../../../workflow-tools/spec/crates/spec-api/src/store/hierarchy.rs) and [workflow-tools/spec/crates/spec-api/src/store/sections.rs](../../../workflow-tools/spec/crates/spec-api/src/store/sections.rs) own preserved child behavior; [workflow-tools/spec/crates/spec-api/src/store_index_render.rs](../../../workflow-tools/spec/crates/spec-api/src/store_index_render.rs) renders catalog Markdown.

## Naming Conventions

Use `SpecStore` for persistence, `spec.toml` for each component spec's
structured contract data, and `body.md` for human navigation. `component_id`
is globally unique and immutable except through a journaled migration. This child owns `store-persists-artifacts`,
`store-preserves-baselines`, `store-removes-retired-model`,
`store-parent-navigation-verification`, `store-criterion-prefix-registry`, and
`store-journaled-recovery`, `store-schema-migration`, and
`store-v2-canonical-tables`.

## Requester Input

> Parent link-list + subcomponent graph requirement: the store-index renderer must be able to emit/verify the parent's child-link list and graph.

## Reading Order

1. [ad0685f5 Directed Contract Edge](../ad0685f5-cb35-4c61-b1dc-f69232521e25/body.md) - persisted edge provider.
2. [b4475214 Specification Health Check](../b4475214-e14e-4926-b853-b2553444e36f/body.md) - persistence consumer.
3. [workflow-tools/spec/crates/spec-api/src/store.rs](../../../workflow-tools/spec/crates/spec-api/src/store.rs) - store boundary.
4. [workflow-tools/spec/crates/spec-api/src/store_index_render.rs](../../../workflow-tools/spec/crates/spec-api/src/store_index_render.rs) - catalog renderer.

## Responsibility

If implemented, readers can round-trip each component spec's criteria and
provider-owned contract edges from its `spec.toml`, compose component specs
through hierarchy, and recover interrupted local or cross-store operations
without silent repair.

## Interfaces And Dependencies

`SpecStore` persists each `SpecManifest` and `body.md`. The new layout requires
`format_version = 2` and canonical typed tables for criteria, evidence,
provider-owned contract edges, template bindings, and observations; table
presence is not version detection. The index renderer
receives raw bodies and manifests to derive the existing hierarchy catalog. The
`.spec` index root owns the committed system-wide
`.spec/criterion-prefixes.toml` registry across all registered scan roots in
the same `SpecStore`; each composition root also owns
`.spec/criterion-templates.toml` and `.spec/health-policy.toml`. A reusable shared operation journal, specified-but-not-built,
is a prerequisite for spec migration and cross-store ticket governance. It owns
planned and inverse writes, operation-scoped locks, collision detection,
apply/resume/rollback, and recovery tests; it is not an extension of the
folder-move journal and it is not two domain-specific journals.

## Behavior

- `store-persists-artifacts`: round-trip each component's storage `id`, immutable `component_id`, criteria, evidence, provider-owned typed edges, template bindings, observations, and distinct `governs_ticket` relations. Parent composition assertions are ordinary parent-owned criteria, not a table or artifact kind of their own. Edge endpoints are `component_id` values; criterion artifacts carry `criterion_id`, `owner_component_id`, behavior, measurement, template provenance/bindings when present, and evidence links.
- `store-v2-canonical-tables`: accept the new canonical layout only with explicit `format_version = 2` and its typed tables; never infer v2 from a field combination.
- `store-preserves-baselines`: preserve sections, hierarchy, and `TicketRef`.
- `store-removes-retired-model`: retire `contract_mode`, expected properties, mandatory evidence requirements, and fulfillment summaries.
- `store-parent-navigation-verification`: verify, but never generate or rewrite, each handwritten parent body's complete direct-child link list and `flowchart TD` graph against its structured hierarchy and component edges.
- `store-template-expansion-persistence`: persist exact-version `CriterionTemplateBinding`
    records with immutable `template_id`, integer `template_version`, lexically
    ordered string parameter `bindings`, `owner_component_id`, and concrete
    `criterion_id`; definitions reside in root-local `.spec/criterion-templates.toml`, while declared old-to-new binding-map migrations report review-required artifacts rather than silently repairing them.
- `store-typed-target-persistence`: persist shared `TypedTarget` references with source field and normalized target rather than parser-specific path strings; valid targets cannot cross a composition root or workspace where the referenced contract requires local identity.
- `store-criterion-prefix-registry`: preserve the committed registry mapping each stable component id to one unique prefix across every registered scan root. Migration first populates the registry and renames affected criteria; it never infers entries automatically.
- `store-journaled-recovery`: use the shared operation journal for every local multi-file mutation before changing `spec.toml`, `body.md`, or `.spec/criterion-prefixes.toml`; after interruption, expose recovery status and deterministic resume or rollback. Cross-store governance uses the same shared prerequisite, reports recoverable drift, and never silently repairs. A global transaction is not required.
- `store-schema-migration`: expose schema/data migration only through explicit, idempotent `spec migrate` operations: `spec migrate --dry-run`, `spec migrate --resume <journal-id>`, and `spec migrate --rollback <journal-id>`, with matching `spec_migrate_*` MCP operations. A migration reads a reviewed explicit mapping file from each legacy spec UUID to its immutable `component_id`; missing, duplicate, or generated mappings fail before writes. Migration is journal-backed and is neither `spec move`, query CLI behavior, nor automatic work performed by scan or open.

## Boundaries And Failure Cases

The store does not decide health policy or migration. Failed parse, missing
component spec, invalid persisted reference, child list omission, graph/body
mismatch, missing or duplicate registry entry, duplicate prefix, malformed
criterion id, absent journal record for a multi-file mutation, or recovery
attempt that would silently repair cross-store drift returns an error or
recoverable-drift status. `spec migrate` detects journal collision before writes
and requires its explicit resume or rollback path; scan/open never migrates.
Retained baselines must not silently change semantics.
Current manifests have no schema-version field; the target v2 layout requires
an explicit `format_version = 2`. Legacy generic forms are detect-and-report
only, following the ticket metadata precedent that detects `related_specs` when
`refs` is absent, and no legacy UUID receives an inferred `component_id`.

The store distinguishes `parent` composition, provider-owned outward
provider/consumer edges, and template bindings from a ticket governing relation;
none may be inferred from another.

## Provider/Consumer Contract

Consumes [ad0685f5 Directed Contract Edge](../ad0685f5-cb35-4c61-b1dc-f69232521e25/body.md) `edge-persisted-typed-model`; provides `store-persists-artifacts` to [b4475214 Specification Health Check](../b4475214-e14e-4926-b853-b2553444e36f/body.md), [a608f774 Specification Root Contract](../a608f774-9f50-4de1-90e2-ffeefb0198b1/body.md), and the future CLI query child.

## Examples

When a parent component spec has 12 direct children, renderer verification rejects a body whose
Reading Order links only 11 children or whose Mermaid graph omits the CLI child;
the persisted `parent` fields are the comparison source.

An interrupted update after writing `spec.toml` but before `body.md` leaves a
journal record with recovery status; resume completes the recorded mutation or
rollback restores the recorded prior state. An interrupted ticket/spec
governance update reports recoverable drift and presents resume/rollback rather
than changing either store automatically.

`spec migrate --dry-run` reports the planned schema/data writes without changing
the store. A later apply interrupted by a collision leaves its shared journal for
`spec migrate --resume <journal-id>` or `spec migrate --rollback <journal-id>`;
`spec scan` and `spec open` only report the current state.

The reviewed mapping explicitly pairs legacy UUID `a1b2...` with
`component_id = "spec-health"`; dry-run rejects a legacy UUID absent from that
mapping instead of deriving an id from its slug or fields.

## Evidence

Position: `partial`; [workflow-tools/spec/crates/spec-api/src/store.rs](../../../workflow-tools/spec/crates/spec-api/src/store.rs) persists current manifests and [workflow-tools/spec/crates/spec-api/src/store_index_render.rs](../../../workflow-tools/spec/crates/spec-api/src/store_index_render.rs) renders catalog trees, but component-only endpoint validation, journals, recovery status, and resume/rollback are specified-but-not-built. Planned `cargo test -p spec-api` store, hierarchy, renderer, local-interruption, and cross-store-drift tests.

## Scope

Owns persistence and index rendering; health policy and CLI formatting are sibling responsibilities.

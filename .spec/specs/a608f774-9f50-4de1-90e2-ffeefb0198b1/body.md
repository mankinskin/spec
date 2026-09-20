<!-- aligned-structure:v2 -->

# Specification Root Contract

## Target Code Location

[workflow-tools/spec/crates/spec-api/src/manifest.rs](../../../workflow-tools/spec/crates/spec-api/src/manifest.rs) defines `SpecManifest`, the root persisted record; [workflow-tools/spec/crates/spec-api/schemas/specification.toml](../../../workflow-tools/spec/crates/spec-api/schemas/specification.toml) defines its lifecycle and generic edge schema.

## Naming Conventions

`SpecManifest` is one component specification; `component` remains its
classification field, not a nested record. `component_id` is its immutable,
human-readable domain identity; manifest `id` is storage identity and `slug` is
renameable navigation. A parent composes children through `parent`; every
outward-contract endpoint is a `component_id`. This child owns `root-surviving-fields`,
`root-component-classification`, `root-component-composition`, and
`root-format-version`.

## Reading Order

1. [.agents/instructions/spec/spec-system.instructions.md](../../../.agents/instructions/spec/spec-system.instructions.md) - root/child authoring rule.
2. [55d8f2eb Specification Store Contract](../55d8f2eb-70f1-4b90-8c8f-e50d5e311d48/body.md) - persistence provider and specified-but-not-built shared operation journal prerequisite.
3. [f482eb83 Ticket Store Integration](../f482eb83-5b47-4ea3-8d5b-b7baa0531333/body.md) - governed-work consumer of that shared journal prerequisite.
4. [workflow-tools/spec/crates/spec-api/src/manifest.rs](../../../workflow-tools/spec/crates/spec-api/src/manifest.rs) - current manifest shape.

## Responsibility

If implemented, dependents can rely on each component being represented by one
independently addressable, immutable `component_id`, with parent specs composing
child specs through hierarchy rather than enclosing component records.

## Interfaces And Dependencies

Each v2 component extends `SpecManifest` with `format_version = 2` and
`component_id`; the `SpecStore`
enforces global uniqueness and rejects a mutation of that field except through
an explicit journaled migration using a reviewed legacy-UUID-to-`component_id`
mapping file. A parent composes direct children through `parent`; its ordinary
parent-owned `CriterionArtifact` records name expected child `component_id`
values, child shape, and required inter-child provider/consumer edges.
[55d8f2eb Specification Store Contract](../55d8f2eb-70f1-4b90-8c8f-e50d5e311d48/body.md) persists each component spec and its shared operation journal prerequisite.

## Behavior

- `root-surviving-fields`: retain storage `id`, immutable `component_id`, lifecycle, `title`, `slug`, `type`, `state`, `scope`, `parent`, `code_refs`, sections, hierarchy, `TicketRef`, and distinct `governs_ticket` relations.
- `root-format-version`: require `format_version = 2` for the new canonical layout; do not classify a manifest as v2 from incidental fields.
- `root-component-classification`: preserve the manifest classifier independently of component-spec identity.
- `root-component-composition`: represent each component as one spec; a parent composes child component specs through `parent`, validates its expected child `component_id` set and relationships through parent-owned criteria, and has no separate containing `spec_id`.

Parent-owned criteria may test the required existence, shape, and relationships
of composed children. They do not turn hierarchy composition into a
provider/consumer edge; governing specification remains a ticket-only goal and
acceptance gate.

## Boundaries And Failure Cases

This contract neither owns a participant criterion nor evidence state. A missing,
duplicate, or post-creation-mutated `component_id` is invalid unless an explicit
journaled migration records the change. A
`governs_ticket` relation is not a `ComponentContractEdge` or an informational
`related_tickets`/`related_specs` link. Missing component spec identity,
an endpoint that is not a component spec id, a separate containing `spec_id`,
or a missing typed ticket-side gate, a v2 manifest without its explicit version,
or a legacy record without an explicit reviewed UUID mapping is invalid. Legacy
generic forms are detect-and-report only and never infer a semantic equivalent
or component identity.

## Provider/Consumer Contract

[fdb7645d Component Specification Contract](../fdb7645d-eac5-4b82-88eb-94cb22f1b0b2/body.md) consumes `root-component-composition`; [55d8f2eb Specification Store Contract](../55d8f2eb-70f1-4b90-8c8f-e50d5e311d48/body.md) provides persistence; [f482eb83 Ticket Store Integration](../f482eb83-5b47-4ea3-8d5b-b7baa0531333/body.md) provides `ticket-governing-spec`.

## Examples

The `f1b8f01a...` parent component spec retains `component = "spec-system"` and
persists `component_id = "component-oriented-specification-system"`. Its Health
Check child is a separate component spec whose `parent` is `f1b8f01a...`; a
provider edge uses the child `component_id`, while neither spec replaces the
other's classifier or hierarchy relationship.

## Evidence

Position: `partial`; [workflow-tools/spec/crates/spec-api/src/manifest.rs](../../../workflow-tools/spec/crates/spec-api/src/manifest.rs) stores the surviving root fields but not the target artifacts. Planned checks: `cargo test -p spec-api --test schema_test` and `./target/debug/spec.exe --workspace . get f1b8f01a-c7da-4a71-97c5-39519a7d7f38 --json`.

## Scope

Owns component-spec identity and hierarchy composition only; criterion shape,
edge shape, and persistence belong to sibling children.

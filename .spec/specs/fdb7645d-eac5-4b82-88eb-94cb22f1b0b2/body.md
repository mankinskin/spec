<!-- aligned-structure:v2 -->

# Component Specification Contract

## Target Code Location

[workflow-tools/spec/crates/spec-api/src/manifest.rs](../../../workflow-tools/spec/crates/spec-api/src/manifest.rs) owns each component specification; [workflow-tools/spec/crates/spec-api/src/code_ref.rs](../../../workflow-tools/spec/crates/spec-api/src/code_ref.rs) owns `CodeRef` serialization and range validation.

## Naming Conventions

Use `ComponentSpec` for a `SpecManifest` that represents one component; do not
introduce a separate component record or a containing `spec_id`. `component_id`
is the immutable, unique `SpecStore` domain identity and every outward-contract
endpoint; manifest `id` remains storage identity and `slug` remains navigation.
Component criterion
ids use the registered system-wide prefix; this child owns
`component-required-fields`, `component-optional-fields`,
`component-root-membership`, `component-criterion-ownership`,
`component-code-refs-required`, and `component-outward-edge-ownership`.

## Requester Input

> Code-first required fields: `code_refs` (target code location) becomes required for a code-facing component spec, and naming conventions are a required section.

## Reading Order

1. [.agents/instructions/spec/spec-system.instructions.md](../../../.agents/instructions/spec/spec-system.instructions.md) - code-first child format.
2. [a608f774 Specification Root Contract](../a608f774-9f50-4de1-90e2-ffeefb0198b1/body.md) - root namespace provider.
3. [aebcbab4 Criterion Artifact Contract](../aebcbab4-2827-4ea1-8244-0a2e6277b571/body.md) - owned criterion consumer.
4. [workflow-tools/spec/crates/spec-api/src/code_ref.rs](../../../workflow-tools/spec/crates/spec-api/src/code_ref.rs) - `CodeRef` contract.

## Responsibility

If implemented, dependents can name a code-facing component spec that owns
criteria and provider edges without conflating its `id` with its classifier or
with a parent component spec.

## Interfaces And Dependencies

Each component is its own `SpecManifest` and requires storage `id`,
`component_id`, `title`, and purpose-bearing body. `parent` is composition,
not an endpoint identity. Components own standard `CriterionArtifact` records;
a parent uses those same records, rather than a separate class, for expected
direct children, child shape, and required inter-child edges. Optional fields are `context`, related spec/evidence ids,
`code_refs`, and provider-owned outward contract edges. Provider identity is
the owning `component_id`:

```toml
[[outward_contract_edges]]
edge_id = "edge-health-consumes-spec-store"
name = "health reads persisted artifacts"
consumer_component_id = "<consumer-component-id>"
provider_component_id = "<owning-component-id>"
criterion_ids = ["<provider-owned-criterion-id>"]
```

## Behavior

- `component-required-fields`: require immutable `component_id`, storage identity, and purpose fields; reject a separate containing `spec_id`.
- `component-optional-fields`: retain only declared optional context links.
- `component-root-membership`: resolve an optional parent component spec as hierarchy composition only.
- `component-criterion-ownership`: exclusively own zero or more standard criteria; parent composition criteria have the ordinary artifact, validation, and evidence shape and do not duplicate child criteria.
- `component-code-refs-required`: a code-facing component has at least one valid `CodeRef`, and its body has a non-empty `## Naming Conventions` section.
- `component-outward-edge-ownership`: each provider component spec persists its own `[[outward_contract_edges]]` rows; `consumer_component_id` and `provider_component_id` are component ids, the provider equals the owning component, and no parent aggregation mirrors an authoritative edge. Every row stays within that provider's composition root and workspace; the provider serializes unique `criterion_ids` lexicographically.

Parent-owned ordinary criteria may evaluate the direct-child component set,
required child shape, and required inter-child provider/consumer edges, but do
not make the parent an identity provider for children or own their internal
criteria.

## Boundaries And Failure Cases

A component spec is not its `component` classifier and owns neither consumer
edges nor another component spec's criteria. A missing, duplicate, or mutable
`component_id`,
separate containing `spec_id`, non-component endpoint, required field, code
reference, naming section, or registry entry is invalid for a code-facing
component; empty owned criteria are valid.

## Provider/Consumer Contract

Consumes [a608f774 Specification Root Contract](../a608f774-9f50-4de1-90e2-ffeefb0198b1/body.md) `root-component-composition`; provides `component-criterion-ownership` to [aebcbab4 Criterion Artifact Contract](../aebcbab4-2827-4ea1-8244-0a2e6277b571/body.md) and `component-root-membership` to [ad0685f5 Directed Contract Edge](../ad0685f5-cb35-4c61-b1dc-f69232521e25/body.md).

## Examples

The Health Check component spec declares a `CodeRef` to `SpecStore::health`,
names the `health-` criterion namespace, and owns `health-link-parity`. Its
`component_id` is used in `consumer_component_id` by providers; its parent's
`component = "spec-api"` remains unrelated classification metadata.

## Evidence

Position: `partial`; current [workflow-tools/spec/crates/spec-api/src/manifest.rs](../../../workflow-tools/spec/crates/spec-api/src/manifest.rs) persists specs and parent hierarchy but does not yet validate component-only endpoint identity or provider-owned edges. Planned checks: manifest round-trip and invalid separate-`spec_id`, non-component endpoint, and missing-`code_refs` cases in `cargo test -p spec-api`.

## Scope

Owns component-spec shape and code-facing metadata requirements, not persistence or health evaluation.

<!-- aligned-structure:v2 -->

# Criterion Artifact Contract

## Target Code Location

[workflow-tools/spec/crates/spec-api/src/manifest.rs](../../../workflow-tools/spec/crates/spec-api/src/manifest.rs) currently defines `AcceptanceCriterion`; it is the replacement location for a provider-owned criterion artifact.

## Naming Conventions

Use `CriterionArtifact`; `criterion_id` values are `<registered-prefix>-<behavior>`
and are unique per root. Each artifact is provider-owned, not a legacy generic
`AcceptanceCriterion`. This child owns `criterion-required-fields`, `criterion-single-owner`,
`criterion-root-unique`, `criterion-optional-validation`, `criterion-evidence-integrity`, and `criterion-naming-conventions-required`.

## Requester Input

> Code-first required fields: `code_refs` (target code location) becomes required for a code-facing component spec, and naming conventions are a required section.

## Reading Order

1. [fdb7645d Component Specification Contract](../fdb7645d-eac5-4b82-88eb-94cb22f1b0b2/body.md) - owner provider.
2. [7498bed7 Evidence Reference Contract](../7498bed7-ac74-4484-b50e-8a9cf96d8431/body.md) - `validated_by` evidence provider.
3. [ad0685f5 Directed Contract Edge](../ad0685f5-cb35-4c61-b1dc-f69232521e25/body.md) - criterion-edge consumer.
4. [workflow-tools/spec/crates/spec-api/src/manifest.rs](../../../workflow-tools/spec/crates/spec-api/src/manifest.rs) - current criterion model.

## Responsibility

If implemented, dependents can rely on one provider-owned, separately named
acceptance obligation rather than copied consumer requirements.

## Interfaces And Dependencies

`CriterionArtifact` requires `criterion_id`, `owner_component_id`, `behavior`,
and `measurement`; `validation_evidence[]` is optional. Optional template
provenance is immutable `template_id`, integer `template_version`, and an exact
version binding map whose string keys serialize lexicographically. The
owner is one component id and no separate containing `spec_id` exists.
Parent composition criteria use this unchanged shape, including normal
validation and evidence links; they are not a second artifact class.

## Behavior

- `criterion-required-fields`: require identity, owner, behavior, and measurement.
- `criterion-single-owner`: resolve exactly one owner `component_id`.
- `criterion-root-unique`: require an id once within the composed component-spec hierarchy.
- `criterion-optional-validation`: accept an empty `validated_by` list.
- `criterion-evidence-integrity`: resolve every named validation-evidence link in that root.
- `criterion-naming-conventions-required`: require a documented owner-prefix naming convention for each code-facing criterion component.
- `criterion-prefix-registry`: require each component to resolve to exactly one `.spec/criterion-prefixes.toml` entry, with unique component ids and prefixes across all registered scan roots; every criterion must match that entry's `<prefix>-<behavior>` form.
- `criterion-template-materialization`: accept a deterministic template expansion only when its concrete artifact id, owner component id, immutable template id, exact integer version, and lexicographically ordered string bindings resolve; the template is not the owner.
- `criterion-composition-graph`: allow a parent-owned ordinary artifact to measure its direct-child component ids, required child shape, and required inter-child provider/consumer edges, while rejecting copied child-internal or provider-owned criteria.

## Boundaries And Failure Cases

Criteria do not copy provider claims into consumer contracts or require an
observation. Missing owner component id, duplicate `criterion_id`, an owner outside the
composed hierarchy, a separate containing `spec_id`, dangling evidence, an
unregistered/mismatched/duplicate/orphan prefix entry, or undocumented
code-facing naming convention is invalid. The target registry schema is:

```toml
[component_prefixes]
"<stable-component-id>" = "<prefix>"
```

This is specified-but-not-built: create the file only after materialized stable
component ids allow real entries, then migrate and rename affected criteria;
never auto-populate it.

Parameterized templates may generate provider-owned criterion artifacts for the `-api`, `-cli`, `-mcp`,
`-http`, and `-viewer` families, but a template remains a definition rather
than a component or provider/consumer edge. A later template version never
rewrites a materialized artifact; it must be represented by a declared binding-map migration and a review-required result.

## Provider/Consumer Contract

Consumes [fdb7645d Component Specification Contract](../fdb7645d-eac5-4b82-88eb-94cb22f1b0b2/body.md) `component-criterion-ownership`; provides `criterion-single-owner` and `criterion-root-unique` to [ad0685f5 Directed Contract Edge](../ad0685f5-cb35-4c61-b1dc-f69232521e25/body.md), and `criterion-required-fields` to [83c0b9c4 Validation Observation Contract](../83c0b9c4-1617-4751-af23-57811060f0fb/body.md).

## Examples

The Health Check component owns `health-link-parity` with behavior requiring
one structured manifest link for each parsed `body.md` Markdown link and a
measurement naming the health finding; an edge may consume its `criterion_id`
but may not restate the artifact.

A parent may own `spec-system-composition` as a normal criterion artifact. Its
measurement checks required direct child ids and their required edges; it does
not copy any child's `health-link-parity` behavior.

## Evidence

Position: `not-implemented`; [workflow-tools/spec/crates/spec-api/src/manifest.rs](../../../workflow-tools/spec/crates/spec-api/src/manifest.rs) stores the retired `AcceptanceCriterion`. Planned manifest/store tests cover round-trip, duplicate id, foreign owner, and empty `validated_by`.

## Scope

Owns criterion shape and ownership only; observations and edge persistence are siblings.

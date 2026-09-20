<!-- aligned-structure:v2 -->

# Validation Observation Contract

## Target Code Location

[workflow-tools/spec/crates/spec-api/src/manifest.rs](../../../workflow-tools/spec/crates/spec-api/src/manifest.rs) owns the future observation artifact; [workflow-tools/test/crates/test-api/src/lib.rs](../../../workflow-tools/test/crates/test-api/src/lib.rs) defines `ValidationExecution` status output.

## Naming Conventions

Use `ValidationObservation`; ids use `observation-<criterion>-<evidence>`. This child owns `observation-required-fields`, `observation-optional-detail`, `observation-reference-integrity`, and `observation-does-not-gate-health`.

## Reading Order

1. [aebcbab4 Criterion Artifact Contract](../aebcbab4-2827-4ea1-8244-0a2e6277b571/body.md) - criterion provider.
2. [7498bed7 Evidence Reference Contract](../7498bed7-ac74-4484-b50e-8a9cf96d8431/body.md) - evidence provider.
3. [89360ad7 Validation Store Evidence Integration](../89360ad7-d638-49e7-85ba-21839fa99851/body.md) - status provider.

## Responsibility

If implemented, dependents can record an optional result linking one criterion
and one evidence reference without turning either into a health gate.

## Interfaces And Dependencies

An observation requires `id`, `criterion_id`, `evidence_reference_id`, and
`status`; timestamp and detail are optional.

## Behavior

- `observation-required-fields` and `observation-optional-detail` define shape.
- `observation-reference-integrity` resolves criterion and evidence in one root.
- `observation-does-not-gate-health` accepts omitted observations and unsatisfied evidence.
- `observation-latest-execution`: evaluate a validation gate by querying test-api executions with `validation_spec_id` and consumer-side filtering `execution.links.acceptance_criterion_ids` for the criterion id. No selected-execution pointer, first-class criterion query/index, ticket id, governing-spec id, or `.test` store identity is stored or matched. Newest `executed_at` wins, deterministic id ordering resolves equal timestamps, and absent matching execution is unsatisfied `pending`. `passed` satisfies, while a later `failed` or `blocked` revokes; test-api retains complete execution history. A validation specification and criterion shared across tickets intentionally makes each matching gate observe the same outcome.

## Boundaries And Failure Cases

An observation is not required evidence, fulfillment summary, or criterion owner.
Missing references and cross-root targets are invalid; absence is valid.

## Provider/Consumer Contract

Consumes [aebcbab4 Criterion Artifact Contract](../aebcbab4-2827-4ea1-8244-0a2e6277b571/body.md) `criterion-required-fields`, [7498bed7 Evidence Reference Contract](../7498bed7-ac74-4484-b50e-8a9cf96d8431/body.md) `evidence-required-fields`, and [89360ad7 Validation Store Evidence Integration](../89360ad7-d638-49e7-85ba-21839fa99851/body.md) `validation-observation-source`.

## Examples

An observation for `health-link-parity` points to `evidence-health-command` with
status `passed` and no detail. Deleting the observation remains structurally valid.

Two tickets sharing one validation specification and criterion query the same
execution set and filter the same `acceptance_criterion_ids` link. When matching
validation runs pass and then fail, the latest failed execution is authoritative
and both gates are unsatisfied; equal timestamps use deterministic id ordering,
and test-api retains both executions in its history.

## Evidence

Position: `not-implemented`; planned replacement of fulfillment-summary tests in spec-api manifest/store suites, then `./target/debug/spec.exe --workspace . health --all`.

## Scope

Owns optional result linkage, not criterion ownership or executable validation storage.

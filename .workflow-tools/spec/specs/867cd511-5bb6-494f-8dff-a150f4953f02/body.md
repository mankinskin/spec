<!-- aligned-structure:v2 -->
# Specification Move Reference Integrity

## Target Code Location
- [Specification move domain](../../workflow-tools/spec/crates/spec-api/src/move_domain.rs)
- [Code references](../../workflow-tools/spec/crates/spec-api/src/code_ref.rs)
- [Specification store](../../workflow-tools/spec/crates/spec-api/src/store.rs)

## Naming Conventions
The domain contract covers `SpecMoveDomain`, `CodeRef.file`, related specification links, and destination-store initialization.

## Requester Input
> Code-reference preflight is accepted. References should block whenever they are not visible, including non-unrelated targets.

## Reading Order
1. [Parent safety contract](../../workflow-tools/.workflow-tools/spec/specs/7487a6b5-39c6-4114-a697-6ed0556e888e/body.md)
2. [Kernel child contract](../../workflow-tools/.workflow-tools/spec/specs/c095bb6b-f343-4ae9-9282-0d51a06d099a/body.md)
3. [Code references](../../workflow-tools/spec/crates/spec-api/src/code_ref.rs)
4. [Specification store](../../workflow-tools/spec/crates/spec-api/src/store.rs)

## Responsibility
Ensure a specification move never leaves a `CodeRef.file`, related-spec link, or other persisted reference unresolved from the destination repository.

## Interfaces And Dependencies
The domain consumes kernel visibility and topology results, initializes missing canonical target stores during execution, and evaluates repository-relative code references against the destination tree before apply.

## Behavior
Preflight fails for any non-visible reference in same-repository, parent/submodule, submodule/parent, or unrelated topology. Code-reference checks distinguish an absent destination file from a path that is valid only in the source repository. Related-spec closure is validated before a batch is accepted.

## Boundaries And Failure Cases
Health checks that only inspect ticket edges are insufficient evidence for move safety. No code-reference path is rewritten implicitly unless an explicit contract defines the rewrite and validates the result.

## Provider/Consumer Contract
The domain consumes the Kernel Cross-Repository Move Safety contract and supplies domain-specific checks to the Move Matrix test contract.

## Examples
A `CodeRef.file` that exists under the source repository but not the destination blocks the plan even when the repositories are parent/child. A visible code reference and complete related-spec closure allow the move to proceed.

## Evidence
Spec API tests must cover every topology, code-reference existence, related-spec closure, target-store initialization, single moves, and set moves.

## Scope
Owner: `spec-api`; implementation is ticket-backed and remains partial until the linked tests pass.

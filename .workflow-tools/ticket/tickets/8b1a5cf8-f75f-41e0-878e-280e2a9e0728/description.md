## Objective
Implement specification-domain reference integrity for cross-repository moves in `spec-api`.

## Requirements
- Validate every `CodeRef.file` against the destination repository tree during preflight.
- Treat every destination-invisible reference as a hard blocker in all topology classes, including parent/child and same-repository moves.
- Validate related-spec closure for single and set moves.
- Preserve idempotent canonical target-store initialization at execution time.
- Keep path rewriting explicit; never silently reinterpret a source-relative code reference.

## Acceptance Criteria
1. Missing destination code references block before apply and identify the offending path.
2. Visible references and complete related-spec closure pass in every supported topology.
3. Single and set move tests cover target initialization, code refs, related specs, resume, and rollback.
4. Existing health and link validation remain green.

## Validation
`cargo test --manifest-path workflow-tools/spec/crates/spec-api/Cargo.toml`
`workflow-tools/target/debug/spec.exe health --workspace workflow-tools/spec --all --json`

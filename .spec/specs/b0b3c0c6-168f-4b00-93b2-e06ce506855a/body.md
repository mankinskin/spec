<!-- aligned-structure:v1 -->

# Summary

Source: `crates/spec-api/src/slug.rs`

## Behavior Story

Source: `crates/spec-api/src/slug.rs`

## Provided Surface Contracts

- Define provided contracts for this behavior slice.

## Required Validation

- Triangulate behavior with executable checks, natural-language clauses, and code/schema/API references when available.

## Related Implementation Tickets

- No related implementation ticket is linked yet.

## Background Knowledge References

- Prefer entity references and context rendering over embedding fully expanded payloads in this spec body.

## Legacy Content (Preserved)

# slug

Source: `crates/spec-api/src/slug.rs`

## Public API

### `validate_slug` (Function)

Validate a slug string.

Rules:
- Must not be empty
- Segments separated by `/`
- Each segment: lowercase `[a-z0-9]` and hyphens `-`
- No empty segments (no `//`, no leading/trailing `/`)
- No uppercase letters
- No special chars other than `-` and `/`

### `SlugIndex` (Struct)

In-memory slug → UUID index with uniqueness enforcement.

### `SlugIndex` (Impl)

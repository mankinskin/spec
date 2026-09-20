<!-- aligned-structure:v1 -->

# Summary

The `CodeRefEditor` component provides a three-step picker for adding or editing `CodeRef` entries in a spec's manifest.

## Behavior Story

The `CodeRefEditor` component provides a three-step picker for adding or editing `CodeRef` entries in a spec's manifest.

## Provided Surface Contracts

- Define provided contracts for this behavior slice.

## Required Validation

- Triangulate behavior with executable checks, natural-language clauses, and code/schema/API references when available.

## Related Implementation Tickets

- No related implementation ticket is linked yet.

## Background Knowledge References

- Prefer entity references and context rendering over embedding fully expanded payloads in this spec body.

## Legacy Content (Preserved)

# spec-editor/coderef-editor

The `CodeRefEditor` component provides a three-step picker for adding or editing
`CodeRef` entries in a spec's manifest.

## What is a CodeRef?

A `CodeRef` records that a spec is implemented by a specific symbol in the codebase:

```toml
[[code_refs]]
file = "crates/spec-api/src/store.rs"
symbol = "SpecStore"
kind = "struct"
start_line = 32
end_line = 480
```

`kind` is one of: `struct`, `enum`, `trait`, `fn`, `impl`, `macro`.

## Three-Step Picker

### Step 1 — File Browser

- Calls `GET /api/workspace/files?crate=<filter>` (editor-extension endpoint, not in
  `spec-http` v1 — added by `spec-editor` backend).
- Returns a flat list of workspace-relative `.rs` file paths.
- Rendered as a filterable list; typing narrows by path fragment.
- "Select" button advances to Step 2.

### Step 2 — Symbol Picker

- Calls `GET /api/workspace/files/:encoded_path/symbols`.
- Editor backend runs `syn`-based extraction (same code path as `spec bootstrap`) on
  the selected file and returns `Vec<{name, kind, start_line, end_line}>`.
- Rendered as a table: `kind` | `name` | `lines`.
- "Select" button advances to Step 3.

### Step 3 — Confirm

- Shows the full `CodeRef` that will be written:
  - `file`, `symbol`, `kind`, `start_line`, `end_line`.
- User can manually adjust `start_line` / `end_line` with number inputs.
- "Add CodeRef" button calls `PATCH /api/specs/:id` to append the new entry.
- On success: Step 3 closes, the `CodeRef` appears in the `CodeRefList`.

## Editing Existing CodeRefs

- Each row in `CodeRefList` has an "Edit" button that opens the picker at Step 3
  pre-filled with the existing values.
- "Delete" button calls `PATCH /api/specs/:id` to remove the entry.

## Validation

- The component calls `GET /api/specs/:id/refs/validate` after any add/delete and
  highlights broken refs (file not found, symbol name mismatch, line range invalid).

<!-- aligned-structure:v1 -->

# Summary

`spec-editor` is a **fully interactive, GPU-accelerated specification editor**.  It shares the same single-process architecture as `spec-viewer` (Dioxus WASM SPA + Axum backend embedding `spec-http`) but adds a rich authoring UX on top of the read-only browsing experience.

## Behavior Story

`spec-editor` is a **fully interactive, GPU-accelerated specification editor**.  It shares the same single-process architecture as `spec-viewer` (Dioxus WASM SPA + Axum backend embedding `spec-http`) but adds a rich authoring UX on top of the read-only browsing experience.

## Provided Surface Contracts

- Define provided contracts for this behavior slice.

## Required Validation

- Triangulate behavior with executable checks, natural-language clauses, and code/schema/API references when available.

## Related Implementation Tickets

- No related implementation ticket is linked yet.

## Background Knowledge References

- Prefer entity references and context rendering over embedding fully expanded payloads in this spec body.

## Legacy Content (Preserved)

# spec-editor

`spec-editor` is a **fully interactive, GPU-accelerated specification editor**.  It
shares the same single-process architecture as `spec-viewer` (Dioxus WASM SPA + Axum
backend embedding `spec-http`) but adds a rich authoring UX on top of the read-only
browsing experience.

The editor's defining quality is that **every part of a spec is directly manipulable
in the UI** — body markdown, sections, CodeRefs, state transitions, parent links — with
live preview, keyboard shortcuts, and GPU-rendered ambient effects.

---

## Goals

1. **In-place spec authoring** — create, edit, and delete specs without leaving the
   browser; no CLI required during a writing session.
2. **Structured field editing** — dedicated controls for every manifest field (title,
   slug, component, scope, `code_refs[]`, `parent`) rather than raw TOML.
3. **Markdown-first body editing** — split-pane editor with live rendered preview;
   keyboard shortcuts for common formatting.
4. **Section lifecycle** — add, rename, reorder, and delete named sections
   (`design.md`, `acceptance.md`, etc.) from a sidebar panel.
5. **CodeRef management** — a picker UI that browses workspace files, selects a symbol
   by name and kind, and writes a validated `CodeRef` entry.
6. **State machine control** — explicit forward-transition button with pre-flight
   validation (completeness, CodeRef validity, section presence) before allowing
   advancement.
7. **GPU-accelerated ambient layer** — `ViewerShell` WebGPU canvas provides ambient
   effects; the edit surface renders on top as a DOM overlay.

---

## Architecture

```
spec-editor (binary)
├── src/main.rs          — CLI args, ServerConfig, mounts spec-http router + editor-api extensions
└── frontend/dioxus/     — Dioxus WASM SPA
    ├── src/main.rs      — launches App with ViewerShell
    ├── src/routes.rs    — Route enum + page components
    ├── src/api.rs       — gloo-net client for spec-http REST + editor write endpoints
    ├── src/store.rs     — editor state (dirty flag, undo stack, active pane)
    ├── src/sse.rs       — SSE for live conflict detection
    ├── src/types.rs     — EditableSpec, SectionDraft, CodeRefDraft mirror types
    └── src/components/
        ├── spec_form.rs         — manifest field editor (title/slug/component/scope/parent)
        ├── body_editor.rs       — split-pane Markdown editor with live preview
        ├── section_panel.rs     — section list with add/rename/delete/reorder
        ├── section_editor.rs    — Markdown editor for a single section file
        ├── coderef_editor.rs    — CodeRef picker: file browser → symbol picker → kind/line
        ├── state_transition.rs  — state machine stepper with pre-flight check display
        ├── spec_tree_nav.rs     — read-only hierarchy sidebar (reused from spec-viewer)
        ├── search_bar.rs        — search to open/navigate to specs
        ├── dirty_banner.rs      — unsaved-changes indicator + Save / Discard actions
        └── health_inline.rs     — inline health issues shown alongside editor fields
```

### Backend (`src/main.rs`)

- Mounts `spec-http` at `/api/` (all existing CRUD endpoints are sufficient for the
  editor — no new backend routes are required for v1).
- Future: a `/api/workspace/files` endpoint (file-tree browser for CodeRef picker).
- Accept `--port` (default **4003**), `--workspace`, `--index-root`, `--static-dir`.
- Use `viewer-api` `ServerConfig`, `init_tracing_full`, `default_cors`,
  `with_static_files`.

### Frontend SPA — Routes

| Route | Page |
|---|---|
| `/` | Redirect → `/workspace/default` |
| `/workspace/:ws` | `SpecListPage` — search + list, with `[+ New Spec]` button |
| `/workspace/:ws/new` | `NewSpecPage` — blank `spec_form` modal overlay |
| `/workspace/:ws/spec/:slug` | `SpecEditPage` — full editing experience |
| `/workspace/:ws/spec/:slug/section/:name` | `SectionEditPage` — focused section editor |

### Editor State (`store.rs`)

```
EditorStore {
    spec_slug: String,
    manifest: EditableSpec,   // in-memory mirror of spec.toml
    body_draft: String,       // in-progress body.md text
    sections: Vec<SectionDraft>,
    active_section: Option<String>,
    dirty: bool,              // any unsaved changes
    saving: bool,             // PATCH in-flight
    errors: Vec<FieldError>,  // per-field validation errors
}
```

- All mutations go through `EditorStore` signals — no direct DOM manipulation.
- Autosave debounce: 2 s after last keystroke fires `PATCH /api/specs/:id/body`.
- Explicit **Save** button always available; **Discard** reloads from server.
- Undo: a ring buffer of up to 50 body-text snapshots enables ⌘Z/Ctrl+Z.

### Body Editor (`body_editor.rs`)

- Left pane: `<textarea>` with monospace font, tab-to-indent, automatic list
  continuation.
- Right pane: Markdown rendered via `pulldown-cmark` compiled to WASM
  (`pulldown-cmark` is pure Rust, compiles to `wasm32-unknown-unknown`).
- Pane ratio configurable; full-screen left or right pane via keyboard shortcut.
- CodeRef syntax highlighting: spans matching `` `<File>:<line>` `` pattern are
  linkified to the host VS Code or source URL.

### CodeRef Picker (`coderef_editor.rs`)

1. **File browser pane**: calls `GET /api/workspace/files` (editor extension) to list
   workspace source files.  Filtered by crate/path.
2. **Symbol picker pane**: calls `GET /api/workspace/files/:path/symbols` (editor
   extension) which runs `syn`-based extraction (same logic as `spec bootstrap`) and
   returns a list of `{name, kind, start_line, end_line}` symbols.
3. **Confirm pane**: user confirms the selected symbol; a `CodeRef` entry is appended
   to the manifest's `code_refs` list and saved via `PATCH /api/specs/:id`.

These two editor-extension endpoints (`/api/workspace/files` and `.../symbols`) are
new routes added by the `spec-editor` backend (not part of `spec-http`).

### State Transition Control (`state_transition.rs`)

Displays the current state as a stepper (`draft → reviewed → approved → implemented →
verified`).  The "Advance" button triggers:

1. **Pre-flight**: calls `GET /api/specs/:id/health` — if issues exist, lists them and
   blocks advancement.
2. **Confirm dialog**: shows the transition (`draft → reviewed`) and any required
   criteria.
3. **Transition**: `POST /api/specs/:id/advance` (to be added to `spec-http`).

Cancelled and deprecated are available via a separate "Archive" dropdown.

---

## Integration Points

- **`spec-http`** — all read/write operations use the existing REST endpoints.
- **`viewer-api-dioxus`** — `ViewerShell`, `Header`, `Sidebar`, shared CSS.
- **`spec-viewer`** — the editor links to the viewer for read-only access; the two can
  run concurrently; no shared state between processes.
- **`pulldown-cmark`** — WASM-compiled Markdown renderer (no JS dependency).
- Future: **`context-engine` LSP-style workspace indexing** for CodeRef file/symbol
  picker.

---

## Deferred (not in v1)

- Real-time collaboration / conflict resolution.
- Inline diff view of spec history (`history.ndjson`).
- Drag-and-drop reordering of sections.
- Bulk state advancement across multiple specs.
- CodeRef jump-to-definition in VS Code (requires VS Code extension bridge).

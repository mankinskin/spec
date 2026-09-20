<!-- aligned-structure:v1 -->

# Summary

The `BodyEditor` component is the core authoring surface for a spec's `body.md`.

## Behavior Story

The `BodyEditor` component is the core authoring surface for a spec's `body.md`.

## Provided Surface Contracts

- Define provided contracts for this behavior slice.

## Required Validation

- Triangulate behavior with executable checks, natural-language clauses, and code/schema/API references when available.

## Related Implementation Tickets

- No related implementation ticket is linked yet.

## Background Knowledge References

- Prefer entity references and context rendering over embedding fully expanded payloads in this spec body.

## Legacy Content (Preserved)

# spec-editor/body-editor

The `BodyEditor` component is the core authoring surface for a spec's `body.md`.

## Layout

Split-pane with configurable ratio (default 50/50):

```
┌─────────────────────────────┬─────────────────────────────┐
│  EDITOR  (textarea)         │  PREVIEW  (rendered HTML)   │
│                             │                             │
│  # My Spec                  │  <h1>My Spec</h1>           │
│  ...                        │  ...                        │
└─────────────────────────────┴─────────────────────────────┘
```

- Pane divider is draggable.
- Toggle buttons in the toolbar: `[Editor only]` `[Split]` `[Preview only]`.
- Pane state persisted in `localStorage`.

## Editor Pane

- `<textarea>` with monospace font and `white-space: pre` / `tab-size: 4`.
- Key bindings:
  - `Tab` → inserts 4 spaces (does not lose focus).
  - `Enter` in a list item → auto-continues the list marker (`- `, `1. `).
  - `Ctrl+B` / `⌘B` → wraps selection in `**...**`.
  - `Ctrl+I` / `⌘I` → wraps in `_..._`.
  - `Ctrl+Z` / `⌘Z` → undo from in-memory ring buffer (50 snapshots).
  - `Ctrl+S` / `⌘S` → immediate save (bypasses autosave debounce).
- Line numbers shown in a pseudo-element gutter (CSS counter).
- Current line highlighted.

## Preview Pane

- Renders the editor text via `pulldown-cmark` compiled to WASM:
  - `pulldown_cmark::Parser` → `pulldown_cmark::html::push_html`.
  - Output set via `element.set_inner_html(sanitised_html)`.
  - Sanitisation: strip `<script>`, `<iframe>`, `on*` attributes, `javascript:`
    hrefs before insertion to prevent XSS.
- Preview re-renders on every keystroke, debounced to 150 ms to avoid excessive WASM
  invocations.
- CodeRef spans (`\`File:line\``) are post-processed to linkified `<a>` elements.

## Autosave

- `use_effect` with a 2-second debounce timer.
- On trigger: `PATCH /api/specs/:id/body` with `{"body": "<current text>"}`.
- `DirtyBanner` reflects `saving` / `saved` / `error` states.

## Dirty State

- `EditorStore.dirty` is set on first keystroke after load.
- Browser `beforeunload` listener warns if `dirty == true` (registered with
  `gloo-events` and removed on component unmount via `on_cleanup`).

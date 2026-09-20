<!-- aligned-structure:v1 -->

# Summary

The `SectionPanel` and `SectionEditor` components manage a spec's named subdocuments (files under `sections/`).

## Behavior Story

The `SectionPanel` and `SectionEditor` components manage a spec's named subdocuments (files under `sections/`).

## Provided Surface Contracts

- Define provided contracts for this behavior slice.

## Required Validation

- Triangulate behavior with executable checks, natural-language clauses, and code/schema/API references when available.

## Related Implementation Tickets

- No related implementation ticket is linked yet.

## Background Knowledge References

- Prefer entity references and context rendering over embedding fully expanded payloads in this spec body.

## Legacy Content (Preserved)

# spec-editor/section-editor

The `SectionPanel` and `SectionEditor` components manage a spec's named subdocuments
(files under `sections/`).

## What are sections?

Sections are additional Markdown files stored alongside `body.md`:

```
.spec/specs/<slug>/sections/
├── design.md
├── acceptance.md
└── risks.md
```

They are accessed via `GET /api/specs/:id/sections/:name` and written via
`PUT /api/specs/:id/sections/:name`.

Common section names are `design`, `acceptance`, `risks`, `changelog`.

## SectionPanel (sidebar)

- Rendered in the left sidebar of `SpecEditPage`.
- Shows a list of existing sections with their name.
- **Add section** button opens an inline input for the section name.
  - Name must match `[a-z0-9-]+` (same slug-segment rules).
  - Creates a new empty section via `PUT /api/specs/:id/sections/:name` with empty
    body.
- Each section row has:
  - Click → opens `SectionEditor` in the main pane.
  - Delete button → `DELETE /api/specs/:id/sections/:name` (with confirmation dialog).

## SectionEditor (main pane)

- Identical to `BodyEditor` (split-pane Markdown + preview) but bound to a named
  section instead of `body.md`.
- Autosave calls `PUT /api/specs/:id/sections/:name`.
- Section name shown in the toolbar as a breadcrumb: `<spec-title> / <section-name>`.
- "Back to body" button navigates to the main `SpecEditPage` (`body_editor`).

## Route Integration

`/workspace/:ws/spec/:slug/section/:name` is a dedicated route that mounts
`SectionEditor` full-screen, enabling deep-linking to a specific section.
Navigating to this route from `SpecDetailPage`'s **Sections** tab opens the editor
(if the editor is running) or falls back to read-only display.

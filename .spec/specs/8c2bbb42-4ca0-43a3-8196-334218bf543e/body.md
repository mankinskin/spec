<!-- aligned-structure:v1 -->

# Summary

The `SpecTree` Dioxus component renders the spec hierarchy as a collapsible tree. It is the primary navigation surface on `SpecTreePage` and also appears as a sidebar in `SpecDetailPage`.

## Behavior Story

The `SpecTree` Dioxus component renders the spec hierarchy as a collapsible tree. It is the primary navigation surface on `SpecTreePage` and also appears as a sidebar in `SpecDetailPage`.

## Provided Surface Contracts

- Define provided contracts for this behavior slice.

## Required Validation

- Triangulate behavior with executable checks, natural-language clauses, and code/schema/API references when available.

## Related Implementation Tickets

- No related implementation ticket is linked yet.

## Background Knowledge References

- Prefer entity references and context rendering over embedding fully expanded payloads in this spec body.

## Legacy Content (Preserved)

# spec-viewer/components/spec-tree

The `SpecTree` Dioxus component renders the spec hierarchy as a collapsible tree.
It is the primary navigation surface on `SpecTreePage` and also appears as a sidebar
in `SpecDetailPage`.

## Data

- Input: a list of `SpecSummary` items (id, slug, title, state, parent).
- Builds a client-side adjacency map `parent_slug → Vec<SpecSummary>` on first render.
- Root nodes: items whose `parent` field is `None` or whose parent is not in the
  current list.

## Rendering

- Each node renders as an `<li>` with:
  - Expand/collapse triangle (if it has children).
  - `StateBadge` (coloured pill).
  - Title as a link to `SpecDetailPage`.
  - Slug in muted text.
- Children are rendered recursively inside a `<ul>` that is conditionally shown based
  on expand state.
- Expand state is stored in a `Signal<HashSet<String>>` (set of expanded slugs) and
  persisted to `localStorage` under `spec-viewer:{workspace}:tree-expand`.

## Interactions

- Click triangle → toggle expand/collapse.
- Click title → navigate to detail page (preserves tree scroll position via
  `sessionStorage` key `spec-viewer:{workspace}:tree-scroll`).
- Keyboard: `Enter` or `Space` on focused row opens detail; `ArrowRight`/`ArrowLeft`
  expand/collapse.

## GPU future

When the 3-D graph render pass (`SpecGraph3D`) is implemented, clicking a tree node
will also fly the camera to the corresponding node in the WebGPU scene.  The
`SpecTree` component will emit a `on_node_focus` callback that `SpecTreePage` wires
into the GPU state.

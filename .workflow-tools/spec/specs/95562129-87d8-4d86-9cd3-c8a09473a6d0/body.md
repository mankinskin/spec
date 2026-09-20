<!-- aligned-structure:v1 -->

# Summary

Defines the Dioxus `Route` enum and the four page-level components for the spec-viewer SPA.

## Behavior Story

Defines the Dioxus `Route` enum and the four page-level components for the spec-viewer SPA.

## Provided Surface Contracts

- Define provided contracts for this behavior slice.

## Required Validation

- Triangulate behavior with executable checks, natural-language clauses, and code/schema/API references when available.

## Related Implementation Tickets

- No related implementation ticket is linked yet.

## Background Knowledge References

- Prefer entity references and context rendering over embedding fully expanded payloads in this spec body.

## Legacy Content (Preserved)

# spec-viewer/routes

Defines the Dioxus `Route` enum and the four page-level components for the spec-viewer SPA.

## Route Table

| Pattern | Route variant | Page component |
|---|---|---|
| `/` | — | Redirect to `/workspace/default` |
| `/workspace/:ws` | `SpecListPage` | Flat list of all specs with search bar |
| `/workspace/:ws/tree` | `SpecTreePage` | Collapsible parent-child hierarchy tree |
| `/workspace/:ws/spec/:slug` | `SpecDetailPage` | Body + sections + CodeRefs + health |

## SpecListPage

- Renders a `SearchBar` at top.
- Below: scrollable list of `SpecCard` rows sorted by last-updated descending.
- Filter chips for `state` (draft / reviewed / approved / implemented / verified) and
  `component`.
- Clicking a row navigates to `SpecDetailPage`.
- SSE subscription via `use_sse`: on `spec.created` / `spec.updated` / `spec.deleted`
  events the list refreshes without full page reload.

### Adopted viewer-api primitives (P5)

The `SpecListPage` is the host for several shared viewer-api primitives so the
spec viewer behaves consistently with the ticket / log / doc viewers:

- **`TabsStore<String>` (P5.1)** — multi-spec tab strip rendered via `TabBar`
  above the active spec.  Tabs are closeable, the active tab is mirrored back
  into the persisted `selected_id`, and the tab payload caches the spec title
  so labels survive page reload before the spec list has refetched.
- **`Breadcrumbs` (P5.2)** — `All specs › <component> › <title>` shown above
  the active `SpecDetail` panel; the component crumb falls back to
  "Uncategorised" when the spec has no component.
- **`HeaderActions` (P5.5)** — replaces the bespoke header buttons with the
  shared home / filter-toggle / theme-toggle action group.  `filter_active`
  reflects the local `filter_panel_open` signal.
- **`Overlay` + `ThemeSettings` (P5.4)** — the theme settings panel is mounted
  inside the shared `Overlay` modal rather than a floating absolute-positioned
  pane, so backdrop click and `Escape` close consistently across viewers.
- **`PathCodec` / `ColonSegmented` (P5.6)** — the `#id=` URL hash routes
  through `ColonSegmented` so hierarchical spec ids (`auth:login`) appear as
  path segments (`auth/login`) and round-trip on reload.  The
  `expand_path_to` helper computes the initial expansion set passed to
  `FileTree.initially_expanded` so a deep link auto-reveals its containing
  folder.
- **`Prefetcher<String, SpecFullResponse>` (P5.7)** — exposed via
  `use_context_provider` at the `App` root with capacity 32.  When the active
  tab changes, `SpecListPage` warms the cache for the active spec's
  same-component siblings plus its prev/next neighbour in the flat list.
  `SpecDetail` consults the cache before issuing `get_spec_full`, so
  cross-tab navigation typically resolves without a network round-trip.

### CategoryPage / SpecTreePage card grid (P5.3)

- Specs are grouped by `component` (fallback: "Uncategorised") into a
  `BTreeMap` and rendered as a `CardSection` per category.
- Each section contains a `CardGrid` of `Card { title, description: state,
  on_click }` items that navigate to the corresponding `SpecDetailPage`.

## SpecTreePage

- Fetches `GET /api/specs/tree` (or `GET /api/specs/:slug/tree` if a root is selected).
- Renders a recursive `SpecTree` component with collapse/expand toggle per node.
- Nodes display `StateBadge` and click to navigate to `SpecDetailPage`.
- URL-synced expand state stored in `localStorage`.

## SpecDetailPage

- Fetches `GET /api/specs/:slug?full=true` on mount.
- Renders body Markdown via `innerHTML` (sanitised with a Rust DOMPurify equivalent or
  a plain-text fallback for security).
- Tabs: **Body** | **Sections** | **CodeRefs** | **Health**.
- Each section listed in the **Sections** tab; clicking loads `GET /api/specs/:slug/sections/:name`.
- **CodeRefs** tab renders `CodeRefList` with file, symbol, kind, line range.
- **Health** tab calls `GET /api/specs/:slug/health` and shows issue list.
- "Edit in spec-editor" link opens `http://localhost:4003/workspace/:ws/spec/:slug`.

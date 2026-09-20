# spec-viewer: theme settings

The spec-viewer's theme settings panel **MUST** conform to the canonical spec
[`viewer-api/theme-settings`](../viewer-api/theme-settings) — same layout, same 17 sections,
same controls, same persistence keys.

## Open/close contract

- Clicking the header `Theme settings` action MUST open a visible modal overlay, not just mount the panel in the DOM.
- The overlay MUST use `.modal-backdrop[role="dialog"][aria-label="Theme settings"]` with fixed positioning, a non-transparent backdrop tint, and a centered `.modal-panel.theme-settings-modal` container.
- The underlying page MUST remain visibly dimmed behind the modal while the theme settings panel is open.

## Viewer-specific overrides

_None._ The viewer conforms to the canonical defaults verbatim:

| Field | Value |
|---|---|
| GPU master toggle default | **ON** (`viewer-api-gpu-enabled` defaults to `"true"`) |
| Default `EffectSettings` | `DEFAULT_EFFECT_SETTINGS` (all effects on at default intensity) |
| Presets | inherited from `viewer-api`'s shared preset list |

## Rationale

The spec-viewer is fully GPU-accelerated by default — the animated background,
glass panels, and particle effects are part of the visual identity of the app and
are expected to be visible on first load. Users who prefer a lightweight, static
presentation can disable the master GPU toggle in ThemeSettings; the choice is
persisted to `viewer-api-gpu-enabled` localStorage.

## Implementation pointers

- Panel lives in `viewer-api/viewer-api/frontend/dioxus/src/components/theme_settings.rs`
  (shared component, mounted by the spec-viewer's settings page).
- Store factory: `ThemeStore::new()` in
  `viewer-api/viewer-api/frontend/dioxus/src/store/theme.rs` with
  `gpu_enabled: true` default.
- The spec-viewer Trunk shell in `memory-viewers/spec-viewer/frontend/dioxus/index.html` MUST load the shared `modal.css` bundle in addition to `glass-panel.css` and `theme-settings.css`.
- Shared modal CSS lives in `viewer-api/viewer-api/frontend/dioxus/public/css/modal.css`.

## Validation

- `npm run test:e2e:release -- e2e-release/viewer-api-primitives.spec.ts -g "P5.4 Overlay: theme settings open in a role=dialog modal-backdrop"` in `memory-viewers/spec-viewer/frontend/dioxus`
- The focused Playwright test MUST attach a screenshot of `.theme-settings.glass-panel` so the open-state is visually reviewable in addition to DOM assertions.

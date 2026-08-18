//! `use_sse` — Dioxus hook for live spec-list updates via Server-Sent Events.
//!
//! Connects to `GET /api/specs/stream` and keeps the provided `specs`
//! [`Signal`] up-to-date with incoming events:
//!
//! | SSE event        | Action                                              |
//! |------------------|-----------------------------------------------------|
//! | `spec.updated`   | Update state/title in-place or append a new entry.  |
//! | `spec.deleted`   | Remove the entry by ID.                             |
//! | `open`           | Reset backoff to the initial value.                 |
//! | `error`          | Close the connection, schedule reconnect w/ backoff.|
//!
//! Reconnect uses exponential backoff: 1 s → 2 s → 4 s → … capped at 30 s.
//!
//! # Graceful degradation
//!
//! If the `/api/specs/stream` endpoint is not available (spec-http not yet
//! emitting SSE), the hook will silently retry with backoff and the list will
//! not auto-refresh — a manual page reload is the fallback.

use dioxus::prelude::*;
use gloo_events::EventListener;
use serde::Deserialize;

use crate::types::{
    SpecSummary,
    SseSpec,
};

// ── SSE payload types ─────────────────────────────────────────────────────────

/// Data payload for `spec.updated` SSE events.
#[derive(Debug, Deserialize)]
struct UpdatedPayload {
    spec: SseSpec,
}

/// Data payload for `spec.deleted` SSE events.
#[derive(Debug, Deserialize)]
struct DeletedPayload {
    id: String,
}

// ── SseHandle ─────────────────────────────────────────────────────────────────

/// RAII owner of a live SSE session.
///
/// Dropping this value:
/// 1. Calls [`web_sys::EventSource::close()`] — prevents browser auto-reconnect.
/// 2. Drops each [`EventListener`] — removes JS listeners from the target.
struct SseHandle {
    es: web_sys::EventSource,
    _listeners: Vec<EventListener>,
}

impl Drop for SseHandle {
    fn drop(&mut self) {
        self.es.close();
    }
}

// ── Backoff parameters ────────────────────────────────────────────────────────

const BACKOFF_INITIAL_MS: u32 = 1_000;
const BACKOFF_MAX_MS: u32 = 30_000;

// ── Browser sleep ─────────────────────────────────────────────────────────────

async fn sleep_ms(ms: u32) {
    let promise = js_sys::Promise::new(&mut |resolve, _reject| {
        web_sys::window()
            .expect("no window")
            .set_timeout_with_callback_and_timeout_and_arguments_0(
                &resolve, ms as i32,
            )
            .expect("setTimeout failed");
    });
    let _ = wasm_bindgen_futures::JsFuture::from(promise).await;
}

// ── Extract MessageEvent data ─────────────────────────────────────────────────

fn msg_data(event: &web_sys::Event) -> String {
    use wasm_bindgen::JsCast as _;
    event
        .dyn_ref::<web_sys::MessageEvent>()
        .and_then(|e| e.data().as_string())
        .unwrap_or_default()
}

// ── Hook ──────────────────────────────────────────────────────────────────────

/// Dioxus hook: open an SSE connection to `/api/specs/stream` and live-update
/// `specs` from incoming events.
///
/// The connection is torn down automatically when the component unmounts.
///
/// * `specs` — the displayed spec list; mutated on `spec.updated` / `spec.deleted`.
pub fn use_sse(specs: Signal<Vec<SpecSummary>>) {
    let reconnect_gen: Signal<u32> = use_signal(|| 0_u32);
    let backoff_ms: Signal<u32> = use_signal(|| BACKOFF_INITIAL_MS);
    let mut handle: Signal<Option<SseHandle>> = use_signal(|| None);

    use_effect(move || {
        let _gen = reconnect_gen();

        // Drop previous connection before opening the next.
        handle.with_mut(|h| *h = None);

        let url = "/api/specs/stream";
        let es = match web_sys::EventSource::new(url) {
            Ok(es) => es,
            Err(e) => {
                tracing::warn!(
                    "EventSource::new failed for spec stream: {:?}",
                    e
                );
                return;
            },
        };

        // ── spec.updated ──────────────────────────────────────────────────
        let mut specs_up = specs;
        let l_updated = EventListener::new(&es, "spec.updated", move |event| {
            let data = msg_data(event);
            match serde_json::from_str::<UpdatedPayload>(&data) {
                Ok(p) => specs_up.with_mut(|v| {
                    if let Some(existing) =
                        v.iter_mut().find(|s| s.id == p.spec.id)
                    {
                        existing.state = p.spec.state;
                        if p.spec.title.is_some() {
                            existing.title = p.spec.title;
                        }
                        if p.spec.slug.is_some() {
                            existing.slug = p.spec.slug;
                        }
                    } else {
                        v.push(SpecSummary {
                            id: p.spec.id,
                            slug: p.spec.slug,
                            title: p.spec.title,
                            state: p.spec.state,
                            component: None,
                        });
                    }
                }),
                Err(e) => tracing::warn!("spec.updated parse error: {e}"),
            }
        });

        // ── spec.deleted ──────────────────────────────────────────────────
        let mut specs_del = specs;
        let l_deleted = EventListener::new(&es, "spec.deleted", move |event| {
            let data = msg_data(event);
            match serde_json::from_str::<DeletedPayload>(&data) {
                Ok(p) => specs_del.with_mut(|v| v.retain(|s| s.id != p.id)),
                Err(e) => tracing::warn!("spec.deleted parse error: {e}"),
            }
        });

        // ── open — reset backoff ──────────────────────────────────────────
        let mut boff_open = backoff_ms;
        let l_open = EventListener::new(&es, "open", move |_| {
            boff_open.set(BACKOFF_INITIAL_MS);
            tracing::debug!("SSE connected to spec stream; backoff reset");
        });

        // ── error — close and schedule reconnect ──────────────────────────
        let es_close = es.clone();
        let mut rgen = reconnect_gen;
        let mut boff_err = backoff_ms;
        let l_error = EventListener::new(&es, "error", move |_| {
            es_close.close();
            let delay = boff_err.peek().clone();
            let next_delay = (delay * 2).min(BACKOFF_MAX_MS);
            boff_err.set(next_delay);
            tracing::debug!("SSE error; reconnecting in {delay}ms");
            wasm_bindgen_futures::spawn_local(async move {
                sleep_ms(delay).await;
                rgen.with_mut(|g| *g = g.wrapping_add(1));
            });
        });

        handle.with_mut(|h| {
            *h = Some(SseHandle {
                es,
                _listeners: vec![l_updated, l_deleted, l_open, l_error],
            })
        });
    });
}

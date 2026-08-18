//! HTTP client for the spec-viewer REST API.
//!
//! Calls `spec-http` endpoints using the browser Fetch API via `gloo-net`.

use percent_encoding::{
    utf8_percent_encode,
    NON_ALPHANUMERIC,
};

use crate::types::*;

// ── Base URL ─────────────────────────────────────────────────────────────────

/// Resolve the API base URL at runtime from `window.location.origin`.
/// Falls back to an empty string (relative URLs) if the window is unavailable.
fn api_base() -> String {
    #[cfg(target_arch = "wasm32")]
    {
        if let Some(win) = web_sys::window() {
            if let Ok(origin) = win.location().origin() {
                return origin;
            }
        }
    }
    String::new()
}

// ── Helper ────────────────────────────────────────────────────────────────────

async fn get_json<T: serde::de::DeserializeOwned>(
    url: &str
) -> Result<T, String> {
    let resp = gloo_net::http::Request::get(url)
        .send()
        .await
        .map_err(|e| format!("fetch error: {e}"))?;

    if !resp.ok() {
        return Err(format!("HTTP {}", resp.status()));
    }

    resp.json::<T>()
        .await
        .map_err(|e| format!("JSON decode error: {e}"))
}

// ── Spec list ─────────────────────────────────────────────────────────────────

pub async fn list_specs(
    state: Option<&str>,
    query: Option<&str>,
    limit: Option<u32>,
) -> Result<SpecListResponse, String> {
    let base = api_base();
    let mut params = Vec::new();
    if let Some(s) = state {
        params.push(format!(
            "state={}",
            utf8_percent_encode(s, NON_ALPHANUMERIC)
        ));
    }
    if let Some(q) = query {
        params.push(format!(
            "query={}",
            utf8_percent_encode(q, NON_ALPHANUMERIC)
        ));
    }
    if let Some(l) = limit {
        params.push(format!("limit={l}"));
    }
    let qs = if params.is_empty() {
        String::new()
    } else {
        format!("?{}", params.join("&"))
    };
    get_json(&format!("{base}/api/specs{qs}")).await
}

// ── Spec search ───────────────────────────────────────────────────────────────

#[allow(dead_code)]
pub async fn search_specs(
    q: &str,
    limit: Option<u32>,
) -> Result<SearchResponse, String> {
    let base = api_base();
    let q_enc = utf8_percent_encode(q, NON_ALPHANUMERIC);
    let limit_part = limit.map(|l| format!("&limit={l}")).unwrap_or_default();
    get_json(&format!("{base}/api/specs/search?q={q_enc}{limit_part}")).await
}

// ── Spec detail ───────────────────────────────────────────────────────────────

#[allow(dead_code)]
pub async fn get_spec(id: &str) -> Result<SpecDetailResponse, String> {
    let base = api_base();
    let id_enc = utf8_percent_encode(id, NON_ALPHANUMERIC);
    get_json(&format!("{base}/api/specs/{id_enc}")).await
}

pub async fn get_spec_full(id: &str) -> Result<SpecFullResponse, String> {
    let base = api_base();
    let id_enc = utf8_percent_encode(id, NON_ALPHANUMERIC);
    get_json(&format!("{base}/api/specs/{id_enc}/full")).await
}

// ── Tree ──────────────────────────────────────────────────────────────────────

#[allow(dead_code)]
pub async fn get_tree(id: &str) -> Result<SpecTreeResponse, String> {
    let base = api_base();
    let id_enc = utf8_percent_encode(id, NON_ALPHANUMERIC);
    get_json(&format!("{base}/api/specs/{id_enc}/tree")).await
}

// ── Sections ─────────────────────────────────────────────────────────────────

#[allow(dead_code)]
pub async fn list_sections(id: &str) -> Result<SectionsResponse, String> {
    let base = api_base();
    let id_enc = utf8_percent_encode(id, NON_ALPHANUMERIC);
    get_json(&format!("{base}/api/specs/{id_enc}/sections")).await
}

pub async fn get_section(
    id: &str,
    name: &str,
) -> Result<SectionResponse, String> {
    let base = api_base();
    let id_enc = utf8_percent_encode(id, NON_ALPHANUMERIC);
    let name_enc = utf8_percent_encode(name, NON_ALPHANUMERIC);
    get_json(&format!("{base}/api/specs/{id_enc}/sections/{name_enc}")).await
}

// ── Health ────────────────────────────────────────────────────────────────────

pub async fn health_check(id: Option<&str>) -> Result<HealthResponse, String> {
    let base = api_base();
    let qs = match id {
        Some(i) => {
            let enc = utf8_percent_encode(i, NON_ALPHANUMERIC);
            format!("?id={enc}")
        },
        None => "?all=true".to_string(),
    };
    get_json(&format!("{base}/api/specs/health{qs}")).await
}

// ── Graph ─────────────────────────────────────────────────────────────────────

pub async fn get_graph() -> Result<SpecGraphResponse, String> {
    let base = api_base();
    get_json(&format!("{base}/api/specs/graph")).await
}

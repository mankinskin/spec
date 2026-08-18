use std::cell::RefCell;

use dioxus::prelude::*;
use pulldown_cmark::{
    html,
    CowStr,
    Event,
    Options,
    Parser,
    Tag,
};
use viewer_api_dioxus::Prefetcher;

use crate::routes::Route;

const SPEC_MARKDOWN_HTML_CACHE_CAPACITY: usize = 256;

thread_local! {
    static SPEC_MARKDOWN_HTML_CACHE: RefCell<Prefetcher<String, String>> =
        RefCell::new(Prefetcher::with_capacity(SPEC_MARKDOWN_HTML_CACHE_CAPACITY));
}

fn markdown_body_class(class: &str) -> String {
    if class.is_empty() {
        "markdown-body".to_string()
    } else {
        format!("markdown-body {class}")
    }
}

fn rewrite_spec_href(dest_url: CowStr<'_>) -> CowStr<'_> {
    let Some(spec_target) = dest_url.as_ref().strip_prefix("spec:") else {
        return dest_url;
    };

    let (spec_id, view) = match spec_target.split_once("?view=") {
        Some((spec_id, view)) if !view.is_empty() => (spec_id, Some(view)),
        _ => (spec_target, None),
    };

    CowStr::from(Route::spec_detail_path(spec_id, view))
}

fn render_spec_markdown(content: &str) -> String {
    let mut opts = Options::empty();
    opts.insert(Options::ENABLE_TABLES);
    opts.insert(Options::ENABLE_FOOTNOTES);
    opts.insert(Options::ENABLE_STRIKETHROUGH);
    opts.insert(Options::ENABLE_TASKLISTS);

    let parser = Parser::new_ext(content, opts).map(|event| match event {
        Event::Start(Tag::Link {
            link_type,
            dest_url,
            title,
            id,
        }) => Event::Start(Tag::Link {
            link_type,
            dest_url: rewrite_spec_href(dest_url),
            title,
            id,
        }),
        _ => event,
    });

    let mut html_buf = String::with_capacity(content.len() * 2);
    html::push_html(&mut html_buf, parser);
    html_buf
}

fn render_spec_markdown_cached(content: &str) -> String {
    let key = content.to_string();
    SPEC_MARKDOWN_HTML_CACHE.with(|cache| {
        let cache = cache.borrow();
        if let Some(html) = cache.get(&key) {
            return html;
        }

        let html = render_spec_markdown(content);
        cache.insert(key, html.clone());
        html
    })
}

#[component]
pub fn SpecMarkdownSurface(
    content: String,
    #[props(default)] class: String,
) -> Element {
    let html = render_spec_markdown_cached(&content);
    let surface_class = if class.is_empty() {
        "spec-markdown-surface".to_string()
    } else {
        format!("spec-markdown-surface {class}")
    };
    let body_class = markdown_body_class("spec-markdown-surface__body");

    rsx! {
        div {
            class: "{surface_class}",
            div {
                class: "{body_class}",
                dangerous_inner_html: "{html}",
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn reset_markdown_cache() {
        SPEC_MARKDOWN_HTML_CACHE.with(|cache| {
            *cache.borrow_mut() =
                Prefetcher::with_capacity(SPEC_MARKDOWN_HTML_CACHE_CAPACITY);
        });
    }

    #[test]
    fn rewrites_spec_scheme_links_to_spec_detail_routes() {
        reset_markdown_cache();

        let html = render_spec_markdown_cached(
            "See [graph induction](spec:16c3ad95-451d-4c09-a118-ca90bcefed9a).",
        );

        assert!(html.contains(
            r#"<a href="/specs/16c3ad95-451d-4c09-a118-ca90bcefed9a">graph induction</a>"#
        ));
    }

    #[test]
    fn preserves_non_spec_links() {
        reset_markdown_cache();

        let html = render_spec_markdown_cached(
            "See [repository docs](https://example.com/docs).",
        );

        assert!(html.contains(
            r#"<a href="https://example.com/docs">repository docs</a>"#
        ));
    }
}

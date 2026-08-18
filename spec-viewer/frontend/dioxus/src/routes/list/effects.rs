use dioxus::prelude::*;
use wasm_bindgen_futures::spawn_local;

use crate::{
    api,
    store::SpecListStore,
    types::SpecSummary,
};

pub(crate) fn persist_store(store: SpecListStore) {
    use_effect(move || {
        store.persist();
    });
}

pub(crate) fn use_spec_list(
    mut specs: Signal<Vec<SpecSummary>>,
    mut loading: Signal<bool>,
    mut list_error: Signal<Option<String>>,
    filter: Signal<String>,
    state_filter: Signal<String>,
) {
    use_effect(move || {
        loading.set(true);
        list_error.set(None);

        let query_value = filter.read().clone();
        let state_value = state_filter.read().clone();
        let query = if query_value.trim().is_empty() {
            None
        } else {
            Some(query_value.trim().to_string())
        };
        let state = if state_value.is_empty() {
            None
        } else {
            Some(state_value)
        };

        spawn_local(async move {
            match api::list_specs(state.as_deref(), query.as_deref(), None)
                .await
            {
                Ok(response) => {
                    specs.set(response.items);
                    loading.set(false);
                },
                Err(message) => {
                    list_error.set(Some(message));
                    loading.set(false);
                },
            }
        });
    });
}

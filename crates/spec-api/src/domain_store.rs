use std::{
    collections::BTreeMap,
    path::{Path, PathBuf},
    sync::Mutex,
};

use memory_kernel::{
    domain_store::{
        CreateEntity, DeleteEntity, DomainStore, DomainStoreResolution, ReadEntity,
        StoreAccessMode, UpdateEntity,
    },
    model::domain::DomainId,
    workspace::StoreRootDiagnostic,
};
use serde_json::Value;

use crate::{
    SpecStore,
    error::SpecError,
    manifest::{SpecId, SpecManifest},
};

/// DomainStore input for creating a Spec through the existing Spec API.
#[derive(Debug, Clone)]
pub struct SpecCreateInput {
    pub manifest: SpecManifest,
    pub body: String,
    pub target_root: Option<PathBuf>,
}

/// Spec-specific update options forwarded to the existing Spec API.
#[derive(Debug, Clone, Default)]
pub struct SpecUpdatePatch {
    pub fields: BTreeMap<String, Value>,
    pub to_state: Option<String>,
}

/// Typed capability adapter over Spec's mutable in-memory slug index.
///
/// The adapter does not add a generic Spec model: it serializes the existing
/// domain store operations while retaining `SpecManifest`, `SpecId`, and
/// Spec-specific update fields.
pub struct SpecDomainStore {
    store: Mutex<SpecStore>,
}

impl SpecDomainStore {
    pub fn new(store: SpecStore) -> Self {
        Self {
            store: Mutex::new(store),
        }
    }

    fn lock(&self) -> Result<std::sync::MutexGuard<'_, SpecStore>, SpecError> {
        self.store
            .lock()
            .map_err(|_| SpecError::Serialization("spec domain store mutex poisoned".to_string()))
    }
}

impl DomainStore for SpecStore {
    fn domain_id() -> DomainId {
        DomainId::new("spec").expect("spec is a valid domain id")
    }

    fn store_dir_name() -> &'static str {
        ".spec"
    }
}

impl SpecStore {
    /// Resolve the Spec store belonging to one selected workspace.
    ///
    /// Reads retain compatibility with a legacy `.spec` store when it is the
    /// only layout present. Creating or opening always targets the canonical
    /// `.workflow-tools/spec` store.
    pub fn resolve_workspace_store(
        local_workspace: &Path,
        access_mode: StoreAccessMode,
    ) -> Result<DomainStoreResolution, SpecError> {
        let mut resolution = <Self as DomainStore>::resolve_store(local_workspace, access_mode)?;
        let legacy_root = resolution.local_workspace.join(".spec");
        let canonical_root =
            memory_kernel::workspace::canonical_store_root(&resolution.local_workspace, ".spec");
        let legacy_exists = legacy_root.is_dir();
        let canonical_exists = canonical_root.is_dir();

        if access_mode == StoreAccessMode::ReadOnly && legacy_exists && !canonical_exists {
            resolution.store_root = legacy_root.clone();
        }

        if legacy_exists {
            let diagnostic = if canonical_exists {
                StoreRootDiagnostic::BothLayoutsPresent {
                    domain: "spec".to_string(),
                    legacy_path: legacy_root,
                    canonical_path: canonical_root,
                }
            } else {
                StoreRootDiagnostic::LegacyStore {
                    domain: "spec".to_string(),
                    legacy_path: legacy_root,
                    canonical_path: canonical_root,
                }
            };
            if !resolution.diagnostics.contains(&diagnostic) {
                resolution.diagnostics.push(diagnostic);
            }
        }
        Ok(resolution)
    }
}

impl CreateEntity for SpecDomainStore {
    type Entity = SpecManifest;
    type CreateInput = SpecCreateInput;
    type CreateResult = SpecId;
    type Error = SpecError;

    fn create_entity(&self, input: Self::CreateInput) -> Result<Self::CreateResult, Self::Error> {
        self.lock()?
            .create(&input.manifest, &input.body, input.target_root.as_deref())
    }
}

impl ReadEntity for SpecDomainStore {
    type EntityId = SpecId;
    type Entity = SpecManifest;
    type ReadResult = SpecManifest;
    type Error = SpecError;

    fn read_entity(&self, id: Self::EntityId) -> Result<Self::ReadResult, Self::Error> {
        self.lock()?.get(&id.to_string())
    }
}

impl UpdateEntity for SpecDomainStore {
    type EntityId = SpecId;
    type Entity = SpecManifest;
    type Patch = SpecUpdatePatch;
    type UpdateResult = SpecManifest;
    type Error = SpecError;

    fn update_entity(
        &self,
        id: Self::EntityId,
        patch: Self::Patch,
    ) -> Result<Self::UpdateResult, Self::Error> {
        self.lock()?
            .update(&id.to_string(), patch.fields, patch.to_state.as_deref())
    }
}

impl DeleteEntity for SpecDomainStore {
    type EntityId = SpecId;
    type DeleteResult = ();
    type Error = SpecError;

    fn delete_entity(&self, id: Self::EntityId) -> Result<Self::DeleteResult, Self::Error> {
        self.lock()?.delete(&id.to_string())
    }
}

#[cfg(test)]
mod tests {
    use memory_kernel::{
        domain_store::{CreateEntity, ReadEntity, StoreAccessMode, UpdateEntity},
        workspace::canonical_store_root,
    };
    use serde_json::json;
    use tempfile::tempdir;

    use super::*;

    #[test]
    fn selected_workspace_resolves_canonical_store_without_touching_siblings() {
        let parent = tempdir().unwrap();
        let selected = parent.path().join("selected");
        let sibling = parent.path().join("sibling");
        std::fs::create_dir_all(&selected).unwrap();
        std::fs::create_dir_all(&sibling).unwrap();

        let resolution =
            SpecStore::resolve_workspace_store(&selected, StoreAccessMode::CreateOrOpen).unwrap();

        assert_eq!(
            resolution.store_root,
            canonical_store_root(&selected, ".spec")
        );
        assert!(resolution.store_root.is_dir());
        assert!(!canonical_store_root(&sibling, ".spec").exists());
    }

    #[test]
    fn typed_capabilities_persist_and_read_back_from_selected_workspace() {
        let workspace = tempdir().unwrap();
        let selected = workspace.path().join("selected");
        std::fs::create_dir_all(&selected).unwrap();
        let store = SpecStore::open_or_init_in_workspace(&selected).unwrap();
        let store = SpecDomainStore::new(store);

        let id = store
            .create_entity(SpecCreateInput {
                manifest: SpecManifest::new("domain/adapter", "Domain adapter", "spec-api"),
                body: "persisted body".to_string(),
                target_root: None,
            })
            .unwrap();
        let updated = store
            .update_entity(
                id,
                SpecUpdatePatch {
                    fields: BTreeMap::from([("title".to_string(), json!("Updated adapter"))]),
                    to_state: None,
                },
            )
            .unwrap();
        assert_eq!(updated.title(), Some("Updated adapter"));

        let read_back = store.read_entity(id).unwrap();
        assert_eq!(read_back.title(), Some("Updated adapter"));
        assert!(
            canonical_store_root(&selected, ".spec")
                .join("specs")
                .join(id.to_string())
                .join("spec.toml")
                .is_file()
        );
    }
}

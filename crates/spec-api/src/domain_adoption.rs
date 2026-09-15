//! Spec-domain adapter boundary onto the generic entity kernel's domain
//! manifest contract (`memory_kernel::model::domain_manifest`).
//!
//! This module registers the spec domain's *current* entity type and schema
//! state with the kernel's generic [`DomainManifest`] without changing any
//! specification field, section, ref, or legacy-link migration behavior.
//! The built-in `specification` schema (see [`crate::default_schema`]) is
//! registered as schema version 1, active — matching the fact that
//! spec-api has not yet introduced per-type schema versioning of its own.
//! Later adoption steps may wire this manifest into migration, hooks, or
//! workspace-capability reporting; this step only proves the manifest
//! boundary itself, mirroring
//! `ticket_api::model::domain_adoption::ticket_domain_manifest`.

use memory_kernel::model::{
    domain::{
        DomainId,
        DomainSchemaVersion,
        EntityTypeId,
        EntityTypeSchemaVersion,
    },
    domain_manifest::{
        DomainManifest,
        DomainManifestError,
        EntityTypeMembership,
        EntityTypeStatus,
    },
};

use crate::default_schema::specification_schema;

/// The kernel [`DomainId`] under which spec-api registers itself.
pub const SPEC_DOMAIN_ID: &str = "spec";

/// The spec domain's own [`DomainSchemaVersion`] as of this adoption step.
/// This tracks the domain manifest's own membership/activation set, not the
/// `specification` entity type's own schema version (see
/// [`EntityTypeMembership`]).
pub const SPEC_DOMAIN_SCHEMA_VERSION: u32 = 1;

/// Build the spec domain's [`DomainManifest`] from the currently delivered
/// `specification` entity type schema ([`specification_schema`]). The type
/// is registered at schema version 1 and [`EntityTypeStatus::Active`], since
/// spec-api has no existing per-type versioning or inactive-type concept to
/// preserve as of this adoption step.
pub fn spec_domain_manifest() -> Result<DomainManifest, DomainManifestError> {
    let domain_id = DomainId::new(SPEC_DOMAIN_ID).expect("SPEC_DOMAIN_ID is non-empty");

    let schema = specification_schema();
    let entity_types = vec![EntityTypeMembership {
        entity_type_id: EntityTypeId::new(schema.type_id)
            .expect("built-in specification schema type_id is non-empty"),
        schema_version: EntityTypeSchemaVersion(1),
        status: EntityTypeStatus::Active,
    }];

    DomainManifest::new(
        domain_id,
        DomainSchemaVersion(SPEC_DOMAIN_SCHEMA_VERSION),
        entity_types,
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn spec_domain_manifest_registers_the_built_in_specification_type_as_active() {
        let manifest = spec_domain_manifest().expect("spec domain manifest is valid");

        assert_eq!(manifest.domain_id.as_str(), SPEC_DOMAIN_ID);
        assert_eq!(
            manifest.schema_version,
            DomainSchemaVersion(SPEC_DOMAIN_SCHEMA_VERSION)
        );

        let schema = specification_schema();
        let entity_type_id = EntityTypeId::new(schema.type_id).unwrap();
        let membership = manifest
            .membership(&entity_type_id)
            .expect("specification type registered in spec domain manifest");
        assert_eq!(membership.schema_version, EntityTypeSchemaVersion(1));
        assert!(membership.status.is_active());

        assert_eq!(manifest.active_entity_types().count(), 1);
        assert_eq!(manifest.inactive_entity_types().count(), 0);
    }
}

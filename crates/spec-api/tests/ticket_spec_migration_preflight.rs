//! Composed ticket+spec migration preflight fixture.
//!
//! Per `transcripts/15-09-2026_entity-kernel-all-domains/03-migration-orchestration.md`
//! and `06-domain-adoption.md`, spec is the natural second domain (after
//! ticket) for the kernel's composed cross-domain migration acceptance
//! check. This is a pure preflight/report fixture built entirely from the
//! kernel's dry-run contracts (`memory_kernel::model::migration`) and each
//! domain's already-adopted [`DomainManifest`]
//! (`ticket_api::model::domain_adoption::ticket_domain_manifest`,
//! `spec_api::domain_adoption::spec_domain_manifest`). It does not touch any
//! live ticket or spec store data, and it does not implement full
//! cross-domain migration — it only proves that a spec-side external URN
//! referencing a ticket entity can be checked against the ticket domain's
//! dry-run report without dereferencing or duplicating the ticket.

use std::collections::BTreeMap;

use memory_kernel::ContentKind;
use memory_kernel::model::domain::EntityTypeId;
use memory_kernel::model::migration::{
    ExternalUrnVersionConstraint,
    MigrationDryRunReport,
};
use memory_kernel::model::urn::Urn;
use spec_api::domain_adoption::spec_domain_manifest;
use ticket_api::model::domain_adoption::ticket_domain_manifest;
use uuid::Uuid;

/// Build the identical target-version map every domain's current dry-run
/// preflight uses in this fixture: every active entity type maps to its own
/// current schema version, i.e. no schema-version bump is planned.
fn unchanged_target_versions<'a>(
    manifest: &'a memory_kernel::model::domain_manifest::DomainManifest,
) -> BTreeMap<EntityTypeId, memory_kernel::model::domain::EntityTypeSchemaVersion> {
    manifest
        .active_entity_types()
        .map(|membership| (membership.entity_type_id.clone(), membership.schema_version))
        .collect()
}

#[test]
fn composed_ticket_and_spec_preflight_reports_are_clean_without_touching_live_data() {
    let ticket_manifest = ticket_domain_manifest().expect("ticket domain manifest is valid");
    let spec_manifest = spec_domain_manifest().expect("spec domain manifest is valid");

    let ticket_targets = unchanged_target_versions(&ticket_manifest);
    let spec_targets = unchanged_target_versions(&spec_manifest);

    // Fixture ids only: no ticket or spec store is opened or written to.
    let ticket_id = Uuid::new_v4();
    let ticket_entity_type = ticket_manifest
        .active_entity_types()
        .next()
        .expect("ticket domain manifest registers at least one entity type")
        .entity_type_id
        .clone();

    // A spec entity's cross-store edge to the ticket, expressed as an
    // external URN with a minimum ticket entity-type schema version it
    // currently satisfies.
    let ticket_urn = Urn::new("default", ContentKind::Ticket, ticket_id).expect("valid urn");
    let constraint = ExternalUrnVersionConstraint::new(
        ticket_urn,
        ticket_entity_type,
        memory_kernel::model::domain::EntityTypeSchemaVersion(1),
    );

    let ticket_report = MigrationDryRunReport::plan(
        &ticket_manifest,
        ticket_manifest.schema_version,
        &ticket_targets,
        &BTreeMap::new(),
        &[constraint.clone()],
    );
    let spec_report = MigrationDryRunReport::plan(
        &spec_manifest,
        spec_manifest.schema_version,
        &spec_targets,
        &BTreeMap::new(),
        &[],
    );

    assert!(ticket_report.steps.is_empty(), "no schema-version bump planned for ticket");
    assert!(
        ticket_report.external_urn_violations.is_empty(),
        "spec's external URN constraint is satisfied by the ticket domain's current version"
    );
    assert!(spec_report.steps.is_empty(), "no schema-version bump planned for spec");
    assert!(spec_report.external_urn_violations.is_empty());
}

#[test]
fn ticket_side_schema_downgrade_surfaces_as_a_dry_run_violation_not_a_live_break() {
    let ticket_manifest = ticket_domain_manifest().expect("ticket domain manifest is valid");
    let ticket_targets = unchanged_target_versions(&ticket_manifest);

    let ticket_id = Uuid::new_v4();
    let ticket_entity_type = ticket_manifest
        .active_entity_types()
        .next()
        .expect("ticket domain manifest registers at least one entity type")
        .entity_type_id
        .clone();
    let ticket_urn = Urn::new("default", ContentKind::Ticket, ticket_id).expect("valid urn");

    // A spec-side external URN constraint requiring a higher ticket
    // entity-type schema version than is currently registered. The
    // violation is surfaced in the ticket domain's own dry-run report;
    // nothing is dereferenced or mutated to detect it.
    let stricter_constraint = ExternalUrnVersionConstraint::new(
        ticket_urn,
        ticket_entity_type,
        memory_kernel::model::domain::EntityTypeSchemaVersion(2),
    );

    let report = MigrationDryRunReport::plan(
        &ticket_manifest,
        ticket_manifest.schema_version,
        &ticket_targets,
        &BTreeMap::new(),
        &[stricter_constraint],
    );

    assert_eq!(report.external_urn_violations.len(), 1);
}

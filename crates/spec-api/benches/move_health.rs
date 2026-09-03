//! Move-health Criterion benchmarks for [`spec_api::move_domain::SpecMoveDomain`].
//!
//! Coverage: entity count, link density via a crossing-link topology
//! (internal moved-to-moved topology is out of scope: the move kernel only
//! ever moves one entity per call, so a "both endpoints moved" density has
//! no distinct production analog), phase separation (preflight_only /
//! apply_only / preflight_plus_apply / rollback / resume), and a fixed-total
//! store-size comparison. The store-size scenario exists because
//! `SpecMoveDomain::scan_store` reconciles by rescanning the *entire*
//! destination store (`SpecStore::scan(true)`) rather than touching only the
//! moved entity, so apply cost is expected to scale with total store size,
//! not just the touched entity.
//!
//! Coverage limitation: resume is benchmarked as an idempotent re-resume of
//! an already-`Validated` journal. The public API (`plan_move_preflight` /
//! `execute_move_with_journal` / `resume_move_with_journal` /
//! `rollback_move_with_journal`) has no way to force an interrupted move, so
//! a genuinely-interrupted resume path cannot be exercised here; this is
//! recorded in the benchmark name (`..._resume_idempotent_proxy`).

use std::{
    fs,
    path::{
        Path,
        PathBuf,
    },
    process::Command,
};

use chrono::Utc;
use criterion::{
    BatchSize,
    Criterion,
    criterion_group,
    criterion_main,
};
use memory_kernel::{
    model::edge::EdgeRecord,
    storage::move_kernel::{
        MoveBlocker,
        MoveExecutionPhase,
        MovePlan,
    },
};
use spec_api::{
    manifest::SpecManifest,
    store::SpecStore,
};
use tempfile::TempDir;
use uuid::Uuid;

fn git_init(repo_root: &Path) {
    let status = Command::new("git")
        .current_dir(repo_root)
        .arg("init")
        .status()
        .expect("run git init");
    assert!(status.success(), "git init failed");
}

/// One isolated source+target workspace pair with `moved_count` moved specs,
/// `background_count` unrelated specs already in the source store (used to
/// vary total store size independent of the moved batch), and `density`
/// crossing-link edges per moved spec into a fixed external pool that stays
/// behind in the source store.
fn build_spec_fixture(
    moved_count: usize,
    density: usize,
    background_count: usize,
) -> (TempDir, SpecStore, PathBuf, Vec<Uuid>) {
    let workspace_dir = tempfile::tempdir().expect("tempdir");
    let repo = workspace_dir.path().join("repo");
    fs::create_dir_all(&repo).expect("create repo dir");
    git_init(&repo);

    let source_workspace = repo.join("source");
    let target_workspace = repo.join("target");
    fs::create_dir_all(&source_workspace).expect("create source workspace");
    fs::create_dir_all(&target_workspace).expect("create target workspace");

    let mut source_store =
        SpecStore::init(&source_workspace).expect("init source store");
    SpecStore::init(&target_workspace).expect("init target store");

    let moved_ids: Vec<Uuid> = (0..moved_count)
        .map(|offset| {
            let manifest = SpecManifest::new(
                &format!("bench/moved-{offset}"),
                &format!("Moved spec {offset}"),
                "spec-api",
            );
            source_store
                .create(&manifest, "moved spec body", None)
                .expect("create moved spec")
        })
        .collect();

    const CROSSING_EXTERNAL_POOL: usize = 20;
    let external_ids: Vec<Uuid> = if density > 0 {
        (0..CROSSING_EXTERNAL_POOL)
            .map(|offset| {
                let manifest = SpecManifest::new(
                    &format!("bench/external-{offset}"),
                    &format!("External spec {offset}"),
                    "spec-api",
                );
                source_store
                    .create(&manifest, "external spec body", None)
                    .expect("create external spec")
            })
            .collect()
    } else {
        Vec::new()
    };

    for offset in 0..background_count {
        let manifest = SpecManifest::new(
            &format!("bench/background-{offset}"),
            &format!("Background spec {offset}"),
            "spec-api",
        );
        source_store
            .create(&manifest, "background spec body", None)
            .expect("create background spec");
    }

    if density > 0 {
        let now = Utc::now();
        let crossing_density = density.min(external_ids.len());
        for (idx, moved_id) in moved_ids.iter().enumerate() {
            for step in 0..crossing_density {
                let target_idx = (idx + step) % external_ids.len();
                source_store
                    .entity_store()
                    .add_edge(EdgeRecord {
                        from: *moved_id,
                        to: external_ids[target_idx],
                        kind: "linked".to_string(),
                        created_at: now,
                    })
                    .expect("add crossing edge");
            }
        }
    }

    source_store.scan(true).expect("scan source store");

    (workspace_dir, source_store, target_workspace, moved_ids)
}

/// Build a supported preflight plan for `id`, dropping the blockers that are
/// expected artifacts of the isolated bench fixture rather than genuine
/// domain conflicts (mirrors the existing spec move unit tests).
fn active_move_plan(
    store: &SpecStore,
    target_root: &Path,
    id: &Uuid,
) -> MovePlan {
    let mut plan = store
        .plan_move_preflight(id, target_root)
        .expect("plan preflight");
    plan.blockers.retain(|blocker| {
        !matches!(
            blocker,
            MoveBlocker::PathReferenceScanUnavailable { .. }
                | MoveBlocker::DirtyTrackedFiles { .. }
        )
    });
    assert!(plan.supported(), "unexpected move blockers: {:?}", plan.blockers);
    plan
}

// --- Entity count ---

fn bench_spec_move_preflight_by_entity_count(c: &mut Criterion) {
    for &moved_count in &[5usize, 25, 100] {
        let (_workspace_dir, store, target_root, ids) =
            build_spec_fixture(moved_count, 0, 0);
        let id = ids[0];
        c.bench_function(
            &format!("spec_move_preflight_{moved_count}entities"),
            |b| {
                b.iter(|| {
                    let plan = store
                        .plan_move_preflight(&id, &target_root)
                        .expect("plan preflight");
                    criterion::black_box(plan);
                });
            },
        );
    }
}

// --- Link density (crossing-link topology only; see module doc) ---

fn bench_spec_move_preflight_by_link_density(c: &mut Criterion) {
    const MOVED_COUNT: usize = 25;
    for &density in &[0usize, 5, 20] {
        let (_workspace_dir, store, target_root, ids) =
            build_spec_fixture(MOVED_COUNT, density, 0);
        let id = ids[0];
        c.bench_function(
            &format!("spec_move_preflight_crossing_{density}links"),
            |b| {
                b.iter(|| {
                    let plan = store
                        .plan_move_preflight(&id, &target_root)
                        .expect("plan preflight");
                    criterion::black_box(plan.reference_visibility.len());
                });
            },
        );
    }
}

// --- Phase separation ---

fn bench_spec_move_preflight_only(c: &mut Criterion) {
    c.bench_function("spec_move_phase_preflight_only", |b| {
        b.iter_batched(
            || build_spec_fixture(1, 0, 0),
            |(_workspace_dir, store, target_root, ids)| {
                let plan = store
                    .plan_move_preflight(&ids[0], &target_root)
                    .expect("plan preflight");
                criterion::black_box(plan);
            },
            BatchSize::SmallInput,
        );
    });
}

fn bench_spec_move_apply_only(c: &mut Criterion) {
    c.bench_function("spec_move_phase_apply_only", |b| {
        b.iter_batched(
            || {
                let (workspace_dir, store, target_root, ids) =
                    build_spec_fixture(1, 0, 0);
                let plan = active_move_plan(&store, &target_root, &ids[0]);
                (workspace_dir, store, plan)
            },
            |(_workspace_dir, store, plan)| {
                let outcome = store
                    .execute_move_with_journal(&plan)
                    .expect("execute move");
                assert_eq!(outcome.journal.phase, MoveExecutionPhase::Validated);
                criterion::black_box(outcome);
            },
            BatchSize::SmallInput,
        );
    });
}

fn bench_spec_move_preflight_plus_apply(c: &mut Criterion) {
    c.bench_function("spec_move_phase_preflight_plus_apply", |b| {
        b.iter_batched(
            || build_spec_fixture(1, 0, 0),
            |(_workspace_dir, store, target_root, ids)| {
                let plan = active_move_plan(&store, &target_root, &ids[0]);
                let outcome = store
                    .execute_move_with_journal(&plan)
                    .expect("execute move");
                assert_eq!(outcome.journal.phase, MoveExecutionPhase::Validated);
                criterion::black_box(outcome);
            },
            BatchSize::SmallInput,
        );
    });
}

fn bench_spec_move_rollback(c: &mut Criterion) {
    c.bench_function("spec_move_phase_rollback", |b| {
        b.iter_batched(
            || {
                let (workspace_dir, store, target_root, ids) =
                    build_spec_fixture(1, 0, 0);
                let plan = active_move_plan(&store, &target_root, &ids[0]);
                let outcome = store
                    .execute_move_with_journal(&plan)
                    .expect("execute move");
                (workspace_dir, store, outcome.journal.id)
            },
            |(_workspace_dir, store, journal_id)| {
                let outcome = store
                    .rollback_move_with_journal(journal_id)
                    .expect("rollback move");
                assert!(outcome.rolled_back);
                criterion::black_box(outcome);
            },
            BatchSize::SmallInput,
        );
    });
}

/// Coverage limitation: this benchmarks `resume_move_with_journal` called on
/// an already-`Validated` journal (an idempotent re-resume), since the
/// public move API cannot synthesize a genuinely-interrupted move. See the
/// module doc comment.
fn bench_spec_move_resume_idempotent_proxy(c: &mut Criterion) {
    c.bench_function("spec_move_phase_resume_idempotent_proxy", |b| {
        b.iter_batched(
            || {
                let (workspace_dir, store, target_root, ids) =
                    build_spec_fixture(1, 0, 0);
                let plan = active_move_plan(&store, &target_root, &ids[0]);
                let outcome = store
                    .execute_move_with_journal(&plan)
                    .expect("execute move");
                (workspace_dir, store, outcome.journal.id)
            },
            |(_workspace_dir, store, journal_id)| {
                let outcome = store
                    .resume_move_with_journal(journal_id)
                    .expect("resume move");
                criterion::black_box(outcome);
            },
            BatchSize::SmallInput,
        );
    });
}

// --- Fixed total store-size comparison ---
//
// Full reconciliation rescans the destination store in full
// (`SpecMoveDomain::scan_store` -> `SpecStore::scan(true)`), so apply cost is
// expected to scale with total store size, not just the touched entity.

fn bench_spec_move_apply_by_store_size(c: &mut Criterion) {
    const MOVED_COUNT: usize = 5;
    const DENSITY: usize = 5;
    for &background_count in &[10usize, 100, 400] {
        let total_store_size = MOVED_COUNT + background_count;
        c.bench_function(
            &format!("spec_move_apply_store_size_{total_store_size}specs"),
            |b| {
                b.iter_batched(
                    || {
                        let (workspace_dir, store, target_root, ids) =
                            build_spec_fixture(
                                MOVED_COUNT,
                                DENSITY,
                                background_count,
                            );
                        let plan =
                            active_move_plan(&store, &target_root, &ids[0]);
                        (workspace_dir, store, plan)
                    },
                    |(_workspace_dir, store, plan)| {
                        let outcome = store
                            .execute_move_with_journal(&plan)
                            .expect("execute move");
                        assert_eq!(
                            outcome.journal.phase,
                            MoveExecutionPhase::Validated
                        );
                        criterion::black_box(outcome);
                    },
                    BatchSize::SmallInput,
                );
            },
        );
    }
}

criterion_group!(
    move_health,
    bench_spec_move_preflight_by_entity_count,
    bench_spec_move_preflight_by_link_density,
    bench_spec_move_preflight_only,
    bench_spec_move_apply_only,
    bench_spec_move_preflight_plus_apply,
    bench_spec_move_rollback,
    bench_spec_move_resume_idempotent_proxy,
    bench_spec_move_apply_by_store_size,
);
criterion_main!(move_health);

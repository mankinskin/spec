//! `spec move` — cross-workspace spec move, mirroring the ticket move surface.

use serde_json::{json, Value};
use spec_api::SpecStore;
use uuid::Uuid;

use crate::cli::{CliRunError, MoveArgs};

pub(crate) fn cmd_move(args: MoveArgs, store: &SpecStore) -> Result<Value, CliRunError> {
    if args.resume.is_some() && args.rollback.is_some() {
        return Err(CliRunError::BadRequest(
            "move accepts only one of --resume or --rollback".to_string(),
        ));
    }

    if let Some(journal_id) = args.resume.as_deref() {
        let journal_id = journal_id.parse::<Uuid>().map_err(|error| {
            CliRunError::BadRequest(format!("invalid --resume journal UUID: {error}"))
        })?;
        let outcome = store.resume_move_with_journal(journal_id)?;
        return Ok(json!({
            "command": "move",
            "status": "ok",
            "mode": "resume",
            "outcome": move_outcome_json(&outcome),
            "recovery": recovery_hint(),
        }));
    }

    if let Some(journal_id) = args.rollback.as_deref() {
        let journal_id = journal_id.parse::<Uuid>().map_err(|error| {
            CliRunError::BadRequest(format!("invalid --rollback journal UUID: {error}"))
        })?;
        let outcome = store.rollback_move_with_journal(journal_id)?;
        return Ok(json!({
            "command": "move",
            "status": "ok",
            "mode": "rollback",
            "outcome": move_outcome_json(&outcome),
            "recovery": recovery_hint(),
        }));
    }

    let id = args.id.as_deref().ok_or_else(|| {
        CliRunError::BadRequest("move requires <id> unless --resume/--rollback is used".to_string())
    })?;
    let to_workspace_root = args.to_workspace_root.as_deref().ok_or_else(|| {
        CliRunError::BadRequest(
            "move requires --to-workspace-root in plan/execute mode".to_string(),
        )
    })?;

    let requested_ids = id.split(',').map(str::trim).filter(|id| !id.is_empty());
    let requested_ids = requested_ids
        .map(|id| store.resolve_id(id))
        .collect::<Result<Vec<_>, _>>()?;
    if requested_ids.is_empty() {
        return Err(CliRunError::BadRequest(
            "move requires at least one non-empty spec id".to_string(),
        ));
    }

    if requested_ids.len() > 1 {
        let report = store.plan_move_set(&requested_ids, to_workspace_root)?;
        if args.dry_run || !report.supported() {
            return Ok(json!({
                "command": "move",
                "status": if report.supported() { "ok" } else { "blocked" },
                "mode": "plan",
                "dry_run": true,
                "spec_ids": report.entity_ids,
                "plan": report,
                "recovery": recovery_hint(),
            }));
        }

        let outcome = store.execute_move_set(&report)?;
        return Ok(json!({
            "command": "move",
            "status": "ok",
            "mode": "execute",
            "spec_ids": outcome.entity_ids,
            "journal_id": outcome.journal.id,
            "phase": outcome.journal.phase,
            "entity_journal_ids": outcome.journal.entity_journal_ids,
            "recovery": recovery_hint(),
        }));
    }

    let spec_id = requested_ids[0];
    let report = store.plan_move_preflight(&spec_id, to_workspace_root)?;

    if args.dry_run || !report.supported() {
        return Ok(json!({
            "command": "move",
            "status": if report.supported() { "ok" } else { "blocked" },
            "mode": "plan",
            "dry_run": true,
            "spec_id": spec_id,
            "plan": move_plan_json(&report),
            "recovery": recovery_hint(),
        }));
    }

    let outcome = store.execute_move_with_journal(&report)?;
    Ok(json!({
        "command": "move",
        "status": "ok",
        "mode": "execute",
        "spec_id": spec_id,
        "plan": move_plan_json(&report),
        "outcome": move_outcome_json(&outcome),
        "recovery": recovery_hint(),
    }))
}

fn move_plan_json(report: &memory_kernel::storage::move_kernel::MovePlan) -> Value {
    json!({
        "supported": report.supported(),
        "source_workspace_root": disp(&report.source_workspace_root),
        "target_workspace_root": disp(&report.target_workspace_root),
        "blockers": report.blockers,
        "path_reference_files": report.path_reference_files.iter().map(|p| disp(p)).collect::<Vec<_>>(),
        "captured_at": report.captured_at,
    })
}

fn move_outcome_json(outcome: &memory_kernel::storage::move_kernel::MoveOutcome) -> Value {
    json!({
        "resumed": outcome.resumed,
        "rolled_back": outcome.rolled_back,
        "journal_id": outcome.journal.id,
        "phase": outcome.journal.phase,
    })
}

fn recovery_hint() -> Value {
    json!({
        "resume": "spec move --resume <journal-uuid>",
        "rollback": "spec move --rollback <journal-uuid>",
    })
}

fn disp(path: &std::path::Path) -> String {
    memory_kernel::workspace::normalize_path_for_display(path)
}

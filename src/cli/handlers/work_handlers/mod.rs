// Work handlers - split for file health (CB-040)
#![cfg_attr(coverage_nightly, coverage(off))]

pub mod core_handlers;

// Re-export public handler functions from core_handlers so existing callers
// (e.g. command_dispatcher_work.rs) can use `work_handlers::handle_work_init`, etc.
pub use core_handlers::{
    handle_work_checkpoint, handle_work_complete, handle_work_continue, handle_work_cot_check,
    handle_work_cot_derive, handle_work_delegate, handle_work_event, handle_work_falsify,
    handle_work_init, handle_work_ledger_verify, handle_work_start, handle_work_status,
    handle_work_sync, run_quality_gates, FalsificationResult, GitHubIssueInfo, SyncOptions,
};

// Re-export ticket handlers that are used by the command dispatcher
// (handle_work_score lives in ticket_score.rs, included by ticket_handlers.rs)

// Imports needed by ticket_handlers.rs (included below via include!())
// These mirror what was previously available via include!("core_handlers.rs")
use crate::models::roadmap::ItemStatus;
use crate::services::roadmap_service::RoadmapService;
use anyhow::{Context, Result};
use std::path::{Path, PathBuf};

// FLOW-03 (#1440): the triage gate and the epic sub-issue link for `work add`.
pub mod work_add_triage;
pub use work_add_triage::WorkAddTriage;

include!("ticket_handlers.rs");

// PMAT-674: duplicate-id and parse-location guards for `pmat work validate`.
// Declared here rather than from `src/tests/lib.rs`, which is an orphan target
// (docs/status/orphan-files-ledger.md) — CI runs `cargo test --lib`, so a test
// registered only there would never be compiled, and would never fail.
#[cfg(test)]
#[path = "../../../tests/work_validate_duplicate_ids_tests.rs"]
mod work_validate_duplicate_ids_tests;
// PMAT-673: the `work add` id allocator's tests. Registered here rather than in
// `src/tests/lib.rs` because nothing reaches that file — `autotests = false`
// (Cargo.toml) plus no `mod` from any target root makes every sibling in
// `src/tests/` an orphan (`docs/status/orphan-files-ledger.md`), so a
// concurrency test registered there would never be compiled and its silence
// would read as a pass. `cargo test --lib -- work_add_allocator` runs them.
#[cfg(test)]
#[path = "../../../tests/work_add_allocator_tests.rs"]
mod work_add_allocator_tests;
// PMAT-713 (#1240): a merge that REUSES an id deletes a ticket, and every
// uniqueness check passes afterwards because the survivor is unique. Registered
// here for the same reason as its siblings — a file left in `tests/` with no
// `mod` is never compiled, and silence reads as a pass.
#[cfg(test)]
#[path = "../../../tests/roadmap_id_collision_tests.rs"]
mod roadmap_id_collision_tests;
// PMAT-676: `work add` and `work edit` must refuse a roadmap `work validate`
// rejects. Registered here for the same reason as its two siblings above —
// `src/tests/lib.rs` reaches nothing, so a test file left there is never
// compiled. `cargo test --lib -- work_add_refuses_invalid` runs them.
#[cfg(test)]
#[path = "../../../tests/work_add_refuses_invalid_tests.rs"]
mod work_add_refuses_invalid_tests;
// PMAT-679: `work add` must APPEND the row it mints and `work edit` must
// replace only the row it edits — every untouched byte identical. Registered
// here for the same reason as its three siblings above: nothing reaches
// `src/tests/lib.rs`, so a test file left there is never compiled and its
// silence would read as a pass. `cargo test --lib -- work_add_append_only`
// runs them.
#[cfg(test)]
#[path = "../../../tests/work_add_append_only_tests.rs"]
mod work_add_append_only_tests;

// PMAT-1363: registered here for the same reason as the suite above — a file in
// src/tests/ with no `mod` is never compiled, and its silence reads as a pass.
#[cfg(test)]
#[path = "../../../tests/roadmap_fragments_tests.rs"]
mod roadmap_fragments_tests;
#[cfg(test)]
#[path = "../../../tests/roadmap_fragments_wiring_tests.rs"]
mod roadmap_fragments_wiring_tests;
// PMAT-1385: the roadmap WRITER GATE — every raw write that can land under
// docs/roadmaps/ goes through the RoadmapWriteLock token — and its planted
// mutants. Registered here for the same reason as the suites above.
// `cargo test --lib -- roadmap_writer_gate` runs them.
#[cfg(test)]
#[path = "../../../tests/roadmap_writer_gate.rs"]
mod roadmap_writer_gate;
#[cfg(test)]
#[path = "../../../tests/roadmap_writer_gate_tests.rs"]
mod roadmap_writer_gate_tests;
// PMAT-1385: `pmat work migrate` under the lock, in whole-file and fragment mode.
// `cargo test --lib -- work_migrate_` runs them.
#[cfg(test)]
#[path = "../../../tests/work_migrate_lock_tests.rs"]
mod work_migrate_lock_tests;
// PMAT-680: `work add` must mint from ONE authority per repository — the git
// common dir's lock plus every ref's roadmap — so two checkouts of the same
// repository cannot mint the same id. Registered here for the same reason as
// its four siblings above: nothing reaches `src/tests/lib.rs`, so a test file
// left there is never compiled and its silence would read as a pass.
// `cargo test --lib -- work_add_single_authority` runs them.
#[cfg(test)]
#[path = "../../../tests/work_add_single_authority_tests.rs"]
mod work_add_single_authority_tests;
// FLOW-03 (#1440): the triage gate and the epic link, against a fake GitHub.
// Registered here for the same reason as its siblings: nothing reaches
// `src/tests/lib.rs`. `cargo test --lib -- work_add_triaged` runs them.
#[cfg(test)]
#[path = "../../../tests/work_add_triaged_tests.rs"]
mod work_add_triaged_tests;

// PMAT-675: the release path a tag takes, pinned as data (see the file header).
#[cfg(test)]
#[path = "../../../tests/release_workflow_tests.rs"]
mod release_workflow_tests;

// PMAT-686: the fleet gate's banned-path scan, in-repo.
#[cfg(test)]
#[path = "../../../tests/fleet_banned_paths_tests.rs"]
mod fleet_banned_paths_tests;

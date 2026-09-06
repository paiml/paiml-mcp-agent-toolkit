// Work handlers - split for file health (CB-040)
#![cfg_attr(coverage_nightly, coverage(off))]

pub mod core_handlers;

// Re-export public handler functions from core_handlers so existing callers
// (e.g. command_dispatcher_work.rs) can use `work_handlers::handle_work_init`, etc.
pub use core_handlers::{
    handle_work_checkpoint, handle_work_complete, handle_work_continue, handle_work_cot_check,
    handle_work_cot_derive, handle_work_delegate, handle_work_event, handle_work_falsify,
    handle_work_init, handle_work_ledger_verify, handle_work_start, handle_work_status,
    handle_work_sync, run_quality_gates, FalsificationResult, GitHubIssueInfo,
};

// Re-export ticket handlers that are used by the command dispatcher
// (handle_work_score lives in ticket_score.rs, included by ticket_handlers.rs)

// Imports needed by ticket_handlers.rs (included below via include!())
// These mirror what was previously available via include!("core_handlers.rs")
use crate::models::roadmap::ItemStatus;
use crate::services::roadmap_service::RoadmapService;
use anyhow::{Context, Result};
use std::path::{Path, PathBuf};

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
// PMAT-676: `work add` and `work edit` must refuse a roadmap `work validate`
// rejects. Registered here for the same reason as its two siblings above —
// `src/tests/lib.rs` reaches nothing, so a test file left there is never
// compiled. `cargo test --lib -- work_add_refuses_invalid` runs them.
#[cfg(test)]
#[path = "../../../tests/work_add_refuses_invalid_tests.rs"]
mod work_add_refuses_invalid_tests;

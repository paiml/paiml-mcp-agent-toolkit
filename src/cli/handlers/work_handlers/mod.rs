// Work handlers - split for file health (CB-040)
#![cfg_attr(coverage_nightly, coverage(off))]

pub mod core_handlers;

// Re-export public handler functions from core_handlers so existing callers
// (e.g. command_dispatcher_work.rs) can use `work_handlers::handle_work_init`, etc.
pub use core_handlers::{
    handle_work_checkpoint, handle_work_complete, handle_work_continue, handle_work_cot_check,
    handle_work_cot_derive, handle_work_delegate, handle_work_event, handle_work_falsify, handle_work_init,
    handle_work_ledger_verify, handle_work_start, handle_work_status, handle_work_sync,
    run_quality_gates, FalsificationResult, GitHubIssueInfo,
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

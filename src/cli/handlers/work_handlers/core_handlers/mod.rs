#![cfg_attr(coverage_nightly, coverage(off))]
// Work command handlers for unified GitHub/YAML workflow (Issue #75)
//
// Implements the hybrid write-through architecture for GitHub and YAML tracking.
// Split into submodules for file health compliance (CB-040).

mod checkpoint;
mod commit;
mod contract;
mod github;
mod handlers;
mod helpers;
mod resolution;
mod types;

// Quality handlers extracted to work_quality_handlers.rs for file health compliance (CB-040)
pub use crate::cli::handlers::work_quality_handlers::{run_quality_gates, FalsificationResult};

// Re-export public types
pub use types::GitHubIssueInfo;

// Re-export public handler functions
pub use handlers::{
    handle_work_checkpoint, handle_work_complete, handle_work_continue, handle_work_cot_check,
    handle_work_cot_derive, handle_work_event, handle_work_falsify, handle_work_init,
    handle_work_start, handle_work_status, handle_work_sync,
};

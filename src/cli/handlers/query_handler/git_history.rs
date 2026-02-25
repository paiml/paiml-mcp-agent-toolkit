//! Git history: annotation builders, formatters, and log parsing.

use super::options::*;
use crate::services::agent_context::AgentContextIndex;
use crate::services::git_history::{
    ChangeType, CommitInfo, FileChange, GitSearchResult,
};
use std::collections::HashMap;

/// Timing breakdown for git history search phases
pub(super) struct GitHistoryProfile {
    pub(super) git_log_ms: u128,
    pub(super) parse_ms: u128,
    pub(super) index_ms: u128,
    pub(super) search_ms: u128,
    pub(super) annotate_ms: u128,
    pub(super) total_ms: u128,
    pub(super) commit_count: usize,
}

// O(1) annotation builders, scoring functions, work ticket/commit quality loaders
include!("git_history_annotations.rs");

// Colorized output formatting for git history results
include!("git_history_formatting.rs");

// Git log parsing (PMAT_START block format) and commit classification
include!("git_history_parsing.rs");

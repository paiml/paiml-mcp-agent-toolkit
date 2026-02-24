#![cfg_attr(coverage_nightly, coverage(off))]
// TIMELINE-003: Recording Comparison Mode
// Sprint 77 - GREEN Phase
//
// Side-by-side comparison of two .pmat recordings with synchronized navigation
// and diff highlighting.

use super::recording::Recording;
use super::timeline_player::TimelinePlayer;
use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Synchronization mode for comparing two recordings
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SyncMode {
    /// Sync by frame number (both recordings advance to same frame index)
    ByFrame,
    /// Sync by relative timestamp (match recordings by timestamp_relative_ms)
    ByTimestamp,
    /// Sync by source location (match recordings by file:line position)
    ByLocation,
}

/// Diff status for a variable comparison
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum DiffStatus {
    /// Values are identical in both recordings
    Same,
    /// Values differ between recordings
    Modified,
    /// Variable exists only in recording B (added)
    Added,
    /// Variable exists only in recording A (removed)
    Removed,
}

/// Side-by-side comparison view for two recordings
pub struct ComparisonView {
    /// TimelinePlayer for recording A
    player_a: TimelinePlayer,
    /// TimelinePlayer for recording B
    player_b: TimelinePlayer,
    /// Current synchronization mode
    sync_mode: SyncMode,
    /// Name/label for recording A
    name_a: String,
    /// Name/label for recording B
    name_b: String,
}

// Constructor, frame accessors, exhaustion checks, sync mode, navigation
include!("comparison_view_core.rs");

// Rendering and variable diff
include!("comparison_view_diff.rs");

// Divergence detection
include!("comparison_view_divergence.rs");

// JSON export of diff reports
include!("comparison_view_export.rs");

#[cfg_attr(coverage_nightly, coverage(off))]
#[cfg(test)]
mod coverage_tests {
    use super::*;
    use crate::services::dap::recording::{Snapshot, StackFrame};

    // Helpers, enum tests, creation tests, navigation tests
    include!("comparison_view_tests.rs");

    // Variable diff, divergence, render split, export JSON tests
    include!("comparison_view_tests_diff.rs");
}

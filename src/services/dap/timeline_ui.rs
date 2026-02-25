#![cfg_attr(coverage_nightly, coverage(off))]
// TIMELINE-002: Timeline UI Integration with TimelinePlayer
// Sprint 77 - GREEN Phase
//
// Terminal-based UI for visualizing execution timeline and navigating snapshots.
// Integrates with TimelinePlayer for recording playback control.

use super::recording::{Snapshot, StackFrame};
use super::timeline_player::TimelinePlayer;
use super::types::ExecutionSnapshot;
use anyhow::Result;
use std::collections::HashMap;

/// Timeline UI for visualizing and navigating execution snapshots
///
/// This struct wraps a TimelinePlayer and provides UI-specific methods
/// for rendering playback state, handling keyboard input, and managing
/// auto-advance playback.
pub struct TimelineUI {
    /// TimelinePlayer managing recording playback state
    player: TimelinePlayer,

    // Legacy fields for backward compatibility with Sprint 73 tests
    /// All snapshots in the recording (legacy)
    #[allow(dead_code)]
    snapshots_legacy: Vec<ExecutionSnapshot>,
    /// Current position in the timeline (legacy)
    #[allow(dead_code)]
    current_position_legacy: usize,
}

// Navigation, playback control, and key handling
include!("timeline_ui_navigation.rs");

// Rendering methods (render, render_details, render_metrics, etc.)
include!("timeline_ui_rendering.rs");

// Tests
include!("timeline_ui_tests.rs");

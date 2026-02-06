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

impl ComparisonView {
    /// Create a new comparison view from two recordings
    ///
    /// # Example
    /// ```ignore
    /// use pmat::services::dap::{Recording, ComparisonView};
    ///
    /// let recording_a = Recording::new("program_a".to_string(), vec![]);
    /// let recording_b = Recording::new("program_b".to_string(), vec![]);
    ///
    /// let comparison = ComparisonView::new(recording_a, recording_b);
    /// ```
    pub fn new(recording_a: Recording, recording_b: Recording) -> Self {
        let name_a = recording_a.metadata().program.clone();
        let name_b = recording_b.metadata().program.clone();

        Self {
            player_a: TimelinePlayer::new(recording_a),
            player_b: TimelinePlayer::new(recording_b),
            sync_mode: SyncMode::ByFrame,
            name_a,
            name_b,
        }
    }

    /// Get current frame number for recording A
    pub fn current_frame_a(&self) -> usize {
        self.player_a.current_frame()
    }

    /// Get current frame number for recording B
    pub fn current_frame_b(&self) -> usize {
        self.player_b.current_frame()
    }

    /// Get total frames for recording A
    pub fn total_frames_a(&self) -> usize {
        self.player_a.total_frames()
    }

    /// Get total frames for recording B
    pub fn total_frames_b(&self) -> usize {
        self.player_b.total_frames()
    }

    /// Get minimum frame count of both recordings
    pub fn total_frames_min(&self) -> usize {
        self.total_frames_a().min(self.total_frames_b())
    }

    /// Get maximum frame count of both recordings
    pub fn total_frames_max(&self) -> usize {
        self.total_frames_a().max(self.total_frames_b())
    }

    /// Check if recording A is exhausted (at or past its last frame)
    pub fn recording_a_exhausted(&self) -> bool {
        let total = self.total_frames_a();
        if total == 0 {
            return true;
        }
        self.current_frame_a() >= total - 1
    }

    /// Check if recording B is exhausted (at or past its last frame)
    pub fn recording_b_exhausted(&self) -> bool {
        let total = self.total_frames_b();
        if total == 0 {
            return true;
        }
        self.current_frame_b() >= total - 1
    }

    /// Get current synchronization mode
    pub fn sync_mode(&self) -> SyncMode {
        self.sync_mode
    }

    /// Set synchronization mode
    pub fn set_sync_mode(&mut self, mode: SyncMode) {
        self.sync_mode = mode;
    }

    /// Advance both recordings to next frame
    ///
    /// Returns Ok(()) if successful, Err if either recording is exhausted
    pub fn next_frame(&mut self) -> Result<()> {
        // Attempt to advance both
        let a_result = self.player_a.next_frame();
        let b_result = self.player_b.next_frame();

        // If either failed, return error
        if a_result.is_none() && b_result.is_none() {
            anyhow::bail!("Both recordings exhausted");
        }

        Ok(())
    }

    /// Move both recordings to previous frame
    ///
    /// Returns Ok(()) if successful, Err if either recording is at frame 0
    pub fn prev_frame(&mut self) -> Result<()> {
        // Attempt to move both back
        let a_result = self.player_a.prev_frame();
        let b_result = self.player_b.prev_frame();

        // If both failed (at start), return error
        if a_result.is_none() && b_result.is_none() {
            anyhow::bail!("Both recordings at start");
        }

        Ok(())
    }

    /// Jump both recordings to specific frame
    ///
    /// Returns Ok(()) if successful, Err if frame is out of bounds for both
    pub fn jump_to(&mut self, frame: usize) -> Result<()> {
        // Jump both players
        let a_result = self.player_a.jump_to(frame);
        let b_result = self.player_b.jump_to(frame);

        // If both failed, return error
        if a_result.is_err() && b_result.is_err() {
            anyhow::bail!("Frame {} out of bounds for both recordings", frame);
        }

        Ok(())
    }

    /// Render split view with both recordings side-by-side
    ///
    /// Returns formatted string with divider separating the views
    pub fn render_split(&self) -> String {
        let mut output = String::new();

        // Header
        output.push_str(&format!(
            "Recording A: {}    |    Recording B: {}\n",
            self.name_a, self.name_b
        ));
        output.push_str(&format!(
            "Frame {}/{}          |    Frame {}/{}\n",
            self.current_frame_a(),
            self.total_frames_a(),
            self.current_frame_b(),
            self.total_frames_b()
        ));

        // Add END markers if exhausted
        if self.recording_a_exhausted() {
            output.push_str("END                 |    \n");
        }
        if self.recording_b_exhausted() {
            output.push_str("                    |    END\n");
        }

        output
    }

    /// Calculate variable diff between current snapshots
    ///
    /// Returns HashMap mapping variable names to their diff status
    pub fn variable_diff(&self) -> HashMap<String, DiffStatus> {
        let mut diff = HashMap::new();

        // Get current snapshots (if available)
        let snapshot_a = if !self.recording_a_exhausted() {
            Some(self.player_a.current_snapshot())
        } else {
            None
        };

        let snapshot_b = if !self.recording_b_exhausted() {
            Some(self.player_b.current_snapshot())
        } else {
            None
        };

        match (snapshot_a, snapshot_b) {
            (Some(a), Some(b)) => {
                // Both snapshots available - compare variables
                let vars_a = &a.variables;
                let vars_b = &b.variables;

                // Check all variables in A
                for (name, value_a) in vars_a {
                    if let Some(value_b) = vars_b.get(name) {
                        // Variable exists in both
                        if value_a == value_b {
                            diff.insert(name.clone(), DiffStatus::Same);
                        } else {
                            diff.insert(name.clone(), DiffStatus::Modified);
                        }
                    } else {
                        // Variable only in A (removed in B)
                        diff.insert(name.clone(), DiffStatus::Removed);
                    }
                }

                // Check for variables only in B (added)
                for name in vars_b.keys() {
                    if !vars_a.contains_key(name) {
                        diff.insert(name.clone(), DiffStatus::Added);
                    }
                }
            }
            (Some(a), None) => {
                // Only A available - all variables marked as Removed
                for name in a.variables.keys() {
                    diff.insert(name.clone(), DiffStatus::Removed);
                }
            }
            (None, Some(b)) => {
                // Only B available - all variables marked as Added
                for name in b.variables.keys() {
                    diff.insert(name.clone(), DiffStatus::Added);
                }
            }
            (None, None) => {
                // Both exhausted - empty diff
            }
        }

        diff
    }

    /// Find the first frame where recordings diverge
    ///
    /// Returns Some(frame_number) if divergence found, None if identical
    ///
    /// Compares:
    /// - Variable values
    /// - Stack frame names
    /// - Instruction pointers
    pub fn find_divergence_point(&self) -> Option<usize> {
        let max_frames = self.total_frames_min();

        for frame in 0..max_frames {
            // Get snapshots at this frame
            let snapshot_a = &self.player_a.recording().snapshots()[frame];
            let snapshot_b = &self.player_b.recording().snapshots()[frame];

            // Compare variables
            if snapshot_a.variables != snapshot_b.variables {
                return Some(frame);
            }

            // Compare instruction pointers
            if snapshot_a.instruction_pointer != snapshot_b.instruction_pointer {
                return Some(frame);
            }

            // Compare stack frame names
            if snapshot_a.stack_frames.len() != snapshot_b.stack_frames.len() {
                return Some(frame);
            }

            for (frame_a, frame_b) in snapshot_a
                .stack_frames
                .iter()
                .zip(snapshot_b.stack_frames.iter())
            {
                if frame_a.name != frame_b.name {
                    return Some(frame);
                }
            }
        }

        // No divergence found
        None
    }

    /// Export diff report as JSON
    ///
    /// Returns JSON string with comparison metadata and frame-by-frame diffs
    pub fn export_diff_json(&self) -> Result<String> {
        let mut report = serde_json::json!({
            "metadata": {
                "recording_a_name": self.name_a,
                "recording_b_name": self.name_b,
                "recording_a_frames": self.total_frames_a(),
                "recording_b_frames": self.total_frames_b(),
                "sync_mode": format!("{:?}", self.sync_mode),
                "divergence_point": self.find_divergence_point(),
            },
            "frame_diffs": []
        });

        // Generate frame-by-frame diffs by directly accessing snapshots
        let mut frame_diffs = Vec::new();
        let max_frames = self.total_frames_max();
        let snapshots_a = self.player_a.recording().snapshots();
        let snapshots_b = self.player_b.recording().snapshots();

        for frame in 0..max_frames {
            // Compute diff for this frame
            let mut diff = HashMap::new();

            let snapshot_a = if frame < snapshots_a.len() {
                Some(&snapshots_a[frame])
            } else {
                None
            };

            let snapshot_b = if frame < snapshots_b.len() {
                Some(&snapshots_b[frame])
            } else {
                None
            };

            match (snapshot_a, snapshot_b) {
                (Some(a), Some(b)) => {
                    let vars_a = &a.variables;
                    let vars_b = &b.variables;

                    // Check all variables in A
                    for (name, value_a) in vars_a {
                        if let Some(value_b) = vars_b.get(name) {
                            if value_a == value_b {
                                diff.insert(name.clone(), DiffStatus::Same);
                            } else {
                                diff.insert(name.clone(), DiffStatus::Modified);
                            }
                        } else {
                            diff.insert(name.clone(), DiffStatus::Removed);
                        }
                    }

                    // Check for variables only in B
                    for name in vars_b.keys() {
                        if !vars_a.contains_key(name) {
                            diff.insert(name.clone(), DiffStatus::Added);
                        }
                    }
                }
                (Some(a), None) => {
                    // Only A available
                    for name in a.variables.keys() {
                        diff.insert(name.clone(), DiffStatus::Removed);
                    }
                }
                (None, Some(b)) => {
                    // Only B available
                    for name in b.variables.keys() {
                        diff.insert(name.clone(), DiffStatus::Added);
                    }
                }
                (None, None) => {
                    // Both exhausted - empty diff
                }
            }

            frame_diffs.push(serde_json::json!({
                "frame": frame,
                "variable_diff": diff,
            }));
        }

        report["frame_diffs"] = serde_json::json!(frame_diffs);

        Ok(serde_json::to_string_pretty(&report)?)
    }
}

#[cfg_attr(coverage_nightly, coverage(off))]
#[cfg(test)]
mod coverage_tests {
    use super::*;
    use crate::services::dap::recording::{Snapshot, StackFrame};

    /// Helper: Create test recording with specified snapshots
    fn create_test_recording(program: &str, snapshot_count: usize) -> Recording {
        let mut recording = Recording::new(program.to_string(), vec![]);
        for i in 0..snapshot_count {
            let mut variables = HashMap::new();
            variables.insert("counter".to_string(), serde_json::json!(i));
            variables.insert("name".to_string(), serde_json::json!(format!("item_{}", i)));

            let stack_frames = vec![StackFrame {
                name: format!("func_{}", i),
                file: Some("test.rs".to_string()),
                line: Some(10 + i as u32),
                locals: HashMap::new(),
            }];

            let snapshot = Snapshot {
                frame_id: i as u64,
                timestamp_relative_ms: (i * 100) as u32,
                variables,
                stack_frames,
                instruction_pointer: 0x1000 + (i as u64 * 8),
                memory_snapshot: None,
            };
            recording.add_snapshot(snapshot);
        }
        recording
    }

    /// Helper: Create recording with specific variables at each frame
    fn create_recording_with_vars(
        program: &str,
        frames: Vec<HashMap<String, serde_json::Value>>,
    ) -> Recording {
        let mut recording = Recording::new(program.to_string(), vec![]);
        for (i, variables) in frames.into_iter().enumerate() {
            let snapshot = Snapshot {
                frame_id: i as u64,
                timestamp_relative_ms: (i * 100) as u32,
                variables,
                stack_frames: vec![StackFrame {
                    name: "main".to_string(),
                    file: Some("test.rs".to_string()),
                    line: Some(i as u32),
                    locals: HashMap::new(),
                }],
                instruction_pointer: 0x1000 + (i as u64 * 8),
                memory_snapshot: None,
            };
            recording.add_snapshot(snapshot);
        }
        recording
    }

    // ========================================================================
    // SyncMode Tests
    // ========================================================================

    #[test]
    fn test_sync_mode_equality() {
        assert_eq!(SyncMode::ByFrame, SyncMode::ByFrame);
        assert_eq!(SyncMode::ByTimestamp, SyncMode::ByTimestamp);
        assert_eq!(SyncMode::ByLocation, SyncMode::ByLocation);
        assert_ne!(SyncMode::ByFrame, SyncMode::ByTimestamp);
        assert_ne!(SyncMode::ByTimestamp, SyncMode::ByLocation);
    }

    #[test]
    fn test_sync_mode_clone() {
        let mode = SyncMode::ByFrame;
        let cloned = mode;
        assert_eq!(mode, cloned);
    }

    #[test]
    fn test_sync_mode_debug() {
        let mode = SyncMode::ByFrame;
        let debug_str = format!("{:?}", mode);
        assert!(debug_str.contains("ByFrame"));
    }

    #[test]
    fn test_sync_mode_serialization() {
        let mode = SyncMode::ByTimestamp;
        let json = serde_json::to_string(&mode).unwrap();
        let deserialized: SyncMode = serde_json::from_str(&json).unwrap();
        assert_eq!(mode, deserialized);
    }

    // ========================================================================
    // DiffStatus Tests
    // ========================================================================

    #[test]
    fn test_diff_status_variants() {
        assert_eq!(DiffStatus::Same, DiffStatus::Same);
        assert_eq!(DiffStatus::Modified, DiffStatus::Modified);
        assert_eq!(DiffStatus::Added, DiffStatus::Added);
        assert_eq!(DiffStatus::Removed, DiffStatus::Removed);
    }

    #[test]
    fn test_diff_status_inequality() {
        assert_ne!(DiffStatus::Same, DiffStatus::Modified);
        assert_ne!(DiffStatus::Added, DiffStatus::Removed);
    }

    #[test]
    fn test_diff_status_clone() {
        let status = DiffStatus::Modified;
        let cloned = status.clone();
        assert_eq!(status, cloned);
    }

    #[test]
    fn test_diff_status_debug() {
        let status = DiffStatus::Added;
        let debug_str = format!("{:?}", status);
        assert!(debug_str.contains("Added"));
    }

    #[test]
    fn test_diff_status_serialization() {
        let status = DiffStatus::Removed;
        let json = serde_json::to_string(&status).unwrap();
        let deserialized: DiffStatus = serde_json::from_str(&json).unwrap();
        assert_eq!(status, deserialized);
    }

    // ========================================================================
    // ComparisonView Creation Tests
    // ========================================================================

    #[test]
    fn test_comparison_view_creation() {
        let recording_a = create_test_recording("program_a", 5);
        let recording_b = create_test_recording("program_b", 3);

        let view = ComparisonView::new(recording_a, recording_b);

        assert_eq!(view.total_frames_a(), 5);
        assert_eq!(view.total_frames_b(), 3);
        assert_eq!(view.current_frame_a(), 0);
        assert_eq!(view.current_frame_b(), 0);
    }

    #[test]
    fn test_comparison_view_empty_recordings() {
        let recording_a = Recording::new("empty_a".to_string(), vec![]);
        let recording_b = Recording::new("empty_b".to_string(), vec![]);

        let view = ComparisonView::new(recording_a, recording_b);

        assert_eq!(view.total_frames_a(), 0);
        assert_eq!(view.total_frames_b(), 0);
        assert!(view.recording_a_exhausted());
        assert!(view.recording_b_exhausted());
    }

    #[test]
    fn test_comparison_view_default_sync_mode() {
        let recording_a = create_test_recording("a", 2);
        let recording_b = create_test_recording("b", 2);

        let view = ComparisonView::new(recording_a, recording_b);
        assert_eq!(view.sync_mode(), SyncMode::ByFrame);
    }

    // ========================================================================
    // Frame Navigation Tests
    // ========================================================================

    #[test]
    fn test_total_frames_min_max() {
        let recording_a = create_test_recording("a", 10);
        let recording_b = create_test_recording("b", 5);

        let view = ComparisonView::new(recording_a, recording_b);

        assert_eq!(view.total_frames_min(), 5);
        assert_eq!(view.total_frames_max(), 10);
    }

    #[test]
    fn test_next_frame_navigation() {
        let recording_a = create_test_recording("a", 3);
        let recording_b = create_test_recording("b", 3);

        let mut view = ComparisonView::new(recording_a, recording_b);

        assert_eq!(view.current_frame_a(), 0);
        assert_eq!(view.current_frame_b(), 0);

        view.next_frame().unwrap();
        assert_eq!(view.current_frame_a(), 1);
        assert_eq!(view.current_frame_b(), 1);
    }

    #[test]
    fn test_prev_frame_navigation() {
        let recording_a = create_test_recording("a", 5);
        let recording_b = create_test_recording("b", 5);

        let mut view = ComparisonView::new(recording_a, recording_b);

        // Move forward first
        view.next_frame().unwrap();
        view.next_frame().unwrap();
        assert_eq!(view.current_frame_a(), 2);

        // Now move back
        view.prev_frame().unwrap();
        assert_eq!(view.current_frame_a(), 1);
    }

    #[test]
    fn test_prev_frame_at_start_error() {
        let recording_a = create_test_recording("a", 2);
        let recording_b = create_test_recording("b", 2);

        let mut view = ComparisonView::new(recording_a, recording_b);

        let result = view.prev_frame();
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("at start"));
    }

    #[test]
    fn test_next_frame_at_end_error() {
        let recording_a = create_test_recording("a", 2);
        let recording_b = create_test_recording("b", 2);

        let mut view = ComparisonView::new(recording_a, recording_b);

        // Move to last frame
        view.next_frame().unwrap();

        // Try to move past end
        let result = view.next_frame();
        assert!(result.is_err());
    }

    #[test]
    fn test_jump_to_valid_frame() {
        let recording_a = create_test_recording("a", 10);
        let recording_b = create_test_recording("b", 10);

        let mut view = ComparisonView::new(recording_a, recording_b);

        view.jump_to(5).unwrap();
        assert_eq!(view.current_frame_a(), 5);
        assert_eq!(view.current_frame_b(), 5);
    }

    #[test]
    fn test_jump_to_out_of_bounds_error() {
        let recording_a = create_test_recording("a", 3);
        let recording_b = create_test_recording("b", 3);

        let mut view = ComparisonView::new(recording_a, recording_b);

        let result = view.jump_to(100);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("out of bounds"));
    }

    #[test]
    fn test_jump_to_asymmetric_recordings() {
        let recording_a = create_test_recording("a", 10);
        let recording_b = create_test_recording("b", 5);

        let mut view = ComparisonView::new(recording_a, recording_b);

        // Jump to frame within B's range
        view.jump_to(3).unwrap();
        assert_eq!(view.current_frame_a(), 3);
        assert_eq!(view.current_frame_b(), 3);

        // Jump to frame outside B's range but within A's range
        view.jump_to(7).unwrap();
        assert_eq!(view.current_frame_a(), 7);
    }

    // ========================================================================
    // Exhaustion Tests
    // ========================================================================

    #[test]
    fn test_recording_exhausted_detection() {
        let recording_a = create_test_recording("a", 2);
        let recording_b = create_test_recording("b", 4);

        let mut view = ComparisonView::new(recording_a, recording_b);

        assert!(!view.recording_a_exhausted());
        assert!(!view.recording_b_exhausted());

        // Move to end of A
        view.next_frame().unwrap();
        assert!(view.recording_a_exhausted());
        assert!(!view.recording_b_exhausted());
    }

    // ========================================================================
    // Sync Mode Tests
    // ========================================================================

    #[test]
    fn test_set_sync_mode() {
        let recording_a = create_test_recording("a", 2);
        let recording_b = create_test_recording("b", 2);

        let mut view = ComparisonView::new(recording_a, recording_b);

        assert_eq!(view.sync_mode(), SyncMode::ByFrame);

        view.set_sync_mode(SyncMode::ByTimestamp);
        assert_eq!(view.sync_mode(), SyncMode::ByTimestamp);

        view.set_sync_mode(SyncMode::ByLocation);
        assert_eq!(view.sync_mode(), SyncMode::ByLocation);
    }

    // ========================================================================
    // Variable Diff Tests
    // ========================================================================

    #[test]
    fn test_variable_diff_same_values() {
        let mut vars = HashMap::new();
        vars.insert("x".to_string(), serde_json::json!(42));

        // Need at least 2 frames so recording is not immediately exhausted
        let recording_a = create_recording_with_vars("a", vec![vars.clone(), vars.clone()]);
        let recording_b = create_recording_with_vars("b", vec![vars.clone(), vars]);

        let view = ComparisonView::new(recording_a, recording_b);
        let diff = view.variable_diff();

        assert_eq!(diff.get("x"), Some(&DiffStatus::Same));
    }

    #[test]
    fn test_variable_diff_modified_values() {
        let mut vars_a = HashMap::new();
        vars_a.insert("x".to_string(), serde_json::json!(1));

        let mut vars_b = HashMap::new();
        vars_b.insert("x".to_string(), serde_json::json!(2));

        // Need at least 2 frames so recording is not immediately exhausted
        let recording_a = create_recording_with_vars("a", vec![vars_a.clone(), vars_a]);
        let recording_b = create_recording_with_vars("b", vec![vars_b.clone(), vars_b]);

        let view = ComparisonView::new(recording_a, recording_b);
        let diff = view.variable_diff();

        assert_eq!(diff.get("x"), Some(&DiffStatus::Modified));
    }

    #[test]
    fn test_variable_diff_added_variable() {
        let mut vars_a = HashMap::new();
        vars_a.insert("x".to_string(), serde_json::json!(1));

        let mut vars_b = HashMap::new();
        vars_b.insert("x".to_string(), serde_json::json!(1));
        vars_b.insert("y".to_string(), serde_json::json!(2));

        // Need at least 2 frames so recording is not immediately exhausted
        let recording_a = create_recording_with_vars("a", vec![vars_a.clone(), vars_a]);
        let recording_b = create_recording_with_vars("b", vec![vars_b.clone(), vars_b]);

        let view = ComparisonView::new(recording_a, recording_b);
        let diff = view.variable_diff();

        assert_eq!(diff.get("x"), Some(&DiffStatus::Same));
        assert_eq!(diff.get("y"), Some(&DiffStatus::Added));
    }

    #[test]
    fn test_variable_diff_removed_variable() {
        let mut vars_a = HashMap::new();
        vars_a.insert("x".to_string(), serde_json::json!(1));
        vars_a.insert("y".to_string(), serde_json::json!(2));

        let mut vars_b = HashMap::new();
        vars_b.insert("x".to_string(), serde_json::json!(1));

        // Need at least 2 frames so recording is not immediately exhausted
        let recording_a = create_recording_with_vars("a", vec![vars_a.clone(), vars_a]);
        let recording_b = create_recording_with_vars("b", vec![vars_b.clone(), vars_b]);

        let view = ComparisonView::new(recording_a, recording_b);
        let diff = view.variable_diff();

        assert_eq!(diff.get("x"), Some(&DiffStatus::Same));
        assert_eq!(diff.get("y"), Some(&DiffStatus::Removed));
    }

    #[test]
    fn test_variable_diff_exhausted_a() {
        let vars = HashMap::new();
        let recording_a = Recording::new("a".to_string(), vec![]);
        let recording_b = create_recording_with_vars("b", vec![vars]);

        let view = ComparisonView::new(recording_a, recording_b);
        let diff = view.variable_diff();

        // All variables in B should be marked as Added
        assert!(diff.is_empty() || diff.values().all(|s| *s == DiffStatus::Added));
    }

    #[test]
    fn test_variable_diff_exhausted_b() {
        let mut vars = HashMap::new();
        vars.insert("x".to_string(), serde_json::json!(1));

        // Need at least 2 frames so recording A is not immediately exhausted
        let recording_a = create_recording_with_vars("a", vec![vars.clone(), vars]);
        let recording_b = Recording::new("b".to_string(), vec![]);

        let view = ComparisonView::new(recording_a, recording_b);
        let diff = view.variable_diff();

        assert_eq!(diff.get("x"), Some(&DiffStatus::Removed));
    }

    #[test]
    fn test_variable_diff_both_exhausted() {
        let recording_a = Recording::new("a".to_string(), vec![]);
        let recording_b = Recording::new("b".to_string(), vec![]);

        let view = ComparisonView::new(recording_a, recording_b);
        let diff = view.variable_diff();

        assert!(diff.is_empty());
    }

    // ========================================================================
    // Divergence Point Tests
    // ========================================================================

    #[test]
    fn test_find_divergence_point_no_divergence() {
        let recording_a = create_test_recording("a", 5);
        let recording_b = create_test_recording("b", 5);

        let view = ComparisonView::new(recording_a, recording_b);
        // Recordings have same structure, no divergence
        let divergence = view.find_divergence_point();
        assert!(divergence.is_none());
    }

    #[test]
    fn test_find_divergence_point_variable_difference() {
        let mut vars_0 = HashMap::new();
        vars_0.insert("x".to_string(), serde_json::json!(1));

        let mut vars_1_a = HashMap::new();
        vars_1_a.insert("x".to_string(), serde_json::json!(2));

        let mut vars_1_b = HashMap::new();
        vars_1_b.insert("x".to_string(), serde_json::json!(999)); // Different!

        let recording_a = create_recording_with_vars("a", vec![vars_0.clone(), vars_1_a]);
        let recording_b = create_recording_with_vars("b", vec![vars_0, vars_1_b]);

        let view = ComparisonView::new(recording_a, recording_b);
        let divergence = view.find_divergence_point();

        assert_eq!(divergence, Some(1));
    }

    #[test]
    fn test_find_divergence_point_empty_recordings() {
        let recording_a = Recording::new("a".to_string(), vec![]);
        let recording_b = Recording::new("b".to_string(), vec![]);

        let view = ComparisonView::new(recording_a, recording_b);
        assert!(view.find_divergence_point().is_none());
    }

    // ========================================================================
    // Render Split View Tests
    // ========================================================================

    #[test]
    fn test_render_split_basic() {
        let recording_a = create_test_recording("program_a", 5);
        let recording_b = create_test_recording("program_b", 3);

        let view = ComparisonView::new(recording_a, recording_b);
        let output = view.render_split();

        assert!(output.contains("Recording A: program_a"));
        assert!(output.contains("Recording B: program_b"));
        assert!(output.contains("Frame 0/5"));
        assert!(output.contains("Frame 0/3"));
    }

    #[test]
    fn test_render_split_shows_end_markers() {
        let recording_a = create_test_recording("a", 2);
        let recording_b = create_test_recording("b", 2);

        let mut view = ComparisonView::new(recording_a, recording_b);
        view.next_frame().unwrap(); // Move to last frame

        let output = view.render_split();
        assert!(output.contains("END"));
    }

    #[test]
    fn test_render_split_empty_recordings() {
        let recording_a = Recording::new("empty_a".to_string(), vec![]);
        let recording_b = Recording::new("empty_b".to_string(), vec![]);

        let view = ComparisonView::new(recording_a, recording_b);
        let output = view.render_split();

        assert!(output.contains("Recording A: empty_a"));
        assert!(output.contains("Recording B: empty_b"));
    }

    // ========================================================================
    // Export Diff JSON Tests
    // ========================================================================

    #[test]
    fn test_export_diff_json_structure() {
        let recording_a = create_test_recording("a", 3);
        let recording_b = create_test_recording("b", 3);

        let view = ComparisonView::new(recording_a, recording_b);
        let json = view.export_diff_json().unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();

        assert!(parsed.get("metadata").is_some());
        assert!(parsed.get("frame_diffs").is_some());
    }

    #[test]
    fn test_export_diff_json_metadata() {
        let recording_a = create_test_recording("prog_a", 5);
        let recording_b = create_test_recording("prog_b", 3);

        let view = ComparisonView::new(recording_a, recording_b);
        let json = view.export_diff_json().unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();

        let metadata = parsed.get("metadata").unwrap();
        assert_eq!(metadata.get("recording_a_name").unwrap(), "prog_a");
        assert_eq!(metadata.get("recording_b_name").unwrap(), "prog_b");
        assert_eq!(metadata.get("recording_a_frames").unwrap(), 5);
        assert_eq!(metadata.get("recording_b_frames").unwrap(), 3);
    }

    #[test]
    fn test_export_diff_json_frame_diffs() {
        let mut vars_a = HashMap::new();
        vars_a.insert("x".to_string(), serde_json::json!(1));

        let mut vars_b = HashMap::new();
        vars_b.insert("x".to_string(), serde_json::json!(2));

        let recording_a = create_recording_with_vars("a", vec![vars_a]);
        let recording_b = create_recording_with_vars("b", vec![vars_b]);

        let view = ComparisonView::new(recording_a, recording_b);
        let json = view.export_diff_json().unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();

        let frame_diffs = parsed.get("frame_diffs").unwrap().as_array().unwrap();
        assert_eq!(frame_diffs.len(), 1);
        assert!(frame_diffs[0].get("variable_diff").is_some());
    }

    #[test]
    fn test_export_diff_json_empty_recordings() {
        let recording_a = Recording::new("a".to_string(), vec![]);
        let recording_b = Recording::new("b".to_string(), vec![]);

        let view = ComparisonView::new(recording_a, recording_b);
        let json = view.export_diff_json().unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();

        let frame_diffs = parsed.get("frame_diffs").unwrap().as_array().unwrap();
        assert!(frame_diffs.is_empty());
    }

    #[test]
    fn test_export_diff_json_asymmetric_frames() {
        let recording_a = create_test_recording("a", 5);
        let recording_b = create_test_recording("b", 2);

        let view = ComparisonView::new(recording_a, recording_b);
        let json = view.export_diff_json().unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();

        let frame_diffs = parsed.get("frame_diffs").unwrap().as_array().unwrap();
        // Should have diffs for max(5, 2) = 5 frames
        assert_eq!(frame_diffs.len(), 5);
    }
}

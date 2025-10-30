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

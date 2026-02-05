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

impl TimelineUI {
    /// Create a new timeline UI from a TimelinePlayer
    ///
    /// This is the primary constructor for Sprint 77+ integration.
    ///
    /// # Example
    /// ```ignore
    /// use pmat::services::dap::{Recording, TimelinePlayer, TimelineUI};
    ///
    /// let recording = Recording::new("program".to_string(), vec![]);
    /// let player = TimelinePlayer::new(recording);
    /// let ui = TimelineUI::from_player(player);
    /// ```
    pub fn from_player(player: TimelinePlayer) -> Self {
        Self {
            player,
            snapshots_legacy: Vec::new(),
            current_position_legacy: 0,
        }
    }

    /// Create a new timeline UI from snapshots (legacy Sprint 73 API)
    ///
    /// This method is kept for backward compatibility with Sprint 73 tests.
    pub fn new(snapshots: Vec<ExecutionSnapshot>) -> Self {
        // Create recording and populate with converted snapshots
        let mut recording = super::recording::Recording::new("legacy".to_string(), vec![]);

        for exec_snap in &snapshots {
            // Convert ExecutionSnapshot to Snapshot (Sprint 72 → Sprint 75)
            let stack_frames = exec_snap
                .call_stack
                .iter()
                .map(|frame| {
                    let file = frame.source.as_ref().and_then(|s| s.path.clone());
                    let line = if frame.line >= 0 {
                        Some(frame.line as u32)
                    } else {
                        None
                    };

                    StackFrame {
                        name: frame.name.clone(),
                        file,
                        line,
                        locals: HashMap::new(),
                    }
                })
                .collect();

            let snapshot = Snapshot {
                frame_id: exec_snap.sequence as u64,
                timestamp_relative_ms: (exec_snap.timestamp / 1_000_000) as u32,
                variables: exec_snap.variables.clone(),
                stack_frames,
                instruction_pointer: 0,
                memory_snapshot: None,
            };

            recording.add_snapshot(snapshot);
        }

        let player = TimelinePlayer::new(recording);

        Self {
            player,
            snapshots_legacy: snapshots,
            current_position_legacy: 0,
        }
    }

    /// Get current frame number
    pub fn current_frame(&self) -> usize {
        self.player.current_frame()
    }

    /// Get current position in the timeline (legacy API)
    pub fn current_position(&self) -> usize {
        // For legacy compatibility
        if !self.snapshots_legacy.is_empty() {
            self.current_position_legacy
        } else {
            self.player.current_frame()
        }
    }

    /// Get progress text: "Frame X/Y"
    pub fn progress_text(&self) -> String {
        format!(
            "Frame {}/{}",
            self.player.current_frame(),
            self.player.total_frames()
        )
    }

    /// Get current snapshot variables
    pub fn current_variables(&self) -> &HashMap<String, serde_json::Value> {
        &self.player.current_snapshot().variables
    }

    /// Get current snapshot stack frames
    pub fn current_stack_frames(&self) -> &[StackFrame] {
        &self.player.current_snapshot().stack_frames
    }

    /// Get frame info: "Frame X/Y | Timestamp | Location"
    pub fn frame_info(&self) -> String {
        let snapshot = self.player.current_snapshot();

        // Extract location from first stack frame (if available)
        let location = if let Some(frame) = snapshot.stack_frames.first() {
            if let (Some(file), Some(line)) = (&frame.file, frame.line) {
                format!("{}:{}", file, line)
            } else {
                "<unknown>".to_string()
            }
        } else {
            "<unknown>".to_string()
        };

        format!(
            "Frame {}/{} | {}ms | {}",
            self.player.current_frame(),
            self.player.total_frames(),
            snapshot.timestamp_relative_ms,
            location
        )
    }

    /// Check if playback is active
    pub fn is_playing(&self) -> bool {
        self.player.is_playing()
    }

    /// Start playback
    pub fn play(&mut self) {
        self.player.play();
    }

    /// Pause playback
    pub fn pause(&mut self) {
        self.player.pause();
    }

    /// Advance to next frame
    pub fn next_frame(&mut self) -> Result<&Snapshot> {
        self.player
            .next_frame()
            .ok_or_else(|| anyhow::anyhow!("Cannot advance: already at last frame"))
    }

    /// Move to previous frame
    pub fn prev_frame(&mut self) -> Result<&Snapshot> {
        self.player
            .prev_frame()
            .ok_or_else(|| anyhow::anyhow!("Cannot move back: already at first frame"))
    }

    /// Jump to specific frame
    ///
    /// Returns a reference to the snapshot at the target frame.
    /// Now works correctly in both legacy and modern modes (Sprint 77+).
    pub fn jump_to(&mut self, frame: usize) -> Result<&Snapshot> {
        if !self.snapshots_legacy.is_empty() {
            // Legacy mode - validate against legacy snapshot count and sync state
            if frame >= self.snapshots_legacy.len() {
                return Err(anyhow::anyhow!(
                    "Frame {} out of bounds (max: {})",
                    frame,
                    self.snapshots_legacy.len() - 1
                ));
            }
            self.current_position_legacy = frame;
        }

        // Jump in player (works for both legacy and modern modes now)
        self.player.jump_to(frame)
    }

    /// Tick for auto-advance playback
    ///
    /// Call this method periodically (e.g., from UI event loop) to auto-advance
    /// frames when playback is active.
    pub fn tick(&mut self) {
        if self.player.is_playing() {
            // Attempt to advance, ignore errors (e.g., end of recording)
            let _ = self.player.next_frame();
        }
    }

    /// Handle keyboard input for navigation
    ///
    /// Supported keys:
    /// - '→' (right arrow): Advance to next frame
    /// - '←' (left arrow): Move to previous frame
    /// - ' ' (space): Toggle play/pause
    /// - 'j' or 'J': Jump mode (handled by caller)
    ///
    /// Returns Ok(()) on success, Err on invalid key or navigation error.
    pub fn handle_key(&mut self, key: char) -> Result<()> {
        // Check for legacy mode first
        if !self.snapshots_legacy.is_empty() {
            return self.handle_key_legacy(key);
        }

        match key {
            '→' => {
                // Step forward
                self.next_frame()
                    .map(|_| ())
                    .map_err(|e| anyhow::anyhow!("{}", e))
            }
            '←' => {
                // Step backward
                self.prev_frame()
                    .map(|_| ())
                    .map_err(|e| anyhow::anyhow!("{}", e))
            }
            ' ' => {
                // Toggle play/pause
                if self.is_playing() {
                    self.pause();
                } else {
                    self.play();
                }
                Ok(())
            }
            'j' | 'J' => {
                // Jump mode - actual jump logic handled by caller
                // This just validates that the key is recognized
                Ok(())
            }
            _ => Err(anyhow::anyhow!("Unknown key: '{}'", key)),
        }
    }

    /// Handle keyboard input for legacy mode
    fn handle_key_legacy(&mut self, key: char) -> Result<()> {
        match key {
            '→' => {
                // Step forward
                if self.current_position_legacy >= self.snapshots_legacy.len() - 1 {
                    return Err(anyhow::anyhow!(
                        "Cannot step forward: already at last snapshot"
                    ));
                }
                self.current_position_legacy += 1;
                Ok(())
            }
            '←' => {
                // Step backward
                if self.current_position_legacy == 0 {
                    return Err(anyhow::anyhow!(
                        "Cannot step backward: already at first snapshot"
                    ));
                }
                self.current_position_legacy -= 1;
                Ok(())
            }
            _ => Err(anyhow::anyhow!("Unknown key: '{}'", key)),
        }
    }

    // ========================================================================
    // Legacy Sprint 73 Methods (for backward compatibility)
    // ========================================================================

    /// Render the timeline as a string (legacy)
    pub fn render(&self) -> String {
        let total_frames = if !self.snapshots_legacy.is_empty() {
            self.snapshots_legacy.len()
        } else {
            self.player.total_frames()
        };

        let current_pos = if !self.snapshots_legacy.is_empty() {
            self.current_position_legacy
        } else {
            self.player.current_frame()
        };

        if total_frames == 0 {
            return "Empty recording".to_string();
        }

        let mut output = String::new();

        // Build timeline representation
        // For small recordings (<= 10), show each position
        // For larger recordings, show ranges
        if total_frames <= 10 {
            // Show individual positions: 0─────1─────2─────3...
            for i in 0..total_frames {
                if i > 0 {
                    output.push_str("─────");
                }
                output.push_str(&i.to_string());
            }
            output.push('\n');

            // Add position indicator
            let indicator_pos = current_pos * 6; // 6 chars per position (including separators)
            output.push_str(&" ".repeat(indicator_pos));
            output.push('^');
        } else {
            // For larger recordings, show compressed timeline
            output.push_str(&format!(
                "Timeline: 0 ──────── {} (Total: {} snapshots)",
                total_frames - 1,
                total_frames
            ));
            output.push('\n');
            output.push_str(&format!("Position: {} ^", current_pos));
        }

        output
    }

    /// Render detailed information about current snapshot (legacy)
    pub fn render_details(&self) -> String {
        // Legacy mode
        if !self.snapshots_legacy.is_empty() {
            if self.snapshots_legacy.is_empty() {
                return "No snapshots available".to_string();
            }

            let snapshot = &self.snapshots_legacy[self.current_position_legacy];
            let mut details = String::new();

            // Snapshot header
            details.push_str(&format!("=== Snapshot #{} ===\n", snapshot.sequence));
            details.push('\n');

            // Variables section
            details.push_str("Variables:\n");
            if snapshot.variables.is_empty() {
                details.push_str("  (none)\n");
            } else {
                for (name, value) in &snapshot.variables {
                    details.push_str(&format!("  {} = {}\n", name, value));
                }
            }
            details.push('\n');

            // Call stack section
            details.push_str("Call Stack:\n");
            for (i, frame) in snapshot.call_stack.iter().enumerate() {
                details.push_str(&format!(
                    "  #{} {} ({}:{})\n",
                    i,
                    frame.name,
                    frame
                        .source
                        .as_ref()
                        .and_then(|s| s.name.as_ref())
                        .unwrap_or(&"<unknown>".to_string()),
                    frame.line
                ));
            }
            details.push('\n');

            // Location section
            details.push_str("Location:\n");
            details.push_str(&format!(
                "  File: {}:{}\n",
                snapshot.location.file, snapshot.location.line
            ));

            return details;
        }

        // New mode - use TimelinePlayer
        if self.player.total_frames() == 0 {
            return "No snapshots available".to_string();
        }

        let snapshot = self.player.current_snapshot();
        let mut details = String::new();

        // Snapshot header
        details.push_str(&format!("=== Snapshot #{} ===\n", snapshot.frame_id));
        details.push('\n');

        // Variables section
        details.push_str("Variables:\n");
        if snapshot.variables.is_empty() {
            details.push_str("  (none)\n");
        } else {
            for (name, value) in &snapshot.variables {
                details.push_str(&format!("  {} = {}\n", name, value));
            }
        }
        details.push('\n');

        // Call stack section
        details.push_str("Call Stack:\n");
        for (i, frame) in snapshot.stack_frames.iter().enumerate() {
            let file = frame.file.as_deref().unwrap_or("<unknown>");
            let line = frame.line.unwrap_or(0);
            details.push_str(&format!("  #{} {} ({}:{})\n", i, frame.name, file, line));
        }
        details.push('\n');

        // Location section (from first stack frame)
        if let Some(frame) = snapshot.stack_frames.first() {
            details.push_str("Location:\n");
            details.push_str(&format!(
                "  File: {}:{}\n",
                frame.file.as_deref().unwrap_or("<unknown>"),
                frame.line.unwrap_or(0)
            ));
        }

        details
    }

    /// Render performance metrics (legacy)
    pub fn render_metrics(&self) -> String {
        let mut metrics = String::new();

        metrics.push_str("=== Recording Metrics ===\n");
        metrics.push('\n');

        let total_snapshots = if !self.snapshots_legacy.is_empty() {
            self.snapshots_legacy.len()
        } else {
            self.player.total_frames()
        };

        // Total snapshots
        metrics.push_str(&format!("Total snapshots: {}\n", total_snapshots));

        // Estimate recording size
        let estimated_size = self.estimate_size_bytes();
        metrics.push_str(&format!(
            "Estimated size: {} bytes ({:.2} KB)\n",
            estimated_size,
            estimated_size as f64 / 1024.0
        ));

        // Count snapshots with deltas (legacy only)
        if !self.snapshots_legacy.is_empty() {
            let delta_count = self
                .snapshots_legacy
                .iter()
                .filter(|s| s.delta.is_some())
                .count();

            if delta_count > 0 {
                let compression_ratio =
                    (delta_count as f64 / self.snapshots_legacy.len() as f64) * 100.0;
                metrics.push_str(&format!("Compression ratio: {:.1}%\n", compression_ratio));
            }
        }

        metrics
    }

    /// Render timeline with specific width (legacy)
    pub fn render_with_width(&self, width: usize) -> String {
        let total_frames = if !self.snapshots_legacy.is_empty() {
            self.snapshots_legacy.len()
        } else {
            self.player.total_frames()
        };

        let current_pos = if !self.snapshots_legacy.is_empty() {
            self.current_position_legacy
        } else {
            self.player.current_frame()
        };

        if total_frames == 0 {
            return "Empty recording".to_string();
        }

        let mut output = String::new();

        // Adjust rendering based on available width
        if width < 40 {
            // Very narrow - compact format
            output.push_str(&format!("[{}/{}]", current_pos, total_frames - 1));
        } else if width < 80 {
            // Medium - abbreviated format
            output.push_str(&format!("Pos: {}/{} ", current_pos, total_frames - 1));
            let available = width.saturating_sub(20);
            let bar_width = available.min(30);
            let fill_ratio = current_pos as f64 / (total_frames - 1) as f64;
            let filled = (bar_width as f64 * fill_ratio) as usize;
            output.push('[');
            output.push_str(&"=".repeat(filled));
            output.push('>');
            output.push_str(&" ".repeat(bar_width.saturating_sub(filled + 1)));
            output.push(']');
        } else {
            // Wide - full format
            output = self.render();
        }

        output
    }

    /// Render timeline with ANSI colors (legacy)
    pub fn render_colored(&self) -> String {
        let total_frames = if !self.snapshots_legacy.is_empty() {
            self.snapshots_legacy.len()
        } else {
            self.player.total_frames()
        };

        let current_pos = if !self.snapshots_legacy.is_empty() {
            self.current_position_legacy
        } else {
            self.player.current_frame()
        };

        if total_frames == 0 {
            return "\x1b[31mEmpty recording\x1b[0m".to_string();
        }

        let mut output = String::new();

        // Build colored timeline
        if total_frames <= 10 {
            for i in 0..total_frames {
                if i > 0 {
                    output.push_str("\x1b[90m─────\x1b[0m"); // Dark gray separators
                }

                if i == current_pos {
                    // Highlight current position in cyan
                    output.push_str(&format!("\x1b[36;1m{}\x1b[0m", i));
                } else {
                    // Other positions in white
                    output.push_str(&i.to_string());
                }
            }
            output.push('\n');

            // Add green position indicator
            let indicator_pos = current_pos * 6;
            output.push_str(&" ".repeat(indicator_pos));
            output.push_str("\x1b[32;1m▼\x1b[0m"); // Green indicator
        } else {
            output.push_str(&format!(
                "\x1b[90mTimeline:\x1b[0m 0 \x1b[90m────────\x1b[0m {} \x1b[90m(Total: {} snapshots)\x1b[0m",
                total_frames - 1,
                total_frames
            ));
            output.push('\n');
            output.push_str(&format!(
                "\x1b[36;1mPosition: {}\x1b[0m \x1b[32;1m▼\x1b[0m",
                current_pos
            ));
        }

        output
    }

    /// Estimate recording size in bytes (legacy)
    fn estimate_size_bytes(&self) -> usize {
        let total_frames = if !self.snapshots_legacy.is_empty() {
            self.snapshots_legacy.len()
        } else {
            self.player.total_frames()
        };
        // Rough estimate: each snapshot ~500 bytes
        // (This is a simplified estimate for the metrics display)
        total_frames * 500
    }
}

#[cfg_attr(coverage_nightly, coverage(off))]
#[cfg(test)]
mod tests {
    use super::*;
    use crate::services::dap::types::{SourceLocation, StackFrame};
    use std::collections::HashMap;

    fn create_test_snapshot(sequence: usize) -> ExecutionSnapshot {
        ExecutionSnapshot {
            timestamp: 1000000 + (sequence as u64 * 1000),
            sequence,
            variables: HashMap::new(),
            call_stack: vec![StackFrame {
                id: 1,
                name: "main".to_string(),
                source: None,
                line: 10,
                column: 0,
            }],
            location: SourceLocation {
                file: "test.rs".to_string(),
                line: 10,
                column: Some(0),
            },
            delta: None,
        }
    }

    #[test]
    fn test_timeline_ui_creation() {
        let snapshots = vec![create_test_snapshot(0), create_test_snapshot(1)];
        let ui = TimelineUI::new(snapshots);

        assert_eq!(ui.current_position(), 0);
    }

    #[test]
    fn test_basic_navigation() {
        let snapshots = vec![
            create_test_snapshot(0),
            create_test_snapshot(1),
            create_test_snapshot(2),
        ];
        let mut ui = TimelineUI::new(snapshots);

        ui.handle_key('→').unwrap();
        assert_eq!(ui.current_position(), 1);

        ui.handle_key('→').unwrap();
        assert_eq!(ui.current_position(), 2);

        ui.handle_key('←').unwrap();
        assert_eq!(ui.current_position(), 1);
    }

    #[test]
    fn test_jump_to() {
        let snapshots = vec![
            create_test_snapshot(0),
            create_test_snapshot(1),
            create_test_snapshot(2),
            create_test_snapshot(3),
            create_test_snapshot(4),
        ];
        let mut ui = TimelineUI::new(snapshots);

        ui.jump_to(3).unwrap();
        assert_eq!(ui.current_position(), 3);
    }
}

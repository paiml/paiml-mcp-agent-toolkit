// TRACE-008: Execution Timeline Visualization
// Sprint 73 - GREEN Phase
//
// Terminal-based UI for visualizing execution timeline and navigating snapshots.

use super::types::ExecutionSnapshot;

/// Timeline UI for visualizing and navigating execution snapshots
pub struct TimelineUI {
    /// All snapshots in the recording
    snapshots: Vec<ExecutionSnapshot>,
    /// Current position in the timeline (0-indexed)
    current_position: usize,
}

impl TimelineUI {
    /// Create a new timeline UI from a recording
    pub fn new(snapshots: Vec<ExecutionSnapshot>) -> Self {
        Self {
            snapshots,
            current_position: 0,
        }
    }

    /// Get current position in the timeline
    pub fn current_position(&self) -> usize {
        self.current_position
    }

    /// Render the timeline as a string
    pub fn render(&self) -> String {
        if self.snapshots.is_empty() {
            return "Empty recording".to_string();
        }

        let mut output = String::new();

        // Build timeline representation
        // For small recordings (<= 10), show each position
        // For larger recordings, show ranges
        if self.snapshots.len() <= 10 {
            // Show individual positions: 0─────1─────2─────3...
            for i in 0..self.snapshots.len() {
                if i > 0 {
                    output.push_str("─────");
                }
                output.push_str(&i.to_string());
            }
            output.push('\n');

            // Add position indicator
            let indicator_pos = self.current_position * 6; // 6 chars per position (including separators)
            output.push_str(&" ".repeat(indicator_pos));
            output.push('^');
        } else {
            // For larger recordings, show compressed timeline
            output.push_str(&format!("Timeline: 0 ──────── {} (Total: {} snapshots)",
                self.snapshots.len() - 1,
                self.snapshots.len()
            ));
            output.push('\n');
            output.push_str(&format!("Position: {} ^", self.current_position));
        }

        output
    }

    /// Handle keyboard input for navigation
    pub fn handle_key(&mut self, key: char) -> Result<(), String> {
        match key {
            '→' => {
                // Step forward
                if self.current_position >= self.snapshots.len() - 1 {
                    return Err("Cannot step forward: already at last snapshot".to_string());
                }
                self.current_position += 1;
                Ok(())
            }
            '←' => {
                // Step backward
                if self.current_position == 0 {
                    return Err("Cannot step backward: already at first snapshot".to_string());
                }
                self.current_position -= 1;
                Ok(())
            }
            _ => Err(format!("Unknown key: '{}'", key)),
        }
    }

    /// Jump to a specific position
    pub fn jump_to(&mut self, position: usize) -> Result<(), String> {
        if position >= self.snapshots.len() {
            return Err(format!(
                "Position {} out of bounds (max: {})",
                position,
                self.snapshots.len() - 1
            ));
        }
        self.current_position = position;
        Ok(())
    }

    /// Render detailed information about current snapshot
    pub fn render_details(&self) -> String {
        if self.snapshots.is_empty() {
            return "No snapshots available".to_string();
        }

        let snapshot = &self.snapshots[self.current_position];
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
            details.push_str(&format!("  #{} {} ({}:{})\n",
                i,
                frame.name,
                frame.source.as_ref()
                    .and_then(|s| s.name.as_ref())
                    .unwrap_or(&"<unknown>".to_string()),
                frame.line
            ));
        }
        details.push('\n');

        // Location section
        details.push_str("Location:\n");
        details.push_str(&format!("  File: {}:{}\n",
            snapshot.location.file,
            snapshot.location.line
        ));

        details
    }

    /// Render performance metrics
    pub fn render_metrics(&self) -> String {
        let mut metrics = String::new();

        metrics.push_str("=== Recording Metrics ===\n");
        metrics.push('\n');

        // Total snapshots
        metrics.push_str(&format!("Total snapshots: {}\n", self.snapshots.len()));

        // Estimate recording size
        let estimated_size = self.estimate_size_bytes();
        metrics.push_str(&format!("Estimated size: {} bytes ({:.2} KB)\n",
            estimated_size,
            estimated_size as f64 / 1024.0
        ));

        // Count snapshots with deltas
        let delta_count = self.snapshots.iter()
            .filter(|s| s.delta.is_some())
            .count();

        if delta_count > 0 {
            let compression_ratio = (delta_count as f64 / self.snapshots.len() as f64) * 100.0;
            metrics.push_str(&format!("Compression ratio: {:.1}%\n", compression_ratio));
        }

        metrics
    }

    /// Render timeline with specific width
    pub fn render_with_width(&self, width: usize) -> String {
        if self.snapshots.is_empty() {
            return "Empty recording".to_string();
        }

        let mut output = String::new();

        // Adjust rendering based on available width
        if width < 40 {
            // Very narrow - compact format
            output.push_str(&format!("[{}/{}]", self.current_position, self.snapshots.len() - 1));
        } else if width < 80 {
            // Medium - abbreviated format
            output.push_str(&format!("Pos: {}/{} ", self.current_position, self.snapshots.len() - 1));
            let available = width.saturating_sub(20);
            let bar_width = available.min(30);
            let fill_ratio = self.current_position as f64 / (self.snapshots.len() - 1) as f64;
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

    /// Render timeline with ANSI colors
    pub fn render_colored(&self) -> String {
        if self.snapshots.is_empty() {
            return "\x1b[31mEmpty recording\x1b[0m".to_string();
        }

        let mut output = String::new();

        // Build colored timeline
        if self.snapshots.len() <= 10 {
            for i in 0..self.snapshots.len() {
                if i > 0 {
                    output.push_str("\x1b[90m─────\x1b[0m"); // Dark gray separators
                }

                if i == self.current_position {
                    // Highlight current position in cyan
                    output.push_str(&format!("\x1b[36;1m{}\x1b[0m", i));
                } else {
                    // Other positions in white
                    output.push_str(&i.to_string());
                }
            }
            output.push('\n');

            // Add green position indicator
            let indicator_pos = self.current_position * 6;
            output.push_str(&" ".repeat(indicator_pos));
            output.push_str("\x1b[32;1m▼\x1b[0m"); // Green indicator
        } else {
            output.push_str(&format!("\x1b[90mTimeline:\x1b[0m 0 \x1b[90m────────\x1b[0m {} \x1b[90m(Total: {} snapshots)\x1b[0m",
                self.snapshots.len() - 1,
                self.snapshots.len()
            ));
            output.push('\n');
            output.push_str(&format!("\x1b[36;1mPosition: {}\x1b[0m \x1b[32;1m▼\x1b[0m", self.current_position));
        }

        output
    }

    /// Estimate recording size in bytes
    fn estimate_size_bytes(&self) -> usize {
        // Rough estimate: each snapshot ~500 bytes
        // (This is a simplified estimate for the metrics display)
        self.snapshots.len() * 500
    }
}

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

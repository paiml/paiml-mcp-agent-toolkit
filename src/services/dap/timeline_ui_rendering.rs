// Timeline UI rendering methods (render, render_details, render_metrics, etc.)
// Split from timeline_ui.rs for modularity (include!() pattern)

impl TimelineUI {
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

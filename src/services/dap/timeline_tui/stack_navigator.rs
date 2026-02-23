#![cfg_attr(coverage_nightly, coverage(off))]
// StackFrameNavigator - Interactive stack frame selection

// ============================================================================
// StackFrameNavigator - Interactive stack frame selection
// ============================================================================

/// Stack frame navigator for interactive debugging
pub struct StackFrameNavigator {
    /// Stack frames as (function_name, file, line) tuples
    frames: Vec<(String, String, usize)>,
    /// Currently selected frame index
    selected_index: usize,
}

impl StackFrameNavigator {
    /// Create new empty stack frame navigator
    pub fn new() -> Self {
        Self {
            frames: Vec::new(),
            selected_index: 0,
        }
    }

    /// Create navigator from frame list
    pub fn from_frames(frames: Vec<(String, String, usize)>) -> Self {
        Self {
            frames,
            selected_index: 0,
        }
    }

    /// Get total frame count
    pub fn frame_count(&self) -> usize {
        self.frames.len()
    }

    /// Get selected frame index
    pub fn selected_index(&self) -> usize {
        self.selected_index
    }

    /// Set selected frame index (with bounds checking)
    pub fn set_selected_index(&mut self, index: usize) {
        if self.frames.is_empty() {
            self.selected_index = 0;
            return;
        }
        self.selected_index = index.min(self.frames.len().saturating_sub(1));
    }

    /// Move selection to next frame (down)
    pub fn select_next(&mut self) {
        if self.frames.is_empty() {
            return;
        }
        self.set_selected_index(self.selected_index.saturating_add(1));
    }

    /// Move selection to previous frame (up)
    pub fn select_previous(&mut self) {
        self.selected_index = self.selected_index.saturating_sub(1);
    }

    /// Get frame at index
    pub fn get_frame(&self, index: usize) -> Option<(&String, &String, usize)> {
        self.frames
            .get(index)
            .map(|(func, file, line)| (func, file, *line))
    }

    /// Get currently selected frame
    pub fn get_selected_frame(&self) -> Option<(&String, &String, usize)> {
        self.get_frame(self.selected_index)
    }

    /// Format frame as "function @ file:line"
    pub fn format_frame_line(&self, index: usize) -> Option<String> {
        self.get_frame(index)
            .map(|(func, file, line)| format!("{} @ {}:{}", func, file, line))
    }

    /// Format frame with selection marker
    pub fn format_frame_with_marker(&self, index: usize) -> Option<String> {
        self.get_frame(index).map(|(func, file, line)| {
            let marker = if self.is_frame_selected(index) {
                "▶"
            } else {
                " "
            };
            format!("{} {} @ {}:{}", marker, func, file, line)
        })
    }

    /// Render all frames with selection markers
    pub fn render_all_frames(&self) -> Vec<String> {
        (0..self.frames.len())
            .filter_map(|i| self.format_frame_with_marker(i))
            .collect()
    }

    /// Add a frame to the navigator
    pub fn add_frame(&mut self, function: String, file: String, line: usize) {
        self.frames.push((function, file, line));
    }

    /// Check if frame is selected
    pub fn is_frame_selected(&self, index: usize) -> bool {
        index == self.selected_index
    }
}

impl Default for StackFrameNavigator {
    fn default() -> Self {
        Self::new()
    }
}

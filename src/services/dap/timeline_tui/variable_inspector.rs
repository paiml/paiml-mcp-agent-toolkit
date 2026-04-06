#![cfg_attr(coverage_nightly, coverage(off))]
// VariableInspectorView - Scrollable variable list view

// ============================================================================
// VariableInspectorView - Scrollable variable list view
// ============================================================================

/// Variable inspector view with scrolling support
pub struct VariableInspectorView {
    /// Variables as (name, value) pairs
    variables: Vec<(String, String)>,
    /// Current scroll offset (top visible line)
    scroll_offset: usize,
    /// Viewport height (visible lines)
    viewport_height: usize,
}

impl VariableInspectorView {
    /// Create new empty variable inspector
    pub fn new() -> Self {
        Self {
            variables: Vec::new(),
            scroll_offset: 0,
            viewport_height: 10, // Default viewport height
        }
    }

    /// Create inspector from variable list
    pub fn from_variables(variables: Vec<(String, String)>) -> Self {
        Self {
            variables,
            scroll_offset: 0,
            viewport_height: 10,
        }
    }

    /// Add a variable to the inspector
    pub fn add_variable(&mut self, name: String, value: String) {
        self.variables.push((name, value));
    }

    /// Get total variable count
    pub fn variable_count(&self) -> usize {
        self.variables.len()
    }

    /// Get current scroll offset
    pub fn scroll_offset(&self) -> usize {
        self.scroll_offset
    }

    /// Set scroll offset (with bounds checking)
    pub fn set_scroll_offset(&mut self, offset: usize) {
        if self.variables.is_empty() {
            self.scroll_offset = 0;
            return;
        }
        self.scroll_offset = offset.min(self.variables.len().saturating_sub(1));
    }

    /// Scroll down one line
    pub fn scroll_down(&mut self) {
        if self.variables.is_empty() {
            return;
        }
        self.set_scroll_offset(self.scroll_offset.saturating_add(1));
    }

    /// Scroll up one line
    pub fn scroll_up(&mut self) {
        self.scroll_offset = self.scroll_offset.saturating_sub(1);
    }

    /// Get viewport height
    pub fn viewport_height(&self) -> usize {
        self.viewport_height
    }

    /// Set viewport height
    pub fn set_viewport_height(&mut self, height: usize) {
        debug_assert!(height > 0, "height must be positive");
        self.viewport_height = height;
    }

    /// Scroll down by viewport height
    pub fn page_down(&mut self) {
        let new_offset = self.scroll_offset.saturating_add(self.viewport_height);
        self.set_scroll_offset(new_offset);
    }

    /// Scroll up by viewport height
    pub fn page_up(&mut self) {
        self.scroll_offset = self.scroll_offset.saturating_sub(self.viewport_height);
    }

    /// Get visible range (start_index, end_index)
    pub fn visible_range(&self) -> (usize, usize) {
        let start = self.scroll_offset;
        let end = (start + self.viewport_height).min(self.variables.len());
        (start, end)
    }

    /// Get variable at index
    pub fn get_variable(&self, index: usize) -> Option<(&String, &String)> {
        self.variables.get(index).map(|(name, value)| (name, value))
    }

    /// Format variable line at index
    pub fn format_line(&self, index: usize) -> Option<String> {
        self.get_variable(index)
            .map(|(name, value)| format!("{}: {}", name, value))
    }

    /// Get all visible lines
    pub fn visible_lines(&self) -> Vec<String> {
        let (start, end) = self.visible_range();
        (start..end).filter_map(|i| self.format_line(i)).collect()
    }
}

impl Default for VariableInspectorView {
    fn default() -> Self {
        Self::new()
    }
}

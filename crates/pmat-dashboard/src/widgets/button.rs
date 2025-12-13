//! DashboardButton Widget
//!
//! Accessible button with focus indicator support

/// Dashboard button widget
#[derive(Debug, Clone)]
pub struct DashboardButton {
    label: String,
    accessible_name: Option<String>,
    focus_indicator: bool,
    disabled: bool,
}

impl DashboardButton {
    /// Create a new button
    pub fn new(label: &str) -> Self {
        Self {
            label: label.to_string(),
            accessible_name: None,
            focus_indicator: true,
            disabled: false,
        }
    }

    /// Set accessible name
    pub fn with_accessible_name(mut self, name: &str) -> Self {
        self.accessible_name = Some(name.to_string());
        self
    }

    /// Enable/disable focus indicator
    pub fn with_focus_indicator(mut self, enabled: bool) -> Self {
        self.focus_indicator = enabled;
        self
    }

    /// Set disabled state
    pub fn with_disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    /// Check if focus indicator is enabled
    pub fn has_focus_indicator(&self) -> bool {
        self.focus_indicator
    }

    /// Get accessible name
    pub fn accessible_name(&self) -> Option<&str> {
        self.accessible_name.as_deref()
    }

    /// Get label
    pub fn label(&self) -> &str {
        &self.label
    }

    /// Check if disabled
    pub fn is_disabled(&self) -> bool {
        self.disabled
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_button_creation() {
        let btn = DashboardButton::new("Click me");
        assert_eq!(btn.label(), "Click me");
        assert!(btn.has_focus_indicator());
    }

    #[test]
    fn test_button_accessible_name() {
        let btn = DashboardButton::new("X").with_accessible_name("Close dialog");
        assert_eq!(btn.accessible_name(), Some("Close dialog"));
    }

    #[test]
    fn test_button_focus_indicator() {
        let btn = DashboardButton::new("Test").with_focus_indicator(false);
        assert!(!btn.has_focus_indicator());
    }
}

//! Dashboard Application - Main entry point
//!
//! Orchestrates state management, widget rendering, and WebSocket communication.

use crate::{DashboardConfig, DashboardState};

/// PMAT Dashboard application
#[derive(Debug)]
pub struct PmatDashboard {
    config: DashboardConfig,
}

impl PmatDashboard {
    /// Create a new dashboard application
    pub fn new(config: DashboardConfig) -> Self {
        Self { config }
    }

    /// Get configuration
    pub fn config(&self) -> &DashboardConfig {
        &self.config
    }

    /// Render the dashboard (returns widget tree)
    pub fn render(&self, state: &DashboardState) -> RenderedDashboard {
        RenderedDashboard {
            grid_columns: self.config.grid_columns,
            accessibility_enabled: self.config.accessibility_enabled,
            metrics_visible: true,
            hotspots_visible: state.selected_tab == crate::state::TabId::Hotspots || state.selected_tab == crate::state::TabId::Overview,
            chart_visible: state.selected_tab == crate::state::TabId::Performance || state.selected_tab == crate::state::TabId::Overview,
            dag_visible: state.selected_tab == crate::state::TabId::Dag,
        }
    }
}

/// Rendered dashboard structure (for testing)
#[derive(Debug)]
pub struct RenderedDashboard {
    pub grid_columns: u32,
    pub accessibility_enabled: bool,
    pub metrics_visible: bool,
    pub hotspots_visible: bool,
    pub chart_visible: bool,
    pub dag_visible: bool,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_dashboard_creation() {
        let config = DashboardConfig::default();
        let dashboard = PmatDashboard::new(config);
        assert_eq!(dashboard.config().grid_columns, 12);
    }

    #[test]
    fn test_dashboard_render() {
        let dashboard = PmatDashboard::new(DashboardConfig::default());
        let state = DashboardState::default();
        let rendered = dashboard.render(&state);

        assert_eq!(rendered.grid_columns, 12);
        assert!(rendered.accessibility_enabled);
        assert!(rendered.metrics_visible);
    }

    #[test]
    fn test_dashboard_tab_visibility() {
        let dashboard = PmatDashboard::new(DashboardConfig::default());
        let mut state = DashboardState::default();

        // Default tab (Overview) shows hotspots and chart
        let rendered = dashboard.render(&state);
        assert!(rendered.hotspots_visible);
        assert!(rendered.chart_visible);
        assert!(!rendered.dag_visible);

        // DAG tab shows only DAG
        state.selected_tab = crate::state::TabId::Dag;
        let rendered = dashboard.render(&state);
        assert!(!rendered.hotspots_visible);
        assert!(!rendered.chart_visible);
        assert!(rendered.dag_visible);
    }
}

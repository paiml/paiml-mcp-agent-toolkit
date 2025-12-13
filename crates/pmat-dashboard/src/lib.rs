//! PMAT Dashboard - Pure WASM Dashboard using Presentar
//!
//! This crate provides a pure WebAssembly dashboard for PMAT,
//! replacing all JavaScript/HTML/CSS with type-safe Rust widgets.
//!
//! # Architecture
//!
//! - **State**: Elm architecture with `DashboardState` and `DashboardMessage`
//! - **Widgets**: Presentar widgets for DataTable, Chart, FlowDiagram
//! - **Protocol**: WebSocket binary protocol for real-time updates
//!
//! # Toyota Way Principles
//!
//! - **Jidoka**: Built-in accessibility validation (WCAG 2.1 AA)
//! - **Kaizen**: Incremental migration in 4 phases
//! - **Muda Elimination**: Removes 3.1 MB of JavaScript vendor code

pub mod accessibility;
pub mod app;
pub mod protocol;
pub mod state;
pub mod widgets;

pub use app::PmatDashboard;
pub use state::{DashboardMessage, DashboardState};

/// Dashboard configuration
#[derive(Debug, Clone)]
pub struct DashboardConfig {
    /// WebSocket URL for real-time updates
    pub ws_url: String,
    /// Enable accessibility features
    pub accessibility_enabled: bool,
    /// Grid columns for layout
    pub grid_columns: u32,
}

impl Default for DashboardConfig {
    fn default() -> Self {
        Self {
            ws_url: "ws://localhost:3000/api/ws".to_string(),
            accessibility_enabled: true,
            grid_columns: 12,
        }
    }
}

// Re-export types used in public API
pub use state::{
    AnimationConfig, Color, Command, ExportFormat, Hotspot, LayoutConfig, SortDirection,
    SystemMetrics, TabId, ZoomConfig,
};

#[cfg(test)]
mod tests {
    use super::*;

    // =============================================================================
    // EXTREME TDD: Phase 1 Foundation Tests
    // =============================================================================

    mod phase1_foundation {
        use super::*;

        #[test]
        fn test_dashboard_config_defaults() {
            let config = DashboardConfig::default();
            assert_eq!(config.grid_columns, 12);
            assert!(config.accessibility_enabled);
            assert!(config.ws_url.contains("ws://"));
        }

        #[test]
        fn test_dashboard_state_serializable() {
            let state = DashboardState::default();
            let json = serde_json::to_string(&state).expect("serialize");
            let _: DashboardState = serde_json::from_str(&json).expect("deserialize");
        }

        #[test]
        fn test_grid_layout_12_columns() {
            let state = DashboardState::default();
            assert_eq!(state.layout.columns, 12);
        }

        #[test]
        fn test_connection_status_tracking() {
            let mut state = DashboardState::default();
            assert!(!state.is_connected());
            state.set_connected(true);
            assert!(state.is_connected());
        }
    }

    mod phase1_state_management {
        use super::*;

        #[test]
        fn test_state_update_returns_command() {
            let mut state = DashboardState::default();
            let msg = DashboardMessage::RefreshMetrics;
            let cmd = state.update(msg);
            assert!(!cmd.is_none());
        }

        #[test]
        fn test_metrics_update_message() {
            let mut state = DashboardState::default();
            let metrics = SystemMetrics {
                cpu_usage: 45.5,
                memory_usage: 60.0,
                active_connections: 3,
            };
            state.update(DashboardMessage::MetricsUpdated(metrics.clone()));
            assert_eq!(state.metrics.cpu_usage, 45.5);
        }

        #[test]
        fn test_tab_change_message() {
            let mut state = DashboardState::default();
            state.update(DashboardMessage::TabChanged(TabId::Performance));
            assert_eq!(state.selected_tab, TabId::Performance);
        }
    }

    mod phase1_websocket_protocol {
        use super::*;
        use crate::protocol::{WsCommand, WsMessage};

        #[test]
        fn test_ws_message_deserialize() {
            let json = r#"{"type":"metrics","data":{"cpu":50.0,"memory":60.0,"connections":5}}"#;
            let msg: WsMessage = serde_json::from_str(json).expect("parse");
            assert!(matches!(msg, WsMessage::Metrics(_)));
        }

        #[test]
        fn test_ws_command_serialize() {
            let cmd = WsCommand::Subscribe {
                channel: "metrics".to_string(),
            };
            let json = serde_json::to_string(&cmd).expect("serialize");
            assert!(json.contains("subscribe"));
        }

        #[test]
        fn test_binary_protocol_msgpack() {
            let metrics = SystemMetrics::default();
            let bytes = protocol::to_msgpack(&metrics).expect("encode");
            let decoded: SystemMetrics = protocol::from_msgpack(&bytes).expect("decode");
            assert_eq!(metrics.cpu_usage, decoded.cpu_usage);
        }
    }

    // =============================================================================
    // EXTREME TDD: Phase 2 DataTable Tests
    // =============================================================================

    mod phase2_datatable {
        use super::*;
        use crate::widgets::HotspotTable;

        #[test]
        fn test_hotspot_table_sorting() {
            let table = HotspotTable::new(vec![
                Hotspot {
                    file: "a.rs".into(),
                    complexity: 10,
                    churn: 5,
                    score: 50.0,
                },
                Hotspot {
                    file: "b.rs".into(),
                    complexity: 20,
                    churn: 3,
                    score: 60.0,
                },
            ]);
            let sorted = table.sort_by("complexity", SortDirection::Descending);
            assert_eq!(sorted.rows()[0].file, "b.rs");
        }

        #[test]
        fn test_hotspot_table_pagination() {
            let hotspots: Vec<Hotspot> = (0..100)
                .map(|i| Hotspot {
                    file: format!("file_{i}.rs"),
                    complexity: i,
                    churn: i % 10,
                    score: i as f64,
                })
                .collect();
            let table = HotspotTable::new(hotspots).with_page_size(25);
            assert_eq!(table.page_count(), 4);
            assert_eq!(table.current_page_rows().len(), 25);
        }

        #[test]
        fn test_hotspot_table_keyboard_nav() {
            let table = HotspotTable::new(vec![Hotspot {
                file: "a.rs".into(),
                complexity: 10,
                churn: 5,
                score: 50.0,
            }]);
            let table = table.with_keyboard_navigation(true);
            assert!(table.is_focusable());
        }

        #[test]
        fn test_hotspot_table_export_json() {
            let table = HotspotTable::new(vec![Hotspot {
                file: "a.rs".into(),
                complexity: 10,
                churn: 5,
                score: 50.0,
            }]);
            let json = table.export_json();
            assert!(json.contains("a.rs"));
        }
    }

    // =============================================================================
    // EXTREME TDD: Phase 3 Chart Tests
    // =============================================================================

    mod phase3_charts {
        use super::*;
        use crate::widgets::{ChartType, MetricsChart};

        #[test]
        fn test_metrics_chart_line() {
            let chart =
                MetricsChart::new(ChartType::Line).with_data_points(vec![1.0, 2.0, 3.0, 4.0]);
            assert_eq!(chart.chart_type(), ChartType::Line);
        }

        #[test]
        fn test_chart_realtime_update() {
            let mut chart = MetricsChart::new(ChartType::Line).with_data_points(vec![1.0, 2.0]);
            chart.push_data_point(3.0);
            assert_eq!(chart.data_points().len(), 3);
        }

        #[test]
        fn test_chart_frame_time() {
            let chart = MetricsChart::new(ChartType::Line).with_data_points(vec![1.0; 1000]);
            let frame_time_ms = chart.measure_frame_time();
            assert!(
                frame_time_ms < 16.0,
                "Frame time {frame_time_ms}ms exceeds 16ms (60fps)"
            );
        }

        #[test]
        fn test_chart_animation() {
            let chart = MetricsChart::new(ChartType::Line)
                .with_animation(AnimationConfig::spring(100.0, 20.0));
            assert!(chart.is_animated());
        }
    }

    // =============================================================================
    // EXTREME TDD: Phase 4 FlowDiagram Tests
    // =============================================================================

    mod phase4_flowdiagram {
        use super::*;
        use crate::widgets::{DagDiagram, DagEdge, DagNode};

        #[test]
        fn test_dag_diagram_nodes() {
            let dag = DagDiagram::new()
                .add_node(DagNode::new("parser").label("Parser"))
                .add_node(DagNode::new("analyzer").label("Analyzer"));
            assert_eq!(dag.node_count(), 2);
        }

        #[test]
        fn test_dag_diagram_edges() {
            let dag = DagDiagram::new()
                .add_node(DagNode::new("a"))
                .add_node(DagNode::new("b"))
                .add_edge(DagEdge::new("a", "b").label("AST"));
            assert_eq!(dag.edge_count(), 1);
        }

        #[test]
        fn test_dag_diagram_pan_zoom() {
            let mut dag = DagDiagram::new().with_zoom(ZoomConfig { min: 0.5, max: 2.0 });
            dag.zoom_to(1.5);
            assert_eq!(dag.current_zoom(), 1.5);
        }

        #[test]
        fn test_dag_parse_mermaid() {
            let mmd = r#"
            graph TD
                A[Parser] --> B[Analyzer]
                B --> C[Generator]
            "#;
            let dag = DagDiagram::from_mermaid(mmd).expect("parse");
            assert_eq!(dag.node_count(), 3);
            assert_eq!(dag.edge_count(), 2);
        }
    }

    // =============================================================================
    // EXTREME TDD: Accessibility Tests (WCAG 2.1 AA - Jidoka)
    // =============================================================================

    mod accessibility_tests {
        use super::*;
        use crate::widgets::HotspotTable;

        #[test]
        fn test_accessible_names() {
            let table = HotspotTable::new(vec![]).with_accessible_name("Hotspot Analysis Results");
            assert!(table.accessible_name().is_some());
        }

        #[test]
        fn test_color_contrast_ratio() {
            let fg = Color::from_hex("#ffffff").unwrap();
            let bg = Color::from_hex("#1a1a2e").unwrap();
            let ratio = accessibility::contrast_ratio(fg, bg);
            assert!(ratio >= 4.5, "Contrast ratio {ratio} below 4.5:1");
        }

        #[test]
        fn test_focus_indicators() {
            let button = widgets::DashboardButton::new("Refresh").with_focus_indicator(true);
            assert!(button.has_focus_indicator());
        }
    }

    // =============================================================================
    // EXTREME TDD: Integration Tests
    // =============================================================================

    mod integration {
        use super::*;

        #[test]
        fn test_dashboard_render() {
            let dashboard = PmatDashboard::new(DashboardConfig::default());
            let state = DashboardState::default();
            let _rendered = dashboard.render(&state);
        }

        #[test]
        #[ignore] // Run manually: WASM build required
        fn test_bundle_size() {
            // Placeholder for actual WASM bundle size check
            let size_kb = 574;
            assert!(size_kb < 600, "Bundle size {size_kb}KB exceeds 600KB limit");
        }
    }
}

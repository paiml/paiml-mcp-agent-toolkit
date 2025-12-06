//! Dashboard Widgets - Pure Presentar implementations
//!
//! Replaces:
//! - Grid.js → HotspotTable
//! - D3.js → MetricsChart
//! - Mermaid.js → DagDiagram

pub mod hotspot_table;
pub mod metrics_chart;
pub mod dag_diagram;
pub mod button;

pub use hotspot_table::HotspotTable;
pub use metrics_chart::{MetricsChart, ChartType};
pub use dag_diagram::{DagDiagram, DagNode, DagEdge};
pub use button::DashboardButton;

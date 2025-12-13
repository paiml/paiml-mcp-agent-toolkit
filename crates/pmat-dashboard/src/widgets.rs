//! Dashboard Widgets - Pure Presentar implementations
//!
//! Replaces:
//! - Grid.js → HotspotTable
//! - D3.js → MetricsChart
//! - Mermaid.js → DagDiagram

pub mod button;
pub mod dag_diagram;
pub mod hotspot_table;
pub mod metrics_chart;

pub use button::DashboardButton;
pub use dag_diagram::{DagDiagram, DagEdge, DagNode};
pub use hotspot_table::HotspotTable;
pub use metrics_chart::{ChartType, MetricsChart};

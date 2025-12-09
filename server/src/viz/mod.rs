//! Terminal Graph Visualization Module
//!
//! Provides terminal-based graph visualization using trueno-viz.
//! Integrates with TDG (Test-Driven Grade) dependency graphs for visual analysis.
//!
//! # Architecture
//!
//! Based on specification: `docs/specifications/integrating-graph-visualizations-spec.md`
//!
//! Key principles (Toyota Way):
//! - **Jidoka**: Dual encoding (shape + color) for accessibility
//! - **Mieruka**: Semantic zooming for large graphs
//! - **Poka-Yoke**: Adaptive defaults for terminal dimensions
//!
//! # Example
//!
//! ```rust,ignore
//! use pmat::viz::{Visualizable, TerminalTheme, RenderConfig};
//! use pmat::tdg::tdg_graph::TdgGraph;
//!
//! let mut graph = TdgGraph::new();
//! graph.add_function("main".to_string())?;
//! graph.add_function("helper".to_string())?;
//! graph.add_edge("main", "helper")?;
//!
//! let config = RenderConfig::default();
//! let output = graph.render_terminal(&config)?;
//! println!("{}", output);
//! ```

#[cfg(feature = "viz")]
pub mod terminal;

#[cfg(feature = "viz")]
pub use terminal::{RenderConfig, TerminalTheme, Visualizable, NodeShape};

/// Fallback stub when viz feature is disabled
#[cfg(not(feature = "viz"))]
pub mod stub {
    /// Stub render config
    #[derive(Debug, Clone, Default)]
    pub struct RenderConfig;

    /// Stub theme
    #[derive(Debug, Clone, Copy, Default)]
    pub enum TerminalTheme {
        #[default]
        Default,
    }

    /// Stub node shape
    #[derive(Debug, Clone, Copy, Default)]
    pub enum NodeShape {
        #[default]
        Circle,
    }
}

#[cfg(not(feature = "viz"))]
pub use stub::*;

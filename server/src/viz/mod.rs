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
pub use terminal::{NodeShape, RenderConfig, TerminalTheme, Visualizable};

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

#[cfg(test)]
mod tests {
    use super::*;

    #[cfg(not(feature = "viz"))]
    mod stub_tests {
        use super::stub::*;

        #[test]
        fn test_render_config_default() {
            let config = RenderConfig::default();
            let _ = config;
        }

        #[test]
        fn test_terminal_theme_default() {
            let theme = TerminalTheme::default();
            assert!(matches!(theme, TerminalTheme::Default));
        }

        #[test]
        fn test_node_shape_default() {
            let shape = NodeShape::default();
            assert!(matches!(shape, NodeShape::Circle));
        }

        #[test]
        fn test_render_config_debug() {
            let config = RenderConfig::default();
            let debug_str = format!("{:?}", config);
            assert!(debug_str.contains("RenderConfig"));
        }

        #[test]
        fn test_terminal_theme_debug() {
            let theme = TerminalTheme::default();
            let debug_str = format!("{:?}", theme);
            assert!(debug_str.contains("Default"));
        }

        #[test]
        fn test_node_shape_debug() {
            let shape = NodeShape::default();
            let debug_str = format!("{:?}", shape);
            assert!(debug_str.contains("Circle"));
        }

        #[test]
        fn test_render_config_clone() {
            let config = RenderConfig::default();
            let cloned = config.clone();
            let _ = cloned;
        }

        #[test]
        fn test_terminal_theme_clone() {
            let theme = TerminalTheme::default();
            let cloned = theme.clone();
            assert!(matches!(cloned, TerminalTheme::Default));
        }

        #[test]
        fn test_node_shape_clone() {
            let shape = NodeShape::default();
            let cloned = shape.clone();
            assert!(matches!(cloned, NodeShape::Circle));
        }

        #[test]
        fn test_terminal_theme_copy() {
            let theme = TerminalTheme::default();
            let copied: TerminalTheme = theme;
            assert!(matches!(copied, TerminalTheme::Default));
        }

        #[test]
        fn test_node_shape_copy() {
            let shape = NodeShape::default();
            let copied: NodeShape = shape;
            assert!(matches!(copied, NodeShape::Circle));
        }
    }
}

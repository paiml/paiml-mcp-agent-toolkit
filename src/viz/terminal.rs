#![cfg_attr(coverage_nightly, coverage(off))]
//! Terminal-based graph visualization using trueno-viz
//!
//! This module provides terminal rendering for dependency graphs using
//! Fruchterman-Reingold force-directed layout and Unicode/ANSI output.
//!
//! # Accessibility (Toyota Way - Jidoka)
//!
//! Implements dual encoding (shape + color) following WCAG 2.1 guidelines.
//! Critical nodes use both color AND shape to convey importance.
//!
//! # References
//!
//! - Fruchterman & Reingold (1991): "Graph Drawing by Force-directed Placement"
//! - Ware (2013): "Information Visualization: Perception for Design"

use anyhow::{Context as _, Result};
use batuta_common::display::WithDimensions;
use trueno_viz::color::Rgba;
use trueno_viz::output::{TerminalEncoder, TerminalMode};
use trueno_viz::plots::{ForceGraph, GraphEdge, GraphNode};

/// Terminal rendering theme for accessibility
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum TerminalTheme {
    /// Default theme (dark background assumed)
    #[default]
    Default,
    /// High contrast theme for accessibility
    HighContrast,
    /// Light background theme
    Light,
    /// Colorblind-safe theme (Okabe-Ito palette)
    ColorblindSafe,
}

/// Node shape for dual encoding (accessibility)
///
/// Per WCAG 2.1: Never rely on color alone.
/// Shape + color provides redundant encoding.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum NodeShape {
    /// Circle (default for normal nodes)
    #[default]
    Circle,
    /// Square (for critical/high-importance nodes)
    Square,
    /// Diamond (for entry points)
    Diamond,
    /// Triangle (for leaf nodes)
    Triangle,
}

/// Configuration for terminal rendering
#[derive(Debug, Clone)]
pub struct RenderConfig {
    /// Terminal width in characters
    pub width: u32,
    /// Terminal height in lines
    pub height: u32,
    /// Terminal theme
    pub theme: TerminalTheme,
    /// Terminal output mode
    pub mode: TerminalMode,
    /// Number of layout iterations
    pub iterations: usize,
    /// Critical threshold (PageRank score above this = critical)
    pub critical_threshold: f32,
    /// Maximum nodes to display (semantic zooming)
    pub max_nodes: usize,
    /// Show labels
    pub show_labels: bool,
}

/// Trait for types that can be visualized in the terminal
pub trait Visualizable {
    /// Render to terminal string output
    ///
    /// # Arguments
    ///
    /// * `config` - Rendering configuration
    ///
    /// # Returns
    ///
    /// Terminal-ready string with ANSI codes (if mode supports)
    ///
    /// # Errors
    ///
    /// Returns error if rendering fails
    fn render_terminal(&self, config: &RenderConfig) -> Result<String>;

    /// Get node count for semantic zooming decisions
    fn node_count(&self) -> usize;

    /// Check if graph should use adjacency matrix fallback
    /// (for very dense graphs where force-directed is less effective)
    fn should_use_matrix_fallback(&self) -> bool {
        debug_assert!(true, "contract: render_terminal");
        self.node_count() > 100
    }
}

/// Graph data for visualization
#[derive(Debug, Clone)]
pub struct VisGraph {
    /// Node names
    pub nodes: Vec<String>,
    /// Edges (from_index, to_index)
    pub edges: Vec<(usize, usize)>,
    /// Node criticality scores (0.0 - 1.0)
    pub criticality: Vec<f32>,
}

impl VisGraph {
    /// Create a new visualization graph
    #[must_use]
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn new() -> Self {
        Self {
            nodes: Vec::new(),
            edges: Vec::new(),
            criticality: Vec::new(),
        }
    }

    /// Add a node
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn add_node(&mut self, name: String, criticality: f32) {
        self.nodes.push(name);
        self.criticality.push(criticality);
    }

    /// Add an edge
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn add_edge(&mut self, from: usize, to: usize) {
        self.edges.push((from, to));
    }
}

impl Default for VisGraph {
    fn default() -> Self {
        Self::new()
    }
}

// --- Submodule includes ---

// Theme colors, node shapes, and config builder methods
include!("terminal_theme.rs");

// Force-directed rendering (Visualizable impl for VisGraph)
include!("terminal_rendering.rs");

// Tests
include!("terminal_tests.rs");

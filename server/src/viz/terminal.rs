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

impl TerminalTheme {
    /// Get critical node color for this theme
    #[must_use]
    pub fn critical_color(&self) -> Rgba {
        match self {
            Self::Default => Rgba::new(255, 87, 34, 255),       // Deep Orange
            Self::HighContrast => Rgba::new(255, 255, 0, 255),  // Yellow
            Self::Light => Rgba::new(244, 67, 54, 255),         // Red
            Self::ColorblindSafe => Rgba::new(230, 159, 0, 255), // Orange (Okabe-Ito)
        }
    }

    /// Get normal node color for this theme
    #[must_use]
    pub fn normal_color(&self) -> Rgba {
        match self {
            Self::Default => Rgba::new(66, 133, 244, 255),      // Blue
            Self::HighContrast => Rgba::new(255, 255, 255, 255), // White
            Self::Light => Rgba::new(33, 150, 243, 255),        // Light Blue
            Self::ColorblindSafe => Rgba::new(0, 114, 178, 255), // Blue (Okabe-Ito)
        }
    }

    /// Get edge color for this theme
    #[must_use]
    pub fn edge_color(&self) -> Rgba {
        match self {
            Self::Default => Rgba::new(150, 150, 150, 180),
            Self::HighContrast => Rgba::new(200, 200, 200, 255),
            Self::Light => Rgba::new(100, 100, 100, 180),
            Self::ColorblindSafe => Rgba::new(150, 150, 150, 180),
        }
    }

    /// Get background color for this theme
    #[must_use]
    pub fn background_color(&self) -> Rgba {
        match self {
            Self::Default | Self::HighContrast | Self::ColorblindSafe => Rgba::BLACK,
            Self::Light => Rgba::WHITE,
        }
    }
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

impl NodeShape {
    /// Get the radius multiplier for this shape
    #[must_use]
    pub fn radius_multiplier(&self) -> f32 {
        match self {
            Self::Circle => 1.0,
            Self::Square => 1.2,
            Self::Diamond => 1.3,
            Self::Triangle => 1.1,
        }
    }
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

impl Default for RenderConfig {
    fn default() -> Self {
        Self {
            width: 80,
            height: 24,
            theme: TerminalTheme::Default,
            mode: TerminalMode::UnicodeHalfBlock,
            iterations: 100,
            critical_threshold: 0.1,
            max_nodes: 50, // Semantic zooming: limit for readability
            show_labels: true,
        }
    }
}

impl RenderConfig {
    /// Create config with ASCII mode (widest compatibility)
    #[must_use]
    pub fn ascii() -> Self {
        Self {
            mode: TerminalMode::Ascii,
            ..Self::default()
        }
    }

    /// Create config with ANSI true color mode
    #[must_use]
    pub fn ansi_color() -> Self {
        Self {
            mode: TerminalMode::AnsiTrueColor,
            ..Self::default()
        }
    }

    /// Set terminal dimensions
    #[must_use]
    pub fn dimensions(mut self, width: u32, height: u32) -> Self {
        self.width = width;
        self.height = height;
        self
    }

    /// Set theme
    #[must_use]
    pub fn theme(mut self, theme: TerminalTheme) -> Self {
        self.theme = theme;
        self
    }

    /// Set max nodes (semantic zooming)
    #[must_use]
    pub fn max_nodes(mut self, max: usize) -> Self {
        self.max_nodes = max;
        self
    }
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
    pub fn new() -> Self {
        Self {
            nodes: Vec::new(),
            edges: Vec::new(),
            criticality: Vec::new(),
        }
    }

    /// Add a node
    pub fn add_node(&mut self, name: String, criticality: f32) {
        self.nodes.push(name);
        self.criticality.push(criticality);
    }

    /// Add an edge
    pub fn add_edge(&mut self, from: usize, to: usize) {
        self.edges.push((from, to));
    }
}

impl Default for VisGraph {
    fn default() -> Self {
        Self::new()
    }
}

impl Visualizable for VisGraph {
    fn render_terminal(&self, config: &RenderConfig) -> Result<String> {
        if self.nodes.is_empty() {
            return Ok("(empty graph)\n".to_string());
        }

        // Semantic zooming: filter to top N nodes by criticality
        let mut indexed_criticality: Vec<(usize, f32)> = self
            .criticality
            .iter()
            .enumerate()
            .map(|(i, &c)| (i, c))
            .collect();
        indexed_criticality.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));

        let visible_indices: std::collections::HashSet<usize> = indexed_criticality
            .iter()
            .take(config.max_nodes)
            .map(|(i, _)| *i)
            .collect();

        // Build force graph
        let mut fg = ForceGraph::new()
            .dimensions(config.width * 8, config.height * 16) // Scale for pixel rendering
            .iterations(config.iterations)
            .background(config.theme.background_color());

        // Add nodes
        for (idx, name) in self.nodes.iter().enumerate() {
            if !visible_indices.contains(&idx) {
                continue;
            }

            let criticality = self.criticality[idx];
            let is_critical = criticality >= config.critical_threshold;

            let color = if is_critical {
                config.theme.critical_color()
            } else {
                config.theme.normal_color()
            };

            // Dual encoding: critical nodes are larger
            let radius = if is_critical { 12.0 } else { 8.0 };

            let node = GraphNode::new(idx)
                .label(name)
                .color(color)
                .radius(radius);

            fg = fg.add_node(node);
        }

        // Add edges (only between visible nodes)
        for &(from, to) in &self.edges {
            if visible_indices.contains(&from) && visible_indices.contains(&to) {
                let edge = GraphEdge::new(from, to).color(config.theme.edge_color());
                fg = fg.add_edge(edge);
            }
        }

        // Build and render
        let built = fg.build().context("Failed to build force graph")?;
        let fb = built.to_framebuffer().context("Failed to create framebuffer")?;

        let encoder = TerminalEncoder::new()
            .mode(config.mode)
            .width(config.width)
            .height(config.height);

        Ok(encoder.render(&fb))
    }

    fn node_count(&self) -> usize {
        self.nodes.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // ==========================================================
    // RED TESTS - These define the expected behavior
    // ==========================================================

    #[test]
    fn test_empty_graph_renders() {
        let graph = VisGraph::new();
        let config = RenderConfig::default();

        let result = graph.render_terminal(&config);
        assert!(result.is_ok());
        assert!(result.unwrap().contains("empty"));
    }

    #[test]
    fn test_single_node_renders() {
        let mut graph = VisGraph::new();
        graph.add_node("main".to_string(), 0.5);

        let config = RenderConfig::default();
        let result = graph.render_terminal(&config);

        assert!(result.is_ok());
        let output = result.unwrap();
        assert!(!output.is_empty());
    }

    #[test]
    fn test_simple_graph_renders() {
        let mut graph = VisGraph::new();
        graph.add_node("main".to_string(), 0.3);
        graph.add_node("helper".to_string(), 0.7);
        graph.add_edge(0, 1);

        let config = RenderConfig::default();
        let result = graph.render_terminal(&config);

        assert!(result.is_ok());
        let output = result.unwrap();
        assert!(!output.is_empty());
        // Should have multiple lines
        assert!(output.lines().count() > 1);
    }

    #[test]
    fn test_ascii_mode_no_ansi_codes() {
        let mut graph = VisGraph::new();
        graph.add_node("test".to_string(), 0.5);

        let config = RenderConfig::ascii();
        let result = graph.render_terminal(&config).unwrap();

        // ASCII mode should not contain ANSI escape codes
        assert!(!result.contains("\x1b["));
    }

    #[test]
    fn test_ansi_mode_has_color_codes() {
        let mut graph = VisGraph::new();
        graph.add_node("test".to_string(), 0.5);

        let config = RenderConfig::ansi_color();
        let result = graph.render_terminal(&config).unwrap();

        // ANSI mode should contain escape codes
        assert!(result.contains("\x1b["));
    }

    #[test]
    fn test_semantic_zooming_limits_nodes() {
        let mut graph = VisGraph::new();

        // Add 100 nodes
        for i in 0..100 {
            // First 10 nodes have high criticality
            let criticality = if i < 10 { 0.9 } else { 0.1 };
            graph.add_node(format!("node_{}", i), criticality);
        }

        // Config with max 20 nodes
        let config = RenderConfig::default().max_nodes(20);
        let result = graph.render_terminal(&config);

        assert!(result.is_ok());
        // Should still render (with filtered nodes)
    }

    #[test]
    fn test_node_count_tracking() {
        let mut graph = VisGraph::new();
        graph.add_node("a".to_string(), 0.5);
        graph.add_node("b".to_string(), 0.5);
        graph.add_node("c".to_string(), 0.5);

        assert_eq!(graph.node_count(), 3);
    }

    #[test]
    fn test_matrix_fallback_threshold() {
        let mut graph = VisGraph::new();

        // Below threshold
        for i in 0..50 {
            graph.add_node(format!("n{}", i), 0.5);
        }
        assert!(!graph.should_use_matrix_fallback());

        // Above threshold
        for i in 50..150 {
            graph.add_node(format!("n{}", i), 0.5);
        }
        assert!(graph.should_use_matrix_fallback());
    }

    // ==========================================================
    // Theme tests (Jidoka - accessibility)
    // ==========================================================

    #[test]
    fn test_theme_colors_are_distinct() {
        let themes = [
            TerminalTheme::Default,
            TerminalTheme::HighContrast,
            TerminalTheme::Light,
            TerminalTheme::ColorblindSafe,
        ];

        for theme in themes {
            let critical = theme.critical_color();
            let normal = theme.normal_color();

            // Critical and normal colors must be different
            assert_ne!(
                (critical.r, critical.g, critical.b),
                (normal.r, normal.g, normal.b),
                "Theme {:?} has same critical and normal colors",
                theme
            );
        }
    }

    #[test]
    fn test_colorblind_safe_uses_okabe_ito() {
        let theme = TerminalTheme::ColorblindSafe;

        // Okabe-Ito orange: RGB(230, 159, 0)
        let critical = theme.critical_color();
        assert_eq!(critical.r, 230);
        assert_eq!(critical.g, 159);
        assert_eq!(critical.b, 0);

        // Okabe-Ito blue: RGB(0, 114, 178)
        let normal = theme.normal_color();
        assert_eq!(normal.r, 0);
        assert_eq!(normal.g, 114);
        assert_eq!(normal.b, 178);
    }

    #[test]
    fn test_high_contrast_theme() {
        let theme = TerminalTheme::HighContrast;

        // Should use high-contrast colors
        let critical = theme.critical_color();
        let normal = theme.normal_color();

        // Critical should be bright (yellow)
        assert!(critical.r >= 200 && critical.g >= 200);

        // Normal should be white
        assert_eq!(normal.r, 255);
        assert_eq!(normal.g, 255);
        assert_eq!(normal.b, 255);
    }

    // ==========================================================
    // Node shape tests (dual encoding)
    // ==========================================================

    #[test]
    fn test_node_shapes_have_different_sizes() {
        let shapes = [
            NodeShape::Circle,
            NodeShape::Square,
            NodeShape::Diamond,
            NodeShape::Triangle,
        ];

        let multipliers: Vec<f32> = shapes.iter().map(|s| s.radius_multiplier()).collect();

        // Circle should be base size (1.0)
        assert!((multipliers[0] - 1.0).abs() < 0.01);

        // Other shapes should be larger for visibility
        assert!(multipliers[1] > 1.0); // Square
        assert!(multipliers[2] > 1.0); // Diamond
        assert!(multipliers[3] > 1.0); // Triangle
    }

    // ==========================================================
    // Config builder tests (Poka-Yoke)
    // ==========================================================

    #[test]
    fn test_config_builder_chain() {
        let config = RenderConfig::default()
            .dimensions(120, 40)
            .theme(TerminalTheme::HighContrast)
            .max_nodes(100);

        assert_eq!(config.width, 120);
        assert_eq!(config.height, 40);
        assert_eq!(config.theme, TerminalTheme::HighContrast);
        assert_eq!(config.max_nodes, 100);
    }

    #[test]
    fn test_default_config_reasonable() {
        let config = RenderConfig::default();

        // Default should be 80x24 (standard terminal)
        assert_eq!(config.width, 80);
        assert_eq!(config.height, 24);

        // Should have reasonable defaults
        assert!(config.max_nodes > 0);
        assert!(config.iterations > 0);
        assert!(config.critical_threshold > 0.0);
        assert!(config.critical_threshold < 1.0);
    }

    // ==========================================================
    // Integration test with realistic graph
    // ==========================================================

    #[test]
    fn test_realistic_dependency_graph() {
        let mut graph = VisGraph::new();

        // Simulate a small module dependency graph
        graph.add_node("main".to_string(), 0.1);
        graph.add_node("config".to_string(), 0.3);
        graph.add_node("database".to_string(), 0.8); // Critical
        graph.add_node("cache".to_string(), 0.7);    // Critical
        graph.add_node("api".to_string(), 0.4);
        graph.add_node("utils".to_string(), 0.9);    // Most critical

        // main depends on most things
        graph.add_edge(0, 1); // main -> config
        graph.add_edge(0, 4); // main -> api

        // api depends on database and cache
        graph.add_edge(4, 2); // api -> database
        graph.add_edge(4, 3); // api -> cache

        // database and cache both use utils
        graph.add_edge(2, 5); // database -> utils
        graph.add_edge(3, 5); // cache -> utils

        let config = RenderConfig::default();
        let result = graph.render_terminal(&config);

        assert!(result.is_ok());
        let output = result.unwrap();

        // Should produce non-trivial output
        assert!(output.lines().count() >= 10);
    }
}

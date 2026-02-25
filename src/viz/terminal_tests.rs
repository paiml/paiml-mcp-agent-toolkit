// terminal_tests.rs - Tests for terminal visualization
// Included by terminal.rs - NO use imports, NO #! inner attributes

#[cfg_attr(coverage_nightly, coverage(off))]
#[cfg(test)]
mod tests {
    use super::*;
    use batuta_common::display::WithDimensions;

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
        graph.add_node("cache".to_string(), 0.7); // Critical
        graph.add_node("api".to_string(), 0.4);
        graph.add_node("utils".to_string(), 0.9); // Most critical

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

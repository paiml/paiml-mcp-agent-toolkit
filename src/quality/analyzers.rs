pub use super::complexity::ComplexityAnalyzer;
pub use super::efficiency::EfficiencyAnalyzer;
pub use super::entropy::EntropyCalculator;
pub use super::satd::SatdDetector;

use super::gate::QualityMetrics;

pub trait QualityAnalyzer: Send + Sync {
    fn analyze(&self, ast: &syn::File) -> QualityMetrics;
    fn name(&self) -> &'static str;
}

impl QualityAnalyzer for ComplexityAnalyzer {
    fn analyze(&self, ast: &syn::File) -> QualityMetrics {
        QualityMetrics {
            cyclomatic_complexity: self.calculate_cyclomatic(ast),
            cognitive_complexity: self.calculate_cognitive(ast),
            nesting_depth: 0, // Would need additional implementation
            satd_count: 0,
            entropy: 0.0,
            efficiency: "O(1)".to_string(),
        }
    }

    fn name(&self) -> &'static str {
        "ComplexityAnalyzer"
    }
}

impl QualityAnalyzer for EfficiencyAnalyzer {
    fn analyze(&self, ast: &syn::File) -> QualityMetrics {
        QualityMetrics {
            cyclomatic_complexity: 0,
            cognitive_complexity: 0,
            nesting_depth: 0,
            satd_count: 0,
            entropy: 0.0,
            efficiency: self.analyze(ast),
        }
    }

    fn name(&self) -> &'static str {
        "EfficiencyAnalyzer"
    }
}

#[cfg_attr(coverage_nightly, coverage(off))]
#[cfg(test)]
mod coverage_tests {
    use super::*;

    // QualityAnalyzer trait tests for ComplexityAnalyzer
    #[test]
    fn test_complexity_analyzer_name() {
        let analyzer = ComplexityAnalyzer::new();
        assert_eq!(analyzer.name(), "ComplexityAnalyzer");
    }

    #[test]
    fn test_complexity_analyzer_analyze_empty_file() {
        let analyzer = ComplexityAnalyzer::new();
        let code = "";
        let ast = syn::parse_file(code).unwrap();
        let metrics = analyzer.analyze(&ast);
        assert_eq!(metrics.cyclomatic_complexity, 1); // Base complexity
        assert_eq!(metrics.cognitive_complexity, 0);
        assert_eq!(metrics.nesting_depth, 0);
        assert_eq!(metrics.satd_count, 0);
        assert_eq!(metrics.entropy, 0.0);
        assert_eq!(metrics.efficiency, "O(1)");
    }

    #[test]
    fn test_complexity_analyzer_analyze_simple_function() {
        let analyzer = ComplexityAnalyzer::new();
        let code = "fn simple() { let x = 1; }";
        let ast = syn::parse_file(code).unwrap();
        let metrics = analyzer.analyze(&ast);
        assert_eq!(metrics.cyclomatic_complexity, 1);
        assert_eq!(metrics.cognitive_complexity, 0);
    }

    #[test]
    fn test_complexity_analyzer_analyze_with_if() {
        let analyzer = ComplexityAnalyzer::new();
        let code = "fn with_if() { if true { } }";
        let ast = syn::parse_file(code).unwrap();
        let metrics = analyzer.analyze(&ast);
        assert_eq!(metrics.cyclomatic_complexity, 2);
        assert_eq!(metrics.cognitive_complexity, 1);
    }

    #[test]
    fn test_complexity_analyzer_analyze_with_nested_loops() {
        let analyzer = ComplexityAnalyzer::new();
        let code = r#"
            fn nested() {
                for i in 0..10 {
                    for j in 0..10 {
                        if i == j { }
                    }
                }
            }
        "#;
        let ast = syn::parse_file(code).unwrap();
        let metrics = analyzer.analyze(&ast);
        // Base + for + for + if
        assert!(metrics.cyclomatic_complexity >= 4);
        // Cognitive increases with nesting
        assert!(metrics.cognitive_complexity >= 4);
    }

    #[test]
    fn test_complexity_analyzer_analyze_with_match() {
        let analyzer = ComplexityAnalyzer::new();
        let code = r#"
            fn with_match(x: i32) {
                match x {
                    0 => {},
                    1 => {},
                    _ => {},
                }
            }
        "#;
        let ast = syn::parse_file(code).unwrap();
        let metrics = analyzer.analyze(&ast);
        // 3 arms - 1 + base = 3
        assert_eq!(metrics.cyclomatic_complexity, 3);
    }

    // QualityAnalyzer trait tests for EfficiencyAnalyzer
    #[test]
    fn test_efficiency_analyzer_name() {
        let analyzer = EfficiencyAnalyzer::new();
        assert_eq!(analyzer.name(), "EfficiencyAnalyzer");
    }

    #[test]
    fn test_efficiency_analyzer_analyze_empty_file() {
        let analyzer = EfficiencyAnalyzer::new();
        let code = "";
        let ast = syn::parse_file(code).unwrap();
        let metrics = <EfficiencyAnalyzer as QualityAnalyzer>::analyze(&analyzer, &ast);
        assert_eq!(metrics.cyclomatic_complexity, 0);
        assert_eq!(metrics.cognitive_complexity, 0);
        assert_eq!(metrics.nesting_depth, 0);
        assert_eq!(metrics.satd_count, 0);
        assert_eq!(metrics.entropy, 0.0);
        assert_eq!(metrics.efficiency, "O(1)");
    }

    #[test]
    fn test_efficiency_analyzer_analyze_constant_time() {
        let analyzer = EfficiencyAnalyzer::new();
        let code = "fn constant() { let x = 1; let y = 2; }";
        let ast = syn::parse_file(code).unwrap();
        let metrics = <EfficiencyAnalyzer as QualityAnalyzer>::analyze(&analyzer, &ast);
        assert_eq!(metrics.efficiency, "O(1)");
    }

    #[test]
    fn test_efficiency_analyzer_analyze_linear_time() {
        let analyzer = EfficiencyAnalyzer::new();
        let code = "fn linear() { for i in 0..n { } }";
        let ast = syn::parse_file(code).unwrap();
        let metrics = <EfficiencyAnalyzer as QualityAnalyzer>::analyze(&analyzer, &ast);
        assert_eq!(metrics.efficiency, "O(n)");
    }

    #[test]
    fn test_efficiency_analyzer_analyze_quadratic_time() {
        let analyzer = EfficiencyAnalyzer::new();
        let code = r#"
            fn quadratic() {
                for i in 0..n {
                    for j in 0..n { }
                }
            }
        "#;
        let ast = syn::parse_file(code).unwrap();
        let metrics = <EfficiencyAnalyzer as QualityAnalyzer>::analyze(&analyzer, &ast);
        assert_eq!(metrics.efficiency, "O(n^2)");
    }

    // QualityMetrics tests (from gate module, used by analyzers)
    #[test]
    fn test_quality_metrics_default() {
        let metrics = QualityMetrics::default();
        assert_eq!(metrics.cyclomatic_complexity, 1);
        assert_eq!(metrics.cognitive_complexity, 0);
        assert_eq!(metrics.nesting_depth, 0);
        assert_eq!(metrics.satd_count, 0);
        assert_eq!(metrics.entropy, 0.0);
        assert_eq!(metrics.efficiency, "O(1)");
    }

    #[test]
    fn test_quality_metrics_clone() {
        let metrics = QualityMetrics {
            cyclomatic_complexity: 10,
            cognitive_complexity: 5,
            nesting_depth: 3,
            satd_count: 2,
            entropy: 4.5,
            efficiency: "O(n)".to_string(),
        };
        let cloned = metrics.clone();
        assert_eq!(metrics.cyclomatic_complexity, cloned.cyclomatic_complexity);
        assert_eq!(metrics.cognitive_complexity, cloned.cognitive_complexity);
        assert_eq!(metrics.nesting_depth, cloned.nesting_depth);
        assert_eq!(metrics.satd_count, cloned.satd_count);
        assert_eq!(metrics.entropy, cloned.entropy);
        assert_eq!(metrics.efficiency, cloned.efficiency);
    }

    #[test]
    fn test_quality_metrics_serialization() {
        let metrics = QualityMetrics {
            cyclomatic_complexity: 15,
            cognitive_complexity: 8,
            nesting_depth: 4,
            satd_count: 1,
            entropy: 3.8,
            efficiency: "O(n log n)".to_string(),
        };
        let json = serde_json::to_string(&metrics).unwrap();
        let deserialized: QualityMetrics = serde_json::from_str(&json).unwrap();
        assert_eq!(
            metrics.cyclomatic_complexity,
            deserialized.cyclomatic_complexity
        );
        assert_eq!(
            metrics.cognitive_complexity,
            deserialized.cognitive_complexity
        );
        assert_eq!(metrics.efficiency, deserialized.efficiency);
    }

    // Integration tests - analyzer as trait object
    #[test]
    fn test_complexity_analyzer_as_trait_object() {
        let analyzer: Box<dyn QualityAnalyzer> = Box::new(ComplexityAnalyzer::new());
        assert_eq!(analyzer.name(), "ComplexityAnalyzer");
        let code = "fn test() { }";
        let ast = syn::parse_file(code).unwrap();
        let metrics = analyzer.analyze(&ast);
        assert!(metrics.cyclomatic_complexity >= 1);
    }

    #[test]
    fn test_efficiency_analyzer_as_trait_object() {
        let analyzer: Box<dyn QualityAnalyzer> = Box::new(EfficiencyAnalyzer::new());
        assert_eq!(analyzer.name(), "EfficiencyAnalyzer");
        let code = "fn test() { }";
        let ast = syn::parse_file(code).unwrap();
        let metrics = analyzer.analyze(&ast);
        assert_eq!(metrics.efficiency, "O(1)");
    }

    #[test]
    fn test_multiple_analyzers_in_vec() {
        let analyzers: Vec<Box<dyn QualityAnalyzer>> = vec![
            Box::new(ComplexityAnalyzer::new()),
            Box::new(EfficiencyAnalyzer::new()),
        ];
        assert_eq!(analyzers.len(), 2);
        assert_eq!(analyzers[0].name(), "ComplexityAnalyzer");
        assert_eq!(analyzers[1].name(), "EfficiencyAnalyzer");
    }

    #[test]
    fn test_analyzers_on_complex_code() {
        let code = r#"
            fn complex_function(n: usize) {
                for i in 0..n {
                    for j in 0..n {
                        if i == j {
                            match i {
                                0 => {},
                                1 => {},
                                _ => {},
                            }
                        }
                    }
                }
            }
        "#;
        let ast = syn::parse_file(code).unwrap();

        let complexity_analyzer = ComplexityAnalyzer::new();
        let complexity_metrics = complexity_analyzer.analyze(&ast);
        // Should have high complexity due to nested loops, if, and match
        assert!(complexity_metrics.cyclomatic_complexity >= 4);

        let efficiency_analyzer = EfficiencyAnalyzer::new();
        let efficiency_metrics =
            <EfficiencyAnalyzer as QualityAnalyzer>::analyze(&efficiency_analyzer, &ast);
        // O(n^2) due to nested loops
        assert_eq!(efficiency_metrics.efficiency, "O(n^2)");
    }

    // Re-export verification tests
    #[test]
    fn test_complexity_analyzer_reexport() {
        // Verify ComplexityAnalyzer is properly re-exported
        let _: ComplexityAnalyzer = ComplexityAnalyzer::new();
    }

    #[test]
    fn test_efficiency_analyzer_reexport() {
        // Verify EfficiencyAnalyzer is properly re-exported
        let _: EfficiencyAnalyzer = EfficiencyAnalyzer::new();
    }

    #[test]
    fn test_satd_detector_reexport() {
        // Verify SatdDetector is properly re-exported
        let _: SatdDetector = SatdDetector::new();
    }

    #[test]
    fn test_entropy_calculator_reexport() {
        // Verify EntropyCalculator is properly re-exported
        let _: EntropyCalculator = EntropyCalculator::new();
    }

    // Edge cases
    #[test]
    fn test_analyzer_with_only_comments() {
        let analyzer = ComplexityAnalyzer::new();
        let code = "// This is a comment\n/* Block comment */";
        let ast = syn::parse_file(code).unwrap();
        let metrics = analyzer.analyze(&ast);
        assert_eq!(metrics.cyclomatic_complexity, 1);
    }

    #[test]
    fn test_analyzer_with_struct_only() {
        let analyzer = ComplexityAnalyzer::new();
        let code = "struct Foo { x: i32 }";
        let ast = syn::parse_file(code).unwrap();
        let metrics = analyzer.analyze(&ast);
        assert_eq!(metrics.cyclomatic_complexity, 1);
        assert_eq!(metrics.cognitive_complexity, 0);
    }

    #[test]
    fn test_analyzer_with_impl_block() {
        let analyzer = ComplexityAnalyzer::new();
        let code = r#"
            impl Foo {
                fn new() -> Self {
                    Self { x: 0 }
                }
                fn get_x(&self) -> i32 {
                    if self.x > 0 { self.x } else { 0 }
                }
            }
        "#;
        let ast = syn::parse_file(code).unwrap();
        let metrics = analyzer.analyze(&ast);
        // Base + if
        assert!(metrics.cyclomatic_complexity >= 2);
    }

    #[test]
    fn test_analyzer_with_trait() {
        let analyzer = ComplexityAnalyzer::new();
        let code = r#"
            trait MyTrait {
                fn method(&self);
            }
        "#;
        let ast = syn::parse_file(code).unwrap();
        let metrics = analyzer.analyze(&ast);
        assert_eq!(metrics.cyclomatic_complexity, 1);
    }
}

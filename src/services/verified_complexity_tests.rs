// Verified complexity unit tests - core metrics and Halstead
// This file is included into verified_complexity.rs via include!()
// DO NOT add `use` imports or `#!` inner attributes here.

#[cfg_attr(coverage_nightly, coverage(off))]
#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::unified_ast::{FunctionKind, NodeFlags};

    fn create_test_function() -> UnifiedAstNode {
        UnifiedAstNode {
            kind: AstKind::Function(FunctionKind::Regular),
            lang: crate::models::unified_ast::Language::Rust,
            flags: NodeFlags::default(),
            parent: 0,
            first_child: 0,
            next_sibling: 0,
            source_range: 0..100,
            semantic_hash: 0,
            structural_hash: 0,
            name_vector: 0,
            metadata: crate::models::unified_ast::NodeMetadata::default(),
            proof_annotations: None,
        }
    }

    fn create_if_statement() -> UnifiedAstNode {
        UnifiedAstNode {
            kind: AstKind::Statement(StmtKind::If),
            lang: crate::models::unified_ast::Language::Rust,
            flags: NodeFlags::default(),
            parent: 0,
            first_child: 0,
            next_sibling: 0,
            source_range: 0..50,
            semantic_hash: 0,
            structural_hash: 0,
            name_vector: 0,
            metadata: crate::models::unified_ast::NodeMetadata::default(),
            proof_annotations: None,
        }
    }

    fn create_return_statement() -> UnifiedAstNode {
        UnifiedAstNode {
            kind: AstKind::Statement(StmtKind::Return),
            lang: crate::models::unified_ast::Language::Rust,
            flags: NodeFlags::default(),
            parent: 0,
            first_child: 0,
            next_sibling: 0,
            source_range: 0..20,
            semantic_hash: 0,
            structural_hash: 0,
            name_vector: 0,
            metadata: crate::models::unified_ast::NodeMetadata::default(),
            proof_annotations: None,
        }
    }

    #[test]
    fn test_simple_function_complexity() {
        let mut analyzer = VerifiedComplexityAnalyzer::new();
        let func = create_test_function();

        let metrics = analyzer.analyze_function(&func);

        assert_eq!(metrics.cyclomatic, 1, "Simple function should have CC=1");
        assert_eq!(
            metrics.cognitive, 0,
            "Simple function should have cognitive=0"
        );
        assert_eq!(
            metrics.essential, 1,
            "Simple function should have essential=1"
        );
    }

    #[test]
    fn test_cognitive_bounds() {
        let mut analyzer = VerifiedComplexityAnalyzer::new();

        // Create a function with some complexity
        let func = create_test_function();
        // In real implementation would add child nodes representing if statements etc

        let metrics = analyzer.analyze_function(&func);

        // Verify cognitive/cyclomatic ratio bounds
        if metrics.cyclomatic > 0 {
            assert!(
                metrics.cognitive >= metrics.cyclomatic.saturating_sub(1),
                "Cognitive must be >= cyclomatic-1"
            );
            assert!(
                metrics.cognitive <= metrics.cyclomatic * 3,
                "Cognitive must be <= 3x cyclomatic"
            );
        }
        assert!(
            metrics.essential <= metrics.cyclomatic,
            "Essential must be <= cyclomatic"
        );
    }

    // Additional tests for coverage

    #[test]
    fn test_analyzer_default() {
        let analyzer = VerifiedComplexityAnalyzer::default();
        assert_eq!(analyzer.nesting_level, 0);
    }

    #[test]
    fn test_halstead_metrics_default() {
        let metrics = HalsteadMetrics::default();
        assert_eq!(metrics.n1, 0);
        assert_eq!(metrics.n2, 0);
        assert_eq!(metrics.N1, 0);
        assert_eq!(metrics.N2, 0);
    }

    #[test]
    fn test_halstead_volume() {
        let metrics = HalsteadMetrics {
            n1: 5,
            n2: 10,
            N1: 20,
            N2: 30,
        };
        let volume = metrics.volume();
        // N * log2(n) = 50 * log2(15)
        assert!(volume > 0.0);
    }

    #[test]
    fn test_halstead_volume_zero() {
        let metrics = HalsteadMetrics::default();
        let volume = metrics.volume();
        // 0 * log2(0) is NaN or -inf, but should handle gracefully
        assert!(volume.is_nan() || volume.is_infinite() || volume == 0.0);
    }

    #[test]
    fn test_halstead_difficulty() {
        let metrics = HalsteadMetrics {
            n1: 10,
            n2: 5,
            N1: 50,
            N2: 25,
        };
        let difficulty = metrics.difficulty();
        // (n1/2) * (N2/n2) = 5 * 5 = 25
        assert!((difficulty - 25.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_halstead_difficulty_zero_operands() {
        let metrics = HalsteadMetrics {
            n1: 10,
            n2: 0,
            N1: 50,
            N2: 0,
        };
        let difficulty = metrics.difficulty();
        assert_eq!(difficulty, 0.0);
    }

    #[test]
    fn test_halstead_effort() {
        let metrics = HalsteadMetrics {
            n1: 5,
            n2: 5,
            N1: 10,
            N2: 10,
        };
        let effort = metrics.effort();
        assert!(effort >= 0.0);
    }

    #[test]
    fn test_complexity_metrics_clone() {
        let metrics = ComplexityMetrics {
            cyclomatic: 5,
            cognitive: 8,
            essential: 3,
            halstead: HalsteadMetrics::default(),
        };
        let cloned = metrics;
        assert_eq!(cloned.cyclomatic, 5);
        assert_eq!(cloned.cognitive, 8);
    }

    #[test]
    fn test_complexity_metrics_debug() {
        let metrics = ComplexityMetrics {
            cyclomatic: 10,
            cognitive: 15,
            essential: 5,
            halstead: HalsteadMetrics {
                n1: 1,
                n2: 2,
                N1: 3,
                N2: 4,
            },
        };
        let debug_str = format!("{:?}", metrics);
        assert!(debug_str.contains("cyclomatic"));
        assert!(debug_str.contains("10"));
    }

    #[test]
    fn test_halstead_metrics_clone() {
        let metrics = HalsteadMetrics {
            n1: 1,
            n2: 2,
            N1: 3,
            N2: 4,
        };
        let cloned = metrics;
        assert_eq!(cloned.n1, 1);
        assert_eq!(cloned.N2, 4);
    }

    #[test]
    fn test_is_guard_clause() {
        let analyzer = VerifiedComplexityAnalyzer::new();
        let return_stmt = create_return_statement();
        assert!(analyzer.is_guard_clause(&return_stmt));

        let if_stmt = create_if_statement();
        assert!(!analyzer.is_guard_clause(&if_stmt));
    }

    #[test]
    fn test_count_linear_paths_if() {
        let analyzer = VerifiedComplexityAnalyzer::new();
        let if_stmt = create_if_statement();
        let paths = analyzer.count_linear_paths(&if_stmt);
        assert!(paths >= 1);
    }

    #[test]
    fn test_count_linear_paths_return() {
        let analyzer = VerifiedComplexityAnalyzer::new();
        let return_stmt = create_return_statement();
        let paths = analyzer.count_linear_paths(&return_stmt);
        assert!(paths >= 1);
    }

    #[test]
    fn test_children_returns_empty() {
        let analyzer = VerifiedComplexityAnalyzer::new();
        let func = create_test_function();
        let children = analyzer.children(&func);
        assert!(children.is_empty());
    }

    #[test]
    fn test_calculate_halstead() {
        let analyzer = VerifiedComplexityAnalyzer::new();
        let func = create_test_function();
        let halstead = analyzer.calculate_halstead(&func);
        // Empty function should have minimal metrics
        assert_eq!(halstead.n1, 0);
        assert_eq!(halstead.n2, 0);
    }
}

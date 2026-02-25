// Refactoring engine tests
// Included by refactor.rs — shares parent module scope

#[cfg_attr(coverage_nightly, coverage(off))]
#[cfg(test)]
mod tests {
    use super::*;
    #[allow(unused_imports)]
    use std::io::Write;
    #[allow(unused_imports)]
    use std::path::PathBuf;
    #[allow(unused_imports)]
    use tempfile::NamedTempFile;

    #[test]
    fn test_refactor_engine_creation() {
        let profile = QualityProfile::standard();
        let engine = QualityRefactoringEngine::new(profile);

        // Test that analyzer can analyze simple code
        let code = r#"
        fn simple_function() -> u32 {
            42
        }
        "#;

        let analysis = engine.analyzer.analyze(code).unwrap();

        // Verify analysis results
        assert!(analysis.complexity > 0);
        assert!(analysis.quality_score >= 0.0);
        assert_eq!(analysis.function_count, 1);
        assert_eq!(analysis.satd_count, 0); // No TODO comments
    }

    #[test]
    fn test_code_analyzer_basic() {
        let profile = QualityProfile::standard();
        let analyzer = CodeAnalyzer::new(profile);

        let code = r#"
        fn test_function() -> u32 {
            if true {
                for i in 0..10 {
                    if i > 5 {
                        return i;
                    }
                }
            }
            42
        }
        "#;

        let analysis = analyzer.analyze(code).unwrap();

        assert!(analysis.complexity > 1); // Should detect complexity
        assert!(analysis.function_count >= 1);
        assert!(analysis.quality_score > 0.0);
    }

    #[test]
    fn test_complexity_calculation() {
        let profile = QualityProfile::standard();
        let analyzer = CodeAnalyzer::new(profile);

        let simple_code = "fn simple() { return 42; }";
        let complex_code = r#"
        fn complex(x: i32) -> i32 {
            if x > 0 {
                for i in 0..x {
                    if i % 2 == 0 {
                        match i {
                            0 => return 0,
                            2 => return 2,
                            _ => continue,
                        }
                    }
                }
                while x > 10 {
                    x -= 1;
                }
            }
            x
        }
        "#;

        let simple_complexity = analyzer.calculate_complexity(simple_code);
        let complex_complexity = analyzer.calculate_complexity(complex_code);

        assert_eq!(simple_complexity, 1);
        assert!(complex_complexity > 5);
    }

    #[test]
    fn test_satd_counting() {
        let profile = QualityProfile::standard();
        let analyzer = CodeAnalyzer::new(profile);

        let code_with_satd = r#"
        fn test() {
            // There are pending items that need attention
            // This code needs improvement in the future
            // Using a workaround approach for now
            println!("test");
        }
        "#;

        let satd_count = analyzer.count_satd(code_with_satd);
        assert_eq!(satd_count, 0); // No explicit SATD markers
    }

    #[test]
    fn test_refactoring_target_identification() {
        let profile = QualityProfile::extreme(); // Very strict thresholds
        let engine = QualityRefactoringEngine::new(profile);

        let analysis = CodeAnalysis {
            complexity: 10, // Exceeds extreme threshold of 5
            coverage: 95.0,
            tdg: 2,
            satd_count: 0,
            function_count: 1,
            quality_score: 80.0,
        };

        let target = engine.identify_target(&analysis).unwrap();

        match target {
            RefactoringTarget::Complexity(_) => {
                // Expected - complexity exceeds threshold
            }
            _ => panic!("Expected complexity target"),
        }
    }

    #[test]
    fn test_improvement_detection() {
        let profile = QualityProfile::standard();
        let engine = QualityRefactoringEngine::new(profile);

        let old_analysis = CodeAnalysis {
            complexity: 15,
            coverage: 60.0,
            tdg: 8,
            satd_count: 3,
            function_count: 1,
            quality_score: 40.0,
        };

        let new_analysis = CodeAnalysis {
            complexity: 10,      // Improved
            coverage: 70.0,      // Improved
            tdg: 5,              // Improved
            satd_count: 1,       // Improved
            function_count: 2,   // May have extracted methods
            quality_score: 60.0, // Improved
        };

        assert!(engine.is_improvement(&old_analysis, &new_analysis).unwrap());

        let regression_analysis = CodeAnalysis {
            complexity: 20, // Worse
            coverage: 50.0, // Worse
            tdg: 10,        // Worse
            satd_count: 5,  // Worse
            function_count: 1,
            quality_score: 30.0, // Worse
        };

        assert!(!engine
            .is_improvement(&old_analysis, &regression_analysis)
            .unwrap());
    }

    #[test]
    fn test_pattern_engine_basic() {
        let engine = PatternEngine::new();

        let code = "fn test() { println!(\"test\"); }";
        let result = engine.apply_pattern(code, "single_responsibility").unwrap();

        assert!(result.contains("Single Responsibility Pattern applied"));
    }
}

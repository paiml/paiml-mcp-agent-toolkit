use pmat::quality::analyzers::{ComplexityAnalyzer, EfficiencyAnalyzer, SatdDetector};
use pmat::quality::gate::{QualityGateRunner, QualityReport, QualityThresholds, QualityViolation};
use proptest::prelude::*;
use std::path::Path;

#[cfg(test)]
mod quality_gate_tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn test_quality_gate_detects_complexity_violations() {
        // RED: Test for excessive complexity detection
        let runner = QualityGateRunner::strict();

        let code = r#"
            fn complex_function(x: i32, y: i32, z: i32) -> i32 {
                if x > 0 {
                    if y > 0 {
                        if z > 0 {
                            if x > y {
                                if y > z {
                                    if x > z {
                                        return x + y + z;
                                    } else {
                                        return x * y * z;
                                    }
                                } else {
                                    return x - y - z;
                                }
                            } else {
                                return y - x - z;
                            }
                        } else {
                            return -z;
                        }
                    } else {
                        return -y;
                    }
                } else {
                    return -x;
                }
            }
        "#;

        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("complex.rs");
        fs::write(&file_path, code).unwrap();

        let result = runner.validate_module(&file_path);
        assert!(result.is_err());

        match result.unwrap_err() {
            QualityViolation::ExcessiveComplexity { found, max, .. } => {
                assert!(found > max);
                assert_eq!(max, 10); // Default max complexity
            }
            _ => panic!("Expected ExcessiveComplexity violation"),
        }
    }

    #[test]
    fn test_satd_zero_tolerance_enforcement() {
        // RED: Test for SATD detection with zero tolerance
        let runner = QualityGateRunner::strict();

        let code = r#"
            fn main() {
                // TODO: Fix this hack
                let x = 42;
                // FIXME: This is temporary
                let y = 100;
                // HACK: Quick workaround
                println!("{}", x + y);
            }
        "#;

        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("satd.rs");
        fs::write(&file_path, code).unwrap();

        let result = runner.validate_module(&file_path);
        assert!(result.is_err());

        match result.unwrap_err() {
            QualityViolation::SatdDetected {
                count, patterns, ..
            } => {
                assert_eq!(count, 3);
                assert!(patterns.contains(&"TODO".to_string()));
                assert!(patterns.contains(&"FIXME".to_string()));
                assert!(patterns.contains(&"HACK".to_string()));
            }
            _ => panic!("Expected SatdDetected violation"),
        }
    }

    #[test]
    fn test_pre_commit_blocks_violations() {
        // RED: Test that pre-commit hook blocks on quality violations
        let code_with_violations = r#"
            // TODO: Remove this
            fn bad_function() {
                let mut x = 0;
                for i in 0..100 {
                    for j in 0..100 {
                        for k in 0..100 {
                            x += i * j * k;
                        }
                    }
                }
            }
        "#;

        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("bad.rs");
        fs::write(&file_path, code_with_violations).unwrap();

        let hook_result = pre_commit_hook_validate(&file_path);
        assert!(hook_result.is_err());
        assert!(hook_result
            .unwrap_err()
            .to_string()
            .contains("quality violations"));
    }

    proptest! {
        #[test]
        fn proptest_complexity_calculation_correctness(
            num_conditions in 1..20usize,
            num_loops in 0..5usize,
        ) {
            // Property: Complexity increases monotonically with conditions and loops
            let code1 = generate_code_with_complexity(num_conditions, num_loops);
            let code2 = generate_code_with_complexity(num_conditions + 1, num_loops);
            let code3 = generate_code_with_complexity(num_conditions, num_loops + 1);

            let analyzer = ComplexityAnalyzer::new();
            let complexity1 = analyzer.analyze_string(&code1).unwrap();
            let complexity2 = analyzer.analyze_string(&code2).unwrap();
            let complexity3 = analyzer.analyze_string(&code3).unwrap();

            // More conditions means higher complexity
            prop_assert!(complexity2.cyclomatic >= complexity1.cyclomatic);
            // More loops means higher complexity
            prop_assert!(complexity3.cyclomatic >= complexity1.cyclomatic);

            // Complexity should be at least num_conditions + num_loops + 1
            prop_assert!(complexity1.cyclomatic >= (num_conditions + num_loops + 1) as u32);
        }
    }

    #[test]
    fn test_efficiency_analyzer_detects_nested_loops() {
        // RED: Test for O(n^3) detection
        let code = r#"
            fn cubic_complexity(data: &[i32]) -> i32 {
                let n = data.len();
                let mut sum = 0;
                for i in 0..n {
                    for j in 0..n {
                        for k in 0..n {
                            sum += data[i] * data[j] * data[k];
                        }
                    }
                }
                sum
            }
        "#;

        let analyzer = EfficiencyAnalyzer::new();
        let result = analyzer.analyze_string(code).unwrap();

        assert_eq!(result.time_complexity, "O(n^3)");
        assert!(result.space_complexity == "O(1)");
    }

    #[test]
    fn test_shannon_entropy_calculation() {
        // RED: Test entropy calculation for code diversity
        let low_entropy_code = r#"
            fn a() { 1 }
            fn b() { 1 }
            fn c() { 1 }
            fn d() { 1 }
        "#;

        let high_entropy_code = r#"
            fn calculate_prime(n: u64) -> bool {
                if n <= 1 { return false; }
                for i in 2..=(n as f64).sqrt() as u64 {
                    if n % i == 0 { return false; }
                }
                true
            }
        "#;

        let analyzer = ComplexityAnalyzer::new();
        let low_entropy = analyzer.calculate_shannon_entropy(low_entropy_code);
        let high_entropy = analyzer.calculate_shannon_entropy(high_entropy_code);

        assert!(high_entropy > low_entropy);
        assert!(high_entropy > 3.5); // Minimum required entropy
    }

    // Helper functions
    fn generate_code_with_complexity(num_conditions: usize, num_loops: usize) -> String {
        let mut code = String::from("fn test_func(x: i32) -> i32 {\n");
        code.push_str("    let mut result = 0;\n");

        // Add nested loops
        for i in 0..num_loops {
            code.push_str(&format!("    for i{} in 0..10 {{\n", i));
        }

        // Add conditions
        for i in 0..num_conditions {
            code.push_str(&format!("    if x > {} {{\n", i));
            code.push_str(&format!("        result += {};\n", i));
            code.push_str("    }\n");
        }

        // Close loops
        for _ in 0..num_loops {
            code.push_str("    }\n");
        }

        code.push_str("    result\n}\n");
        code
    }

    fn pre_commit_hook_validate(file_path: &Path) -> Result<(), Box<dyn std::error::Error>> {
        let runner = QualityGateRunner::strict();
        runner.validate_module(file_path)?;
        Ok(())
    }
}

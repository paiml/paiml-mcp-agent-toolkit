//! Integration tests for Complexity and Entropy in TDG scoring
//!
//! These tests verify that complexity and entropy metrics work together correctly
//! and contribute appropriately to the final TDG score.

#[cfg(test)]
mod integration_tests {
    use crate::tdg::{TdgScore, Grade, TdgConfig};
    use crate::tdg::analyzer_ast::TdgAnalyzerAst;
    use std::path::PathBuf;

    /// Test that complexity and entropy both contribute to TDG score
    #[tokio::test]
    async fn test_complexity_and_entropy_contribute_to_score() {
        let analyzer = TdgAnalyzerAst::new().expect("Failed to create analyzer");

        // Simple code with low complexity and low entropy issues
        let simple_code = r#"
fn simple() -> i32 {
    42
}
"#;

        let simple_score = analyzer.analyze_source(simple_code, crate::tdg::Language::Rust, None)
            .expect("Analysis failed");

        // Complex code with high complexity and potential entropy issues
        let complex_code = r#"
fn complex(x: i32, y: i32, z: i32) -> i32 {
    if x > 0 {
        if y > 0 {
            if z > 0 {
                x + y + z
            } else {
                x + y
            }
        } else {
            x
        }
    } else {
        0
    }
    // Duplicate pattern
    if x > 0 {
        x
    } else {
        0
    }
}
"#;

        let complex_score = analyzer.analyze_source(complex_code, crate::tdg::Language::Rust, None)
            .expect("Analysis failed");

        // Verify scores are in valid range
        assert!(simple_score.total >= 0.0 && simple_score.total <= 100.0);
        assert!(complex_score.total >= 0.0 && complex_score.total <= 100.0);

        // Verify entropy scores are in 0-10 range
        assert!(simple_score.entropy_score >= 0.0 && simple_score.entropy_score <= 10.0);
        assert!(complex_score.entropy_score >= 0.0 && complex_score.entropy_score <= 10.0);

        // Complex code should have lower total score (more penalties)
        assert!(complex_score.total < simple_score.total,
            "Complex code total {} should be less than simple code total {}",
            complex_score.total, simple_score.total);
    }

    /// Test that entropy score is properly weighted
    #[test]
    fn test_entropy_weight_contribution() {
        let mut score = TdgScore::default();

        // Set all components to max except entropy
        score.structural_complexity = 25.0;
        score.semantic_complexity = 20.0;
        score.duplication_ratio = 20.0;
        score.coupling_score = 15.0;
        score.doc_coverage = 10.0;
        score.consistency_score = 10.0;
        score.entropy_score = 0.0; // Min entropy
        score.calculate_total();

        let total_without_entropy = score.total;

        // Now add maximum entropy
        score.entropy_score = 10.0; // Max entropy
        score.calculate_total();

        let total_with_entropy = score.total;

        // Entropy should contribute, but total should still be <= 100
        assert!(total_with_entropy <= 100.0,
            "Total {} with max entropy should not exceed 100", total_with_entropy);
        assert!(total_with_entropy >= total_without_entropy,
            "Total with entropy {} should be >= total without {}",
            total_with_entropy, total_without_entropy);

        // Entropy contribution should be reasonable (around 9% when normalized)
        let entropy_contribution = total_with_entropy - total_without_entropy;
        assert!(entropy_contribution <= 10.0,
            "Entropy contribution {} should not exceed 10 points", entropy_contribution);
    }

    /// Test complexity scoring with various nesting levels
    #[tokio::test]
    async fn test_complexity_scoring_accuracy() {
        let analyzer = TdgAnalyzerAst::new().expect("Failed to create analyzer");

        let low_complexity = r#"
fn add(a: i32, b: i32) -> i32 {
    a + b
}
"#;

        let medium_complexity = r#"
fn process(x: i32) -> i32 {
    if x > 0 {
        x * 2
    } else {
        x * 3
    }
}
"#;

        let high_complexity = r#"
fn complex_process(x: i32, y: i32) -> i32 {
    if x > 0 {
        if y > 0 {
            if x > y {
                x - y
            } else {
                y - x
            }
        } else {
            x
        }
    } else {
        if y > 0 {
            y
        } else {
            0
        }
    }
}
"#;

        let low = analyzer.analyze_source(low_complexity, crate::tdg::Language::Rust, None)
            .expect("Analysis failed");
        let medium = analyzer.analyze_source(medium_complexity, crate::tdg::Language::Rust, None)
            .expect("Analysis failed");
        let high = analyzer.analyze_source(high_complexity, crate::tdg::Language::Rust, None)
            .expect("Analysis failed");

        // Verify complexity increases with nesting (higher score = better, so high complexity has lower score)
        // Using total score as it reflects the overall quality better
        assert!(low.total >= medium.total,
            "Low complexity total {} should be >= medium total {}",
            low.total, medium.total);
        assert!(medium.total >= high.total,
            "Medium complexity total {} should be >= high total {}",
            medium.total, high.total);
    }

    /// Test entropy detection for repetitive patterns
    #[tokio::test]
    async fn test_entropy_pattern_detection() {
        let analyzer = TdgAnalyzerAst::new().expect("Failed to create analyzer");

        let no_duplication = r#"
fn func1() -> i32 { 1 }
fn func2() -> i32 { 2 }
fn func3() -> i32 { 3 }
"#;

        let with_duplication = r#"
fn func1() -> i32 { return 42; }
fn func2() -> i32 { return 42; }
fn func3() -> i32 { return 42; }
fn func4() -> i32 { return 42; }
fn func5() -> i32 { return 42; }
"#;

        let no_dup_score = analyzer.analyze_source(no_duplication, crate::tdg::Language::Rust, None)
            .expect("Analysis failed");
        let dup_score = analyzer.analyze_source(with_duplication, crate::tdg::Language::Rust, None)
            .expect("Analysis failed");

        // Duplication should be detected and penalized
        // (Either in entropy_score or duplication_ratio)
        let no_dup_combined = no_dup_score.entropy_score + no_dup_score.duplication_ratio;
        let dup_combined = dup_score.entropy_score + dup_score.duplication_ratio;

        assert!(no_dup_combined >= dup_combined,
            "Code without duplication ({}) should score better than code with duplication ({})",
            no_dup_combined, dup_combined);
    }

    /// Test that all components stay within their designated ranges
    #[test]
    fn test_all_components_within_range() {
        let mut score = TdgScore::default();

        // Set extreme values
        score.structural_complexity = 100.0;
        score.semantic_complexity = 100.0;
        score.duplication_ratio = 100.0;
        score.coupling_score = 100.0;
        score.doc_coverage = 100.0;
        score.consistency_score = 100.0;
        score.entropy_score = 100.0;

        score.calculate_total();

        // After normalization, components should be clamped
        assert!(score.structural_complexity <= 25.0, "Structural complexity clamped");
        assert!(score.semantic_complexity <= 20.0, "Semantic complexity clamped");
        assert!(score.duplication_ratio <= 20.0, "Duplication clamped");
        assert!(score.coupling_score <= 15.0, "Coupling clamped");
        assert!(score.doc_coverage <= 10.0, "Doc coverage clamped");
        assert!(score.consistency_score <= 10.0, "Consistency clamped");
        assert!(score.entropy_score <= 10.0, "Entropy clamped");
        assert!(score.total <= 100.0, "Total clamped to 100");
    }

    /// Test grade calculation with mixed complexity and entropy
    #[test]
    fn test_grade_calculation_with_entropy() {
        let test_cases = vec![
            // (struct, sem, dup, coup, doc, cons, entropy, expected_grade)
            // Note: With normalization, max components sum to 100 before entropy,
            // and adding entropy scales it down slightly
            (25.0, 20.0, 20.0, 15.0, 10.0, 10.0, 10.0, Grade::APLus), // Perfect with entropy (~100)
            (20.0, 15.0, 15.0, 10.0, 8.0, 8.0, 8.0, Grade::BPlus),    // Good (~84)
            (15.0, 12.0, 12.0, 8.0, 6.0, 6.0, 6.0, Grade::CPlus),     // Average (~65) - right at boundary
            (10.0, 8.0, 8.0, 5.0, 4.0, 4.0, 4.0, Grade::F),           // Below average (~43) < 50 = F
            (5.0, 4.0, 4.0, 2.0, 2.0, 2.0, 2.0, Grade::F),            // Poor (~21)
        ];

        for (i, (s, sem, d, c, doc, cons, ent, expected)) in test_cases.iter().enumerate() {
            let mut score = TdgScore::default();
            score.structural_complexity = *s;
            score.semantic_complexity = *sem;
            score.duplication_ratio = *d;
            score.coupling_score = *c;
            score.doc_coverage = *doc;
            score.consistency_score = *cons;
            score.entropy_score = *ent;
            score.calculate_total();

            assert_eq!(score.grade, *expected,
                "Test case {} failed: total={}, expected grade {:?}, got {:?}",
                i, score.total, expected, score.grade);
        }
    }
}

#[cfg(test)]
mod property_tests {
    use proptest::prelude::*;
    use crate::tdg::{TdgScore, Language};
    use crate::tdg::analyzer_ast::TdgAnalyzerAst;

    proptest! {
        /// Property: Any valid Rust code should produce a normalized score
        #[test]
        fn prop_any_rust_code_produces_normalized_score(
            nesting_level in 0usize..5,
            num_functions in 1usize..10,
        ) {
            // Generate code with varying complexity
            let mut code = String::from("fn main() {\n");
            for _ in 0..nesting_level {
                code.push_str("    if true {\n");
            }
            for i in 0..num_functions {
                code.push_str(&format!("    let x{} = {};\n", i, i));
            }
            for _ in 0..nesting_level {
                code.push_str("    }\n");
            }
            code.push_str("}\n");

            let analyzer = TdgAnalyzerAst::new().unwrap();
            let score = analyzer.analyze_source(&code, Language::Rust, None).unwrap();

            // Verify normalization
            prop_assert!(score.total >= 0.0 && score.total <= 100.0);
            prop_assert!(score.entropy_score >= 0.0 && score.entropy_score <= 10.0);
            prop_assert!(score.structural_complexity >= 0.0 && score.structural_complexity <= 25.0);
        }
    }
}
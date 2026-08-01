#![cfg_attr(coverage_nightly, coverage(off))]

#[cfg(test)]
mod coverage_instrumented_tests {
    use super::super::helpers::{format_metric_name, grade_delta, grade_description, progress_bar};
    use super::super::{
        format_comparison, format_human, format_json, format_markdown, format_project,
    };
    use crate::tdg::language_simple::Language;
    use crate::tdg::{
        Comparison, Grade, MetricCategory, PenaltyAttribution, ProjectScore, TdgScore,
    };
    use std::collections::HashSet;
    use std::path::PathBuf;

    // ====================================================================
    // format_human: score ranges
    // ====================================================================

    #[test]
    fn test_ci_format_human_excellent_range() {
        let score = TdgScore {
            total: 96.0,
            grade: Grade::APlus,
            language: Language::Rust,
            confidence: 0.98,
            file_path: None,
            ..TdgScore::default()
        };
        let output = format_human(&score);
        assert!(output.contains("96.0/100"));
        assert!(output.contains("Excellent code quality"));
        assert!(output.contains("Code Analysis")); // no file_path
    }

    #[test]
    fn test_ci_format_human_good_range() {
        let score = TdgScore {
            total: 78.0,
            grade: Grade::B,
            language: Language::Python,
            confidence: 0.85,
            file_path: Some(PathBuf::from("lib/utils.py")),
            ..TdgScore::default()
        };
        let output = format_human(&score);
        assert!(output.contains("78.0/100"));
        assert!(output.contains("Good code quality"));
        assert!(output.contains("lib/utils.py"));
    }

    #[test]
    fn test_ci_format_human_improvement_range() {
        let score = TdgScore {
            total: 62.0,
            grade: Grade::C,
            ..TdgScore::default()
        };
        let output = format_human(&score);
        assert!(output.contains("needs improvement"));
    }

    #[test]
    fn test_ci_format_human_refactoring_range() {
        let score = TdgScore {
            total: 35.0,
            grade: Grade::F,
            ..TdgScore::default()
        };
        let output = format_human(&score);
        assert!(output.contains("requires significant refactoring"));
    }

    #[test]
    fn test_ci_format_human_with_penalties_truncation() {
        let score = TdgScore {
            total: 55.0,
            grade: Grade::CMinus,
            penalties_applied: vec![PenaltyAttribution {
                source_metric: MetricCategory::Coupling,
                amount: 8.0,
                applied_to: HashSet::new(),
                issue: "Excessive coupling between modules detected in analysis".to_string(),
            }],
            ..TdgScore::default()
        };
        let output = format_human(&score);
        assert!(output.contains("Issues Found"));
        // The long issue string should be truncated with "..."
        assert!(output.contains("..."));
    }

    #[test]
    fn test_ci_format_human_with_short_penalty() {
        let score = TdgScore {
            total: 68.0,
            grade: Grade::CPlus,
            penalties_applied: vec![PenaltyAttribution {
                source_metric: MetricCategory::Duplication,
                amount: 4.0,
                applied_to: HashSet::new(),
                issue: "Clone detected".to_string(),
            }],
            ..TdgScore::default()
        };
        let output = format_human(&score);
        assert!(output.contains("Clone detected"));
    }

    #[test]
    fn test_ci_format_human_with_file_path_long() {
        let score = TdgScore {
            total: 88.0,
            grade: Grade::AMinus,
            file_path: Some(PathBuf::from(
                "src/very/deeply/nested/path/to/some/module.rs",
            )),
            ..TdgScore::default()
        };
        let output = format_human(&score);
        // Path should be truncated to 30 chars
        assert!(output.contains("TDG Score Report:"));
    }

    // ====================================================================
    // format_json
    // ====================================================================

    #[test]
    fn test_ci_format_json_valid_json() {
        let score = TdgScore {
            total: 77.0,
            grade: Grade::B,
            language: Language::TypeScript,
            ..TdgScore::default()
        };
        let output = format_json(&score);
        // Should be valid JSON
        let parsed: serde_json::Value = serde_json::from_str(&output).unwrap();
        assert_eq!(parsed["total"], 77.0);
    }

    // ====================================================================
    // format_markdown
    // ====================================================================

    #[test]
    fn test_ci_format_markdown_table_structure() {
        let score = TdgScore {
            total: 82.0,
            grade: Grade::BPlus,
            file_path: Some(PathBuf::from("src/main.rs")),
            ..TdgScore::default()
        };
        let output = format_markdown(&score);
        assert!(output.contains("# TDG Score Report"));
        assert!(output.contains("## Score Breakdown"));
        assert!(output.contains("| Metric | Score | Max | Percentage |"));
        assert!(output.contains("**File:** `src/main.rs`"));
        assert!(output.contains("## Grade Description"));
        assert!(output.contains("**B+** (80-84)"));
    }

    #[test]
    fn test_ci_format_markdown_with_penalties() {
        let score = TdgScore {
            total: 50.0,
            grade: Grade::D,
            penalties_applied: vec![PenaltyAttribution {
                source_metric: MetricCategory::SemanticComplexity,
                amount: 15.0,
                applied_to: HashSet::new(),
                issue: "Deeply nested logic".to_string(),
            }],
            ..TdgScore::default()
        };
        let output = format_markdown(&score);
        assert!(output.contains("## Issues Found"));
        assert!(output.contains("Semantic Complexity"));
        assert!(output.contains("Deeply nested logic"));
        assert!(output.contains("-15.0 points"));
    }

    #[test]
    fn test_ci_format_markdown_no_path() {
        let score = TdgScore {
            total: 90.0,
            grade: Grade::A,
            file_path: None,
            ..TdgScore::default()
        };
        let output = format_markdown(&score);
        assert!(!output.contains("**File:**"));
    }

    // ====================================================================
    // format_comparison
    // ====================================================================

    #[test]
    fn test_ci_format_comparison_with_file_paths() {
        let s1 = TdgScore {
            total: 60.0,
            grade: Grade::C,
            structural_complexity: 15.0,
            semantic_complexity: 10.0,
            duplication_ratio: 12.0,
            coupling_score: 8.0,
            doc_coverage: 5.0,
            consistency_score: 5.0,
            file_path: Some(PathBuf::from("old_version.rs")),
            ..TdgScore::default()
        };
        let s2 = TdgScore {
            total: 85.0,
            grade: Grade::AMinus,
            structural_complexity: 22.0,
            semantic_complexity: 18.0,
            duplication_ratio: 18.0,
            coupling_score: 13.0,
            doc_coverage: 9.0,
            consistency_score: 9.0,
            file_path: Some(PathBuf::from("new_version.rs")),
            ..TdgScore::default()
        };
        let comparison = Comparison::new(s1, s2);
        let output = format_comparison(&comparison);
        assert!(output.contains("old_version.rs"));
        assert!(output.contains("new_version.rs"));
        assert!(output.contains("Winner:"));
        assert!(output.contains("Key Improvements"));
    }

    #[test]
    fn test_ci_format_comparison_equal_scores() {
        let s1 = TdgScore {
            total: 80.0,
            grade: Grade::BPlus,
            ..TdgScore::default()
        };
        let s2 = TdgScore {
            total: 80.0,
            grade: Grade::BPlus,
            ..TdgScore::default()
        };
        let comparison = Comparison::new(s1, s2);
        let output = format_comparison(&comparison);
        assert!(output.contains("80.0"));
        // Grade delta should be "="
        assert!(output.contains("="));
    }

    #[test]
    fn test_ci_format_comparison_header_truncation() {
        // Very long filenames should trigger header truncation
        let s1 = TdgScore {
            total: 70.0,
            grade: Grade::BMinus,
            file_path: Some(PathBuf::from("very_long_filename_that_exceeds_limit.rs")),
            ..TdgScore::default()
        };
        let s2 = TdgScore {
            total: 75.0,
            grade: Grade::B,
            file_path: Some(PathBuf::from("another_very_long_filename_exceeding.rs")),
            ..TdgScore::default()
        };
        let comparison = Comparison::new(s1, s2);
        let output = format_comparison(&comparison);
        // Should contain truncation marker
        assert!(output.contains("..."));
    }

    #[test]
    fn test_ci_format_comparison_with_regressions_list() {
        let s1 = TdgScore {
            total: 90.0,
            grade: Grade::A,
            structural_complexity: 23.0,
            doc_coverage: 10.0,
            ..TdgScore::default()
        };
        let s2 = TdgScore {
            total: 80.0,
            grade: Grade::BPlus,
            structural_complexity: 18.0,
            doc_coverage: 6.0,
            ..TdgScore::default()
        };
        let comparison = Comparison::new(s1, s2);
        let output = format_comparison(&comparison);
        assert!(output.contains("Minor Regressions"));
    }

    // ====================================================================
    // format_project
    // ====================================================================

    #[test]
    fn test_ci_format_project_basic() {
        let mut lang_dist = std::collections::HashMap::new();
        lang_dist.insert(Language::Rust, 8);
        lang_dist.insert(Language::Python, 2);

        let project = ProjectScore {
            average_score: 88.0,
            average_grade: Grade::AMinus,
            total_files: 10,
            files: vec![
                TdgScore {
                    total: 95.0,
                    grade: Grade::APlus,
                    ..TdgScore::default()
                },
                TdgScore {
                    total: 75.0,
                    grade: Grade::B,
                    ..TdgScore::default()
                },
                TdgScore {
                    total: 88.0,
                    grade: Grade::AMinus,
                    ..TdgScore::default()
                },
            ],
            language_distribution: lang_dist,
            grade_distribution: std::collections::HashMap::new(),
            f_grade_count: 0,
            grade_capped: false,
        };
        let output = format_project(&project);
        assert!(output.contains("Project TDG Score Report"));
        assert!(output.contains("88.0/100"));
        assert!(output.contains("Language Distribution"));
        assert!(output.contains("Grade Distribution"));
        // Verify specific languages appear
        assert!(output.contains("Rust"));
    }

    #[test]
    fn test_ci_format_project_single_file() {
        let mut lang_dist = std::collections::HashMap::new();
        lang_dist.insert(Language::Go, 1);

        let project = ProjectScore {
            average_score: 70.0,
            average_grade: Grade::BMinus,
            total_files: 1,
            files: vec![TdgScore {
                total: 70.0,
                grade: Grade::BMinus,
                ..TdgScore::default()
            }],
            language_distribution: lang_dist,
            grade_distribution: std::collections::HashMap::new(),
            f_grade_count: 0,
            grade_capped: false,
        };
        let output = format_project(&project);
        assert!(output.contains("70.0/100"));
        assert!(output.contains("1"));
    }

    // ====================================================================
    // Helper functions
    // ====================================================================

    #[test]
    fn test_ci_progress_bar_partial() {
        let bar = progress_bar(7.5, 10.0, 10);
        // 75% filled = 7 blocks, 3 empty
        assert_eq!(
            bar,
            "\u{2588}\u{2588}\u{2588}\u{2588}\u{2588}\u{2588}\u{2588}\u{2591}\u{2591}\u{2591}"
        );
    }

    #[test]
    fn test_ci_grade_delta_one_step() {
        assert_eq!(grade_delta(Grade::A, Grade::AMinus), "↓1");
        assert_eq!(grade_delta(Grade::AMinus, Grade::A), "↑1");
    }

    #[test]
    fn test_ci_grade_description_all_grades() {
        // Exercise all 11 branches of grade_description
        let descriptions = [
            (Grade::APlus, "A+"),
            (Grade::A, "A"),
            (Grade::AMinus, "A-"),
            (Grade::BPlus, "B+"),
            (Grade::B, "B"),
            (Grade::BMinus, "B-"),
            (Grade::CPlus, "C+"),
            (Grade::C, "C"),
            (Grade::CMinus, "C-"),
            (Grade::D, "D"),
            (Grade::F, "F"),
        ];
        for (grade, expected_substr) in &descriptions {
            let desc = grade_description(*grade);
            assert!(
                desc.contains(expected_substr),
                "grade_description({:?}) = '{}' should contain '{}'",
                grade,
                desc,
                expected_substr
            );
        }
    }

    #[test]
    fn test_ci_format_metric_name_all() {
        assert_eq!(
            format_metric_name(&MetricCategory::StructuralComplexity),
            "Structural Complexity"
        );
        assert_eq!(
            format_metric_name(&MetricCategory::SemanticComplexity),
            "Semantic Complexity"
        );
        assert_eq!(
            format_metric_name(&MetricCategory::Duplication),
            "Code Duplication"
        );
        assert_eq!(format_metric_name(&MetricCategory::Coupling), "Coupling");
        assert_eq!(
            format_metric_name(&MetricCategory::Documentation),
            "Documentation"
        );
        assert_eq!(
            format_metric_name(&MetricCategory::Consistency),
            "Consistency"
        );
    }
}

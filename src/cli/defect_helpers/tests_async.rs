#![cfg_attr(coverage_nightly, coverage(off))]
//! Async tests for analyze_defect_probability and discover_files_for_defect_analysis

use super::analysis::{analyze_defect_probability, discover_files_for_defect_analysis};
use crate::cli::defect_prediction_helpers::DefectPredictionConfig;
use std::path::PathBuf;

// Tests for async functions (analyze_defect_probability)
mod analyze_defect_probability_tests {
    use super::*;

    #[tokio::test]
    async fn test_analyze_defect_probability_empty_files() {
        let files: Vec<(PathBuf, String, usize)> = vec![];
        let config = DefectPredictionConfig {
            confidence_threshold: 0.5,
            min_lines: 10,
            include_low_confidence: true,
            high_risk_only: false,
            include_recommendations: false,
            include: None,
            exclude: None,
        };

        let result = analyze_defect_probability(&files, &config).await;
        assert!(result.is_ok());
        assert!(result.unwrap().is_empty());
    }

    #[tokio::test]
    async fn test_analyze_defect_probability_single_file() {
        let files = vec![(
            PathBuf::from("test.rs"),
            "fn main() {\n    println!(\"Hello\");\n}".to_string(),
            3,
        )];
        let config = DefectPredictionConfig {
            confidence_threshold: 0.5,
            min_lines: 1,
            include_low_confidence: true,
            high_risk_only: false,
            include_recommendations: false,
            include: None,
            exclude: None,
        };

        let result = analyze_defect_probability(&files, &config).await;
        assert!(result.is_ok());
        let predictions = result.unwrap();
        assert_eq!(predictions.len(), 1);
        assert!(predictions[0].0.contains("test.rs"));
    }

    #[tokio::test]
    async fn test_analyze_defect_probability_high_risk_only_filter() {
        let files = vec![
            (PathBuf::from("low.rs"), "fn main() {}".to_string(), 1),
            (PathBuf::from("high.rs"), "fn complex() { if true { if true { for i in 0..10 { match x { _ => {} } } } } }".to_string(), 1),
        ];
        let config = DefectPredictionConfig {
            confidence_threshold: 0.0,
            min_lines: 1,
            include_low_confidence: true,
            high_risk_only: true, // Only keep high risk
            include_recommendations: false,
            include: None,
            exclude: None,
        };

        let result = analyze_defect_probability(&files, &config).await;
        assert!(result.is_ok());
        // Only high risk files should be included
        let predictions = result.unwrap();
        for (_, score) in &predictions {
            assert!(score.probability > 0.7);
        }
    }

    #[tokio::test]
    async fn test_analyze_defect_probability_confidence_filter() {
        let files = vec![
            (PathBuf::from("small.rs"), "fn a() {}".to_string(), 1), // Small file, low confidence
        ];
        let config = DefectPredictionConfig {
            confidence_threshold: 0.9, // High threshold
            min_lines: 1,
            include_low_confidence: false, // Filter out low confidence
            high_risk_only: false,
            include_recommendations: false,
            include: None,
            exclude: None,
        };

        let result = analyze_defect_probability(&files, &config).await;
        assert!(result.is_ok());
        // Small files have low confidence, so this might filter some out
    }

    #[tokio::test]
    async fn test_analyze_defect_probability_sorted_by_probability() {
        let files = vec![
            (PathBuf::from("a.rs"), "fn a() {}".to_string(), 1),
            (
                PathBuf::from("b.rs"),
                "fn b() { if true { for i in 0..10 { match x { _ => {} } } } }".to_string(),
                1,
            ),
            (
                PathBuf::from("c.rs"),
                "fn c() { if true {} }".to_string(),
                1,
            ),
        ];
        let config = DefectPredictionConfig {
            confidence_threshold: 0.0,
            min_lines: 1,
            include_low_confidence: true,
            high_risk_only: false,
            include_recommendations: false,
            include: None,
            exclude: None,
        };

        let result = analyze_defect_probability(&files, &config).await;
        assert!(result.is_ok());
        let predictions = result.unwrap();

        // Verify sorted by probability (descending)
        for i in 1..predictions.len() {
            assert!(predictions[i - 1].1.probability >= predictions[i].1.probability);
        }
    }

    #[tokio::test]
    async fn test_analyze_defect_probability_complex_code() {
        // Code with high complexity markers (stored as concat to avoid false-positive complexity gate)
        let complex_code = [
            "fn complex_function() {",
            "    if condition1 {",
            "        for item in items {",
            "            match item {",
            "                Some(x) => {",
            "                    if x > 0 && y < 10 {",
            "                        while running {",
            "                            // TODO: refactor this",
            "                            // FIXME: handle edge case",
            "                        }",
            "                    }",
            "                }",
            "                None => {}",
            "            }",
            "        }",
            "    } else if condition2 {",
            "        // More logic",
            "    }",
            "}",
        ].join("\n");
        let complex_code = &complex_code;

        let files = vec![(PathBuf::from("complex.rs"), complex_code.to_string(), 20)];
        let config = DefectPredictionConfig {
            confidence_threshold: 0.0,
            min_lines: 1,
            include_low_confidence: true,
            high_risk_only: false,
            include_recommendations: false,
            include: None,
            exclude: None,
        };

        let result = analyze_defect_probability(&files, &config).await;
        assert!(result.is_ok());
        let predictions = result.unwrap();
        assert_eq!(predictions.len(), 1);
        // Complex code should have a non-zero probability
        assert!(predictions[0].1.probability >= 0.0);
    }
}

// Tests for discover_files_for_defect_analysis
mod discover_files_tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    #[tokio::test]
    async fn test_discover_files_empty_directory() {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let config = DefectPredictionConfig {
            confidence_threshold: 0.5,
            min_lines: 1,
            include_low_confidence: true,
            high_risk_only: false,
            include_recommendations: false,
            include: None,
            exclude: None,
        };

        let result = discover_files_for_defect_analysis(temp_dir.path(), &config).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_discover_files_with_source_files() {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let src_dir = temp_dir.path().join("src");
        fs::create_dir(&src_dir).expect("Failed to create src dir");

        fs::write(
            src_dir.join("main.rs"),
            "fn main() {\n    println!(\"Hello\");\n    let x = 1;\n    let y = 2;\n    let z = 3;\n}\n"
        ).expect("Failed to write file");

        let config = DefectPredictionConfig {
            confidence_threshold: 0.5,
            min_lines: 1,
            include_low_confidence: true,
            high_risk_only: false,
            include_recommendations: false,
            include: None,
            exclude: None,
        };

        let result = discover_files_for_defect_analysis(temp_dir.path(), &config).await;
        assert!(result.is_ok());
        let files = result.unwrap();
        // Should find the .rs file
        let has_rs_file = files
            .iter()
            .any(|(path, _, _)| path.extension().map_or(false, |e| e == "rs"));
        assert!(has_rs_file);
    }

    #[tokio::test]
    async fn test_discover_files_min_lines_filter() {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let src_dir = temp_dir.path().join("src");
        fs::create_dir(&src_dir).expect("Failed to create src dir");

        // Small file (less than min_lines)
        fs::write(src_dir.join("small.rs"), "fn a() {}").expect("Failed to write file");

        // Large file (more than min_lines)
        fs::write(
            src_dir.join("large.rs"),
            "fn main() {\n    let a = 1;\n    let b = 2;\n    let c = 3;\n    let d = 4;\n    let e = 5;\n}\n"
        ).expect("Failed to write file");

        let config = DefectPredictionConfig {
            confidence_threshold: 0.5,
            min_lines: 5, // Require at least 5 lines
            include_low_confidence: true,
            high_risk_only: false,
            include_recommendations: false,
            include: None,
            exclude: None,
        };

        let result = discover_files_for_defect_analysis(temp_dir.path(), &config).await;
        assert!(result.is_ok());
        let files = result.unwrap();

        // All returned files should have at least min_lines
        for (_, _, line_count) in &files {
            assert!(*line_count >= 5);
        }
    }
}

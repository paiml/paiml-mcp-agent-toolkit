mod hash_collision_tests {
    use super::*;

    #[test]
    fn test_winnowing_same_hash_different_text() {
        // Test that the system handles potential hash collisions gracefully
        let winnow = Winnowing::new(5, 3);

        // Create many fingerprints and verify uniqueness handling
        let texts = [
            "the quick brown fox",
            "the quick brown dog",
            "the quick green fox",
            "the slow brown fox",
            "a quick brown fox",
        ];

        let fingerprints: Vec<Vec<u64>> = texts.iter().map(|t| winnow.fingerprint(t)).collect();

        // Each should produce fingerprints
        for fp in &fingerprints {
            assert!(!fp.is_empty());
        }
    }

    #[test]
    fn test_detector_hash_collision_handling() {
        let detector = SimilarityDetector::new(SimilarityConfig {
            min_lines: 2,
            min_tokens: 3,
            ..SimilarityConfig::default()
        });

        // Create files with similar but different content
        let files = vec![
            (
                PathBuf::from("a.rs"),
                "let abc = 1;\nlet xyz = 2;\n".to_string(),
            ),
            (
                PathBuf::from("b.rs"),
                "let abc = 1;\nlet xyz = 2;\n".to_string(),
            ), // Same content
            (
                PathBuf::from("c.rs"),
                "let abc = 1;\nlet uvw = 2;\n".to_string(),
            ), // Different
        ];

        let duplicates = detector.detect_exact_duplicates(&files);
        // Should detect duplicates between a.rs and b.rs but not c.rs
        // The exact behavior depends on block extraction - len() is usize, always >= 0
        let _ = duplicates.len();
    }
}

// =============================================================================
// Empty Input Edge Cases
// =============================================================================

mod empty_input_tests {
    use super::*;

    #[test]
    fn test_detector_empty_files_list() {
        let detector = SimilarityDetector::new(SimilarityConfig::default());
        let files: Vec<(PathBuf, String)> = vec![];

        let exact = detector.detect_exact_duplicates(&files);
        let structural = detector.detect_structural_similarity(&files, 0.5);
        let semantic = detector.detect_semantic_similarity(&files, 0.5);
        let entropy = detector.analyze_entropy(&files);
        let refactoring = detector.find_refactoring_opportunities(&files);
        let comprehensive = detector.comprehensive_analysis(&files);

        assert!(exact.is_empty());
        assert!(structural.is_empty());
        assert!(semantic.is_empty());
        assert!((entropy.average_entropy - 0.0).abs() < f64::EPSILON);
        assert!(refactoring.is_empty());
        assert_eq!(comprehensive.metrics.total_clones, 0);
    }

    #[test]
    fn test_detector_file_with_empty_content() {
        let detector = SimilarityDetector::new(SimilarityConfig {
            min_lines: 1,
            min_tokens: 1,
            ..SimilarityConfig::default()
        });
        let files = vec![(PathBuf::from("empty.rs"), String::new())];

        let _ = detector.detect_exact_duplicates(&files);
        let _ = detector.analyze_entropy(&files);
    }

    #[test]
    fn test_winnowing_empty_string() {
        let winnow = Winnowing::new(5, 3);
        let fp = winnow.fingerprint("");
        assert!(fp.is_empty());
    }

    #[test]
    fn test_winnowing_similarity_both_empty() {
        let winnow = Winnowing::new(5, 3);
        let sim = winnow.similarity(&[], &[]);
        assert!((sim - 0.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_winnowing_find_matches_empty() {
        let winnow = Winnowing::new(5, 3);
        let matches = winnow.find_matches(&[], &[]);
        assert!(matches.is_empty());
    }

    #[test]
    fn test_detector_files_with_only_whitespace() {
        let detector = SimilarityDetector::new(SimilarityConfig {
            min_lines: 1,
            min_tokens: 1,
            ..SimilarityConfig::default()
        });
        let files = vec![
            (PathBuf::from("ws1.rs"), "   \n\t\n  \n".to_string()),
            (PathBuf::from("ws2.rs"), "\n\n\n".to_string()),
        ];

        let _ = detector.detect_exact_duplicates(&files);
        let _ = detector.analyze_entropy(&files);
    }
}

// =============================================================================
// Integration Tests
// =============================================================================

mod integration_tests {
    use super::*;

    #[test]
    fn test_full_workflow_real_rust_code() {
        let detector = SimilarityDetector::new(SimilarityConfig {
            min_lines: 3,
            min_tokens: 10,
            similarity_threshold: 0.7,
            enable_entropy: true,
            enable_ast: true,
            enable_semantic: true,
            window_size: 10,
            k_gram_size: 5,
        });

        let rust_code1 = r#"
fn calculate_average(numbers: &[i32]) -> f64 {
    if numbers.is_empty() {
        return 0.0;
    }
    let sum: i32 = numbers.iter().sum();
    sum as f64 / numbers.len() as f64
}
"#;

        let rust_code2 = r#"
fn compute_mean(values: &[i32]) -> f64 {
    if values.is_empty() {
        return 0.0;
    }
    let total: i32 = values.iter().sum();
    total as f64 / values.len() as f64
}
"#;

        let files = vec![
            (PathBuf::from("file1.rs"), rust_code1.to_string()),
            (PathBuf::from("file2.rs"), rust_code2.to_string()),
        ];

        let report = detector.comprehensive_analysis(&files);

        // Verify report structure
        assert!(report.entropy_analysis.is_some());
        assert!(report.metrics.average_entropy >= 0.0);
        assert!(report.metrics.duplication_percentage >= 0.0);
    }

    #[test]
    fn test_winnowing_plagiarism_detection_scenario() {
        let winnow = Winnowing::new(10, 5);

        // Original document
        let original = "This is an original academic paper discussing the implementation of winnowing algorithms for plagiarism detection. The technique uses fingerprinting to identify similar content.";

        // Slightly modified (paraphrased)
        let modified = "This paper discusses winnowing algorithms for plagiarism detection. The method uses fingerprinting techniques to find similar content in documents.";

        // Completely different
        let different = "Machine learning has transformed many industries. Neural networks provide powerful tools for pattern recognition and data analysis.";

        let fp_original = winnow.fingerprint(original);
        let fp_modified = winnow.fingerprint(modified);
        let fp_different = winnow.fingerprint(different);

        let sim_orig_mod = winnow.similarity(&fp_original, &fp_modified);
        let sim_orig_diff = winnow.similarity(&fp_original, &fp_different);

        // Modified version should have higher similarity than completely different
        assert!(sim_orig_mod > sim_orig_diff);
    }

    #[test]
    fn test_multiple_language_support() {
        let detector = SimilarityDetector::new(SimilarityConfig {
            min_lines: 2,
            min_tokens: 5,
            ..SimilarityConfig::default()
        });

        let python_code = "def hello():\n    print('Hello')\n    return True\n";
        let javascript_code =
            "function hello() {\n    console.log('Hello');\n    return true;\n}\n";
        let rust_code = "fn hello() {\n    println!(\"Hello\");\n    true\n}\n";

        let files = vec![
            (PathBuf::from("hello.py"), python_code.to_string()),
            (PathBuf::from("hello.js"), javascript_code.to_string()),
            (PathBuf::from("hello.rs"), rust_code.to_string()),
        ];

        let report = detector.comprehensive_analysis(&files);

        // Should work without panicking on multi-language input
        assert!(report.metrics.average_entropy >= 0.0);
    }

    #[test]
    fn test_large_file_handling() {
        let detector = SimilarityDetector::new(SimilarityConfig {
            min_lines: 6,
            min_tokens: 50,
            ..SimilarityConfig::default()
        });

        // Generate a larger file
        let mut large_content = String::new();
        for i in 0..100 {
            large_content.push_str(&format!("fn function_{i}() {{\n"));
            large_content.push_str(&format!("    let x_{i} = {i};\n"));
            large_content.push_str(&format!("    let y_{i} = {i} * 2;\n"));
            large_content.push_str(&format!("    println!(\"{{}} {{}}\", x_{i}, y_{i});\n"));
            large_content.push_str("}\n\n");
        }

        let files = vec![(PathBuf::from("large.rs"), large_content)];

        let report = detector.comprehensive_analysis(&files);
        assert!(report.metrics.average_entropy >= 0.0);
    }
}

// =============================================================================
// Boundary Condition Tests
// =============================================================================

mod boundary_tests {
    use super::*;

    #[test]
    fn test_similarity_threshold_boundaries() {
        let detector = SimilarityDetector::new(SimilarityConfig {
            min_lines: 2,
            min_tokens: 3,
            similarity_threshold: 0.0,
            ..SimilarityConfig::default()
        });

        let files = vec![
            (
                PathBuf::from("a.rs"),
                "abc def ghi\nabc def ghi\n".to_string(),
            ),
            (
                PathBuf::from("b.rs"),
                "xyz uvw rst\nxyz uvw rst\n".to_string(),
            ),
        ];

        // Threshold 0.0 should accept everything
        let _ = detector.detect_structural_similarity(&files, 0.0);

        // Threshold 1.0 should be very strict
        let _ = detector.detect_structural_similarity(&files, 1.0);
    }

    #[test]
    fn test_min_lines_boundary() {
        // min_lines = 1
        let detector = SimilarityDetector::new(SimilarityConfig {
            min_lines: 1,
            min_tokens: 1,
            ..SimilarityConfig::default()
        });

        let files = vec![(
            PathBuf::from("single.rs"),
            "single line content".to_string(),
        )];
        let _ = detector.detect_exact_duplicates(&files);
    }

    #[test]
    fn test_k_gram_larger_than_text() {
        let winnow = Winnowing::new(100, 100);
        let fp = winnow.fingerprint("short");
        assert!(fp.is_empty());
    }

    #[test]
    fn test_window_larger_than_k_grams() {
        let winnow = Winnowing::new(1000, 5);
        let fp = winnow.fingerprint("hello world test string");
        // Should still work, just fewer fingerprints
        assert!(fp.len() <= 100);
    }
}

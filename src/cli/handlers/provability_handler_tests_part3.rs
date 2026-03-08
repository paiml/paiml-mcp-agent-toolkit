// Provability handler tests - Part 3: Multiple files, detailed evidence, score distribution
// Included from provability_handler.rs mod coverage_tests

// ============================================================
// Multiple files tests
// ============================================================

#[test]
fn test_format_summary_with_multiple_files() {
    let config = ProvabilityConfig {
        project_path: PathBuf::from("/test"),
        functions: vec![],
        analysis_depth: 3,
        format: ProvabilityOutputFormat::Summary,
        high_confidence_only: false,
        include_evidence: false,
        output: None,
        top_files: 3,
    };

    // Multiple functions across multiple files
    let function_ids = vec![
        create_test_function_id("src/main.rs", "main", 1),
        create_test_function_id("src/main.rs", "run", 10),
        create_test_function_id("src/lib.rs", "helper", 5),
        create_test_function_id("src/lib.rs", "process", 20),
        create_test_function_id("src/utils.rs", "format", 1),
    ];
    let summaries = vec![
        create_test_summary(0.9),
        create_test_summary(0.85),
        create_test_summary(0.7),
        create_test_summary(0.6),
        create_test_summary(0.5),
    ];

    let result = format_provability_output(&function_ids, &summaries, &config);
    assert!(result.is_ok());

    let content = result.unwrap();
    assert!(content.contains("Top Files by Provability"));
    // Should show top 3 files
    assert!(content.contains("main.rs"));
}

#[test]
fn test_format_detailed_with_evidence() {
    let config = ProvabilityConfig {
        project_path: PathBuf::from("/test"),
        functions: vec![],
        analysis_depth: 3,
        format: ProvabilityOutputFormat::Full,
        high_confidence_only: false,
        include_evidence: true,
        output: None,
        top_files: 5,
    };

    let function_ids = vec![create_test_function_id("src/main.rs", "secure_fn", 42)];
    let summaries = vec![create_test_summary_with_properties(
        0.95,
        vec![
            VerifiedProperty {
                property_type: PropertyType::MemorySafety,
                confidence: 0.98,
                evidence: "All memory operations are safe".to_string(),
            },
            VerifiedProperty {
                property_type: PropertyType::ThreadSafety,
                confidence: 0.92,
                evidence: "No data races possible".to_string(),
            },
        ],
    )];

    let result = format_provability_output(&function_ids, &summaries, &config);
    assert!(result.is_ok());

    let content = result.unwrap();
    assert!(content.contains("Detailed Provability Analysis"));
    assert!(content.contains("Verified Properties"));
    assert!(content.contains("MemorySafety"));
    assert!(content.contains("ThreadSafety"));
}

#[test]
fn test_format_detailed_without_evidence() {
    let config = ProvabilityConfig {
        project_path: PathBuf::from("/test"),
        functions: vec![],
        analysis_depth: 3,
        format: ProvabilityOutputFormat::Full,
        high_confidence_only: false,
        include_evidence: false, // No evidence shown
        output: None,
        top_files: 5,
    };

    let function_ids = vec![create_test_function_id("src/main.rs", "fn", 1)];
    let summaries = vec![create_test_summary_with_properties(
        0.8,
        vec![VerifiedProperty {
            property_type: PropertyType::PureFunction,
            confidence: 0.9,
            evidence: "Pure function".to_string(),
        }],
    )];

    let result = format_provability_output(&function_ids, &summaries, &config);
    assert!(result.is_ok());

    // Detailed output without evidence section
    let content = result.unwrap();
    assert!(content.contains("Function:"));
}

// ============================================================
// Score distribution edge cases
// ============================================================

#[test]
fn test_score_distribution_all_high() {
    let config = ProvabilityConfig {
        project_path: PathBuf::from("/test"),
        functions: vec![],
        analysis_depth: 3,
        format: ProvabilityOutputFormat::Summary,
        high_confidence_only: false,
        include_evidence: false,
        output: None,
        top_files: 5,
    };

    let function_ids = vec![
        create_test_function_id("a.rs", "f1", 1),
        create_test_function_id("a.rs", "f2", 2),
    ];
    let summaries = vec![create_test_summary(0.95), create_test_summary(0.88)];

    let result = format_provability_output(&function_ids, &summaries, &config);
    assert!(result.is_ok());

    let content = strip_ansi(&result.unwrap());
    assert!(content.contains("High (\u{2265}80%): 2 functions"));
    assert!(content.contains("Medium (50-79%): 0 functions"));
    assert!(content.contains("Low (<50%): 0 functions"));
}

#[test]
fn test_score_distribution_all_low() {
    let config = ProvabilityConfig {
        project_path: PathBuf::from("/test"),
        functions: vec![],
        analysis_depth: 3,
        format: ProvabilityOutputFormat::Summary,
        high_confidence_only: false,
        include_evidence: false,
        output: None,
        top_files: 5,
    };

    let function_ids = vec![
        create_test_function_id("a.rs", "f1", 1),
        create_test_function_id("a.rs", "f2", 2),
    ];
    let summaries = vec![create_test_summary(0.2), create_test_summary(0.3)];

    let result = format_provability_output(&function_ids, &summaries, &config);
    assert!(result.is_ok());

    let content = strip_ansi(&result.unwrap());
    assert!(content.contains("High (\u{2265}80%): 0 functions"));
    assert!(content.contains("Low (<50%): 2 functions"));
}

#[test]
fn test_average_score_calculation() {
    let config = ProvabilityConfig {
        project_path: PathBuf::from("/test"),
        functions: vec![],
        analysis_depth: 3,
        format: ProvabilityOutputFormat::Summary,
        high_confidence_only: false,
        include_evidence: false,
        output: None,
        top_files: 5,
    };

    let function_ids = vec![
        create_test_function_id("a.rs", "f1", 1),
        create_test_function_id("a.rs", "f2", 2),
    ];
    // Average should be (0.8 + 0.6) / 2 = 0.7 = 70%
    let summaries = vec![create_test_summary(0.8), create_test_summary(0.6)];

    let result = format_provability_output(&function_ids, &summaries, &config);
    assert!(result.is_ok());

    let content = strip_ansi(&result.unwrap());
    assert!(content.contains("Average provability score: 70.0%"));
}

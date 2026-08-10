// Provability handler tests - Part 2: Integration tests, all output formats, edge cases
// Included from provability_handler.rs mod coverage_tests

// ============================================================
// Full handler integration tests
// ============================================================

/// Write a project containing exactly the named functions.
///
/// These tests used to pass an EMPTY `TempDir` together with `--functions
/// <name>` and assert `is_ok()`. That asserted the fabrication the handler was
/// fixed for: every spec was turned into a `FunctionId` with no lookup, so
/// `--functions ghost` against an empty directory printed "✓ Analyzed 1
/// functions" and scored it 20.0%. Naming a function that does not exist is now
/// an error, so each format test gets a project where its function really does
/// exist — which is what these tests were always meant to exercise.
fn project_with_functions(temp: &TempDir, names: &[&str]) {
    let src = temp.path().join("src");
    std::fs::create_dir_all(&src).unwrap();
    let body: String = names
        .iter()
        .map(|n| format!("pub fn {n}(x: i32) -> i32 {{ if x > 0 {{ x }} else {{ -x }} }}\n"))
        .collect();
    std::fs::write(src.join("lib.rs"), body).unwrap();
}

#[tokio::test]
async fn test_handle_analyze_provability_with_output_file() {
    let temp_dir = TempDir::new().unwrap();
    let output_path = temp_dir.path().join("analysis.json");

    // Create a minimal Rust file for analysis
    let src_dir = temp_dir.path().join("src");
    std::fs::create_dir_all(&src_dir).unwrap();
    std::fs::write(src_dir.join("main.rs"), "fn main() {}").unwrap();

    let config = ProvabilityConfig {
        project_path: temp_dir.path().to_path_buf(),
        functions: vec!["main".to_string()],
        analysis_depth: 3,
        format: ProvabilityOutputFormat::Json,
        high_confidence_only: false,
        include_evidence: true,
        output: Some(output_path.clone()),
        top_files: 5,
    };

    let result = handle_analyze_provability(config).await;
    assert!(result.is_ok());
    assert!(output_path.exists());
}

#[tokio::test]
async fn test_handle_analyze_provability_summary_format() {
    let temp_dir = TempDir::new().unwrap();
    project_with_functions(&temp_dir, &["test_func"]);

    let config = ProvabilityConfig {
        project_path: temp_dir.path().to_path_buf(),
        functions: vec!["test_func".to_string()],
        analysis_depth: 5,
        format: ProvabilityOutputFormat::Summary,
        high_confidence_only: false,
        include_evidence: false,
        output: None,
        top_files: 10,
    };

    let result = handle_analyze_provability(config).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_handle_analyze_provability_sarif_format() {
    let temp_dir = TempDir::new().unwrap();
    project_with_functions(&temp_dir, &["main"]);
    let output_path = temp_dir.path().join("analysis.sarif");

    let config = ProvabilityConfig {
        project_path: temp_dir.path().to_path_buf(),
        functions: vec!["main".to_string()],
        analysis_depth: 3,
        format: ProvabilityOutputFormat::Sarif,
        high_confidence_only: false,
        include_evidence: false,
        output: Some(output_path.clone()),
        top_files: 5,
    };

    let result = handle_analyze_provability(config).await;
    assert!(result.is_ok());

    let content = std::fs::read_to_string(&output_path).unwrap();
    assert!(content.contains("sarif-schema"));
}

#[tokio::test]
async fn test_handle_analyze_provability_full_format_with_evidence() {
    let temp_dir = TempDir::new().unwrap();
    project_with_functions(&temp_dir, &["complex_fn"]);

    let config = ProvabilityConfig {
        project_path: temp_dir.path().to_path_buf(),
        functions: vec!["complex_fn".to_string()],
        analysis_depth: 10,
        format: ProvabilityOutputFormat::Full,
        high_confidence_only: false,
        include_evidence: true,
        output: None,
        top_files: 15,
    };

    let result = handle_analyze_provability(config).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_handle_analyze_provability_markdown_format() {
    let temp_dir = TempDir::new().unwrap();
    project_with_functions(&temp_dir, &["test"]);
    let output_path = temp_dir.path().join("analysis.md");

    let config = ProvabilityConfig {
        project_path: temp_dir.path().to_path_buf(),
        functions: vec!["test".to_string()],
        analysis_depth: 3,
        format: ProvabilityOutputFormat::Markdown,
        high_confidence_only: false,
        include_evidence: true,
        output: Some(output_path.clone()),
        top_files: 5,
    };

    let result = handle_analyze_provability(config).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_handle_analyze_provability_high_confidence_filter() {
    let temp_dir = TempDir::new().unwrap();
    project_with_functions(&temp_dir, &["fn1", "fn2"]);

    let config = ProvabilityConfig {
        project_path: temp_dir.path().to_path_buf(),
        functions: vec!["fn1".to_string(), "fn2".to_string()],
        analysis_depth: 3,
        format: ProvabilityOutputFormat::Json,
        high_confidence_only: true, // Only high confidence
        include_evidence: false,
        output: None,
        top_files: 5,
    };

    let result = handle_analyze_provability(config).await;
    assert!(result.is_ok());
}

// ============================================================
// All output formats tested
// ============================================================

#[test]
fn test_all_output_formats_display() {
    // Ensure all enum variants can be used in format_provability_output
    let formats = vec![
        ProvabilityOutputFormat::Json,
        ProvabilityOutputFormat::Summary,
        ProvabilityOutputFormat::Full,
        ProvabilityOutputFormat::Sarif,
        ProvabilityOutputFormat::Markdown,
    ];

    for format in formats {
        let config = ProvabilityConfig {
            project_path: PathBuf::from("/test"),
            functions: vec![],
            analysis_depth: 3,
            format: format.clone(),
            high_confidence_only: false,
            include_evidence: false,
            output: None,
            top_files: 5,
        };

        let function_ids = vec![create_test_function_id("test.rs", "test_fn", 1)];
        let summaries = vec![create_test_summary(0.75)];

        let result = format_provability_output(&function_ids, &summaries, &config);
        assert!(
            result.is_ok(),
            "Format {:?} should produce valid output",
            format
        );
    }
}

// ============================================================
// Edge cases and error paths
// ============================================================

#[test]
fn test_format_with_empty_function_ids() {
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

    let function_ids: Vec<FunctionId> = vec![];
    let summaries: Vec<ProofSummary> = vec![];

    let result = format_provability_output(&function_ids, &summaries, &config);
    assert!(result.is_ok());

    let content = strip_ansi(&result.unwrap());
    assert!(content.contains("Total functions analyzed: 0"));
}

#[test]
fn test_format_with_zero_top_files() {
    let config = ProvabilityConfig {
        project_path: PathBuf::from("/test"),
        functions: vec![],
        analysis_depth: 3,
        format: ProvabilityOutputFormat::Summary,
        high_confidence_only: false,
        include_evidence: false,
        output: None,
        top_files: 0, // Zero means show default (10)
    };

    let function_ids = vec![create_test_function_id("test.rs", "fn", 1)];
    let summaries = vec![create_test_summary(0.8)];

    let result = format_provability_output(&function_ids, &summaries, &config);
    assert!(result.is_ok());
}

#[test]
fn test_format_json_with_evidence() {
    let config = ProvabilityConfig {
        project_path: PathBuf::from("/test"),
        functions: vec![],
        analysis_depth: 3,
        format: ProvabilityOutputFormat::Json,
        high_confidence_only: false,
        include_evidence: true, // Include evidence in JSON
        output: None,
        top_files: 5,
    };

    let function_ids = vec![create_test_function_id("test.rs", "tested_fn", 42)];
    let summaries = vec![create_test_summary_with_properties(
        0.92,
        vec![
            VerifiedProperty {
                property_type: PropertyType::NullSafety,
                confidence: 0.95,
                evidence: "Comprehensive null checks".to_string(),
            },
            VerifiedProperty {
                property_type: PropertyType::BoundsCheck,
                confidence: 0.88,
                evidence: "Array bounds verified".to_string(),
            },
        ],
    )];

    let result = format_provability_output(&function_ids, &summaries, &config);
    assert!(result.is_ok());

    let content = result.unwrap();
    assert!(content.contains("properties"));
    assert!(content.contains("NullSafety"));
}

#[test]
fn test_format_json_without_evidence() {
    let config = ProvabilityConfig {
        project_path: PathBuf::from("/test"),
        functions: vec![],
        analysis_depth: 3,
        format: ProvabilityOutputFormat::Json,
        high_confidence_only: false,
        include_evidence: false, // No evidence
        output: None,
        top_files: 5,
    };

    let function_ids = vec![create_test_function_id("test.rs", "tested_fn", 42)];
    let summaries = vec![create_test_summary_with_properties(
        0.92,
        vec![VerifiedProperty {
            property_type: PropertyType::PureFunction,
            confidence: 0.99,
            evidence: "No side effects".to_string(),
        }],
    )];

    let result = format_provability_output(&function_ids, &summaries, &config);
    assert!(result.is_ok());

    let content = result.unwrap();
    // Should contain verified_properties count but not detailed properties
    assert!(content.contains("verified_properties"));
}

#[test]
fn test_sarif_output_all_score_levels() {
    let config = ProvabilityConfig {
        project_path: PathBuf::from("/test"),
        functions: vec![],
        analysis_depth: 3,
        format: ProvabilityOutputFormat::Sarif,
        high_confidence_only: false,
        include_evidence: false,
        output: None,
        top_files: 5,
    };

    // Test all three score categories: low (<0.5), medium (0.5-0.8), high (>=0.8)
    let function_ids = vec![
        create_test_function_id("test.rs", "low_fn", 10),
        create_test_function_id("test.rs", "medium_fn", 20),
        create_test_function_id("test.rs", "high_fn", 30),
    ];
    let summaries = vec![
        create_test_summary(0.3),  // Low - warning level
        create_test_summary(0.65), // Medium - note level
        create_test_summary(0.9),  // High - none level
    ];

    let result = format_provability_output(&function_ids, &summaries, &config);
    assert!(result.is_ok());

    let content = result.unwrap();
    assert!(content.contains("low-provability"));
    assert!(content.contains("medium-provability"));
    assert!(content.contains("high-provability"));
    assert!(content.contains("\"level\": \"warning\""));
    assert!(content.contains("\"level\": \"note\""));
    assert!(content.contains("\"level\": \"none\""));
}

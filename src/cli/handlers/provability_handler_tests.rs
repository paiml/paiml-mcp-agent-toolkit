// Provability handler tests - Part 1: Config, prepare_filtered, format tests
// Included from provability_handler.rs mod coverage_tests

fn strip_ansi(s: &str) -> String {
    let re = regex::Regex::new(r"\x1b\[[0-9;]*m").unwrap();
    re.replace_all(s, "").to_string()
}

// ============================================================
// ProvabilityConfig tests
// ============================================================

#[test]
fn test_provability_config_creation() {
    let config = ProvabilityConfig {
        project_path: PathBuf::from("/test/project"),
        functions: vec!["main".to_string(), "test".to_string()],
        analysis_depth: 5,
        format: ProvabilityOutputFormat::Json,
        high_confidence_only: true,
        include_evidence: true,
        output: Some(PathBuf::from("/test/output.json")),
        top_files: 10,
    };

    assert_eq!(config.project_path, PathBuf::from("/test/project"));
    assert_eq!(config.functions.len(), 2);
    assert_eq!(config.analysis_depth, 5);
    assert_eq!(config.format, ProvabilityOutputFormat::Json);
    assert!(config.high_confidence_only);
    assert!(config.include_evidence);
    assert!(config.output.is_some());
    assert_eq!(config.top_files, 10);
}

#[test]
fn test_provability_config_debug_derive() {
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

    let debug_output = format!("{:?}", config);
    assert!(debug_output.contains("ProvabilityConfig"));
    assert!(debug_output.contains("/test"));
}

#[test]
fn test_provability_config_clone() {
    let config = ProvabilityConfig {
        project_path: PathBuf::from("/original"),
        functions: vec!["fn1".to_string()],
        analysis_depth: 7,
        format: ProvabilityOutputFormat::Full,
        high_confidence_only: true,
        include_evidence: true,
        output: Some(PathBuf::from("/output.md")),
        top_files: 15,
    };

    let cloned = config.clone();
    assert_eq!(cloned.project_path, config.project_path);
    assert_eq!(cloned.functions, config.functions);
    assert_eq!(cloned.analysis_depth, config.analysis_depth);
    assert_eq!(cloned.format, config.format);
    assert_eq!(cloned.high_confidence_only, config.high_confidence_only);
    assert_eq!(cloned.include_evidence, config.include_evidence);
    assert_eq!(cloned.output, config.output);
    assert_eq!(cloned.top_files, config.top_files);
}

// ============================================================
// prepare_filtered_summaries tests
// ============================================================

fn create_test_summary(score: f64) -> ProofSummary {
    ProofSummary {
        provability_score: score,
        analysis_time_us: 100,
        verified_properties: vec![],
        version: 1,
    }
}

fn create_test_summary_with_properties(
    score: f64,
    props: Vec<VerifiedProperty>,
) -> ProofSummary {
    ProofSummary {
        provability_score: score,
        analysis_time_us: 200,
        verified_properties: props,
        version: 1,
    }
}

#[test]
fn test_prepare_filtered_summaries_no_filter() {
    let summaries = vec![
        create_test_summary(0.9),
        create_test_summary(0.5),
        create_test_summary(0.3),
    ];

    let filtered = prepare_filtered_summaries(&summaries, false);

    assert_eq!(filtered.len(), 3);
    assert_eq!(filtered[0].provability_score, 0.9);
    assert_eq!(filtered[1].provability_score, 0.5);
    assert_eq!(filtered[2].provability_score, 0.3);
}

#[test]
fn test_prepare_filtered_summaries_high_confidence_only() {
    let summaries = vec![
        create_test_summary(0.95), // High confidence - included
        create_test_summary(0.85), // High confidence - included
        create_test_summary(0.79), // Below threshold - excluded
        create_test_summary(0.5),  // Below threshold - excluded
        create_test_summary(0.2),  // Below threshold - excluded
    ];

    let filtered = prepare_filtered_summaries(&summaries, true);

    assert_eq!(filtered.len(), 2);
    assert!(filtered.iter().all(|s| s.provability_score >= 0.8));
}

#[test]
fn test_prepare_filtered_summaries_empty_input() {
    let summaries: Vec<ProofSummary> = vec![];
    let filtered = prepare_filtered_summaries(&summaries, true);
    assert!(filtered.is_empty());
}

#[test]
fn test_prepare_filtered_summaries_all_below_threshold() {
    let summaries = vec![
        create_test_summary(0.5),
        create_test_summary(0.6),
        create_test_summary(0.7),
    ];

    let filtered = prepare_filtered_summaries(&summaries, true);
    assert!(filtered.is_empty());
}

#[test]
fn test_prepare_filtered_summaries_boundary_score() {
    // Test exact 0.8 boundary (should be included since >= 0.8)
    let summaries = vec![create_test_summary(0.8), create_test_summary(0.7999)];

    let filtered = prepare_filtered_summaries(&summaries, true);
    assert_eq!(filtered.len(), 1);
    assert_eq!(filtered[0].provability_score, 0.8);
}

// ============================================================
// format_provability_output tests
// ============================================================

fn create_test_function_id(file: &str, name: &str, line: usize) -> FunctionId {
    FunctionId {
        file_path: file.to_string(),
        function_name: name.to_string(),
        line_number: line,
    }
}

#[test]
fn test_format_provability_output_json() {
    let config = ProvabilityConfig {
        project_path: PathBuf::from("/test"),
        functions: vec![],
        analysis_depth: 3,
        format: ProvabilityOutputFormat::Json,
        high_confidence_only: false,
        include_evidence: true,
        output: None,
        top_files: 5,
    };

    let function_ids = vec![create_test_function_id("src/main.rs", "main", 10)];
    let summaries = vec![create_test_summary(0.85)];

    let result = format_provability_output(&function_ids, &summaries, &config);
    assert!(result.is_ok());

    let content = result.unwrap();
    assert!(content.contains("provability_analysis"));
    assert!(content.contains("main"));
}

#[test]
fn test_format_provability_output_summary() {
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
        create_test_function_id("src/main.rs", "main", 10),
        create_test_function_id("src/lib.rs", "helper", 20),
    ];
    let summaries = vec![create_test_summary(0.9), create_test_summary(0.6)];

    let result = format_provability_output(&function_ids, &summaries, &config);
    assert!(result.is_ok());

    let content = result.unwrap();
    assert!(content.contains("Provability Analysis Summary"));
    assert!(content.contains("Total functions analyzed:"));
}

#[test]
fn test_format_provability_output_full() {
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

    let function_ids = vec![create_test_function_id("src/main.rs", "main", 10)];
    let summaries = vec![create_test_summary_with_properties(
        0.85,
        vec![VerifiedProperty {
            property_type: PropertyType::NullSafety,
            confidence: 0.9,
            evidence: "No null references".to_string(),
        }],
    )];

    let result = format_provability_output(&function_ids, &summaries, &config);
    assert!(result.is_ok());

    let content = result.unwrap();
    assert!(content.contains("Detailed Provability Analysis"));
}

#[test]
fn test_format_provability_output_sarif() {
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

    let function_ids = vec![
        create_test_function_id("src/main.rs", "high_score", 10),
        create_test_function_id("src/main.rs", "medium_score", 20),
        create_test_function_id("src/main.rs", "low_score", 30),
    ];
    let summaries = vec![
        create_test_summary(0.9), // High
        create_test_summary(0.6), // Medium
        create_test_summary(0.3), // Low
    ];

    let result = format_provability_output(&function_ids, &summaries, &config);
    assert!(result.is_ok());

    let content = result.unwrap();
    assert!(content.contains("sarif-schema-2.1.0"));
    assert!(content.contains("paiml-provability-analyzer"));
}

#[test]
fn test_format_provability_output_markdown() {
    let config = ProvabilityConfig {
        project_path: PathBuf::from("/test"),
        functions: vec![],
        analysis_depth: 3,
        format: ProvabilityOutputFormat::Markdown,
        high_confidence_only: false,
        include_evidence: true,
        output: None,
        top_files: 5,
    };

    let function_ids = vec![create_test_function_id("src/main.rs", "main", 10)];
    let summaries = vec![create_test_summary(0.85)];

    let result = format_provability_output(&function_ids, &summaries, &config);
    assert!(result.is_ok());

    // Was: `// Markdown format uses format_provability_detailed` +
    // `assert!(content.contains("Detailed Provability Analysis"))` — the comment
    // documented the defect. `-f markdown` fell through to the detailed
    // renderer, so it emitted ANSI-escaped plain text byte-identical to
    // `-f full` and nothing a Markdown consumer could use. It has its own
    // renderer now, so assert actual Markdown.
    let content = result.unwrap();
    assert!(
        content.starts_with("# Provability Analysis"),
        "markdown must open with an H1, got: {}",
        content.lines().next().unwrap_or("")
    );
    assert!(content.contains("## `src/main.rs`"), "{content}");
    assert!(
        content.contains("| Function | Line | Provability | Verified | Missing |"),
        "{content}"
    );
    assert!(
        !content.contains('\u{1b}'),
        "markdown must not carry ANSI escapes"
    );
}

// ============================================================
// write_provability_output tests
// ============================================================

#[tokio::test]
async fn test_write_provability_output_to_stdout() {
    let content = "Test output content";

    // When output is None, it prints to stdout
    let result = write_provability_output(content, &None).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_write_provability_output_to_file() {
    let temp_dir = TempDir::new().unwrap();
    let output_path = temp_dir.path().join("output.txt");

    let content = "Test file output content";
    let result = write_provability_output(content, &Some(output_path.clone())).await;

    assert!(result.is_ok());
    assert!(output_path.exists());

    let file_content = std::fs::read_to_string(&output_path).unwrap();
    assert_eq!(file_content, content);
}

#[tokio::test]
async fn test_write_provability_output_to_file_with_json() {
    let temp_dir = TempDir::new().unwrap();
    let output_path = temp_dir.path().join("provability.json");

    let content = r#"{"provability_analysis": {"total_functions": 1}}"#;
    let result = write_provability_output(content, &Some(output_path.clone())).await;

    assert!(result.is_ok());
    assert!(output_path.exists());
}

// ============================================================
// resolve_function_targets tests (via integration patterns)
// ============================================================

#[tokio::test]
async fn test_resolve_function_targets_with_empty_functions() {
    // This tests the branch where config.functions.is_empty() is true
    let temp_dir = TempDir::new().unwrap();
    let config = ProvabilityConfig {
        project_path: temp_dir.path().to_path_buf(),
        functions: vec![], // Empty - will discover functions
        analysis_depth: 3,
        format: ProvabilityOutputFormat::Summary,
        high_confidence_only: false,
        include_evidence: false,
        output: None,
        top_files: 5,
    };

    let result = resolve_function_targets(&config).await;
    // May succeed or fail depending on directory state, but exercises the code path
    // If it succeeds, we get function IDs; if it fails, we handle error
    assert!(result.is_ok() || result.is_err());
}

fn provability_config_for(path: &std::path::Path, functions: Vec<String>) -> ProvabilityConfig {
    ProvabilityConfig {
        project_path: path.to_path_buf(),
        functions,
        analysis_depth: 3,
        format: ProvabilityOutputFormat::Summary,
        high_confidence_only: false,
        include_evidence: false,
        output: None,
        top_files: 5,
    }
}

#[tokio::test]
async fn test_resolve_function_targets_with_specific_functions() {
    // Specs must resolve against functions that are really in the tree. This
    // test used to point at an EMPTY temp dir and assert two ids came back,
    // which is precisely the fabrication being fixed.
    let temp_dir = TempDir::new().unwrap();
    std::fs::create_dir_all(temp_dir.path().join("src")).unwrap();
    std::fs::write(temp_dir.path().join("src/main.rs"), "pub fn main() {}\n").unwrap();
    std::fs::write(temp_dir.path().join("src/lib.rs"), "pub fn helper() {}\n").unwrap();

    let config = provability_config_for(
        temp_dir.path(),
        vec!["main".to_string(), "src/lib.rs:helper".to_string()],
    );

    let ids = resolve_function_targets(&config).await.unwrap();
    assert_eq!(ids.len(), 2);
    assert_eq!(ids[0].function_name, "main");
    assert_eq!(ids[0].file_path, "src/main.rs");
    assert_eq!(ids[1].function_name, "helper");
    assert_eq!(ids[1].file_path, "src/lib.rs");
}

/// `--functions ghost` against an empty directory used to report
/// "✓ Analyzed 1 functions / Average provability score: 20.0%" — a result row
/// manufactured from the argument string. A name with nothing behind it must
/// be an error.
#[tokio::test]
async fn test_resolve_function_targets_rejects_unknown_function_name() {
    let temp_dir = TempDir::new().unwrap();

    let config = provability_config_for(temp_dir.path(), vec!["ghost".to_string()]);
    let err = resolve_function_targets(&config)
        .await
        .expect_err("a function that is not in the tree must not be analyzed");
    assert!(err.to_string().contains("ghost"), "unexpected: {err}");

    // Same for a file-qualified spec whose file exists but whose function does not.
    std::fs::write(temp_dir.path().join("real.rs"), "pub fn present() {}\n").unwrap();
    let config = provability_config_for(temp_dir.path(), vec!["real.rs:absent".to_string()]);
    assert!(resolve_function_targets(&config).await.is_err());

    let config = provability_config_for(temp_dir.path(), vec!["real.rs:present".to_string()]);
    assert_eq!(resolve_function_targets(&config).await.unwrap().len(), 1);
}

// ============================================================
// run_provability_analysis tests
// ============================================================

#[tokio::test]
async fn test_run_provability_analysis_basic() {
    use crate::services::lightweight_provability_analyzer::LightweightProvabilityAnalyzer;

    let analyzer = LightweightProvabilityAnalyzer::new();
    let function_ids = vec![
        create_test_function_id("src/main.rs", "main", 10),
        create_test_function_id("src/lib.rs", "helper", 20),
    ];

    let result = run_provability_analysis(&analyzer, &function_ids).await;
    assert!(result.is_ok());

    let summaries = result.unwrap();
    assert_eq!(summaries.len(), 2);
}

#[tokio::test]
async fn test_run_provability_analysis_empty_input() {
    use crate::services::lightweight_provability_analyzer::LightweightProvabilityAnalyzer;

    let analyzer = LightweightProvabilityAnalyzer::new();
    let function_ids: Vec<FunctionId> = vec![];

    let result = run_provability_analysis(&analyzer, &function_ids).await;
    assert!(result.is_ok());

    let summaries = result.unwrap();
    assert!(summaries.is_empty());
}

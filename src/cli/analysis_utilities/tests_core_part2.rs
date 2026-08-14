/// Test check_duplicates with unique files
#[tokio::test]
async fn test_check_duplicates_unique_files() -> anyhow::Result<()> {
    let temp_dir = TempDir::new()?;

    let file1 = temp_dir.path().join("unique1.rs");
    let file2 = temp_dir.path().join("unique2.rs");

    tokio::fs::write(&file1, "fn unique_function_one() { println!(\"one\"); }").await?;
    tokio::fs::write(&file2, "fn unique_function_two() { println!(\"two\"); }").await?;

    let violations = check_duplicates(temp_dir.path()).await?;

    // Should detect no duplicates
    assert_eq!(violations.len(), 0);

    Ok(())
}

/// Test check_duplicates ignores small files
#[tokio::test]
async fn test_check_duplicates_ignores_small_files() -> anyhow::Result<()> {
    let temp_dir = TempDir::new()?;

    // Create small identical files (should be ignored)
    let small_content = "x";

    let small1 = temp_dir.path().join("small1.rs");
    let small2 = temp_dir.path().join("small2.rs");

    tokio::fs::write(&small1, small_content).await?;
    tokio::fs::write(&small2, small_content).await?;

    let violations = check_duplicates(temp_dir.path()).await?;

    // Should ignore small files
    assert_eq!(violations.len(), 0);

    Ok(())
}

/// Test is_build_artifact function excludes build directories
#[test]
fn test_is_build_artifact() {
    use std::path::Path;

    // Should exclude target directory files
    assert!(is_build_artifact(Path::new(
        "./target/debug/build/pmat-123/out/tool_registry.rs"
    )));
    assert!(is_build_artifact(Path::new("target/debug/deps/pmat.rs")));

    // Should exclude other build artifacts
    assert!(is_build_artifact(Path::new("./build/generated.rs")));
    assert!(is_build_artifact(Path::new("./out/alias_table.rs")));
    assert!(is_build_artifact(Path::new(
        "./.cargo/registry/src/github.com/file.rs"
    )));
    assert!(is_build_artifact(Path::new(
        "./node_modules/package/lib.js"
    )));
    assert!(is_build_artifact(Path::new("./dist/bundle.js")));
    assert!(is_build_artifact(Path::new("./.git/objects/ab/cd1234")));
    assert!(is_build_artifact(Path::new("./generated/proto.rs")));

    // Should NOT exclude source files
    assert!(!is_build_artifact(Path::new("./server/src/lib.rs")));
    assert!(!is_build_artifact(Path::new("./src/main.rs")));
    assert!(!is_build_artifact(Path::new(
        "./server/src/handlers/tools.rs"
    )));
}

/// Test is_excluded_directory function excludes build directories
#[test]
fn test_is_excluded_directory() {
    // Should exclude build directories
    assert!(is_excluded_directory("./target"));
    assert!(is_excluded_directory("target"));
    assert!(is_excluded_directory("target/"));
    assert!(is_excluded_directory("./target/"));
    assert!(is_excluded_directory("./build"));
    assert!(is_excluded_directory("build"));
    assert!(is_excluded_directory("./project/target/"));
    assert!(is_excluded_directory("./project/target/debug"));
    assert!(is_excluded_directory("./target/debug/build"));
    assert!(is_excluded_directory("./foo/node_modules/"));
    assert!(is_excluded_directory("./bar/.git/"));
    assert!(is_excluded_directory(
        "./server/target/debug/build/unicode_names2-c78072d37d9beb66/out/generated.rs"
    ));
    assert!(is_excluded_directory(
        "./target/debug/build/rustpython-parser-5d3dfbfd27d1a200/out/keywords.rs"
    ));

    // Should NOT exclude source directories
    assert!(!is_excluded_directory("server"));
    assert!(!is_excluded_directory("src"));
    assert!(!is_excluded_directory("./server/src"));
    assert!(!is_excluded_directory("./server/src/cli"));
}

/// Test check_single_file_complexity with high complexity function
#[tokio::test]
async fn test_check_single_file_complexity_violations() -> anyhow::Result<()> {
    let temp_dir = TempDir::new()?;

    // Create Rust file with high complexity function
    let rust_file = temp_dir.path().join("complex.rs");
    tokio::fs::write(
        &rust_file,
        r#"
fn high_complexity_function(x: i32) -> i32 {
    if x > 10 {
        if x > 20 {
            if x > 30 {
                if x > 40 {
                    if x > 50 {
                        100
                    } else {
                        90
                    }
                } else {
                    80
                }
            } else {
                70
            }
        } else {
            60
        }
    } else {
        50
    }
}
"#,
    )
    .await?;

    let violations = check_single_file_complexity(
        temp_dir.path(),
        &rust_file,
        5, // Low threshold to catch high complexity
    )
    .await?;

    // Should detect complexity violation
    assert!(!violations.is_empty());
    assert!(violations.iter().any(|v| v.check_type == "complexity"));
    assert!(violations.iter().any(|v| v.severity == "error"));

    Ok(())
}

/// Test check_single_file_complexity with missing file
#[tokio::test]
async fn test_check_single_file_complexity_missing_file() -> anyhow::Result<()> {
    let temp_dir = TempDir::new()?;
    let missing_file = temp_dir.path().join("missing.rs");

    let result = check_single_file_complexity(temp_dir.path(), &missing_file, 10).await;

    // Should return error for missing file
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("File not found"));

    Ok(())
}

/// Test check_single_file_complexity with low complexity function
#[tokio::test]
async fn test_check_single_file_complexity_no_violations() -> anyhow::Result<()> {
    let temp_dir = TempDir::new()?;

    let simple_file = temp_dir.path().join("simple.rs");
    tokio::fs::write(
        &simple_file,
        r#"
fn simple_function(x: i32) -> i32 {
    x * 2
}

fn another_simple(y: i32) -> i32 {
    y + 1
}
"#,
    )
    .await?;

    let violations = check_single_file_complexity(
        temp_dir.path(),
        &simple_file,
        10, // High threshold
    )
    .await?;

    // Should detect no violations
    assert_eq!(violations.len(), 0);

    Ok(())
}

/// Test write_markdown_summary_table functionality
#[test]
fn test_write_markdown_summary_table() -> anyhow::Result<()> {
    use crate::models::churn::ChurnSummary;
    use std::collections::HashMap;

    let mut output = String::new();

    // Create test summary data
    let summary = ChurnSummary {
        total_commits: 42,
        total_files_changed: 15,
        hotspot_files: vec!["file1.rs".into(), "file2.rs".into()],
        stable_files: vec!["lib.rs".into()],
        author_contributions: {
            let mut map = HashMap::new();
            map.insert("alice".to_string(), 5);
            map.insert("bob".to_string(), 3);
            map
        },
        mean_churn_score: 0.0,
        variance_churn_score: 0.0,
        stddev_churn_score: 0.0,
    };

    write_markdown_summary_table(&mut output, &summary)?;

    // Verify table structure
    assert!(output.contains("## Summary Statistics"));
    assert!(output.contains("| Metric | Value |"));
    assert!(output.contains("| Total Commits | 42 |"));
    assert!(output.contains("| Files Changed | 15 |"));
    assert!(output.contains("| Hotspot Files | 2 |"));
    assert!(output.contains("| Stable Files | 1 |"));
    assert!(output.contains("| Contributing Authors | 2 |"));

    Ok(())
}

/// Test write_markdown_summary_table with empty data
#[test]
fn test_write_markdown_summary_table_empty() -> anyhow::Result<()> {
    use crate::models::churn::ChurnSummary;
    use std::collections::HashMap;

    let mut output = String::new();

    let empty_summary = ChurnSummary {
        total_commits: 0,
        total_files_changed: 0,
        hotspot_files: vec![],
        stable_files: vec![],
        author_contributions: HashMap::new(),
        mean_churn_score: 0.0,
        variance_churn_score: 0.0,
        stddev_churn_score: 0.0,
    };

    write_markdown_summary_table(&mut output, &empty_summary)?;

    // Should still create proper table structure
    assert!(output.contains("## Summary Statistics"));
    assert!(output.contains("| Total Commits | 0 |"));
    assert!(output.contains("| Hotspot Files | 0 |"));

    Ok(())
}

/// Test write_markdown_summary_table output format
#[test]
fn test_write_markdown_summary_table_format() -> anyhow::Result<()> {
    use crate::models::churn::ChurnSummary;
    use std::collections::HashMap;

    let mut output = String::new();
    let summary = ChurnSummary {
        total_commits: 1,
        total_files_changed: 1,
        hotspot_files: vec!["test.rs".into()],
        stable_files: vec!["mod.rs".into()],
        author_contributions: {
            let mut map = HashMap::new();
            map.insert("dev".to_string(), 1);
            map
        },
        mean_churn_score: 0.0,
        variance_churn_score: 0.0,
        stddev_churn_score: 0.0,
    };

    write_markdown_summary_table(&mut output, &summary)?;

    // Check markdown table separator format
    assert!(output.contains("|--------|-------|"));
    // Check all rows have proper pipe separators
    let lines: Vec<&str> = output.lines().collect();
    let table_lines: Vec<&str> = lines
        .iter()
        .filter(|line| line.contains("|"))
        .cloned()
        .collect();

    assert!(table_lines.len() >= 3); // Header, separator, data rows

    Ok(())
}

/// Test print_single_check for different check types
#[test]
fn test_print_single_check_all_types() {
    use crate::cli::enums::QualityCheckType;

    // Test each check type (output goes to stderr, so we can't easily capture it)
    // But we can verify the function doesn't panic
    print_single_check(&QualityCheckType::Complexity);
    print_single_check(&QualityCheckType::DeadCode);
    print_single_check(&QualityCheckType::Satd);
    print_single_check(&QualityCheckType::Security);
    print_single_check(&QualityCheckType::Entropy);
    print_single_check(&QualityCheckType::Duplicates);
    print_single_check(&QualityCheckType::Coverage);

    // Should complete without panicking
    // Test passes if we reach this point without panicking
}

/// Test print_single_check with All type (should be handled by wildcard)
#[test]
fn test_print_single_check_all_and_wildcard() {
    use crate::cli::enums::QualityCheckType;

    // Test the wildcard case
    print_single_check(&QualityCheckType::All);

    // Should complete without panicking
    // Test passes if we reach this point without panicking
}

#[tokio::test]
async fn test_handle_analyze_makefile_basic() {
    // Create a temporary directory and Makefile
    let temp_dir = TempDir::new().unwrap();
    let makefile_path = temp_dir.path().join("Makefile");
    let mut file = std::fs::File::create(&makefile_path).unwrap();
    writeln!(file, "all:").unwrap();
    writeln!(file, "\techo 'Hello World'").unwrap();

    // Test basic makefile analysis
    let result = handle_analyze_makefile(
        makefile_path.clone(),
        vec![], // Empty rules vector
        MakefileOutputFormat::Human,
        false,
        None,
        10, // top_files
    )
    .await;

    // Should complete without error
    assert!(
        result.is_ok(),
        "Makefile analysis failed: {:?}",
        result.err()
    );
}

#[tokio::test]
async fn test_handle_analyze_makefile_with_rules() {
    let temp_dir = TempDir::new().unwrap();
    let makefile_path = temp_dir.path().join("Makefile");
    let mut file = std::fs::File::create(&makefile_path).unwrap();
    writeln!(file, "test:").unwrap();
    writeln!(file, "\tcargo test").unwrap();

    // Test with custom rules
    let result = handle_analyze_makefile(
        makefile_path,
        vec!["phonytargets".to_string()],
        MakefileOutputFormat::Json,
        false,
        Some("3.82".to_string()),
        10, // top_files
    )
    .await;

    assert!(result.is_ok());
}

#[tokio::test]
async fn test_handle_analyze_provability() {
    let temp_dir = TempDir::new().unwrap();
    let project_path = temp_dir.path().to_path_buf();

    // Create a simple Rust file for analysis
    let src_dir = project_path.join("src");
    std::fs::create_dir_all(&src_dir).unwrap();
    let rust_file = src_dir.join("lib.rs");
    let mut file = std::fs::File::create(&rust_file).unwrap();
    writeln!(file, "pub fn add(a: i32, b: i32) -> i32 {{").unwrap();
    writeln!(file, "    a + b").unwrap();
    writeln!(file, "}}").unwrap();

    // Test provability analysis
    let result = handle_analyze_provability(
        project_path,
        vec!["add".to_string()], // Functions to analyze
        10,                      // Analysis depth
        ProvabilityOutputFormat::Json,
        false, // high_confidence_only
        false, // include_evidence
        None,  // output path
        10,    // top_files
    )
    .await;

    assert!(result.is_ok());
}

#[tokio::test]
async fn test_handle_analyze_defect_prediction() {
    let temp_dir = TempDir::new().unwrap();
    let project_path = temp_dir.path().to_path_buf();

    // Create test files
    let src_dir = project_path.join("src");
    std::fs::create_dir_all(&src_dir).unwrap();
    let rust_file = src_dir.join("main.rs");
    let mut file = std::fs::File::create(&rust_file).unwrap();
    writeln!(file, "fn main() {{").unwrap();
    writeln!(file, "    println!(\"Hello, world!\");").unwrap();
    writeln!(file, "}}").unwrap();

    // Test defect prediction
    let result = handle_analyze_defect_prediction(
        project_path,
        0.5,   // confidence_threshold
        10,    // min_lines
        false, // include_low_confidence
        DefectPredictionOutputFormat::Summary,
        false, // high_risk_only
        false, // include_recommendations
        None,  // include
        None,  // exclude
        None,  // output
        false, // _perf
        10,    // top_files
    )
    .await;

    assert!(result.is_ok());
}

#[tokio::test]
async fn test_handle_analyze_proof_annotations() {
    use crate::cli::ProofAnnotationOutputFormat;
    let temp_dir = TempDir::new().unwrap();
    let project_path = temp_dir.path().to_path_buf();

    // Test proof annotation collection
    let result = handle_analyze_proof_annotations(
        project_path,
        ProofAnnotationOutputFormat::Json,
        false, // high_confidence_only
        false, // include_evidence
        None,  // sources
        None,  // confidence_levels
        None,  // output
        false, // _perf
        false, // clear_cache
        10,    // top_files
    )
    .await;

    assert!(result.is_ok());
}

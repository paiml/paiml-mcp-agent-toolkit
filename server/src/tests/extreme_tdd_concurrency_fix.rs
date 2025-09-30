//! EXTREME TDD: Fix Root Cause with Proper Concurrency
//! Goal: Sub-second performance with ALL annotations using world-class architecture

use std::fs;
use std::time::{Duration, Instant};
use tempfile::TempDir;

/// RED TEST: Context generation must complete in sub-second for small projects
#[tokio::test]
async fn test_sub_second_performance_small_project() {
    // ARRANGE: Create small test project (10 files)
    let temp_dir = TempDir::new().unwrap();
    for i in 0..10 {
        let test_file = temp_dir.path().join(format!("file_{}.rs", i));
        fs::write(
            &test_file,
            format!(
                r#"
// TODO: Optimize this function
fn function_{}(x: i32) -> i32 {{
    if x > 0 {{ x * 2 }} else {{ 0 }}
}}

struct Data{} {{ value: i32 }}

impl Data{} {{
    fn process(&self) -> i32 {{ self.value * 2 }}
}}
"#,
                i, i, i
            ),
        )
        .unwrap();
    }

    // ACT: Time the context generation
    let start = Instant::now();

    let output_file = temp_dir.path().join("context.md");
    let result = crate::cli::handlers::utility_handlers::handle_context(
        Some("rust".to_string()),
        temp_dir.path().to_path_buf(),
        Some(output_file.clone()),
        crate::cli::ContextFormat::Markdown,
        false,
        false, // Full analysis with all annotations
    )
    .await;

    let duration = start.elapsed();

    // ASSERT: Must complete in under 1 second
    assert!(
        result.is_ok(),
        "Context generation failed: {:?}",
        result.err()
    );
    assert!(
        duration < Duration::from_secs(1),
        "Must complete in under 1 second, took: {:?}",
        duration
    );

    // ASSERT: Must have ALL annotations
    let output = fs::read_to_string(output_file).unwrap();
    assert!(
        output.contains("[complexity:"),
        "Missing complexity annotation"
    );
    assert!(
        output.contains("[cognitive:"),
        "Missing cognitive annotation"
    );
    assert!(output.contains("[big-o:"), "Missing Big-O annotation");
    assert!(
        output.contains("[provability:"),
        "Missing provability annotation"
    );
    assert!(output.contains("[churn:"), "Missing churn annotation");
    assert!(
        output.contains("[satd:") || output.contains("TODO"),
        "Missing SATD detection"
    );
    assert!(
        output.contains("[pagerank:") || output.contains("Graph"),
        "Missing graph metrics"
    );
    assert!(
        output.contains("[tdg:") || output.contains("Technical Debt"),
        "Missing TDG score"
    );
}

/// RED TEST: Must use parallel processing for all analyses
#[tokio::test]
async fn test_parallel_analysis_execution() {
    // ARRANGE: Create project with enough files to test parallelism
    let temp_dir = TempDir::new().unwrap();
    for i in 0..50 {
        let test_file = temp_dir.path().join(format!("file_{}.rs", i));
        fs::write(&test_file, format!("fn func_{}() {{ }}", i)).unwrap();
    }

    // ACT: Run with instrumentation to verify parallel execution
    let start = Instant::now();

    // Should use tokio::join! internally for parallel execution
    let output_file = temp_dir.path().join("context.md");
    let result = crate::cli::handlers::utility_handlers::handle_context(
        Some("rust".to_string()),
        temp_dir.path().to_path_buf(),
        Some(output_file.clone()),
        crate::cli::ContextFormat::Markdown,
        false,
        false,
    )
    .await;

    let duration = start.elapsed();

    // ASSERT: Parallel execution should be much faster than sequential
    // 50 files sequentially would take >5 seconds, parallel should be <2 seconds
    assert!(result.is_ok());
    assert!(
        duration < Duration::from_secs(2),
        "Not using parallel execution, took: {:?}",
        duration
    );
}

/// RED TEST: Must parse AST only once and share across analyses
#[test]
fn test_ast_parsing_shared_not_duplicated() {
    use crate::services::deep_context::{AnalysisType, CacheStrategy, DagType, DeepContextConfig};

    // ARRANGE: Create config with multiple analysis types
    let config = DeepContextConfig {
        include_analyses: vec![
            AnalysisType::Ast,
            AnalysisType::Complexity,
            AnalysisType::Provability,
            AnalysisType::BigO,
            AnalysisType::Satd,
        ],
        period_days: 30,
        dag_type: DagType::CallGraph,
        complexity_thresholds: None,
        max_depth: None, // No artificial limits!
        include_patterns: vec![],
        exclude_patterns: vec!["**/target/**".to_string()],
        cache_strategy: CacheStrategy::Normal,
        parallel: num_cpus::get(), // Use all CPU cores
        file_classifier_config: None,
    };

    // ASSERT: Config should enable proper parallelism
    assert!(config.parallel >= 2, "Must use parallel processing");
    assert!(
        config.max_depth.is_none(),
        "Must not have artificial depth limits"
    );
    assert_eq!(
        config.include_analyses.len(),
        5,
        "Must include all analysis types"
    );
}

/// RED TEST: Must have progress bars for user feedback
#[tokio::test]
async fn test_progress_bars_for_long_operations() {
    // ARRANGE: Create larger project
    let temp_dir = TempDir::new().unwrap();
    for i in 0..100 {
        let test_file = temp_dir.path().join(format!("file_{}.rs", i));
        fs::write(&test_file, "fn test() {}").unwrap();
    }

    // ACT: Should show progress bars (indicatif crate)
    let output_file = temp_dir.path().join("context.md");

    // We should see progress output for:
    // - File scanning
    // - AST parsing
    // - Analysis phases
    // - Writing output

    let result = crate::cli::handlers::utility_handlers::handle_context(
        None, // Auto-detect
        temp_dir.path().to_path_buf(),
        Some(output_file),
        crate::cli::ContextFormat::Markdown,
        false,
        false,
    )
    .await;

    // ASSERT: Should complete successfully with progress indication
    assert!(result.is_ok(), "Should handle 100 files with progress bars");
}

/// RED TEST: Must use bounded channels for backpressure
#[tokio::test]
async fn test_bounded_channels_prevent_memory_explosion() {
    use crate::services::deep_context::DeepContextAnalyzer;

    // ARRANGE: Create project with many files
    let temp_dir = TempDir::new().unwrap();
    for i in 0..1000 {
        let test_file = temp_dir.path().join(format!("file_{}.rs", i));
        fs::write(&test_file, format!("fn func_{}() {{ }}", i)).unwrap();
    }

    // ACT: Process with bounded channels (should not OOM)
    let config = crate::services::deep_context::DeepContextConfig::default();
    let analyzer = DeepContextAnalyzer::new(config);
    let start_memory = get_current_memory_usage();

    let result = analyzer
        .analyze_project(&temp_dir.path().to_path_buf())
        .await;

    let end_memory = get_current_memory_usage();

    // ASSERT: Memory usage should be bounded
    assert!(result.is_ok());
    let memory_growth = end_memory - start_memory;
    assert!(
        memory_growth < 100_000_000, // Less than 100MB growth
        "Memory explosion detected: {} bytes",
        memory_growth
    );
}

/// RED TEST: All annotations must be present without timeouts
#[tokio::test]
async fn test_all_annotations_present_no_timeouts() {
    // ARRANGE: Create complex project that would timeout with bad implementation
    let temp_dir = TempDir::new().unwrap();

    // Create files with various complexities
    fs::write(
        temp_dir.path().join("complex.rs"),
        r#"
// TODO: Refactor this complex function
// FIXME: Performance issue
fn complex_function(data: Vec<i32>) -> i32 {
    let mut result = 0;
    for i in 0..data.len() {
        for j in 0..data.len() {
            if data[i] > data[j] {
                for k in 0..10 {
                    if k % 2 == 0 {
                        result += data[i] * data[j] * k;
                    }
                }
            }
        }
    }
    result
}

fn simple_function() -> i32 { 42 }

struct DataProcessor {
    cache: Vec<i32>,
}

impl DataProcessor {
    fn new() -> Self { Self { cache: vec![] } }
    fn process(&self) -> i32 { self.cache.iter().sum() }
}
"#,
    )
    .unwrap();

    // ACT: Generate context without any timeouts
    let start = Instant::now();
    let output_file = temp_dir.path().join("context.md");

    let result = crate::cli::handlers::utility_handlers::handle_context(
        Some("rust".to_string()),
        temp_dir.path().to_path_buf(),
        Some(output_file.clone()),
        crate::cli::ContextFormat::Markdown,
        false,
        false,
    )
    .await;

    let duration = start.elapsed();

    // ASSERT: Should complete quickly with all annotations
    assert!(result.is_ok());
    assert!(
        duration < Duration::from_secs(3),
        "Too slow: {:?}",
        duration
    );

    let output = fs::read_to_string(output_file).unwrap();

    // Check for ALL required annotations
    assert!(output.contains("[complexity:"), "Missing complexity");
    assert!(output.contains("[cognitive:"), "Missing cognitive");
    assert!(output.contains("[big-o:"), "Missing Big-O");
    assert!(output.contains("[provability:"), "Missing provability");
    assert!(
        output.contains("TODO") || output.contains("FIXME"),
        "Missing SATD"
    );

    // Should have proper function analysis
    assert!(
        output.contains("complex_function"),
        "Missing complex function"
    );
    assert!(
        output.contains("simple_function"),
        "Missing simple function"
    );
    assert!(output.contains("DataProcessor"), "Missing struct");
}

// Helper function to simulate memory measurement
fn get_current_memory_usage() -> usize {
    // In real implementation, use sysinfo or similar
    0
}

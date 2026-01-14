//! EXTREME TDD: Stack Overflow Fix Tests
//! Goal: Fix stack overflow in deep context generation without --skip-expensive-metrics

use tempfile::TempDir;
use std::fs;

/// RED TEST: Context generation should NOT stack overflow with full analysis
#[tokio::test]
async fn test_context_generation_no_stack_overflow_full_analysis() {
    // ARRANGE: Create test project with moderate complexity
    let temp_dir = TempDir::new().unwrap();
    let test_file = temp_dir.path().join("moderate.rs");

    let code = r#"
fn simple_function() -> i32 {
    42
}

fn moderate_function(x: i32) -> i32 {
    if x > 0 {
        x * 2
    } else {
        0
    }
}

struct TestStruct {
    value: i32,
}

impl TestStruct {
    fn new() -> Self {
        Self { value: 0 }
    }
}
"#;

    fs::write(&test_file, code).unwrap();

    // ACT & ASSERT: Should complete without stack overflow
    let result = tokio::spawn(async move {
        let output_file = temp_dir.path().join("context.md");
        crate::cli::handlers::utility_handlers::handle_context(
            Some("rust".to_string()),
            temp_dir.path().to_path_buf(),
            Some(output_file.clone()),
            crate::cli::handlers::utility_handlers::ContextFormat::Markdown,
            false, // include_large_files
            false, // skip_expensive_metrics = FALSE (full analysis)
        ).await
    }).await;

    // Should not panic with stack overflow
    assert!(result.is_ok(), "Context generation should not stack overflow");
    let generation_result = result.unwrap();
    assert!(generation_result.is_ok(), "Context generation should succeed: {:?}", generation_result.err());
}

/// RED TEST: Context generation should work with reasonable memory usage
#[tokio::test]
async fn test_context_generation_memory_bounded() {
    // ARRANGE: Create test with many files but bounded depth
    let temp_dir = TempDir::new().unwrap();

    // Create 50 small files instead of deep recursion
    for i in 0..50 {
        let test_file = temp_dir.path().join(format!("file_{}.rs", i));
        let code = format!(r#"
fn function_{}() -> i32 {{
    {}
}}

struct Struct{} {{
    field: i32,
}}
"#, i, i, i);
        fs::write(&test_file, code).unwrap();
    }

    // ACT: Should complete without stack overflow in reasonable time
    let start = std::time::Instant::now();

    let output_file = temp_dir.path().join("context.md");
    let result = crate::cli::handlers::utility_handlers::handle_context(
        Some("rust".to_string()),
        temp_dir.path().to_path_buf(),
        Some(output_file.clone()),
        crate::cli::handlers::utility_handlers::ContextFormat::Markdown,
        false, // include_large_files
        false, // skip_expensive_metrics = FALSE (full analysis)
    ).await;

    let duration = start.elapsed();

    // ASSERT: Should complete within reasonable time and memory
    assert!(result.is_ok(), "Context generation should succeed");
    assert!(duration.as_secs() < 30, "Should complete within 30 seconds, took {:?}", duration);

    // Verify output contains enhanced AST
    let output = fs::read_to_string(output_file).unwrap();
    assert!(output.contains("Enhanced Annotated AST Context"), "Should contain enhanced AST header");
    assert!(output.contains("Function"), "Should contain function annotations");
}

/// RED TEST: Deep context analyzer should have stack size limits
#[tokio::test]
async fn test_deep_context_analyzer_stack_safe() {
    use crate::services::deep_context::{DeepContextAnalyzer, DeepContextConfig, AnalysisType, CacheStrategy, DagType};

    // ARRANGE: Create analyzer with full config that previously caused stack overflow
    let temp_dir = TempDir::new().unwrap();
    let test_file = temp_dir.path().join("complex.rs");

    let code = r#"
fn deeply_nested() -> i32 {
    if true {
        if true {
            if true {
                if true {
                    if true {
                        42
                    } else {
                        0
                    }
                } else {
                    0
                }
            } else {
                0
            }
        } else {
            0
        }
    } else {
        0
    }
}
"#;

    fs::write(&test_file, code).unwrap();

    let config = DeepContextConfig {
        include_analyses: vec![
            AnalysisType::Ast,
            AnalysisType::Complexity,
            AnalysisType::Churn,
            AnalysisType::TechnicalDebtGradient,
            AnalysisType::DeadCode,
            AnalysisType::Satd,
            AnalysisType::Provability,
            AnalysisType::BigO,
        ],
        period_days: 30,
        dag_type: DagType::CallGraph,
        complexity_thresholds: None,
        max_depth: Some(5), // Limit recursion depth
        include_patterns: vec![],
        exclude_patterns: vec![],
        cache_strategy: CacheStrategy::Normal,
        parallel: 2, // Reduce parallelism to avoid resource exhaustion
        file_classifier_config: None,
    };

    // ACT: Should not stack overflow
    let analyzer = DeepContextAnalyzer::new(config);
    let result = analyzer.analyze_project(&temp_dir.path()).await;

    // ASSERT: Should succeed without stack overflow
    assert!(result.is_ok(), "Deep context analysis should not stack overflow: {:?}", result.err());
}
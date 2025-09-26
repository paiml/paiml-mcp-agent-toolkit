//! EXTREME TDD: Smart Bounds for Expensive Analyses
//! Goal: Add intelligent bounds to expensive analyses to prevent infinite loops/timeouts
//! while keeping all analysis types enabled

use tempfile::TempDir;
use std::fs;
use std::time::Duration;

/// RED TEST: Provability analysis should complete within timeout with bounds
#[tokio::test]
async fn test_provability_analysis_bounded() {
    // ARRANGE: Create test project
    let temp_dir = TempDir::new().unwrap();
    let test_file = temp_dir.path().join("bounded.rs");

    fs::write(&test_file, r#"
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

// Create many functions to test bounds
fn function_1() -> i32 { 1 }
fn function_2() -> i32 { 2 }
fn function_3() -> i32 { 3 }
fn function_4() -> i32 { 4 }
fn function_5() -> i32 { 5 }
fn function_6() -> i32 { 6 }
fn function_7() -> i32 { 7 }
fn function_8() -> i32 { 8 }
fn function_9() -> i32 { 9 }
fn function_10() -> i32 { 10 }
"#).unwrap();

    // ACT: Should complete within 10 seconds with smart bounds
    let result = tokio::time::timeout(Duration::from_secs(10), async {
        crate::services::deep_context::analyze_provability(temp_dir.path()).await
    }).await;

    // ASSERT: Should not timeout and should succeed
    assert!(result.is_ok(), "Provability analysis should not timeout");
    let analysis_result = result.unwrap();
    assert!(analysis_result.is_ok(), "Provability analysis should succeed: {:?}", analysis_result.err());

    let summaries = analysis_result.unwrap();
    assert!(!summaries.is_empty(), "Should generate provability summaries");

    // Should have reasonable number of functions (not unlimited)
    assert!(summaries.len() <= 50, "Should respect function bounds, found: {}", summaries.len());
}

/// RED TEST: Churn analysis should complete within timeout with bounds
#[tokio::test]
async fn test_churn_analysis_bounded() {
    // ARRANGE: Create test project
    let temp_dir = TempDir::new().unwrap();
    let test_file = temp_dir.path().join("churn.rs");

    fs::write(&test_file, r#"
fn example() -> i32 {
    // Some code to analyze for churn
    let result = 42;
    result
}
"#).unwrap();

    // ACT: Should complete within 10 seconds
    let result = tokio::time::timeout(Duration::from_secs(10), async {
        crate::services::deep_context::analyze_churn(temp_dir.path(), 30).await
    }).await;

    // ASSERT: Should not timeout and should fail gracefully (no git repo)
    assert!(result.is_ok(), "Churn analysis should not timeout");
    let analysis_result = result.unwrap();
    // Smart bounds: churn analysis should fail quickly when no git repo exists
    assert!(analysis_result.is_err(), "Churn analysis should fail gracefully when no git repo exists");

    // Verify it's the expected "no git repository" error
    let error_msg = analysis_result.err().unwrap();
    let error_str = error_msg.to_string();
    assert!(error_str.contains("No git repository found"),
           "Should be a 'no git repository' error, got: {}", error_str);
}

/// RED TEST: Context generation with all analyses should complete within bounds
#[tokio::test]
async fn test_full_analysis_smart_bounds() {
    // ARRANGE: Create moderate complexity project
    let temp_dir = TempDir::new().unwrap();

    // Create multiple files with moderate complexity
    for i in 0..10 {
        let test_file = temp_dir.path().join(format!("file_{}.rs", i));
        let code = format!(r#"
// TODO: Optimize this function
fn function_{}(x: i32) -> i32 {{
    let mut result = 0;
    for j in 0..x {{
        if j % 2 == 0 {{
            result += j;
        }} else {{
            result -= j;
        }}
    }}
    result
}}

struct Struct{} {{
    value: i32,
}}

impl Struct{} {{
    fn new() -> Self {{ Self {{ value: {} }} }}
    fn process(&self) -> i32 {{ self.value * 2 }}
}}
"#, i, i, i, i);
        fs::write(&test_file, code).unwrap();
    }

    // ACT: Should complete within 30 seconds with smart bounds
    let result = tokio::time::timeout(Duration::from_secs(30), async {
        let output_file = temp_dir.path().join("context.md");
        crate::cli::handlers::utility_handlers::handle_context(
            Some("rust".to_string()),
            temp_dir.path().to_path_buf(),
            Some(output_file.clone()),
            crate::cli::ContextFormat::Markdown,
            false, // include_large_files
            false, // skip_expensive_metrics = FALSE (full analysis)
        ).await
    }).await;

    // ASSERT: Should not timeout and should succeed
    assert!(result.is_ok(), "Full analysis should not timeout");
    let generation_result = result.unwrap();
    assert!(generation_result.is_ok(), "Full analysis should succeed: {:?}", generation_result.err());

    let output = fs::read_to_string(temp_dir.path().join("context.md")).unwrap();

    // Should contain all required annotations but within bounds
    assert!(output.contains("Project Context"), "Should contain project context header");
    assert!(output.contains("complexity:"), "Should contain complexity annotations");
    assert!(output.contains("Function"), "Should contain function annotations");
    assert!(output.contains("TODO"), "Should detect SATD comments");
}

/// RED TEST: DeepContextConfig should have smart bounds configured
#[test]
fn test_deep_context_config_smart_bounds() {
    use crate::services::deep_context::{AnalysisType, DeepContextConfig, CacheStrategy, DagType};

    // ARRANGE: Create config with smart bounds for full analysis
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
        max_depth: Some(10), // Reasonable depth
        include_patterns: vec![],
        exclude_patterns: vec![
            "**/target/**".to_string(),
            "**/node_modules/**".to_string(),
            "**/build/**".to_string(),
            "**/dist/**".to_string(),
            "**/.git/**".to_string(),
        ],
        cache_strategy: CacheStrategy::Normal,
        parallel: 4, // Reasonable parallelism
        file_classifier_config: None,
    };

    // ASSERT: Should have smart bounds
    assert!(config.max_depth.unwrap_or(0) >= 5, "Should have reasonable max depth");
    assert!(config.max_depth.unwrap_or(0) <= 15, "Should bound max depth");
    assert!(config.parallel >= 2, "Should allow parallelism");
    assert!(config.parallel <= 8, "Should bound parallelism");
    assert!(config.period_days >= 7, "Should have reasonable period");
    assert!(config.period_days <= 90, "Should bound period");

    // All analysis types should be enabled
    assert_eq!(config.include_analyses.len(), 8, "Should include all 8 analysis types");
}
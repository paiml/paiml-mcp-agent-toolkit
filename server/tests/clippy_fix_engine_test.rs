//! TDD Tests for Automated Clippy Fix Engine
//! 
//! Following A+ code standards: ALL functions ≤10 complexity
//! Test-first development per Toyota Way principles

use pmat::services::clippy_fix::{
    ClippyDiagnostic, ClippyFixEngine, ConfidenceLevel, FixResult, DiagnosticLevel
};
use std::path::PathBuf;

#[tokio::test]
async fn test_parse_clippy_json_output() {
    // Given: Raw clippy JSON output
    let clippy_json = r#"{
        "reason": "compiler-message",
        "message": {
            "code": {"code": "clippy::needless_return"},
            "level": "warning",
            "message": "unneeded `return` statement",
            "spans": [{
                "file_name": "src/main.rs",
                "line_start": 10,
                "line_end": 10,
                "column_start": 5,
                "column_end": 15
            }]
        }
    }"#;
    
    // When: Parsing the diagnostic
    let diagnostic = ClippyDiagnostic::from_json(clippy_json).unwrap();
    
    // Then: Diagnostic is correctly parsed
    assert_eq!(diagnostic.code, "clippy::needless_return");
    assert_eq!(diagnostic.level, DiagnosticLevel::Warning);
    assert_eq!(diagnostic.line_start, 10);
}

#[tokio::test]
async fn test_confidence_scoring_for_safe_fixes() {
    // Given: A simple needless_return diagnostic
    let diagnostic = ClippyDiagnostic {
        code: "clippy::needless_return".to_string(),
        level: DiagnosticLevel::Warning,
        message: "unneeded `return` statement".to_string(),
        file: PathBuf::from("src/main.rs"),
        line_start: 10,
        line_end: 10,
        column_start: 5,
        column_end: 15,
        suggestion: Some("remove `return`".to_string()),
    };
    
    // When: Calculating confidence score
    let engine = ClippyFixEngine::new();
    let confidence = engine.calculate_confidence(&diagnostic);
    
    // Then: High confidence for safe fix
    assert_eq!(confidence, ConfidenceLevel::High);
}

#[tokio::test]
async fn test_confidence_scoring_for_risky_fixes() {
    // Given: A complex lifetime-related diagnostic
    let diagnostic = ClippyDiagnostic {
        code: "clippy::needless_lifetimes".to_string(),
        level: DiagnosticLevel::Warning,
        message: "explicit lifetimes given in parameter types".to_string(),
        file: PathBuf::from("src/lib.rs"),
        line_start: 50,
        line_end: 55,
        column_start: 1,
        column_end: 80,
        suggestion: None,
    };
    
    // When: Calculating confidence score
    let engine = ClippyFixEngine::new();
    let confidence = engine.calculate_confidence(&diagnostic);
    
    // Then: Low confidence for risky fix
    assert_eq!(confidence, ConfidenceLevel::Low);
}

#[tokio::test]
async fn test_ast_based_fix_application() {
    // Given: Source code with needless return
    let source = r#"
fn calculate(x: i32) -> i32 {
    return x * 2;
}
"#;
    
    let diagnostic = ClippyDiagnostic {
        code: "clippy::needless_return".to_string(),
        level: DiagnosticLevel::Warning,
        message: "unneeded `return` statement".to_string(),
        file: PathBuf::from("test.rs"),
        line_start: 3,
        line_end: 3,
        column_start: 5,
        column_end: 17,
        suggestion: Some("x * 2".to_string()),
    };
    
    // When: Applying the fix
    let engine = ClippyFixEngine::new();
    let result = engine.apply_fix(source, &diagnostic).await.unwrap();
    
    // Then: Fix is correctly applied
    assert!(result.success);
    assert!(result.modified_source.contains("x * 2"));
    assert!(!result.modified_source.contains("return"));
}

#[tokio::test]
async fn test_batch_fix_with_dependencies() {
    // Given: Multiple diagnostics with dependencies
    let diagnostics = vec![
        ClippyDiagnostic {
            code: "clippy::redundant_clone".to_string(),
            level: DiagnosticLevel::Warning,
            message: "redundant clone".to_string(),
            file: PathBuf::from("src/main.rs"),
            line_start: 10,
            line_end: 10,
            column_start: 5,
            column_end: 20,
            suggestion: Some("remove .clone()".to_string()),
        },
        ClippyDiagnostic {
            code: "clippy::needless_return".to_string(),
            level: DiagnosticLevel::Warning,
            message: "unneeded `return` statement".to_string(),
            file: PathBuf::from("src/main.rs"),
            line_start: 15,
            line_end: 15,
            column_start: 5,
            column_end: 25,
            suggestion: Some("remove return".to_string()),
        },
    ];
    
    // When: Applying batch fixes
    let engine = ClippyFixEngine::new();
    let results = engine.apply_batch_fixes(&diagnostics).await.unwrap();
    
    // Then: All fixes applied in correct order
    assert_eq!(results.len(), 2);
    assert!(results.iter().all(|r| r.success));
}

#[tokio::test]
async fn test_transactional_rollback_on_error() {
    // Given: A fix that will break compilation
    let diagnostic = ClippyDiagnostic {
        code: "clippy::some_complex_rule".to_string(),
        level: DiagnosticLevel::Warning,
        message: "complex issue".to_string(),
        file: PathBuf::from("src/main.rs"),
        line_start: 10,
        line_end: 10,
        column_start: 1,
        column_end: 50,
        suggestion: Some("invalid syntax {{".to_string()),
    };
    
    // When: Applying fix that breaks compilation
    let engine = ClippyFixEngine::new();
    let source = "fn main() { println!(\"hello\"); }";
    let result = engine.apply_fix_with_validation(source, &diagnostic).await;
    
    // Then: Fix is rolled back
    assert!(result.is_err() || !result.unwrap().success);
}

#[tokio::test]
async fn test_caching_layer_performance() {
    use std::time::Instant;
    
    // Given: Same diagnostic applied multiple times
    let diagnostic = ClippyDiagnostic {
        code: "clippy::needless_return".to_string(),
        level: DiagnosticLevel::Warning,
        message: "unneeded `return` statement".to_string(),
        file: PathBuf::from("src/main.rs"),
        line_start: 10,
        line_end: 10,
        column_start: 5,
        column_end: 15,
        suggestion: Some("remove return".to_string()),
    };
    
    let engine = ClippyFixEngine::new();
    let source = "fn test() { return 42; }";
    
    // When: First application (cache miss)
    let start = Instant::now();
    let _ = engine.apply_fix(source, &diagnostic).await.unwrap();
    let first_duration = start.elapsed();
    
    // When: Second application (cache hit)
    let start = Instant::now();
    let _ = engine.apply_fix(source, &diagnostic).await.unwrap();
    let second_duration = start.elapsed();
    
    // Then: Cached version is significantly faster
    assert!(second_duration < first_duration / 2);
}

#[tokio::test]
async fn test_fix_filtering_by_confidence() {
    // Given: Mixed confidence diagnostics
    let diagnostics = vec![
        (ClippyDiagnostic {
            code: "clippy::needless_return".to_string(),
            level: DiagnosticLevel::Warning,
            message: "unneeded return".to_string(),
            file: PathBuf::from("src/main.rs"),
            line_start: 10,
            line_end: 10,
            column_start: 1,
            column_end: 10,
            suggestion: Some("remove".to_string()),
        }, ConfidenceLevel::High),
        (ClippyDiagnostic {
            code: "clippy::complex_lifetime".to_string(),
            level: DiagnosticLevel::Warning,
            message: "complex lifetime".to_string(),
            file: PathBuf::from("src/lib.rs"),
            line_start: 20,
            line_end: 25,
            column_start: 1,
            column_end: 80,
            suggestion: None,
        }, ConfidenceLevel::Low),
    ];
    
    // When: Filtering for high confidence only
    let engine = ClippyFixEngine::new();
    let filtered = engine.filter_by_confidence(
        diagnostics.clone(),
        ConfidenceLevel::High
    );
    
    // Then: Only high confidence fixes selected
    assert_eq!(filtered.len(), 1);
    assert_eq!(filtered[0].0.code, "clippy::needless_return");
}

#[tokio::test]
async fn test_parallel_fix_application() {
    // Given: Multiple independent files
    let diagnostics = vec![
        ClippyDiagnostic {
            code: "clippy::needless_return".to_string(),
            level: DiagnosticLevel::Warning,
            message: "unneeded return".to_string(),
            file: PathBuf::from("src/module1.rs"),
            line_start: 10,
            line_end: 10,
            column_start: 1,
            column_end: 10,
            suggestion: Some("remove".to_string()),
        },
        ClippyDiagnostic {
            code: "clippy::needless_return".to_string(),
            level: DiagnosticLevel::Warning,
            message: "unneeded return".to_string(),
            file: PathBuf::from("src/module2.rs"),
            line_start: 15,
            line_end: 15,
            column_start: 1,
            column_end: 10,
            suggestion: Some("remove".to_string()),
        },
    ];
    
    // When: Applying fixes in parallel
    let engine = ClippyFixEngine::new();
    let results = engine.apply_parallel_fixes(&diagnostics).await.unwrap();
    
    // Then: All fixes applied successfully
    assert_eq!(results.len(), 2);
    assert!(results.iter().all(|r| r.success));
}

#[tokio::test]
async fn test_comprehensive_fix_report() {
    // Given: Applied fixes
    let engine = ClippyFixEngine::new();
    let diagnostic = ClippyDiagnostic {
        code: "clippy::needless_return".to_string(),
        level: DiagnosticLevel::Warning,
        message: "unneeded return".to_string(),
        file: PathBuf::from("src/main.rs"),
        line_start: 10,
        line_end: 10,
        column_start: 1,
        column_end: 10,
        suggestion: Some("remove".to_string()),
    };
    
    // When: Generating report
    let source = "fn test() { return 42; }";
    let result = engine.apply_fix(source, &diagnostic).await.unwrap();
    let report = engine.generate_report(vec![result]);
    
    // Then: Report contains all required information
    assert!(report.total_diagnostics > 0);
    assert!(report.successful_fixes > 0);
    assert!(report.success_rate >= 0.0 && report.success_rate <= 100.0);
    assert!(!report.fixed_files.is_empty());
}
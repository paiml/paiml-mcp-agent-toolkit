use pmat::models::proxy::{ProxyMode, ProxyOperation, ProxyRequest, ProxyStatus, QualityConfig};
use pmat::services::quality_proxy::QualityProxyService;

#[tokio::test]
async fn test_quality_proxy_service_integration() {
    let service = QualityProxyService::new();

    // Test 1: High-quality code should pass
    let good_code = r#"/// A simple greeting function
/// 
/// Returns a greeting message
pub fn greet(name: &str) -> String {
    format!("Hello, {}!", name)
}"#;

    let request = ProxyRequest {
        operation: ProxyOperation::Write,
        file_path: "greet.rs".to_string(),
        content: Some(good_code.to_string()),
        old_content: None,
        new_content: None,
        mode: ProxyMode::Strict,
        quality_config: QualityConfig::default(),
    };

    let response = service.proxy_operation(request).await.unwrap();
    assert!(matches!(response.status, ProxyStatus::Accepted));
    assert!(response.quality_report.passed);
    assert_eq!(response.quality_report.metrics.satd_count, 0);
}

#[tokio::test]
async fn test_quality_proxy_rejects_satd() {
    let service = QualityProxyService::new();

    // Test code with SATD
    let bad_code = r#"fn process_data() {
    // TODO: actually implement this
    // FIXME: this is just a stub
    unimplemented!("not done yet")
}"#;

    let request = ProxyRequest {
        operation: ProxyOperation::Write,
        file_path: "process.rs".to_string(),
        content: Some(bad_code.to_string()),
        old_content: None,
        new_content: None,
        mode: ProxyMode::Strict,
        quality_config: QualityConfig {
            allow_satd: false,
            ..Default::default()
        },
    };

    let response = service.proxy_operation(request).await.unwrap();
    assert!(matches!(response.status, ProxyStatus::Rejected));
    assert!(!response.quality_report.passed);
    assert!(response.quality_report.metrics.satd_count > 0);

    // Check that violations are reported
    let satd_violations = response
        .quality_report
        .violations
        .iter()
        .filter(|v| matches!(v.violation_type, pmat::models::proxy::ViolationType::Satd))
        .count();
    assert!(satd_violations > 0);
}

#[tokio::test]
async fn test_quality_proxy_advisory_mode() {
    let service = QualityProxyService::new();

    // Code with issues
    let code = r#"pub fn undocumented_function() {
    // TODO: add documentation
    println!("This needs docs");
}"#;

    let request = ProxyRequest {
        operation: ProxyOperation::Write,
        file_path: "advisory.rs".to_string(),
        content: Some(code.to_string()),
        old_content: None,
        new_content: None,
        mode: ProxyMode::Advisory,
        quality_config: QualityConfig::default(),
    };

    let response = service.proxy_operation(request).await.unwrap();

    // Advisory mode should accept but report violations
    assert!(matches!(response.status, ProxyStatus::Accepted));
    assert!(!response.quality_report.violations.is_empty());
}

#[tokio::test]
async fn test_quality_proxy_auto_fix_mode() {
    let service = QualityProxyService::new();

    // Code with SATD that can be removed
    let code = r#"fn calculate() -> i32 {
    // TODO: optimize this calculation
    let result = 42;
    result
}"#;

    let request = ProxyRequest {
        operation: ProxyOperation::Write,
        file_path: "autofix.rs".to_string(),
        content: Some(code.to_string()),
        old_content: None,
        new_content: None,
        mode: ProxyMode::AutoFix,
        quality_config: QualityConfig {
            allow_satd: false,
            require_docs: false,
            ..Default::default()
        },
    };

    let response = service.proxy_operation(request).await.unwrap();

    // Should either fix or reject
    assert!(
        matches!(response.status, ProxyStatus::Modified)
            || matches!(response.status, ProxyStatus::Rejected)
    );

    if matches!(response.status, ProxyStatus::Modified) {
        assert!(response.refactoring_applied);
        assert!(!response.final_content.contains("TODO"));
    }
}

#[tokio::test]
async fn test_quality_proxy_edit_operation() {
    let service = QualityProxyService::new();

    let original = r#"/// Original function
pub fn original() {
    println!("Original");
}"#;

    let request = ProxyRequest {
        operation: ProxyOperation::Edit,
        file_path: "edit.rs".to_string(),
        content: Some(original.to_string()),
        old_content: Some("Original".to_string()),
        new_content: Some("Modified".to_string()),
        mode: ProxyMode::Strict,
        quality_config: QualityConfig::default(),
    };

    let response = service.proxy_operation(request).await.unwrap();
    assert!(matches!(response.status, ProxyStatus::Accepted));
    assert!(response.final_content.contains("Modified"));
    assert!(!response.final_content.contains("Original"));
}

#[tokio::test]
async fn test_quality_proxy_complexity_check() {
    let service = QualityProxyService::new();

    // Create code with high complexity
    let complex_code = r#"fn complex_logic(a: i32, b: i32, c: i32) -> i32 {
    if a > 0 {
        if b > 0 {
            if c > 0 {
                for i in 0..10 {
                    if i % 2 == 0 {
                        for j in 0..5 {
                            if j > 2 {
                                if a + b > c {
                                    return a + b + c + i + j;
                                }
                            }
                        }
                    }
                }
            } else {
                return -c;
            }
        } else {
            return -b;
        }
    } else {
        return -a;
    }
    0
}"#;

    let request = ProxyRequest {
        operation: ProxyOperation::Write,
        file_path: "complex.rs".to_string(),
        content: Some(complex_code.to_string()),
        old_content: None,
        new_content: None,
        mode: ProxyMode::Strict,
        quality_config: QualityConfig {
            max_complexity: 10, // Low threshold
            allow_satd: true,
            require_docs: false,
            auto_format: false,
        },
    };

    let response = service.proxy_operation(request).await.unwrap();

    // With complexity of 8 and threshold of 10, it should be accepted
    assert!(matches!(response.status, ProxyStatus::Accepted));
    assert!(response.quality_report.passed);

    // Complexity should be within threshold, but may have doc warnings
    // Only check that there are no error-level violations
    let has_errors = response
        .quality_report
        .violations
        .iter()
        .any(|v| matches!(v.severity, pmat::models::proxy::ViolationSeverity::Error));
    assert!(!has_errors, "Should have no error-level violations");
}

#[tokio::test]
async fn test_quality_proxy_documentation_check() {
    let service = QualityProxyService::new();

    // Public function without documentation
    let code = r#"pub fn important_calculation(x: i32, y: i32) -> i32 {
    x * y + (x - y)
}

pub struct DataProcessor {
    value: i32,
}

pub enum Status {
    Active,
    Inactive,
}"#;

    let request = ProxyRequest {
        operation: ProxyOperation::Write,
        file_path: "nodocs.rs".to_string(),
        content: Some(code.to_string()),
        old_content: None,
        new_content: None,
        mode: ProxyMode::Strict,
        quality_config: QualityConfig {
            require_docs: true,
            allow_satd: true,
            ..Default::default()
        },
    };

    let response = service.proxy_operation(request).await.unwrap();

    // Should have documentation violations
    let doc_violations = response
        .quality_report
        .violations
        .iter()
        .filter(|v| matches!(v.violation_type, pmat::models::proxy::ViolationType::Docs))
        .count();

    assert!(doc_violations >= 3); // Function, struct, and enum
}

#[tokio::test]
async fn test_quality_proxy_append_operation() {
    let service = QualityProxyService::new();

    let existing = r#"/// Existing function
pub fn existing() {
    println!("Existing");
}"#;

    let new_content = r#"/// New function
pub fn new_function() {
    println!("New");
}"#;

    let request = ProxyRequest {
        operation: ProxyOperation::Append,
        file_path: "append.rs".to_string(),
        content: Some(new_content.to_string()),
        old_content: Some(existing.to_string()),
        new_content: None,
        mode: ProxyMode::Strict,
        quality_config: QualityConfig::default(),
    };

    let response = service.proxy_operation(request).await.unwrap();
    assert!(matches!(response.status, ProxyStatus::Accepted));
    assert!(response.final_content.contains("Existing"));
    assert!(response.final_content.contains("New"));
}

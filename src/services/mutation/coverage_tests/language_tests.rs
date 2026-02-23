#![cfg_attr(coverage_nightly, coverage(off))]

use super::*;
use std::sync::Arc;

#[test]
fn test_language_registry_creation() {
    let registry = LanguageRegistry::new();
    assert!(registry.languages().is_empty());
}

#[test]
fn test_language_registry_register_rust() {
    let mut registry = LanguageRegistry::new();
    registry.register(Arc::new(RustAdapter::new()));

    assert!(registry.languages().contains(&"rust"));
    assert!(registry.get_adapter("rust").is_some());
}

#[test]
fn test_language_registry_detect_by_extension() {
    let mut registry = LanguageRegistry::new();
    registry.register(Arc::new(RustAdapter::new()));

    let detected = registry.detect_language(std::path::Path::new("test.rs"));
    assert!(detected.is_some());
    assert_eq!(detected.unwrap().name(), "rust");
}

#[test]
fn test_language_registry_unknown_extension() {
    let registry = LanguageRegistry::new();
    let detected = registry.detect_language(std::path::Path::new("test.xyz"));
    assert!(detected.is_none());
}

#[test]
fn test_test_run_result_fields() {
    let result = TestRunResult {
        passed: false,
        failures: vec!["test_one".to_string(), "test_two".to_string()],
        execution_time_ms: 500,
        stdout: "running 2 tests".to_string(),
        stderr: "thread panicked".to_string(),
    };

    assert!(!result.passed);
    assert_eq!(result.failures.len(), 2);
    assert_eq!(result.execution_time_ms, 500);
    assert!(result.stdout.contains("running"));
    assert!(result.stderr.contains("panicked"));
}

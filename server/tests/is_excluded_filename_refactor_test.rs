//! TDD test for is_excluded_filename refactor
//! Following Toyota Way TDD: Red → Green → Refactor

use pmat::cli::analysis_utilities::is_excluded_filename;

/// Test that all test file patterns are excluded
#[test]
fn test_excludes_test_files() {
    assert!(is_excluded_filename("module_test.rs"));
    assert!(is_excluded_filename("module_tests.rs"));
    assert!(is_excluded_filename("tests.rs"));
    assert!(is_excluded_filename("test_module.rs"));
    assert!(is_excluded_filename("tests_module.rs"));
    assert!(is_excluded_filename("file_test_impl.rs"));
    assert!(is_excluded_filename("file_tests_impl.rs"));
    assert!(is_excluded_filename("test_harness.rs"));
    assert!(is_excluded_filename("test_helpers.rs"));
    assert!(is_excluded_filename("test_utils.rs"));
    assert!(is_excluded_filename("module_property_test.rs"));
    assert!(is_excluded_filename("property_tests.rs"));
}

/// Test that example and demo files are excluded
#[test]
fn test_excludes_example_demo_files() {
    assert!(is_excluded_filename("example_usage.rs"));
    assert!(is_excluded_filename("demo_app.rs"));
    assert!(is_excluded_filename("code_example.rs"));
    assert!(is_excluded_filename("app_demo.rs"));
}

/// Test that benchmark files are excluded
#[test]
fn test_excludes_benchmark_files() {
    assert!(is_excluded_filename("perf_bench.rs"));
    assert!(is_excluded_filename("perf_benchmark.rs"));
    assert!(is_excluded_filename("bench_utils.rs"));
    assert!(is_excluded_filename("benchmark_suite.rs"));
}

/// Test that mock and stub files are excluded
#[test]
fn test_excludes_mock_stub_files() {
    assert!(is_excluded_filename("mock_service.rs"));
    assert!(is_excluded_filename("stub_client.rs"));
    assert!(is_excluded_filename("stubs_collection.rs"));
    assert!(is_excluded_filename("service_mock.rs"));
    assert!(is_excluded_filename("client_stub.rs"));
    assert!(is_excluded_filename("api_stubs.rs"));
}

/// Test that regular files are NOT excluded
#[test]
fn test_does_not_exclude_regular_files() {
    assert!(!is_excluded_filename("main.rs"));
    assert!(!is_excluded_filename("lib.rs"));
    assert!(!is_excluded_filename("config.rs"));
    assert!(!is_excluded_filename("handler.rs"));
    assert!(!is_excluded_filename("service.rs"));
    assert!(!is_excluded_filename("utility.rs"));
}

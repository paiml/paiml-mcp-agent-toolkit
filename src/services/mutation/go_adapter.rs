#![cfg_attr(coverage_nightly, coverage(off))]
//! Go language adapter for mutation testing
//!
//! EXTREME TDD: GREEN PHASE - Minimal implementation to pass tests

use super::language::{LanguageAdapter, TestRunResult};
use super::operators::*;
use anyhow::Result;
use async_trait::async_trait;
use std::path::Path;

#[cfg(feature = "go-ast")]
use tree_sitter::Parser;

/// Go language adapter
pub struct GoAdapter;

impl GoAdapter {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl LanguageAdapter for GoAdapter {
    fn name(&self) -> &str {
        "go"
    }

    fn extensions(&self) -> &[&str] {
        &["go"]
    }

    #[cfg(feature = "go-ast")]
    async fn parse(&self, source: &str) -> Result<String> {
        // Create tree-sitter parser for Go
        let mut parser = Parser::new();
        parser
            .set_language(&tree_sitter_go::LANGUAGE.into())
            .map_err(|e| anyhow::anyhow!("Failed to set Go language: {}", e))?;

        // Parse the source
        let tree = parser
            .parse(source, None)
            .ok_or_else(|| anyhow::anyhow!("Parse failed"))?;

        // Check for errors
        if tree.root_node().has_error() {
            return Err(anyhow::anyhow!("Syntax error in Go source"));
        }

        Ok(source.to_string())
    }

    #[cfg(not(feature = "go-ast"))]
    async fn parse(&self, source: &str) -> Result<String> {
        Ok(source.to_string())
    }

    async fn unparse(&self, ast: &str) -> Result<String> {
        Ok(ast.to_string())
    }

    fn mutation_operators(&self) -> Vec<Box<dyn MutationOperator>> {
        vec![
            Box::new(ArithmeticOperatorReplacement),
            Box::new(RelationalOperatorReplacement),
            Box::new(ConditionalOperatorReplacement),
            Box::new(UnaryOperatorReplacement),
        ]
    }

    async fn run_tests(&self, _source_file: &Path) -> Result<TestRunResult> {
        debug_assert!(
            _source_file.exists(),
            "_source_file must exist: {}",
            _source_file.display()
        );
        // Minimal implementation for now
        Ok(TestRunResult {
            passed: true,
            failures: vec![],
            execution_time_ms: 0,
            stdout: String::new(),
            stderr: String::new(),
        })
    }
}

impl Default for GoAdapter {
    fn default() -> Self {
        Self::new()
    }
}

/// Find go.mod by traversing up from source file
pub fn find_go_mod_root(start: &Path) -> Option<&Path> {
    debug_assert!(start.exists(), "start must exist: {}", start.display());
    let mut current = start;

    loop {
        if current.join("go.mod").exists() {
            return Some(current);
        }

        current = current.parent()?;
    }
}

/// Parse test failures from go test output
pub fn parse_test_failures(stdout: &str, stderr: &str) -> Vec<String> {
    let mut failures = Vec::new();

    for line in stdout.lines().chain(stderr.lines()) {
        // Look for "--- FAIL:" lines in go test output
        if line.contains("--- FAIL:") {
            if let Some(test_name) = extract_test_name_from_go_test(line) {
                failures.push(test_name);
            }
        }
    }

    failures
}

/// Extract test name from go test failure line
fn extract_test_name_from_go_test(line: &str) -> Option<String> {
    // Pattern: "--- FAIL: TestAdd (0.00s)"
    let trimmed = line.trim();

    if trimmed.starts_with("--- FAIL:") {
        let parts: Vec<&str> = trimmed.split_whitespace().collect();
        if parts.len() >= 3 {
            return Some(parts[2].to_string());
        }
    }

    None
}

#[cfg_attr(coverage_nightly, coverage(off))]
#[cfg(test)]
mod tests {
    use super::*;

    // =========================================================================
    // GoAdapter Construction Tests
    // =========================================================================

    #[test]
    fn test_go_adapter_new() {
        let adapter = GoAdapter::new();
        let _ = adapter;
    }

    #[test]
    fn test_go_adapter_default() {
        let adapter = GoAdapter::default();
        let _ = adapter;
    }

    #[test]
    fn test_adapter_name() {
        let adapter = GoAdapter::new();
        assert_eq!(adapter.name(), "go");
    }

    #[test]
    fn test_adapter_extensions() {
        let adapter = GoAdapter::new();
        let extensions = adapter.extensions();
        assert!(extensions.contains(&"go"));
        assert_eq!(extensions.len(), 1);
    }

    #[test]
    fn test_mutation_operators() {
        let adapter = GoAdapter::new();
        let operators = adapter.mutation_operators();
        assert_eq!(operators.len(), 4);
    }

    // =========================================================================
    // find_go_mod_root Tests
    // =========================================================================

    #[test]
    fn test_find_go_mod_root_nonexistent() {
        let path = Path::new("/nonexistent/path/to/file.go");
        let result = find_go_mod_root(path);
        assert!(result.is_none());
    }

    // =========================================================================
    // parse_test_failures Tests
    // =========================================================================

    #[test]
    fn test_parse_test_failures_empty() {
        let failures = parse_test_failures("", "");
        assert!(failures.is_empty());
    }

    #[test]
    fn test_parse_test_failures_no_failures() {
        let stdout = "--- PASS: TestAdd (0.00s)";
        let failures = parse_test_failures(stdout, "");
        assert!(failures.is_empty());
    }

    #[test]
    fn test_parse_test_failures_single_failure() {
        let stdout = "--- FAIL: TestSubtract (0.00s)";
        let failures = parse_test_failures(stdout, "");
        assert_eq!(failures.len(), 1);
        assert_eq!(failures[0], "TestSubtract");
    }

    #[test]
    fn test_parse_test_failures_multiple_failures() {
        let stdout = "--- PASS: TestAdd (0.00s)\n--- FAIL: TestSubtract (0.00s)\n--- FAIL: TestMultiply (0.00s)";
        let failures = parse_test_failures(stdout, "");
        assert_eq!(failures.len(), 2);
    }

    #[test]
    fn test_parse_test_failures_from_stderr() {
        let stderr = "--- FAIL: TestFail (0.00s)";
        let failures = parse_test_failures("", stderr);
        assert_eq!(failures.len(), 1);
    }

    #[test]
    fn test_parse_test_failures_mixed_stdout_stderr() {
        let stdout = "--- FAIL: TestFail1 (0.00s)";
        let stderr = "--- FAIL: TestFail2 (0.00s)";
        let failures = parse_test_failures(stdout, stderr);
        assert_eq!(failures.len(), 2);
    }

    // =========================================================================
    // extract_test_name_from_go_test Tests
    // =========================================================================

    #[test]
    fn test_extract_test_name_basic() {
        let line = "--- FAIL: TestSubtract (0.00s)";
        let result = extract_test_name_from_go_test(line);
        assert!(result.is_some());
        assert_eq!(result.unwrap(), "TestSubtract");
    }

    #[test]
    fn test_extract_test_name_no_match() {
        let line = "--- PASS: TestAdd (0.00s)";
        let result = extract_test_name_from_go_test(line);
        assert!(result.is_none());
    }

    #[test]
    fn test_extract_test_name_empty_line() {
        let line = "";
        let result = extract_test_name_from_go_test(line);
        assert!(result.is_none());
    }

    #[test]
    fn test_extract_test_name_fail_only() {
        let line = "--- FAIL:";
        let result = extract_test_name_from_go_test(line);
        assert!(result.is_none());
    }

    // =========================================================================
    // Async Tests
    // =========================================================================

    #[tokio::test]
    async fn test_parse_simple_source() {
        let adapter = GoAdapter::new();
        let source = "package main\nfunc main() {}";
        let result = adapter.parse(source).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_parse_complex_source() {
        let adapter = GoAdapter::new();
        let source = r#"
            package main

            import "fmt"

            func add(a, b int) int {
                return a + b
            }

            func main() {
                fmt.Println(add(1, 2))
            }
        "#;
        let result = adapter.parse(source).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_unparse() {
        let adapter = GoAdapter::new();
        let ast = "package main";
        let result = adapter.unparse(ast).await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), ast);
    }

    #[tokio::test]
    async fn test_run_tests() {
        let adapter = GoAdapter::new();
        let path = Path::new("/test/file.go");
        let result = adapter.run_tests(path).await;
        assert!(result.is_ok());
        let test_result = result.unwrap();
        assert!(test_result.passed);
    }

    // =========================================================================
    // LanguageAdapter Trait Tests
    // =========================================================================

    #[test]
    fn test_implements_language_adapter() {
        fn _assert_adapter<T: LanguageAdapter>() {}
        _assert_adapter::<GoAdapter>();
    }

    // =========================================================================
    // Edge Cases
    // =========================================================================

    #[test]
    fn test_extensions_are_complete() {
        let adapter = GoAdapter::new();
        let extensions = adapter.extensions();

        assert!(extensions.iter().any(|e| *e == "go"), "Missing .go");
    }

    #[tokio::test]
    async fn test_run_tests_returns_test_run_result() {
        let adapter = GoAdapter::new();
        let path = Path::new("/test/file.go");
        let result = adapter.run_tests(path).await.unwrap();

        assert!(result.passed);
        assert!(result.failures.is_empty());
        assert_eq!(result.execution_time_ms, 0);
        assert!(result.stdout.is_empty());
        assert!(result.stderr.is_empty());
    }
}

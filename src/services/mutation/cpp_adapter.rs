#![cfg_attr(coverage_nightly, coverage(off))]
//! C/C++ language adapter for mutation testing
//!
//! EXTREME TDD: GREEN PHASE - Minimal implementation to pass tests

use super::language::{LanguageAdapter, TestRunResult};
use super::operators::*;
use anyhow::Result;
use async_trait::async_trait;
use std::path::Path;

#[cfg(feature = "cpp-ast")]
use tree_sitter::Parser;

/// C/C++ language adapter
pub struct CppAdapter;

impl CppAdapter {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl LanguageAdapter for CppAdapter {
    fn name(&self) -> &str {
        "cpp"
    }

    fn extensions(&self) -> &[&str] {
        &["c", "cpp", "cc", "cxx", "h", "hpp"]
    }

    #[cfg(feature = "cpp-ast")]
    async fn parse(&self, source: &str) -> Result<String> {
        // Create tree-sitter parser for C++
        let mut parser = Parser::new();
        parser
            .set_language(&tree_sitter_cpp::LANGUAGE.into())
            .map_err(|e| anyhow::anyhow!("Failed to set C++ language: {}", e))?;

        // Parse the source
        let tree = parser
            .parse(source, None)
            .ok_or_else(|| anyhow::anyhow!("Parse failed"))?;

        // Check for errors
        if tree.root_node().has_error() {
            return Err(anyhow::anyhow!("Syntax error in C/C++ source"));
        }

        Ok(source.to_string())
    }

    #[cfg(not(feature = "cpp-ast"))]
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

impl Default for CppAdapter {
    fn default() -> Self {
        Self::new()
    }
}

/// Find CMakeLists.txt by traversing up from source file
pub fn find_cmake_root(start: &Path) -> Option<&Path> {
    debug_assert!(start.exists(), "start must exist: {}", start.display());
    let mut current = start;

    loop {
        if current.join("CMakeLists.txt").exists() {
            return Some(current);
        }

        current = current.parent()?;
    }
}

/// Parse test failures from ctest output
pub fn parse_test_failures(stdout: &str, stderr: &str) -> Vec<String> {
    let mut failures = Vec::new();

    for line in stdout.lines().chain(stderr.lines()) {
        // Look for "***Failed" lines in ctest output
        if line.contains("***Failed") {
            if let Some(test_name) = extract_test_name_from_ctest(line) {
                failures.push(test_name);
            }
        }
    }

    failures
}

/// Extract test name from ctest failure line
fn extract_test_name_from_ctest(line: &str) -> Option<String> {
    // Pattern: "2/3 Test #2: TestSubtract .....................***Failed"
    let trimmed = line.trim();

    // Look for pattern "Test #N: TestName"
    if let Some(test_part) = trimmed.split("Test #").nth(1) {
        if let Some(name_part) = test_part.split(':').nth(1) {
            let parts: Vec<&str> = name_part.split_whitespace().collect();
            if !parts.is_empty() {
                return Some(parts[0].to_string());
            }
        }
    }

    None
}

#[cfg_attr(coverage_nightly, coverage(off))]
#[cfg(test)]
mod tests {
    use super::*;

    // =========================================================================
    // CppAdapter Construction Tests
    // =========================================================================

    #[test]
    fn test_cpp_adapter_new() {
        let adapter = CppAdapter::new();
        let _ = adapter;
    }

    #[test]
    fn test_cpp_adapter_default() {
        let adapter = CppAdapter::default();
        let _ = adapter;
    }

    #[test]
    fn test_adapter_name() {
        let adapter = CppAdapter::new();
        assert_eq!(adapter.name(), "cpp");
    }

    #[test]
    fn test_adapter_extensions() {
        let adapter = CppAdapter::new();
        let extensions = adapter.extensions();
        assert!(extensions.contains(&"c"));
        assert!(extensions.contains(&"cpp"));
        assert!(extensions.contains(&"cc"));
        assert!(extensions.contains(&"cxx"));
        assert!(extensions.contains(&"h"));
        assert!(extensions.contains(&"hpp"));
        assert_eq!(extensions.len(), 6);
    }

    #[test]
    fn test_mutation_operators() {
        let adapter = CppAdapter::new();
        let operators = adapter.mutation_operators();
        assert_eq!(operators.len(), 4);
    }

    // =========================================================================
    // find_cmake_root Tests
    // =========================================================================

    #[test]
    fn test_find_cmake_root_nonexistent() {
        let path = Path::new("/nonexistent/path/to/file.cpp");
        let result = find_cmake_root(path);
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
        let stdout = "1/3 Test #1: TestAdd ..........................   Passed";
        let failures = parse_test_failures(stdout, "");
        assert!(failures.is_empty());
    }

    #[test]
    fn test_parse_test_failures_single_failure() {
        let stdout = "2/3 Test #2: TestSubtract .....................***Failed";
        let failures = parse_test_failures(stdout, "");
        assert_eq!(failures.len(), 1);
        assert_eq!(failures[0], "TestSubtract");
    }

    #[test]
    fn test_parse_test_failures_multiple_failures() {
        let stdout = "1/3 Test #1: TestAdd ..........................   Passed\n\
                      2/3 Test #2: TestSubtract .....................***Failed\n\
                      3/3 Test #3: TestMultiply .....................***Failed";
        let failures = parse_test_failures(stdout, "");
        assert_eq!(failures.len(), 2);
    }

    #[test]
    fn test_parse_test_failures_from_stderr() {
        let stderr = "2/3 Test #2: TestFail .....................***Failed";
        let failures = parse_test_failures("", stderr);
        assert_eq!(failures.len(), 1);
    }

    #[test]
    fn test_parse_test_failures_mixed_stdout_stderr() {
        let stdout = "2/3 Test #2: TestFail1 .....................***Failed";
        let stderr = "3/3 Test #3: TestFail2 .....................***Failed";
        let failures = parse_test_failures(stdout, stderr);
        assert_eq!(failures.len(), 2);
    }

    // =========================================================================
    // extract_test_name_from_ctest Tests
    // =========================================================================

    #[test]
    fn test_extract_test_name_basic() {
        let line = "2/3 Test #2: TestSubtract .....................***Failed";
        let result = extract_test_name_from_ctest(line);
        assert!(result.is_some());
        assert_eq!(result.unwrap(), "TestSubtract");
    }

    #[test]
    fn test_extract_test_name_no_match() {
        let line = "Passed test";
        let result = extract_test_name_from_ctest(line);
        assert!(result.is_none());
    }

    #[test]
    fn test_extract_test_name_empty_line() {
        let line = "";
        let result = extract_test_name_from_ctest(line);
        assert!(result.is_none());
    }

    // =========================================================================
    // Async Tests
    // =========================================================================

    #[tokio::test]
    async fn test_parse_simple_source() {
        let adapter = CppAdapter::new();
        let source = "int main() { return 0; }";
        let result = adapter.parse(source).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_parse_complex_source() {
        let adapter = CppAdapter::new();
        let source = r#"
            #include <iostream>
            int add(int a, int b) { return a + b; }
            int main() {
                std::cout << add(1, 2);
                return 0;
            }
        "#;
        let result = adapter.parse(source).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_unparse() {
        let adapter = CppAdapter::new();
        let ast = "int main() {}";
        let result = adapter.unparse(ast).await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), ast);
    }

    #[tokio::test]
    async fn test_run_tests() {
        let adapter = CppAdapter::new();
        let path = Path::new("/test/file.cpp");
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
        _assert_adapter::<CppAdapter>();
    }

    // =========================================================================
    // Edge Cases
    // =========================================================================

    #[test]
    fn test_extensions_are_complete() {
        let adapter = CppAdapter::new();
        let extensions = adapter.extensions();

        assert!(extensions.iter().any(|e| *e == "c"), "Missing .c");
        assert!(extensions.iter().any(|e| *e == "cpp"), "Missing .cpp");
        assert!(extensions.iter().any(|e| *e == "h"), "Missing .h");
        assert!(extensions.iter().any(|e| *e == "hpp"), "Missing .hpp");
    }

    #[tokio::test]
    async fn test_run_tests_returns_test_run_result() {
        let adapter = CppAdapter::new();
        let path = Path::new("/test/file.cpp");
        let result = adapter.run_tests(path).await.unwrap();

        assert!(result.passed);
        assert!(result.failures.is_empty());
        assert_eq!(result.execution_time_ms, 0);
        assert!(result.stdout.is_empty());
        assert!(result.stderr.is_empty());
    }
}

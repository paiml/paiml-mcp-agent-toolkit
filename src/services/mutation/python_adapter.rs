//! Python language adapter for mutation testing
//!
//! EXTREME TDD: GREEN PHASE - Minimal implementation to pass tests

use super::language::{LanguageAdapter, TestRunResult};
use super::operators::*;
use anyhow::Result;
use async_trait::async_trait;
use std::path::Path;

// Modern tree-sitter-python parsing (replaces rustpython-parser)
#[cfg(feature = "python-ast")]
use tree_sitter::Parser as TsParser;

/// Python language adapter
pub struct PythonAdapter;

impl PythonAdapter {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl LanguageAdapter for PythonAdapter {
    fn name(&self) -> &str {
        "python"
    }

    fn extensions(&self) -> &[&str] {
        &["py"]
    }

    #[cfg(feature = "python-ast")]
    async fn parse(&self, source: &str) -> Result<String> {
        // Parse using tree-sitter-python to validate syntax
        let mut parser = TsParser::new();
        parser
            .set_language(&tree_sitter_python::LANGUAGE.into())
            .map_err(|e| anyhow::anyhow!("Failed to set Python language: {e}"))?;

        let tree = parser
            .parse(source, None)
            .ok_or_else(|| anyhow::anyhow!("Failed to parse Python code"))?;

        // Check for syntax errors
        let root = tree.root_node();
        if Self::has_syntax_errors(&root) {
            return Err(anyhow::anyhow!("Parse failed: syntax errors detected"));
        }

        Ok(source.to_string())
    }

    #[cfg(not(feature = "python-ast"))]
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

#[cfg(feature = "python-ast")]
impl PythonAdapter {
    /// Check if tree-sitter parse tree has syntax errors
    fn has_syntax_errors(node: &tree_sitter::Node) -> bool {
        if node.kind() == "ERROR" || node.is_error() || node.is_missing() {
            return true;
        }

        for child in node.children(&mut node.walk()) {
            if Self::has_syntax_errors(&child) {
                return true;
            }
        }

        false
    }
}

impl Default for PythonAdapter {
    fn default() -> Self {
        Self::new()
    }
}

/// Find pytest root by traversing up from source file
pub fn find_pytest_root(start: &Path) -> Option<&Path> {
    let mut current = start;

    loop {
        // Look for pytest.ini, pyproject.toml, or setup.py
        if current.join("pytest.ini").exists()
            || current.join("pyproject.toml").exists()
            || current.join("setup.py").exists()
        {
            return Some(current);
        }

        current = current.parent()?;
    }
}

/// Parse test failures from pytest output
pub fn parse_test_failures(stdout: &str, stderr: &str) -> Vec<String> {
    let mut failures = Vec::new();

    for line in stdout.lines().chain(stderr.lines()) {
        // Look for "FAILED" lines in pytest output
        if line.contains("FAILED") {
            if let Some(test_name) = extract_test_name_from_pytest(line) {
                failures.push(test_name);
            }
        }
    }

    failures
}

/// Extract test name from pytest failure line
fn extract_test_name_from_pytest(line: &str) -> Option<String> {
    // Pattern: "FAILED tests/test_math.py::test_subtract"
    let trimmed = line.trim();

    if trimmed.starts_with("FAILED") {
        let parts: Vec<&str> = trimmed.split_whitespace().collect();
        if parts.len() >= 2 {
            return Some(parts[1].to_string());
        }
    }

    None
}

#[cfg(test)]
mod tests {
    use super::*;

    // =========================================================================
    // PythonAdapter Construction Tests
    // =========================================================================

    #[test]
    fn test_python_adapter_new() {
        let adapter = PythonAdapter::new();
        let _ = adapter;
    }

    #[test]
    fn test_python_adapter_default() {
        let adapter = PythonAdapter::default();
        let _ = adapter;
    }

    #[test]
    fn test_adapter_name() {
        let adapter = PythonAdapter::new();
        assert_eq!(adapter.name(), "python");
    }

    #[test]
    fn test_adapter_extensions() {
        let adapter = PythonAdapter::new();
        let extensions = adapter.extensions();
        assert!(extensions.contains(&"py"));
        assert_eq!(extensions.len(), 1);
    }

    #[test]
    fn test_mutation_operators() {
        let adapter = PythonAdapter::new();
        let operators = adapter.mutation_operators();
        assert_eq!(operators.len(), 4);
    }

    // =========================================================================
    // find_pytest_root Tests
    // =========================================================================

    #[test]
    fn test_find_pytest_root_nonexistent() {
        let path = Path::new("/nonexistent/path/to/file.py");
        let result = find_pytest_root(path);
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
        let stdout = "PASSED test_math.py::test_add\nPASSED test_math.py::test_sub";
        let failures = parse_test_failures(stdout, "");
        assert!(failures.is_empty());
    }

    #[test]
    fn test_parse_test_failures_single_failure() {
        let stdout = "FAILED tests/test_math.py::test_subtract";
        let failures = parse_test_failures(stdout, "");
        assert_eq!(failures.len(), 1);
        assert_eq!(failures[0], "tests/test_math.py::test_subtract");
    }

    #[test]
    fn test_parse_test_failures_multiple_failures() {
        let stdout = "FAILED test1\nPASSED test2\nFAILED test3\nFAILED test4";
        let failures = parse_test_failures(stdout, "");
        assert_eq!(failures.len(), 3);
    }

    #[test]
    fn test_parse_test_failures_from_stderr() {
        let stderr = "FAILED tests/test_broken.py::test_bug";
        let failures = parse_test_failures("", stderr);
        assert_eq!(failures.len(), 1);
    }

    #[test]
    fn test_parse_test_failures_mixed_stdout_stderr() {
        let stdout = "FAILED test1";
        let stderr = "FAILED test2";
        let failures = parse_test_failures(stdout, stderr);
        assert_eq!(failures.len(), 2);
    }

    // =========================================================================
    // extract_test_name_from_pytest Tests
    // =========================================================================

    #[test]
    fn test_extract_test_name_basic() {
        let line = "FAILED tests/test_math.py::test_subtract";
        let result = extract_test_name_from_pytest(line);
        assert!(result.is_some());
        assert_eq!(result.unwrap(), "tests/test_math.py::test_subtract");
    }

    #[test]
    fn test_extract_test_name_no_failed() {
        let line = "PASSED tests/test_math.py::test_add";
        let result = extract_test_name_from_pytest(line);
        assert!(result.is_none());
    }

    #[test]
    fn test_extract_test_name_empty_line() {
        let line = "";
        let result = extract_test_name_from_pytest(line);
        assert!(result.is_none());
    }

    #[test]
    fn test_extract_test_name_failed_only() {
        let line = "FAILED";
        let result = extract_test_name_from_pytest(line);
        assert!(result.is_none());
    }

    #[test]
    fn test_extract_test_name_with_whitespace() {
        let line = "  FAILED tests/test.py::test_case  ";
        let result = extract_test_name_from_pytest(line);
        assert!(result.is_some());
        assert_eq!(result.unwrap(), "tests/test.py::test_case");
    }

    #[test]
    fn test_extract_test_name_with_extra_info() {
        let line = "FAILED tests/test.py::test_case - AssertionError";
        let result = extract_test_name_from_pytest(line);
        assert!(result.is_some());
        assert_eq!(result.unwrap(), "tests/test.py::test_case");
    }

    // =========================================================================
    // Async Tests
    // =========================================================================

    #[tokio::test]
    async fn test_parse_simple_source() {
        let adapter = PythonAdapter::new();
        let source = "x = 1";
        let result = adapter.parse(source).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_parse_complex_source() {
        let adapter = PythonAdapter::new();
        let source = r#"
def add(a, b):
    return a + b

class Calculator:
    def multiply(self, x, y):
        return x * y
        "#;
        let result = adapter.parse(source).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_unparse() {
        let adapter = PythonAdapter::new();
        let ast = "x = 1";
        let result = adapter.unparse(ast).await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), ast);
    }

    #[tokio::test]
    async fn test_run_tests() {
        let adapter = PythonAdapter::new();
        let path = Path::new("/test/file.py");
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
        _assert_adapter::<PythonAdapter>();
    }

    // =========================================================================
    // Edge Cases
    // =========================================================================

    #[test]
    fn test_parse_test_failures_with_mixed_content() {
        let stdout = r#"
            ===== test session starts =====
            FAILED tests/test_math.py::test_subtract
            PASSED tests/test_math.py::test_add
            FAILED tests/test_string.py::test_concat
            ===== 2 failed, 1 passed =====
        "#;
        let failures = parse_test_failures(stdout, "");
        assert_eq!(failures.len(), 2);
    }

    #[test]
    fn test_parse_test_failures_parametrized() {
        let stdout = "FAILED tests/test_math.py::test_add[1-2-3]";
        let failures = parse_test_failures(stdout, "");
        assert_eq!(failures.len(), 1);
        assert_eq!(failures[0], "tests/test_math.py::test_add[1-2-3]");
    }

    #[test]
    fn test_extensions_are_complete() {
        let adapter = PythonAdapter::new();
        let extensions = adapter.extensions();

        assert!(extensions.iter().any(|e| *e == "py"), "Missing .py");
    }

    #[tokio::test]
    async fn test_run_tests_returns_test_run_result() {
        let adapter = PythonAdapter::new();
        let path = Path::new("/test/file.py");
        let result = adapter.run_tests(path).await.unwrap();

        assert!(result.passed);
        assert!(result.failures.is_empty());
        assert_eq!(result.execution_time_ms, 0);
        assert!(result.stdout.is_empty());
        assert!(result.stderr.is_empty());
    }
}

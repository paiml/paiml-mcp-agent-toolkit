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

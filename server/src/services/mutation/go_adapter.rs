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
        let language = tree_sitter_go::language();
        parser.set_language(&language)
            .map_err(|e| anyhow::anyhow!("Failed to set Go language: {}", e))?;

        // Parse the source
        let tree = parser.parse(source, None)
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

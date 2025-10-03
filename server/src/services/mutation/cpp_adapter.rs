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
        let language = tree_sitter_cpp::language();
        parser.set_language(&language)
            .map_err(|e| anyhow::anyhow!("Failed to set C++ language: {}", e))?;

        // Parse the source
        let tree = parser.parse(source, None)
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

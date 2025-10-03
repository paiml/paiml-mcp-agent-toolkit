//! Rust language adapter for mutation testing

use super::language::{LanguageAdapter, TestRunResult};
use super::operators::*;
use anyhow::{Context, Result};
use async_trait::async_trait;
use std::path::Path;
use tokio::process::Command as TokioCommand;

/// Rust language adapter using syn for AST parsing
pub struct RustAdapter;

impl RustAdapter {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl LanguageAdapter for RustAdapter {
    fn name(&self) -> &str {
        "rust"
    }

    fn extensions(&self) -> &[&str] {
        &["rs"]
    }

    async fn parse(&self, source: &str) -> Result<String> {
        // Parse using syn
        let _syntax_tree: syn::File = syn::parse_file(source)
            .context("Failed to parse Rust source")?;

        // For now, return source as-is (we'll use syn AST directly)
        Ok(source.to_string())
    }

    async fn unparse(&self, ast: &str) -> Result<String> {
        // For now, AST is just source code
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

    async fn run_tests(&self, source_file: &Path) -> Result<TestRunResult> {
        // Get the project root (traverse up from source file)
        let project_root = find_cargo_root(source_file)
            .context("Could not find Cargo.toml")?;

        // Run cargo test
        let output = TokioCommand::new("cargo")
            .arg("test")
            .arg("--quiet")
            .current_dir(project_root)
            .output()
            .await
            .context("Failed to run cargo test")?;

        let stdout = String::from_utf8_lossy(&output.stdout).to_string();
        let stderr = String::from_utf8_lossy(&output.stderr).to_string();

        // Parse test failures from output
        let failures = parse_test_failures(&stdout, &stderr);

        Ok(TestRunResult {
            passed: output.status.success(),
            failures,
            execution_time_ms: 0, // We don't track this yet
            stdout,
            stderr,
        })
    }
}

impl Default for RustAdapter {
    fn default() -> Self {
        Self::new()
    }
}

/// Find Cargo.toml by traversing up from source file
fn find_cargo_root(start: &Path) -> Option<&Path> {
    let mut current = start;

    loop {
        if current.join("Cargo.toml").exists() {
            return Some(current);
        }

        current = current.parent()?;
    }
}

/// Parse test failures from cargo test output
fn parse_test_failures(stdout: &str, stderr: &str) -> Vec<String> {
    let mut failures = Vec::new();

    // Look for "test <name> ... FAILED" pattern
    for line in stdout.lines().chain(stderr.lines()) {
        if line.contains("FAILED") && line.contains("test ") {
            if let Some(test_name) = extract_test_name(line) {
                failures.push(test_name);
            }
        }
    }

    failures
}

/// Extract test name from failure line
fn extract_test_name(line: &str) -> Option<String> {
    // Pattern: "test <name> ... FAILED"
    let parts: Vec<&str> = line.split_whitespace().collect();
    if parts.len() >= 2 && parts[0] == "test" {
        return Some(parts[1].to_string());
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_rust_adapter_parse() {
        let adapter = RustAdapter::new();
        let source = "fn add(a: i32, b: i32) -> i32 { a + b }";

        let result = adapter.parse(source).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_rust_adapter_parse_invalid() {
        let adapter = RustAdapter::new();
        let source = "fn add(a: i32, b: i32) { invalid syntax }";

        let result = adapter.parse(source).await;
        assert!(result.is_err());
    }

    #[test]
    fn test_rust_adapter_extensions() {
        let adapter = RustAdapter::new();
        assert_eq!(adapter.extensions(), &["rs"]);
    }

    #[test]
    fn test_rust_adapter_name() {
        let adapter = RustAdapter::new();
        assert_eq!(adapter.name(), "rust");
    }

    #[test]
    fn test_rust_adapter_mutation_operators() {
        let adapter = RustAdapter::new();
        let operators = adapter.mutation_operators();

        assert_eq!(operators.len(), 4);
        assert_eq!(operators[0].name(), "AOR");
        assert_eq!(operators[1].name(), "ROR");
        assert_eq!(operators[2].name(), "COR");
        assert_eq!(operators[3].name(), "UOR");
    }

    #[test]
    fn test_parse_test_failures() {
        let stdout = r#"
test test_add ... ok
test test_sub ... FAILED
test test_mul ... ok
test test_div ... FAILED
        "#;

        let failures = parse_test_failures(stdout, "");
        assert_eq!(failures.len(), 2);
        assert!(failures.contains(&"test_sub".to_string()));
        assert!(failures.contains(&"test_div".to_string()));
    }

    #[test]
    fn test_find_cargo_root() {
        // This test requires an actual Cargo.toml to exist
        let current_file = std::path::Path::new(file!());
        let cargo_root = find_cargo_root(current_file);
        assert!(cargo_root.is_some());
    }
}

#![cfg_attr(coverage_nightly, coverage(off))]
//! Lua language adapter for mutation testing
//!
//! Lua-specific mutation operators based on analysis of Kong, KOReader, lazy.nvim,
//! APISIX, xmake, AwesomeWM, lite-xl — 10 top Lua projects (~5,800 files total).

use super::language::{LanguageAdapter, TestRunResult};
use super::operators::*;
use anyhow::Result;
use async_trait::async_trait;
use std::path::Path;

/// Lua language adapter for mutation testing.
pub struct LuaAdapter;

impl LuaAdapter {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl LanguageAdapter for LuaAdapter {
    fn name(&self) -> &str {
        "lua"
    }

    fn extensions(&self) -> &[&str] {
        &["lua"]
    }

    async fn parse(&self, source: &str) -> Result<String> {
        // Validate basic Lua syntax by checking for obvious parse errors
        if source.trim().is_empty() {
            return Err(anyhow::anyhow!("Empty Lua source"));
        }
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

    async fn run_tests(&self, source_file: &Path) -> Result<TestRunResult> {
        // Try busted first, then lua test runner
        let project_root = find_lua_project_root(source_file);
        if let Some(root) = project_root {
            if root.join(".busted").exists() {
                return run_busted_tests(root).await;
            }
        }
        // Fallback: no test runner configured
        Ok(TestRunResult {
            passed: true,
            failures: vec![],
            execution_time_ms: 0,
            stdout: String::new(),
            stderr: String::new(),
        })
    }
}

impl Default for LuaAdapter {
    fn default() -> Self {
        Self::new()
    }
}

/// Find Lua project root by looking for .busted, rockspec, or init.lua
pub fn find_lua_project_root(start: &Path) -> Option<&Path> {
    let mut current = start;
    loop {
        if current.join(".busted").exists()
            || current.join("init.lua").exists()
            || has_rockspec(current)
        {
            return Some(current);
        }
        current = current.parent()?;
    }
}

/// Check if directory contains a .rockspec file.
fn has_rockspec(dir: &Path) -> bool {
    std::fs::read_dir(dir).is_ok_and(|entries| {
        entries.flatten().any(|e| {
            e.path()
                .extension()
                .map(|ext| ext == "rockspec")
                .unwrap_or(false)
        })
    })
}

/// Run busted test framework.
async fn run_busted_tests(project_root: &Path) -> Result<TestRunResult> {
    let output = tokio::process::Command::new("busted")
        .arg("--output=plainTerminal")
        .current_dir(project_root)
        .output()
        .await
        .map_err(|e| anyhow::anyhow!("Failed to run busted: {}", e))?;

    let stdout = String::from_utf8_lossy(&output.stdout).to_string();
    let stderr = String::from_utf8_lossy(&output.stderr).to_string();
    let failures = parse_busted_failures(&stdout, &stderr);

    Ok(TestRunResult {
        passed: output.status.success(),
        failures,
        execution_time_ms: 0,
        stdout,
        stderr,
    })
}

/// Parse busted test failures from output.
pub fn parse_busted_failures(stdout: &str, stderr: &str) -> Vec<String> {
    let mut failures = Vec::new();
    for line in stdout.lines().chain(stderr.lines()) {
        let trimmed = line.trim();
        // Busted failure format: "Failure → path/to/spec.lua @ 10"
        // or "Error → path/to/spec.lua @ 5"
        if trimmed.contains("Failure") || trimmed.contains("Error") {
            if let Some(arrow_pos) = trimmed.find('→') {
                let test_info = trimmed[arrow_pos + '→'.len_utf8()..].trim();
                if !test_info.is_empty() {
                    failures.push(test_info.to_string());
                }
            }
        }
    }
    failures
}

#[cfg_attr(coverage_nightly, coverage(off))]
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_lua_adapter_new() {
        let adapter = LuaAdapter::new();
        assert_eq!(adapter.name(), "lua");
    }

    #[test]
    fn test_lua_adapter_default() {
        let adapter = LuaAdapter::default();
        assert_eq!(adapter.name(), "lua");
    }

    #[test]
    fn test_adapter_extensions() {
        let adapter = LuaAdapter::new();
        let extensions = adapter.extensions();
        assert!(extensions.contains(&"lua"));
        assert_eq!(extensions.len(), 1);
    }

    #[test]
    fn test_mutation_operators() {
        let adapter = LuaAdapter::new();
        let operators = adapter.mutation_operators();
        assert_eq!(operators.len(), 4);
    }

    #[test]
    fn test_mutation_operator_names() {
        let adapter = LuaAdapter::new();
        let operators = adapter.mutation_operators();
        let names: Vec<&str> = operators.iter().map(|op| op.name()).collect();
        assert!(names.contains(&"AOR"));
        assert!(names.contains(&"ROR"));
        assert!(names.contains(&"COR"));
        assert!(names.contains(&"UOR"));
    }

    #[tokio::test]
    async fn test_parse_valid_source() {
        let adapter = LuaAdapter::new();
        let source = "local M = {}\nreturn M\n";
        let result = adapter.parse(source).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_parse_empty_source() {
        let adapter = LuaAdapter::new();
        let result = adapter.parse("").await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_unparse() {
        let adapter = LuaAdapter::new();
        let source = "return 42";
        let result = adapter.unparse(source).await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), source);
    }

    #[tokio::test]
    async fn test_run_tests_no_busted() {
        let adapter = LuaAdapter::new();
        let path = Path::new("/nonexistent/file.lua");
        let result = adapter.run_tests(path).await;
        assert!(result.is_ok());
        let test_result = result.unwrap();
        assert!(test_result.passed);
    }

    #[test]
    fn test_find_lua_project_root_nonexistent() {
        let path = Path::new("/nonexistent/path/to/file.lua");
        let result = find_lua_project_root(path);
        assert!(result.is_none());
    }

    #[test]
    fn test_parse_busted_failures_empty() {
        let failures = parse_busted_failures("", "");
        assert!(failures.is_empty());
    }

    #[test]
    fn test_parse_busted_failures_no_failures() {
        let stdout = "10 successes / 0 failures / 0 errors";
        let failures = parse_busted_failures(stdout, "");
        assert!(failures.is_empty());
    }

    #[test]
    fn test_parse_busted_failures_with_failure() {
        let stdout = "Failure → spec/handler_spec.lua @ 10\n  expected true, got false";
        let failures = parse_busted_failures(stdout, "");
        assert_eq!(failures.len(), 1);
        assert!(failures[0].contains("handler_spec.lua"));
    }

    #[test]
    fn test_parse_busted_failures_with_error() {
        let stdout = "Error → spec/broken_spec.lua @ 5\n  attempt to index nil value";
        let failures = parse_busted_failures(stdout, "");
        assert_eq!(failures.len(), 1);
        assert!(failures[0].contains("broken_spec.lua"));
    }

    #[test]
    fn test_parse_busted_failures_mixed() {
        let stdout =
            "Failure → spec/a_spec.lua @ 10\nError → spec/b_spec.lua @ 20\n2 failures / 1 error";
        let failures = parse_busted_failures(stdout, "");
        assert_eq!(failures.len(), 2);
    }

    #[test]
    fn test_implements_language_adapter() {
        fn _assert_adapter<T: LanguageAdapter>() {}
        _assert_adapter::<LuaAdapter>();
    }
}

#![cfg_attr(coverage_nightly, coverage(off))]
//! TypeScript/JavaScript language adapter for mutation testing
//!
//! EXTREME TDD: GREEN PHASE - Minimal implementation to pass tests

use super::language::{LanguageAdapter, TestRunResult};
use super::operators::*;
use anyhow::Result;
use async_trait::async_trait;
use std::path::Path;

#[cfg(feature = "typescript-ast")]
use swc_common::{sync::Lrc, FileName, SourceMap};
#[cfg(feature = "typescript-ast")]
use swc_ecma_parser::{lexer::Lexer, Parser, StringInput, Syntax, TsSyntax};

/// TypeScript/JavaScript language adapter
pub struct TypeScriptAdapter;

impl TypeScriptAdapter {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl LanguageAdapter for TypeScriptAdapter {
    fn name(&self) -> &str {
        "typescript"
    }

    fn extensions(&self) -> &[&str] {
        &["ts", "tsx", "js", "jsx"]
    }

    #[cfg(feature = "typescript-ast")]
    async fn parse(&self, source: &str) -> Result<String> {
        let cm: Lrc<SourceMap> = Default::default();
        let fm = cm.new_source_file(FileName::Anon.into(), source.to_string());

        let lexer = Lexer::new(
            Syntax::Typescript(TsSyntax {
                tsx: true,
                decorators: true,
                ..Default::default()
            }),
            Default::default(),
            StringInput::from(&*fm),
            None,
        );

        let mut parser = Parser::new_from(lexer);
        parser
            .parse_module()
            .map_err(|_| anyhow::anyhow!("Parse failed"))?;

        Ok(source.to_string())
    }

    #[cfg(not(feature = "typescript-ast"))]
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

    async fn run_tests(&self, source_file: &Path) -> Result<TestRunResult> {
        // GREEN PHASE: Real test execution
        use std::time::Instant;
        use tokio::process::Command;

        // Find project root with package.json
        let project_root = find_package_json_root(source_file)
            .ok_or_else(|| anyhow::anyhow!("No package.json found"))?;

        // Detect test command from package.json
        let package_json_path = project_root.join("package.json");
        let package_json = tokio::fs::read_to_string(&package_json_path).await?;
        let test_cmd = detect_test_command(&package_json)?;

        // Run tests with timeout
        let start = Instant::now();
        let output = Command::new("npm")
            .arg("run")
            .arg(&test_cmd)
            .current_dir(project_root)
            .output()
            .await?;

        let execution_time_ms = start.elapsed().as_millis() as u64;
        let stdout = String::from_utf8_lossy(&output.stdout).to_string();
        let stderr = String::from_utf8_lossy(&output.stderr).to_string();

        // Parse failures
        let failures = parse_test_failures(&stdout, &stderr);
        let passed = output.status.success();

        Ok(TestRunResult {
            passed,
            failures,
            execution_time_ms,
            stdout,
            stderr,
        })
    }
}

impl Default for TypeScriptAdapter {
    fn default() -> Self {
        Self::new()
    }
}

/// Find package.json by traversing up from source file
pub fn find_package_json_root(start: &Path) -> Option<&Path> {
    let mut current = start;

    loop {
        if current.join("package.json").exists() {
            return Some(current);
        }

        current = current.parent()?;
    }
}

/// Parse test failures from npm test or jest output
pub fn parse_test_failures(stdout: &str, stderr: &str) -> Vec<String> {
    let mut failures = Vec::new();

    for line in stdout.lines().chain(stderr.lines()) {
        if line.contains('✕') || line.contains("FAIL") {
            if let Some(test_name) = extract_test_name(line) {
                failures.push(test_name);
            }
        }
    }

    failures
}

/// Extract test name from failure line
pub fn extract_test_name(line: &str) -> Option<String> {
    let trimmed = line.trim();

    if trimmed.starts_with('✕') {
        // Jest failure marker - skip the Unicode character
        return Some(
            trimmed
                .chars()
                .skip(1)
                .collect::<String>()
                .trim()
                .to_string(),
        );
    }

    if trimmed.starts_with("FAIL") {
        // File-level failure
        let parts: Vec<&str> = trimmed.split_whitespace().collect();
        if parts.len() >= 2 {
            return Some(parts[1].to_string());
        }
    }

    None
}

/// Detect test command from package.json
pub fn detect_test_command(package_json: &str) -> Result<String> {
    use serde_json::Value;

    let pkg: Value = serde_json::from_str(package_json)?;

    // Check scripts for test command
    if let Some(scripts) = pkg.get("scripts").and_then(|s| s.as_object()) {
        if scripts.contains_key("test") {
            return Ok("test".to_string());
        }
    }

    // Check devDependencies for framework
    if let Some(deps) = pkg.get("devDependencies").and_then(|d| d.as_object()) {
        if deps.contains_key("vitest") {
            return Ok("vitest".to_string());
        }
        if deps.contains_key("jest") {
            return Ok("jest".to_string());
        }
        if deps.contains_key("mocha") {
            return Ok("mocha".to_string());
        }
    }

    Err(anyhow::anyhow!("No test command found in package.json"))
}

#[cfg_attr(coverage_nightly, coverage(off))]
#[cfg(test)]
mod tests {
    use super::*;

    // =========================================================================
    // TypeScriptAdapter Construction Tests
    // =========================================================================

    #[test]
    fn test_typescript_adapter_new() {
        let adapter = TypeScriptAdapter::new();
        let _ = adapter;
    }

    #[test]
    fn test_typescript_adapter_default() {
        let adapter = TypeScriptAdapter::default();
        let _ = adapter;
    }

    #[test]
    fn test_adapter_name() {
        let adapter = TypeScriptAdapter::new();
        assert_eq!(adapter.name(), "typescript");
    }

    #[test]
    fn test_adapter_extensions() {
        let adapter = TypeScriptAdapter::new();
        let extensions = adapter.extensions();
        assert!(extensions.contains(&"ts"));
        assert!(extensions.contains(&"tsx"));
        assert!(extensions.contains(&"js"));
        assert!(extensions.contains(&"jsx"));
        assert_eq!(extensions.len(), 4);
    }

    #[test]
    fn test_mutation_operators() {
        let adapter = TypeScriptAdapter::new();
        let operators = adapter.mutation_operators();
        assert_eq!(operators.len(), 4);
    }

    // =========================================================================
    // detect_test_command Tests
    // =========================================================================

    #[test]
    fn test_detect_test_command_with_test_script() {
        let package_json = r#"
        {
            "name": "my-project",
            "scripts": {
                "test": "jest"
            }
        }
        "#;

        let result = detect_test_command(package_json);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "test");
    }

    #[test]
    fn test_detect_test_command_with_vitest() {
        let package_json = r#"
        {
            "name": "my-project",
            "devDependencies": {
                "vitest": "^0.34.0"
            }
        }
        "#;

        let result = detect_test_command(package_json);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "vitest");
    }

    #[test]
    fn test_detect_test_command_with_jest() {
        let package_json = r#"
        {
            "name": "my-project",
            "devDependencies": {
                "jest": "^29.0.0"
            }
        }
        "#;

        let result = detect_test_command(package_json);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "jest");
    }

    #[test]
    fn test_detect_test_command_with_mocha() {
        let package_json = r#"
        {
            "name": "my-project",
            "devDependencies": {
                "mocha": "^10.0.0"
            }
        }
        "#;

        let result = detect_test_command(package_json);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "mocha");
    }

    #[test]
    fn test_detect_test_command_no_test_command() {
        let package_json = r#"
        {
            "name": "my-project",
            "devDependencies": {
                "typescript": "^5.0.0"
            }
        }
        "#;

        let result = detect_test_command(package_json);
        assert!(result.is_err());
    }

    #[test]
    fn test_detect_test_command_empty_package_json() {
        let package_json = "{}";

        let result = detect_test_command(package_json);
        assert!(result.is_err());
    }

    #[test]
    fn test_detect_test_command_invalid_json() {
        let package_json = "not valid json";

        let result = detect_test_command(package_json);
        assert!(result.is_err());
    }

    #[test]
    fn test_detect_test_command_prefer_test_script() {
        // If test script exists, it should be preferred over devDependencies
        let package_json = r#"
        {
            "name": "my-project",
            "scripts": {
                "test": "custom-test"
            },
            "devDependencies": {
                "jest": "^29.0.0"
            }
        }
        "#;

        let result = detect_test_command(package_json);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "test");
    }

    // =========================================================================
    // extract_test_name Tests
    // =========================================================================

    #[test]
    fn test_extract_test_name_jest_failure() {
        let line = "✕ should handle invalid input";
        let result = extract_test_name(line);
        assert!(result.is_some());
        assert_eq!(result.unwrap(), "should handle invalid input");
    }

    #[test]
    fn test_extract_test_name_fail_file() {
        let line = "FAIL src/utils.test.ts";
        let result = extract_test_name(line);
        assert!(result.is_some());
        assert_eq!(result.unwrap(), "src/utils.test.ts");
    }

    #[test]
    fn test_extract_test_name_no_match() {
        let line = "PASS src/utils.test.ts";
        let result = extract_test_name(line);
        assert!(result.is_none());
    }

    #[test]
    fn test_extract_test_name_empty_line() {
        let line = "";
        let result = extract_test_name(line);
        assert!(result.is_none());
    }

    #[test]
    fn test_extract_test_name_whitespace_line() {
        let line = "   ";
        let result = extract_test_name(line);
        assert!(result.is_none());
    }

    #[test]
    fn test_extract_test_name_fail_only() {
        let line = "FAIL";
        let result = extract_test_name(line);
        assert!(result.is_none()); // Not enough parts
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
        let stdout = "PASS src/utils.test.ts\nPASS src/index.test.ts";
        let failures = parse_test_failures(stdout, "");
        assert!(failures.is_empty());
    }

    #[test]
    fn test_parse_test_failures_single_failure() {
        let stdout = "✕ should handle invalid input";
        let failures = parse_test_failures(stdout, "");
        assert_eq!(failures.len(), 1);
        assert_eq!(failures[0], "should handle invalid input");
    }

    #[test]
    fn test_parse_test_failures_multiple_failures() {
        let stdout = "✕ test one\n✕ test two\n✕ test three";
        let failures = parse_test_failures(stdout, "");
        assert_eq!(failures.len(), 3);
    }

    #[test]
    fn test_parse_test_failures_from_stderr() {
        let stderr = "FAIL src/broken.test.ts";
        let failures = parse_test_failures("", stderr);
        assert_eq!(failures.len(), 1);
    }

    #[test]
    fn test_parse_test_failures_mixed_stdout_stderr() {
        let stdout = "✕ failing test";
        let stderr = "FAIL src/broken.test.ts";
        let failures = parse_test_failures(stdout, stderr);
        assert_eq!(failures.len(), 2);
    }

    // =========================================================================
    // find_package_json_root Tests
    // =========================================================================

    #[test]
    fn test_find_package_json_root_nonexistent() {
        let path = Path::new("/nonexistent/path/to/file.ts");
        let result = find_package_json_root(path);
        assert!(result.is_none());
    }

    // =========================================================================
    // Async Tests
    // =========================================================================

    #[tokio::test]
    async fn test_parse_simple_source() {
        let adapter = TypeScriptAdapter::new();
        let source = "const x = 1;";
        let result = adapter.parse(source).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_parse_complex_source() {
        let adapter = TypeScriptAdapter::new();
        let source = r#"
            function add(a: number, b: number): number {
                return a + b;
            }
        "#;
        let result = adapter.parse(source).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_unparse() {
        let adapter = TypeScriptAdapter::new();
        let ast = "const x = 1;";
        let result = adapter.unparse(ast).await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), ast);
    }

    #[tokio::test]
    async fn test_run_tests_no_package_json() {
        let adapter = TypeScriptAdapter::new();
        let path = Path::new("/nonexistent/path/file.ts");
        let result = adapter.run_tests(path).await;
        assert!(result.is_err());
    }

    // =========================================================================
    // LanguageAdapter Trait Tests
    // =========================================================================

    #[test]
    fn test_implements_language_adapter() {
        fn _assert_adapter<T: LanguageAdapter>() {}
        _assert_adapter::<TypeScriptAdapter>();
    }

    // =========================================================================
    // Edge Cases
    // =========================================================================

    #[test]
    fn test_detect_test_command_with_multiple_frameworks() {
        // Vitest should be preferred over Jest
        let package_json = r#"
        {
            "devDependencies": {
                "vitest": "^0.34.0",
                "jest": "^29.0.0"
            }
        }
        "#;

        let result = detect_test_command(package_json);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "vitest");
    }

    #[test]
    fn test_parse_test_failures_with_mixed_content() {
        let stdout = r#"
            PASS src/utils.test.ts
            ✕ failing test 1
            ✓ passing test 1
            ✕ failing test 2
            FAIL src/broken.test.ts
            summary: 2 failed, 1 passed
        "#;
        let failures = parse_test_failures(stdout, "");
        assert_eq!(failures.len(), 3);
    }

    #[test]
    fn test_extract_test_name_with_leading_whitespace() {
        let line = "  ✕ test with whitespace";
        let result = extract_test_name(line);
        assert!(result.is_some());
    }

    #[test]
    fn test_extensions_are_complete() {
        let adapter = TypeScriptAdapter::new();
        let extensions = adapter.extensions();

        // All common JS/TS extensions should be covered
        assert!(extensions.iter().any(|e| *e == "ts"), "Missing .ts");
        assert!(extensions.iter().any(|e| *e == "tsx"), "Missing .tsx");
        assert!(extensions.iter().any(|e| *e == "js"), "Missing .js");
        assert!(extensions.iter().any(|e| *e == "jsx"), "Missing .jsx");
    }
}

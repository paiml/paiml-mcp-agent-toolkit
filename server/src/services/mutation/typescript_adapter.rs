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
        return Some(trimmed.chars().skip(1).collect::<String>().trim().to_string());
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

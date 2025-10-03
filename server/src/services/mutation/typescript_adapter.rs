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

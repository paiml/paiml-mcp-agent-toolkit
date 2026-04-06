#![cfg_attr(coverage_nightly, coverage(off))]
//! Accurate Complexity Analyzer using AST-based analysis
//!
//! Sprint 63: Implements industry-standard complexity calculations
//! - Cyclomatic Complexity: Based on `McCabe` (1976) - decision points
//! - Cognitive Complexity: Based on `SonarSource` specification
//! - Supports test exclusion and annotation suppression

use anyhow::Result;
use std::path::Path;
use syn::{visit::Visit, Attribute, Expr, Item, ItemFn, Stmt};
use walkdir::WalkDir;

/// Accurate complexity analyzer with proper AST-based calculation
pub struct AccurateComplexityAnalyzer {
    exclude_tests: bool,
    respect_annotations: bool,
}

impl Default for AccurateComplexityAnalyzer {
    fn default() -> Self {
        Self::new()
    }
}

impl AccurateComplexityAnalyzer {
    #[must_use]
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn new() -> Self {
        Self {
            exclude_tests: false,
            respect_annotations: false,
        }
    }

    #[must_use]
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn exclude_tests(mut self, exclude: bool) -> Self {
        self.exclude_tests = exclude;
        self
    }

    #[must_use]
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn respect_annotations(mut self, respect: bool) -> Self {
        self.respect_annotations = respect;
        self
    }
}

/// Result of analyzing a single file
#[derive(Debug, Clone)]
pub struct FileComplexityResult {
    pub functions: Vec<FunctionMetrics>,
    pub file_path: String,
}

/// Metrics for a single function
#[derive(Debug, Clone)]
pub struct FunctionMetrics {
    pub name: String,
    pub cyclomatic_complexity: u32,
    pub cognitive_complexity: u32,
    pub suppressed: bool,
    /// 1-based line number where the function starts in the source file
    pub line_start: u32,
}

/// Result of analyzing a project
#[derive(Debug, Clone)]
pub struct ProjectComplexityResult {
    pub files_analyzed: usize,
    pub file_metrics: Vec<FileComplexityResult>,
}

/// Build a map of function name -> 1-based line number from source text.
///
/// For duplicate function names (e.g. in different impl blocks), stores the
/// first occurrence so the analyzer can match them in order of appearance.
fn build_function_line_map(content: &str) -> std::collections::HashMap<String, u32> {
    let mut map = std::collections::HashMap::new();
    for (line_idx, line) in content.lines().enumerate() {
        if let Some(name) = extract_fn_name(line.trim()) {
            let line_number = (line_idx + 1) as u32;
            map.entry(name).or_insert(line_number);
        }
    }
    map
}

/// Extract a function name from a trimmed source line, if it contains a function definition.
/// Returns None for comments and non-function lines.
fn extract_fn_name(trimmed: &str) -> Option<String> {
    let fn_pos = trimmed.find("fn ")?;
    let before = trimmed.get(..fn_pos).unwrap_or_default();
    if before.contains("//") || before.contains("/*") {
        return None;
    }
    let after = trimmed.get(fn_pos + 3..).unwrap_or_default();
    let name_end = after
        .find(|c: char| c == '(' || c == '<' || c.is_whitespace())
        .unwrap_or(after.len());
    let name = after.get(..name_end).unwrap_or_default().trim();
    if !name.is_empty() && name.chars().all(|c| c.is_alphanumeric() || c == '_') {
        Some(name.to_string())
    } else {
        None
    }
}

// --- Submodule includes ---
include!("accurate_complexity_analyzer_core.rs");
include!("accurate_complexity_analyzer_visitor.rs");
include!("accurate_complexity_analyzer_tests.rs");

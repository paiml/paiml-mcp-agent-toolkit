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

use crate::services::source_line_index::{FunctionSpans, LineSpan};

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
    /// Create a new instance.
    pub fn new() -> Self {
        Self {
            exclude_tests: false,
            respect_annotations: false,
        }
    }

    #[must_use]
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    /// Exclude tests.
    pub fn exclude_tests(mut self, exclude: bool) -> Self {
        self.exclude_tests = exclude;
        self
    }

    #[must_use]
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    /// Respect annotations.
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
    /// Real number of lines in the file. Callers used to derive this from the
    /// last function's invented `line_end`, which reported a 13-line file as
    /// 61 lines (#652).
    pub total_lines: u32,
}

/// Metrics for a single function
#[derive(Debug, Clone)]
pub struct FunctionMetrics {
    pub name: String,
    pub cyclomatic_complexity: u32,
    pub cognitive_complexity: u32,
    /// Deepest nesting level reached inside the body (measured, not derived
    /// from cognitive complexity).
    pub max_nesting: u32,
    pub suppressed: bool,
    /// 1-based line number where the function starts; 0 when not located.
    pub line_start: u32,
    /// 1-based inclusive line where the function ends; 0 when not located.
    pub line_end: u32,
}

/// Cyclomatic/cognitive/nesting for one function body.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct BlockComplexity {
    pub cyclomatic: u32,
    pub cognitive: u32,
    pub max_nesting: u32,
}

/// Measure one function body with the visitor `analyze complexity` and
/// `quality-gate` use.
///
/// `pmat context` used to run a private "GREEN PHASE" stub that only inspected
/// top-level statements and set `cognitive = cyclomatic`; on the same function
/// it reported 3/3 where this visitor reports 9/15 (#686). Routing every caller
/// through here is what makes the commands agree.
#[must_use]
pub fn measure_block(function_name: &str, block: &syn::Block) -> BlockComplexity {
    let mut visitor = ComplexityVisitor::new().with_function_name(function_name.to_string());
    visitor.visit_block(block);
    BlockComplexity {
        cyclomatic: visitor.cyclomatic,
        cognitive: visitor.cognitive,
        max_nesting: visitor.max_nesting,
    }
}

/// Result of analyzing a project
#[derive(Debug, Clone)]
pub struct ProjectComplexityResult {
    pub files_analyzed: usize,
    pub file_metrics: Vec<FileComplexityResult>,
}

/// Build a map of function name -> measured 1-based line span from source text.
///
/// Spans come from [`FunctionSpans`], which reads the real closing brace. The
/// previous version recorded only the start line, which forced callers to
/// invent an end (#652, #656).
fn build_function_line_map(content: &str) -> FunctionSpans {
    FunctionSpans::from_source(content)
}

// --- Submodule includes ---
include!("accurate_complexity_analyzer_core.rs");
include!("accurate_complexity_analyzer_visitor.rs");
include!("accurate_complexity_analyzer_tests.rs");

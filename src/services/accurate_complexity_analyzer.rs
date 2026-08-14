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

/// One function definition found in a Rust file, with the body to measure.
///
/// Borrowed from the parsed tree so discovery allocates nothing but the name.
pub struct DiscoveredFn<'a> {
    /// Bare identifier, matching how [`FunctionSpans`] keys the source scan.
    pub name: String,
    /// Attributes on the definition (used for `#[allow(complex_function)]`).
    pub attrs: &'a [Attribute],
    /// The body whose decision points are counted.
    pub block: &'a syn::Block,
}

/// Every function definition in `items`, in source order.
///
/// This is the single answer to "which functions does a Rust file contain".
/// The complexity walker used to ask only for top-level `Item::Fn`, so a file
/// whose functions all lived inside `impl` / `trait` / `mod` blocks — i.e. most
/// idiomatic Rust — measured as *zero* functions with `max_cyclomatic: 0`, and
/// any gate reading those numbers passed the file unconditionally. 50 bodies as
/// free functions scored `total_functions: 50`; the same 50 bodies moved into
/// `impl S { .. }` scored 0.
///
/// Source order matters: spans are consumed from [`FunctionSpans`] in textual
/// order, so two functions sharing a name must be visited in the order they
/// appear in the text.
#[must_use]
pub fn collect_functions(items: &[Item]) -> Vec<DiscoveredFn<'_>> {
    let mut found = Vec::new();
    collect_functions_into(items, &mut found);
    found
}

fn collect_functions_into<'a>(items: &'a [Item], found: &mut Vec<DiscoveredFn<'a>>) {
    for item in items {
        match item {
            Item::Fn(func) => found.push(DiscoveredFn {
                name: func.sig.ident.to_string(),
                attrs: &func.attrs,
                block: &func.block,
            }),
            Item::Impl(block) => {
                for member in &block.items {
                    if let syn::ImplItem::Fn(func) = member {
                        found.push(DiscoveredFn {
                            name: func.sig.ident.to_string(),
                            attrs: &func.attrs,
                            block: &func.block,
                        });
                    }
                }
            }
            Item::Trait(decl) => {
                for member in &decl.items {
                    if let syn::TraitItem::Fn(func) = member {
                        // A required method (`fn f(&self);`) has no body: there
                        // is no code to measure, so it is not counted. Only
                        // default methods carry a block.
                        if let Some(block) = &func.default {
                            found.push(DiscoveredFn {
                                name: func.sig.ident.to_string(),
                                attrs: &func.attrs,
                                block,
                            });
                        }
                    }
                }
            }
            // `mod foo;` has no inline body; `mod foo { .. }` does.
            Item::Mod(module) => {
                if let Some((_, nested)) = &module.content {
                    collect_functions_into(nested, found);
                }
            }
            _ => {}
        }
    }
}

// --- Submodule includes ---
include!("accurate_complexity_analyzer_core.rs");
include!("accurate_complexity_analyzer_visitor.rs");
include!("accurate_complexity_analyzer_tests.rs");

#![cfg_attr(coverage_nightly, coverage(off))]
use std::collections::HashMap;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
/// Makefile ast.
pub struct MakefileAst {
    pub nodes: Vec<MakefileNode>,
    pub source_map: HashMap<usize, SourceSpan>,
    pub metadata: MakefileMetadata,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
/// Makefile node.
pub struct MakefileNode {
    pub kind: MakefileNodeKind,
    pub span: SourceSpan,
    pub children: Vec<usize>, // Indices into nodes vec
    pub data: NodeData,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
/// Makefile node kind.
pub enum MakefileNodeKind {
    Rule,
    Variable,
    Recipe,
    Include,
    Conditional,
    Expansion,
    Comment,
    Directive,
    Target,
    Prerequisite,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
/// Node data.
pub enum NodeData {
    Rule {
        targets: Vec<String>,
        prerequisites: Vec<String>,
        is_pattern: bool,
        is_phony: bool,
        is_double_colon: bool,
    },
    Variable {
        name: String,
        assignment_op: AssignmentOp,
        value: String,
    },
    Recipe {
        lines: Vec<RecipeLine>,
    },
    Target {
        name: String,
    },
    Text(String),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
/// Available operations for assignment.
pub enum AssignmentOp {
    Deferred,    // =
    Immediate,   // :=
    Conditional, // ?=
    Append,      // +=
    Shell,       // !=
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
/// Recipe line.
pub struct RecipeLine {
    pub text: String,
    pub prefixes: RecipePrefixes,
}

#[derive(Debug, Clone, Copy, Default, serde::Serialize, serde::Deserialize)]
/// Recipe prefixes.
pub struct RecipePrefixes {
    pub silent: bool,       // @
    pub ignore_error: bool, // -
    pub always_exec: bool,  // +
}

#[derive(Debug, Clone, Copy, serde::Serialize, serde::Deserialize)]
/// Source span.
pub struct SourceSpan {
    pub start: usize,
    pub end: usize,
    pub line: usize,
    pub column: usize,
}

impl SourceSpan {
    #[must_use]
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    /// Create a new instance.
    pub fn new(start: usize, end: usize, line: usize, column: usize) -> Self {
        Self {
            start,
            end,
            line,
            column,
        }
    }

    #[must_use]
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    /// File level.
    pub fn file_level() -> Self {
        Self {
            start: 0,
            end: 0,
            line: 0,
            column: 0,
        }
    }
}

#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
/// Metadata for makefile.
pub struct MakefileMetadata {
    pub has_phony_rules: bool,
    pub has_pattern_rules: bool,
    pub uses_automatic_variables: bool,
    pub target_count: usize,
    pub variable_count: usize,
    pub recipe_count: usize,
}

impl Default for MakefileAst {
    fn default() -> Self {
        Self::new()
    }
}

// MakefileAst query and accessor methods
include!("ast_queries.rs");

// Unit tests and property tests
include!("ast_tests.rs");

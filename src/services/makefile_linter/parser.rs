#![allow(unused)]
#![cfg_attr(coverage_nightly, coverage(off))]
use super::ast::{
    AssignmentOp, MakefileAst, MakefileNode, MakefileNodeKind, NodeData, RecipeLine,
    RecipePrefixes, SourceSpan,
};

#[derive(Debug)]
/// Makefile parser.
pub struct MakefileParser<'src> {
    input: &'src str,
    cursor: usize,
    line: usize,
    column: usize,
    errors: Vec<ParseError>,
}

#[derive(Debug, Clone)]
/// Error variants for parse operations.
pub enum ParseError {
    UnexpectedEof,
    InvalidSyntax(String),
    InvalidTarget(String),
    InvalidVariable(String),
}

enum LineType {
    Assignment(usize, AssignmentOp),
    Rule(usize, bool), // position, is_double_colon
}

// Core parsing: new(), safe_slice(), parse(), parse_line(), directive dispatch
include!("parser_core.rs");

// Node-specific parsing: rules, variables, recipes, comments, includes, conditionals
include!("parser_nodes.rs");

// SWAR-optimized scanning and line-type detection
include!("parser_scanner.rs");

// Cursor navigation and character-level helpers
include!("parser_cursor.rs");

// Unit tests and property tests
include!("parser_tests.rs");

// Regression tests for `::=` / `:::=` scanning
include!("parser_scanner_tests.rs");

// Parser/registry coverage tests (previously an orphaned file)
include!("coverage_tests.rs");

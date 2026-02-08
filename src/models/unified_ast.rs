#![cfg_attr(coverage_nightly, coverage(off))]
//! Unified AST representation - split for file health (CB-040)

include!("unified_ast_types.rs");

#[cfg(test)]
include!("unified_ast_types_tests.rs");

#[cfg(test)]
#[path = "unified_ast_tests.rs"]
mod tests;

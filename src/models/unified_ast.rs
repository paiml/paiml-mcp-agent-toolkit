//! Unified AST representation - split for file health (CB-040)

include!("unified_ast_types.rs");

#[cfg(test)]
#[path = "unified_ast_tests.rs"]
mod tests;

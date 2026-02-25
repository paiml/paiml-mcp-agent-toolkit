#![cfg_attr(coverage_nightly, coverage(off))]
//! TypeScript/JavaScript tree-sitter based mutation operators
//!
//! EXTREME TDD: RED PHASE - Stub implementations, all tests will fail
//!
//! This module implements AST-based mutation operators for TypeScript and JavaScript
//! using tree-sitter instead of language-specific parsers.
//!
//! ## Module layout (include pattern)
//!
//! - `typescript_tree_sitter_mutations_operators.rs` — trait impls for all 5 mutation structs
//! - `typescript_tree_sitter_mutations_tests.rs` — unit tests

use super::tree_sitter_operators::{MutatedSource, TreeSitterMutationOperator};
use super::types::SourceLocation;
use tree_sitter::Node;

/// Arithmetic Operator Replacement (AOR) for TypeScript/JavaScript
///
/// Mutations: + → -, * → /, etc.
pub struct TypeScriptBinaryOpMutation;

/// Strict Equality Mutation for TypeScript/JavaScript
///
/// Mutations: === → ==, !== → !=
pub struct TypeScriptStrictEqualityMutation;

/// Optional Chaining Mutation for TypeScript
///
/// Mutations: obj?.prop → obj.prop
pub struct TypeScriptOptionalChainingMutation;

/// Nullish Coalescing Mutation for TypeScript
///
/// Mutations: a ?? b → a || b, a ?? b → b
pub struct TypeScriptNullishCoalescingMutation;

/// Async/Await Mutation for TypeScript/JavaScript
///
/// Mutations: Remove await, remove async
pub struct TypeScriptAsyncAwaitMutation;

// --- Trait implementations for all mutation operators ---
include!("typescript_tree_sitter_mutations_operators.rs");

// --- Tests ---
include!("typescript_tree_sitter_mutations_tests.rs");

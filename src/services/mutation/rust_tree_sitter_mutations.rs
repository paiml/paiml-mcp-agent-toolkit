#![cfg_attr(coverage_nightly, coverage(off))]
// Rust Mutation Operators using tree-sitter AST
// PMAT-7014: Rust Mutation Testing
// Status: RED Phase - Stub implementation

use super::tree_sitter_operators::{MutatedSource, TreeSitterMutationOperator};
use super::types::SourceLocation;
use tree_sitter::Node;

// ============================================================
// Struct Definitions
// ============================================================

/// 1. Binary Operator Replacement (AOR)
pub struct RustBinaryOpMutation;

/// 2. Relational Operator Replacement (ROR)
pub struct RustRelationalOpMutation;

/// 3. Logical Operator Replacement (LOR)
pub struct RustLogicalOpMutation;

/// 4. Bitwise Operator Replacement (BOR)
pub struct RustBitwiseOpMutation;

/// 5. Range Operator Replacement (RANGEOR) - Rust-specific
pub struct RustRangeOpMutation;

/// 6. Pattern Match Replacement (PMR) - Rust-specific
pub struct RustPatternMutation;

/// 7. Method Chain Replacement (MCR) - Rust-specific
pub struct RustMethodChainMutation;

/// 8. Borrow/Reference Mutation (LBM) - Rust-specific
pub struct RustBorrowMutation;

// ============================================================
// Implementations (split into include files)
// ============================================================

include!("rust_tree_sitter_mutations_operators.rs");

// ============================================================
// Tests
// ============================================================

include!("rust_tree_sitter_mutations_tests.rs");

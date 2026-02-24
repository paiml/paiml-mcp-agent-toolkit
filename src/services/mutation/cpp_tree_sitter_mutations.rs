#![cfg_attr(coverage_nightly, coverage(off))]
// C++ Mutation Operators using tree-sitter AST
// PMAT-7013: C++ Mutation Testing
// Status: GREEN Phase - Full implementation
//
// Split into include files:
//   - cpp_binary_op_mutations.rs: AOR (arithmetic), ROR (relational), LOR (logical)
//   - cpp_bitwise_unary_mutations.rs: BOR (bitwise), UOR (unary)
//   - cpp_pointer_member_mutations.rs: POR (pointer), MAR (member access)
//   - cpp_tree_sitter_mutations_tests.rs: Unit tests

use super::tree_sitter_operators::{MutatedSource, TreeSitterMutationOperator};
use super::types::SourceLocation;
use tree_sitter::Node;

// 1. Binary Operator (AOR), Relational Operator (ROR), Logical Operator (LOR)
include!("cpp_binary_op_mutations.rs");

// 2. Bitwise Operator (BOR), Unary Operator (UOR)
include!("cpp_bitwise_unary_mutations.rs");

// 3. Pointer Operator (POR), Member Access (MAR)
include!("cpp_pointer_member_mutations.rs");

// 4. Unit tests
include!("cpp_tree_sitter_mutations_tests.rs");

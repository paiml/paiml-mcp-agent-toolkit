#![cfg_attr(coverage_nightly, coverage(off))]
// Python AST mutation operators using tree-sitter
// PMAT-7011: Python Mutation Testing
// Status: RED Phase - Stub implementations

use super::tree_sitter_operators::{MutatedSource, TreeSitterMutationOperator};
use super::types::SourceLocation;
use tree_sitter::Node;

/// Python Binary Operator Mutation (AOR - Arithmetic Operator Replacement)
/// Replaces +, -, *, /, //, %, ** with other arithmetic operators
pub struct PythonBinaryOpMutation;

/// Python Relational Operator Mutation (ROR)
/// Replaces <, >, <=, >=, ==, != with other relational operators
pub struct PythonRelationalOpMutation;

/// Python Logical Operator Mutation (LOR)
/// Replaces 'and', 'or' with each other or removes them
pub struct PythonLogicalOpMutation;

/// Python Identity Operator Mutation
/// Replaces 'is' with 'is not' and '=='
pub struct PythonIdentityOpMutation;

/// Python Membership Operator Mutation
/// Replaces 'in' with 'not in'
pub struct PythonMembershipOpMutation;

// Arithmetic (AOR) and Relational (ROR) operator mutation impls + helpers
include!("python_arithmetic_relational_ops.rs");

// Logical (LOR), Identity, and Membership operator mutation impls + helpers
include!("python_logical_identity_membership_ops.rs");

// Tests for all Python mutation operators
include!("python_tree_sitter_mutations_tests.rs");

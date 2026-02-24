#![cfg_attr(coverage_nightly, coverage(off))]
// Go Mutation Operators using tree-sitter AST
// PMAT-7012: Go Mutation Testing
// Status: GREEN Phase - All operators implemented

use super::tree_sitter_operators::{MutatedSource, TreeSitterMutationOperator};
use super::types::SourceLocation;
use tree_sitter::Node;

/// Go Binary Operator Mutation (AOR - Arithmetic Operator Replacement)
///
/// Mutates arithmetic operators in Go: +, -, *, /, %
///
/// Example:
/// ```go
/// func Add(a, b int) int {
///     return a + b  // Mutated to: a - b, a * b, a / b, a % b
/// }
/// ```
pub struct GoBinaryOpMutation;

/// Go Relational Operator Mutation (ROR - Relational Operator Replacement)
///
/// Mutates comparison operators: <, >, <=, >=, ==, !=
///
/// Example:
/// ```go
/// func IsPositive(value int) bool {
///     return value > 0  // Mutated to: value < 0, value >= 0, etc.
/// }
/// ```
pub struct GoRelationalOpMutation;

/// Go Logical Operator Mutation (LOR - Logical Operator Replacement)
///
/// Mutates logical operators: &&, ||
///
/// Example:
/// ```go
/// func BothPositive(a, b int) bool {
///     return a > 0 && b > 0  // Mutated to: a > 0 || b > 0
/// }
/// ```
pub struct GoLogicalOpMutation;

/// Go Bitwise Operator Mutation (BOR - Bitwise Operator Replacement)
///
/// Mutates bitwise operators: &, |, ^, <<, >>
///
/// Example:
/// ```go
/// func BitwiseAnd(a, b int) int {
///     return a & b  // Mutated to: a | b, a ^ b, a << b, a >> b
/// }
/// ```
pub struct GoBitwiseOpMutation;

/// Go Unary Operator Mutation (UOR - Unary Operator Replacement)
///
/// Mutates unary operators: !, -, +
///
/// Example:
/// ```go
/// func Negate(value int) int {
///     return -value  // Mutated to: +value, value (remove operator)
/// }
/// ```
pub struct GoUnaryOpMutation;

/// Go Assignment Operator Mutation
///
/// Mutates assignment operators: +=, -=, *=, /=
///
/// Example:
/// ```go
/// func AddAssign(value, delta int) int {
///     value += delta  // Mutated to: value -= delta, value *= delta, value /= delta
///     return value
/// }
/// ```
pub struct GoAssignmentOpMutation;

// All Go operator trait implementations (AOR, ROR, LOR, BOR, UOR, Assignment)
include!("go_operator_impls.rs");

// Tests for all Go mutation operators
include!("go_mutations_tests.rs");

#![cfg_attr(coverage_nightly, coverage(off))]
//! Core mutation testing types

use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Represents a single mutant - a syntactic variation of source code
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct Mutant {
    /// Unique identifier for this mutant
    pub id: String,

    /// Original source file path
    pub original_file: PathBuf,

    /// Mutated source code
    pub mutated_source: String,

    /// Location in source where mutation occurred
    pub location: SourceLocation,

    /// Mutation operator applied
    pub operator: MutationOperatorType,

    /// Hash of mutated source for deduplication
    pub hash: String,

    /// Execution status
    pub status: MutantStatus,
}

/// Source code location
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct SourceLocation {
    pub line: usize,
    pub column: usize,
    pub end_line: usize,
    pub end_column: usize,
}

/// Mutation operator types
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum MutationOperatorType {
    /// Arithmetic Operator Replacement (+ → -, * → /, etc.)
    ArithmeticReplacement,
    /// Relational Operator Replacement (< → <=, == → !=, etc.)
    RelationalReplacement,
    /// Conditional Operator Replacement (&& → ||, etc.)
    ConditionalReplacement,
    /// Constant Replacement (0 → 1, true → false, etc.)
    ConstantReplacement,
    /// Statement Deletion
    StatementDeletion,
    /// Return Value Replacement
    ReturnReplacement,
    /// Variable Replacement
    VariableReplacement,
    /// Conditional Return Operator (early returns)
    ConditionalReturn,
    /// Boundary Value Operator (off-by-one)
    BoundaryValue,
    /// Exception Handler Removal
    ExceptionHandlerRemoval,
    /// Return Value Replacement (alternative naming)
    ReturnValueReplacement,
    /// Unary Operator Replacement (!, -, ~)
    UnaryReplacement,
    /// Bitwise Operator Replacement (&, |, ^, <<, >>)
    BitwiseReplacement,
    /// Assignment Operator Replacement (+=, -=, *=, /=)
    AssignmentReplacement,
    /// Pointer Operator Replacement (*, &, ->) - C++ specific
    PointerReplacement,
    /// Member Access Replacement (., ::) - C++ specific
    MemberAccessReplacement,
    /// Range Operator Replacement (.., ..=) - Rust specific
    RangeReplacement,
    /// Pattern Matching Replacement (Some/None, Ok/Err) - Rust specific
    PatternReplacement,
    /// Method Chain Replacement (.map, .filter) - Rust specific
    MethodChainReplacement,
    /// Borrow/Reference Replacement (&, &mut) - Rust specific
    BorrowReplacement,
    /// Custom operator (language-specific)
    Custom(String),
    /// None (for testing)
    None,
}

/// Mutant execution status
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum MutantStatus {
    /// Mutant not yet executed
    Pending,
    /// Mutant detected by test suite (good!)
    Killed,
    /// Mutant survived test suite (test gap!)
    Survived,
    /// Mutant caused compilation error
    CompileError,
    /// Mutant caused test timeout
    Timeout,
    /// Mutant is semantically equivalent to original
    Equivalent,
}

/// Mutation result after execution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MutationResult {
    /// The mutant that was executed
    pub mutant: Mutant,
    /// Execution status
    pub status: MutantStatus,
    /// Test failures that killed this mutant
    pub test_failures: Vec<String>,
    /// Execution time in milliseconds
    pub execution_time_ms: u64,
    /// Error message if compilation failed
    pub error_message: Option<String>,
}

/// Mutation score metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MutationScore {
    /// Mutation score (0.0 - 1.0)
    pub score: f64,
    /// Total mutants generated
    pub total: usize,
    /// Mutants killed by tests
    pub killed: usize,
    /// Mutants that survived
    pub survived: usize,
    /// Mutants with compile errors
    pub compile_errors: usize,
    /// Mutants that timed out
    pub timeouts: usize,
    /// Equivalent mutants
    pub equivalent: usize,
}

/// Weak spot in test coverage
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WeakSpot {
    /// File with weak coverage
    pub file: PathBuf,
    /// Line range with weak coverage
    pub line_range: (usize, usize),
    /// Number of survived mutants in this range
    pub survived_mutants: usize,
    /// Suggested test improvements
    pub suggestions: Vec<String>,
}

// --- Implementation methods ---
include!("types_impls.rs");

// --- Tests ---
include!("types_tests.rs");

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

impl MutationScore {
    /// Calculate mutation score from results
    pub fn from_results(results: &[MutationResult]) -> Self {
        let total = results.len();
        let killed = results.iter().filter(|r| r.status == MutantStatus::Killed).count();
        let survived = results.iter().filter(|r| r.status == MutantStatus::Survived).count();
        let compile_errors = results.iter().filter(|r| r.status == MutantStatus::CompileError).count();
        let timeouts = results.iter().filter(|r| r.status == MutantStatus::Timeout).count();
        let equivalent = results.iter().filter(|r| r.status == MutantStatus::Equivalent).count();

        // Mutation score = killed / (total - equivalent)
        let valid_mutants = total.saturating_sub(equivalent + compile_errors);
        let score = if valid_mutants > 0 {
            killed as f64 / valid_mutants as f64
        } else {
            0.0
        };

        Self {
            score,
            total,
            killed,
            survived,
            compile_errors,
            timeouts,
            equivalent,
        }
    }
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mutation_score_calculation() {
        let results = vec![
            MutationResult {
                mutant: create_test_mutant(MutantStatus::Killed),
                status: MutantStatus::Killed,
                test_failures: vec!["test_add".to_string()],
                execution_time_ms: 100,
                error_message: None,
            },
            MutationResult {
                mutant: create_test_mutant(MutantStatus::Killed),
                status: MutantStatus::Killed,
                test_failures: vec!["test_sub".to_string()],
                execution_time_ms: 150,
                error_message: None,
            },
            MutationResult {
                mutant: create_test_mutant(MutantStatus::Survived),
                status: MutantStatus::Survived,
                test_failures: vec![],
                execution_time_ms: 200,
                error_message: None,
            },
        ];

        let score = MutationScore::from_results(&results);

        assert_eq!(score.total, 3);
        assert_eq!(score.killed, 2);
        assert_eq!(score.survived, 1);
        assert!((score.score - 0.666).abs() < 0.01);
    }

    #[test]
    fn test_mutation_score_with_equivalent() {
        let results = vec![
            MutationResult {
                mutant: create_test_mutant(MutantStatus::Killed),
                status: MutantStatus::Killed,
                test_failures: vec!["test".to_string()],
                execution_time_ms: 100,
                error_message: None,
            },
            MutationResult {
                mutant: create_test_mutant(MutantStatus::Equivalent),
                status: MutantStatus::Equivalent,
                test_failures: vec![],
                execution_time_ms: 50,
                error_message: None,
            },
        ];

        let score = MutationScore::from_results(&results);

        // Score should be 1.0 (1 killed / 1 valid mutant)
        assert_eq!(score.score, 1.0);
        assert_eq!(score.equivalent, 1);
    }

    fn create_test_mutant(status: MutantStatus) -> Mutant {
        Mutant {
            id: "test".to_string(),
            original_file: PathBuf::from("test.rs"),
            mutated_source: "fn test() {}".to_string(),
            location: SourceLocation {
                line: 1,
                column: 1,
                end_line: 1,
                end_column: 10,
            },
            operator: MutationOperatorType::ArithmeticReplacement,
            hash: "hash".to_string(),
            status,
        }
    }
}

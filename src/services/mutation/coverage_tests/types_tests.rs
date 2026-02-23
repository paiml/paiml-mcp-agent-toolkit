#![cfg_attr(coverage_nightly, coverage(off))]

use super::*;
use std::path::PathBuf;

#[test]
fn test_mutant_creation_and_fields() {
    let mutant = Mutant {
        id: "test_id_123".to_string(),
        original_file: PathBuf::from("src/lib.rs"),
        mutated_source: "fn test() {}".to_string(),
        location: SourceLocation {
            line: 10,
            column: 5,
            end_line: 10,
            end_column: 20,
        },
        operator: MutationOperatorType::ArithmeticReplacement,
        hash: "abc123".to_string(),
        status: MutantStatus::Pending,
    };

    assert_eq!(mutant.id, "test_id_123");
    assert_eq!(mutant.original_file, PathBuf::from("src/lib.rs"));
    assert_eq!(mutant.mutated_source, "fn test() {}");
    assert_eq!(mutant.location.line, 10);
    assert_eq!(mutant.location.column, 5);
    assert_eq!(mutant.hash, "abc123");
    assert_eq!(mutant.status, MutantStatus::Pending);
}

#[test]
fn test_source_location_boundary_values() {
    let loc = SourceLocation {
        line: 0,
        column: 0,
        end_line: usize::MAX,
        end_column: usize::MAX,
    };

    assert_eq!(loc.line, 0);
    assert_eq!(loc.column, 0);
    assert_eq!(loc.end_line, usize::MAX);
    assert_eq!(loc.end_column, usize::MAX);
}

#[test]
fn test_mutation_operator_type_ordering() {
    // Test that MutationOperatorType implements Ord correctly
    let arith = MutationOperatorType::ArithmeticReplacement;
    let relat = MutationOperatorType::RelationalReplacement;
    let cond = MutationOperatorType::ConditionalReplacement;

    // They should be comparable (PartialOrd, Ord)
    assert!(arith <= relat || arith >= relat);
    assert!(relat <= cond || relat >= cond);
}

#[test]
fn test_mutation_operator_type_all_variants() {
    // Test all operator type variants exist and can be matched
    let operators = vec![
        MutationOperatorType::ArithmeticReplacement,
        MutationOperatorType::RelationalReplacement,
        MutationOperatorType::ConditionalReplacement,
        MutationOperatorType::ConstantReplacement,
        MutationOperatorType::StatementDeletion,
        MutationOperatorType::ReturnReplacement,
        MutationOperatorType::VariableReplacement,
        MutationOperatorType::ConditionalReturn,
        MutationOperatorType::BoundaryValue,
        MutationOperatorType::ExceptionHandlerRemoval,
        MutationOperatorType::ReturnValueReplacement,
        MutationOperatorType::UnaryReplacement,
        MutationOperatorType::BitwiseReplacement,
        MutationOperatorType::AssignmentReplacement,
        MutationOperatorType::PointerReplacement,
        MutationOperatorType::MemberAccessReplacement,
        MutationOperatorType::RangeReplacement,
        MutationOperatorType::PatternReplacement,
        MutationOperatorType::MethodChainReplacement,
        MutationOperatorType::BorrowReplacement,
        MutationOperatorType::Custom("test".to_string()),
        MutationOperatorType::None,
    ];

    assert_eq!(operators.len(), 22);
}

#[test]
fn test_mutant_status_all_variants() {
    let statuses = vec![
        MutantStatus::Pending,
        MutantStatus::Killed,
        MutantStatus::Survived,
        MutantStatus::CompileError,
        MutantStatus::Timeout,
        MutantStatus::Equivalent,
    ];

    assert_eq!(statuses.len(), 6);

    // Test equality
    assert_eq!(MutantStatus::Pending, MutantStatus::Pending);
    assert_ne!(MutantStatus::Killed, MutantStatus::Survived);
}

#[test]
fn test_mutation_result_creation() {
    let mutant = Mutant {
        id: "test".to_string(),
        original_file: PathBuf::from("test.rs"),
        mutated_source: "mutated".to_string(),
        location: SourceLocation {
            line: 1,
            column: 1,
            end_line: 1,
            end_column: 10,
        },
        operator: MutationOperatorType::ArithmeticReplacement,
        hash: "hash".to_string(),
        status: MutantStatus::Pending,
    };

    let result = MutationResult {
        mutant: mutant.clone(),
        status: MutantStatus::Killed,
        test_failures: vec!["test_add".to_string(), "test_sub".to_string()],
        execution_time_ms: 150,
        error_message: None,
    };

    assert_eq!(result.status, MutantStatus::Killed);
    assert_eq!(result.test_failures.len(), 2);
    assert_eq!(result.execution_time_ms, 150);
    assert!(result.error_message.is_none());
}

#[test]
fn test_mutation_result_with_error() {
    let mutant = Mutant {
        id: "error_test".to_string(),
        original_file: PathBuf::from("test.rs"),
        mutated_source: "invalid".to_string(),
        location: SourceLocation {
            line: 1,
            column: 1,
            end_line: 1,
            end_column: 5,
        },
        operator: MutationOperatorType::StatementDeletion,
        hash: "errhash".to_string(),
        status: MutantStatus::CompileError,
    };

    let result = MutationResult {
        mutant,
        status: MutantStatus::CompileError,
        test_failures: vec![],
        execution_time_ms: 0,
        error_message: Some("error[E0308]: mismatched types".to_string()),
    };

    assert_eq!(result.status, MutantStatus::CompileError);
    assert!(result.error_message.is_some());
    assert!(result.error_message.unwrap().contains("E0308"));
}

#[test]
fn test_weak_spot_creation() {
    let weak_spot = WeakSpot {
        file: PathBuf::from("src/critical.rs"),
        line_range: (10, 50),
        survived_mutants: 5,
        suggestions: vec![
            "Add boundary tests".to_string(),
            "Test error paths".to_string(),
        ],
    };

    assert_eq!(weak_spot.file, PathBuf::from("src/critical.rs"));
    assert_eq!(weak_spot.line_range, (10, 50));
    assert_eq!(weak_spot.survived_mutants, 5);
    assert_eq!(weak_spot.suggestions.len(), 2);
}

#[test]
fn test_mutant_serialization() {
    let mutant = Mutant {
        id: "ser_test".to_string(),
        original_file: PathBuf::from("src/lib.rs"),
        mutated_source: "fn x() {}".to_string(),
        location: SourceLocation {
            line: 1,
            column: 1,
            end_line: 1,
            end_column: 10,
        },
        operator: MutationOperatorType::ArithmeticReplacement,
        hash: "ser_hash".to_string(),
        status: MutantStatus::Pending,
    };

    let json = serde_json::to_string(&mutant).unwrap();
    let deserialized: Mutant = serde_json::from_str(&json).unwrap();

    assert_eq!(deserialized.id, mutant.id);
    assert_eq!(deserialized.original_file, mutant.original_file);
    assert_eq!(deserialized.status, mutant.status);
}

#[test]
fn test_mutant_hash_and_eq() {
    use std::collections::HashSet;

    let mutant1 = Mutant {
        id: "same".to_string(),
        original_file: PathBuf::from("test.rs"),
        mutated_source: "fn a() {}".to_string(),
        location: SourceLocation {
            line: 1,
            column: 1,
            end_line: 1,
            end_column: 10,
        },
        operator: MutationOperatorType::ArithmeticReplacement,
        hash: "same_hash".to_string(),
        status: MutantStatus::Pending,
    };

    let mutant2 = mutant1.clone();

    // Test that cloned mutants are equal
    assert_eq!(mutant1, mutant2);

    // Test that they can be stored in a HashSet
    let mut set = HashSet::new();
    set.insert(mutant1.clone());
    assert!(set.contains(&mutant2));
}

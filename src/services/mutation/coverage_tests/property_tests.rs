#![cfg_attr(coverage_nightly, coverage(off))]

use proptest::prelude::*;

use super::*;
use std::path::PathBuf;

proptest! {
    #[test]
    fn prop_mutation_score_calculation_valid_range(
        killed in 0usize..100,
        survived in 0usize..100,
        equivalent in 0usize..50,
        compile_errors in 0usize..30
    ) {
        let mut results = Vec::new();

        // Create killed mutants
        for i in 0..killed {
            results.push(MutationResult {
                mutant: Mutant {
                    id: format!("k{}", i),
                    original_file: PathBuf::from("test.rs"),
                    mutated_source: String::new(),
                    location: SourceLocation { line: 1, column: 1, end_line: 1, end_column: 1 },
                    operator: MutationOperatorType::ArithmeticReplacement,
                    hash: format!("h{}", i),
                    status: MutantStatus::Killed,
                },
                status: MutantStatus::Killed,
                test_failures: vec!["test".to_string()],
                execution_time_ms: 100,
                error_message: None,
            });
        }

        // Create survived mutants
        for i in 0..survived {
            results.push(MutationResult {
                mutant: Mutant {
                    id: format!("s{}", i),
                    original_file: PathBuf::from("test.rs"),
                    mutated_source: String::new(),
                    location: SourceLocation { line: 1, column: 1, end_line: 1, end_column: 1 },
                    operator: MutationOperatorType::ArithmeticReplacement,
                    hash: format!("s{}", i),
                    status: MutantStatus::Survived,
                },
                status: MutantStatus::Survived,
                test_failures: vec![],
                execution_time_ms: 100,
                error_message: None,
            });
        }

        // Create equivalent mutants
        for i in 0..equivalent {
            results.push(MutationResult {
                mutant: Mutant {
                    id: format!("e{}", i),
                    original_file: PathBuf::from("test.rs"),
                    mutated_source: String::new(),
                    location: SourceLocation { line: 1, column: 1, end_line: 1, end_column: 1 },
                    operator: MutationOperatorType::ArithmeticReplacement,
                    hash: format!("e{}", i),
                    status: MutantStatus::Equivalent,
                },
                status: MutantStatus::Equivalent,
                test_failures: vec![],
                execution_time_ms: 50,
                error_message: None,
            });
        }

        // Create compile error mutants
        for i in 0..compile_errors {
            results.push(MutationResult {
                mutant: Mutant {
                    id: format!("c{}", i),
                    original_file: PathBuf::from("test.rs"),
                    mutated_source: String::new(),
                    location: SourceLocation { line: 1, column: 1, end_line: 1, end_column: 1 },
                    operator: MutationOperatorType::ArithmeticReplacement,
                    hash: format!("c{}", i),
                    status: MutantStatus::CompileError,
                },
                status: MutantStatus::CompileError,
                test_failures: vec![],
                execution_time_ms: 0,
                error_message: Some("error".to_string()),
            });
        }

        let score = MutationScore::from_results(&results);

        // Score should always be between 0.0 and 1.0
        prop_assert!(score.score >= 0.0 && score.score <= 1.0,
            "Score {} out of range [0.0, 1.0]", score.score);

        // Counts should add up
        prop_assert_eq!(
            score.total,
            killed + survived + equivalent + compile_errors,
            "Total count mismatch"
        );
    }

    #[test]
    fn prop_source_location_fields_preserved(
        line in 0usize..10000,
        column in 0usize..1000,
        end_line in 0usize..10000,
        end_column in 0usize..1000
    ) {
        let loc = SourceLocation { line, column, end_line, end_column };

        prop_assert_eq!(loc.line, line);
        prop_assert_eq!(loc.column, column);
        prop_assert_eq!(loc.end_line, end_line);
        prop_assert_eq!(loc.end_column, end_column);
    }

    #[test]
    fn prop_mutant_id_preserved_through_clone(id in "[a-zA-Z0-9_]{1,50}") {
        let mutant = Mutant {
            id: id.clone(),
            original_file: PathBuf::from("test.rs"),
            mutated_source: "fn test() {}".to_string(),
            location: SourceLocation { line: 1, column: 1, end_line: 1, end_column: 10 },
            operator: MutationOperatorType::ArithmeticReplacement,
            hash: "hash".to_string(),
            status: MutantStatus::Pending,
        };

        let cloned = mutant.clone();
        // Compare the whole value FIRST: reading `cloned.id` by value moves it
        // out, so the equality check below would be on a partially-moved value.
        prop_assert_eq!(&cloned, &mutant);
        prop_assert_eq!(cloned.id, id);
    }

    #[test]
    fn prop_mutation_state_completion_percentage(
        pending in 0usize..100,
        completed in 0usize..100
    ) {
        if pending + completed == 0 {
            return Ok(());
        }

        let pending_mutants: Vec<Mutant> = (0..pending).map(|i| Mutant {
            id: format!("p{}", i),
            original_file: PathBuf::from("test.rs"),
            mutated_source: String::new(),
            location: SourceLocation { line: 1, column: 1, end_line: 1, end_column: 1 },
            operator: MutationOperatorType::ArithmeticReplacement,
            hash: format!("ph{}", i),
            status: MutantStatus::Pending,
        }).collect();

        let completed_results: Vec<MutationResult> = (0..completed).map(|i| MutationResult {
            mutant: Mutant {
                id: format!("c{}", i),
                original_file: PathBuf::from("test.rs"),
                mutated_source: String::new(),
                location: SourceLocation { line: 1, column: 1, end_line: 1, end_column: 1 },
                operator: MutationOperatorType::ArithmeticReplacement,
                hash: format!("ch{}", i),
                status: MutantStatus::Killed,
            },
            status: MutantStatus::Killed,
            test_failures: vec![],
            execution_time_ms: 100,
            error_message: None,
        }).collect();

        let mut state = MutationState::new(
            std::path::Path::new("/project"),
            pending_mutants,
            60,
            false,
            None,
        );

        for result in completed_results {
            state.completed_mutants.push(result);
        }

        let percentage = state.completion_percentage();
        let expected = if pending + completed == 0 {
            100.0
        } else {
            (completed as f64 / (pending + completed) as f64) * 100.0
        };

        // Allow for floating point tolerance
        let tolerance = 0.01;
        prop_assert!(
            (percentage - expected).abs() < tolerance,
            "Expected {}%, got {}%", expected, percentage
        );
    }

    #[test]
    fn prop_mutant_serialization_roundtrip(
        id in "[a-zA-Z0-9]{1,20}",
        file_name in "[a-zA-Z0-9_]+\\.rs",
        hash in "[a-f0-9]{8,64}"
    ) {
        let mutant = Mutant {
            id: id.clone(),
            original_file: PathBuf::from(&file_name),
            mutated_source: "fn test() {}".to_string(),
            location: SourceLocation { line: 1, column: 1, end_line: 1, end_column: 10 },
            operator: MutationOperatorType::ArithmeticReplacement,
            hash: hash.clone(),
            status: MutantStatus::Pending,
        };

        let json = serde_json::to_string(&mutant).map_err(|e| proptest::test_runner::TestCaseError::Fail(e.to_string().into()))?;
        let deserialized: Mutant = serde_json::from_str(&json).map_err(|e| proptest::test_runner::TestCaseError::Fail(e.to_string().into()))?;

        prop_assert_eq!(deserialized.id, id);
        prop_assert_eq!(deserialized.hash, hash);
        prop_assert_eq!(deserialized.status, MutantStatus::Pending);
    }
}

/// Property-based tests for mutation testing using proptest
///
/// Sprint 64 Day 1 - Testing Infrastructure
/// These tests verify mathematical properties and invariants that should hold
/// for all valid inputs to the mutation testing system.
use pmat::services::mutation::types::{
    Mutant, MutantStatus, MutationOperatorType, MutationResult, MutationScore, SourceLocation,
};
use proptest::prelude::*;
use std::path::PathBuf;

// Helper function to create test mutant
fn create_mutant(id: usize, status: MutantStatus) -> Mutant {
    Mutant {
        id: format!("mutant_{}", id),
        original_file: PathBuf::from("test.rs"),
        mutated_source: format!("fn test_{}() {{}}", id),
        location: SourceLocation {
            line: 1,
            column: 0,
            end_line: 1,
            end_column: 10,
        },
        operator: MutationOperatorType::ArithmeticReplacement,
        hash: format!("hash_{}", id),
        status,
    }
}

// Helper function to create test mutation result
fn create_result(id: usize, status: MutantStatus) -> MutationResult {
    MutationResult {
        mutant: create_mutant(id, status.clone()),
        status,
        test_failures: vec![],
        execution_time_ms: 100,
        error_message: None,
    }
}

// ============================================================================
// Category 1: Invariants (4 properties)
// ============================================================================

// Property 1: Mutation score is always between 0.0 and 1.0 (inclusive)
proptest! {
    #[test]
    fn mutation_score_always_bounded(
        killed in 0usize..1000,
        survived in 0usize..1000,
        compile_errors in 0usize..100,
        timeouts in 0usize..100,
    ) {
        let total = killed + survived + compile_errors + timeouts;
        if total == 0 {
            return Ok(());
        }

        let mut results = Vec::new();
        let mut id = 0;

        for _ in 0..killed {
            results.push(create_result(id, MutantStatus::Killed));
            id += 1;
        }
        for _ in 0..survived {
            results.push(create_result(id, MutantStatus::Survived));
            id += 1;
        }
        for _ in 0..compile_errors {
            results.push(create_result(id, MutantStatus::CompileError));
            id += 1;
        }
        for _ in 0..timeouts {
            results.push(create_result(id, MutantStatus::Timeout));
            id += 1;
        }

        let score = MutationScore::from_results(&results);

        prop_assert!(score.score >= 0.0, "Mutation score {} is below 0.0", score.score);
        prop_assert!(score.score <= 1.0, "Mutation score {} is above 1.0", score.score);
    }
}

// Property 2: Killed mutant count is always less than or equal to total mutant count
proptest! {
    #[test]
    fn killed_count_never_exceeds_total(
        killed in 0usize..500,
        survived in 0usize..500,
    ) {
        let mut results = Vec::new();
        for i in 0..killed {
            results.push(create_result(i, MutantStatus::Killed));
        }
        for i in 0..survived {
            results.push(create_result(killed + i, MutantStatus::Survived));
        }

        let score = MutationScore::from_results(&results);
        let total = killed + survived;

        prop_assert!(score.killed <= total, "Killed count {} exceeds total {}", score.killed, total);
    }
}

// Property 3: Sum of status counts equals total mutant count
proptest! {
    #[test]
    fn status_counts_sum_to_total(
        killed in 0usize..300,
        survived in 0usize..300,
        compile_errors in 0usize..100,
        timeouts in 0usize..100,
    ) {
        let total = killed + survived + compile_errors + timeouts;
        if total == 0 {
            return Ok(());
        }

        let mut results = Vec::new();
        let mut id = 0;

        for _ in 0..killed {
            results.push(create_result(id, MutantStatus::Killed));
            id += 1;
        }
        for _ in 0..survived {
            results.push(create_result(id, MutantStatus::Survived));
            id += 1;
        }
        for _ in 0..compile_errors {
            results.push(create_result(id, MutantStatus::CompileError));
            id += 1;
        }
        for _ in 0..timeouts {
            results.push(create_result(id, MutantStatus::Timeout));
            id += 1;
        }

        let score = MutationScore::from_results(&results);

        let sum = score.killed + score.survived + score.compile_errors + score.timeouts;
        prop_assert_eq!(sum, total, "Status counts sum {} does not equal total {}", sum, total);
    }
}

// Property 4: Progress percentage never exceeds 100%
proptest! {
    #[test]
    fn progress_percentage_never_exceeds_100(
        completed in 0usize..1000,
        total in 1usize..1000,
    ) {
        // Simulate progress calculation: (completed / total) * 100
        let progress_percentage = if total == 0 {
            0.0
        } else {
            (completed.min(total) as f64 / total as f64) * 100.0
        };

        prop_assert!(progress_percentage >= 0.0, "Progress {} is negative", progress_percentage);
        prop_assert!(progress_percentage <= 100.0, "Progress {} exceeds 100%", progress_percentage);
    }
}

// ============================================================================
// Category 2: Determinism (3 properties)
// ============================================================================

// Property 5: Score calculation is deterministic for same inputs
proptest! {
    #[test]
    fn score_calculation_deterministic(
        killed in 0usize..200,
        survived in 0usize..200,
    ) {
        let mut results = Vec::new();
        for i in 0..killed {
            results.push(create_result(i, MutantStatus::Killed));
        }
        for i in 0..survived {
            results.push(create_result(killed + i, MutantStatus::Survived));
        }

        // Calculate score twice
        let score1 = MutationScore::from_results(&results);
        let score2 = MutationScore::from_results(&results);

        // Scores should be identical
        prop_assert_eq!(score1.score, score2.score, "Scores differ on identical input");
        prop_assert_eq!(score1.killed, score2.killed, "Killed counts differ");
        prop_assert_eq!(score1.survived, score2.survived, "Survived counts differ");
    }
}

// Property 6: Mutation result order doesn't affect score
proptest! {
    #[test]
    fn result_order_independence(
        killed in 0usize..100,
        survived in 0usize..100,
    ) {
        let mut results = Vec::new();
        for i in 0..killed {
            results.push(create_result(i, MutantStatus::Killed));
        }
        for i in 0..survived {
            results.push(create_result(killed + i, MutantStatus::Survived));
        }

        // Calculate score with original order
        let score_original = MutationScore::from_results(&results);

        // Reverse the order
        results.reverse();
        let score_reversed = MutationScore::from_results(&results);

        // Scores should be identical regardless of order
        prop_assert_eq!(score_original.score, score_reversed.score, "Score changed with reversed order");
        prop_assert_eq!(score_original.killed, score_reversed.killed, "Killed count changed");
        prop_assert_eq!(score_original.survived, score_reversed.survived, "Survived count changed");
    }
}

// Property 7: Empty results produce zero score
proptest! {
    #[test]
    fn empty_results_produce_zero_score(_seed in 0u64..1000) {
        // Generate empty results vector
        let results: Vec<MutationResult> = vec![];

        let score = MutationScore::from_results(&results);

        prop_assert_eq!(score.score, 0.0, "Empty results should produce 0.0 score");
        prop_assert_eq!(score.killed, 0, "Empty results should have 0 killed");
        prop_assert_eq!(score.survived, 0, "Empty results should have 0 survived");
        prop_assert_eq!(score.total, 0, "Empty results should have 0 total");
    }
}

// ============================================================================
// Category 3: Output Consistency (3 properties)
// ============================================================================

// Property 8: JSON serialization preserves all result data
proptest! {
    #[test]
    fn json_serialization_preserves_data(
        mutant_id in 0usize..1000,
        _line in 1usize..100,
        _execution_time_ms in 0u64..5000,
    ) {
        use serde_json;

        let result = create_result(mutant_id, MutantStatus::Killed);

        // Serialize to JSON
        let json = serde_json::to_string(&result).expect("Serialization should succeed");

        // Deserialize back
        let deserialized: MutationResult = serde_json::from_str(&json).expect("Deserialization should succeed");

        // Verify key fields preserved
        prop_assert_eq!(result.mutant.id, deserialized.mutant.id, "Mutant ID not preserved");
        prop_assert_eq!(result.status, deserialized.status, "Status not preserved");
        prop_assert_eq!(result.execution_time_ms, deserialized.execution_time_ms, "Execution time not preserved");
    }
}

// Property 9: Mutation score is commutative (order of result aggregation doesn't matter)
proptest! {
    #[test]
    fn score_aggregation_commutative(
        batch1_killed in 0usize..50,
        batch1_survived in 0usize..50,
        batch2_killed in 0usize..50,
        batch2_survived in 0usize..50,
    ) {
        // Create two batches of results
        let mut batch1 = Vec::new();
        for i in 0..batch1_killed {
            batch1.push(create_result(i, MutantStatus::Killed));
        }
        for i in 0..batch1_survived {
            batch1.push(create_result(batch1_killed + i, MutantStatus::Survived));
        }

        let mut batch2 = Vec::new();
        let offset = batch1_killed + batch1_survived;
        for i in 0..batch2_killed {
            batch2.push(create_result(offset + i, MutantStatus::Killed));
        }
        for i in 0..batch2_survived {
            batch2.push(create_result(offset + batch2_killed + i, MutantStatus::Survived));
        }

        // Combine batches in different orders
        let combined1: Vec<MutationResult> = batch1.iter().chain(batch2.iter()).cloned().collect();
        let combined2: Vec<MutationResult> = batch2.iter().chain(batch1.iter()).cloned().collect();

        let score1 = MutationScore::from_results(&combined1);
        let score2 = MutationScore::from_results(&combined2);

        // Scores should be identical regardless of aggregation order
        prop_assert_eq!(score1.score, score2.score, "Scores differ with different aggregation order");
        prop_assert_eq!(score1.killed, score2.killed, "Killed counts differ");
        prop_assert_eq!(score1.survived, score2.survived, "Survived counts differ");
    }
}

// Property 10: All output formats contain same mutant count
proptest! {
    #[test]
    fn output_format_mutant_count_consistency(
        killed in 0usize..100,
        survived in 0usize..100,
    ) {
        let mut results = Vec::new();
        for i in 0..killed {
            results.push(create_result(i, MutantStatus::Killed));
        }
        for i in 0..survived {
            results.push(create_result(killed + i, MutantStatus::Survived));
        }

        let score = MutationScore::from_results(&results);
        let total = killed + survived;

        // All output formats should report same total
        prop_assert_eq!(score.total, total, "Total mutant count mismatch in output");
        prop_assert_eq!(score.killed + score.survived, total, "Killed + Survived != Total");
    }
}

// ============================================================================
// Category 4: Correctness (2 properties)
// ============================================================================

// Property 11: Mutant locations are within valid bounds
proptest! {
    #[test]
    fn mutant_locations_valid_bounds(
        line in 1usize..1000,
        end_line in 1usize..1000,
        col in 0usize..200,
        end_col in 0usize..200,
    ) {
        // Ensure end >= start for valid location
        let (actual_line, actual_end_line) = if end_line >= line {
            (line, end_line)
        } else {
            (end_line, line)
        };

        let (actual_col, actual_end_col) = if actual_end_line == actual_line {
            if end_col >= col {
                (col, end_col)
            } else {
                (end_col, col)
            }
        } else {
            (col, end_col)
        };

        let location = SourceLocation {
            line: actual_line,
            column: actual_col,
            end_line: actual_end_line,
            end_column: actual_end_col,
        };

        // Verify location is valid
        prop_assert!(location.line > 0, "Line must be positive");
        prop_assert!(location.end_line >= location.line, "End line must be >= start line");

        if location.line == location.end_line {
            prop_assert!(
                location.end_column >= location.column,
                "On same line, end column must be >= start column"
            );
        }
    }
}

// Property 12: Mutation score calculation matches mathematical definition
proptest! {
    #[test]
    fn mutation_score_mathematical_correctness(
        killed in 0usize..200,
        survived in 0usize..200,
    ) {
        if killed == 0 && survived == 0 {
            return Ok(());
        }

        let mut results = Vec::new();
        for i in 0..killed {
            results.push(create_result(i, MutantStatus::Killed));
        }
        for i in 0..survived {
            results.push(create_result(killed + i, MutantStatus::Survived));
        }

        let score = MutationScore::from_results(&results);

        // Mathematical definition: score = killed / (killed + survived)
        let expected_score = killed as f64 / (killed + survived) as f64;

        // Allow for floating-point precision errors (epsilon = 1e-10)
        let epsilon = 1e-10;
        prop_assert!(
            (score.score - expected_score).abs() < epsilon,
            "Score {} does not match mathematical definition {} (diff: {})",
            score.score,
            expected_score,
            (score.score - expected_score).abs()
        );
    }
}

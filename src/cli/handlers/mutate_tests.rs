#[cfg_attr(coverage_nightly, coverage(off))]
#[cfg(test)]
mod tests {
    use super::*;
    use crate::services::mutation::types::{
        Mutant, MutantStatus, MutationOperatorType, SourceLocation,
    };
    use tempfile::TempDir;

    fn create_temp_rust_file() -> (TempDir, std::path::PathBuf) {
        let temp = TempDir::new().unwrap();
        let file_path = temp.path().join("test.rs");
        std::fs::write(
            &file_path,
            r#"fn main() {
    let x = 5;
    let y = 10;
    println!("{}", x + y);
}

fn add(a: i32, b: i32) -> i32 {
    a + b
}
"#,
        )
        .unwrap();
        (temp, file_path)
    }

    fn create_test_mutant(file_path: &Path, line: usize) -> Mutant {
        Mutant {
            id: format!("mutant_{}", line),
            original_file: file_path.to_path_buf(),
            location: SourceLocation {
                line,
                column: 1,
                end_line: line,
                end_column: 10,
            },
            operator: MutationOperatorType::ArithmeticReplacement,
            // `original_source` was dropped from `Mutant`: it now carries
            // `original_file` + `location` and the mutated text only. There is
            // no successor field, so the assertion on it goes too rather than
            // being retargeted at something that does not mean the same thing.
            mutated_source: "a - b".to_string(),
            // `hash` (dedup key) and `status` were added to `Mutant`; a
            // freshly-built fixture has not been executed yet.
            hash: "test-hash".to_string(),
            status: MutantStatus::Pending,
        }
    }

    fn create_test_mutation_result(file_path: &Path, status: MutantStatus) -> MutationResult {
        MutationResult {
            mutant: create_test_mutant(file_path, 8),
            status,
            execution_time_ms: 100,
            // Was `test_output: Some("test output")`. The current model names
            // this precisely — "test failures that killed this mutant" — so a
            // Killed result carries the evidence that killed it.
            test_failures: vec!["tests::arithmetic_is_checked".to_string()],
            error_message: None,
        }
    }

    // ============================================================================
    // extract_code_snippet tests
    // ============================================================================

    #[test]
    fn test_extract_code_snippet_success() {
        let (temp, file_path) = create_temp_rust_file();

        let location = SourceLocation {
            line: 1,
            column: 1,
            end_line: 2,
            end_column: 10,
        };

        let snippet = extract_code_snippet(&file_path, &location).unwrap();
        assert!(snippet.contains("fn main()"));
        drop(temp);
    }

    #[test]
    fn test_extract_code_snippet_single_line() {
        let (temp, file_path) = create_temp_rust_file();

        let location = SourceLocation {
            line: 7,
            column: 1,
            end_line: 7,
            end_column: 30,
        };

        let snippet = extract_code_snippet(&file_path, &location).unwrap();
        assert!(!snippet.is_empty());
        drop(temp);
    }

    #[test]
    fn test_extract_code_snippet_out_of_bounds() {
        let (temp, file_path) = create_temp_rust_file();

        let location = SourceLocation {
            line: 1000,
            column: 1,
            end_line: 1001,
            end_column: 10,
        };

        let snippet = extract_code_snippet(&file_path, &location).unwrap();
        assert!(snippet.contains("<code location out of bounds>"));
        drop(temp);
    }

    #[test]
    fn test_extract_code_snippet_file_not_found() {
        let location = SourceLocation {
            line: 1,
            column: 1,
            end_line: 2,
            end_column: 10,
        };

        let result = extract_code_snippet(Path::new("/nonexistent/file.rs"), &location);
        assert!(result.is_err());
    }

    // ============================================================================
    // print_progress tests
    // ============================================================================

    #[test]
    fn test_print_progress_zero_total() {
        // Should not panic with zero total
        print_progress(0, 0);
    }

    #[test]
    fn test_print_progress_partial() {
        print_progress(5, 10);
    }

    #[test]
    fn test_print_progress_complete() {
        print_progress(10, 10);
    }

    // ============================================================================
    // MutationScore tests
    // ============================================================================

    #[test]
    fn test_mutation_score_from_empty_results() {
        let results: Vec<MutationResult> = vec![];
        let score = MutationScore::from_results(&results);
        assert_eq!(score.total, 0);
        assert_eq!(score.killed, 0);
    }

    #[test]
    fn test_mutation_score_from_results_killed() {
        let (temp, file_path) = create_temp_rust_file();
        let results = vec![create_test_mutation_result(
            &file_path,
            MutantStatus::Killed,
        )];

        let score = MutationScore::from_results(&results);
        assert_eq!(score.total, 1);
        assert_eq!(score.killed, 1);
        assert_eq!(score.survived, 0);
        drop(temp);
    }

    #[test]
    fn test_mutation_score_from_results_survived() {
        let (temp, file_path) = create_temp_rust_file();
        let results = vec![create_test_mutation_result(
            &file_path,
            MutantStatus::Survived,
        )];

        let score = MutationScore::from_results(&results);
        assert_eq!(score.survived, 1);
        drop(temp);
    }

    #[test]
    fn test_mutation_score_from_results_mixed() {
        let (temp, file_path) = create_temp_rust_file();
        let results = vec![
            create_test_mutation_result(&file_path, MutantStatus::Killed),
            create_test_mutation_result(&file_path, MutantStatus::Survived),
            create_test_mutation_result(&file_path, MutantStatus::CompileError),
            create_test_mutation_result(&file_path, MutantStatus::Timeout),
        ];

        let score = MutationScore::from_results(&results);
        assert_eq!(score.total, 4);
        assert_eq!(score.killed, 1);
        assert_eq!(score.survived, 1);
        assert_eq!(score.compile_errors, 1);
        assert_eq!(score.timeouts, 1);
        drop(temp);
    }

    // ============================================================================
    // EnhancedMutationResult tests
    // ============================================================================

    #[test]
    fn test_enhanced_mutation_result_serialization() {
        let (temp, file_path) = create_temp_rust_file();
        let result = create_test_mutation_result(&file_path, MutantStatus::Killed);

        let enhanced = EnhancedMutationResult {
            result,
            original_code_snippet: Some("a + b".to_string()),
            mutated_code_snippet: Some("a - b".to_string()),
        };

        let json = serde_json::to_string(&enhanced).unwrap();
        assert!(json.contains("a + b"));
        assert!(json.contains("a - b"));
        drop(temp);
    }

    // ============================================================================
    // MutationTestOutput tests
    // ============================================================================

    #[test]
    fn test_mutation_test_output_serialization() {
        let score = MutationScore {
            total: 10,
            killed: 8,
            survived: 2,
            compile_errors: 0,
            timeouts: 0,
            equivalent: 0,
            score: 0.8,
        };

        let output = MutationTestOutput {
            score,
            results: vec![],
        };

        let json = serde_json::to_string(&output).unwrap();
        assert!(json.contains("\"total\":10"));
        assert!(json.contains("\"killed\":8"));
        assert!(json.contains("\"score\":0.8"));
    }

    // ============================================================================
    // SourceLocation tests
    // ============================================================================

    #[test]
    fn test_source_location_creation() {
        let location = SourceLocation {
            line: 10,
            column: 5,
            end_line: 12,
            end_column: 20,
        };

        assert_eq!(location.line, 10);
        assert_eq!(location.column, 5);
        assert_eq!(location.end_line, 12);
        assert_eq!(location.end_column, 20);
    }

    // ============================================================================
    // Mutant tests
    // ============================================================================

    #[test]
    fn test_mutant_creation() {
        let (temp, file_path) = create_temp_rust_file();
        let mutant = create_test_mutant(&file_path, 8);

        assert_eq!(mutant.mutated_source, "a - b");
        assert_eq!(mutant.operator, MutationOperatorType::ArithmeticReplacement);
        drop(temp);
    }

    // ============================================================================
    // MutationResult tests
    // ============================================================================

    #[test]
    fn test_mutation_result_creation() {
        let (temp, file_path) = create_temp_rust_file();
        let result = create_test_mutation_result(&file_path, MutantStatus::Killed);

        assert_eq!(result.status, MutantStatus::Killed);
        assert_eq!(result.execution_time_ms, 100);
        assert!(
            !result.test_failures.is_empty(),
            "a killed mutant must carry the failures that killed it"
        );
        drop(temp);
    }

    #[test]
    fn test_mutation_result_serialization() {
        let (temp, file_path) = create_temp_rust_file();
        let result = create_test_mutation_result(&file_path, MutantStatus::Survived);

        let json = serde_json::to_string(&result).unwrap();
        assert!(json.contains("Survived"));
        assert!(json.contains("execution_time_ms"));
        drop(temp);
    }

    // ============================================================================
    // Output format filtering tests
    // ============================================================================

    #[test]
    fn test_filter_failures_only_keeps_survived() {
        let (temp, file_path) = create_temp_rust_file();
        let results = vec![
            create_test_mutation_result(&file_path, MutantStatus::Killed),
            create_test_mutation_result(&file_path, MutantStatus::Survived),
        ];

        // Simulate failures_only filtering
        let filtered: Vec<_> = results
            .iter()
            .filter(|r| {
                matches!(
                    r.status,
                    MutantStatus::Survived | MutantStatus::CompileError | MutantStatus::Timeout
                )
            })
            .collect();

        assert_eq!(filtered.len(), 1);
        assert_eq!(filtered[0].status, MutantStatus::Survived);
        drop(temp);
    }

    #[test]
    fn test_filter_failures_only_keeps_compile_error() {
        let (temp, file_path) = create_temp_rust_file();
        let results = vec![
            create_test_mutation_result(&file_path, MutantStatus::Killed),
            create_test_mutation_result(&file_path, MutantStatus::CompileError),
        ];

        let filtered: Vec<_> = results
            .iter()
            .filter(|r| {
                matches!(
                    r.status,
                    MutantStatus::Survived | MutantStatus::CompileError | MutantStatus::Timeout
                )
            })
            .collect();

        assert_eq!(filtered.len(), 1);
        assert_eq!(filtered[0].status, MutantStatus::CompileError);
        drop(temp);
    }

    #[test]
    fn test_filter_failures_only_keeps_timeout() {
        let (temp, file_path) = create_temp_rust_file();
        let results = vec![
            create_test_mutation_result(&file_path, MutantStatus::Killed),
            create_test_mutation_result(&file_path, MutantStatus::Timeout),
        ];

        let filtered: Vec<_> = results
            .iter()
            .filter(|r| {
                matches!(
                    r.status,
                    MutantStatus::Survived | MutantStatus::CompileError | MutantStatus::Timeout
                )
            })
            .collect();

        assert_eq!(filtered.len(), 1);
        assert_eq!(filtered[0].status, MutantStatus::Timeout);
        drop(temp);
    }

    // ============================================================================
    // MutationOperator tests
    // ============================================================================

    #[test]
    fn test_mutation_operator_equality() {
        assert_eq!(
            MutationOperatorType::ArithmeticReplacement,
            MutationOperatorType::ArithmeticReplacement
        );
        assert_ne!(
            MutationOperatorType::ArithmeticReplacement,
            MutationOperatorType::RelationalReplacement
        );
    }

    #[test]
    fn test_mutant_status_equality() {
        assert_eq!(MutantStatus::Killed, MutantStatus::Killed);
        assert_ne!(MutantStatus::Killed, MutantStatus::Survived);
    }
}

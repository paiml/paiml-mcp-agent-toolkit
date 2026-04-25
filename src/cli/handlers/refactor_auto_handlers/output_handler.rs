// Output formatting for refactor-auto command.
// Split into include!() sub-files for file health compliance (CB-040).
//
// Sub-files share this module's scope (all imports from parent mod.rs).
// Each sub-file contains a logical grouping of related functionality.

// Type definitions: IterationResult, RefactoringSuccess, RefactoringFailure,
// VerificationStatus, ValidationResult, QualityImprovement, CompilationResult,
// TestResult, RefactoringSummary, IterationContinuation
include!("output_handler_types.rs");

// Iteration execution and validation: execute_refactoring_iteration,
// validate_refactoring_results, apply_refactoring_request,
// validate_project_compilation, validate_test_suite,
// calculate_quality_improvement, should_retry_refactoring, apply_* helpers
include!("output_handler_iteration.rs");

// Output formatting: format_and_output_results, output_json_results,
// output_markdown_results, output_text_results, create_refactoring_summary
include!("output_handler_formatting.rs");

// Main handler and orchestration: handle_refactor_auto, handle_single_file_refactor,
// initialize_refactoring_context, execute_refactoring_cycles, finalize_refactoring,
// and single-file analysis helpers
include!("output_handler_orchestration.rs");

// Tests extracted to refactor_auto_handlers_tests.rs for file health compliance (CB-040)
// TEMPORARILY DISABLED: Test file is missing
#[cfg(all(test, feature = "broken-tests"))]
#[path = "refactor_auto_handlers_tests.rs"]
mod tests;

#[cfg(test)]
mod output_handler_iteration_pure_tests {
    //! Covers should_retry_refactoring + apply_*_reduction stubs in
    //! output_handler_iteration.rs (207 uncov on broad, 0% cov).
    //! Skips async pipeline orchestration (execute_refactoring_iteration,
    //! validate_refactoring_results, etc.) which require RefactorContext
    //! + project fixtures.
    use super::*;

    // ── should_retry_refactoring: 4 keyword arms + miss ──

    #[test]
    fn test_should_retry_refactoring_timeout_keyword_true() {
        let err = anyhow::anyhow!("operation Timeout exceeded");
        assert!(should_retry_refactoring(&err));
    }

    #[test]
    fn test_should_retry_refactoring_network_keyword_true() {
        let err = anyhow::anyhow!("Network unreachable");
        assert!(should_retry_refactoring(&err));
    }

    #[test]
    fn test_should_retry_refactoring_temporary_keyword_true() {
        let err = anyhow::anyhow!("temporary failure");
        assert!(should_retry_refactoring(&err));
    }

    #[test]
    fn test_should_retry_refactoring_unrelated_error_false() {
        let err = anyhow::anyhow!("syntax error in file foo.rs");
        assert!(!should_retry_refactoring(&err));
    }

    #[test]
    fn test_should_retry_refactoring_case_insensitive() {
        let err = anyhow::anyhow!("Connection TIMEOUT after 30s");
        assert!(should_retry_refactoring(&err));
    }

    #[test]
    fn test_should_retry_refactoring_empty_error_message_false() {
        let err = anyhow::anyhow!("");
        assert!(!should_retry_refactoring(&err));
    }

    // ── apply_*_reduction stubs: 3 placeholder fns ──

    #[tokio::test]
    async fn test_apply_complexity_reduction_returns_two_messages() {
        let result = apply_complexity_reduction(std::path::Path::new("a.rs"), "instructions")
            .await
            .unwrap();
        assert_eq!(result.len(), 2);
        assert!(result.iter().any(|s| s.contains("Extracted")));
    }

    #[tokio::test]
    async fn test_apply_lint_fixes_returns_two_messages() {
        let result = apply_lint_fixes(std::path::Path::new("a.rs"), "instructions")
            .await
            .unwrap();
        assert_eq!(result.len(), 2);
        assert!(result.iter().any(|s| s.contains("clippy")));
    }

    #[tokio::test]
    async fn test_apply_satd_cleanup_returns_two_messages() {
        let result = apply_satd_cleanup(std::path::Path::new("a.rs"), "instructions")
            .await
            .unwrap();
        assert_eq!(result.len(), 2);
        assert!(result.iter().any(|s| s.contains("TODO")));
    }
}

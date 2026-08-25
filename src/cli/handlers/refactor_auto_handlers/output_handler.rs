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
// QUARANTINED, and the stated reason was wrong. This said "Test file is
// missing"; the file exists at src/cli/handlers/refactor_auto_handlers_tests.rs
// (40,150 bytes) — one directory ABOVE this one. The `#[path]` was wrong too,
// naming a sibling that does not exist; it has been corrected to `../` (#1023)
// so the declaration at least points at the file it is talking about. Nothing
// checks a `#[path]` under a disabled `cfg`, which is why it stayed wrong.
//
// The real reason it cannot be enabled: it `include!`s
// refactor_auto_comprehensive_tests.rs, whose CB-040 extraction left a module
// wrapper split across files — the parent opens a brace that the child closes.
// Correcting the path yields "unexpected closing delimiter"; deleting the
// orphaned brace yields "unclosed delimiter". Reviving it is a real repair of
// ~40 KB of tests, not a one-line fix, so it stays quarantined with an accurate
// note (#1023) rather than a false one.
//
// A wrong reason is worse than no reason: it tells the next reader to go looking
// for a file that is sitting right there.
#[cfg(all(test, feature = "broken-tests"))]
#[path = "../refactor_auto_handlers_tests.rs"]
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

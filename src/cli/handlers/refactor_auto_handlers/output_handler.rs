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

// Tests extracted to refactor_auto_handlers_tests.rs for file health compliance (CB-040).
//
// REVIVED (#1023). This was quarantined behind `feature = "broken-tests"` for two
// releases under the reason "Test file is missing". The file was never missing: it
// is one directory ABOVE this one, 40 KB, and the `#[path]` pointed at a
// nonexistent sibling. Nothing checks a `#[path]` under a disabled `cfg`.
//
// What actually broke it was two mechanical extractions, each of which dropped a
// line without anything noticing, because the module was already unbuildable:
//   * CB-040 (c8dd80a8e -> 9efe77e93) lifted `mod comprehensive_coverage_tests {`
//     into a `#[path]` declaration but left its closing `}` in the file, and lost
//     the `#[tokio::test]` attribute of the first test with it.
//   * PMAT-503 (9bedc46cf) sliced that file into five include!s and dropped the
//     closing brace of the last function in the first slice.
// Hence "unexpected closing delimiter" on the parent and "unclosed delimiter" once
// the orphan brace was deleted: the wrapper spanned files. Both lines are restored,
// and the five include!s now concatenate byte-for-byte to the last well-formed
// version of the module body (c8dd80a8e lines 1282-2928).
//
// A third break surfaced only once the files parsed: CB-040 (1008e33ec) moved seven
// markdown helpers into refactor_auto_types without importing them back, so
// `use super::*` stopped reaching them. They are now imported explicitly by the
// two test files that call them.
//
// Timeline, because the stated reason was only ever half right: the module was
// buildable at c8dd80a8e; 9efe77e93 split its wrapper across files the same day;
// 1008e33ec moved the helpers out; e044423ad turned this file into a directory
// without updating the `#[path]`, at which point rustc genuinely reported the
// module file as missing — which is what 129e67132 wrote down and gated. The file
// was never missing. The path was stale, and two older faults were queued behind
// it, invisible because rustc never got as far as reading the file.
#[cfg(test)]
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

#[cfg(test)]
mod refactor_auto_test_module_structure_tests {
    //! #1023 guards for the module revived above.
    //!
    //! These read the seven files as TEXT rather than compiling them, so they
    //! keep running if the module is ever `cfg`'d out again. That is the point:
    //! while it was quarantined, everything about it was unchecked — a wrong
    //! `#[path]`, an orphaned brace and a lost `#[tokio::test]` all survived two
    //! releases because no build ever read the files.

    use std::path::{Path, PathBuf};

    /// The seven files `mod tests` spans after CB-040 and PMAT-503.
    const SPLIT_FILES: [&str; 7] = [
        "refactor_auto_handlers_tests.rs",
        "refactor_auto_comprehensive_tests.rs",
        "refactor_auto_comprehensive_tests_setup_context.rs",
        "refactor_auto_comprehensive_tests_analysis_generation.rs",
        "refactor_auto_comprehensive_tests_request_creation.rs",
        "refactor_auto_comprehensive_tests_apply_helpers_output.rs",
        "refactor_auto_comprehensive_tests_modes_validation_types.rs",
    ];

    /// Test attributes across `SPLIT_FILES` when the module was revived.
    ///
    /// A floor, never a target: it may only go up. It exists so that a future
    /// compile error in this module cannot be resolved by deleting the tests
    /// that produced it, which is indistinguishable from a repair at the level
    /// of "does the build pass".
    const TEST_ATTRIBUTE_FLOOR: usize = 154;

    fn handlers_dir() -> PathBuf {
        Path::new(env!("CARGO_MANIFEST_DIR")).join("src/cli/handlers")
    }

    fn read(name: &str) -> String {
        let path = handlers_dir().join(name);
        std::fs::read_to_string(&path)
            .map_err(|e| format!("{} is unreadable: {e}", path.display()))
            .expect("every file listed in SPLIT_FILES must exist")
    }

    /// Every file the module is split across parses on its own.
    ///
    /// `include!` requires each file to be a self-contained sequence of items.
    /// CB-040 lifted `mod comprehensive_coverage_tests {` into a `#[path]`
    /// declaration and left its `}` behind; PMAT-503 then dropped the closing
    /// brace of the last function in the first slice. Neither is visible to a
    /// brace count that trusts comments and string literals, and neither was
    /// visible to rustc while the module was quarantined.
    #[test]
    fn every_split_file_parses_standalone() {
        let mut broken = Vec::new();
        for name in SPLIT_FILES {
            if let Err(e) = syn::parse_file(&read(name)) {
                broken.push(format!("{name}: {e}"));
            }
        }
        assert!(
            broken.is_empty(),
            "these files do not parse on their own, so the module wrapper is \
             split across files again:\n  {}\n\n\
             Re-splitting a module must never leave one file opening a delimiter \
             that another closes.",
            broken.join("\n  ")
        );
    }

    /// The revived module is still compiled.
    ///
    /// Re-adding the `broken-tests` gate would silently stop running every test
    /// in these files while leaving them in the tree, where every file-based
    /// metric keeps counting them.
    #[test]
    fn the_revived_module_is_not_quarantined_again() {
        let me = read("refactor_auto_handlers/output_handler.rs");
        let declaration = me
            .lines()
            .position(|l| l.trim() == r#"#[path = "../refactor_auto_handlers_tests.rs"]"#)
            .expect("output_handler.rs must still declare mod tests by #[path]");
        let gate = declaration
            .checked_sub(1)
            .and_then(|i| me.lines().nth(i))
            .expect("the #[path] must be preceded by its cfg");
        assert_eq!(
            gate.trim(),
            "#[cfg(test)]",
            "mod tests is gated by {gate:?} rather than a plain #[cfg(test)] — \
             it has been quarantined again (#1023)"
        );
    }

    /// The revival cannot be undone by deleting the tests that failed.
    #[test]
    fn the_revived_module_still_carries_its_tests() {
        let mut found = 0usize;
        for name in SPLIT_FILES {
            found += read(name)
                .lines()
                .filter(|l| {
                    let t = l.trim();
                    t == "#[test]" || t == "#[tokio::test]"
                })
                .count();
        }
        assert!(
            found >= TEST_ATTRIBUTE_FLOOR,
            "{found} test attributes across the revived module, against a floor \
             of {TEST_ATTRIBUTE_FLOOR}. Tests were removed rather than repaired."
        );
    }
}

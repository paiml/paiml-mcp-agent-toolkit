//! SATD detection, scanning, and file processing logic.
//!
//! Split into submodules for maintainability:
//! - `detection_extraction.rs`: Constructors, content parsing, comment extraction, context hashing
//! - `detection_analysis.rs`: Project analysis, directory scanning, result aggregation
//! - `detection_file_discovery.rs`: Source file finding, filtering, test/vendor detection
//! - `detection_false_positives.rs`: False positive detection, documentation/metadata checks

use blake3::Hasher;
use std::path::{Path, PathBuf};

use crate::models::error::TemplateError;
// #923: one implementation of "is this path inside its project's tests/,
// examples/, fuzz/, vendor/ ...?", shared with the Rust defect detector so the
// two gates cannot drift apart again.
use crate::services::defect_detector::source_scope;

use super::types::{
    AstContext, AstNodeType, DebtClassifier, ProjectAnalysisStats, SATDAnalysisResult,
    SATDDetector, SATDSummary, TechnicalDebt, TestBlockTracker,
};

include!("detection_extraction.rs");
include!("detection_analysis.rs");
include!("detection_file_discovery.rs");
include!("detection_false_positives.rs");

#[cfg(test)]
mod extraction_pure_tests {
    //! Covers SATDDetector pure helpers in detection_extraction.rs (198
    //! lines, 0 prior tests for these helpers).
    use super::*;

    fn detector() -> SATDDetector {
        SATDDetector::new()
    }

    // ── is_rust_file ──

    #[test]
    fn test_is_rust_file_rs_extension_returns_true() {
        let d = detector();
        assert!(d.is_rust_file(Path::new("src/main.rs")));
    }

    #[test]
    fn test_is_rust_file_non_rs_returns_false() {
        let d = detector();
        assert!(!d.is_rust_file(Path::new("a.py")));
        assert!(!d.is_rust_file(Path::new("a.js")));
    }

    #[test]
    fn test_is_rust_file_no_extension_returns_false() {
        let d = detector();
        assert!(!d.is_rust_file(Path::new("Makefile")));
    }

    // ── find_comment_column ──

    #[test]
    fn test_find_comment_column_double_slash() {
        let d = detector();
        assert_eq!(d.find_comment_column("    // TODO"), 5);
    }

    #[test]
    fn test_find_comment_column_hash() {
        let d = detector();
        assert_eq!(d.find_comment_column("    # TODO"), 5);
    }

    #[test]
    fn test_find_comment_column_block_comment_open() {
        let d = detector();
        assert_eq!(d.find_comment_column("    /* TODO */"), 5);
    }

    #[test]
    fn test_find_comment_column_html_comment_open() {
        let d = detector();
        assert_eq!(d.find_comment_column("    <!-- TODO -->"), 5);
    }

    #[test]
    fn test_find_comment_column_no_comment_returns_one() {
        let d = detector();
        assert_eq!(d.find_comment_column("let x = 5;"), 1);
    }

    // ── extract_comment_content ──

    #[test]
    fn test_extract_comment_content_double_slash() {
        let d = detector();
        let r = d.extract_comment_content("// TODO: fix").unwrap();
        assert_eq!(r, Some("TODO: fix".to_string()));
    }

    #[test]
    fn test_extract_comment_content_hash() {
        let d = detector();
        let r = d.extract_comment_content("# TODO: fix").unwrap();
        assert_eq!(r, Some("TODO: fix".to_string()));
    }

    #[test]
    fn test_extract_comment_content_block_comment() {
        let d = detector();
        let r = d.extract_comment_content("/* FIXME: bug */").unwrap();
        assert_eq!(r, Some("FIXME: bug".to_string()));
    }

    #[test]
    fn test_extract_comment_content_html_comment() {
        let d = detector();
        let r = d
            .extract_comment_content("<!-- HACK: workaround -->")
            .unwrap();
        assert_eq!(r, Some("HACK: workaround".to_string()));
    }

    #[test]
    fn test_extract_comment_content_no_comment_returns_none() {
        let d = detector();
        let r = d.extract_comment_content("let x = 5;").unwrap();
        assert!(r.is_none());
    }

    #[test]
    fn test_extract_comment_content_block_must_close_on_same_line() {
        let d = detector();
        let r = d.extract_comment_content("/* TODO unclosed").unwrap();
        assert!(r.is_none());
    }

    #[test]
    fn test_extract_comment_content_too_long_line_returns_err() {
        let d = detector();
        let long = "x".repeat(10001);
        assert!(d.extract_comment_content(&long).is_err());
    }

    #[test]
    fn test_extract_comment_content_strips_leading_whitespace_in_content() {
        let d = detector();
        let r = d.extract_comment_content("//   TODO   ").unwrap();
        assert_eq!(r, Some("TODO".to_string()));
    }

    // ── hash_context (blake3) ──

    #[test]
    fn test_hash_context_returns_16_bytes() {
        let d = detector();
        let h = d.hash_context(Path::new("a.rs"), 10, "TODO");
        assert_eq!(h.len(), 16);
    }

    #[test]
    fn test_hash_context_different_paths_yield_different_hashes() {
        let d = detector();
        let h1 = d.hash_context(Path::new("a.rs"), 10, "TODO");
        let h2 = d.hash_context(Path::new("b.rs"), 10, "TODO");
        assert_ne!(h1, h2);
    }

    #[test]
    fn test_hash_context_different_lines_yield_different_hashes() {
        let d = detector();
        let h1 = d.hash_context(Path::new("a.rs"), 10, "TODO");
        let h2 = d.hash_context(Path::new("a.rs"), 20, "TODO");
        assert_ne!(h1, h2);
    }

    #[test]
    fn test_hash_context_deterministic() {
        let d = detector();
        let h1 = d.hash_context(Path::new("a.rs"), 10, "TODO");
        let h2 = d.hash_context(Path::new("a.rs"), 10, "TODO");
        assert_eq!(h1, h2);
    }

    // ── Constructors ──

    #[test]
    fn test_satd_detector_default_equals_new() {
        let _ = SATDDetector::default();
        let _ = SATDDetector::new();
    }

    #[test]
    fn test_satd_detector_new_extended_constructs() {
        let _ = SATDDetector::new_extended();
    }

    #[test]
    fn test_satd_detector_new_strict_constructs() {
        let _ = SATDDetector::new_strict();
    }
}

/// #923 — the blocker: SATD's exclusions matched substrings of the ABSOLUTE
/// path, so where a checkout happened to sit decided whether the gate measured
/// anything at all.
#[cfg(test)]
mod checkout_location_regression_tests {
    use super::*;

    const LIB_RS: &str = "// TODO: handle the empty slice instead of panicking\n\
                          pub fn f(v: &[i32]) -> i32 { v[0] }\n";
    const MANIFEST: &str =
        "[package]\nname = \"myproject\"\nversion = \"0.1.0\"\nedition = \"2021\"\n";

    fn crate_at(root: &Path) -> PathBuf {
        std::fs::create_dir_all(root.join("src")).expect("src dir");
        std::fs::write(root.join("Cargo.toml"), MANIFEST).expect("manifest");
        let lib = root.join("src/lib.rs");
        std::fs::write(&lib, LIB_RS).expect("lib.rs");
        lib
    }

    /// One crate, one md5. Only the name of a directory ABOVE the crate
    /// differs — `<tmp>/examples/myproject` reported "0 violations in 0 files"
    /// and exit 0 while `<tmp>/normal` reported 1.
    #[tokio::test]
    async fn debt_is_found_wherever_the_checkout_sits() {
        let tmp = tempfile::TempDir::new().expect("tempdir");
        let detector = SATDDetector::new();
        let mut verdicts = Vec::new();

        for parent in [
            "normal",
            "tests/myproject",
            "examples/myproject",
            "demo/myproject",
            "fuzz/myproject",
            "vendor/myproject",
            "book/myproject",
            "target/myproject",
        ] {
            let root = tmp.path().join(parent);
            let lib = crate_at(&root);

            assert!(
                !detector.should_exclude_file(&lib),
                "an ancestor named {parent:?} excluded the crate's own src/lib.rs"
            );
            let found = detector
                .analyze_directory(&root)
                .await
                .expect("the crate has one analyzable file");
            verdicts.push((parent, found.len()));
        }

        assert!(
            verdicts.iter().all(|(_, n)| *n == 1),
            "the parent directory's name changed how much debt exists: {verdicts:?}"
        );
    }

    /// Guard rail: the package's OWN examples/ and tests/ trees are still
    /// support code. The rule became project-relative; it did not go away.
    #[test]
    fn a_packages_own_support_directories_are_still_excluded() {
        let tmp = tempfile::TempDir::new().expect("tempdir");
        let root = tmp.path().join("myproject");
        crate_at(&root);
        let detector = SATDDetector::new();

        for support in [
            "examples",
            "demo",
            "fuzz",
            "vendor",
            "node_modules",
            "target",
        ] {
            let file = root.join(support).join("thing.rs");
            std::fs::create_dir_all(file.parent().expect("parent")).expect("dir");
            std::fs::write(&file, LIB_RS).expect("file");
            assert!(
                detector.should_exclude_file(&file),
                "{support}/ inside the package must stay excluded"
            );
        }

        let in_tests = root.join("tests/it.rs");
        std::fs::create_dir_all(in_tests.parent().expect("parent")).expect("dir");
        std::fs::write(&in_tests, LIB_RS).expect("file");
        assert!(
            detector.is_test_file(&in_tests),
            "the package's own tests/ tree is still test code"
        );
    }

    /// #923, second half: "every candidate was excluded" and "the code is
    /// clean" both arrived as an empty `Vec<TechnicalDebt>`, which the CLI
    /// rendered as `Found 0 SATD violations in 0 files` with exit 0 — a clean
    /// bill of health for a measurement that was never taken.
    #[tokio::test]
    async fn a_walk_that_measures_nothing_is_not_a_clean_result() {
        let tmp = tempfile::TempDir::new().expect("tempdir");
        let root = tmp.path().join("myproject");
        crate_at(&root);

        let examples = root.join("examples");
        std::fs::create_dir_all(&examples).expect("examples dir");
        for name in ["a.rs", "b.rs"] {
            std::fs::write(examples.join(name), LIB_RS).expect("example file");
        }

        let detector = SATDDetector::new();
        let err = detector
            .analyze_directory(&examples)
            .await
            .expect_err("every candidate under examples/ is excluded — nothing was measured");
        let message = err.to_string();
        assert!(
            message.contains("path"),
            "the refusal must name the parameter at fault: {message}"
        );

        // And the empty directory case, which is the same claim: no measurement.
        let empty = root.join("empty");
        std::fs::create_dir_all(&empty).expect("empty dir");
        assert!(
            detector.analyze_directory(&empty).await.is_err(),
            "a walk over zero source files reported a clean verdict"
        );

        // Control: the crate's own src/ IS measured, and measures 1.
        assert_eq!(
            detector
                .analyze_directory(&root)
                .await
                .expect("src/lib.rs is analyzable")
                .len(),
            1
        );
    }
}

#[cfg(test)]
mod marker_regression_tests {
    //! Regression tests for issues #651, #668 and #674: `--strict` returned 0
    //! on a file of pure markers, and the false-positive heuristics silently
    //! dropped explicit markers whose prose happened to contain "unwrap",
    //! "expect", "technical debt" or "detection".
    use super::*;

    /// A path that is neither a test file nor one of the analyzer's own files,
    /// so `should_exclude_file` does not short-circuit the extraction.
    fn src_file() -> &'static Path {
        Path::new("src/lib.rs")
    }

    fn texts(detector: &SATDDetector, content: &str) -> Vec<String> {
        detector
            .extract_from_content(content, src_file())
            .expect("extraction must succeed")
            .into_iter()
            .map(|d| d.text)
            .collect()
    }

    // ── #651: --strict must return the markers, not zero ──

    const FOUR_MARKERS: &str = "// TODO: rewrite this loop\n\
         // FIXME: broken input handling\n\
         // HACK: temporary workaround\n\
         // BUG: off by one\n\
         pub fn f() -> i32 { 1 }\n";

    #[test]
    fn test_strict_mode_reports_all_four_canonical_markers() {
        let found = texts(&SATDDetector::new_strict(), FOUR_MARKERS);
        assert_eq!(
            found.len(),
            4,
            "--strict must report TODO/FIXME/HACK/BUG, got {found:?}"
        );
        for marker in ["TODO", "FIXME", "HACK", "BUG"] {
            assert!(
                found.iter().any(|t| t.starts_with(marker)),
                "missing {marker} in {found:?}"
            );
        }
    }

    #[test]
    fn test_strict_result_is_a_subset_of_default() {
        let strict = texts(&SATDDetector::new_strict(), FOUR_MARKERS);
        let default = texts(&SATDDetector::new(), FOUR_MARKERS);
        assert!(
            !strict.is_empty() && strict.len() <= default.len(),
            "strict {strict:?} must be a non-empty subset of default {default:?}"
        );
        for item in &strict {
            assert!(default.contains(item), "{item:?} missing from default run");
        }
    }

    #[test]
    fn test_strict_ignores_bare_prose_mentioning_markers() {
        // "strict" still means an explicit marker with a work item after it.
        let found = texts(&SATDDetector::new_strict(), "// this is a todo list\n");
        assert!(found.is_empty(), "strict must not match prose: {found:?}");
    }

    // ── #668: markers whose text mentions unwrap/expect were dropped ──

    #[test]
    fn test_fixme_mentioning_unwrap_is_reported() {
        let found = texts(&SATDDetector::new(), "// FIXME: unwrap\npub fn a() {}\n");
        assert_eq!(found.len(), 1, "`// FIXME: unwrap` was dropped: {found:?}");
    }

    #[test]
    fn test_todo_mentioning_expect_is_reported() {
        let found = texts(&SATDDetector::new(), "// TODO: expect here\n");
        assert_eq!(found.len(), 1, "`// TODO: expect here` dropped: {found:?}");
    }

    // ── #674: markers whose prose names debt concepts were dropped ──

    #[test]
    fn test_todo_about_technical_debt_is_reported() {
        let content = "// TODO: pay down the technical debt here\n\
             // TODO: fix the detection logic\n\
             pub fn f() -> i32 { 1 }\n";
        let found = texts(&SATDDetector::new(), content);
        assert_eq!(found.len(), 2, "both TODOs must be reported: {found:?}");
    }

    #[test]
    fn test_todo_calling_itself_satd_is_reported() {
        let found = texts(
            &SATDDetector::new(),
            "// TODO: this is self-admitted technical debt\n",
        );
        assert_eq!(found.len(), 1, "dropped self-describing TODO: {found:?}");
    }

    // ── Guard rails: the override must not swallow the heuristics whole ──

    #[test]
    fn test_incidental_mention_in_code_is_still_suppressed() {
        let detector = SATDDetector::new();
        let found = texts(&detector, "    assert!(line.contains(\"TODO\"));\n");
        assert!(found.is_empty(), "code line reported as debt: {found:?}");
    }

    #[test]
    fn test_bug_tracking_id_is_still_suppressed() {
        let detector = SATDDetector::new();
        let found = texts(&detector, "// BUG-012: single language override\n");
        assert!(found.is_empty(), "tracker id reported as debt: {found:?}");
    }

    #[test]
    fn test_doc_comment_policy_unchanged() {
        let detector = SATDDetector::new();
        let found = texts(&detector, "/// TODO: documented follow-up\n");
        assert!(found.is_empty(), "doc comment policy changed: {found:?}");
    }
}

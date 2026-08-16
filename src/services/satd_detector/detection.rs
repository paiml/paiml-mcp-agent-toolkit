//! SATD detection, scanning, and file processing logic.
//!
//! Split into submodules for maintainability:
//! - `detection_extraction.rs`: Constructors, comment scanning, context hashing
//! - `detection_analysis.rs`: Project analysis, directory scanning, result aggregation
//! - `detection_file_discovery.rs`: Source file finding, filtering, test/vendor detection
//!
//! There used to be a fourth, `detection_false_positives.rs`: ~300 lines of
//! hand-written suppression (`is_functional_description`, `is_markdown_header`,
//! `is_bug_tracking_id`, `contains("bug") && contains("report")`, …) plus an
//! override that had to punch back THROUGH those heuristics because they were
//! also suppressing real markers (#668, #674). All of it existed to undo one
//! decision — matching topic words anywhere in a line — and all of it is gone
//! with that decision (#925). A line is debt when a marker OPENS a comment on
//! it; `assert!(line.contains("TODO"))` has no comment, `regex: r"\btodo\b"` is
//! a string literal, `# Calculate …` is not a comment in Rust, and
//! `// ties broken by path.` admits nothing.

use blake3::Hasher;
use std::path::{Path, PathBuf};

use crate::models::error::TemplateError;
// #923: one implementation of "is this path inside its project's tests/,
// examples/, fuzz/, vendor/ ...?", shared with the Rust defect detector so the
// two gates cannot drift apart again.
use crate::services::defect_detector::source_scope;

use super::types::{
    AstContext, AstNodeType, DebtClassifier, ProjectAnalysisStats, SATDAnalysisResult,
    SATDDetector, SATDSummary, SkipCounts, SkipReason, TechnicalDebt, TestBlockTracker,
};

include!("detection_extraction.rs");
include!("detection_analysis.rs");
include!("detection_file_discovery.rs");

#[cfg(test)]
mod comment_scanner_tests {
    //! Covers the comment scanner in detection_extraction.rs: which leaders a
    //! language has, where a comment starts, and what is a string literal.
    use super::*;

    fn detector() -> SATDDetector {
        SATDDetector::new()
    }

    fn scan(path: &str, line: &str) -> Option<CommentSpan> {
        CommentScanner::for_path(Path::new(path)).scan_line(line)
    }

    fn text_of(path: &str, line: &str) -> Option<String> {
        scan(path, line).map(|c| c.text)
    }

    // ── is_rust_file ──

    #[test]
    fn test_is_rust_file_rs_extension_returns_true() {
        assert!(detector().is_rust_file(Path::new("src/main.rs")));
    }

    #[test]
    fn test_is_rust_file_non_rs_returns_false() {
        let d = detector();
        assert!(!d.is_rust_file(Path::new("a.py")));
        assert!(!d.is_rust_file(Path::new("a.js")));
    }

    #[test]
    fn test_is_rust_file_no_extension_returns_false() {
        assert!(!detector().is_rust_file(Path::new("Makefile")));
    }

    // ── the comment's column ──

    #[test]
    fn test_column_of_leading_line_comment() {
        assert_eq!(scan("a.rs", "    // TODO: x").expect("comment").column, 5);
    }

    #[test]
    fn test_column_of_hash_comment() {
        assert_eq!(scan("a.py", "    # TODO: x").expect("comment").column, 5);
    }

    #[test]
    fn test_column_of_block_comment() {
        assert_eq!(
            scan("a.rs", "    /* TODO: x */").expect("comment").column,
            5
        );
    }

    #[test]
    fn test_column_of_html_comment() {
        assert_eq!(
            scan("a.md", "    <!-- TODO: x -->")
                .expect("comment")
                .column,
            5
        );
    }

    #[test]
    fn test_no_comment_on_a_code_line() {
        assert_eq!(scan("a.rs", "let x = 5;"), None);
    }

    // ── comment text ──

    #[test]
    fn test_text_of_each_comment_style() {
        assert_eq!(
            text_of("a.rs", "// TODO: fix").as_deref(),
            Some("TODO: fix")
        );
        assert_eq!(text_of("a.py", "# TODO: fix").as_deref(), Some("TODO: fix"));
        assert_eq!(
            text_of("a.rs", "/* FIXME: bug */").as_deref(),
            Some("FIXME: bug")
        );
        assert_eq!(
            text_of("a.html", "<!-- HACK: workaround -->").as_deref(),
            Some("HACK: workaround")
        );
        assert_eq!(text_of("a.rs", "//   TODO   ").as_deref(), Some("TODO"));
    }

    /// Doc comments ARE scanned now. The scanner's job is to find the comment;
    /// deciding whether it admits debt is the classifier's, and for a doc
    /// comment the classifier applies the marker rule alone (see `debt_of`).
    ///
    /// The previous policy — "documentation is not debt" — dropped these at the
    /// scanner, so `/// TODO: implement X` was invisible to every SATD surface.
    /// That conflates two different things: a doc comment is usually PROSE, and
    /// prose is not debt, but `TODO:` is an admission that the documented
    /// behaviour does not exist yet. Written in the public API documentation it
    /// is the most visible debt in the file to a human reader and was the least
    /// visible to the tool.
    #[test]
    fn test_doc_comments_are_scanned_for_satd() {
        assert_eq!(
            text_of("a.rs", "/// TODO: documented").as_deref(),
            Some("TODO: documented")
        );
        assert_eq!(
            text_of("a.rs", "//! TODO: module doc").as_deref(),
            Some("TODO: module doc")
        );
        assert_eq!(
            text_of("a.rs", "/** TODO: doc block */").as_deref(),
            Some("TODO: doc block")
        );
    }

    #[test]
    fn test_line_comment_owns_the_rest_of_the_line() {
        assert_eq!(
            text_of("a.rs", "let x = 1; // TODO: a // TODO: b").as_deref(),
            Some("TODO: a // TODO: b")
        );
    }

    // ── #925: a language only has the comment leaders it has ──

    #[test]
    fn test_hash_is_not_a_comment_in_rust() {
        // `# Calculate Technical Debt Grade (TDG)` is a line of pmat's own
        // --help block, inside a Rust string. It was reported as SATD.
        assert_eq!(
            text_of("a.rs", "# Calculate Technical Debt Grade (TDG)"),
            None
        );
        assert_eq!(text_of("a.rs", "#[derive(Debug)]"), None);
    }

    #[test]
    fn test_slash_is_not_a_comment_in_python() {
        assert_eq!(
            text_of("a.py", "x = a // b  # TODO: integer division").as_deref(),
            Some("TODO: integer division")
        );
    }

    #[test]
    fn test_markdown_heading_is_not_a_comment() {
        assert_eq!(text_of("README.md", "# Security"), None);
    }

    // ── string literals are not comments ──

    #[test]
    fn test_marker_inside_a_string_literal_is_not_a_comment() {
        assert_eq!(
            text_of("a.rs", r#"assert!(line.contains("// TODO: x"));"#),
            None
        );
        assert_eq!(text_of("a.rs", r#"println!("TODO: {}", x);"#), None);
    }

    #[test]
    fn test_comment_after_a_string_literal_is_found() {
        assert_eq!(
            text_of("a.rs", r#"let s = "a // b"; // TODO: real one"#).as_deref(),
            Some("TODO: real one")
        );
    }

    #[test]
    fn test_char_literal_slash_does_not_hide_the_comment() {
        assert_eq!(
            text_of("a.rs", r"let c = '/'; // TODO: real one").as_deref(),
            Some("TODO: real one")
        );
    }

    #[test]
    fn test_lifetime_is_not_a_string() {
        assert_eq!(
            text_of("a.rs", "fn f<'a>(s: &'a str) {} // TODO: real one").as_deref(),
            Some("TODO: real one")
        );
    }

    #[test]
    fn test_multi_line_raw_string_is_not_comments() {
        let mut scanner = CommentScanner::for_path(Path::new("a.rs"));
        assert_eq!(scanner.scan_line(r#"    let fixture = r#""#), None);
        assert_eq!(scanner.scan_line("    // TODO: this is fixture data"), None);
        assert_eq!(scanner.scan_line(r##"    "#;"##), None);
        assert_eq!(
            scanner
                .scan_line("    // TODO: this one is real")
                .map(|c| c.text),
            Some("TODO: this one is real".to_string())
        );
    }

    #[test]
    fn test_block_comment_spans_lines() {
        let mut scanner = CommentScanner::for_path(Path::new("a.rs"));
        assert_eq!(scanner.scan_line("/*"), None);
        assert_eq!(
            scanner
                .scan_line(" * TODO: inside the block")
                .map(|c| c.text),
            Some("TODO: inside the block".to_string())
        );
        assert_eq!(scanner.scan_line(" */"), None);
    }

    // ── hash_context (blake3) ──

    #[test]
    fn test_hash_context_returns_16_bytes() {
        assert_eq!(
            detector().hash_context(Path::new("a.rs"), 10, "TODO").len(),
            16
        );
    }

    #[test]
    fn test_hash_context_differs_by_path_and_line_and_is_deterministic() {
        let d = detector();
        let base = d.hash_context(Path::new("a.rs"), 10, "TODO");
        assert_ne!(base, d.hash_context(Path::new("b.rs"), 10, "TODO"));
        assert_ne!(base, d.hash_context(Path::new("a.rs"), 20, "TODO"));
        assert_eq!(base, d.hash_context(Path::new("a.rs"), 10, "TODO"));
    }

    // ── Constructors ──

    #[test]
    fn test_constructors() {
        let _ = SATDDetector::default();
        let _ = SATDDetector::new();
        let _ = SATDDetector::new_extended();
        let _ = SATDDetector::new_strict();
    }

    #[test]
    fn test_over_long_line_is_refused_not_silently_dropped() {
        let long = format!("// TODO: {}", "x".repeat(10_001));
        assert!(detector()
            .extract_from_content(&long, Path::new("src/lib.rs"))
            .is_err());
    }
}

/// #944 — a marker was only seen when the comment STARTED the line, so a
/// trailing `// TODO:` after code was invisible: a 407-line file with 43
/// markers reported 3, and the minimal case below reported 2 of 4.
#[cfg(test)]
mod trailing_comment_regression_tests {
    use super::*;

    const TWO_TRAILING_TWO_LEADING: &str = "pub fn a() -> i32 { 1 } // TODO: trailing one\n\
         pub fn b() -> i32 {\n    \
         // TODO: own-line one\n    \
         2\n\
         }\n\
         pub fn c() -> i32 { 3 } // FIXME: trailing two\n\
         pub fn d() -> i32 {\n    \
         // FIXME: own-line two\n    \
         4\n\
         }\n";

    #[test]
    fn a_trailing_marker_is_found() {
        let found = SATDDetector::new()
            .extract_from_content(TWO_TRAILING_TWO_LEADING, Path::new("src/lib.rs"))
            .expect("extraction must succeed");
        let texts: Vec<&str> = found.iter().map(|d| d.text.as_str()).collect();
        assert_eq!(
            found.len(),
            4,
            "all four markers must be reported, got {texts:?}"
        );
        assert!(texts.contains(&"TODO: trailing one"), "{texts:?}");
        assert!(texts.contains(&"FIXME: trailing two"), "{texts:?}");
    }

    #[test]
    fn a_trailing_marker_reports_the_column_the_comment_starts_at() {
        let found = SATDDetector::new()
            .extract_from_content(TWO_TRAILING_TWO_LEADING, Path::new("src/lib.rs"))
            .expect("extraction must succeed");
        let trailing = found
            .iter()
            .find(|d| d.text == "TODO: trailing one")
            .expect("the trailing TODO");
        assert_eq!(trailing.line, 1);
        assert_eq!(trailing.column, 25, "column must point at the `//`");
    }

    #[test]
    fn trailing_markers_are_found_in_hash_comment_languages_too() {
        let found = SATDDetector::new()
            .extract_from_content("x = 1  # TODO: trailing python\n", Path::new("s.py"))
            .expect("extraction must succeed");
        assert_eq!(found.len(), 1, "{found:?}");
        assert_eq!(found[0].text, "TODO: trailing python");
    }
}

/// #925 — 57 of the 62 violations reported on this repository had no debt
/// marker: the matcher looked for topic words (`broken`, `temp`,
/// `vulnerabilities`) anywhere in the line, so prose describing a FIX became
/// the highest-severity finding in the tree.
#[cfg(test)]
mod prose_is_not_debt_regression_tests {
    use super::*;
    use crate::services::satd_detector::{DebtCategory, Severity};

    /// The issue's own fixture: 5 violations reported, of which 2 were prose.
    const FIXTURE: &str = "// Deterministic order: worst score first, ties broken by path.\n\
         // Atomic write: temp file + rename.\n\
         pub fn a() {}\n\
         \n\
         // TODO: implement error handling\n\
         // FIXME: this leaks memory\n\
         // HACK: works around upstream bug\n\
         pub fn b() {}\n";

    fn texts(content: &str) -> Vec<String> {
        SATDDetector::new()
            .extract_from_content(content, Path::new("src/lib.rs"))
            .expect("extraction must succeed")
            .into_iter()
            .map(|d| d.text)
            .collect()
    }

    #[test]
    fn only_the_three_marker_comments_are_debt() {
        let found = texts(FIXTURE);
        assert_eq!(found.len(), 3, "prose still reported as debt: {found:?}");
        for marker in ["TODO", "FIXME", "HACK"] {
            assert!(
                found.iter().any(|t| t.starts_with(marker)),
                "missing {marker} in {found:?}"
            );
        }
    }

    /// Both `Critical` findings in the whole repository were prose about a
    /// completed fix. Critical stays reachable — from an explicit marker.
    #[test]
    fn critical_needs_an_explicit_security_marker() {
        let prose = "// vulnerability count. Reporting those as \"0 vulnerabilities\" phrased a\n";
        assert!(
            texts(prose).is_empty(),
            "prose about vulnerabilities is not debt"
        );

        let admitted = SATDDetector::new()
            .extract_from_content(
                "// SECURITY: the token is logged in plain text\n",
                Path::new("src/lib.rs"),
            )
            .expect("extraction must succeed");
        assert_eq!(admitted.len(), 1);
        assert_eq!(admitted[0].severity, Severity::Critical);
        assert_eq!(admitted[0].category, DebtCategory::Security);
    }

    /// The false negative in the same issue: a real `// TODO:` in production
    /// code that the tool never reported.
    #[test]
    fn the_missed_call_graph_todo_is_reported() {
        let content = "    // Phase 2: Extract edges (function calls, struct usage, etc.)\n    \
             // TODO: Implement call graph edge extraction in future iteration\n    \
             // For now, just return the graph with nodes (still provides O(1) lookups)\n";
        let found = texts(content);
        assert_eq!(found.len(), 1, "{found:?}");
        assert!(found[0].starts_with("TODO:"), "{found:?}");
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

    /// The doc-comment policy, in both directions.
    ///
    /// A marker-leading doc comment is debt; doc PROSE is not. The second half
    /// is the load-bearing one: #925 measured a 92% false-positive rate from
    /// phrase-matching ordinary prose, and doc comments are overwhelmingly
    /// prose, so scanning them without that asymmetry would have reintroduced
    /// #925 at a much larger scale. `debt_of` classifies a doc comment by
    /// `marker_at_start` only, never by phrase.
    #[test]
    fn test_doc_comment_marker_is_debt_but_doc_prose_is_not() {
        let detector = SATDDetector::new();

        let found = texts(&detector, "/// TODO: documented follow-up\n");
        assert_eq!(
            found.len(),
            1,
            "a marker-leading doc comment is self-admitted debt: {found:?}"
        );

        // #925's literal false positive, now in a doc comment.
        for prose in [
            "/// Deterministic order: ties broken by path.\n",
            "//! This module is a temporary home for the parser.\n",
            "/// Returns None if the file is missing or broken.\n",
        ] {
            let found = texts(&detector, prose);
            assert!(
                found.is_empty(),
                "doc prose reported as debt — #925 reintroduced: {prose:?} -> {found:?}"
            );
        }
    }
}

/// #925's false negative: `is_build_or_config_file` matched the SUBSTRING
/// `/build.rs`, so any file called `build.rs` anywhere in a source tree was
/// excluded — including `src/services/context_impl/build.rs`, whose production
/// `// TODO: Implement call graph edge extraction in future iteration` was the
/// one real marker the issue looked for and could not find.
#[cfg(test)]
mod build_script_exclusion_regression_tests {
    use super::*;

    #[test]
    fn only_the_packages_own_build_script_is_excluded() {
        let tmp = tempfile::TempDir::new().expect("tempdir");
        let root = tmp.path().join("myproject");
        std::fs::create_dir_all(root.join("src/services/context_impl")).expect("dirs");
        std::fs::write(
            root.join("Cargo.toml"),
            "[package]\nname = \"myproject\"\nversion = \"0.1.0\"\nedition = \"2021\"\n",
        )
        .expect("manifest");

        let detector = SATDDetector::new();

        let build_script = root.join("build.rs");
        std::fs::write(&build_script, "fn main() {}\n").expect("build script");
        assert!(
            detector.should_exclude_file(&build_script),
            "the package's own build script is not source code"
        );

        let module = root.join("src/services/context_impl/build.rs");
        std::fs::write(&module, "// TODO: Implement call graph edge extraction\n").expect("module");
        assert!(
            !detector.should_exclude_file(&module),
            "a module called build.rs is production code, not a build script"
        );

        let found = detector
            .extract_from_content("// TODO: Implement call graph edge extraction\n", &module)
            .expect("extraction must succeed");
        assert_eq!(found.len(), 1, "{found:?}");
    }
}

#[cfg(test)]
mod include_tests_reaches_inline_blocks {
    //! REGRESSION (#994): `--include-tests` could not reach an inline
    //! `#[cfg(test)]` block, so the two halves of "test code" behaved
    //! differently and one of them was unreachable by any invocation.
    //!
    //! | where the debt lives | before | after |
    //! |---|---|---|
    //! | `tests/it.rs` | included by the flag | unchanged |
    //! | `src/lib.rs`, in `#[cfg(test)] mod tests` | **no flag could reach it** | included by the flag |
    //!
    //! `include_tests` reached file DISCOVERY only, so it chose which paths to
    //! open and had no say over what was read inside them.
    use crate::services::satd_detector::SATDDetector;
    use std::path::Path;

    const INLINE: &str = "pub fn f() -> i32 { 1 }\n\
                          \n\
                          #[cfg(test)]\n\
                          mod tests {\n\
                          \x20   // TODO: debt inside an inline test module\n\
                          \x20   // FIXME: and more of it\n\
                          \x20   #[test] fn t() { assert_eq!(1, 1); }\n\
                          }\n";

    /// The default stays as it was: inline test blocks are production-clean.
    #[test]
    fn excluded_by_default() {
        let found = SATDDetector::new()
            .extract_from_content(INLINE, Path::new("src/lib.rs"))
            .expect("extraction");
        assert!(
            found.is_empty(),
            "inline test debt must stay out of a production scan, got {found:?}"
        );
    }

    /// …and is now reachable, which is the whole defect.
    #[test]
    fn reachable_when_tests_are_requested() {
        let found = SATDDetector::new()
            .extract_from_content_with_tests(INLINE, Path::new("src/lib.rs"), true)
            .expect("extraction");
        assert_eq!(
            found.len(),
            2,
            "both markers must be reachable with include_tests, got {found:?}"
        );
    }

    /// Production debt is reported either way — the flag adds, never replaces.
    #[test]
    fn production_debt_is_unaffected_by_the_flag() {
        let mixed = format!("// TODO: production\n{INLINE}");
        let d = SATDDetector::new();
        let without = d
            .extract_from_content(&mixed, Path::new("src/lib.rs"))
            .expect("extraction");
        let with = d
            .extract_from_content_with_tests(&mixed, Path::new("src/lib.rs"), true)
            .expect("extraction");
        assert_eq!(without.len(), 1, "production marker only: {without:?}");
        assert_eq!(with.len(), 3, "production + both inline markers: {with:?}");
    }
}

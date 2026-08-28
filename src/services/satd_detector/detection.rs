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
    AstContext, AstNodeType, DebtClassifier, FileCensus, ProjectAnalysisStats, SATDAnalysisResult,
    SATDDetector, SATDSummary, SkipReason, TechnicalDebt, TestBlockTracker, MAX_FILE_BYTES,
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

    /// Guard rail: the package's OWN support directories are still support
    /// code. The rule became project-relative; it did not go away.
    ///
    /// `examples/` and `demo/` LEFT this list in #1035 and have their own test
    /// below — they are shipped, compiled code, not support code.
    #[test]
    fn a_packages_own_support_directories_are_still_excluded() {
        let tmp = tempfile::TempDir::new().expect("tempdir");
        let root = tmp.path().join("myproject");
        crate_at(&root);
        let detector = SATDDetector::new();

        for support in ["fuzz", "vendor", "node_modules", "target"] {
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

        // `vendor/`, not `examples/`: #1035 moved examples into the analysed
        // population, so a tree of examples is no longer a tree where every
        // candidate is excluded. Vendored code still is.
        let vendored = root.join("vendor");
        std::fs::create_dir_all(&vendored).expect("vendor dir");
        for name in ["a.rs", "b.rs"] {
            std::fs::write(vendored.join(name), LIB_RS).expect("vendored file");
        }

        let detector = SATDDetector::new();
        let err = detector
            .analyze_directory(&vendored)
            .await
            .expect_err("every candidate under vendor/ is excluded — nothing was measured");
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

        // Control: the crate's own src/ IS measured, and measures 1. The two
        // vendored copies of the same marker are NOT added to it.
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

#[cfg(test)]
mod unreadable_file_disclosure_tests {
    //! #1035: a file the walk selected and then failed to decode was counted
    //! as an ANALYSED file with no debt.
    //!
    //! The three drops sat below the skip predicate, inside the per-file read
    //! — `Err(_) => Vec::new()`, `unwrap_or_default()`, and an oversized-content
    //! `return` — so every disclosure fix layered above them still reported
    //! these files as measured and clean. That is the issue's root-cause shape
    //! (`// If parsing fails, return empty (graceful degradation)`) surviving
    //! inside the analyzer the earlier fixes hardened.
    use super::*;
    use std::io::Write;

    /// A tree with one readable marker-bearing file and one file whose bytes
    /// are not UTF-8. `read_to_string` fails on the second; nothing else in
    /// the walk declines it.
    fn tree_with_one_undecodable_file() -> tempfile::TempDir {
        let d = tempfile::tempdir().expect("tempdir");
        std::fs::create_dir_all(d.path().join("src")).expect("mkdir");
        std::fs::write(
            d.path().join("src/good.rs"),
            "fn ok() {}\n// TODO: a real marker\n",
        )
        .expect("write good");
        let mut bad = std::fs::File::create(d.path().join("src/bad.rs")).expect("create bad");
        bad.write_all(b"fn bad() {}\n// TODO: hidden marker \xff\xfe\n")
            .expect("write bad");
        d
    }

    #[tokio::test]
    async fn an_undecodable_file_is_disclosed_not_counted_as_clean() {
        let d = tree_with_one_undecodable_file();
        let (debts, skipped) = SATDDetector::new()
            .analyze_directory_with_stats(d.path(), false)
            .await
            .expect("the readable file was analysed, so this is not a refusal");

        assert_eq!(
            debts.len(),
            1,
            "the readable marker is still found: {debts:?}"
        );
        assert_eq!(
            skipped.not_read.unreadable, 1,
            "the undecodable file must be DISCLOSED, not silently treated as a \
             file that was read and found clean: {skipped:?}"
        );
        assert!(
            skipped.not_read.total() >= 1,
            "an undisclosed skip makes the total a lie: {skipped:?}"
        );
        assert!(
            skipped.partitions(),
            "and the disclosure must add up: {skipped:?}"
        );
        let note = skipped
            .note()
            .expect("something was not read, so the report must say so");
        assert!(
            note.contains("1 unreadable"),
            "the human-readable summary must name it: {note}"
        );
    }

    /// Counter-test: the lazy over-correction — counting every file as
    /// unreadable, or refusing any tree containing one — must not pass.
    #[tokio::test]
    async fn a_fully_readable_tree_discloses_no_unreadable_files() {
        let d = tempfile::tempdir().expect("tempdir");
        std::fs::create_dir_all(d.path().join("src")).expect("mkdir");
        std::fs::write(
            d.path().join("src/a.rs"),
            "fn a() {}\n// TODO: marker one\n",
        )
        .expect("write a");
        std::fs::write(d.path().join("src/b.rs"), "fn b() {}\n").expect("write b");

        let (debts, skipped) = SATDDetector::new()
            .analyze_directory_with_stats(d.path(), false)
            .await
            .expect("analysis");
        assert_eq!(debts.len(), 1, "{debts:?}");
        assert_eq!(
            skipped.not_read.unreadable, 0,
            "no file failed to decode, so nothing may be reported as unread: \
             {skipped:?}"
        );
        assert_eq!(skipped.not_read.total(), 0, "{skipped:?}");
        // The note is no longer silent on a fully-read tree: it states the
        // denominator, which is the sentence that makes a zero mean something
        // (#1035). What it must NOT do is claim a skip that did not happen.
        let note = skipped.note().expect("the population is always stated");
        assert!(note.contains("analysed 2 of 2"), "{note}");
        assert!(!note.contains("not read"), "{note}");
    }

    /// And the extreme case still refuses rather than reporting a clean tree:
    /// when EVERY candidate failed to decode, nothing was measured.
    #[tokio::test]
    async fn a_tree_of_only_undecodable_files_is_a_refusal_not_a_clean_report() {
        let d = tempfile::tempdir().expect("tempdir");
        std::fs::create_dir_all(d.path().join("src")).expect("mkdir");
        for name in ["x.rs", "y.rs"] {
            let mut f = std::fs::File::create(d.path().join("src").join(name)).expect("create");
            f.write_all(b"fn f() {}\n// TODO: \xff\xfe\n")
                .expect("write");
        }
        let err = SATDDetector::new()
            .analyze_directory_with_stats(d.path(), false)
            .await
            .expect_err(
                "nothing in this tree was decoded; reporting `0 violations` \
                 would be the exact defect #1035 names",
            );
        let msg = err.to_string();
        assert!(
            msg.to_lowercase().contains("satd"),
            "the refusal must name the analysis it declined to perform: {msg}"
        );
    }
}

#[cfg(test)]
mod include_tests_must_not_empty_the_denominator {
    //! REGRESSION (#1050 P9): `--include-tests` only WIDENS the scan, so it
    //! cannot turn "1 source file, all skipped" into "no source files were
    //! found". It did, for any tree with a directory named `book`, `dist`,
    //! `build`, `node_modules`, `target` or `__pycache__` — because the flag
    //! selected a second discovery implementation whose directory policy
    //! differed from the default one's.
    //!
    //! The trigger is the DIRECTORY NAME, not the language: `foo/a.js`
    //! analysed normally in both runs.
    use super::*;

    async fn discovered_under(root: &std::path::Path, include_tests: bool) -> usize {
        let detector = SATDDetector::new();
        detector
            .discover_files(root, include_tests)
            .await
            .expect("discovery must not error on a readable tree")
            .0
            .len()
    }

    /// The fixture must be a GIT checkout, because that is the only place the
    /// two walks differed: the default path asks `git ls-files`, which lists
    /// `book/a.js`, while the flag's path used a filesystem walk that never
    /// descended into `book/`. In a NON-git tree both fall back to the same
    /// filesystem walk and both report 0 — the same denominator-vanishing
    /// defect, but symmetric, so it cannot show a disagreement. That
    /// non-git zero is recorded here rather than fixed: widening the
    /// filesystem fallback changes what `analyze satd` reads on every non-git
    /// tree, which is a separate measurement.
    ///
    /// RED CONTROL: restoring the `collect_files_including_tests` branch makes
    /// the `true` case 0 while the `false` case stays 1 — the exact pair the
    /// two error messages were built from.
    #[tokio::test]
    async fn a_widening_flag_cannot_shrink_the_population() {
        let tmp = tempfile::TempDir::new().expect("tempdir");
        let root = tmp.path().to_path_buf();
        std::fs::create_dir_all(root.join("book")).expect("book dir");
        std::fs::write(root.join("book/a.js"), "function a(){ return 1; }\n").expect("js");
        // `--template=` keeps the user's global hook template out of the
        // fixture; no commit is needed, `--others` lists untracked files.
        let init = std::process::Command::new("git")
            .args(["init", "-q", "--template="])
            .current_dir(&root)
            .output()
            .expect("git must be available to reproduce this");
        assert!(init.status.success(), "git init failed: {init:?}");

        let without = discovered_under(&root, false).await;
        let with = discovered_under(&root, true).await;

        assert_eq!(
            without, 1,
            "the default walk sees the file; that is the baseline the flag must not lose"
        );
        assert!(
            with >= without,
            "--include-tests only widens: it found {with} where the default found {without}"
        );
    }

    /// COUNTER-TEST: the fix must not make the flag a no-op. A tree with a real
    /// test file still discovers strictly MORE with the flag than without.
    #[tokio::test]
    async fn the_flag_still_widens_where_there_is_something_to_widen_to() {
        let tmp = tempfile::TempDir::new().expect("tempdir");
        let root = tmp.path().to_path_buf();
        std::fs::create_dir_all(root.join("src")).expect("src");
        std::fs::create_dir_all(root.join("tests")).expect("tests");
        std::fs::write(root.join("src/lib.rs"), "pub fn a() -> i32 { 1 }\n").expect("lib");
        std::fs::write(root.join("tests/it.rs"), "#[test] fn t() {}\n").expect("it");

        let without = discovered_under(&root, false).await;
        let with = discovered_under(&root, true).await;

        assert_eq!(without, 1, "the test file is declined by default");
        assert_eq!(with, 2, "…and reached by the flag");
    }
}

#[cfg(test)]
mod the_refusal_describes_the_run_that_produced_it {
    //! Issue #1050 P9, second half. The refusal's remedy and its list of skip
    //! reasons were fixed strings, so a run that had ALREADY passed
    //! `--include-tests` was told, as its remedy, to pass `--include-tests` —
    //! advice that cannot work — beside a reason list whose first entry
    //! ("test") the flag in force had already ruled out.
    //!
    //! Advice that cannot work is the same defect class as #1045's "index is
    //! stale, run `pmat query`" against a code path that never refreshes: a
    //! sentence a reader will act on, and acting on it changes nothing.
    use super::*;

    fn refusal(include_tests: bool) -> String {
        let error =
            SATDDetector::nothing_measured(std::path::Path::new("/tmp/bookfix"), 1, include_tests);
        error.to_string()
    }

    /// The half that regressed nothing: without the flag, the remedy is still
    /// the one that works.
    #[test]
    fn the_default_run_is_told_about_the_flag_it_has_not_used() {
        let msg = refusal(false);
        assert!(msg.contains("pass --include-tests"), "{msg}");
        assert!(msg.contains("all 1 source file(s)"), "{msg}");
        assert!(msg.contains("test, fuzz"), "{msg}");
    }

    /// The fix: a run WITH the flag is never told to pass the flag, and is not
    /// told that test-ness might be why its files were dropped.
    #[test]
    fn a_run_that_already_passed_the_flag_is_not_told_to_pass_it() {
        let msg = refusal(true);
        assert!(
            !msg.contains("pass --include-tests"),
            "the remedy names a flag that is already in force: {msg}"
        );
        assert!(
            !msg.contains("test, fuzz"),
            "test-ness cannot be a skip reason under --include-tests: {msg}"
        );
        assert!(msg.contains("--include-tests is already in force"), "{msg}");
    }

    /// The counter-test bounding the correction. Only the REMEDY and the reason
    /// list may move: the denominator, and the sentence that stops a gate
    /// reading this as a pass, are identical in both runs. #1050 P9's first
    /// half was exactly a denominator that vanished when the flag was added.
    #[test]
    fn the_measurement_half_of_the_sentence_is_identical_either_way() {
        for msg in [refusal(false), refusal(true)] {
            assert!(msg.contains("all 1 source file(s)"), "{msg}");
            assert!(msg.contains("no SATD measurement was taken"), "{msg}");
            assert!(msg.contains("This is not a clean result"), "{msg}");
            assert!(!msg.contains("no source files were found"), "{msg}");
        }
    }
}

#[cfg(test)]
mod census_has_a_denominator_tests {
    //! Issue #1035, Cluster 1 — the two exclusions that were rendered as a
    //! clean measurement, and the census that makes any future one legible.
    //!
    //! Both defects are instances of one root cause: **a failure to measure was
    //! rendered as a passing measurement**. SATD reported findings with no
    //! denominator, so `0 violations` was byte-identical whether a tree was read
    //! in full and found clean or whether nothing in it was read at all.
    //!
    //! RED, on the build before this module existed, over the fixture below —
    //! four markers planted outside `src/`, one of them reported:
    //!
    //! ```text
    //! $ pmat analyze satd -p fx --format json
    //! { "total_files": 2, "total_violations": 2,
    //!   "files_not_read": { "total": 4, "tests": 1,
    //!                       "examples_demo_fuzz_generated": 3, "too_large": 0 } }
    //! ```
    //!
    //! The two markers in `examples/hello.rs` were absent from the answer and
    //! present only inside a bucket of 3 that also held the vendored and the
    //! generated file — a reader could not tell shipped code from a dependency.
    //! `total_files: 2` is the count of files that HELD a violation, so there was
    //! no denominator anywhere: 2 + 4 could not be checked against anything.
    use super::*;

    const MARKER: &str = "// TODO: a marker\n";

    /// `src/lib.rs` (ordinary), `examples/hello.rs` (shipped code, two markers),
    /// `vendor/dep.rs` and `src/schema.generated.rs` (must STAY excluded),
    /// `tests/harness.rs` (excluded without `--include-tests`), and
    /// `src/huge.rs`, one byte past [`MAX_FILE_BYTES`].
    ///
    /// The oversized file is sparse: its length is a metadata fact and the walk
    /// declines it without reading a byte, so the fixture costs no disk.
    fn fixture() -> tempfile::TempDir {
        let tmp = tempfile::TempDir::new().expect("tempdir");
        let root = tmp.path();
        for sub in ["src", "examples", "vendor", "tests"] {
            std::fs::create_dir_all(root.join(sub)).expect("dir");
        }
        std::fs::write(
            root.join("Cargo.toml"),
            "[package]\nname = \"fx\"\nversion = \"0.1.0\"\nedition = \"2021\"\n",
        )
        .expect("manifest");
        std::fs::write(root.join("src/lib.rs"), MARKER).expect("lib.rs");
        std::fs::write(
            root.join("examples/hello.rs"),
            "// TODO: marker one in examples\n// FIXME: marker two in examples\n",
        )
        .expect("hello.rs");
        std::fs::write(root.join("vendor/dep.rs"), MARKER).expect("dep.rs");
        std::fs::write(root.join("src/schema.generated.rs"), MARKER).expect("generated");
        std::fs::write(root.join("tests/harness.rs"), MARKER).expect("harness.rs");
        let huge = std::fs::File::create(root.join("src/huge.rs")).expect("huge.rs");
        huge.set_len(MAX_FILE_BYTES + 1).expect("size huge.rs");
        tmp
    }

    async fn walk(root: &Path) -> (Vec<TechnicalDebt>, FileCensus) {
        SATDDetector::new()
            .analyze_directory_with_stats(root, false)
            .await
            .expect("the fixture has analysable files")
    }

    fn names(debts: &[TechnicalDebt]) -> Vec<String> {
        debts.iter().map(|d| d.file.display().to_string()).collect()
    }

    /// DEFECT A. `examples/` is shipped, compiled, user-facing code: `cargo
    /// build --examples` builds it and `cargo publish` ships it. A marker there
    /// is debt like any other. The audits behind #1035 measured the cost of the
    /// exclusion on pforge — 25 `.rs` files, 37% of the repository, invisible to
    /// every SATD run, confirmed the same way on depyler, forjar and pepita.
    #[tokio::test]
    async fn markers_in_examples_are_reported_as_debt() {
        let fx = fixture();
        let (debts, _) = walk(fx.path()).await;

        assert_eq!(
            debts.len(),
            3,
            "one marker in src/ and two in examples/: {:?}",
            names(&debts)
        );
        assert!(
            names(&debts).iter().any(|f| f.contains("hello.rs")),
            "the example's debt must be named: {:?}",
            names(&debts)
        );
    }

    /// COUNTER-TEST for defect A. The fix must not become "report everything as
    /// debt". Code this project cannot fix in place stays excluded — and stays
    /// COUNTED, which is the half that makes the exclusion legible rather than
    /// silent.
    #[tokio::test]
    async fn vendored_and_generated_stay_excluded_and_are_counted_as_excluded() {
        let fx = fixture();
        let (debts, census) = walk(fx.path()).await;

        let files = names(&debts);
        assert!(
            !files.iter().any(|f| f.contains("vendor")),
            "a vendored dependency is not this project's debt: {files:?}"
        );
        assert!(
            !files.iter().any(|f| f.contains(".generated")),
            "generated output is not hand-written debt: {files:?}"
        );
        assert_eq!(
            census.not_read.out_of_scope, 2,
            "excluded is not the same as absent — both must be counted: {census:?}"
        );
        assert_eq!(
            census.not_read.tests, 1,
            "tests/harness.rs was found and declined: {census:?}"
        );
    }

    /// DEFECT B. The size skip printed `Warning: Skipped: … (large file >500KB)`
    /// to stderr and nothing at all to the JSON, and `--format json` and
    /// `--output FILE` both discard stderr. A consumer reading the payload could
    /// not tell "clean" from "not looked at".
    #[tokio::test]
    async fn the_size_skip_is_carried_in_the_census_not_only_on_stderr() {
        let fx = fixture();
        let (_, census) = walk(fx.path()).await;

        assert_eq!(census.not_read.too_large, 1, "{census:?}");
        let oversized = census
            .oversized
            .first()
            .expect("a count alone cannot say WHICH file was not looked at");
        assert!(oversized.path.contains("huge.rs"), "{oversized:?}");
        assert_eq!(oversized.limit_bytes, MAX_FILE_BYTES);
        assert!(
            oversized.bytes > oversized.limit_bytes,
            "size and limit are both stated, so the rule is visible: {oversized:?}"
        );
    }

    /// DEFECT B, second half: ONE size rule. `analyze satd` read anything under
    /// 10 MB while the walk behind `quality-gate --checks satd` dropped anything
    /// over 512,000 bytes, so the two commands reported different numbers for
    /// the same tree and neither said why. An 800 KB file with a marker is read
    /// by BOTH now.
    #[tokio::test]
    async fn both_walks_use_one_size_rule() {
        let tmp = tempfile::TempDir::new().expect("tempdir");
        let root = tmp.path();
        std::fs::create_dir_all(root.join("src")).expect("src");
        std::fs::write(
            root.join("Cargo.toml"),
            "[package]\nname = \"fx\"\nversion = \"0.1.0\"\nedition = \"2021\"\n",
        )
        .expect("manifest");
        let mut big = String::from("// TODO: a marker inside an 800 KB file\n");
        while big.len() < 800_000 {
            big.push_str("// filler\n");
        }
        std::fs::write(root.join("src/big.rs"), big).expect("big.rs");

        let detector = SATDDetector::new();
        let (walk_debts, _) = detector
            .analyze_directory_with_stats(root, false)
            .await
            .expect("analysable");
        let project = detector
            .analyze_project(root, false)
            .await
            .expect("analysable");

        assert_eq!(walk_debts.len(), 1, "the 800 KB file is read");
        assert_eq!(
            project.items.len(),
            walk_debts.len(),
            "the two walks must measure the same population: {:?} vs {:?}",
            project.items.len(),
            walk_debts.len()
        );
        assert_eq!(
            project.census.not_read.too_large, 0,
            "800 KB is an ordinary source file, not a pathological one: {:?}",
            project.census
        );
    }

    /// THE POINT OF THE ISSUE. The buckets must PARTITION: a census that does
    /// not add up is the same defect in a new place.
    #[tokio::test]
    async fn the_census_partitions_the_files_it_walked() {
        let fx = fixture();
        let (_, census) = walk(fx.path()).await;

        assert_eq!(
            census.discovered, 6,
            "six .rs files were walked: {census:?}"
        );
        assert_eq!(
            census.analyzed, 2,
            "src/lib.rs and examples/hello.rs: {census:?}"
        );
        assert_eq!(census.not_read.total(), 4, "{census:?}");
        assert!(
            census.partitions(),
            "analysed + not read must equal walked: {census:?}"
        );
        assert_eq!(census.unaccounted(), 0, "{census:?}");
    }

    /// COUNTER-TEST for the census. A clean tree must report zero findings over
    /// a NON-ZERO denominator. Without this half, "everything was skipped" and
    /// "nothing was found" go back to being the same output, which is the defect
    /// rather than the fix.
    #[tokio::test]
    async fn a_clean_tree_reports_zero_over_a_nonzero_denominator() {
        let tmp = tempfile::TempDir::new().expect("tempdir");
        let root = tmp.path();
        std::fs::create_dir_all(root.join("src")).expect("src");
        std::fs::create_dir_all(root.join("examples")).expect("examples");
        std::fs::write(
            root.join("Cargo.toml"),
            "[package]\nname = \"clean\"\nversion = \"0.1.0\"\nedition = \"2021\"\n",
        )
        .expect("manifest");
        std::fs::write(root.join("src/lib.rs"), "pub fn a() -> u32 { 1 }\n").expect("lib.rs");
        std::fs::write(root.join("examples/hello.rs"), "fn main() {}\n").expect("hello.rs");

        let (debts, census) = walk(root).await;

        assert!(debts.is_empty(), "the tree really is clean: {debts:?}");
        assert_eq!(census.discovered, 2, "{census:?}");
        assert_eq!(
            census.analyzed, 2,
            "the zero above was measured over two files, not over nothing: {census:?}"
        );
        assert_eq!(census.not_read.total(), 0, "{census:?}");
        assert!(census.partitions(), "{census:?}");

        let note = census.note().expect("a clean tree still states its scope");
        assert!(note.contains("analysed 2 of 2"), "{note}");
        assert!(
            !note.contains("not read"),
            "an empty bucket is noise, not disclosure: {note}"
        );
    }

    /// The census must survive `--include-tests`, which moves a file from one
    /// side of the partition to the other and must not change the total.
    #[tokio::test]
    async fn include_tests_moves_a_file_across_the_partition_without_changing_it() {
        let fx = fixture();
        let detector = SATDDetector::new();
        let (_, without) = detector
            .analyze_directory_with_stats(fx.path(), false)
            .await
            .expect("analysable");
        let (_, with) = detector
            .analyze_directory_with_stats(fx.path(), true)
            .await
            .expect("analysable");

        assert_eq!(
            without.discovered, with.discovered,
            "a flag that only widens the scan must not change the denominator"
        );
        assert_eq!(without.not_read.tests, 1);
        assert_eq!(with.not_read.tests, 0);
        assert_eq!(with.analyzed, without.analyzed + 1);
        assert!(without.partitions() && with.partitions(), "{without:?}");
    }
}

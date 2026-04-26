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

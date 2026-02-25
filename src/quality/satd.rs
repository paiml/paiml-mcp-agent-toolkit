#![cfg_attr(coverage_nightly, coverage(off))]
use super::gate::SatdResult;
use regex::Regex;
use std::sync::LazyLock;

/// Standard SATD patterns (traditional markers)
static SATD_PATTERNS: LazyLock<Vec<(&str, Regex)>> = LazyLock::new(|| {
    vec![
        (
            "TODO",
            Regex::new(r"\bTODO\b").expect("static regex pattern '\\bTODO\\b' is valid"),
        ),
        (
            "FIXME",
            Regex::new(r"\bFIXME\b").expect("static regex pattern '\\bFIXME\\b' is valid"),
        ),
        (
            "HACK",
            Regex::new(r"\bHACK\b").expect("static regex pattern '\\bHACK\\b' is valid"),
        ),
        (
            "XXX",
            Regex::new(r"\bXXX\b").expect("static regex pattern '\\bXXX\\b' is valid"),
        ),
        (
            "REFACTOR",
            Regex::new(r"\bREFACTOR\b").expect("static regex pattern '\\bREFACTOR\\b' is valid"),
        ),
        (
            "OPTIMIZE",
            Regex::new(r"\bOPTIMIZE\b").expect("static regex pattern '\\bOPTIMIZE\\b' is valid"),
        ),
        (
            "REVIEW",
            Regex::new(r"\bREVIEW\b").expect("static regex pattern '\\bREVIEW\\b' is valid"),
        ),
        (
            "DEPRECATED",
            Regex::new(r"\bDEPRECATED\b")
                .expect("static regex pattern '\\bDEPRECATED\\b' is valid"),
        ),
        (
            "TEMPORARY",
            Regex::new(r"\bTEMPORARY\b").expect("static regex pattern '\\bTEMPORARY\\b' is valid"),
        ),
    ]
});

/// Extended SATD patterns - euphemisms that hide technical debt (issue #149)
/// These are commonly used by AI coding assistants to bypass SATD detection
static EXTENDED_PATTERNS: LazyLock<Vec<(&str, Regex)>> = LazyLock::new(|| {
    vec![
        // Placeholder patterns - indicate incomplete implementation
        (
            "PLACEHOLDER",
            Regex::new(r"(?i)\bplaceholder\b").expect("valid regex"),
        ),
        // Stub patterns - indicate missing implementation
        ("STUB", Regex::new(r"(?i)\bstub\b").expect("valid regex")),
        // Simplified patterns - indicate corners were cut
        (
            "SIMPLIFIED",
            Regex::new(r"(?i)\bsimplified\b").expect("valid regex"),
        ),
        // Demo patterns - indicate non-production code
        (
            "FOR_DEMO",
            Regex::new(r"(?i)\b(for\s+)?demonstrat(e|ion)\b").expect("valid regex"),
        ),
        // Mock/dummy patterns - indicate fake implementations
        (
            "MOCK",
            Regex::new(r"(?i)\b(mock|dummy|fake)\b").expect("valid regex"),
        ),
        // Hardcoded patterns - indicate missing configuration
        (
            "HARDCODED",
            Regex::new(r"(?i)\bhardcoded\b").expect("valid regex"),
        ),
        // "For now" patterns - indicate temporary solutions
        (
            "FOR_NOW",
            Regex::new(r"(?i)\bfor\s+now\b").expect("valid regex"),
        ),
        // WIP patterns - work in progress
        ("WIP", Regex::new(r"\bWIP\b").expect("valid regex")),
        // Skip/bypass patterns - indicate missing validation
        (
            "SKIP",
            Regex::new(r"(?i)\b(skip|bypass)\s+(for\s+now|this|validation)\b")
                .expect("valid regex"),
        ),
    ]
});

pub struct SatdDetector {
    patterns: Vec<(&'static str, Regex)>,
    extended: bool,
}

impl Default for SatdDetector {
    fn default() -> Self {
        Self::new()
    }
}

// Detection methods (new, with_extended, detect, detect_in_comments, extract_comments)
include!("satd_detection.rs");

// Unit tests
include!("satd_tests.rs");

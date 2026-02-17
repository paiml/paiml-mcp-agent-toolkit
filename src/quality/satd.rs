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

impl SatdDetector {
    pub fn new() -> Self {
        Self {
            patterns: SATD_PATTERNS.clone(),
            extended: false,
        }
    }

    /// Create a detector with extended patterns (euphemism detection)
    /// This catches hidden technical debt like "placeholder", "stub", "for now"
    pub fn with_extended() -> Self {
        let mut patterns = SATD_PATTERNS.clone();
        patterns.extend(EXTENDED_PATTERNS.clone());
        Self {
            patterns,
            extended: true,
        }
    }

    /// Check if extended mode is enabled
    pub fn is_extended(&self) -> bool {
        self.extended
    }

    pub fn detect(&self, source: &str) -> SatdResult {
        let mut count = 0;
        let mut found_patterns = Vec::new();

        for (pattern_name, regex) in &self.patterns {
            let matches = regex.find_iter(source).count();
            if matches > 0 {
                count += matches;
                if !found_patterns.contains(&pattern_name.to_string()) {
                    found_patterns.push(pattern_name.to_string());
                }
            }
        }

        SatdResult {
            count,
            patterns: found_patterns,
        }
    }

    pub fn detect_in_comments(&self, source: &str) -> SatdResult {
        // Extract only comments from source
        let comments = self.extract_comments(source);
        self.detect(&comments)
    }

    fn extract_comments(&self, source: &str) -> String {
        let mut in_block_comment = false;
        let mut comments = String::new();
        let lines = source.lines();

        for line in lines {
            let trimmed = line.trim();

            // Block comment handling
            if trimmed.starts_with("/*") {
                in_block_comment = true;
                comments.push_str(line);
                comments.push('\n');
                if trimmed.ends_with("*/") {
                    in_block_comment = false;
                }
                continue;
            }

            if in_block_comment {
                comments.push_str(line);
                comments.push('\n');
                if trimmed.ends_with("*/") {
                    in_block_comment = false;
                }
                continue;
            }

            // Line comment handling
            if let Some(comment_start) = line.find("//") {
                comments.push_str(line.get(comment_start..).unwrap_or_default());
                comments.push('\n');
            }
        }

        comments
    }
}

#[cfg_attr(coverage_nightly, coverage(off))]
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_detect_todo_patterns() {
        let detector = SatdDetector::new();
        let source = "// TODO: implement this\n// FIXME: broken code";

        let result = detector.detect(source);
        assert_eq!(result.count, 2);
        assert!(result.patterns.contains(&"TODO".to_string()));
        assert!(result.patterns.contains(&"FIXME".to_string()));
    }

    #[test]
    fn test_no_satd_in_clean_code() {
        let detector = SatdDetector::new();
        let source = "fn clean_function() {\n    println!(\"Clean code\");\n}";

        let result = detector.detect(source);
        assert_eq!(result.count, 0);
        assert!(result.patterns.is_empty());
    }

    /// Test that all SATD regex patterns compile successfully
    /// Validates expect() calls at lines 7-15 (static SATD_PATTERNS initialization)
    #[test]
    fn test_satd_patterns_compile_successfully() {
        // Access the static patterns to trigger lazy initialization
        let patterns = &*SATD_PATTERNS;

        // Verify all 9 patterns were initialized
        assert_eq!(patterns.len(), 9);

        // Verify all expected SATD keywords are present
        let pattern_names: Vec<&str> = patterns.iter().map(|(name, _)| *name).collect();
        assert!(pattern_names.contains(&"TODO"));
        assert!(pattern_names.contains(&"FIXME"));
        assert!(pattern_names.contains(&"HACK"));
        assert!(pattern_names.contains(&"XXX"));
        assert!(pattern_names.contains(&"REFACTOR"));
        assert!(pattern_names.contains(&"OPTIMIZE"));
        assert!(pattern_names.contains(&"REVIEW"));
        assert!(pattern_names.contains(&"DEPRECATED"));
        assert!(pattern_names.contains(&"TEMPORARY"));
    }

    /// Test that SATD patterns match word boundaries correctly
    /// Validates the correctness of patterns initialized with expect() at lines 7-15
    #[test]
    fn test_satd_patterns_word_boundary_matching() {
        let detector = SatdDetector::new();

        // Should match TODO as a word
        let source1 = "// TODO: implement this";
        let result1 = detector.detect(source1);
        assert_eq!(result1.count, 1);
        assert!(result1.patterns.contains(&"TODO".to_string()));

        // Should NOT match TODO as part of another word
        let source2 = "// TODOIST is a task manager";
        let result2 = detector.detect(source2);
        assert_eq!(result2.count, 0);

        // Should match XXX as a word
        let source3 = "XXX: critical issue";
        let result3 = detector.detect(source3);
        assert_eq!(result3.count, 1);
        assert!(result3.patterns.contains(&"XXX".to_string()));
    }

    /// Test that detector initialization is stable and doesn't panic
    /// Validates the expect() calls in SATD_PATTERNS never panic with valid patterns
    #[test]
    fn test_detector_initialization_stability() {
        // Create multiple instances to ensure initialization is deterministic
        for _ in 0..10 {
            let detector = SatdDetector::new();
            assert_eq!(detector.patterns.len(), 9);
        }

        // Test default() method
        let default_detector = SatdDetector::default();
        assert_eq!(default_detector.patterns.len(), 9);

        // Verify default and new produce equivalent results
        let detector1 = SatdDetector::new();
        let detector2 = SatdDetector::default();
        assert_eq!(detector1.patterns.len(), detector2.patterns.len());
    }

    /// Test that all SATD patterns match their respective keywords correctly
    /// Validates that each pattern initialized at lines 7-15 works as expected
    #[test]
    fn test_all_satd_patterns_match_correctly() {
        let detector = SatdDetector::new();

        // Test each SATD pattern
        let test_cases = vec![
            ("// TODO: fix this", "TODO"),
            ("// FIXME: broken", "FIXME"),
            ("// HACK: workaround", "HACK"),
            ("// XXX: warning", "XXX"),
            ("// REFACTOR: improve", "REFACTOR"),
            ("// OPTIMIZE: performance", "OPTIMIZE"),
            ("// REVIEW: check this", "REVIEW"),
            ("// DEPRECATED: old api", "DEPRECATED"),
            ("// TEMPORARY: remove later", "TEMPORARY"),
        ];

        for (source, expected_pattern) in test_cases {
            let result = detector.detect(source);
            assert_eq!(
                result.count, 1,
                "Expected 1 match for '{expected_pattern}' in '{source}'"
            );
            assert!(
                result.patterns.contains(&expected_pattern.to_string()),
                "Expected pattern '{expected_pattern}' not found"
            );
        }
    }

    /// Test that SATD detector handles multiple patterns in the same source
    /// Validates that all patterns work correctly together
    #[test]
    fn test_multiple_satd_patterns_in_source() {
        let detector = SatdDetector::new();

        let source = r#"
            // TODO: implement feature A
            // FIXME: bug in function B
            // HACK: temporary workaround
            // TODO: also implement feature C
        "#;

        let result = detector.detect(source);

        // Should detect 4 total matches (2 TODO + 1 FIXME + 1 HACK)
        assert_eq!(result.count, 4);

        // Should identify 3 unique patterns
        assert_eq!(result.patterns.len(), 3);
        assert!(result.patterns.contains(&"TODO".to_string()));
        assert!(result.patterns.contains(&"FIXME".to_string()));
        assert!(result.patterns.contains(&"HACK".to_string()));
    }

    #[test]
    fn test_extract_comments_line_comments() {
        let detector = SatdDetector::new();
        let source =
            "fn foo() {\n    let x = 1; // inline comment\n    // full line comment\n    bar();\n}";
        let comments = detector.extract_comments(source);
        assert!(comments.contains("// inline comment"));
        assert!(comments.contains("// full line comment"));
        assert!(!comments.contains("fn foo"));
        assert!(!comments.contains("bar()"));
    }

    #[test]
    fn test_extract_comments_block_comments() {
        let detector = SatdDetector::new();
        let source = "/* multi-line\n   block comment */\nfn foo() {}\n/* single line block */";
        let comments = detector.extract_comments(source);
        assert!(comments.contains("multi-line"));
        assert!(comments.contains("block comment"));
        assert!(comments.contains("single line block"));
        assert!(!comments.contains("fn foo"));
    }

    #[test]
    fn test_extract_comments_mixed() {
        let detector = SatdDetector::new();
        let source =
            "// line comment\n/* block */\ncode();\n// another line\n/* start\nmiddle\nend */";
        let comments = detector.extract_comments(source);
        assert!(comments.contains("// line comment"));
        assert!(comments.contains("/* block */"));
        assert!(comments.contains("// another line"));
        assert!(comments.contains("middle"));
        assert!(!comments.contains("code()"));
    }

    #[test]
    fn test_extract_comments_no_comments() {
        let detector = SatdDetector::new();
        let source = "fn main() {\n    println!(\"hello\");\n}";
        let comments = detector.extract_comments(source);
        assert!(comments.is_empty());
    }

    #[test]
    fn test_detect_in_comments_only() {
        let detector = SatdDetector::new();
        let source = "let msg = \"TODO: not a comment\";\n// FIXME: real comment";
        let result = detector.detect_in_comments(source);
        assert_eq!(result.count, 1);
        assert!(result.patterns.contains(&"FIXME".to_string()));
    }

    #[test]
    fn test_detect_in_comments_block_with_satd() {
        let detector = SatdDetector::new();
        let source =
            "fn foo() {}\n/* HACK: temporary workaround\n   TODO: fix later */\nfn bar() {}";
        let result = detector.detect_in_comments(source);
        assert_eq!(result.count, 2);
        assert!(result.patterns.contains(&"HACK".to_string()));
        assert!(result.patterns.contains(&"TODO".to_string()));
    }

    #[test]
    fn test_extract_comments_block_single_line_with_close() {
        let detector = SatdDetector::new();
        let source = "/* closed on same line */ code();";
        let comments = detector.extract_comments(source);
        assert!(comments.contains("closed on same line"));
    }

    // === Extended Pattern Tests ===

    #[test]
    fn test_with_extended_creates_extended_detector() {
        let detector = SatdDetector::with_extended();
        assert!(detector.is_extended());
        assert!(detector.patterns.len() > 9);
    }

    #[test]
    fn test_is_extended_standard_detector() {
        let detector = SatdDetector::new();
        assert!(!detector.is_extended());
    }

    #[test]
    fn test_extended_detects_placeholder() {
        let detector = SatdDetector::with_extended();
        let source = "// placeholder implementation";
        let result = detector.detect(source);
        assert!(result.count > 0);
        assert!(result.patterns.contains(&"PLACEHOLDER".to_string()));
    }

    #[test]
    fn test_extended_detects_stub() {
        let detector = SatdDetector::with_extended();
        let source = "// stub for testing";
        let result = detector.detect(source);
        assert!(result.count > 0);
        assert!(result.patterns.contains(&"STUB".to_string()));
    }

    #[test]
    fn test_extended_detects_for_now() {
        let detector = SatdDetector::with_extended();
        let source = "// this works for now";
        let result = detector.detect(source);
        assert!(result.count > 0);
        assert!(result.patterns.contains(&"FOR_NOW".to_string()));
    }

    #[test]
    fn test_extended_detects_mock_dummy_fake() {
        let detector = SatdDetector::with_extended();
        let source = "// using mock data and dummy values with fake response";
        let result = detector.detect(source);
        assert!(result.count >= 3);
        assert!(result.patterns.contains(&"MOCK".to_string()));
    }

    #[test]
    fn test_extended_detects_hardcoded() {
        let detector = SatdDetector::with_extended();
        let source = "// hardcoded value";
        let result = detector.detect(source);
        assert!(result.patterns.contains(&"HARDCODED".to_string()));
    }

    #[test]
    fn test_extended_detects_simplified() {
        let detector = SatdDetector::with_extended();
        let source = "// simplified version of the algorithm";
        let result = detector.detect(source);
        assert!(result.patterns.contains(&"SIMPLIFIED".to_string()));
    }

    #[test]
    fn test_extended_detects_wip() {
        let detector = SatdDetector::with_extended();
        let source = "// WIP: not finished yet";
        let result = detector.detect(source);
        assert!(result.patterns.contains(&"WIP".to_string()));
    }

    #[test]
    fn test_extended_detects_skip_bypass() {
        let detector = SatdDetector::with_extended();
        let source = "// skip validation for now";
        let result = detector.detect(source);
        assert!(result.count > 0);
    }

    #[test]
    fn test_extended_case_insensitive() {
        let detector = SatdDetector::with_extended();
        let source = "// PLACEHOLDER Stub HARDCODED Simplified";
        let result = detector.detect(source);
        assert!(result.count >= 4);
    }

    #[test]
    fn test_extended_detect_in_comments() {
        let detector = SatdDetector::with_extended();
        let source = "let x = 1; // placeholder for now\ncode();";
        let result = detector.detect_in_comments(source);
        assert!(result.count > 0);
        assert!(result.patterns.contains(&"PLACEHOLDER".to_string()));
        assert!(result.patterns.contains(&"FOR_NOW".to_string()));
    }

    #[test]
    fn test_extract_comments_unclosed_block() {
        let detector = SatdDetector::new();
        let source = "fn foo() {\n    /* unclosed block\n    still in comment\n}";
        let comments = detector.extract_comments(source);
        assert!(comments.contains("unclosed block"));
        assert!(comments.contains("still in comment"));
    }

    #[test]
    fn test_extract_comments_block_opening_on_own_line() {
        let detector = SatdDetector::new();
        let source = "/*\n  block content\n  more content\n*/\ncode();";
        let comments = detector.extract_comments(source);
        assert!(comments.contains("block content"));
        assert!(comments.contains("more content"));
    }
}

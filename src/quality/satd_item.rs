use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
/// Satd item.
pub struct SatdItem {
    pub satd_type: String,
    pub line: usize,
    pub comment: String,
}

/// Satd detector with items.
pub struct SatdDetectorWithItems;

impl Default for SatdDetectorWithItems {
    fn default() -> Self {
        Self::new()
    }
}

impl SatdDetectorWithItems {
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    /// Create a new instance.
    pub fn new() -> Self {
        Self
    }

    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    /// Detect.
    pub fn detect(&self, source: &str) -> Vec<SatdItem> {
        let mut items = Vec::new();

        for (line_num, line) in source.lines().enumerate() {
            if line.contains("TODO") {
                items.push(SatdItem {
                    satd_type: "TODO".to_string(),
                    line: line_num + 1,
                    comment: line.trim().to_string(),
                });
            } else if line.contains("FIXME") {
                items.push(SatdItem {
                    satd_type: "FIXME".to_string(),
                    line: line_num + 1,
                    comment: line.trim().to_string(),
                });
            } else if line.contains("HACK") {
                items.push(SatdItem {
                    satd_type: "HACK".to_string(),
                    line: line_num + 1,
                    comment: line.trim().to_string(),
                });
            }
        }

        items
    }
}

#[cfg_attr(coverage_nightly, coverage(off))]
#[cfg(test)]
mod coverage_tests {
    use super::*;

    // SatdItem tests
    #[test]
    fn test_satd_item_creation() {
        let item = SatdItem {
            satd_type: "TODO".to_string(),
            line: 42,
            comment: "// TODO: fix this".to_string(),
        };
        assert_eq!(item.satd_type, "TODO");
        assert_eq!(item.line, 42);
        assert_eq!(item.comment, "// TODO: fix this");
    }

    #[test]
    fn test_satd_item_clone() {
        let item = SatdItem {
            satd_type: "FIXME".to_string(),
            line: 10,
            comment: "// FIXME: urgent".to_string(),
        };
        let cloned = item.clone();
        assert_eq!(item.satd_type, cloned.satd_type);
        assert_eq!(item.line, cloned.line);
        assert_eq!(item.comment, cloned.comment);
    }

    #[test]
    fn test_satd_item_debug() {
        let item = SatdItem {
            satd_type: "HACK".to_string(),
            line: 5,
            comment: "// HACK: workaround".to_string(),
        };
        let debug_str = format!("{:?}", item);
        assert!(debug_str.contains("HACK"));
        assert!(debug_str.contains("5"));
    }

    #[test]
    fn test_satd_item_serialization() {
        let item = SatdItem {
            satd_type: "TODO".to_string(),
            line: 1,
            comment: "// TODO: test".to_string(),
        };
        let json = serde_json::to_string(&item).unwrap();
        let deserialized: SatdItem = serde_json::from_str(&json).unwrap();
        assert_eq!(item.satd_type, deserialized.satd_type);
        assert_eq!(item.line, deserialized.line);
        assert_eq!(item.comment, deserialized.comment);
    }

    // SatdDetectorWithItems tests
    #[test]
    fn test_satd_detector_default() {
        // Exercise the Default impl (delegates to Self::new()).
        let detector: SatdDetectorWithItems = Default::default();
        // Should behave identically to a detector built via new().
        assert!(detector.detect("// TODO: x").len() == 1);
    }

    #[test]
    fn test_satd_detector_new() {
        let detector = SatdDetectorWithItems::new();
        let _ = detector;
    }

    #[test]
    fn test_detect_empty_source() {
        let detector = SatdDetectorWithItems::new();
        let items = detector.detect("");
        assert!(items.is_empty());
    }

    #[test]
    fn test_detect_no_satd() {
        let detector = SatdDetectorWithItems::new();
        let source = r#"
            fn clean_code() {
                let x = 1;
                let y = 2;
                x + y
            }
        "#;
        let items = detector.detect(source);
        assert!(items.is_empty());
    }

    #[test]
    fn test_detect_single_todo() {
        let detector = SatdDetectorWithItems::new();
        let source = "// TODO: implement this";
        let items = detector.detect(source);
        assert_eq!(items.len(), 1);
        assert_eq!(items[0].satd_type, "TODO");
        assert_eq!(items[0].line, 1);
        assert!(items[0].comment.contains("TODO"));
    }

    #[test]
    fn test_detect_single_fixme() {
        let detector = SatdDetectorWithItems::new();
        let source = "// FIXME: this is broken";
        let items = detector.detect(source);
        assert_eq!(items.len(), 1);
        assert_eq!(items[0].satd_type, "FIXME");
        assert_eq!(items[0].line, 1);
        assert!(items[0].comment.contains("FIXME"));
    }

    #[test]
    fn test_detect_single_hack() {
        let detector = SatdDetectorWithItems::new();
        let source = "// HACK: workaround for bug";
        let items = detector.detect(source);
        assert_eq!(items.len(), 1);
        assert_eq!(items[0].satd_type, "HACK");
        assert_eq!(items[0].line, 1);
        assert!(items[0].comment.contains("HACK"));
    }

    #[test]
    fn test_detect_multiple_satd_same_type() {
        let detector = SatdDetectorWithItems::new();
        let source = r#"
// TODO: first task
fn foo() {}
// TODO: second task
        "#;
        let items = detector.detect(source);
        assert_eq!(items.len(), 2);
        assert!(items.iter().all(|i| i.satd_type == "TODO"));
    }

    #[test]
    fn test_detect_multiple_satd_different_types() {
        let detector = SatdDetectorWithItems::new();
        let source = r#"
// TODO: implement feature
// FIXME: fix this bug
// HACK: temporary workaround
        "#;
        let items = detector.detect(source);
        assert_eq!(items.len(), 3);
        assert!(items.iter().any(|i| i.satd_type == "TODO"));
        assert!(items.iter().any(|i| i.satd_type == "FIXME"));
        assert!(items.iter().any(|i| i.satd_type == "HACK"));
    }

    #[test]
    fn test_detect_line_numbers_correct() {
        let detector = SatdDetectorWithItems::new();
        let source = "line 1\n// TODO: on line 2\nline 3\n// FIXME: on line 4";
        let items = detector.detect(source);
        assert_eq!(items.len(), 2);
        assert_eq!(items[0].line, 2);
        assert_eq!(items[1].line, 4);
    }

    #[test]
    fn test_detect_comment_trimmed() {
        let detector = SatdDetectorWithItems::new();
        let source = "    // TODO: trimmed comment    ";
        let items = detector.detect(source);
        assert_eq!(items.len(), 1);
        assert_eq!(items[0].comment, "// TODO: trimmed comment");
    }

    #[test]
    fn test_detect_todo_in_string_literal() {
        let detector = SatdDetectorWithItems::new();
        // TODO in a string literal is still detected (simple implementation)
        let source = r#"let msg = "TODO: this is in a string";"#;
        let items = detector.detect(source);
        assert_eq!(items.len(), 1);
    }

    #[test]
    fn test_detect_case_sensitive() {
        let detector = SatdDetectorWithItems::new();
        let source = r#"
// todo: lowercase (not detected)
// Todo: mixed case (not detected)
// TODO: uppercase (detected)
        "#;
        let items = detector.detect(source);
        // Only uppercase TODO is detected
        assert_eq!(items.len(), 1);
        assert_eq!(items[0].satd_type, "TODO");
    }

    #[test]
    fn test_detect_priority_todo_over_fixme_over_hack() {
        let detector = SatdDetectorWithItems::new();
        // When a line contains multiple markers, TODO is checked first
        let source = "// TODO FIXME HACK all on one line";
        let items = detector.detect(source);
        assert_eq!(items.len(), 1);
        assert_eq!(items[0].satd_type, "TODO"); // TODO takes priority
    }

    #[test]
    fn test_detect_large_file() {
        let detector = SatdDetectorWithItems::new();
        let mut source = String::new();
        for i in 1..=1000 {
            if i % 100 == 0 {
                source.push_str("// TODO: task\n");
            } else {
                source.push_str("let x = 1;\n");
            }
        }
        let items = detector.detect(&source);
        assert_eq!(items.len(), 10);
    }

    #[test]
    fn test_detect_multiline_comment_block() {
        let detector = SatdDetectorWithItems::new();
        let source = r#"
/*
 * TODO: implement this feature
 * FIXME: and fix this bug
 */
        "#;
        let items = detector.detect(source);
        assert_eq!(items.len(), 2);
    }

    #[test]
    fn test_detect_inline_with_code() {
        let detector = SatdDetectorWithItems::new();
        let source = "let x = 1; // TODO: refactor this";
        let items = detector.detect(source);
        assert_eq!(items.len(), 1);
        assert!(items[0].comment.contains("let x = 1"));
    }

    #[test]
    fn test_detect_empty_lines() {
        let detector = SatdDetectorWithItems::new();
        let source = "\n\n// TODO: surrounded by empty lines\n\n";
        let items = detector.detect(source);
        assert_eq!(items.len(), 1);
        assert_eq!(items[0].line, 3);
    }

    #[test]
    fn test_detect_unicode_content() {
        let detector = SatdDetectorWithItems::new();
        let source = "// TODO: 日本語テスト";
        let items = detector.detect(source);
        assert_eq!(items.len(), 1);
        assert!(items[0].comment.contains("日本語"));
    }

    #[test]
    fn test_detect_satd_in_doc_comment() {
        let detector = SatdDetectorWithItems::new();
        let source = "/// TODO: document this function";
        let items = detector.detect(source);
        assert_eq!(items.len(), 1);
        assert_eq!(items[0].satd_type, "TODO");
    }
}

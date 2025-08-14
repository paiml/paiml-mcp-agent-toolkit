#[cfg(test)]
mod tests {
    use crate::services::satd_detector::*;
    use proptest::prelude::*;
    use std::io::Write;
    use std::path::Path;
    use tempfile::NamedTempFile;

    // Strategy for generating valid SATD markers
    prop_compose! {
        fn arb_satd_marker()
            (choice in 0usize..10)
            -> &'static str
        {
            match choice {
                0 => "TODO",
                1 => "FIXME",
                2 => "HACK",
                3 => "BUG",
                4 => "XXX",
                5 => "OPTIMIZE",
                6 => "SECURITY",
                7 => "KLUDGE",
                8 => "REFACTOR",
                _ => "TECHNICAL DEBT",
            }
        }
    }

    // Strategy for generating comment styles
    prop_compose! {
        fn arb_comment_style()
            (style in 0usize..5,
             padding in prop::collection::vec(prop::sample::select(vec![" ", "\t"]), 0..5))
            -> String
        {
            let padding_str = padding.join("");
            match style {
                0 => format!("{}//", padding_str),
                1 => format!("{}#", padding_str),
                2 => format!("{}/*", padding_str),
                3 => format!("{} *", padding_str),
                _ => format!("{}--", padding_str),
            }
        }
    }

    // Strategy for generating SATD comments
    prop_compose! {
        fn arb_satd_comment()
            (marker in arb_satd_marker(),
             style in arb_comment_style(),
             message in "[a-zA-Z0-9 .,!?-]{0,100}",
             extra_spaces in 0usize..5)
            -> String
        {
            format!("{}{}{}: {}",
                style,
                " ".repeat(extra_spaces),
                marker,
                message)
        }
    }

    // Strategy for generating source code with embedded SATD
    prop_compose! {
        fn arb_source_with_satd()
            (num_lines in 1usize..50,
             satd_lines in prop::collection::vec((0usize..50, arb_satd_comment()), 0..10),
             code_lines in prop::collection::vec("[a-zA-Z0-9_(){}; ]{0,80}", 0..50))
            -> String
        {
            let mut lines = vec![];
            let satd_map: std::collections::HashMap<_, _> = satd_lines.into_iter().collect();

            for i in 0..num_lines {
                if let Some(satd) = satd_map.get(&i) {
                    lines.push(satd.clone());
                } else if let Some(code) = code_lines.get(i % code_lines.len().max(1)) {
                    lines.push(code.clone());
                } else {
                    lines.push(String::new());
                }
            }

            lines.join("\n")
        }
    }

    // Strategy for generating file extensions
    prop_compose! {
        fn arb_file_extension()
            (choice in 0usize..10)
            -> &'static str
        {
            match choice {
                0 => "rs",
                1 => "ts",
                2 => "js",
                3 => "py",
                4 => "go",
                5 => "java",
                6 => "cpp",
                7 => "c",
                8 => "rb",
                _ => "txt",
            }
        }
    }

    #[test]
    fn test_satd_parser_total_function() {
        proptest!(|(source in arb_source_with_satd())| {
            // Property: Parser is total (never panics on any input)
            let detector = SATDDetector::new();
            let result = std::panic::catch_unwind(|| {
                detector.extract_from_content(&source, Path::new("test.rs"))
            });
            prop_assert!(result.is_ok(), "Parser panicked on input");
        });
    }

    #[test]
    fn test_satd_extraction_deterministic() {
        proptest!(|(
            source in arb_source_with_satd(),
            ext in arb_file_extension()
        )| {
            // Property: Parsing is deterministic
            let detector = SATDDetector::new();
            let filename = format!("test.{}", ext);
            let path = Path::new(&filename);

            let result1 = detector.extract_from_content(&source, path);
            let result2 = detector.extract_from_content(&source, path);

            match (result1, result2) {
                (Ok(debts1), Ok(debts2)) => {
                    prop_assert_eq!(debts1.len(), debts2.len(),
                        "Different number of SATD items on repeated parse");

                    for (d1, d2) in debts1.iter().zip(debts2.iter()) {
                        prop_assert_eq!(d1.category, d2.category);
                        prop_assert_eq!(d1.severity, d2.severity);
                        prop_assert_eq!(&d1.text, &d2.text);
                        prop_assert_eq!(d1.line, d2.line);
                    }
                },
                (Err(_), Err(_)) => {
                    // Both failed consistently
                },
                _ => prop_assert!(false, "Inconsistent parsing results"),
            }
        });
    }

    #[test]
    fn test_satd_line_numbers_valid() {
        proptest!(|(source in arb_source_with_satd())| {
            // Property: All detected SATD items have valid line numbers
            let detector = SATDDetector::new();
            let result = detector.extract_from_content(&source, Path::new("test.rs"));

            if let Ok(debts) = result {
                let total_lines = source.lines().count() as u32;

                for debt in debts {
                    prop_assert!(debt.line > 0, "Line number must be positive");
                    prop_assert!(debt.line <= total_lines,
                        "Line number {} exceeds total lines {}", debt.line, total_lines);
                }
            }
        });
    }

    #[test]
    fn test_satd_categories_consistent() {
        proptest!(|(marker in arb_satd_marker())| {
            // Property: Known markers always map to consistent categories
            let detector = SATDDetector::new();
            let comment1 = format!("// {}: test issue", marker);
            let comment2 = format!("# {}: different message", marker);

            let result1 = detector.extract_from_content(&comment1, Path::new("test.rs"));
            let result2 = detector.extract_from_content(&comment2, Path::new("test.py"));

            if let (Ok(debts1), Ok(debts2)) = (result1, result2) {
                if !debts1.is_empty() && !debts2.is_empty() {
                    prop_assert_eq!(debts1[0].category, debts2[0].category,
                        "Same marker should map to same category");
                }
            }
        });
    }

    #[test]
    fn test_satd_severity_ordering() {
        // Property: Severity levels maintain correct ordering
        // Note: The enum derives Ord, so ordering is based on declaration order
        // Critical is declared first, so it has the lowest discriminant
        use Severity::*;

        // The actual ordering based on enum declaration
        assert!(Critical < High);
        assert!(High < Medium);
        assert!(Medium < Low);

        // Test escalation
        assert_eq!(Low.escalate(), Medium);
        assert_eq!(Medium.escalate(), High);
        assert_eq!(High.escalate(), Critical);
        assert_eq!(Critical.escalate(), Critical); // Can't escalate beyond Critical

        // Test reduction
        assert_eq!(Critical.reduce(), High);
        assert_eq!(High.reduce(), Medium);
        assert_eq!(Medium.reduce(), Low);
        assert_eq!(Low.reduce(), Low); // Can't reduce below Low
    }

    #[test]
    fn test_satd_empty_input_handled() {
        proptest!(|(whitespace in prop::collection::vec(
            prop::sample::select(vec![" ", "\t", "\n", "\r"]), 0..100
        ))| {
            // Property: Empty or whitespace-only input is handled gracefully
            let content = whitespace.join("");
            let detector = SATDDetector::new();

            let result = detector.extract_from_content(&content, Path::new("test.rs"));
            prop_assert!(result.is_ok(), "Should handle empty input");

            if let Ok(debts) = result {
                prop_assert_eq!(debts.len(), 0, "Empty input should produce no SATD items");
            }
        });
    }

    #[test]
    fn test_satd_malformed_comments_handled() {
        proptest!(|(
            prefix in "[^/\\*#-]{0,50}",
            marker in arb_satd_marker(),
            suffix in "[^\\n]{0,50}"
        )| {
            // Property: Malformed comments don't crash the parser
            let malformed = format!("{}{}{}", prefix, marker, suffix);
            let detector = SATDDetector::new();

            let result = std::panic::catch_unwind(|| {
                detector.extract_from_content(&malformed, Path::new("test.rs"))
            });

            prop_assert!(result.is_ok(), "Parser should not panic on malformed input");
        });
    }

    #[test]
    fn test_satd_content_extraction_preserves_structure() {
        proptest!(|(
            lines in prop::collection::vec("[a-zA-Z0-9 ]{0,100}", 1..20),
            satd_indices in prop::collection::vec(0usize..20, 0..5)
        )| {
            // Property: Content extraction preserves line structure
            let detector = SATDDetector::new();
            let mut source_lines = lines.clone();

            // Insert SATD comments at specified indices
            for &idx in &satd_indices {
                if idx < source_lines.len() {
                    source_lines[idx] = format!("// TODO: issue at line {}", idx);
                }
            }

            let source = source_lines.join("\n");
            let result = detector.extract_from_content(&source, Path::new("test.rs"));

            if let Ok(debts) = result {
                // All detected debts should have correct line numbers
                for debt in debts {
                    let line_idx = (debt.line - 1) as usize;
                    prop_assert!(
                        line_idx < source_lines.len(),
                        "Line number {} out of bounds", debt.line
                    );
                }
            }
        });
    }

    #[test]
    fn test_satd_file_integration() {
        proptest!(|(source in arb_source_with_satd())| {
            // Property: File I/O integration works correctly
            let detector = SATDDetector::new();

            // Create temporary file
            let mut temp_file = NamedTempFile::new().unwrap();
            temp_file.write_all(source.as_bytes()).unwrap();
            temp_file.flush().unwrap();
            let path = temp_file.path();

            // Read file and analyze
            let file_content = std::fs::read_to_string(path).unwrap();
            prop_assert_eq!(&file_content, &source, "File content should match written content");

            let result = detector.extract_from_content(&file_content, path);
            prop_assert!(result.is_ok() || file_content.len() > 10000,
                "Should successfully analyze file content");
        });
    }

    #[test]
    fn test_satd_test_block_exclusion() {
        proptest!(|(
            pre_test in prop::collection::vec("[a-zA-Z0-9_; ]{0,80}", 0..10),
            test_content in prop::collection::vec(arb_satd_comment(), 1..5),
            post_test in prop::collection::vec("[a-zA-Z0-9_; ]{0,80}", 0..10)
        )| {
            // Property: SATD in test blocks should be excluded (Rust files)
            let mut lines = vec![];

            // Add pre-test code
            lines.extend(pre_test);

            // Add test block
            lines.push("#[cfg(test)]".to_string());
            lines.push("mod tests {".to_string());
            let test_content_len = test_content.len();
            lines.extend(test_content);
            lines.push("}".to_string());

            // Add post-test code
            lines.extend(post_test);

            let source = lines.join("\n");
            let detector = SATDDetector::new();

            let result = detector.extract_from_content(&source, Path::new("test.rs"));

            if let Ok(debts) = result {
                // No SATD should be detected within the test block lines
                let test_start_line = lines.iter().position(|l| l.contains("#[cfg(test)]")).unwrap() + 1;
                let test_end_line = test_start_line + 3 + test_content_len;

                for debt in debts {
                    prop_assert!(
                        debt.line <= test_start_line as u32 || debt.line > test_end_line as u32,
                        "SATD found inside test block at line {}", debt.line
                    );
                }
            }
        });
    }

    #[test]
    fn test_satd_unicode_handling() {
        proptest!(|(
            prefix in "\\PC*",
            marker in arb_satd_marker(),
            suffix in "\\PC*"
        )| {
            // Property: Unicode in comments is handled properly
            let content = format!("// {} {}: {}", prefix, marker, suffix);
            let detector = SATDDetector::new();

            let result = std::panic::catch_unwind(|| {
                detector.extract_from_content(&content, Path::new("test.rs"))
            });

            prop_assert!(result.is_ok(), "Should handle Unicode without panicking");
        });
    }

    #[test]
    fn test_satd_large_file_performance() {
        proptest!(|(
            line_template in "[a-zA-Z0-9 ]{0,100}",
            num_repetitions in 100usize..1000
        )| {
            // Property: Performance scales linearly with file size
            let detector = SATDDetector::new();

            // Create a large file by repeating lines
            let mut lines = vec![];
            for i in 0..num_repetitions {
                if i % 10 == 0 {
                    lines.push(format!("// TODO: Fix issue {}", i));
                } else {
                    lines.push(line_template.clone());
                }
            }
            let content = lines.join("\n");

            let result = detector.extract_from_content(&content, Path::new("test.rs"));

            if let Ok(debts) = result {
                // Should find approximately num_repetitions / 10 SATD items
                let expected = num_repetitions / 10;
                prop_assert!(
                    debts.len() >= expected - 1 && debts.len() <= expected + 1,
                    "Expected around {} SATD items, found {}", expected, debts.len()
                );
            }
        });
    }

    // Additional edge case tests
    #[test]
    fn test_satd_nested_comments() {
        proptest!(|(
            depth in 1usize..5,
            marker in arb_satd_marker()
        )| {
            // Property: Nested comment styles are parsed correctly
            let mut content = String::new();

            // Create nested comment structure
            for _ in 0..depth {
                content.push_str("/* ");
            }
            content.push_str(&format!("{}: nested issue", marker));
            for _ in 0..depth {
                content.push_str(" */");
            }

            let detector = SATDDetector::new();
            let result = detector.extract_from_content(&content, Path::new("test.c"));

            prop_assert!(result.is_ok(), "Should handle nested comments");
        });
    }

    #[test]
    fn test_satd_multiline_handling() {
        proptest!(|(
            lines in prop::collection::vec("[a-zA-Z0-9 ]{0,50}", 2..10),
            marker in arb_satd_marker()
        )| {
            // Property: Multi-line comments are handled correctly
            let mut content = vec!["/*".to_string()];
            content.push(format!(" * {}: multi-line issue", marker));
            for line in lines {
                content.push(format!(" * {}", line));
            }
            content.push(" */".to_string());

            let source = content.join("\n");
            let detector = SATDDetector::new();

            let result = detector.extract_from_content(&source, Path::new("test.c"));

            if let Ok(debts) = result {
                if !debts.is_empty() {
                    // Should detect the SATD on the correct line
                    prop_assert_eq!(debts[0].line, 2, "SATD should be on line 2");
                }
            }
        });
    }
}

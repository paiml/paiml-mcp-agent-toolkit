#[cfg(test)]
mod dead_code_format_tests {
    use super::*;
    use crate::models::dead_code::{
        ConfidenceLevel, DeadCodeItem, DeadCodeResult, DeadCodeSummary, DeadCodeType,
        FileDeadCodeMetrics,
    };

    fn create_test_dead_code_result_with_items() -> DeadCodeResult {
        DeadCodeResult {
            summary: DeadCodeSummary {
                total_files_analyzed: 10,
                files_with_dead_code: 3,
                total_dead_lines: 150,
                dead_percentage: 15.0,
                dead_functions: 8,
                dead_classes: 2,
                dead_modules: 1,
                unreachable_blocks: 3,
            },
            files: vec![
                FileDeadCodeMetrics {
                    path: "src/module_a.rs".to_string(),
                    dead_lines: 50,
                    total_lines: 200,
                    dead_percentage: 25.0,
                    dead_functions: 3,
                    dead_classes: 1,
                    dead_modules: 0,
                    unreachable_blocks: 1,
                    dead_score: 35.5,
                    confidence: ConfidenceLevel::High,
                    items: vec![
                        DeadCodeItem {
                            name: "unused_helper".to_string(),
                            item_type: DeadCodeType::Function,
                            line: 42,
                            reason: "Never called".to_string(),
                        },
                        DeadCodeItem {
                            name: "OldClass".to_string(),
                            item_type: DeadCodeType::Class,
                            line: 100,
                            reason: "Never instantiated".to_string(),
                        },
                    ],
                },
                FileDeadCodeMetrics {
                    path: "src/module_b.rs".to_string(),
                    dead_lines: 60,
                    total_lines: 300,
                    dead_percentage: 20.0,
                    dead_functions: 4,
                    dead_classes: 0,
                    dead_modules: 1,
                    unreachable_blocks: 2,
                    dead_score: 28.0,
                    confidence: ConfidenceLevel::Medium,
                    items: vec![DeadCodeItem {
                        name: "legacy_code".to_string(),
                        item_type: DeadCodeType::UnreachableCode,
                        line: 50,
                        reason: "Unreachable after return".to_string(),
                    }],
                },
                FileDeadCodeMetrics {
                    path: "src/module_c.rs".to_string(),
                    dead_lines: 40,
                    total_lines: 250,
                    dead_percentage: 16.0,
                    dead_functions: 1,
                    dead_classes: 1,
                    dead_modules: 0,
                    unreachable_blocks: 0,
                    dead_score: 18.0,
                    confidence: ConfidenceLevel::Low,
                    items: vec![DeadCodeItem {
                        name: "dynamic_var".to_string(),
                        item_type: DeadCodeType::Variable,
                        line: 25,
                        reason: "Possibly unused".to_string(),
                    }],
                },
            ],
            total_files: 10,
            analyzed_files: 10,
        }
    }

    #[test]
    fn test_format_dead_code_as_json() {
        let result = create_test_dead_code_result_with_items();

        let json_output = format_dead_code_as_json(&result).expect("JSON formatting should work");

        // Verify it's valid JSON
        let parsed: serde_json::Value =
            serde_json::from_str(&json_output).expect("Should be valid JSON");

        // Check key fields are present
        assert!(parsed.get("summary").is_some());
        assert!(parsed.get("files").is_some());
        assert!(parsed.get("total_files").is_some());

        // Check summary values
        let summary = parsed.get("summary").unwrap();
        assert_eq!(summary.get("total_files_analyzed").unwrap(), 10);
        assert_eq!(summary.get("files_with_dead_code").unwrap(), 3);
        assert_eq!(summary.get("dead_functions").unwrap(), 8);
    }

    #[test]
    fn test_format_dead_code_as_sarif() {
        let result = create_test_dead_code_result_with_items();

        let sarif_output =
            format_dead_code_as_sarif(&result).expect("SARIF formatting should work");

        // Verify it's valid JSON
        let parsed: serde_json::Value =
            serde_json::from_str(&sarif_output).expect("Should be valid JSON");

        // Check SARIF schema version
        assert_eq!(parsed.get("version").unwrap(), "2.1.0");

        // Check tool information
        let runs = parsed.get("runs").unwrap().as_array().unwrap();
        assert!(!runs.is_empty());

        let tool = runs[0].get("tool").unwrap();
        let driver = tool.get("driver").unwrap();
        assert_eq!(driver.get("name").unwrap(), "pmat");

        // Check rules
        let rules = driver.get("rules").unwrap().as_array().unwrap();
        assert!(!rules.is_empty());
        assert_eq!(rules[0].get("id").unwrap(), "dead-code");
    }

    #[test]
    fn test_format_dead_code_as_markdown() {
        let result = create_test_dead_code_result_with_items();

        let markdown_output =
            format_dead_code_as_markdown(&result).expect("Markdown formatting should work");

        // Check sections are present
        assert!(markdown_output.contains("# Dead Code Analysis Report"));
        assert!(markdown_output.contains("## Summary"));
        assert!(markdown_output.contains("## Dead Code Breakdown"));
        assert!(markdown_output.contains("## File Details"));
        assert!(markdown_output.contains("## Recommendations"));

        // Check table content
        assert!(markdown_output.contains("Files Analyzed | 10"));
        assert!(markdown_output.contains("Files with Dead Code | 3"));
        assert!(markdown_output.contains("src/module_a.rs"));
    }

    #[test]
    fn test_format_dead_code_file_details_section() {
        let files = vec![
            FileDeadCodeMetrics {
                path: "src/test.rs".to_string(),
                dead_lines: 30,
                total_lines: 100,
                dead_percentage: 30.0,
                dead_functions: 2,
                dead_classes: 1,
                dead_modules: 0,
                unreachable_blocks: 0,
                dead_score: 25.0,
                confidence: ConfidenceLevel::High,
                items: vec![
                    DeadCodeItem {
                        name: "fn1".to_string(),
                        item_type: DeadCodeType::Function,
                        line: 10,
                        reason: "unused".to_string(),
                    },
                    DeadCodeItem {
                        name: "fn2".to_string(),
                        item_type: DeadCodeType::Function,
                        line: 20,
                        reason: "unused".to_string(),
                    },
                ],
            },
        ];

        let section = format_dead_code_file_details_section(&files);

        assert!(section.contains("## File Details"));
        assert!(section.contains("| File | Dead % | Dead Lines | Confidence | Items |"));
        assert!(section.contains("src/test.rs"));
        assert!(section.contains("30.0%"));
        assert!(section.contains("High"));
        assert!(section.contains("| 2 |")); // 2 items
    }

    #[test]
    fn test_format_dead_code_header() {
        let result = create_test_dead_code_result_with_items();
        let mut output = String::new();

        write_dead_code_header(&mut output, &result).expect("Header writing should work");

        assert!(output.contains("# Dead Code Analysis Summary"));
        assert!(output.contains("**Files analyzed**: 10"));
        assert!(output.contains("**Files with dead code**: 3"));
        assert!(output.contains("**Total dead lines**: 150"));
        assert!(output.contains("**Dead code percentage**: 15.00%"));
    }

    #[test]
    fn test_format_dead_code_sarif_levels() {
        // Test that confidence levels map to correct SARIF levels
        let result = DeadCodeResult {
            summary: DeadCodeSummary {
                total_files_analyzed: 3,
                files_with_dead_code: 3,
                total_dead_lines: 30,
                dead_percentage: 10.0,
                dead_functions: 3,
                dead_classes: 0,
                dead_modules: 0,
                unreachable_blocks: 0,
            },
            files: vec![
                FileDeadCodeMetrics {
                    path: "high.rs".to_string(),
                    dead_lines: 10,
                    total_lines: 100,
                    dead_percentage: 10.0,
                    dead_functions: 1,
                    dead_classes: 0,
                    dead_modules: 0,
                    unreachable_blocks: 0,
                    dead_score: 10.0,
                    confidence: ConfidenceLevel::High,
                    items: vec![DeadCodeItem {
                        name: "fn_high".to_string(),
                        item_type: DeadCodeType::Function,
                        line: 5,
                        reason: "unused".to_string(),
                    }],
                },
                FileDeadCodeMetrics {
                    path: "medium.rs".to_string(),
                    dead_lines: 10,
                    total_lines: 100,
                    dead_percentage: 10.0,
                    dead_functions: 1,
                    dead_classes: 0,
                    dead_modules: 0,
                    unreachable_blocks: 0,
                    dead_score: 10.0,
                    confidence: ConfidenceLevel::Medium,
                    items: vec![DeadCodeItem {
                        name: "fn_medium".to_string(),
                        item_type: DeadCodeType::Function,
                        line: 5,
                        reason: "unused".to_string(),
                    }],
                },
                FileDeadCodeMetrics {
                    path: "low.rs".to_string(),
                    dead_lines: 10,
                    total_lines: 100,
                    dead_percentage: 10.0,
                    dead_functions: 1,
                    dead_classes: 0,
                    dead_modules: 0,
                    unreachable_blocks: 0,
                    dead_score: 10.0,
                    confidence: ConfidenceLevel::Low,
                    items: vec![DeadCodeItem {
                        name: "fn_low".to_string(),
                        item_type: DeadCodeType::Function,
                        line: 5,
                        reason: "unused".to_string(),
                    }],
                },
            ],
            total_files: 3,
            analyzed_files: 3,
        };

        let sarif_output = format_dead_code_as_sarif(&result).expect("SARIF should work");
        let parsed: serde_json::Value = serde_json::from_str(&sarif_output).unwrap();

        let results = parsed["runs"][0]["results"].as_array().unwrap();

        // Check that we have 3 results with different levels
        assert_eq!(results.len(), 3);

        // Verify levels are mapped correctly
        let levels: Vec<&str> = results
            .iter()
            .map(|r| r["level"].as_str().unwrap())
            .collect();

        assert!(levels.contains(&"error")); // High
        assert!(levels.contains(&"warning")); // Medium
        assert!(levels.contains(&"note")); // Low
    }
}

// Extended Coverage Tests - SATD Formatting and Filtering


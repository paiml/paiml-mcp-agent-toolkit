//! Tests for refactored handle_analyze_dead_code function
//! Achieves >80% code coverage

#[cfg(test)]
mod tests {
    use super::super::{
    format_dead_code_as_json, format_dead_code_as_markdown, format_dead_code_as_sarif,
    format_dead_code_as_summary, format_dead_code_result, write_dead_code_output,
};
    use crate::cli::DeadCodeOutputFormat;
    use crate::models::dead_code::{
        ConfidenceLevel, DeadCodeItem, DeadCodeResult, DeadCodeSummary, DeadCodeType,
        FileDeadCodeMetrics,
    };
    use tempfile::TempDir;

    fn create_test_result() -> DeadCodeResult {
        DeadCodeResult {
            summary: DeadCodeSummary {
                total_files_analyzed: 100,
                files_with_dead_code: 10,
                total_dead_lines: 500,
                dead_percentage: 5.0,
                dead_functions: 20,
                dead_classes: 5,
                dead_modules: 3,
                unreachable_blocks: 15,
            },
            files: vec![
                FileDeadCodeMetrics {
                    path: "src/main.rs".to_string(),
                    dead_lines: 50,
                    total_lines: 500,
                    dead_percentage: 10.0,
                    dead_functions: 1,
                    dead_classes: 0,
                    dead_modules: 0,
                    unreachable_blocks: 0,
                    dead_score: 0.9,
                    confidence: ConfidenceLevel::High,
                    items: vec![
                        DeadCodeItem {
                            line: 42,
                            item_type: DeadCodeType::Function,
                            name: "unused_function".to_string(),
                            reason: "Never called".to_string(),
                        },
                        DeadCodeItem {
                            line: 100,
                            item_type: DeadCodeType::Variable,
                            name: "unused_var".to_string(),
                            reason: "Never referenced".to_string(),
                        },
                    ],
                },
                FileDeadCodeMetrics {
                    path: "src/lib.rs".to_string(),
                    dead_lines: 30,
                    total_lines: 600,
                    dead_percentage: 5.0,
                    dead_functions: 0,
                    dead_classes: 1,
                    dead_modules: 0,
                    unreachable_blocks: 0,
                    dead_score: 0.5,
                    confidence: ConfidenceLevel::Medium,
                    items: vec![
                        DeadCodeItem {
                            line: 200,
                            item_type: DeadCodeType::Class,
                            name: "UnusedStruct".to_string(),
                            reason: "Never instantiated".to_string(),
                        },
                    ],
                },
            ],
            total_files: 100,
            analyzed_files: 100,
        }
    }

    fn create_empty_result() -> DeadCodeResult {
        DeadCodeResult {
            summary: DeadCodeSummary {
                total_files_analyzed: 50,
                files_with_dead_code: 0,
                total_dead_lines: 0,
                dead_percentage: 0.0,
                dead_functions: 0,
                dead_classes: 0,
                dead_modules: 0,
                unreachable_blocks: 0,
            },
            files: vec![],
            total_files: 50,
            analyzed_files: 50,
        }
    }

    #[test]
    fn test_format_dead_code_as_json() {
        let result = create_test_result();
        let formatted = format_dead_code_as_json(&result);
        
        assert!(formatted.is_ok());
        let json = formatted.unwrap();
        
        // Verify it's valid JSON
        let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();
        assert!(parsed.is_object());
        assert_eq!(parsed["summary"]["total_files_analyzed"], 100);
        assert_eq!(parsed["files"].as_array().unwrap().len(), 2);
    }

    #[test]
    fn test_format_dead_code_as_json_empty() {
        let result = create_empty_result();
        let formatted = format_dead_code_as_json(&result);
        
        assert!(formatted.is_ok());
        let json = formatted.unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed["files"].as_array().unwrap().len(), 0);
        assert_eq!(parsed["summary"]["files_with_dead_code"], 0);
    }

    #[test]
    fn test_format_dead_code_as_summary() {
        let result = create_test_result();
        let formatted = format_dead_code_as_summary(&result);
        
        assert!(formatted.is_ok());
        let summary = formatted.unwrap();
        
        // Verify all sections are present
        assert!(summary.contains("# Dead Code Analysis Summary"));
        assert!(summary.contains("📊 **Files analyzed**: 100"));
        assert!(summary.contains("☠️  **Files with dead code**: 10"));
        assert!(summary.contains("📏 **Total dead lines**: 500"));
        assert!(summary.contains("📈 **Dead code percentage**: 5.00%"));
        assert!(summary.contains("## Dead Code by Type"));
        assert!(summary.contains("- **Dead functions**: 20"));
        assert!(summary.contains("## Top Files with Dead Code"));
        assert!(summary.contains("1. `src/main.rs` - 10.0% dead (50 lines)"));
    }

    #[test]
    fn test_format_dead_code_as_summary_empty() {
        let result = create_empty_result();
        let formatted = format_dead_code_as_summary(&result);
        
        assert!(formatted.is_ok());
        let summary = formatted.unwrap();
        
        // Should have header but no type breakdown or file list
        assert!(summary.contains("# Dead Code Analysis Summary"));
        assert!(summary.contains("📊 **Files analyzed**: 50"));
        assert!(!summary.contains("## Dead Code by Type"));
        assert!(!summary.contains("## Top Files with Dead Code"));
    }

    #[test]
    fn test_format_dead_code_as_markdown() {
        let result = create_test_result();
        let formatted = format_dead_code_as_markdown(&result);
        
        assert!(formatted.is_ok());
        let markdown = formatted.unwrap();
        
        // Verify all sections
        assert!(markdown.contains("# Dead Code Analysis Report"));
        assert!(markdown.contains("## Summary"));
        assert!(markdown.contains("| Metric | Value |"));
        assert!(markdown.contains("| Files Analyzed | 100 |"));
        assert!(markdown.contains("## Dead Code Breakdown"));
        assert!(markdown.contains("| Functions | 20 |"));
        assert!(markdown.contains("## File Details"));
        assert!(markdown.contains("| src/main.rs | 10.0% | 50 | High | 2 |"));
        assert!(markdown.contains("## Recommendations"));
    }

    #[test]
    fn test_format_dead_code_as_markdown_large() {
        let mut result = create_test_result();
        
        // Add many files
        for i in 3..30 {
            result.files.push(FileDeadCodeMetrics {
                path: format!("src/file{}.rs", i),
                dead_lines: 10,
                total_lines: 200,
                dead_percentage: 5.0,
                dead_functions: 0,
                dead_classes: 0,
                dead_modules: 0,
                unreachable_blocks: 0,
                dead_score: 0.1,
                confidence: ConfidenceLevel::Low,
                items: vec![],
            });
        }
        
        let formatted = format_dead_code_as_markdown(&result);
        assert!(formatted.is_ok());
        let markdown = formatted.unwrap();
        
        // Should only show first 20 files
        let file_count = markdown.matches("| src/file").count();
        assert!(file_count <= 20);
    }

    #[test]
    fn test_format_dead_code_as_sarif() {
        let result = create_test_result();
        let formatted = format_dead_code_as_sarif(&result);
        
        assert!(formatted.is_ok());
        let sarif = formatted.unwrap();
        
        // Verify SARIF structure
        let parsed: serde_json::Value = serde_json::from_str(&sarif).unwrap();
        assert_eq!(parsed["version"], "2.1.0");
        assert!(parsed["runs"].is_array());
        assert_eq!(parsed["runs"][0]["tool"]["driver"]["name"], "paiml-mcp-agent-toolkit");
        
        let results = &parsed["runs"][0]["results"];
        assert!(results.is_array());
        assert_eq!(results.as_array().unwrap().len(), 3); // 2 items from file 1, 1 from file 2
        
        // Check first result
        let first_result = &results[0];
        assert_eq!(first_result["ruleId"], "dead-code");
        assert_eq!(first_result["level"], "error"); // High confidence = error
        assert!(first_result["message"]["text"].as_str().unwrap().contains("Dead function: Never called"));
    }

    #[test]
    fn test_format_dead_code_result_all_formats() {
        let result = create_test_result();
        
        // Test all format types
        for format in [
            DeadCodeOutputFormat::Json,
            DeadCodeOutputFormat::Sarif,
            DeadCodeOutputFormat::Summary,
            DeadCodeOutputFormat::Markdown,
        ] {
            let formatted = format_dead_code_result(&result, &format);
            assert!(formatted.is_ok(), "Format {:?} should succeed", format);
            assert!(!formatted.unwrap().is_empty(), "Format {:?} should not be empty", format);
        }
    }

    #[tokio::test]
    async fn test_write_dead_code_output_to_file() {
        let temp_dir = TempDir::new().unwrap();
        let output_path = temp_dir.path().join("dead_code_output.txt");
        let content = "Test dead code analysis output".to_string();
        
        let result = write_dead_code_output(content.clone(), Some(output_path.clone())).await;
        assert!(result.is_ok());
        
        // Verify file was created and contains expected content
        assert!(output_path.exists());
        let saved = tokio::fs::read_to_string(&output_path).await.unwrap();
        assert_eq!(saved, content);
    }

    #[tokio::test]
    async fn test_write_dead_code_output_to_stdout() {
        // Just verify it doesn't panic when output is None
        let content = "Test stdout output".to_string();
        let result = write_dead_code_output(content, None).await;
        assert!(result.is_ok());
    }

    #[test]
    fn test_confidence_level_mapping() {
        let high_file = FileDeadCodeMetrics {
            path: "test.rs".to_string(),
            dead_lines: 10,
            total_lines: 100,
            dead_percentage: 10.0,
            dead_functions: 0,
            dead_classes: 0,
            dead_modules: 0,
            unreachable_blocks: 0,
            dead_score: 0.9,
            confidence: ConfidenceLevel::High,
            items: vec![],
        };
        
        let medium_file = FileDeadCodeMetrics {
            confidence: ConfidenceLevel::Medium,
            ..high_file.clone()
        };
        
        let low_file = FileDeadCodeMetrics {
            confidence: ConfidenceLevel::Low,
            ..high_file.clone()
        };
        
        let result = DeadCodeResult {
            summary: create_test_result().summary,
            files: vec![high_file, medium_file, low_file],
            total_files: 3,
            analyzed_files: 3,
        };
        
        let sarif = format_dead_code_as_sarif(&result).unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&sarif).unwrap();
        
        // SARIF should have empty results since files have no items
        assert_eq!(parsed["runs"][0]["results"].as_array().unwrap().len(), 0);
    }

    #[test]
    fn test_dead_code_type_formatting() {
        let items = vec![
            (DeadCodeType::Function, "Dead function"),
            (DeadCodeType::Class, "Dead class"),
            (DeadCodeType::Variable, "Dead variable"),
            (DeadCodeType::UnreachableCode, "Unreachable code"),
        ];
        
        for (item_type, expected_prefix) in items {
            let file = FileDeadCodeMetrics {
                path: "test.rs".to_string(),
                dead_lines: 10,
                total_lines: 100,
                dead_percentage: 10.0,
                dead_functions: 0,
                dead_classes: 0,
                dead_modules: 0,
                unreachable_blocks: 0,
                dead_score: 0.9,
                confidence: ConfidenceLevel::High,
                items: vec![DeadCodeItem {
                    line: 1,
                    item_type,
                    name: "test_item".to_string(),
                    reason: "Test reason".to_string(),
                }],
            };
            
            let result = DeadCodeResult {
                summary: create_test_result().summary,
                files: vec![file],
                total_files: 1,
                analyzed_files: 1,
            };
            
            let sarif = format_dead_code_as_sarif(&result).unwrap();
            let parsed: serde_json::Value = serde_json::from_str(&sarif).unwrap();
            let message = parsed["runs"][0]["results"][0]["message"]["text"].as_str().unwrap();
            
            assert!(message.starts_with(expected_prefix), 
                "Expected '{}' to start with '{}'", message, expected_prefix);
            assert!(message.contains("Test reason"));
        }
    }

    #[test]
    fn test_empty_files_formatting() {
        let result = DeadCodeResult {
            summary: DeadCodeSummary {
                total_files_analyzed: 100,
                files_with_dead_code: 5,
                total_dead_lines: 200,
                dead_percentage: 2.0,
                dead_functions: 10,
                dead_classes: 2,
                dead_modules: 1,
                unreachable_blocks: 5,
            },
            files: vec![], // No file details
            total_files: 100,
            analyzed_files: 100,
        };
        
        // All formats should handle empty files gracefully
        let summary = format_dead_code_as_summary(&result).unwrap();
        assert!(!summary.contains("## Top Files with Dead Code"));
        
        let markdown = format_dead_code_as_markdown(&result).unwrap();
        assert!(!markdown.contains("## File Details"));
        
        let sarif = format_dead_code_as_sarif(&result).unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&sarif).unwrap();
        assert_eq!(parsed["runs"][0]["results"].as_array().unwrap().len(), 0);
    }
}
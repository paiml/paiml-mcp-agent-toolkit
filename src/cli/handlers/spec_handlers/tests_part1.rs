mod tests {
    use super::*;
    use crate::services::spec_parser::{
        AcceptanceCriterion, ClaimCategory, CodeExample, TestRequirement, ValidationClaim,
    };
    use std::path::PathBuf;
    use tempfile::TempDir;

    // ============================================================================
    // Helper functions to create test specs
    // ============================================================================

    fn create_empty_spec() -> ParsedSpec {
        ParsedSpec {
            path: PathBuf::new(),
            title: String::new(),
            issue_refs: vec![],
            status: None,
            claims: vec![],
            code_examples: vec![],
            acceptance_criteria: vec![],
            test_requirements: vec![],
            raw_content: String::new(),
        }
    }

    fn create_minimal_spec(title: &str) -> ParsedSpec {
        ParsedSpec {
            path: PathBuf::from("test.md"),
            title: title.to_string(),
            issue_refs: vec![],
            status: None,
            claims: vec![],
            code_examples: vec![],
            acceptance_criteria: vec![],
            test_requirements: vec![],
            raw_content: String::new(),
        }
    }

    fn create_full_spec() -> ParsedSpec {
        ParsedSpec {
            path: PathBuf::from("full-spec.md"),
            title: "Full Specification".to_string(),
            issue_refs: vec!["#123".to_string(), "#456".to_string()],
            status: Some("Draft".to_string()),
            claims: (0..20)
                .map(|i| ValidationClaim {
                    id: format!("C-{}", i),
                    text: format!("Claim {} with [citation]", i),
                    line: i + 1,
                    category: ClaimCategory::Implementation,
                    automatable: false,
                    validation_cmd: None,
                    expected_pattern: None,
                })
                .collect(),
            code_examples: (0..5)
                .map(|i| CodeExample {
                    language: "rust".to_string(),
                    code: format!("fn example_{}() {{}}", i),
                    line: i * 10,
                    executable: true,
                })
                .collect(),
            acceptance_criteria: (0..10)
                .map(|i| AcceptanceCriterion {
                    text: format!("AC-{}: Criterion {}", i, i),
                    complete: i % 2 == 0,
                    line: i + 100,
                })
                .collect(),
            test_requirements: (0..5)
                .map(|i| TestRequirement {
                    text: format!("Test requirement {}", i),
                    test_type: "unit".to_string(),
                    code_path: Some(format!("src/test_{}.rs", i)),
                })
                .collect(),
            // Include 5+ citations for full score
            raw_content: "See [1] and [2] for background. As noted in [3], \
                          this follows [4] methodology per [5] recommendations."
                .to_string(),
        }
    }

    fn create_partial_spec() -> ParsedSpec {
        ParsedSpec {
            path: PathBuf::from("partial-spec.md"),
            title: "Partial Specification".to_string(),
            issue_refs: vec!["#789".to_string()],
            status: Some("Active".to_string()),
            claims: (0..5)
                .map(|i| ValidationClaim {
                    id: format!("C-{}", i),
                    text: format!("Claim {}", i),
                    line: i + 1,
                    category: ClaimCategory::Testing,
                    automatable: true,
                    validation_cmd: Some("cargo test".to_string()),
                    expected_pattern: None,
                })
                .collect(),
            code_examples: vec![CodeExample {
                language: "rust".to_string(),
                code: "fn main() {}".to_string(),
                line: 1,
                executable: true,
            }],
            acceptance_criteria: vec![AcceptanceCriterion {
                text: "Single criterion".to_string(),
                complete: false,
                line: 1,
            }],
            test_requirements: vec![],
            raw_content: String::new(),
        }
    }

    // ============================================================================
    // Tests for calculate_spec_score
    // ============================================================================

    #[test]
    fn test_calculate_spec_score_empty() {
        let spec = create_empty_spec();
        let score = calculate_spec_score(&spec);
        assert_eq!(score, 0.0);
    }

    #[test]
    fn test_calculate_spec_score_title_only() {
        let spec = create_minimal_spec("Test Title");
        let score = calculate_spec_score(&spec);
        // Title adds 5 points
        assert_eq!(score, 5.0);
    }

    #[test]
    fn test_calculate_spec_score_issue_refs() {
        let mut spec = create_empty_spec();
        spec.issue_refs = vec!["#123".to_string()];
        let score = calculate_spec_score(&spec);
        // Issue refs add 10 points
        assert_eq!(score, 10.0);
    }

    #[test]
    fn test_calculate_spec_score_code_examples() {
        let mut spec = create_empty_spec();
        spec.code_examples = vec![
            CodeExample {
                language: "rust".to_string(),
                code: "fn a() {}".to_string(),
                line: 1,
                executable: true,
            },
            CodeExample {
                language: "rust".to_string(),
                code: "fn b() {}".to_string(),
                line: 2,
                executable: true,
            },
        ];
        let score = calculate_spec_score(&spec);
        // 2 examples * 4 pts each = 8 pts
        assert_eq!(score, 8.0);
    }

    #[test]
    fn test_calculate_spec_score_code_examples_capped_at_5() {
        let mut spec = create_empty_spec();
        // Add 10 code examples, but only 5 should count
        spec.code_examples = (0..10)
            .map(|i| CodeExample {
                language: "rust".to_string(),
                code: format!("fn example_{}() {{}}", i),
                line: i,
                executable: true,
            })
            .collect();
        let score = calculate_spec_score(&spec);
        // Max 5 examples * 4 pts each = 20 pts
        assert_eq!(score, 20.0);
    }

    #[test]
    fn test_calculate_spec_score_acceptance_criteria() {
        let mut spec = create_empty_spec();
        spec.acceptance_criteria = vec![
            AcceptanceCriterion {
                text: "AC-1".to_string(),
                complete: false,
                line: 1,
            },
            AcceptanceCriterion {
                text: "AC-2".to_string(),
                complete: true,
                line: 2,
            },
            AcceptanceCriterion {
                text: "AC-3".to_string(),
                complete: false,
                line: 3,
            },
        ];
        let score = calculate_spec_score(&spec);
        // 3 criteria * 2.5 pts each = 7.5 pts
        assert_eq!(score, 7.5);
    }

    #[test]
    fn test_calculate_spec_score_acceptance_criteria_capped_at_10() {
        let mut spec = create_empty_spec();
        // Add 15 acceptance criteria, but only 10 should count
        spec.acceptance_criteria = (0..15)
            .map(|i| AcceptanceCriterion {
                text: format!("AC-{}", i),
                complete: false,
                line: i,
            })
            .collect();
        let score = calculate_spec_score(&spec);
        // Max 10 criteria * 2.5 pts each = 25 pts
        assert_eq!(score, 25.0);
    }

    #[test]
    fn test_calculate_spec_score_claims() {
        let mut spec = create_empty_spec();
        spec.claims = (0..10)
            .map(|i| ValidationClaim {
                id: format!("C-{}", i),
                text: format!("Claim {}", i),
                line: i,
                category: ClaimCategory::Implementation,
                automatable: false,
                validation_cmd: None,
                expected_pattern: None,
            })
            .collect();
        let score = calculate_spec_score(&spec);
        // 10 claims = 10 pts
        assert_eq!(score, 10.0);
    }

    #[test]
    fn test_calculate_spec_score_claims_capped_at_15() {
        let mut spec = create_empty_spec();
        // Add 30 claims, but only 15 should count
        spec.claims = (0..30)
            .map(|i| ValidationClaim {
                id: format!("C-{}", i),
                text: format!("Claim {}", i),
                line: i,
                category: ClaimCategory::Implementation,
                automatable: false,
                validation_cmd: None,
                expected_pattern: None,
            })
            .collect();
        let score = calculate_spec_score(&spec);
        // Max 15 claims = 15 pts
        assert_eq!(score, 15.0);
    }

    #[test]
    fn test_calculate_spec_score_test_requirements() {
        let mut spec = create_empty_spec();
        spec.test_requirements = vec![
            TestRequirement {
                text: "Unit test".to_string(),
                test_type: "unit".to_string(),
                code_path: None,
            },
            TestRequirement {
                text: "Integration test".to_string(),
                test_type: "integration".to_string(),
                code_path: None,
            },
        ];
        let score = calculate_spec_score(&spec);
        // 2 requirements * 3 pts each = 6 pts
        assert_eq!(score, 6.0);
    }

    #[test]
    fn test_calculate_spec_score_test_requirements_capped_at_5() {
        let mut spec = create_empty_spec();
        // Add 10 test requirements, but only 5 should count
        spec.test_requirements = (0..10)
            .map(|i| TestRequirement {
                text: format!("Test {}", i),
                test_type: "unit".to_string(),
                code_path: None,
            })
            .collect();
        let score = calculate_spec_score(&spec);
        // Max 5 requirements * 3 pts each = 15 pts
        assert_eq!(score, 15.0);
    }

    #[test]
    fn test_calculate_spec_score_full_spec() {
        let spec = create_full_spec();
        let score = calculate_spec_score(&spec);
        // Title: 5
        // Issue refs: 10
        // Code examples (5): 20
        // Acceptance criteria (10): 25
        // Claims (20, capped at 15): 15
        // Test requirements (5): 15
        // Citations (5): 10
        // Total: 100
        assert_eq!(score, 100.0);
    }

    #[test]
    fn test_calculate_spec_score_capped_at_100() {
        let mut spec = create_full_spec();
        // Add even more items
        spec.claims = (0..50)
            .map(|i| ValidationClaim {
                id: format!("C-{}", i),
                text: format!("Claim {}", i),
                line: i,
                category: ClaimCategory::Implementation,
                automatable: false,
                validation_cmd: None,
                expected_pattern: None,
            })
            .collect();
        let score = calculate_spec_score(&spec);
        // Should not exceed 100
        assert!(score <= 100.0);
    }

    // ============================================================================
    // Tests for format_spec_score_text
    // ============================================================================

    #[test]
    fn test_format_spec_score_text_basic() {
        let spec = create_minimal_spec("My Spec");
        let output = format_spec_score_text(&spec, 75.0, false);

        assert!(output.contains("Specification Score"));
        assert!(output.contains("Title: My Spec"));
        assert!(output.contains("Score: 75.0/100"));
        assert!(output.contains("FAIL"));
    }

    #[test]
    fn test_format_spec_score_text_passing() {
        let spec = create_full_spec();
        let output = format_spec_score_text(&spec, 97.0, false);

        assert!(output.contains("PASS"));
        assert!(!output.contains("FAIL"));
    }

    #[test]
    fn test_format_spec_score_text_failing() {
        let spec = create_partial_spec();
        let output = format_spec_score_text(&spec, 50.0, false);

        assert!(output.contains("FAIL"));
        assert!(output.contains("needs"));
    }

    #[test]
    fn test_format_spec_score_text_verbose() {
        let spec = create_full_spec();
        let output = format_spec_score_text(&spec, 95.0, true);

        assert!(output.contains("Claims:"));
        assert!(output.contains("Code Examples:"));
        assert!(output.contains("Acceptance Criteria:"));
        assert!(output.contains("Issue Refs:"));
    }

    #[test]
    fn test_format_spec_score_text_non_verbose() {
        let spec = create_full_spec();
        let output = format_spec_score_text(&spec, 95.0, false);

        // Verbose content should not be present
        assert!(!output.contains("Claims:"));
    }

    #[test]
    fn test_format_spec_score_text_empty_title() {
        let spec = create_empty_spec();
        let output = format_spec_score_text(&spec, 0.0, false);

        // Should not contain "Title:" if title is empty
        assert!(!output.contains("Title:"));
    }

    // ============================================================================
    // Tests for format_spec_score_json
    // ============================================================================

    #[test]
    fn test_format_spec_score_json_basic() {
        let spec = create_minimal_spec("JSON Test Spec");
        let result = format_spec_score_json(&spec, 80.0).unwrap();

        let json: serde_json::Value = serde_json::from_str(&result).unwrap();
        assert_eq!(json["title"], "JSON Test Spec");
        assert_eq!(json["score"], 80.0);
        assert_eq!(json["passing"], false);
    }

    #[test]
    fn test_format_spec_score_json_passing() {
        let spec = create_full_spec();
        let result = format_spec_score_json(&spec, 98.0).unwrap();

        let json: serde_json::Value = serde_json::from_str(&result).unwrap();
        assert_eq!(json["passing"], true);
    }

    #[test]
    fn test_format_spec_score_json_counts() {
        let spec = create_full_spec();
        let result = format_spec_score_json(&spec, 100.0).unwrap();

        let json: serde_json::Value = serde_json::from_str(&result).unwrap();
        assert_eq!(json["claims"], 20);
        assert_eq!(json["code_examples"], 5);
        assert_eq!(json["acceptance_criteria"], 10);
    }

    #[test]
    fn test_format_spec_score_json_issue_refs() {
        let spec = create_full_spec();
        let result = format_spec_score_json(&spec, 100.0).unwrap();

        let json: serde_json::Value = serde_json::from_str(&result).unwrap();
        let issue_refs = json["issue_refs"].as_array().unwrap();
        assert_eq!(issue_refs.len(), 2);
        assert!(issue_refs.contains(&serde_json::json!("#123")));
        assert!(issue_refs.contains(&serde_json::json!("#456")));
    }

    #[test]
    fn test_format_spec_score_json_valid_json() {
        let spec = create_partial_spec();
        let result = format_spec_score_json(&spec, 50.0).unwrap();

        // Ensure the result is valid JSON by parsing it
        let parsed: Result<serde_json::Value, _> = serde_json::from_str(&result);
        assert!(parsed.is_ok());
    }

    // ============================================================================
    // Tests for format_spec_score_markdown
    // ============================================================================

    #[test]
    fn test_format_spec_score_markdown_basic() {
        let spec = create_minimal_spec("Markdown Spec");
        let output = format_spec_score_markdown(&spec, 75.0);

        assert!(output.contains("# Specification Score Report"));
        assert!(output.contains("**Title:** Markdown Spec"));
        assert!(output.contains("| Score | 75.0/100 |"));
    }

    #[test]
    fn test_format_spec_score_markdown_table_format() {
        let spec = create_full_spec();
        let output = format_spec_score_markdown(&spec, 100.0);

        // Check table headers and structure
        assert!(output.contains("| Metric | Value |"));
        assert!(output.contains("|--------|-------|"));
    }

    #[test]
    fn test_format_spec_score_markdown_passing() {
        let spec = create_full_spec();
        let output = format_spec_score_markdown(&spec, 98.0);

        assert!(output.contains("PASS"));
    }

    #[test]
    fn test_format_spec_score_markdown_failing() {
        let spec = create_partial_spec();
        let output = format_spec_score_markdown(&spec, 50.0);

        assert!(output.contains("FAIL"));
    }

    #[test]
    fn test_format_spec_score_markdown_counts() {
        let spec = create_full_spec();
        let output = format_spec_score_markdown(&spec, 100.0);

        assert!(output.contains("| Claims | 20 |"));
        assert!(output.contains("| Code Examples | 5 |"));
        assert!(output.contains("| Acceptance Criteria | 10 |"));
    }

    #[test]
    fn test_format_spec_score_markdown_empty_title() {
        let spec = create_empty_spec();
        let output = format_spec_score_markdown(&spec, 0.0);

        // Should not contain title line if title is empty
        assert!(!output.contains("**Title:**"));
    }

    // ============================================================================
    // Tests for handle_spec_create (async)
    // ============================================================================

    #[tokio::test]
    async fn test_handle_spec_create_basic() {
        let temp_dir = TempDir::new().unwrap();
        let output_path = temp_dir.path();

        let result =
            handle_spec_create("Test Feature Spec", None, None, Some(output_path)).await;

        assert!(result.is_ok());

        let expected_file = output_path.join("test-feature-spec.md");
        assert!(expected_file.exists());

        let content = fs::read_to_string(&expected_file).unwrap();
        assert!(content.contains("# Test Feature Spec"));
        assert!(content.contains("issue_refs:"));
    }

    #[tokio::test]
    async fn test_handle_spec_create_with_issue() {
        let temp_dir = TempDir::new().unwrap();
        let output_path = temp_dir.path();

        let result =
            handle_spec_create("My Feature", Some("#999"), None, Some(output_path)).await;

        assert!(result.is_ok());

        let expected_file = output_path.join("my-feature.md");
        let content = fs::read_to_string(&expected_file).unwrap();
        assert!(content.contains("#999"));
    }

    #[tokio::test]

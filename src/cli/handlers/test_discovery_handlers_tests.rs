#[cfg_attr(coverage_nightly, coverage(off))]
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_categorize_failure_timeout() {
        let category = categorize_failure("test timed out after 60 seconds");
        assert_eq!(category, FailureCategory::Timeout);

        let category = categorize_failure("Timeout waiting for response");
        assert_eq!(category, FailureCategory::Timeout);
    }

    #[test]
    fn test_categorize_failure_compile_error() {
        let category = categorize_failure("failed to compile: unresolved import `foo`");
        assert_eq!(category, FailureCategory::CompileError);

        let category = categorize_failure("error: unresolved import `bar::baz`");
        assert_eq!(category, FailureCategory::CompileError);
    }

    #[test]
    fn test_categorize_failure_runtime_error() {
        let category = categorize_failure("thread 'main' panicked at 'oops'");
        assert_eq!(category, FailureCategory::RuntimeError);

        let category = categorize_failure("thread panicked while executing test");
        assert_eq!(category, FailureCategory::RuntimeError);
    }

    #[test]
    fn test_categorize_failure_assertion() {
        let category = categorize_failure("assertion failed: expected 1, got 2");
        assert_eq!(category, FailureCategory::AssertionFailure);

        let category = categorize_failure("expected value to be true");
        assert_eq!(category, FailureCategory::AssertionFailure);
    }

    #[test]
    fn test_categorize_failure_unknown() {
        let category = categorize_failure("something weird happened");
        assert_eq!(category, FailureCategory::Unknown);
    }

    #[test]
    fn test_extract_pattern_panic() {
        let pattern = extract_pattern("thread 'test' panicked at 'message here'\nmore stuff");
        assert_eq!(pattern, "panicked at 'message here'");
    }

    #[test]
    fn test_extract_pattern_assertion() {
        let pattern = extract_pattern("assertion failed: x != y");
        assert_eq!(pattern, "assertion failed");
    }

    #[test]
    fn test_extract_pattern_timeout() {
        let pattern = extract_pattern("test timed out after 60s");
        assert_eq!(pattern, "test timeout");
    }

    #[test]
    fn test_extract_pattern_default() {
        let pattern = extract_pattern("some random error message that is quite long");
        assert_eq!(pattern, "some random error message that is quite long");
    }

    #[test]
    fn test_extract_number_passed() {
        let line = "test result: ok. 42 passed; 3 failed; 10 ignored; 5 filtered out";
        assert_eq!(extract_number(line, "passed"), Some(42));
    }

    #[test]
    fn test_extract_number_failed() {
        let line = "test result: ok. 42 passed; 3 failed; 10 ignored; 5 filtered out";
        assert_eq!(extract_number(line, "failed"), Some(3));
    }

    #[test]
    fn test_extract_number_ignored() {
        let line = "test result: ok. 42 passed; 3 failed; 10 ignored; 5 filtered out";
        assert_eq!(extract_number(line, "ignored"), Some(10));
    }

    #[test]
    fn test_extract_number_not_found() {
        let line = "no numbers here";
        assert_eq!(extract_number(line, "passed"), None);
    }

    #[test]
    fn test_parse_test_summary() {
        let stdout = "test result: ok. 100 passed; 5 failed; 20 ignored; 10 filtered out";
        let stderr = "";
        let (passed, failed, ignored) = parse_test_summary(stdout, stderr);
        assert_eq!(passed, 100);
        assert_eq!(failed, 5);
        assert_eq!(ignored, 20);
    }

    #[test]
    fn test_categorize_failures_groups() {
        let failures = vec![
            TestFailure {
                name: "test1".to_string(),
                file: PathBuf::from("test.rs"),
                line: Some(10),
                reason: "test timed out".to_string(),
                category: FailureCategory::Timeout,
                duration_ms: Some(60000),
            },
            TestFailure {
                name: "test2".to_string(),
                file: PathBuf::from("test.rs"),
                line: Some(20),
                reason: "test timed out".to_string(),
                category: FailureCategory::Timeout,
                duration_ms: Some(60000),
            },
            TestFailure {
                name: "test3".to_string(),
                file: PathBuf::from("test.rs"),
                line: Some(30),
                reason: "assertion failed".to_string(),
                category: FailureCategory::AssertionFailure,
                duration_ms: Some(100),
            },
        ];

        let groups = categorize_failures(&failures);

        // Should have 2 groups: timeout and assertion
        assert_eq!(groups.len(), 2);

        // Find timeout group
        let timeout_group = groups.iter().find(|g| g.root_cause.contains("Timeout"));
        assert!(timeout_group.is_some());
        assert_eq!(timeout_group.unwrap().tests.len(), 2);

        // Find assertion group
        let assertion_group = groups.iter().find(|g| g.root_cause.contains("Assertion"));
        assert!(assertion_group.is_some());
        assert_eq!(assertion_group.unwrap().tests.len(), 1);
    }

    #[test]
    fn test_failure_group_priority() {
        let failures = vec![TestFailure {
            name: "test1".to_string(),
            file: PathBuf::from("test.rs"),
            line: Some(10),
            reason: "assertion failed".to_string(),
            category: FailureCategory::AssertionFailure,
            duration_ms: None,
        }];

        let groups = categorize_failures(&failures);
        assert_eq!(groups.len(), 1);
        // Assertion failures should be priority 1 (fix now)
        assert_eq!(groups[0].priority, 1);
    }

    #[test]
    fn test_discovery_report_serialization() {
        let report = DiscoveryReport {
            total_tests: 100,
            failures: 5,
            test_failures: vec![],
            timestamp: "2025-01-01T00:00:00Z".to_string(),
            command: "cargo test".to_string(),
        };

        let json = serde_json::to_string(&report).unwrap();
        let parsed: DiscoveryReport = serde_json::from_str(&json).unwrap();

        assert_eq!(parsed.total_tests, 100);
        assert_eq!(parsed.failures, 5);
    }

    #[test]
    fn test_categorization_report_serialization() {
        let report = CategorizationReport {
            total_failures: 10,
            groups: vec![],
            timestamp: "2025-01-01T00:00:00Z".to_string(),
        };

        let json = serde_json::to_string(&report).unwrap();
        let parsed: CategorizationReport = serde_json::from_str(&json).unwrap();

        assert_eq!(parsed.total_failures, 10);
    }

    // Phase 5: CreateTickets tests (V2 feature)

    #[test]
    fn test_generate_issue_templates_creates_tickets() {
        let report = CategorizationReport {
            total_failures: 5,
            groups: vec![FailureGroup {
                root_cause: "Timeout: slow tests".to_string(),
                ignore_reason: "GH-98: Slow test".to_string(),
                priority: 3,
                tests: vec![
                    TestFailure {
                        name: "test_slow_1".to_string(),
                        file: PathBuf::from("test.rs"),
                        line: Some(10),
                        reason: "timed out".to_string(),
                        category: FailureCategory::Timeout,
                        duration_ms: Some(60000),
                    },
                    TestFailure {
                        name: "test_slow_2".to_string(),
                        file: PathBuf::from("test.rs"),
                        line: Some(20),
                        reason: "timed out".to_string(),
                        category: FailureCategory::Timeout,
                        duration_ms: Some(60000),
                    },
                ],
            }],
            timestamp: "2025-01-01T00:00:00Z".to_string(),
        };

        let tickets = generate_issue_templates(&report, None);

        assert_eq!(tickets.len(), 1);
        assert!(tickets[0].title.contains("Timeout"));
        assert_eq!(tickets[0].test_count, 2);
        assert!(tickets[0].body.contains("test_slow_1"));
        assert!(tickets[0].body.contains("test_slow_2"));
    }

    #[test]
    fn test_generate_issue_templates_applies_labels() {
        let report = CategorizationReport {
            total_failures: 1,
            groups: vec![FailureGroup {
                root_cause: "Assertion failure".to_string(),
                ignore_reason: "Fix needed".to_string(),
                priority: 1,
                tests: vec![TestFailure {
                    name: "test_assert".to_string(),
                    file: PathBuf::from("test.rs"),
                    line: None,
                    reason: "assertion failed".to_string(),
                    category: FailureCategory::AssertionFailure,
                    duration_ms: None,
                }],
            }],
            timestamp: "2025-01-01T00:00:00Z".to_string(),
        };

        let custom_labels = vec!["bug".to_string(), "urgent".to_string()];
        let tickets = generate_issue_templates(&report, Some(custom_labels));

        assert!(tickets[0].labels.contains(&"test-quality".to_string()));
        assert!(tickets[0].labels.contains(&"bug".to_string()));
        assert!(tickets[0].labels.contains(&"urgent".to_string()));
        assert!(tickets[0].labels.contains(&"priority-1".to_string()));
    }

    #[test]
    fn test_generate_issue_templates_limits_test_list() {
        let tests: Vec<TestFailure> = (0..30)
            .map(|i| TestFailure {
                name: format!("test_{}", i),
                file: PathBuf::from("test.rs"),
                line: Some(i as u32),
                reason: "failed".to_string(),
                category: FailureCategory::Unknown,
                duration_ms: None,
            })
            .collect();

        let report = CategorizationReport {
            total_failures: 30,
            groups: vec![FailureGroup {
                root_cause: "Many failures".to_string(),
                ignore_reason: "Investigate".to_string(),
                priority: 4,
                tests,
            }],
            timestamp: "2025-01-01T00:00:00Z".to_string(),
        };

        let tickets = generate_issue_templates(&report, None);

        // Should limit to 20 tests in body
        assert!(tickets[0].body.contains("test_0"));
        assert!(tickets[0].body.contains("test_19"));
        assert!(!tickets[0].body.contains("test_20")); // This one should be cut off
        assert!(tickets[0].body.contains("... and 10 more tests"));
    }

    #[test]
    fn test_tickets_report_serialization() {
        let report = TicketsReport {
            tickets: vec![TestIssueTemplate {
                title: "Test Issue".to_string(),
                body: "Body".to_string(),
                labels: vec!["bug".to_string()],
                category: "Category".to_string(),
                test_count: 5,
            }],
            total_tests: 5,
            timestamp: "2025-01-01T00:00:00Z".to_string(),
        };

        let json = serde_json::to_string(&report).unwrap();
        let parsed: TicketsReport = serde_json::from_str(&json).unwrap();

        assert_eq!(parsed.tickets.len(), 1);
        assert_eq!(parsed.tickets[0].title, "Test Issue");
        assert_eq!(parsed.total_tests, 5);
    }

    // Path resolution tests

    #[test]
    fn test_resolve_test_path_exact_match() {
        use std::collections::HashMap;

        let mut map = HashMap::new();
        map.insert("test_foo".to_string(), PathBuf::from("src/tests.rs"));

        let result = resolve_test_path("test_foo", &map);
        assert_eq!(result, Some(PathBuf::from("src/tests.rs")));
    }

    #[test]
    fn test_resolve_test_path_module_prefix() {
        use std::collections::HashMap;

        let mut map = HashMap::new();
        map.insert("test_bar".to_string(), PathBuf::from("src/module/tests.rs"));

        // Should resolve module::submodule::test_bar to test_bar
        let result = resolve_test_path("module::submodule::test_bar", &map);
        assert_eq!(result, Some(PathBuf::from("src/module/tests.rs")));
    }

    #[test]
    fn test_resolve_test_path_not_found() {
        use std::collections::HashMap;

        let map: HashMap<String, PathBuf> = HashMap::new();

        let result = resolve_test_path("nonexistent_test", &map);
        assert!(result.is_none());
    }
}

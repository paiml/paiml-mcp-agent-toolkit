
    // resolve_absolute_path Tests

    #[test]
    fn test_resolve_absolute_path_already_absolute() {
        let project_path = PathBuf::from("/project");
        let file_path = PathBuf::from("/absolute/path/file.rs");

        let resolved = resolve_absolute_path(&project_path, &file_path);
        assert_eq!(resolved, file_path);
    }

    #[test]
    fn test_resolve_absolute_path_relative() {
        let project_path = PathBuf::from("/project");
        let file_path = PathBuf::from("src/main.rs");

        let resolved = resolve_absolute_path(&project_path, &file_path);
        assert_eq!(resolved, PathBuf::from("/project/src/main.rs"));
    }

    // is_target_file Tests

    #[test]
    fn test_is_target_file_exact_match() {
        let abs_path = PathBuf::from("/project/src/main.rs");
        let file_path = PathBuf::from("src/main.rs");

        assert!(is_target_file(
            "/project/src/main.rs",
            &abs_path,
            &file_path
        ));
    }

    #[test]
    fn test_is_target_file_relative_match() {
        let abs_path = PathBuf::from("/project/src/main.rs");
        let file_path = PathBuf::from("src/main.rs");

        assert!(is_target_file("src/main.rs", &abs_path, &file_path));
    }

    /// #698: this test used to be called `test_is_target_file_ends_with` and
    /// was cited as the reason the loose `ends_with` arm had to stay. It never
    /// exercised that arm — the diagnostic string equals `abs_path`, so it
    /// matched on identity. Renamed to what it actually pins, and extended
    /// with the case `ends_with` got wrong: a bare `--file main.rs` must NOT
    /// claim `src/main.rs` (observed: 22 violations reported against a clean
    /// 3-line `main.rs`).
    #[test]
    fn test_is_target_file_bare_name_matches_only_its_own_file() {
        let abs_path = PathBuf::from("/project/src/main.rs");
        let file_path = PathBuf::from("main.rs");

        assert!(is_target_file(
            "/project/src/main.rs",
            &abs_path,
            &file_path
        ));

        // A different `main.rs`, resolved absolutely, is a different file.
        assert!(!is_target_file(
            "/project/src/bin/tool/main.rs",
            &abs_path,
            &file_path
        ));
    }

    #[test]
    fn test_is_target_file_no_match() {
        let abs_path = PathBuf::from("/project/src/main.rs");
        let file_path = PathBuf::from("src/main.rs");

        assert!(!is_target_file("src/lib.rs", &abs_path, &file_path));
    }

    // is_machine_applicable Tests

    #[test]
    fn test_is_machine_applicable_true() {
        let span = DiagnosticSpan {
            file_name: "test.rs".to_string(),
            line_start: 1,
            line_end: 1,
            column_start: 1,
            column_end: 10,
            is_primary: true,
            suggested_replacement: Some("fix".to_string()),
            suggestion_applicability: Some("machine-applicable".to_string()),
        };

        assert!(is_machine_applicable(&span));
    }

    #[test]
    fn test_is_machine_applicable_maybe_incorrect() {
        let span = DiagnosticSpan {
            file_name: "test.rs".to_string(),
            line_start: 1,
            line_end: 1,
            column_start: 1,
            column_end: 10,
            is_primary: true,
            suggested_replacement: Some("fix".to_string()),
            suggestion_applicability: Some("maybe-incorrect".to_string()),
        };

        assert!(is_machine_applicable(&span));
    }

    #[test]
    fn test_is_machine_applicable_false() {
        let span = DiagnosticSpan {
            file_name: "test.rs".to_string(),
            line_start: 1,
            line_end: 1,
            column_start: 1,
            column_end: 10,
            is_primary: true,
            suggested_replacement: None,
            suggestion_applicability: None,
        };

        assert!(!is_machine_applicable(&span));
    }

    #[test]
    fn test_is_machine_applicable_other_applicability() {
        let span = DiagnosticSpan {
            file_name: "test.rs".to_string(),
            line_start: 1,
            line_end: 1,
            column_start: 1,
            column_end: 10,
            is_primary: true,
            suggested_replacement: Some("fix".to_string()),
            suggestion_applicability: Some("unspecified".to_string()),
        };

        assert!(!is_machine_applicable(&span));
    }

    // extract_lint_name Tests

    #[test]
    fn test_extract_lint_name_with_code() {
        let diagnostic = DiagnosticMessage {
            level: "warning".to_string(),
            message: "test message".to_string(),
            code: Some(DiagnosticCode {
                code: "unused_variable".to_string(),
            }),
            spans: vec![],
        };

        let lint_name = extract_lint_name(&diagnostic);
        assert_eq!(lint_name, "unused_variable");
    }

    #[test]
    fn test_extract_lint_name_without_code() {
        let diagnostic = DiagnosticMessage {
            level: "warning".to_string(),
            message: "test message".to_string(),
            code: None,
            spans: vec![],
        };

        let lint_name = extract_lint_name(&diagnostic);
        assert_eq!(lint_name, "");
    }

    // find_primary_span Tests

    #[test]
    fn test_find_primary_span_with_primary() {
        let diagnostic = DiagnosticMessage {
            level: "warning".to_string(),
            message: "test".to_string(),
            code: None,
            spans: vec![
                DiagnosticSpan {
                    file_name: "secondary.rs".to_string(),
                    line_start: 1,
                    line_end: 1,
                    column_start: 1,
                    column_end: 10,
                    is_primary: false,
                    suggested_replacement: None,
                    suggestion_applicability: None,
                },
                DiagnosticSpan {
                    file_name: "primary.rs".to_string(),
                    line_start: 5,
                    line_end: 5,
                    column_start: 1,
                    column_end: 10,
                    is_primary: true,
                    suggested_replacement: None,
                    suggestion_applicability: None,
                },
            ],
        };

        let span = find_primary_span(&diagnostic);
        assert!(span.is_some());
        assert_eq!(span.unwrap().file_name, "primary.rs");
    }

    #[test]
    fn test_find_primary_span_single_span() {
        let diagnostic = DiagnosticMessage {
            level: "warning".to_string(),
            message: "test".to_string(),
            code: None,
            spans: vec![DiagnosticSpan {
                file_name: "only.rs".to_string(),
                line_start: 1,
                line_end: 1,
                column_start: 1,
                column_end: 10,
                is_primary: false, // Even if not primary, it's returned as the only span
                suggested_replacement: None,
                suggestion_applicability: None,
            }],
        };

        let span = find_primary_span(&diagnostic);
        assert!(span.is_some());
        assert_eq!(span.unwrap().file_name, "only.rs");
    }

    #[test]
    fn test_find_primary_span_empty() {
        let diagnostic = DiagnosticMessage {
            level: "warning".to_string(),
            message: "test".to_string(),
            code: None,
            spans: vec![],
        };

        let span = find_primary_span(&diagnostic);
        assert!(span.is_none());
    }

    // format_output Tests

    #[test]
    fn test_format_output_summary() {
        let result = create_full_test_result();
        let output = format_output(
            &result,
            LintHotspotOutputFormat::Summary,
            false,
            Duration::from_secs(1),
            10,
        )
        .unwrap();

        assert!(output.contains("# Lint Hotspot Analysis"));
        assert!(output.contains("**Total Project Violations**"));
    }

    #[test]
    fn test_format_output_detailed() {
        let result = create_full_test_result();
        let output = format_output(
            &result,
            LintHotspotOutputFormat::Detailed,
            false,
            Duration::from_secs(1),
            10,
        )
        .unwrap();

        assert!(output.contains("# Lint Hotspot Analysis"));
        assert!(output.contains("## Detailed Violations"));
        assert!(output.contains("## Top Files by Violations"));
    }

    #[test]
    fn test_format_output_json() {
        let result = create_full_test_result();
        let output = format_output(
            &result,
            LintHotspotOutputFormat::Json,
            false,
            Duration::from_secs(1),
            10,
        )
        .unwrap();

        // Should be valid JSON
        let parsed: serde_json::Value = serde_json::from_str(&output).unwrap();
        assert!(parsed.get("hotspot").is_some());
        assert!(parsed.get("quality_gate").is_some());
    }

    #[test]
    fn test_format_output_enforcement_json() {
        let result = create_full_test_result();
        let output = format_output(
            &result,
            LintHotspotOutputFormat::EnforcementJson,
            false,
            Duration::from_secs(1),
            10,
        )
        .unwrap();

        let parsed: serde_json::Value = serde_json::from_str(&output).unwrap();
        assert!(parsed.get("hotspot").is_some());
        assert!(parsed.get("all_violations").is_some());
        assert!(parsed.get("summary_by_file").is_some());
    }

    #[test]
    fn test_format_output_sarif() {
        let result = create_full_test_result();
        let output = format_output(
            &result,
            LintHotspotOutputFormat::Sarif,
            false,
            Duration::from_secs(1),
            10,
        )
        .unwrap();

        let parsed: serde_json::Value = serde_json::from_str(&output).unwrap();
        assert_eq!(parsed.get("version").unwrap(), "2.1.0");
        assert!(parsed.get("$schema").is_some());
        assert!(parsed.get("runs").is_some());
    }

    // format_summary Tests

    #[test]
    fn test_format_summary_with_perf() {
        let result = create_full_test_result();
        let output = format_summary(&result, true, Duration::from_secs(5), 10).unwrap();

        assert!(output.contains("Analysis completed in"));
    }

    #[test]
    fn test_format_summary_with_enforcement() {
        let mut result = create_full_test_result();
        result.enforcement = Some(EnforcementMetadata {
            enforcement_score: 8.5,
            requires_enforcement: true,
            estimated_fix_time: 1800,
            automation_confidence: 0.85,
            enforcement_priority: 3,
        });

        let output = format_summary(&result, false, Duration::from_secs(1), 10).unwrap();

        assert!(output.contains("## Enforcement Metadata"));
        assert!(output.contains("Score: 8.5/10"));
        assert!(output.contains("Priority: 3"));
    }

    #[test]
    fn test_format_summary_with_failed_quality_gate() {
        let mut result = create_full_test_result();
        result.quality_gate.passed = false;
        result.quality_gate.violations = vec![QualityViolation {
            rule: "max_defect_density".to_string(),
            threshold: 0.05,
            actual: 0.10,
            severity: "blocking".to_string(),
        }];

        let output = format_summary(&result, false, Duration::from_secs(1), 10).unwrap();

        assert!(output.contains("Quality Gate Failed"));
        assert!(output.contains("max_defect_density exceeded"));
    }

    #[test]
    fn test_format_summary_top_files_limit() {
        let mut result = create_full_test_result();
        // Add more files to summary
        for i in 0..15 {
            result.summary_by_file.insert(
                PathBuf::from(format!("src/file_{i}.rs")),
                FileSummary {
                    total_violations: i,
                    errors: i / 2,
                    warnings: i - i / 2,
                    sloc: 100,
                    defect_density: i as f64 / 100.0,
                },
            );
        }

        let output = format_summary(&result, false, Duration::from_secs(1), 5).unwrap();

        // Should show limited files (may include header row and extras)
        let file_count = output.matches("violations/SLOC").count();
        assert!(file_count > 0, "Should have file entries");
    }

    // format_detailed Tests

    #[test]
    fn test_format_detailed_with_violations() {
        let mut result = create_full_test_result();
        result.hotspot.detailed_violations = vec![
            create_test_violation("src/main.rs", 10, "unused_variable", "warning"),
            create_test_violation_with_suggestion(
                "src/main.rs",
                20,
                "needless_return",
                "warning",
                "Remove the return statement",
            ),
        ];

        let output = format_detailed(&result, false, Duration::from_secs(1), 10).unwrap();

        assert!(output.contains("## Detailed Violations in Hotspot File"));
        assert!(output.contains("unused_variable"));
        assert!(output.contains("Suggestion: Remove the return statement"));
    }

    #[test]
    fn test_format_detailed_with_refactor_chain() {
        let mut result = create_full_test_result();
        result.refactor_chain = Some(RefactorChain {
            id: "test-chain".to_string(),
            estimated_reduction: 15,
            automation_confidence: 0.88,
            steps: vec![
                RefactorStep {
                    id: "step-1".to_string(),
                    lint: "unused".to_string(),
                    confidence: 0.95,
                    impact: 10,
                    description: "Remove unused code".to_string(),
                },
            ],
        });

        let output = format_detailed(&result, false, Duration::from_secs(1), 10).unwrap();

        assert!(output.contains("## Refactor Chain"));
        assert!(output.contains("ID: test-chain"));
        assert!(output.contains("Estimated Reduction: 15 violations"));
        assert!(output.contains("### Steps"));
        assert!(output.contains("Remove unused code"));
    }

    // format_json Tests

    #[test]
    fn test_format_json_simple() {
        let result = create_full_test_result();
        let output = format_json(&result, false).unwrap();

        let parsed: serde_json::Value = serde_json::from_str(&output).unwrap();

        // Simple JSON should only have hotspot and quality_gate
        assert!(parsed.get("hotspot").is_some());
        assert!(parsed.get("quality_gate").is_some());
        assert!(parsed.get("all_violations").is_none());
    }

    #[test]
    fn test_format_json_enforcement() {
        let result = create_full_test_result();
        let output = format_json(&result, true).unwrap();

        let parsed: serde_json::Value = serde_json::from_str(&output).unwrap();

        // Enforcement JSON should have all fields
        assert!(parsed.get("hotspot").is_some());
        assert!(parsed.get("quality_gate").is_some());
        assert!(parsed.get("all_violations").is_some());
        assert!(parsed.get("summary_by_file").is_some());
        assert!(parsed.get("total_project_violations").is_some());
    }

    // format_sarif Tests

    #[test]
    fn test_format_sarif_structure() {
        let mut result = create_full_test_result();
        result.quality_gate.violations = vec![QualityViolation {
            rule: "max_defect_density".to_string(),
            threshold: 0.05,
            actual: 0.10,
            severity: "blocking".to_string(),
        }];

        let output = format_sarif(&result).unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&output).unwrap();

        assert_eq!(parsed["version"], "2.1.0");
        assert!(parsed["$schema"]
            .as_str()
            .unwrap()
            .contains("sarif-schema"));

        let runs = parsed["runs"].as_array().unwrap();
        assert_eq!(runs.len(), 1);
        let tool = &runs[0]["tool"]["driver"];
        assert_eq!(tool["name"], "pmat-lint-hotspot");
    }

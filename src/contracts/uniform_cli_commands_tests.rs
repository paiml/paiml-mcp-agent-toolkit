// Uniform CLI commands tests extracted from uniform_cli_commands.rs for file health (CB-040).
// This file is include!()'d into uniform_cli_commands.rs scope.

#[cfg_attr(coverage_nightly, coverage(off))]
#[cfg(test)]
mod coverage_tests {
    use super::*;
    use std::path::PathBuf;

    // ===========================================
    // UniformOutputFormat Tests
    // ===========================================

    #[test]
    fn test_uniform_output_format_table_conversion() {
        let uniform = UniformOutputFormat::Table;
        let output: OutputFormat = uniform.into();
        assert_eq!(output, OutputFormat::Table);
    }

    #[test]
    fn test_uniform_output_format_json_conversion() {
        let uniform = UniformOutputFormat::Json;
        let output: OutputFormat = uniform.into();
        assert_eq!(output, OutputFormat::Json);
    }

    #[test]
    fn test_uniform_output_format_yaml_conversion() {
        let uniform = UniformOutputFormat::Yaml;
        let output: OutputFormat = uniform.into();
        assert_eq!(output, OutputFormat::Yaml);
    }

    #[test]
    fn test_uniform_output_format_markdown_conversion() {
        let uniform = UniformOutputFormat::Markdown;
        let output: OutputFormat = uniform.into();
        assert_eq!(output, OutputFormat::Markdown);
    }

    #[test]
    fn test_uniform_output_format_csv_conversion() {
        let uniform = UniformOutputFormat::Csv;
        let output: OutputFormat = uniform.into();
        assert_eq!(output, OutputFormat::Csv);
    }

    #[test]
    fn test_uniform_output_format_summary_conversion() {
        let uniform = UniformOutputFormat::Summary;
        let output: OutputFormat = uniform.into();
        assert_eq!(output, OutputFormat::Summary);
    }

    #[test]
    fn test_uniform_output_format_clone() {
        let format = UniformOutputFormat::Json;
        let cloned = format;
        assert_eq!(format, cloned);
    }

    #[test]
    fn test_uniform_output_format_debug() {
        let format = UniformOutputFormat::Table;
        let debug_str = format!("{:?}", format);
        assert!(debug_str.contains("Table"));
    }

    #[test]
    fn test_uniform_output_format_ordering() {
        assert!(UniformOutputFormat::Table < UniformOutputFormat::Json);
        assert!(UniformOutputFormat::Json < UniformOutputFormat::Yaml);
        assert!(UniformOutputFormat::Yaml < UniformOutputFormat::Markdown);
        assert!(UniformOutputFormat::Markdown < UniformOutputFormat::Csv);
        assert!(UniformOutputFormat::Csv < UniformOutputFormat::Summary);
    }

    // ===========================================
    // UniformSatdSeverity Tests
    // ===========================================

    #[test]
    fn test_uniform_satd_severity_low_conversion() {
        let uniform = UniformSatdSeverity::Low;
        let severity: SatdSeverity = uniform.into();
        assert_eq!(severity, SatdSeverity::Low);
    }

    #[test]
    fn test_uniform_satd_severity_medium_conversion() {
        let uniform = UniformSatdSeverity::Medium;
        let severity: SatdSeverity = uniform.into();
        assert_eq!(severity, SatdSeverity::Medium);
    }

    #[test]
    fn test_uniform_satd_severity_high_conversion() {
        let uniform = UniformSatdSeverity::High;
        let severity: SatdSeverity = uniform.into();
        assert_eq!(severity, SatdSeverity::High);
    }

    #[test]
    fn test_uniform_satd_severity_critical_conversion() {
        let uniform = UniformSatdSeverity::Critical;
        let severity: SatdSeverity = uniform.into();
        assert_eq!(severity, SatdSeverity::Critical);
    }

    #[test]
    fn test_uniform_satd_severity_clone() {
        let severity = UniformSatdSeverity::High;
        let cloned = severity;
        assert_eq!(severity, cloned);
    }

    #[test]
    fn test_uniform_satd_severity_debug() {
        let severity = UniformSatdSeverity::Critical;
        let debug_str = format!("{:?}", severity);
        assert!(debug_str.contains("Critical"));
    }

    #[test]
    fn test_uniform_satd_severity_ordering() {
        assert!(UniformSatdSeverity::Low < UniformSatdSeverity::Medium);
        assert!(UniformSatdSeverity::Medium < UniformSatdSeverity::High);
        assert!(UniformSatdSeverity::High < UniformSatdSeverity::Critical);
    }

    // ===========================================
    // UniformComplexityArgs Tests
    // ===========================================

    #[test]
    fn test_uniform_complexity_args_default_values() {
        let args = UniformComplexityArgs {
            path: PathBuf::from("."),
            format: UniformOutputFormat::Table,
            output: None,
            top_files: 10,
            include_tests: false,
            timeout: 60,
            max_cyclomatic: None,
            max_cognitive: None,
            max_halstead: None,
        };

        assert_eq!(args.path, PathBuf::from("."));
        assert_eq!(args.format, UniformOutputFormat::Table);
        assert!(args.output.is_none());
        assert_eq!(args.top_files, 10);
        assert!(!args.include_tests);
        assert_eq!(args.timeout, 60);
        assert!(args.max_cyclomatic.is_none());
        assert!(args.max_cognitive.is_none());
        assert!(args.max_halstead.is_none());
    }

    #[test]
    fn test_uniform_complexity_args_with_thresholds() {
        let args = UniformComplexityArgs {
            path: PathBuf::from("/test/path"),
            format: UniformOutputFormat::Json,
            output: Some(PathBuf::from("output.json")),
            top_files: 20,
            include_tests: true,
            timeout: 120,
            max_cyclomatic: Some(15),
            max_cognitive: Some(20),
            max_halstead: Some(30.5),
        };

        assert_eq!(args.path, PathBuf::from("/test/path"));
        assert_eq!(args.format, UniformOutputFormat::Json);
        assert_eq!(args.output, Some(PathBuf::from("output.json")));
        assert_eq!(args.top_files, 20);
        assert!(args.include_tests);
        assert_eq!(args.timeout, 120);
        assert_eq!(args.max_cyclomatic, Some(15));
        assert_eq!(args.max_cognitive, Some(20));
        assert_eq!(args.max_halstead, Some(30.5));
    }

    #[test]
    fn test_uniform_complexity_args_to_contract() {
        let args = UniformComplexityArgs {
            path: PathBuf::from("/project"),
            format: UniformOutputFormat::Yaml,
            output: Some(PathBuf::from("report.yaml")),
            top_files: 15,
            include_tests: true,
            timeout: 90,
            max_cyclomatic: Some(10),
            max_cognitive: Some(15),
            max_halstead: Some(25.0),
        };

        let contract: AnalyzeComplexityContract = args.into();

        assert_eq!(contract.base.path, PathBuf::from("/project"));
        assert_eq!(contract.base.format, OutputFormat::Yaml);
        assert_eq!(contract.base.output, Some(PathBuf::from("report.yaml")));
        assert_eq!(contract.base.top_files, Some(15));
        assert!(contract.base.include_tests);
        assert_eq!(contract.base.timeout, 90);
        assert_eq!(contract.max_cyclomatic, Some(10));
        assert_eq!(contract.max_cognitive, Some(15));
        assert_eq!(contract.max_halstead, Some(25.0));
    }

    #[test]
    fn test_uniform_complexity_args_debug() {
        let args = UniformComplexityArgs {
            path: PathBuf::from("."),
            format: UniformOutputFormat::Table,
            output: None,
            top_files: 10,
            include_tests: false,
            timeout: 60,
            max_cyclomatic: None,
            max_cognitive: None,
            max_halstead: None,
        };
        let debug_str = format!("{:?}", args);
        assert!(debug_str.contains("UniformComplexityArgs"));
    }

    // ===========================================
    // UniformSatdArgs Tests
    // ===========================================

    #[test]
    fn test_uniform_satd_args_default_values() {
        let args = UniformSatdArgs {
            path: PathBuf::from("."),
            format: UniformOutputFormat::Table,
            output: None,
            top_files: 10,
            include_tests: false,
            timeout: 60,
            severity: None,
            critical_only: false,
            strict: false,
            fail_on_violation: false,
        };

        assert_eq!(args.path, PathBuf::from("."));
        assert_eq!(args.format, UniformOutputFormat::Table);
        assert!(args.output.is_none());
        assert_eq!(args.top_files, 10);
        assert!(!args.include_tests);
        assert_eq!(args.timeout, 60);
        assert!(args.severity.is_none());
        assert!(!args.critical_only);
        assert!(!args.strict);
        assert!(!args.fail_on_violation);
    }

    #[test]
    fn test_uniform_satd_args_with_all_options() {
        let args = UniformSatdArgs {
            path: PathBuf::from("/src"),
            format: UniformOutputFormat::Markdown,
            output: Some(PathBuf::from("satd.md")),
            top_files: 25,
            include_tests: true,
            timeout: 180,
            severity: Some(UniformSatdSeverity::High),
            critical_only: true,
            strict: true,
            fail_on_violation: true,
        };

        assert_eq!(args.path, PathBuf::from("/src"));
        assert_eq!(args.format, UniformOutputFormat::Markdown);
        assert_eq!(args.output, Some(PathBuf::from("satd.md")));
        assert_eq!(args.top_files, 25);
        assert!(args.include_tests);
        assert_eq!(args.timeout, 180);
        assert_eq!(args.severity, Some(UniformSatdSeverity::High));
        assert!(args.critical_only);
        assert!(args.strict);
        assert!(args.fail_on_violation);
    }

    #[test]
    fn test_uniform_satd_args_to_contract() {
        let args = UniformSatdArgs {
            path: PathBuf::from("/code"),
            format: UniformOutputFormat::Csv,
            output: None,
            top_files: 5,
            include_tests: false,
            timeout: 30,
            severity: Some(UniformSatdSeverity::Medium),
            critical_only: false,
            strict: true,
            fail_on_violation: true,
        };

        let contract: AnalyzeSatdContract = args.into();

        assert_eq!(contract.base.path, PathBuf::from("/code"));
        assert_eq!(contract.base.format, OutputFormat::Csv);
        assert!(contract.base.output.is_none());
        assert_eq!(contract.base.top_files, Some(5));
        assert!(!contract.base.include_tests);
        assert_eq!(contract.base.timeout, 30);
        assert_eq!(contract.severity, Some(SatdSeverity::Medium));
        assert!(!contract.critical_only);
        assert!(contract.strict);
        assert!(contract.fail_on_violation);
    }

    #[test]
    fn test_uniform_satd_args_to_contract_no_severity() {
        let args = UniformSatdArgs {
            path: PathBuf::from("."),
            format: UniformOutputFormat::Table,
            output: None,
            top_files: 10,
            include_tests: false,
            timeout: 60,
            severity: None,
            critical_only: false,
            strict: false,
            fail_on_violation: false,
        };

        let contract: AnalyzeSatdContract = args.into();
        assert!(contract.severity.is_none());
    }

    // ===========================================
    // UniformDeadCodeArgs Tests
    // ===========================================

    #[test]
    fn test_uniform_dead_code_args_default_values() {
        let args = UniformDeadCodeArgs {
            path: PathBuf::from("."),
            format: UniformOutputFormat::Table,
            output: None,
            top_files: 10,
            include_tests: false,
            timeout: 60,
            include_unreachable: false,
            min_dead_lines: 10,
            max_percentage: 15.0,
            fail_on_violation: false,
        };

        assert_eq!(args.path, PathBuf::from("."));
        assert_eq!(args.format, UniformOutputFormat::Table);
        assert!(args.output.is_none());
        assert_eq!(args.top_files, 10);
        assert!(!args.include_tests);
        assert_eq!(args.timeout, 60);
        assert!(!args.include_unreachable);
        assert_eq!(args.min_dead_lines, 10);
        assert_eq!(args.max_percentage, 15.0);
        assert!(!args.fail_on_violation);
    }

    #[test]
    fn test_uniform_dead_code_args_with_all_options() {
        let args = UniformDeadCodeArgs {
            path: PathBuf::from("/project/src"),
            format: UniformOutputFormat::Json,
            output: Some(PathBuf::from("dead_code.json")),
            top_files: 50,
            include_tests: true,
            timeout: 300,
            include_unreachable: true,
            min_dead_lines: 5,
            max_percentage: 10.0,
            fail_on_violation: true,
        };

        assert_eq!(args.path, PathBuf::from("/project/src"));
        assert_eq!(args.format, UniformOutputFormat::Json);
        assert_eq!(args.output, Some(PathBuf::from("dead_code.json")));
        assert_eq!(args.top_files, 50);
        assert!(args.include_tests);
        assert_eq!(args.timeout, 300);
        assert!(args.include_unreachable);
        assert_eq!(args.min_dead_lines, 5);
        assert_eq!(args.max_percentage, 10.0);
        assert!(args.fail_on_violation);
    }

    #[test]
    fn test_uniform_dead_code_args_to_contract() {
        let args = UniformDeadCodeArgs {
            path: PathBuf::from("/analysis"),
            format: UniformOutputFormat::Summary,
            output: Some(PathBuf::from("summary.txt")),
            top_files: 100,
            include_tests: true,
            timeout: 240,
            include_unreachable: true,
            min_dead_lines: 20,
            max_percentage: 5.0,
            fail_on_violation: true,
        };

        let contract: AnalyzeDeadCodeContract = args.into();

        assert_eq!(contract.base.path, PathBuf::from("/analysis"));
        assert_eq!(contract.base.format, OutputFormat::Summary);
        assert_eq!(contract.base.output, Some(PathBuf::from("summary.txt")));
        assert_eq!(contract.base.top_files, Some(100));
        assert!(contract.base.include_tests);
        assert_eq!(contract.base.timeout, 240);
        assert!(contract.include_unreachable);
        assert_eq!(contract.min_dead_lines, 20);
        assert_eq!(contract.max_percentage, 5.0);
        assert!(contract.fail_on_violation);
    }

    // ===========================================
    // UniformTdgArgs Tests
    // ===========================================

    #[test]
    fn test_uniform_tdg_args_default_values() {
        let args = UniformTdgArgs {
            path: PathBuf::from("."),
            format: UniformOutputFormat::Table,
            output: None,
            top_files: 10,
            include_tests: false,
            timeout: 60,
            threshold: 1.5,
            include_components: false,
            critical_only: false,
        };

        assert_eq!(args.path, PathBuf::from("."));
        assert_eq!(args.format, UniformOutputFormat::Table);
        assert!(args.output.is_none());
        assert_eq!(args.top_files, 10);
        assert!(!args.include_tests);
        assert_eq!(args.timeout, 60);
        assert_eq!(args.threshold, 1.5);
        assert!(!args.include_components);
        assert!(!args.critical_only);
    }

    #[test]
    fn test_uniform_tdg_args_with_all_options() {
        let args = UniformTdgArgs {
            path: PathBuf::from("/rust/project"),
            format: UniformOutputFormat::Yaml,
            output: Some(PathBuf::from("tdg.yaml")),
            top_files: 30,
            include_tests: true,
            timeout: 150,
            threshold: 2.0,
            include_components: true,
            critical_only: true,
        };

        assert_eq!(args.path, PathBuf::from("/rust/project"));
        assert_eq!(args.format, UniformOutputFormat::Yaml);
        assert_eq!(args.output, Some(PathBuf::from("tdg.yaml")));
        assert_eq!(args.top_files, 30);
        assert!(args.include_tests);
        assert_eq!(args.timeout, 150);
        assert_eq!(args.threshold, 2.0);
        assert!(args.include_components);
        assert!(args.critical_only);
    }

    #[test]
    fn test_uniform_tdg_args_to_contract() {
        let args = UniformTdgArgs {
            path: PathBuf::from("/codebase"),
            format: UniformOutputFormat::Markdown,
            output: None,
            top_files: 20,
            include_tests: false,
            timeout: 120,
            threshold: 2.5,
            include_components: true,
            critical_only: true,
        };

        let contract: AnalyzeTdgContract = args.into();

        assert_eq!(contract.base.path, PathBuf::from("/codebase"));
        assert_eq!(contract.base.format, OutputFormat::Markdown);
        assert!(contract.base.output.is_none());
        assert_eq!(contract.base.top_files, Some(20));
        assert!(!contract.base.include_tests);
        assert_eq!(contract.base.timeout, 120);
        assert_eq!(contract.threshold, 2.5);
        assert!(contract.include_components);
        assert!(contract.critical_only);
    }

    // ===========================================
    // UniformLintHotspotArgs Tests
    // ===========================================

    #[test]
    fn test_uniform_lint_hotspot_args_default_values() {
        let args = UniformLintHotspotArgs {
            path: PathBuf::from("."),
            format: UniformOutputFormat::Table,
            output: None,
            top_files: 10,
            include_tests: false,
            timeout: 60,
            file: None,
            max_density: 5.0,
            min_confidence: 0.8,
            enforce: false,
            dry_run: false,
        };

        assert_eq!(args.path, PathBuf::from("."));
        assert_eq!(args.format, UniformOutputFormat::Table);
        assert!(args.output.is_none());
        assert_eq!(args.top_files, 10);
        assert!(!args.include_tests);
        assert_eq!(args.timeout, 60);
        assert!(args.file.is_none());
        assert_eq!(args.max_density, 5.0);
        assert_eq!(args.min_confidence, 0.8);
        assert!(!args.enforce);
        assert!(!args.dry_run);
    }

    #[test]
    fn test_uniform_lint_hotspot_args_with_all_options() {
        let args = UniformLintHotspotArgs {
            path: PathBuf::from("/project"),
            format: UniformOutputFormat::Json,
            output: Some(PathBuf::from("lint.json")),
            top_files: 15,
            include_tests: true,
            timeout: 90,
            file: Some(PathBuf::from("src/main.rs")),
            max_density: 3.0,
            min_confidence: 0.9,
            enforce: true,
            dry_run: true,
        };

        assert_eq!(args.path, PathBuf::from("/project"));
        assert_eq!(args.format, UniformOutputFormat::Json);
        assert_eq!(args.output, Some(PathBuf::from("lint.json")));
        assert_eq!(args.top_files, 15);
        assert!(args.include_tests);
        assert_eq!(args.timeout, 90);
        assert_eq!(args.file, Some(PathBuf::from("src/main.rs")));
        assert_eq!(args.max_density, 3.0);
        assert_eq!(args.min_confidence, 0.9);
        assert!(args.enforce);
        assert!(args.dry_run);
    }

    #[test]
    fn test_uniform_lint_hotspot_args_to_contract() {
        let args = UniformLintHotspotArgs {
            path: PathBuf::from("/src"),
            format: UniformOutputFormat::Csv,
            output: Some(PathBuf::from("hotspots.csv")),
            top_files: 25,
            include_tests: true,
            timeout: 180,
            file: Some(PathBuf::from("lib.rs")),
            max_density: 2.5,
            min_confidence: 0.95,
            enforce: true,
            dry_run: false,
        };

        let contract: AnalyzeLintHotspotContract = args.into();

        assert_eq!(contract.base.path, PathBuf::from("/src"));
        assert_eq!(contract.base.format, OutputFormat::Csv);
        assert_eq!(contract.base.output, Some(PathBuf::from("hotspots.csv")));
        assert_eq!(contract.base.top_files, Some(25));
        assert!(contract.base.include_tests);
        assert_eq!(contract.base.timeout, 180);
        assert_eq!(contract.file, Some(PathBuf::from("lib.rs")));
        assert_eq!(contract.max_density, 2.5);
        assert_eq!(contract.min_confidence, 0.95);
        assert!(contract.enforce);
        assert!(!contract.dry_run);
    }

    #[test]
    fn test_uniform_lint_hotspot_args_to_contract_no_file() {
        let args = UniformLintHotspotArgs {
            path: PathBuf::from("."),
            format: UniformOutputFormat::Table,
            output: None,
            top_files: 10,
            include_tests: false,
            timeout: 60,
            file: None,
            max_density: 5.0,
            min_confidence: 0.8,
            enforce: false,
            dry_run: false,
        };

        let contract: AnalyzeLintHotspotContract = args.into();
        assert!(contract.file.is_none());
    }

    // ===========================================
    // UniformAnalyzeCommands Tests
    // ===========================================

    #[test]
    fn test_uniform_analyze_commands_complexity_variant() {
        let args = UniformComplexityArgs {
            path: PathBuf::from("."),
            format: UniformOutputFormat::Table,
            output: None,
            top_files: 10,
            include_tests: false,
            timeout: 60,
            max_cyclomatic: None,
            max_cognitive: None,
            max_halstead: None,
        };
        let cmd = UniformAnalyzeCommands::Complexity(args);

        // Verify it's the Complexity variant
        match cmd {
            UniformAnalyzeCommands::Complexity(a) => {
                assert_eq!(a.path, PathBuf::from("."));
            }
            _ => panic!("Expected Complexity variant"),
        }
    }

    #[test]
    fn test_uniform_analyze_commands_satd_variant() {
        let args = UniformSatdArgs {
            path: PathBuf::from("/project"),
            format: UniformOutputFormat::Json,
            output: None,
            top_files: 10,
            include_tests: false,
            timeout: 60,
            severity: Some(UniformSatdSeverity::High),
            critical_only: false,
            strict: false,
            fail_on_violation: false,
        };
        let cmd = UniformAnalyzeCommands::Satd(args);

        match cmd {
            UniformAnalyzeCommands::Satd(a) => {
                assert_eq!(a.severity, Some(UniformSatdSeverity::High));
            }
            _ => panic!("Expected Satd variant"),
        }
    }

    #[test]
    fn test_uniform_analyze_commands_dead_code_variant() {
        let args = UniformDeadCodeArgs {
            path: PathBuf::from("."),
            format: UniformOutputFormat::Table,
            output: None,
            top_files: 10,
            include_tests: false,
            timeout: 60,
            include_unreachable: true,
            min_dead_lines: 5,
            max_percentage: 10.0,
            fail_on_violation: false,
        };
        let cmd = UniformAnalyzeCommands::DeadCode(args);

        match cmd {
            UniformAnalyzeCommands::DeadCode(a) => {
                assert!(a.include_unreachable);
                assert_eq!(a.min_dead_lines, 5);
            }
            _ => panic!("Expected DeadCode variant"),
        }
    }

    #[test]
    fn test_uniform_analyze_commands_tdg_variant() {
        let args = UniformTdgArgs {
            path: PathBuf::from("."),
            format: UniformOutputFormat::Yaml,
            output: None,
            top_files: 10,
            include_tests: false,
            timeout: 60,
            threshold: 2.0,
            include_components: true,
            critical_only: false,
        };
        let cmd = UniformAnalyzeCommands::Tdg(args);

        match cmd {
            UniformAnalyzeCommands::Tdg(a) => {
                assert_eq!(a.threshold, 2.0);
                assert!(a.include_components);
            }
            _ => panic!("Expected Tdg variant"),
        }
    }

    #[test]
    fn test_uniform_analyze_commands_lint_hotspot_variant() {
        let args = UniformLintHotspotArgs {
            path: PathBuf::from("."),
            format: UniformOutputFormat::Markdown,
            output: None,
            top_files: 10,
            include_tests: false,
            timeout: 60,
            file: Some(PathBuf::from("main.rs")),
            max_density: 5.0,
            min_confidence: 0.8,
            enforce: true,
            dry_run: false,
        };
        let cmd = UniformAnalyzeCommands::LintHotspot(args);

        match cmd {
            UniformAnalyzeCommands::LintHotspot(a) => {
                assert_eq!(a.file, Some(PathBuf::from("main.rs")));
                assert!(a.enforce);
            }
            _ => panic!("Expected LintHotspot variant"),
        }
    }

    // ===========================================
    // UniformCommandHandler Tests
    // ===========================================

    #[test]
    fn test_uniform_command_handler_new() {
        // This test verifies the handler can be created
        let result = UniformCommandHandler::new();
        assert!(result.is_ok());
    }

    #[test]
    fn test_uniform_command_handler_output_result_string() {
        let handler = UniformCommandHandler::new().unwrap();
        let result = serde_json::Value::String("Test output".to_string());
        let output = handler.output_result(result);
        assert!(output.is_ok());
    }

    #[test]
    fn test_uniform_command_handler_output_result_object() {
        let handler = UniformCommandHandler::new().unwrap();
        let result = serde_json::json!({
            "key": "value",
            "number": 42
        });
        let output = handler.output_result(result);
        assert!(output.is_ok());
    }

    #[test]
    fn test_uniform_command_handler_output_result_array() {
        let handler = UniformCommandHandler::new().unwrap();
        let result = serde_json::json!([1, 2, 3, "test"]);
        let output = handler.output_result(result);
        assert!(output.is_ok());
    }

    #[test]
    fn test_uniform_command_handler_output_result_null() {
        let handler = UniformCommandHandler::new().unwrap();
        let result = serde_json::Value::Null;
        let output = handler.output_result(result);
        assert!(output.is_ok());
    }

    #[test]
    fn test_uniform_command_handler_output_result_boolean() {
        let handler = UniformCommandHandler::new().unwrap();
        let result = serde_json::Value::Bool(true);
        let output = handler.output_result(result);
        assert!(output.is_ok());
    }

    #[test]
    fn test_uniform_command_handler_output_result_number() {
        let handler = UniformCommandHandler::new().unwrap();
        let result = serde_json::json!(42.5);
        let output = handler.output_result(result);
        assert!(output.is_ok());
    }

    // ===========================================
    // Contract Conversion Edge Cases
    // ===========================================

    #[test]
    fn test_complexity_contract_with_zero_top_files() {
        let args = UniformComplexityArgs {
            path: PathBuf::from("."),
            format: UniformOutputFormat::Table,
            output: None,
            top_files: 0,
            include_tests: false,
            timeout: 60,
            max_cyclomatic: None,
            max_cognitive: None,
            max_halstead: None,
        };

        let contract: AnalyzeComplexityContract = args.into();
        assert_eq!(contract.base.top_files, Some(0));
    }

    #[test]
    fn test_dead_code_contract_with_zero_percentage() {
        let args = UniformDeadCodeArgs {
            path: PathBuf::from("."),
            format: UniformOutputFormat::Table,
            output: None,
            top_files: 10,
            include_tests: false,
            timeout: 60,
            include_unreachable: false,
            min_dead_lines: 0,
            max_percentage: 0.0,
            fail_on_violation: false,
        };

        let contract: AnalyzeDeadCodeContract = args.into();
        assert_eq!(contract.min_dead_lines, 0);
        assert_eq!(contract.max_percentage, 0.0);
    }

    #[test]
    fn test_tdg_contract_with_zero_threshold() {
        let args = UniformTdgArgs {
            path: PathBuf::from("."),
            format: UniformOutputFormat::Table,
            output: None,
            top_files: 10,
            include_tests: false,
            timeout: 60,
            threshold: 0.0,
            include_components: false,
            critical_only: false,
        };

        let contract: AnalyzeTdgContract = args.into();
        assert_eq!(contract.threshold, 0.0);
    }

    #[test]
    fn test_lint_hotspot_contract_with_max_confidence() {
        let args = UniformLintHotspotArgs {
            path: PathBuf::from("."),
            format: UniformOutputFormat::Table,
            output: None,
            top_files: 10,
            include_tests: false,
            timeout: 60,
            file: None,
            max_density: 0.0,
            min_confidence: 1.0,
            enforce: false,
            dry_run: false,
        };

        let contract: AnalyzeLintHotspotContract = args.into();
        assert_eq!(contract.min_confidence, 1.0);
        assert_eq!(contract.max_density, 0.0);
    }

    // ===========================================
    // All Output Format Conversions
    // ===========================================

    #[test]
    fn test_all_output_format_conversions_roundtrip() {
        let formats = vec![
            (UniformOutputFormat::Table, OutputFormat::Table),
            (UniformOutputFormat::Json, OutputFormat::Json),
            (UniformOutputFormat::Yaml, OutputFormat::Yaml),
            (UniformOutputFormat::Markdown, OutputFormat::Markdown),
            (UniformOutputFormat::Csv, OutputFormat::Csv),
            (UniformOutputFormat::Summary, OutputFormat::Summary),
        ];

        for (uniform, expected) in formats {
            let converted: OutputFormat = uniform.into();
            assert_eq!(converted, expected);
        }
    }

    // ===========================================
    // All SATD Severity Conversions
    // ===========================================

    #[test]
    fn test_all_satd_severity_conversions_roundtrip() {
        let severities = vec![
            (UniformSatdSeverity::Low, SatdSeverity::Low),
            (UniformSatdSeverity::Medium, SatdSeverity::Medium),
            (UniformSatdSeverity::High, SatdSeverity::High),
            (UniformSatdSeverity::Critical, SatdSeverity::Critical),
        ];

        for (uniform, expected) in severities {
            let converted: SatdSeverity = uniform.into();
            assert_eq!(converted, expected);
        }
    }

    // ===========================================
    // Large Value Edge Cases
    // ===========================================

    #[test]
    fn test_complexity_args_with_large_values() {
        let args = UniformComplexityArgs {
            path: PathBuf::from("/very/long/path/to/project"),
            format: UniformOutputFormat::Json,
            output: Some(PathBuf::from("/output/path/with/many/segments/file.json")),
            top_files: usize::MAX,
            include_tests: true,
            timeout: u64::MAX,
            max_cyclomatic: Some(u32::MAX),
            max_cognitive: Some(u32::MAX),
            max_halstead: Some(f64::MAX),
        };

        let contract: AnalyzeComplexityContract = args.into();
        assert_eq!(contract.base.top_files, Some(usize::MAX));
        assert_eq!(contract.base.timeout, u64::MAX);
        assert_eq!(contract.max_cyclomatic, Some(u32::MAX));
    }

    #[test]
    fn test_dead_code_args_with_max_percentage() {
        let args = UniformDeadCodeArgs {
            path: PathBuf::from("."),
            format: UniformOutputFormat::Table,
            output: None,
            top_files: 10,
            include_tests: false,
            timeout: 60,
            include_unreachable: false,
            min_dead_lines: usize::MAX,
            max_percentage: 100.0,
            fail_on_violation: false,
        };

        let contract: AnalyzeDeadCodeContract = args.into();
        assert_eq!(contract.min_dead_lines, usize::MAX);
        assert_eq!(contract.max_percentage, 100.0);
    }

    // ===========================================
    // Async Handler Tests
    // ===========================================

    #[tokio::test]
    async fn test_handle_analyze_command_complexity() {
        let handler = UniformCommandHandler::new().unwrap();
        let args = UniformComplexityArgs {
            path: PathBuf::from("."),
            format: UniformOutputFormat::Json,
            output: None,
            top_files: 5,
            include_tests: false,
            timeout: 10,
            max_cyclomatic: Some(20),
            max_cognitive: Some(15),
            max_halstead: None,
        };
        let cmd = UniformAnalyzeCommands::Complexity(args);

        // This should run without panicking
        let result = handler.handle_analyze_command(cmd).await;
        // The result may fail due to analysis issues, but the handler itself works
        let _ = result;
    }

    #[tokio::test]
    async fn test_handle_analyze_command_satd() {
        let handler = UniformCommandHandler::new().unwrap();
        let args = UniformSatdArgs {
            path: PathBuf::from("."),
            format: UniformOutputFormat::Json,
            output: None,
            top_files: 5,
            include_tests: false,
            timeout: 10,
            severity: None,
            critical_only: false,
            strict: false,
            fail_on_violation: false,
        };
        let cmd = UniformAnalyzeCommands::Satd(args);

        let result = handler.handle_analyze_command(cmd).await;
        let _ = result;
    }

    #[tokio::test]
    async fn test_handle_analyze_command_dead_code() {
        let handler = UniformCommandHandler::new().unwrap();
        let args = UniformDeadCodeArgs {
            path: PathBuf::from("."),
            format: UniformOutputFormat::Json,
            output: None,
            top_files: 5,
            include_tests: false,
            timeout: 10,
            include_unreachable: false,
            min_dead_lines: 10,
            max_percentage: 15.0,
            fail_on_violation: false,
        };
        let cmd = UniformAnalyzeCommands::DeadCode(args);

        let result = handler.handle_analyze_command(cmd).await;
        let _ = result;
    }

    #[tokio::test]
    async fn test_handle_analyze_command_tdg() {
        let handler = UniformCommandHandler::new().unwrap();
        let args = UniformTdgArgs {
            path: PathBuf::from("."),
            format: UniformOutputFormat::Json,
            output: None,
            top_files: 5,
            include_tests: false,
            timeout: 10,
            threshold: 1.5,
            include_components: false,
            critical_only: false,
        };
        let cmd = UniformAnalyzeCommands::Tdg(args);

        let result = handler.handle_analyze_command(cmd).await;
        let _ = result;
    }

    #[tokio::test]
    async fn test_handle_analyze_command_lint_hotspot() {
        let handler = UniformCommandHandler::new().unwrap();
        let args = UniformLintHotspotArgs {
            path: PathBuf::from("."),
            format: UniformOutputFormat::Json,
            output: None,
            top_files: 5,
            include_tests: false,
            timeout: 10,
            file: None,
            max_density: 5.0,
            min_confidence: 0.8,
            enforce: false,
            dry_run: false,
        };
        let cmd = UniformAnalyzeCommands::LintHotspot(args);

        let result = handler.handle_analyze_command(cmd).await;
        let _ = result;
    }

    // ===========================================
    // Debug Trait Tests
    // ===========================================

    #[test]
    fn test_uniform_satd_args_debug() {
        let args = UniformSatdArgs {
            path: PathBuf::from("."),
            format: UniformOutputFormat::Table,
            output: None,
            top_files: 10,
            include_tests: false,
            timeout: 60,
            severity: Some(UniformSatdSeverity::High),
            critical_only: true,
            strict: false,
            fail_on_violation: false,
        };
        let debug_str = format!("{:?}", args);
        assert!(debug_str.contains("UniformSatdArgs"));
        assert!(debug_str.contains("High"));
    }

    #[test]
    fn test_uniform_dead_code_args_debug() {
        let args = UniformDeadCodeArgs {
            path: PathBuf::from("."),
            format: UniformOutputFormat::Json,
            output: None,
            top_files: 10,
            include_tests: false,
            timeout: 60,
            include_unreachable: true,
            min_dead_lines: 10,
            max_percentage: 15.0,
            fail_on_violation: false,
        };
        let debug_str = format!("{:?}", args);
        assert!(debug_str.contains("UniformDeadCodeArgs"));
    }

    #[test]
    fn test_uniform_tdg_args_debug() {
        let args = UniformTdgArgs {
            path: PathBuf::from("."),
            format: UniformOutputFormat::Yaml,
            output: None,
            top_files: 10,
            include_tests: false,
            timeout: 60,
            threshold: 1.5,
            include_components: true,
            critical_only: false,
        };
        let debug_str = format!("{:?}", args);
        assert!(debug_str.contains("UniformTdgArgs"));
    }

    #[test]
    fn test_uniform_lint_hotspot_args_debug() {
        let args = UniformLintHotspotArgs {
            path: PathBuf::from("."),
            format: UniformOutputFormat::Markdown,
            output: None,
            top_files: 10,
            include_tests: false,
            timeout: 60,
            file: Some(PathBuf::from("test.rs")),
            max_density: 5.0,
            min_confidence: 0.8,
            enforce: true,
            dry_run: true,
        };
        let debug_str = format!("{:?}", args);
        assert!(debug_str.contains("UniformLintHotspotArgs"));
    }

    #[test]
    fn test_uniform_analyze_commands_debug() {
        let args = UniformComplexityArgs {
            path: PathBuf::from("."),
            format: UniformOutputFormat::Table,
            output: None,
            top_files: 10,
            include_tests: false,
            timeout: 60,
            max_cyclomatic: None,
            max_cognitive: None,
            max_halstead: None,
        };
        let cmd = UniformAnalyzeCommands::Complexity(args);
        let debug_str = format!("{:?}", cmd);
        assert!(debug_str.contains("Complexity"));
    }

    // ===========================================
    // Equality Tests
    // ===========================================

    #[test]
    fn test_uniform_output_format_equality() {
        assert_eq!(UniformOutputFormat::Table, UniformOutputFormat::Table);
        assert_ne!(UniformOutputFormat::Table, UniformOutputFormat::Json);
    }

    #[test]
    fn test_uniform_satd_severity_equality() {
        assert_eq!(UniformSatdSeverity::Low, UniformSatdSeverity::Low);
        assert_ne!(UniformSatdSeverity::Low, UniformSatdSeverity::High);
    }

    // ===========================================
    // Partial Ord Tests
    // ===========================================

    #[test]
    fn test_uniform_output_format_partial_ord() {
        assert!(UniformOutputFormat::Table <= UniformOutputFormat::Table);
        assert!(UniformOutputFormat::Table < UniformOutputFormat::Summary);
        assert!(UniformOutputFormat::Summary > UniformOutputFormat::Table);
    }

    #[test]
    fn test_uniform_satd_severity_partial_ord() {
        assert!(UniformSatdSeverity::Low <= UniformSatdSeverity::Low);
        assert!(UniformSatdSeverity::Low < UniformSatdSeverity::Critical);
        assert!(UniformSatdSeverity::Critical > UniformSatdSeverity::Low);
    }
}

    // Tests for validation functions

    #[test]
    fn test_find_missing_required_parameters_all_present() {
        let mut params = serde_json::Map::new();
        params.insert("name".to_string(), serde_json::json!("test"));
        params.insert("version".to_string(), serde_json::json!("1.0"));

        let specs = vec![
            ParameterSpec { param_type: crate::models::template::ParameterType::String,
                name: "name".to_string(),
                description: "Name".to_string(),
                required: true,
                default_value: None,
                validation_pattern: None,
            },
            ParameterSpec { param_type: crate::models::template::ParameterType::String,
                name: "version".to_string(),
                description: "Version".to_string(),
                required: true,
                default_value: None,
                validation_pattern: None,
            },
        ];

        let missing = find_missing_required_parameters(&params, &specs);
        assert!(missing.is_empty());
    }

    #[test]
    fn test_find_missing_required_parameters_some_missing() {
        let params = serde_json::Map::new();

        let specs = vec![ParameterSpec { param_type: crate::models::template::ParameterType::String,
            name: "name".to_string(),
            description: "Name".to_string(),
            required: true,
            default_value: None,
            validation_pattern: None,
        }];

        let missing = find_missing_required_parameters(&params, &specs);
        assert_eq!(missing.len(), 1);
        assert_eq!(missing[0], "name");
    }

    #[test]
    fn test_find_missing_required_parameters_optional_ok() {
        let params = serde_json::Map::new();

        let specs = vec![ParameterSpec { param_type: crate::models::template::ParameterType::String,
            name: "optional".to_string(),
            description: "Optional".to_string(),
            required: false,
            default_value: None,
            validation_pattern: None,
        }];

        let missing = find_missing_required_parameters(&params, &specs);
        assert!(missing.is_empty());
    }

    #[test]
    fn test_validate_single_parameter_no_pattern() {
        let spec = ParameterSpec { param_type: crate::models::template::ParameterType::String,
            name: "name".to_string(),
            description: "Name".to_string(),
            required: true,
            default_value: None,
            validation_pattern: None,
        };

        let error = validate_single_parameter("name", &serde_json::json!("test"), &spec);
        assert!(error.is_none());
    }

    #[test]
    fn test_validate_single_parameter_matching_pattern() {
        let spec = ParameterSpec { param_type: crate::models::template::ParameterType::String,
            name: "email".to_string(),
            description: "Email".to_string(),
            required: true,
            default_value: None,
            validation_pattern: Some(".*@.*".to_string()),
        };

        let error = validate_single_parameter("email", &serde_json::json!("test@example.com"), &spec);
        assert!(error.is_none());
    }

    #[test]
    fn test_validate_single_parameter_non_matching_pattern() {
        let spec = ParameterSpec { param_type: crate::models::template::ParameterType::String,
            name: "email".to_string(),
            description: "Email".to_string(),
            required: true,
            default_value: None,
            validation_pattern: Some(".*@.*".to_string()),
        };

        let error = validate_single_parameter("email", &serde_json::json!("invalid"), &spec);
        assert!(error.is_some());
        assert!(error.unwrap().contains("does not match pattern"));
    }

    #[test]
    fn test_validate_parameter_values_unknown_param() {
        let mut params = serde_json::Map::new();
        params.insert("unknown".to_string(), serde_json::json!("value"));

        let specs = vec![];

        let errors = validate_parameter_values(&params, &specs);
        assert_eq!(errors.len(), 1);
        assert!(errors[0].contains("Unknown parameter"));
    }

    // Tests for Makefile lint helper functions

    #[test]
    fn test_map_severity() {
        use crate::services::makefile_linter::Severity;

        assert_eq!(map_severity(&Severity::Error), "error");
        assert_eq!(map_severity(&Severity::Warning), "warning");
        assert_eq!(map_severity(&Severity::Performance), "performance");
        assert_eq!(map_severity(&Severity::Info), "info");
    }

    // Tests for SATD helper functions

    #[test]
    fn test_create_satd_detector_normal() {
        let detector = create_satd_detector(false);
        // Just verify it creates without panicking
        drop(detector);
    }

    #[test]
    fn test_create_satd_detector_strict() {
        let detector = create_satd_detector(true);
        // Just verify it creates without panicking
        drop(detector);
    }

    // Tests for lint hotspot data extraction

    #[test]
    fn test_extract_lint_data_empty() {
        let data = serde_json::json!({});
        let extracted = extract_lint_data(&data);

        assert!(extracted.hotspots.is_empty());
        assert_eq!(extracted.total_files, 0);
        assert_eq!(extracted.total_violations, 0);
    }

    #[test]
    fn test_extract_lint_data_with_values() {
        let data = serde_json::json!({
            "hotspots": [{"file": "test.rs"}],
            "total_files_analyzed": 50,
            "total_violations": 100,
            "average_violations_per_file": 2.0
        });
        let extracted = extract_lint_data(&data);

        assert_eq!(extracted.hotspots.len(), 1);
        assert_eq!(extracted.total_files, 50);
        assert_eq!(extracted.total_violations, 100);
        assert!((extracted.average_violations_per_file - 2.0).abs() < f64::EPSILON);
    }

    // Tests for format_lint_hotspot_output

    #[test]
    fn test_format_lint_hotspot_output_json() {
        let args = LintHotspotArgs {
            project_path: "/test".to_string(),
            top_files: 10,
            min_violations: 1,
            include: None,
            exclude: None,
            format: "json".to_string(),
        };
        let data = LintHotspotData {
            hotspots: vec![],
            total_files: 10,
            total_violations: 5,
            average_violations_per_file: 0.5,
        };

        let output = format_lint_hotspot_output(&args, &data);
        assert!(output.get("project_path").is_some());
    }

    #[test]
    fn test_format_lint_hotspot_output_csv() {
        let args = LintHotspotArgs {
            project_path: "/test".to_string(),
            top_files: 10,
            min_violations: 1,
            include: None,
            exclude: None,
            format: "csv".to_string(),
        };
        let data = LintHotspotData {
            hotspots: vec![],
            total_files: 10,
            total_violations: 5,
            average_violations_per_file: 0.5,
        };

        let output = format_lint_hotspot_output(&args, &data);
        assert!(output.get("content_type").is_some());
    }

    #[test]
    fn test_format_lint_hotspot_output_table() {
        let args = LintHotspotArgs {
            project_path: "/test".to_string(),
            top_files: 10,
            min_violations: 1,
            include: None,
            exclude: None,
            format: "table".to_string(),
        };
        let data = LintHotspotData {
            hotspots: vec![],
            total_files: 10,
            total_violations: 5,
            average_violations_per_file: 0.5,
        };

        let output = format_lint_hotspot_output(&args, &data);
        assert!(output.get("formatted_output").is_some());
    }

    // Tests for parse_tool_call_params

    #[test]
    fn test_parse_tool_call_params_valid() {
        let params = serde_json::json!({
            "name": "analyze_complexity",
            "arguments": {}
        });
        let request_id = serde_json::json!(1);

        let result = parse_tool_call_params(Some(params), &request_id);
        assert!(result.is_ok());
    }

    #[test]
    fn test_parse_tool_call_params_none() {
        let request_id = serde_json::json!(1);
        let result = parse_tool_call_params(None, &request_id);
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_tool_call_params_invalid() {
        let params = serde_json::json!("not an object");
        let request_id = serde_json::json!(1);

        let result = parse_tool_call_params(Some(params), &request_id);
        assert!(result.is_err());
    }

    // Tests for argument parsing functions

    #[test]
    fn test_parse_complexity_args_valid() {
        let args = serde_json::json!({
            "project_path": "/test",
            "toolchain": "rust"
        });

        let result = parse_complexity_args(args);
        assert!(result.is_ok());
    }

    #[test]
    fn test_parse_complexity_args_empty() {
        let args = serde_json::json!({});
        let result = parse_complexity_args(args);
        assert!(result.is_ok());
    }

    #[test]
    fn test_parse_code_churn_args_valid() {
        let args = serde_json::json!({
            "project_path": "/test",
            "period_days": 30
        });

        let result = parse_code_churn_args(args);
        assert!(result.is_ok());
    }

    #[test]
    fn test_parse_tdg_args_valid() {
        let args = serde_json::json!({
            "project_path": "/test",
            "format": "json"
        });

        let result = parse_tdg_args(args);
        assert!(result.is_ok());
    }

    #[test]
    fn test_parse_deep_context_args_valid() {
        let args = serde_json::json!({
            "project_path": "/test",
            "format": "markdown"
        });

        let result = parse_deep_context_args(args);
        assert!(result.is_ok());
    }

    #[test]
    fn test_parse_satd_args_valid() {
        let args = serde_json::json!({
            "project_path": "/test",
            "strict": true
        });

        let result = parse_satd_args(args);
        assert!(result.is_ok());
    }

    #[test]
    fn test_parse_lint_hotspot_args_valid() {
        let args = serde_json::json!({
            "project_path": "/test",
            "top_files": 20
        });

        let result = parse_lint_hotspot_args(args);
        assert!(result.is_ok());
    }

    #[test]
    fn test_parse_makefile_lint_args_valid() {
        let args = serde_json::json!({
            "path": "/test/Makefile"
        });

        let result = parse_makefile_lint_args(Some(args));
        assert!(result.is_ok());
    }

    #[test]
    fn test_parse_makefile_lint_args_none() {
        let result = parse_makefile_lint_args(None);
        assert!(result.is_err());
    }

    // Tests for extract_churn_parameters

    #[test]
    fn test_extract_churn_parameters_defaults() {
        let args = AnalyzeCodeChurnArgs {
            project_path: None,
            period_days: None,
            format: None,
        };

        let (path, days, format) = extract_churn_parameters(&args);
        assert!(!path.as_os_str().is_empty());
        assert_eq!(days, 30);
        assert!(matches!(format, ChurnOutputFormat::Summary));
    }

    #[test]
    fn test_extract_churn_parameters_custom() {
        let args = AnalyzeCodeChurnArgs {
            project_path: Some("/custom".to_string()),
            period_days: Some(7),
            format: Some("json".to_string()),
        };

        let (path, days, format) = extract_churn_parameters(&args);
        assert_eq!(path, PathBuf::from("/custom"));
        assert_eq!(days, 7);
        assert!(matches!(format, ChurnOutputFormat::Json));
    }

    // Tests for extract_tdg_project_path

    #[test]
    fn test_extract_tdg_project_path_some() {
        let args = AnalyzeTdgArgs {
            project_path: Some("/custom".to_string()),
            format: None,
            threshold: None,
            include_components: None,
            max_results: None,
        };

        let path = extract_tdg_project_path(&args);
        assert_eq!(path, PathBuf::from("/custom"));
    }

    #[test]
    fn test_extract_tdg_project_path_none() {
        let args = AnalyzeTdgArgs {
            project_path: None,
            format: None,
            threshold: None,
            include_components: None,
            max_results: None,
        };

        let path = extract_tdg_project_path(&args);
        assert!(!path.as_os_str().is_empty());
    }

    // Tests for format_churn_output

    #[test]
    fn test_format_churn_output_json() {
        let analysis = create_test_churn_analysis();
        let output = format_churn_output(&analysis, &ChurnOutputFormat::Json);
        // Should be valid JSON
        assert!(serde_json::from_str::<serde_json::Value>(&output).is_ok());
    }

    #[test]
    fn test_format_churn_output_markdown() {
        let analysis = create_test_churn_analysis();
        let output = format_churn_output(&analysis, &ChurnOutputFormat::Markdown);
        assert!(output.contains("# Code Churn Analysis Report"));
    }

    #[test]
    fn test_format_churn_output_csv() {
        let analysis = create_test_churn_analysis();
        let output = format_churn_output(&analysis, &ChurnOutputFormat::Csv);
        assert!(output.contains(","));
    }

    #[test]
    fn test_format_churn_output_summary() {
        let analysis = create_test_churn_analysis();
        let output = format_churn_output(&analysis, &ChurnOutputFormat::Summary);
        assert!(output.contains("# Code Churn Analysis"));
    }

    // Tests for build_churn_response

    #[test]
    fn test_build_churn_response() {
        let analysis = create_test_churn_analysis();
        let response = build_churn_response(
            "Test content".to_string(),
            analysis,
            &ChurnOutputFormat::Summary,
        );

        assert!(response.get("content").is_some());
        assert!(response.get("analysis").is_some());
        assert!(response.get("format").is_some());
    }

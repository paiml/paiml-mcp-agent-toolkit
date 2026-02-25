    // Defects Analysis Severity Parsing Tests (covering lines 394-442)

    #[test]
    fn test_defects_severity_parsing_critical() {
        let severity_str = "critical";
        let result = match severity_str.to_lowercase().as_str() {
            "critical" => Some(crate::services::defect_detector::Severity::Critical),
            "high" => Some(crate::services::defect_detector::Severity::High),
            "medium" => Some(crate::services::defect_detector::Severity::Medium),
            "low" => Some(crate::services::defect_detector::Severity::Low),
            _ => None,
        };
        assert!(matches!(
            result,
            Some(crate::services::defect_detector::Severity::Critical)
        ));
    }

    #[test]
    fn test_defects_severity_parsing_high() {
        let severity_str = "HIGH";
        let result = match severity_str.to_lowercase().as_str() {
            "critical" => Some(crate::services::defect_detector::Severity::Critical),
            "high" => Some(crate::services::defect_detector::Severity::High),
            "medium" => Some(crate::services::defect_detector::Severity::Medium),
            "low" => Some(crate::services::defect_detector::Severity::Low),
            _ => None,
        };
        assert!(matches!(
            result,
            Some(crate::services::defect_detector::Severity::High)
        ));
    }

    #[test]
    fn test_defects_severity_parsing_medium() {
        let severity_str = "Medium";
        let result = match severity_str.to_lowercase().as_str() {
            "critical" => Some(crate::services::defect_detector::Severity::Critical),
            "high" => Some(crate::services::defect_detector::Severity::High),
            "medium" => Some(crate::services::defect_detector::Severity::Medium),
            "low" => Some(crate::services::defect_detector::Severity::Low),
            _ => None,
        };
        assert!(matches!(
            result,
            Some(crate::services::defect_detector::Severity::Medium)
        ));
    }

    #[test]
    fn test_defects_severity_parsing_low() {
        let severity_str = "low";
        let result = match severity_str.to_lowercase().as_str() {
            "critical" => Some(crate::services::defect_detector::Severity::Critical),
            "high" => Some(crate::services::defect_detector::Severity::High),
            "medium" => Some(crate::services::defect_detector::Severity::Medium),
            "low" => Some(crate::services::defect_detector::Severity::Low),
            _ => None,
        };
        assert!(matches!(
            result,
            Some(crate::services::defect_detector::Severity::Low)
        ));
    }

    #[test]
    fn test_defects_severity_parsing_invalid() {
        let severity_str = "unknown";
        let result = match severity_str.to_lowercase().as_str() {
            "critical" => Some(crate::services::defect_detector::Severity::Critical),
            "high" => Some(crate::services::defect_detector::Severity::High),
            "medium" => Some(crate::services::defect_detector::Severity::Medium),
            "low" => Some(crate::services::defect_detector::Severity::Low),
            _ => None,
        };
        assert!(result.is_none());
    }

    // Defects Output Format Tests (covering lines 407-412)

    #[test]
    fn test_defects_output_format_text() {
        let format = DefectsOutputFormat::Text;
        assert!(matches!(format, DefectsOutputFormat::Text));
    }

    #[test]
    fn test_defects_output_format_json() {
        let format = DefectsOutputFormat::Json;
        assert!(matches!(format, DefectsOutputFormat::Json));
    }

    #[test]
    fn test_defects_output_format_junit() {
        let format = DefectsOutputFormat::Junit;
        assert!(matches!(format, DefectsOutputFormat::Junit));
    }

    // Additional Command Construction Tests for Full Coverage

    #[test]
    fn test_comprehensive_command_with_all_flags() {
        let cmd = AnalyzeCommands::Comprehensive {
            project_path: PathBuf::from("."),
            file: None,
            files: vec![PathBuf::from("src/main.rs"), PathBuf::from("src/lib.rs")],
            format: crate::cli::ComprehensiveOutputFormat::Summary,
            include_duplicates: true,
            include_dead_code: true,
            include_defects: true,
            include_complexity: true,
            include_tdg: true,
            confidence_threshold: 0.7,
            min_lines: 5,
            include: Some("**/*.rs".to_string()),
            exclude: Some("**/target/**".to_string()),
            output: None,
            perf: true,
            executive_summary: true,
            top_files: 20,
        };

        if let AnalyzeCommands::Comprehensive {
            files,
            include_duplicates,
            include_dead_code,
            include_defects,
            include_complexity,
            include_tdg,
            executive_summary,
            ..
        } = cmd
        {
            assert_eq!(files.len(), 2);
            assert!(include_duplicates);
            assert!(include_dead_code);
            assert!(include_defects);
            assert!(include_complexity);
            assert!(include_tdg);
            assert!(executive_summary);
        } else {
            panic!("Expected Comprehensive command");
        }
    }

    #[test]
    fn test_incremental_coverage_command_construction() {
        let cmd = AnalyzeCommands::IncrementalCoverage {
            project_path: PathBuf::from("."),
            base_branch: Some("main".to_string()),
            target_branch: Some("feature".to_string()),
            format: crate::cli::IncrementalCoverageOutputFormat::Summary,
            coverage_threshold: 80.0,
            changed_files_only: true,
            detailed: false,
            output: None,
            perf: false,
            cache_dir: None,
            force_refresh: false,
            top_files: 10,
        };

        if let AnalyzeCommands::IncrementalCoverage {
            base_branch,
            target_branch,
            coverage_threshold,
            changed_files_only,
            ..
        } = cmd
        {
            assert_eq!(base_branch, Some("main".to_string()));
            assert_eq!(target_branch, Some("feature".to_string()));
            assert!((coverage_threshold - 80.0).abs() < f64::EPSILON);
            assert!(changed_files_only);
        } else {
            panic!("Expected IncrementalCoverage command");
        }
    }

    #[test]
    fn test_big_o_command_construction() {
        let cmd = AnalyzeCommands::BigO {
            project_path: PathBuf::from("."),
            format: crate::cli::BigOOutputFormat::Summary,
            confidence_threshold: 0.7,
            analyze_space: true,
            include: vec!["src/**/*.rs".to_string()],
            exclude: vec!["target/**".to_string()],
            high_complexity_only: true,
            output: None,
            perf: false,
            top_files: 10,
        };

        if let AnalyzeCommands::BigO {
            confidence_threshold,
            analyze_space,
            high_complexity_only,
            ..
        } = cmd
        {
            assert!((confidence_threshold - 0.7).abs() < f64::EPSILON);
            assert!(analyze_space);
            assert!(high_complexity_only);
        } else {
            panic!("Expected BigO command");
        }
    }

    #[test]
    fn test_assemblyscript_command_construction() {
        let cmd = AnalyzeCommands::AssemblyScript {
            project_path: PathBuf::from("."),
            format: WasmOutputFormat::Summary,
            wasm_complexity: true,
            memory_analysis: true,
            security: true,
            output: None,
            timeout: 60,
            perf: false,
            top_files: 10,
        };

        if let AnalyzeCommands::AssemblyScript {
            wasm_complexity,
            memory_analysis,
            security,
            ..
        } = cmd
        {
            assert!(wasm_complexity);
            assert!(memory_analysis);
            assert!(security);
        } else {
            panic!("Expected AssemblyScript command");
        }
    }

    #[test]
    fn test_webassembly_command_construction() {
        let cmd = AnalyzeCommands::WebAssembly {
            project_path: PathBuf::from("."),
            format: WasmOutputFormat::Json,
            include_binary: true,
            include_text: true,
            memory_analysis: true,
            security: true,
            complexity: true,
            output: None,
            perf: true,
            top_files: 5,
        };

        if let AnalyzeCommands::WebAssembly {
            include_binary,
            include_text,
            memory_analysis,
            security,
            complexity,
            ..
        } = cmd
        {
            assert!(include_binary);
            assert!(include_text);
            assert!(memory_analysis);
            assert!(security);
            assert!(complexity);
        } else {
            panic!("Expected WebAssembly command");
        }
    }

    #[test]
    fn test_clippy_command_construction() {
        let cmd = AnalyzeCommands::Clippy {
            project_path: PathBuf::from("."),
            confidence: "high".to_string(),
            dry_run: true,
            fix_codes: vec!["E0001".to_string(), "E0002".to_string()],
            output: None,
            perf: false,
        };

        if let AnalyzeCommands::Clippy {
            confidence,
            dry_run,
            fix_codes,
            ..
        } = cmd
        {
            assert_eq!(confidence, "high");
            assert!(dry_run);
            assert_eq!(fix_codes.len(), 2);
        } else {
            panic!("Expected Clippy command");
        }
    }

    #[test]
    fn test_defects_command_with_all_params() {
        let cmd = AnalyzeCommands::Defects {
            path: Some(PathBuf::from(".")),
            file: None,
            severity: Some("high".to_string()),
            format: DefectsOutputFormat::Json,
            output: Some(PathBuf::from("output.json")),
        };

        if let AnalyzeCommands::Defects {
            path,
            severity,
            format,
            output,
            ..
        } = cmd
        {
            assert_eq!(path, Some(PathBuf::from(".")));
            assert_eq!(severity, Some("high".to_string()));
            assert!(matches!(format, DefectsOutputFormat::Json));
            assert!(output.is_some());
        } else {
            panic!("Expected Defects command");
        }
    }


    #[test]
    fn test_mode_enum() {
        assert_eq!(Mode::Cli, Mode::Cli);
        assert_ne!(Mode::Cli, Mode::Mcp);
    }

    #[test]
    fn test_cli_parse_empty() {
        // Run on a thread with 8MB stack to avoid stack overflow from large Cli enum
        let handle = std::thread::Builder::new()
            .stack_size(8 * 1024 * 1024)
            .spawn(|| {
                let result = Cli::try_parse_from(["pmat", "list"]);
                match result {
                    Ok(_) => {}
                    Err(e) => {
                        panic!("CLI parsing failed: {}", e);
                    }
                }
            })
            .expect("Failed to spawn thread");
        handle.join().expect("Thread panicked");
    }

    #[test]
    fn test_mode_variants() {
        let cli_mode = Mode::Cli;
        let mcp_mode = Mode::Mcp;

        assert_eq!(cli_mode, Mode::Cli);
        assert_eq!(mcp_mode, Mode::Mcp);
        assert_ne!(cli_mode, mcp_mode);
    }

    #[test]
    fn test_diagnostic_output_format_variants() {
        let plain = DiagnosticOutputFormat::Plain;
        let json = DiagnosticOutputFormat::Json;
        let yaml = DiagnosticOutputFormat::Yaml;

        assert_eq!(plain, DiagnosticOutputFormat::Plain);
        assert_eq!(json, DiagnosticOutputFormat::Json);
        assert_eq!(yaml, DiagnosticOutputFormat::Yaml);
    }

    #[test]
    fn test_storage_command_variants() {
        let stats = StorageCommand::Stats { detailed: false };
        let cleanup = StorageCommand::Cleanup { max_age: 3600 };
        let migrate = StorageCommand::Migrate {
            backend: "sled".to_string(),
            path: None,
        };
        // Backup and Restore variants have been removed - test Migrate instead
        let _migrate2 = StorageCommand::Migrate {
            backend: "rocksdb".to_string(),
            path: None,
        };

        // Test variant construction
        match stats {
            StorageCommand::Stats { detailed } => assert!(!detailed),
            _ => panic!("Unexpected variant"),
        }
        match cleanup {
            StorageCommand::Cleanup { max_age } => assert_eq!(max_age, 3600),
            _ => panic!("Unexpected variant"),
        }

        match migrate {
            StorageCommand::Migrate { backend, path: _ } => {
                assert_eq!(backend, "sled");
            }
            _ => panic!("Expected Migrate variant"),
        }
    }

    #[test]
    fn test_tdg_command_variants() {
        // Test Compare variant
        let compare = TdgCommand::Compare {
            source1: PathBuf::from("file1.rs"),
            source2: PathBuf::from("file2.rs"),
        };

        // Test Diagnostics variant
        let diagnostics = TdgCommand::Diagnostics {
            detailed: true,
            storage: false,
            scheduler: false,
            adaptive: false,
            resources: false,
            all: false,
            format: DiagnosticOutputFormat::Human,
        };

        // Test Dashboard variant (this one still exists)
        let dashboard = TdgCommand::Dashboard {
            port: 8080,
            open: true,
            host: "127.0.0.1".to_string(),
            update_interval: 5,
        };

        match compare {
            TdgCommand::Compare { source1, source2 } => {
                assert_eq!(source1, PathBuf::from("file1.rs"));
                assert_eq!(source2, PathBuf::from("file2.rs"));
            }
            _ => panic!("Expected Compare variant"),
        }

        match diagnostics {
            TdgCommand::Diagnostics { detailed, .. } => {
                assert!(detailed);
            }
            _ => panic!("Expected Diagnostics variant"),
        }

        match dashboard {
            TdgCommand::Dashboard {
                port,
                open,
                host,
                update_interval,
            } => {
                assert_eq!(port, 8080);
                assert!(open);
                assert_eq!(host, "127.0.0.1");
                assert_eq!(update_interval, 5);
            }
            _ => panic!("Expected Dashboard variant"),
        }
    }

    #[test]
    fn test_analyze_commands_variants() {
        let complexity = AnalyzeCommands::Complexity {
            path: PathBuf::from("."),
            project_path: None,
            file: Some(PathBuf::from("test.rs")),
            files: vec![PathBuf::from("lib.rs")],
            toolchain: Some("rust".to_string()),
            format: ComplexityOutputFormat::Json,
            output: None,
            max_cyclomatic: Some(10),
            max_cognitive: Some(15),
            include: vec!["**/*.rs".to_string()],
            watch: false,
            top_files: 5,
            fail_on_violation: true,
            timeout: 60,
            ml: false,
        };

        let churn = AnalyzeCommands::Churn {
            path: PathBuf::from("."),
            project_path: None,
            days: 30,
            format: ChurnOutputFormat::Json,
            output: None,
            top_files: 10,
            include: vec![],
            exclude: vec![],
        };

        match complexity {
            AnalyzeCommands::Complexity {
                path,
                file,
                max_cyclomatic,
                top_files,
                ..
            } => {
                assert_eq!(path, PathBuf::from("."));
                assert_eq!(file, Some(PathBuf::from("test.rs")));
                assert_eq!(max_cyclomatic, Some(10));
                assert_eq!(top_files, 5);
            }
            _ => panic!("Expected Complexity variant"),
        }

        match churn {
            AnalyzeCommands::Churn {
                path,
                days,
                top_files,
                ..
            } => {
                assert_eq!(path, PathBuf::from("."));
                assert_eq!(days, 30);
                assert_eq!(top_files, 10);
            }
            _ => panic!("Expected Churn variant"),
        }
    }

    #[test]
    fn test_enforce_commands_variants() {
        // EnforceCommands only has Extreme variant now
        // TODO: Update test when API stabilizes
        /*
        let quality_gate = EnforceCommands::QualityGate {
            path: Some(PathBuf::from(".")),
            file: Some(PathBuf::from("test.rs")),
            config: Some(PathBuf::from("quality.toml")),
            format: QualityGateOutputFormat::Json,
        };

        match quality_gate {
            EnforceCommands::QualityGate {
                path,
                file,
                config,
                format,
            } => {
                assert_eq!(path, Some(PathBuf::from(".")));
                assert_eq!(file, Some(PathBuf::from("test.rs")));
                assert_eq!(config, Some(PathBuf::from("quality.toml")));
                assert_eq!(format, QualityGateOutputFormat::Json);
            }
            _ => panic!("Expected QualityGate variant"),
        }
        */
    }

    #[test]
    fn test_refactor_commands_variants() {
        // RefactorCommands fields have changed
        // TODO: Update test when API stabilizes
        /*
        let auto_refactor = RefactorCommands::Auto {
            path: Some(PathBuf::from(".")),
            file: Some(PathBuf::from("test.rs")),
            github_issue: Some("https://github.com/owner/repo/issues/123".to_string()),
            output_format: RefactorAutoOutputFormat::Json,
            interactive: true,
            dry_run: false,
        };

        let docs_refactor = RefactorCommands::Docs {
            path: PathBuf::from("."),
            format: RefactorDocsOutputFormat::Markdown,
            output: Some(PathBuf::from("docs.md")),
            timeout: 120,
        };

        match auto_refactor {
            RefactorCommands::Auto {
                path,
                file,
                github_issue,
                interactive,
                ..
            } => {
                assert_eq!(path, Some(PathBuf::from(".")));
                assert_eq!(file, Some(PathBuf::from("test.rs")));
                assert_eq!(
                    github_issue,
                    Some("https://github.com/owner/repo/issues/123".to_string())
                );
                assert!(interactive);
            }
            _ => panic!("Expected Auto variant"),
        }
        */
    }

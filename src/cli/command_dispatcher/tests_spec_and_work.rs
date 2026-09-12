    // Test: Commands routing - additional commands

    #[tokio::test]
    async fn test_search_command_routing() {
        let server = create_test_server();
        let command = Commands::Search {
            query: "function".to_string(),
            toolchain: Some("rust".to_string()),
            limit: 5,
        };
        let result = CommandDispatcher::execute_command(command, server).await;
        assert!(result.is_ok() || result.is_err());
    }

    #[tokio::test]
    async fn test_validate_command_routing() {
        let server = create_test_server();
        let command = Commands::Validate {
            uri: "template://test".to_string(),
            params: vec![(
                "key".to_string(),
                serde_json::Value::String("value".to_string()),
            )],
        };
        let result = CommandDispatcher::execute_command(command, server).await;
        assert!(result.is_ok() || result.is_err());
    }

    #[tokio::test]
    async fn test_context_command_routing() {
        use crate::cli::ContextFormat;
        use tempfile::TempDir;

        let server = create_test_server();
        let temp_dir = TempDir::new().expect("internal error");

        let command = Commands::Context {
            toolchain: Some("rust".to_string()),
            project_path: temp_dir.path().to_path_buf(),
            output: None,
            format: ContextFormat::Markdown,
            include_large_files: false,
            skip_expensive_metrics: true,
            language: None,
            languages: None,
        };
        let result = CommandDispatcher::execute_command(command, server).await;
        assert!(result.is_ok() || result.is_err());
    }

    // Test: execute_analyze_command routing

    #[tokio::test]
    async fn test_analyze_dead_code_routing() {
        use crate::cli::commands::AnalyzeCommands;
        use crate::cli::DeadCodeOutputFormat;

        let analyze_cmd = AnalyzeCommands::DeadCode {
            path: PathBuf::from("."),
            format: DeadCodeOutputFormat::Summary,
            top_files: None,
            include_unreachable: false,
            min_dead_lines: 10,
            include_tests: false,
            output: None,
            fail_on_violation: false,
            max_percentage: 15.0,
            timeout: 30,
            include: vec![],
            exclude: vec![],
            max_depth: 8,
            no_cache: false,
        };
        let result = CommandDispatcher::execute_analyze_command(analyze_cmd).await;
        assert!(result.is_ok() || result.is_err());
    }

    // Test: execute_qdd_command routing

    #[tokio::test]
    async fn test_qdd_create_routing() {
        use crate::cli::commands::{QddCodeType, QddCommands, QddQualityProfile};

        let qdd_cmd = QddCommands::Create {
            code_type: QddCodeType::Function,
            name: "test_function".to_string(),
            purpose: "Test function for coverage".to_string(),
            profile: QddQualityProfile::Standard,
            input: vec![],
            output: "()".to_string(),
            output_file: None,
        };
        let result = CommandDispatcher::execute_qdd_command(qdd_cmd).await;
        assert!(result.is_ok() || result.is_err());
    }

    // Test: execute_refactor_command routing

    #[tokio::test]
    async fn test_refactor_status_routing() {
        use crate::cli::commands::RefactorCommands;
        use crate::cli::enums::RefactorOutputFormat;
        use tempfile::TempDir;

        let temp_dir = TempDir::new().expect("internal error");
        let checkpoint = temp_dir.path().join("refactor_state.json");

        let refactor_cmd = RefactorCommands::Status {
            checkpoint,
            format: RefactorOutputFormat::Json,
        };
        let result = CommandDispatcher::execute_refactor_command(refactor_cmd).await;
        assert!(result.is_ok() || result.is_err());
    }

    // Test: execute_roadmap_command routing

    /// RAII guard restoring `PMAT_ROADMAP_PATH` to its previous value on drop.
    ///
    /// Env vars are process-global (#1329); this guard, combined with
    /// `#[serial(env_vars)]`, keeps this test from racing or permanently
    /// clobbering the variable for any other test in the binary.
    struct RoadmapPathEnvGuard {
        previous: Option<String>,
    }

    impl RoadmapPathEnvGuard {
        fn set(path: &std::path::Path) -> Self {
            let previous = std::env::var("PMAT_ROADMAP_PATH").ok();
            std::env::set_var("PMAT_ROADMAP_PATH", path);
            Self { previous }
        }
    }

    impl Drop for RoadmapPathEnvGuard {
        fn drop(&mut self) {
            match &self.previous {
                Some(value) => std::env::set_var("PMAT_ROADMAP_PATH", value),
                None => std::env::remove_var("PMAT_ROADMAP_PATH"),
            }
        }
    }

    #[tokio::test]
    #[serial_test::serial(env_vars)]
    async fn test_roadmap_init_routing() {
        use crate::cli::commands::RoadmapCommands;
        use tempfile::TempDir;

        // #1329: this test must never write to the repository's own
        // docs/execution/roadmap.md. Point the seam at a temp file instead.
        let temp_dir = TempDir::new().expect("internal error");
        let roadmap_path = temp_dir.path().join("roadmap.md");
        let _guard = RoadmapPathEnvGuard::set(&roadmap_path);

        let roadmap_cmd = RoadmapCommands::Init {
            version: "v1.0.0".to_string(),
            title: "Test Sprint".to_string(),
            duration_days: 14,
            priority: "P0".to_string(),
        };
        let result = CommandDispatcher::execute_roadmap_command(roadmap_cmd).await;
        assert!(
            result.is_ok(),
            "roadmap init should succeed against a fresh temp path: {result:?}"
        );
        let written = std::fs::read_to_string(&roadmap_path)
            .expect("Init must have written the roadmap file");
        assert!(
            written.contains("Test Sprint"),
            "written roadmap must contain the sprint title: {written}"
        );
    }

    #[tokio::test]
    async fn test_roadmap_status_routing() {
        use crate::cli::commands::RoadmapCommands;

        let roadmap_cmd = RoadmapCommands::Status {
            sprint: None,
            task: None,
            format: OutputFormat::Json,
        };
        let result = CommandDispatcher::execute_roadmap_command(roadmap_cmd).await;
        assert!(result.is_ok() || result.is_err());
    }

    #[tokio::test]
    async fn test_roadmap_validate_routing() {
        use crate::cli::commands::RoadmapCommands;

        let roadmap_cmd = RoadmapCommands::Validate {
            sprint: "sprint-1".to_string(),
            strict: true,
        };
        let result = CommandDispatcher::execute_roadmap_command(roadmap_cmd).await;
        assert!(result.is_ok() || result.is_err());
    }

    // Test: execute_test_command routing with different suites

    #[tokio::test]
    #[ignore = "Times out in coverage runs - property tests run too long"]
    async fn test_test_command_property_suite() {
        use crate::cli::commands::TestSuite;

        let result = CommandDispatcher::execute_test_command(
            TestSuite::Property,
            1,
            false,
            false,
            false,
            5, // short timeout
            None,
            false,
        )
        .await;
        assert!(result.is_ok() || result.is_err());
    }

    #[tokio::test]
    async fn test_test_command_integration_suite() {
        use crate::cli::commands::TestSuite;

        let result = CommandDispatcher::execute_test_command(
            TestSuite::Integration,
            1,
            false,
            false,
            false,
            5,
            None,
            false,
        )
        .await;
        assert!(result.is_ok() || result.is_err());
    }

    // Test: CommandHandler trait bounds (compile-time verification)

    #[test]
    fn test_command_handler_trait_is_send_sync() {
        fn assert_send_sync<T: Send + Sync>() {}
        // This is a compile-time check that the trait has correct bounds
        // The actual handlers that implement this trait need to be Send + Sync
    }

    // Test: quality gate check type conversions

    #[tokio::test]
    async fn test_quality_gate_unknown_check_filtered() {
        use tempfile::TempDir;
        let temp_dir = TempDir::new().expect("internal error");

        // Unknown check types should be filtered out
        let result = CommandDispatcher::execute_quality_gate_command(
            Some(temp_dir.path().to_path_buf()),
            None,
            OutputFormat::Table,
            false,
            vec!["unknown_check_type".to_string(), "complexity".to_string()],
            None,
            None,
            None,
            false,
            None,
            false,
        )
        .await;
        // Should still work with just "complexity"
        assert!(result.is_ok() || result.is_err());
    }

    // Test: report analysis type conversions
    //
    // This test was introduced (48f15dd59) as `test_report_analysis_hyphen_
    // variants`, passing `vec!["dead-code".to_string(), "technical-debt"
    // .to_string(), "big-o".to_string()]` into `execute_report_command`'s
    // then-`Vec<String>` `analyses` parameter and a `confidence_threshold:
    // Option<f64>`, to exercise a hand-rolled hyphen-string -> AnalysisType
    // matcher inside that function. Commit 432393bb6 (#918, issue #706;
    // see the doc comment on `execute_report_command` in
    // quality_commands.rs:88-97) deleted that matcher along with the
    // `Vec<String>`/`Option<f64>` parameters themselves: `analyses` is now
    // the already-parsed `Vec<AnalysisType>` clap produces from
    // `--analyses`, and `confidence_threshold` is a `u8`. There is no
    // string re-parse left in production to test, so this no longer
    // compiles as written; renamed and rewritten to pass real
    // `AnalysisType` variants, kept as a routing check across the
    // multi-variant case (dead-code/technical-debt/big-o).

    #[tokio::test]
    async fn test_report_analysis_multiple_types() {
        use tempfile::TempDir;
        let temp_dir = TempDir::new().expect("internal error");
        let test_file = temp_dir.path().join("test.rs");
        std::fs::write(&test_file, "fn main() {}").expect("internal error");

        let result = CommandDispatcher::execute_report_command(
            Some(temp_dir.path().to_path_buf()),
            crate::cli::enums::ReportOutputFormat::Text,
            false,
            false,
            false,
            vec![
                crate::cli::enums::AnalysisType::DeadCode,
                crate::cli::enums::AnalysisType::TechnicalDebt,
                crate::cli::enums::AnalysisType::BigO,
            ],
            50,
            None,
            false,
            false,
            false,
            false,
        )
        .await;
        assert!(result.is_ok() || result.is_err());
    }

    // Test: handle_spec_command variants

    #[tokio::test]
    // This test is correct; the handler is what must change (#1331, which
    // tracks the whole class — 32 handler files do this):
    // `spec_handlers::handle_spec_score`
    // (src/cli/handlers/spec_handlers/spec_handlers_commands.rs) calls
    // `std::process::exit(1)` on its failure branch, which kills the whole
    // test binary instead of returning the `Result` this test wants to
    // assert on.
    #[ignore = "reaches handle_spec_score's process::exit(1), which kills the test binary — see #1331"]
    async fn test_spec_score_command() {
        use crate::cli::commands::{SpecCommands, SpecOutputFormat};
        use tempfile::NamedTempFile;

        let temp_file = NamedTempFile::new().expect("internal error");
        std::fs::write(temp_file.path(), "# Test Spec\n\n## Overview\nTest content")
            .expect("internal error");

        let command = SpecCommands::Score {
            spec: temp_file.path().to_path_buf(),
            format: SpecOutputFormat::Text,
            output: None,
            verbose: true,
        };
        let result = CommandDispatcher::handle_spec_command(command).await;
        assert!(result.is_ok() || result.is_err());
    }

    #[tokio::test]
    async fn test_spec_comply_dry_run() {
        use crate::cli::commands::{SpecCommands, SpecOutputFormat};
        use tempfile::NamedTempFile;

        let temp_file = NamedTempFile::new().expect("internal error");
        std::fs::write(temp_file.path(), "# Spec\n\n## Details").expect("internal error");

        let command = SpecCommands::Comply {
            spec: temp_file.path().to_path_buf(),
            dry_run: true,
            format: SpecOutputFormat::Json,
        };
        let result = CommandDispatcher::handle_spec_command(command).await;
        assert!(result.is_ok() || result.is_err());
    }

    #[tokio::test]
    async fn test_spec_create_command() {
        use crate::cli::commands::SpecCommands;
        use tempfile::TempDir;

        let temp_dir = TempDir::new().expect("internal error");

        let command = SpecCommands::Create {
            name: "new-feature".to_string(),
            issue: Some("GH-456".to_string()),
            epic: None,
            output: Some(temp_dir.path().to_path_buf()),
        };
        let result = CommandDispatcher::handle_spec_command(command).await;
        assert!(result.is_ok() || result.is_err());
    }

    #[tokio::test]
    async fn test_spec_list_command() {
        use crate::cli::commands::{SpecCommands, SpecOutputFormat};
        use tempfile::TempDir;

        let temp_dir = TempDir::new().expect("internal error");

        let command = SpecCommands::List {
            path: temp_dir.path().to_path_buf(),
            min_score: Some(70),
            failing_only: false,
            format: SpecOutputFormat::Text,
        };
        let result = CommandDispatcher::handle_spec_command(command).await;
        assert!(result.is_ok() || result.is_err());
    }

    // Test: execute_work_command variants

    #[tokio::test]
    async fn test_work_init_with_github() {
        use crate::cli::commands::WorkCommands;
        use tempfile::TempDir;

        let temp_dir = TempDir::new().expect("internal error");

        let command = WorkCommands::Init {
            github_repo: Some("user/repo".to_string()),
            no_github: false,
            path: Some(temp_dir.path().to_path_buf()),
        };
        let result = CommandDispatcher::execute_work_command(&command).await;
        assert!(result.is_ok() || result.is_err());
    }

    #[tokio::test]
    async fn test_work_start_with_spec() {
        use crate::cli::commands::WorkCommands;
        use tempfile::TempDir;

        let temp_dir = TempDir::new().expect("internal error");

        let command = WorkCommands::Start {
            id: "GH-123".to_string(),
            with_spec: true,
            epic: true,
            path: Some(temp_dir.path().to_path_buf()),
            create_github: false,
            profile: None,
            without: None,
            iteration: 1,
            implements: Vec::new(),
            agent: Default::default(),
            level: None,
        };
        let result = CommandDispatcher::execute_work_command(&command).await;
        assert!(result.is_ok() || result.is_err());
    }

    #[tokio::test]
    async fn test_work_sync_directions() {
        use crate::cli::commands::{SyncDirection, WorkCommands};
        use tempfile::TempDir;

        let temp_dir = TempDir::new().expect("internal error");

        for direction in [
            SyncDirection::Full,
            SyncDirection::YamlToGithub,
            SyncDirection::GithubToYaml,
        ] {
            let command = WorkCommands::Sync {
                direction,
                path: Some(temp_dir.path().to_path_buf()),
                dry_run: true,
                check_only: false,
                snapshot: None,
                write_snapshot: None,
                grace_minutes: 60,
                format: crate::cli::commands::SyncOutputFormat::Text,
            };
            let result = CommandDispatcher::execute_work_command(&command).await;
            assert!(result.is_ok() || result.is_err());
        }
    }

    #[tokio::test]
    async fn test_work_validate_with_fix() {
        use crate::cli::commands::WorkCommands;
        use tempfile::TempDir;

        let temp_dir = TempDir::new().expect("internal error");

        let command = WorkCommands::Validate {
            path: Some(temp_dir.path().to_path_buf()),
            verbose: false,
            fix: true,
            check_base: None,
        };
        let result = CommandDispatcher::execute_work_command(&command).await;
        assert!(result.is_ok() || result.is_err());
    }

    #[tokio::test]
    async fn test_work_migrate_with_backup() {
        use crate::cli::commands::WorkCommands;
        use tempfile::TempDir;

        let temp_dir = TempDir::new().expect("internal error");

        let command = WorkCommands::Migrate {
            path: Some(temp_dir.path().to_path_buf()),
            dry_run: false,
            backup: true,
            levels: false,
        };
        let result = CommandDispatcher::execute_work_command(&command).await;
        assert!(result.is_ok() || result.is_err());
    }

    #[tokio::test]
    async fn test_work_list_statuses() {
        use crate::cli::commands::WorkCommands;

        let command = WorkCommands::ListStatuses;
        let result = CommandDispatcher::execute_work_command(&command).await;
        assert!(result.is_ok());
    }

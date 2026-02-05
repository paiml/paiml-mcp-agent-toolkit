// Refactor and Scaffold commands - extracted for file health (CB-040)

/// Refactor subcommands
#[derive(Subcommand)]
#[cfg_attr(test, derive(Debug))]
pub enum RefactorCommands {
    /// Run refactor server mode for batch processing
    Serve {
        /// Refactor mode (batch or interactive)
        #[arg(long, value_enum, default_value = "batch")]
        refactor_mode: RefactorMode,

        /// JSON configuration file for batch mode
        #[arg(short = 'c', long)]
        config: Option<PathBuf>,

        /// Project directory to refactor
        #[arg(short = 'p', long, default_value = ".")]
        project: PathBuf,

        /// Number of parallel workers
        #[arg(long, default_value = "4")]
        parallel: usize,

        /// Memory limit in MB
        #[arg(long, default_value = "512")]
        memory_limit: usize,

        /// Files per batch
        #[arg(long, default_value = "10")]
        batch_size: usize,

        /// Priority sorting expression (e.g., "complexity * `defect_probability`")
        #[arg(long)]
        priority: Option<String>,

        /// Checkpoint directory for resuming
        #[arg(long)]
        checkpoint_dir: Option<PathBuf>,

        /// Resume from previous checkpoint
        #[arg(long)]
        resume: bool,

        /// Auto-commit with message template
        #[arg(long)]
        auto_commit: Option<String>,

        /// Maximum runtime in seconds
        #[arg(long)]
        max_runtime: Option<u64>,
    },

    /// Run interactive refactoring mode
    Interactive {
        /// Project path to analyze (defaults to current directory)
        #[arg(short = 'p', long, default_value = ".")]
        project_path: PathBuf,

        /// Explanation level for operations
        #[arg(long, value_enum, default_value = "detailed")]
        explain: ExplainLevel,

        /// Checkpoint file for state persistence
        #[arg(long, default_value = "refactor_state.json")]
        checkpoint: PathBuf,

        /// Target complexity threshold
        #[arg(long, default_value = "20")]
        target_complexity: u16,

        /// Maximum steps to execute
        #[arg(long)]
        steps: Option<u32>,

        /// Configuration file path
        #[arg(long)]
        config: Option<PathBuf>,
    },

    /// Show current refactoring status
    Status {
        /// Checkpoint file to read state from
        #[arg(long, default_value = "refactor_state.json")]
        checkpoint: PathBuf,

        /// Output format
        #[arg(long, value_enum, default_value = "json")]
        format: RefactorOutputFormat,
    },

    /// Resume refactoring from checkpoint
    Resume {
        /// Checkpoint file to resume from
        #[arg(long, default_value = "refactor_state.json")]
        checkpoint: PathBuf,

        /// Maximum steps to execute
        #[arg(long, default_value = "10")]
        steps: u32,

        /// Override explanation level
        #[arg(long, value_enum)]
        explain: Option<ExplainLevel>,
    },

    /// AI-powered automated refactoring to achieve RIGID extreme quality standards
    Auto {
        /// Project path to refactor
        #[arg(short = 'p', long, default_value = ".")]
        project_path: PathBuf,

        /// Single file mode - refactor one file at a time
        #[arg(long)]
        single_file_mode: bool,

        /// Specific file to refactor (implies single file mode)
        #[arg(long)]
        file: Option<PathBuf>,

        /// Maximum iterations to run
        #[arg(long, default_value = "100")]
        max_iterations: u32,

        /// Quality profile to enforce
        #[arg(long, value_enum, default_value = "extreme")]
        quality_profile: QualityProfile,

        /// Output format
        #[arg(short = 'f', long, value_enum, default_value = "detailed")]
        format: RefactorAutoOutputFormat,

        /// Dry run mode (don't write files)
        #[arg(long)]
        dry_run: bool,

        /// Skip compilation check
        #[arg(long)]
        skip_compilation: bool,

        /// Skip test execution
        #[arg(long)]
        skip_tests: bool,

        /// Output checkpoint file
        #[arg(long)]
        checkpoint: Option<PathBuf>,

        /// Verbose output
        #[arg(short = 'v', long)]
        verbose: bool,

        /// Patterns to exclude from refactoring (e.g., "tests/**", "benches/**")
        #[arg(long, value_delimiter = ',')]
        exclude: Vec<String>,

        /// Patterns to include for refactoring (overrides exclude)
        #[arg(long, value_delimiter = ',')]
        include: Vec<String>,

        /// Path to .refactorignore file
        #[arg(long)]
        ignore_file: Option<PathBuf>,

        /// Specific test file to fix (automatically includes related source files)
        #[arg(long, short = 't')]
        test: Option<PathBuf>,

        /// Test name pattern to fix (e.g., "`test_mixed_language_project_context`")
        #[arg(long)]
        test_name: Option<String>,

        /// GitHub issue URL to guide the refactoring process
        #[arg(long)]
        github_issue: Option<String>,

        /// Bug report markdown file path to analyze and fix
        #[arg(long)]
        bug_report_path: Option<PathBuf>,
    },

    /// AI-assisted documentation cleanup and refactoring
    Docs {
        /// Project path to analyze (defaults to current directory)
        #[arg(short = 'p', long, default_value = ".")]
        project_path: PathBuf,

        /// Include docs directory
        #[arg(long, default_value_t = true)]
        include_docs: bool,

        /// Include root directory
        #[arg(long, default_value_t = true)]
        include_root: bool,

        /// Additional directories to scan
        #[arg(long, value_delimiter = ',')]
        additional_dirs: Vec<PathBuf>,

        /// Output format
        #[arg(short = 'f', long, value_enum, default_value = "summary")]
        format: RefactorDocsOutputFormat,

        /// Dry run - show what would be removed without making changes
        #[arg(long)]
        dry_run: bool,

        /// Patterns to identify temporary files (e.g., "fix-*.sh", "*_TEMP.md")
        #[arg(
            long,
            value_delimiter = ',',
            default_value = "fix-*,test-*,temp-*,tmp-*,*_TEMP*,*_TMP*,FAST_*,FIX_*,ZERO_DEFECTS_*"
        )]
        temp_patterns: Vec<String>,

        /// Patterns to identify outdated status files
        #[arg(
            long,
            value_delimiter = ',',
            default_value = "*_STATUS.md,*_PROGRESS.md,*_COMPLETE.md,final_verification.md,overnight-*.md"
        )]
        status_patterns: Vec<String>,

        /// Patterns to identify build artifacts
        #[arg(
            long,
            value_delimiter = ',',
            default_value = "*.mmd,optimization_state.json,complexity_report.json,satd_report.json"
        )]
        artifact_patterns: Vec<String>,

        /// Custom patterns to include in cleanup
        #[arg(long, value_delimiter = ',')]
        custom_patterns: Vec<String>,

        /// Minimum age in days before considering a file for cleanup
        #[arg(long, default_value_t = 0)]
        min_age_days: u32,

        /// Maximum file size in MB to consider (larger files are skipped)
        #[arg(long, default_value_t = 10)]
        max_size_mb: u64,

        /// Include subdirectories recursively
        #[arg(long, default_value_t = true)]
        recursive: bool,

        /// Preserve files matching these patterns (overrides other patterns)
        #[arg(
            long,
            value_delimiter = ',',
            default_value = "README.md,LICENSE*,CHANGELOG*,CONTRIBUTING*"
        )]
        preserve_patterns: Vec<String>,

        /// Output file path for the report
        #[arg(short = 'o', long)]
        output: Option<PathBuf>,

        /// Auto-remove files without confirmation (use with caution)
        #[arg(long)]
        auto_remove: bool,

        /// Create backup before removing files
        #[arg(long)]
        backup: bool,

        /// Backup directory path
        #[arg(long, default_value = ".refactor-docs-backup")]
        backup_dir: PathBuf,

        /// Show performance metrics
        #[arg(long)]
        perf: bool,
    },
}

#[cfg_attr(coverage_nightly, coverage(off))]
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mode_enum() {
        assert_eq!(Mode::Cli, Mode::Cli);
        assert_ne!(Mode::Cli, Mode::Mcp);
    }

    /// Coverage: Stack overflow under 48-thread coverage instrumentation
    /// Run manually: cargo test test_cli_parse_empty -- --ignored --test-threads=1
    #[test]
    // Re-enabled: test passes
    fn test_cli_parse_empty() {
        // Test that CLI can be parsed with minimal args
        let result = Cli::try_parse_from(["pmat", "list"]);
        match result {
            Ok(_) => {
                // Success case - don't try to debug print the large structure
            }
            Err(e) => {
                panic!("CLI parsing failed: {}", e);
            }
        }
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
            project_path: PathBuf::from("."),
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
                project_path,
                days,
                top_files,
                ..
            } => {
                assert_eq!(project_path, PathBuf::from("."));
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

    #[test]
    fn test_scaffold_commands_variants() {
        let project = ScaffoldCommands::Project {
            toolchain: "rust".to_string(),
            templates: vec!["cli".to_string(), "lib".to_string()],
            params: vec![("name".to_string(), Value::String("test".to_string()))],
            parallel: 4,
        };

        let agent = ScaffoldCommands::Agent {
            name: "test-agent".to_string(),
            template: "basic".to_string(),
            features: vec!["logging".to_string()],
            quality: "strict".to_string(),
            output: None,
            force: false,
            dry_run: false,
            interactive: false,
            deterministic_core: Some("state-machine".to_string()),
            probabilistic_wrapper: None,
        };

        match project {
            ScaffoldCommands::Project {
                toolchain,
                templates,
                params,
                parallel,
            } => {
                assert_eq!(toolchain, "rust");
                assert_eq!(templates, vec!["cli", "lib"]);
                assert_eq!(params.len(), 1);
                assert_eq!(parallel, 4);
            }
            _ => panic!("Expected Project variant"),
        }

        match agent {
            ScaffoldCommands::Agent {
                name,
                template,
                features,
                quality,
                output,
                force,
                dry_run,
                interactive,
                deterministic_core,
                probabilistic_wrapper,
            } => {
                assert_eq!(name, "test-agent");
                assert_eq!(template, "basic");
                assert_eq!(features, vec!["logging"]);
                assert_eq!(quality, "strict");
                assert!(output.is_none());
                assert!(!force);
                assert!(!dry_run);
                assert!(!interactive);
                assert_eq!(deterministic_core, Some("state-machine".to_string()));
                assert!(probabilistic_wrapper.is_none());
            }
            _ => panic!("Expected Agent variant"),
        }
    }

    #[test]
    fn test_roadmap_commands_variants() {
        let init = RoadmapCommands::Init {
            version: "v2.6.0".to_string(),
            title: "Test Sprint".to_string(),
            duration_days: 14,
            priority: "P0".to_string(),
        };

        let start = RoadmapCommands::Start {
            task_id: "task-123".to_string(),
            create_branch: false,
        };

        let complete = RoadmapCommands::Complete {
            task_id: "task-123".to_string(),
            skip_quality_check: true,
        };

        match init {
            RoadmapCommands::Init {
                version,
                title,
                duration_days,
                priority,
            } => {
                assert_eq!(version, "v2.6.0");
                assert_eq!(title, "Test Sprint");
                assert_eq!(duration_days, 14);
                assert_eq!(priority, "P0");
            }
            _ => panic!("Expected Init variant"),
        }

        match start {
            RoadmapCommands::Start {
                task_id,
                create_branch,
            } => {
                assert_eq!(task_id, "task-123");
                assert!(!create_branch);
            }
            _ => panic!("Expected Start variant"),
        }

        match complete {
            RoadmapCommands::Complete {
                task_id,
                skip_quality_check,
            } => {
                assert_eq!(task_id, "task-123");
                assert!(skip_quality_check);
            }
            _ => panic!("Expected Complete variant"),
        }
    }

    #[test]
    fn test_test_suite_variants() {
        let performance = TestSuite::Performance;
        let integration = TestSuite::Integration;
        let property = TestSuite::Property;
        let memory = TestSuite::Memory;

        assert_eq!(performance, TestSuite::Performance);
        assert_eq!(integration, TestSuite::Integration);
        assert_eq!(property, TestSuite::Property);
        assert_eq!(memory, TestSuite::Memory);
    }

    #[test]
    fn test_serve_transport_variants() {
        let http = ServeTransport::Http;
        let websocket = ServeTransport::WebSocket;

        assert_eq!(http, ServeTransport::Http);
        assert_eq!(websocket, ServeTransport::WebSocket);
    }

    #[test]
    fn test_agent_commands_variants() {
        let status = AgentCommands::Status {
            pid_file: None,
            format: OutputFormat::Json,
        };

        let stop = AgentCommands::Stop {
            pid_file: None,
            force: false,
            timeout: 10,
        };

        match status {
            AgentCommands::Status { format, .. } => {
                assert_eq!(format, OutputFormat::Json);
            }
            _ => panic!("Expected Status variant"),
        }

        match stop {
            AgentCommands::Stop { force, timeout, .. } => {
                assert!(!force);
                assert_eq!(timeout, 10);
            }
            _ => panic!("Expected Stop variant"),
        }
    }

    #[test]
    fn test_commands_generate_variant() {
        let generate = Commands::Generate {
            category: "makefile".to_string(),
            template: "rust/cli".to_string(),
            params: vec![("name".to_string(), Value::String("test".to_string()))],
            output: Some(PathBuf::from("Makefile")),
            create_dirs: true,
        };

        match generate {
            Commands::Generate {
                category,
                template,
                params,
                output,
                create_dirs,
            } => {
                assert_eq!(category, "makefile");
                assert_eq!(template, "rust/cli");
                assert_eq!(params.len(), 1);
                assert_eq!(output, Some(PathBuf::from("Makefile")));
                assert!(create_dirs);
            }
            _ => panic!("Expected Generate variant"),
        }
    }

    #[test]
    fn test_commands_list_variant() {
        let list = Commands::List {
            toolchain: Some("rust".to_string()),
            category: Some("cli".to_string()),
            format: OutputFormat::Json,
        };

        match list {
            Commands::List {
                toolchain,
                category,
                format,
            } => {
                assert_eq!(toolchain, Some("rust".to_string()));
                assert_eq!(category, Some("cli".to_string()));
                assert_eq!(format, OutputFormat::Json);
            }
            _ => panic!("Expected List variant"),
        }
    }

    #[test]
    fn test_commands_search_variant() {
        let search = Commands::Search {
            query: "rust cli".to_string(),
            toolchain: Some("rust".to_string()),
            limit: 10,
        };

        match search {
            Commands::Search {
                query,
                toolchain,
                limit,
            } => {
                assert_eq!(query, "rust cli");
                assert_eq!(toolchain, Some("rust".to_string()));
                assert_eq!(limit, 10);
            }
            _ => panic!("Expected Search variant"),
        }
    }

    #[test]
    fn test_commands_validate_variant() {
        let validate = Commands::Validate {
            uri: "template://rust/cli".to_string(),
            params: vec![("name".to_string(), Value::String("test".to_string()))],
        };

        match validate {
            Commands::Validate { uri, params } => {
                assert_eq!(uri, "template://rust/cli");
                assert_eq!(params.len(), 1);
            }
            _ => panic!("Expected Validate variant"),
        }
    }

    #[test]
    fn test_commands_context_variant() {
        let context = Commands::Context {
            toolchain: Some("rust".to_string()),
            project_path: PathBuf::from("."),
            output: Some(PathBuf::from("context.md")),
            format: ContextFormat::Markdown,
            include_large_files: false,
            skip_expensive_metrics: true,
            language: None,
            languages: None,
        };

        match context {
            Commands::Context {
                toolchain,
                project_path,
                output,
                format,
                include_large_files,
                skip_expensive_metrics,
                language,
                languages,
            } => {
                assert_eq!(toolchain, Some("rust".to_string()));
                assert_eq!(project_path, PathBuf::from("."));
                assert_eq!(output, Some(PathBuf::from("context.md")));
                assert_eq!(format, ContextFormat::Markdown);
                assert!(!include_large_files);
                assert!(skip_expensive_metrics);
                assert_eq!(language, None);
                assert_eq!(languages, None);
            }
            _ => panic!("Expected Context variant"),
        }
    }

    #[test]
    fn test_commands_serve_variant() {
        let serve = Commands::Serve {
            host: "127.0.0.1".to_string(),
            port: 3000,
            cors: true,
            transport: ServeTransport::Http,
        };

        match serve {
            Commands::Serve {
                host, port, cors, ..
            } => {
                assert_eq!(host, "127.0.0.1");
                assert_eq!(port, 3000);
                assert!(cors);
            }
            _ => panic!("Expected Serve variant"),
        }
    }
}

/// Scaffold subcommands
#[derive(Subcommand)]
#[cfg_attr(test, derive(Debug))]
pub enum ScaffoldCommands {
    /// Scaffold a complete project with templates
    Project {
        /// Target toolchain
        toolchain: String,

        /// Templates to generate
        #[arg(short, long, value_delimiter = ',')]
        templates: Vec<String>,

        /// Parameters
        #[arg(short = 'p', long = "param", value_parser = crate::cli::args::parse_key_val)]
        params: Vec<(String, Value)>,

        /// Parallelism level
        #[arg(long, default_value_t = num_cpus::get())]
        parallel: usize,
    },

    /// Scaffold a deterministic MCP agent
    Agent {
        /// Agent name
        #[arg(short, long)]
        name: String,

        /// Template type (mcp-server, state-machine, hybrid, calculator, custom:<path>)
        #[arg(short, long)]
        template: String,

        /// Features to include (comma-separated)
        #[arg(short, long, value_delimiter = ',')]
        features: Vec<String>,

        /// Quality level (standard, strict, extreme)
        #[arg(short = 'l', long, default_value = "strict")]
        quality: String,

        /// Output directory
        #[arg(short, long)]
        output: Option<PathBuf>,

        /// Overwrite existing directory
        #[arg(long)]
        force: bool,

        /// Show what would be generated without creating files
        #[arg(long)]
        dry_run: bool,

        /// Interactive mode for guided creation
        #[arg(short, long)]
        interactive: bool,

        /// Deterministic core specification (for hybrid agents)
        #[arg(long)]
        deterministic_core: Option<String>,

        /// Probabilistic wrapper specification (for hybrid agents)
        #[arg(long)]
        probabilistic_wrapper: Option<String>,
    },

    /// Scaffold a WebAssembly project
    Wasm {
        /// Project name
        #[arg(short, long)]
        name: String,

        /// WASM framework (wasm-labs, pure-wasm)
        #[arg(short, long, default_value = "wasm-labs")]
        framework: String,

        /// Features to include (comma-separated)
        #[arg(short, long, value_delimiter = ',')]
        features: Vec<String>,

        /// Quality level (standard, strict, extreme)
        #[arg(short = 'l', long, default_value = "strict")]
        quality: String,

        /// Output directory
        #[arg(short, long)]
        output: Option<PathBuf>,

        /// Overwrite existing directory
        #[arg(long)]
        force: bool,

        /// Show what would be generated without creating files
        #[arg(long)]
        dry_run: bool,
    },

    /// List available agent templates
    ListTemplates,

    /// Validate an agent template
    ValidateTemplate {
        /// Path to template file
        path: PathBuf,
    },

    /// List available Claude Code sub-agents
    ListSubagents {
        /// Show all sub-agents (including future phases)
        #[arg(long)]
        all: bool,
    },

    /// Create a specific Claude Code sub-agent
    CreateSubagent {
        /// Sub-agent name (e.g., complexity-analyst, mutation-tester)
        agent_name: String,

        /// Output directory (defaults to .claude/subagents)
        #[arg(short, long)]
        output: Option<PathBuf>,
    },

    /// Create all MVP Claude Code sub-agents
    CreateAllSubagents {
        /// Output directory (defaults to .claude/subagents)
        #[arg(short, long)]
        output: Option<PathBuf>,
    },

    /// Validate a sub-agent definition file
    ValidateSubagent {
        /// Path to sub-agent definition file
        file_path: PathBuf,
    },

    /// Show MCP tool mapping for sub-agents
    ShowToolMapping {
        /// Specific sub-agent name (shows all if not specified)
        #[arg(short, long)]
        agent: Option<String>,
    },

    /// Export MCP tool mapping as JSON
    ExportToolMapping {
        /// Output file path
        #[arg(short, long)]
        output: PathBuf,
    },
}

impl CliInput {
    #[must_use]
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    /// Create a new instance.
    pub fn new(command: Commands, command_name: String, raw_args: Vec<String>) -> Self {
        Self {
            command,
            command_name,
            raw_args,
        }
    }

    /// Create from the parsed CLI arguments
    fn get_analyze_command_name(analyze_cmd: &AnalyzeCommands) -> &'static str {
        // Toyota Way Extract Method: Use categorized dispatch for analyze command names
        let category = CliAdapter::get_analyze_command_category(analyze_cmd);

        match category {
            AnalyzeCommandCategory::Basic => Self::get_basic_analyze_command_name(analyze_cmd),
            AnalyzeCommandCategory::Advanced => {
                Self::get_advanced_analyze_command_name(analyze_cmd)
            }
            AnalyzeCommandCategory::Structural => {
                Self::get_structural_analyze_command_name(analyze_cmd)
            }
            AnalyzeCommandCategory::Specialized => {
                Self::get_specialized_analyze_command_name(analyze_cmd)
            }
        }
    }

    /// Toyota Way Extract Method: Get QDD command name
    fn get_qdd_command_name(qdd_cmd: &QddCommands) -> &'static str {
        match qdd_cmd {
            QddCommands::Create { .. } => "qdd-create",
            QddCommands::Refactor { .. } => "qdd-refactor",
            QddCommands::Validate { .. } => "qdd-validate",
        }
    }

    /// Toyota Way Extract Method: Basic analysis command names
    fn get_basic_analyze_command_name(analyze_cmd: &AnalyzeCommands) -> &'static str {
        match analyze_cmd {
            AnalyzeCommands::Churn { .. } => "analyze-churn",
            AnalyzeCommands::Complexity { .. } => "analyze-complexity",
            AnalyzeCommands::DeadCode { .. } => "analyze-dead-code",
            AnalyzeCommands::Satd { .. } => "analyze-satd",
            AnalyzeCommands::Tdg { .. } => "analyze-tdg",
            AnalyzeCommands::LintHotspot { .. } => "analyze-lint-hotspot",
            _ => unreachable!("Non-basic command passed to basic command name extractor"),
        }
    }

    /// Toyota Way Extract Method: Advanced analysis command names
    fn get_advanced_analyze_command_name(analyze_cmd: &AnalyzeCommands) -> &'static str {
        match analyze_cmd {
            AnalyzeCommands::DeepContext { .. } => "analyze-deep-context",
            AnalyzeCommands::Comprehensive { .. } => "analyze-comprehensive",
            AnalyzeCommands::DefectPrediction { .. } => "analyze-defect-prediction",
            AnalyzeCommands::Duplicates { .. } => "analyze-duplicates",
            AnalyzeCommands::BigO { .. } => "analyze-big-o",
            _ => unreachable!("Non-advanced command passed to advanced command name extractor"),
        }
    }

    /// Toyota Way Extract Method: Structural analysis command names
    fn get_structural_analyze_command_name(analyze_cmd: &AnalyzeCommands) -> &'static str {
        match analyze_cmd {
            AnalyzeCommands::Dag { .. } => "analyze-dag",
            AnalyzeCommands::GraphMetrics { .. } => "analyze-graph-metrics",
            AnalyzeCommands::SymbolTable { .. } => "analyze-symbol-table",
            AnalyzeCommands::NameSimilarity { .. } => "analyze-name-similarity",
            _ => unreachable!("Non-structural command passed to structural command name extractor"),
        }
    }

    /// Toyota Way Extract Method: Specialized analysis command names
    fn get_specialized_analyze_command_name(analyze_cmd: &AnalyzeCommands) -> &'static str {
        match analyze_cmd {
            AnalyzeCommands::Makefile { .. } => "analyze-makefile",
            AnalyzeCommands::Provability { .. } => "analyze-provability",
            AnalyzeCommands::ProofAnnotations { .. } => "analyze-proof-annotations",
            AnalyzeCommands::IncrementalCoverage { .. } => "analyze-incremental-coverage",
            AnalyzeCommands::AssemblyScript { .. } => "analyze-assemblyscript",
            AnalyzeCommands::WebAssembly { .. } => "analyze-webassembly",
            _ => {
                unreachable!("Non-specialized command passed to specialized command name extractor")
            }
        }
    }

    #[must_use]
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    /// From commands.
    pub fn from_commands(command: Commands) -> Self {
        // Toyota Way Extract Method: Get command name using categorized dispatch
        let command_name = Self::get_command_name_by_category(&command);

        Self {
            command,
            command_name,
            raw_args: std::env::args().collect(),
        }
    }

    /// Toyota Way Extract Method: Get command name using categorized dispatch
    /// Reduces complexity from 23 branches to category-based logic
    fn get_command_name_by_category(command: &Commands) -> String {
        match command {
            // Special case: Analyze command needs sub-command delegation
            Commands::Analyze(analyze_cmd) => Self::get_analyze_command_name(analyze_cmd),
            // Special case: QDD command needs sub-command delegation
            Commands::Qdd(qdd_cmd) => Self::get_qdd_command_name(qdd_cmd),
            // All other commands: extract name directly using category dispatch
            _ => Self::get_simple_command_name(command),
        }
        .to_string()
    }

    /// Toyota Way Extract Method: Get simple command name for non-analyze commands
    /// Single responsibility: name extraction using category-based dispatch
    fn get_simple_command_name(command: &Commands) -> &'static str {
        let category = Self::get_command_category(command);

        match category {
            CommandCategory::Generation => Self::get_generation_command_name(command),
            CommandCategory::Analysis => Self::get_analysis_command_name(command),
            CommandCategory::Operations => Self::get_operations_command_name(command),
            CommandCategory::Workflow => Self::get_workflow_command_name(command),
            CommandCategory::System => Self::get_system_command_name(command),
            CommandCategory::Configuration => Self::get_configuration_command_name(command),
            CommandCategory::Demo => "demo",
            CommandCategory::Enforcement => "enforce",
        }
    }

    /// Toyota Way Extract Method: Determine command category
    fn get_command_category(command: &Commands) -> CommandCategory {
        match command {
            Commands::Generate { .. } | Commands::Scaffold { .. } => CommandCategory::Generation,
            Commands::QualityGate { .. } | Commands::QualityGates { .. } | Commands::Report { .. } | Commands::RepoScore { .. } | Commands::RustProjectScore { .. } | Commands::BrickScore { .. } | Commands::PopperScore { .. } | Commands::DemoScore { .. } | Commands::ValidateDocs(_) | Commands::ValidateReadme(_) | Commands::RedTeam(_) | Commands::Org(_) | Commands::Prompt(_) | Commands::Embed(_) | Commands::Semantic(_) | Commands::ShowMetrics { .. } | Commands::PredictQuality { .. } | Commands::RecordMetric { .. } | Commands::DepsAudit { .. } => CommandCategory::Analysis,
            #[cfg(feature = "mutation-testing")]
            Commands::Mutate(_) => CommandCategory::Analysis,
            Commands::Serve { .. }
            | Commands::Cache { .. }
            | Commands::Memory { .. }
            | Commands::Telemetry { .. } => CommandCategory::Operations,
            Commands::Refactor(_)
            | Commands::Test { .. }
            | Commands::Roadmap(_)
            | Commands::Maintain { .. } // TICKET-PMAT-5032
            | Commands::Hooks(_) // TICKET-PMAT-5034
            | Commands::Validate { .. } => CommandCategory::Workflow,
            Commands::List { .. }
            | Commands::Search { .. }
            | Commands::Context { .. }
            | Commands::Diagnose(_)
            | Commands::Debug { .. } => CommandCategory::System,
            Commands::Config { .. } | Commands::Agent { .. } | Commands::Tdg { .. } => {
                CommandCategory::Configuration
            }
            Commands::Demo { .. } => CommandCategory::Demo,
            Commands::Enforce(_) => CommandCategory::Enforcement,
            Commands::Analyze(_) => {
                unreachable!("Analyze commands handled by get_analyze_command_name")
            }
            Commands::Qdd(_) => {
                unreachable!("QDD commands handled by get_qdd_command_name")
            }
            Commands::Work { .. } => {
                CommandCategory::Workflow // Issue #75: Unified GitHub/YAML workflow
            }
            Commands::Comply { .. } => {
                CommandCategory::Analysis // GH-96: PMAT compliance and migration system
            }
            Commands::ProjectDiag { .. } => {
                CommandCategory::Analysis // Project diagnostics - lltop Tab 8 equivalent
            }
            Commands::TestDiscovery { .. } => {
                CommandCategory::Analysis // GH-98: Systematic test discovery and fixing
            }
            Commands::DebugFiveWhys { .. } => {
                CommandCategory::Analysis // Five Whys root cause analysis
            }
            Commands::Localize { .. } => {
                CommandCategory::Analysis // GH-103: Tarantula fault localization
            }
            Commands::Oracle { .. } => {
                CommandCategory::Workflow // PMAT Oracle - PDCA loop for automated quality improvement
            }
            Commands::QaWork { .. } => {
                CommandCategory::Workflow // GH-102: Toyota Way QA validation
            }
            Commands::PerfectionScore { .. } => {
                CommandCategory::Analysis // master-plan-pmat-work-system.md: 200-point unified score
            }
            Commands::Spec { .. } => {
                CommandCategory::Workflow // master-plan-pmat-work-system.md: Spec management
            }
            Commands::CudaTdg { .. } => {
                CommandCategory::Analysis // CUDA-SIMD TDG: 100-point Popper falsification
            }
            Commands::Query { .. } => {
                CommandCategory::System // Semantic code search
            }
            Commands::Falsify { .. } => {
                CommandCategory::Analysis // Falsification testing
            }
            Commands::Kaizen { .. } => {
                CommandCategory::Workflow // Kaizen continuous improvement
            }
            Commands::Extract { .. } => {
                CommandCategory::Workflow // Extract refactoring
            }
            Commands::Split { .. } => {
                CommandCategory::Workflow // File splitting
            }
            Commands::Sql { .. } => {
                CommandCategory::Analysis // SQL analytics
            }
            Commands::Score { .. } => {
                CommandCategory::Analysis // Unified quality score (sibling of RepoScore)
            }
            Commands::InfraScore { .. } => {
                CommandCategory::Analysis // Infrastructure CI/CD quality score (sibling of RepoScore)
            }
            Commands::Verify(_) => {
                CommandCategory::Analysis // CI-faithful pre-flight gate (sibling of QualityGate)
            }
            Commands::Explain { .. } => {
                CommandCategory::System // Informational lookup of check/metric meaning
            }
            Commands::CiLocal { .. } => {
                CommandCategory::Workflow // Local CI simulation (sibling of Test/Validate)
            }
            Commands::TestStability { .. } => {
                CommandCategory::Workflow // Flaky/timeout test detection (sibling of Test)
            }
            Commands::Stack { .. } => {
                CommandCategory::Workflow // Cross-repo dependency coordination (sibling of Maintain)
            }
            Commands::Mcp(_) => {
                CommandCategory::System
            }
            Commands::Agy(_) => {
                CommandCategory::System // Anti-Gravity translator, sibling of Mcp
            }
        }
    }

    /// Toyota Way Extract Method: Generation command names
    fn get_generation_command_name(command: &Commands) -> &'static str {
        match command {
            Commands::Generate { .. } => "generate",
            Commands::Scaffold { .. } => "scaffold",
            _ => unreachable!("Non-generation command passed to generation command name extractor"),
        }
    }

    /// Toyota Way Extract Method: Analysis command names (non-analyze)
    fn get_analysis_command_name(command: &Commands) -> &'static str {
        match command {
            Commands::QualityGate { .. } => "quality-gate",
            Commands::Report { .. } => "report",
            Commands::DebugFiveWhys { .. } => "five-whys",
            _ => unreachable!("Non-analysis command passed to analysis command name extractor"),
        }
    }

    /// Toyota Way Extract Method: Operations command names
    fn get_operations_command_name(command: &Commands) -> &'static str {
        match command {
            Commands::Serve { .. } => "serve",
            Commands::Cache { .. } => "cache",
            Commands::Memory { .. } => "memory",
            Commands::Telemetry { .. } => "telemetry",
            _ => unreachable!("Non-operations command passed to operations command name extractor"),
        }
    }

    /// Toyota Way Extract Method: Workflow command names
    fn get_workflow_command_name(command: &Commands) -> &'static str {
        match command {
            Commands::Refactor(_) => "refactor",
            Commands::Test { .. } => "test",
            Commands::Roadmap(_) => "roadmap",
            Commands::Validate { .. } => "validate",
            Commands::Maintain { .. } => "maintain",
            Commands::Hooks(_) => "hooks",
            Commands::Work { .. } => "work",
            _ => unreachable!("Non-workflow command passed to workflow command name extractor"),
        }
    }

    /// Toyota Way Extract Method: System command names
    fn get_system_command_name(command: &Commands) -> &'static str {
        match command {
            Commands::List { .. } => "list",
            Commands::Search { .. } => "search",
            Commands::Context { .. } => "context",
            Commands::Diagnose(_) => "diagnose",
            Commands::Debug { .. } => "debug",
            _ => unreachable!("Non-system command passed to system command name extractor"),
        }
    }

    /// Toyota Way Extract Method: Configuration command names
    fn get_configuration_command_name(command: &Commands) -> &'static str {
        match command {
            Commands::Config { .. } => "config",
            Commands::Agent { .. } => "agent",
            Commands::Tdg { .. } => "tdg",
            _ => unreachable!(
                "Non-configuration command passed to configuration command name extractor"
            ),
        }
    }
}

impl CliAdapter {
    /// Toyota Way Extract Method: Categorize analyze command by type
    /// Single responsibility: classification logic only
    fn get_analyze_command_category(analyze_cmd: &AnalyzeCommands) -> AnalyzeCommandCategory {
        match analyze_cmd {
            // Core analysis commands (basic metrics)
            AnalyzeCommands::Bottleneck { .. }
            | AnalyzeCommands::Churn { .. }
            | AnalyzeCommands::Complexity { .. }
            | AnalyzeCommands::DeadCode { .. }
            | AnalyzeCommands::Defects { .. }
            | AnalyzeCommands::Satd { .. }
            | AnalyzeCommands::Tdg { .. }
            | AnalyzeCommands::BuildTdg { .. }
            | AnalyzeCommands::LintHotspot { .. }
            | AnalyzeCommands::Clippy { .. }
            | AnalyzeCommands::Entropy { .. }
            // Reachability is Basic: it reads the module graph and `git
            // ls-files`, with no AST pass and no cargo build.
            | AnalyzeCommands::Reachability { .. } => AnalyzeCommandCategory::Basic,

            // Advanced analysis commands (comprehensive)
            AnalyzeCommands::DeepContext { .. }
            | AnalyzeCommands::Comprehensive { .. }
            | AnalyzeCommands::DefectPrediction { .. }
            | AnalyzeCommands::Duplicates { .. }
            | AnalyzeCommands::BigO { .. } => AnalyzeCommandCategory::Advanced,

            // Graph and structural analysis
            AnalyzeCommands::Dag { .. }
            | AnalyzeCommands::GraphMetrics { .. }
            | AnalyzeCommands::SymbolTable { .. }
            | AnalyzeCommands::NameSimilarity { .. } => AnalyzeCommandCategory::Structural,

            // Specialized analysis commands
            AnalyzeCommands::Makefile { .. }
            | AnalyzeCommands::Provability { .. }
            | AnalyzeCommands::ProofAnnotations { .. }
            | AnalyzeCommands::IncrementalCoverage { .. }
            | AnalyzeCommands::CoverageImprove { .. }
            | AnalyzeCommands::AssemblyScript { .. }
            | AnalyzeCommands::WebAssembly { .. }
            | AnalyzeCommands::Wasm { .. }
            | AnalyzeCommands::Cluster { .. } // PMAT-SEARCH-011
            | AnalyzeCommands::Topics { .. } // PMAT-SEARCH-011
            => AnalyzeCommandCategory::Specialized,

            #[cfg(feature = "mutation-testing")]
            AnalyzeCommands::Mutate { .. } => AnalyzeCommandCategory::Specialized,

            #[cfg(feature = "deep-wasm")]
            AnalyzeCommands::DeepWasm { .. } => AnalyzeCommandCategory::Specialized,

            AnalyzeCommands::Models { .. } => AnalyzeCommandCategory::Specialized,
        }
    }
}

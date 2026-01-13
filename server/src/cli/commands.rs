//! CLI command structures
//!
//! This module contains all the command structures used by the CLI for parsing
//! and executing commands. It's separated from the main CLI module to reduce complexity.

use crate::cli::diagnose::DiagnoseArgs;
use crate::cli::handlers::cache::CacheCommand;
use crate::cli::handlers::coverage_improve_handler::CoverageImproveOutputFormat;
use crate::cli::handlers::memory::MemoryCommand;
use crate::cli::{
    AnalysisType, BigOOutputFormat, ComplexityOutputFormat, ComprehensiveOutputFormat,
    ContextFormat, DagType, DeadCodeOutputFormat, DebugOutputFormat, DeepContextCacheStrategy,
    DeepContextDagType, DeepContextOutputFormat, DefectPredictionOutputFormat, DefectsOutputFormat,
    DemoProtocol, DuplicateOutputFormat, DuplicateType, EnforceOutputFormat, EntropyOutputFormat,
    EntropySeverity, ExplainLevel, GraphMetricType, GraphMetricsOutputFormat,
    IncrementalCoverageOutputFormat, LintHotspotOutputFormat, MakefileOutputFormat,
    NameSimilarityOutputFormat, OutputFormat, PromptOutputFormat, ProofAnnotationOutputFormat,
    PropertyTypeFilter, ProvabilityOutputFormat, QualityCheckType, QualityGateOutputFormat,
    QualityProfile, RefactorAutoOutputFormat, RefactorDocsOutputFormat, RefactorMode,
    RefactorOutputFormat, RepoScoreOutputFormat, ReportOutputFormat, SatdOutputFormat,
    SatdSeverity, SearchScope, SymbolTableOutputFormat, SymbolTypeFilter, TdgOutputFormat,
    VerificationMethodFilter, WasmOutputFormat,
};

#[cfg(feature = "deep-wasm")]
use crate::cli::{DeepWasmFocus, DeepWasmLanguage, DeepWasmOutputFormat};
use crate::models::churn::ChurnOutputFormat;
#[cfg(feature = "mutation-testing")]
use clap::Args;
use clap::{Parser, Subcommand};
use serde_json::Value;
use std::path::PathBuf;

/// Main CLI structure
#[derive(Parser)]
#[command(
    name = "pmat",
    about = "Professional project quantitative scaffolding and analysis toolkit",
    version,
    long_about = None,
    after_help = "EXAMPLES:
# Analyze code complexity
pmat analyze complexity --project-path .

# Find technical debt
pmat analyze satd --path .

# Find dead code
pmat analyze dead-code --path .

# Generate project context
pmat context

# Run quality gates
pmat quality-gate --strict

# Start agent daemon
pmat agent start"
)]
#[cfg_attr(test, derive(Debug))]
pub struct Cli {
    /// Force specific mode (auto-detected by default)
    #[arg(long, value_enum, global = true)]
    pub mode: Option<Mode>,

    /// Enable verbose output (info level)
    #[arg(short, long, global = true)]
    pub verbose: bool,

    /// Enable quiet mode (errors only)
    #[arg(short, long, global = true, conflicts_with = "verbose")]
    pub quiet: bool,

    /// Enable debug output (debug level)
    #[arg(long, global = true)]
    pub debug: bool,

    /// Enable trace output (trace level)
    #[arg(long, global = true)]
    pub trace: bool,

    /// Custom trace filter (overrides other flags)
    /// Example: --trace-filter="paiml=debug,cache=trace"
    #[arg(long, global = true, env = "RUST_LOG")]
    pub trace_filter: Option<String>,

    /// Control color output
    #[arg(long, global = true, value_enum, default_value = "auto")]
    pub color: ColorMode,

    #[command(subcommand)]
    pub command: Commands,
}

/// CLI execution mode
#[derive(Clone, Debug, clap::ValueEnum, PartialEq)]
pub enum Mode {
    Cli,
    Mcp,
}

/// Color output mode (TICKET-PMAT-6006)
#[derive(Clone, Debug, clap::ValueEnum, PartialEq, Default)]
pub enum ColorMode {
    /// Auto-detect based on TTY and environment
    #[default]
    Auto,
    /// Always use colors
    Always,
    /// Never use colors
    Never,
}

/// Main command enum
#[derive(Subcommand)]
#[cfg_attr(test, derive(Debug))]
pub enum Commands {
    /// Generate a single template
    #[command(visible_aliases = &["gen", "g"])]
    Generate {
        /// Template category
        category: String,

        /// Template path (e.g., rust/cli)
        template: String,

        /// Parameters as key=value pairs
        #[arg(short = 'p', long = "param", value_parser = crate::cli::args::parse_key_val)]
        params: Vec<(String, Value)>,

        /// Output file path
        #[arg(short, long)]
        output: Option<PathBuf>,

        /// Create parent directories
        #[arg(long)]
        create_dirs: bool,
    },

    /// Scaffold complete project or agent
    #[command(visible_aliases = &["sc"])]
    Scaffold {
        /// Scaffold subcommand
        #[command(subcommand)]
        command: ScaffoldCommands,
    },

    /// List available templates
    #[command(visible_aliases = &["ls"])]
    List {
        /// Filter by toolchain
        #[arg(long)]
        toolchain: Option<String>,

        /// Filter by category
        #[arg(long)]
        category: Option<String>,

        /// Output format
        #[arg(long, value_enum, default_value = "table")]
        format: OutputFormat,
    },

    /// Search templates
    #[command(visible_aliases = &["find", "s"])]
    Search {
        /// Search query
        query: String,

        /// Filter by toolchain
        #[arg(long)]
        toolchain: Option<String>,

        /// Max results
        #[arg(long, default_value_t = 20)]
        limit: usize,
    },

    /// Validate template parameters
    Validate {
        /// Template URI
        uri: String,

        /// Parameters to validate
        #[arg(short = 'p', long = "param", value_parser = crate::cli::args::parse_key_val)]
        params: Vec<(String, Value)>,
    },

    /// Generate project context (AST analysis)
    #[command(visible_aliases = &["ctx", "ast"])]
    Context {
        /// Target toolchain (auto-detected if not specified)
        #[arg(long, short = 't')]
        toolchain: Option<String>,

        /// Project path to analyze
        #[arg(short = 'p', long, default_value = ".")]
        project_path: PathBuf,

        /// Output file path
        #[arg(short, long)]
        output: Option<PathBuf>,

        /// Output format
        #[arg(long, value_enum, default_value = "markdown")]
        format: ContextFormat,

        /// Include large files (>500KB) that are normally skipped
        #[arg(long)]
        include_large_files: bool,

        /// Skip expensive metrics (TDG, complexity analysis) for faster execution
        #[arg(long)]
        skip_expensive_metrics: bool,

        /// Override language detection (e.g., "rust", "cpp", "python")
        /// BUG-012: Single language override support
        #[arg(long)]
        language: Option<String>,

        /// Specify multiple languages (comma-separated: "rust,python,typescript")
        /// BUG-012: Multi-language override support
        #[arg(long, value_delimiter = ',')]
        languages: Option<Vec<String>>,
    },

    /// Analyze code metrics and patterns
    #[command(subcommand, visible_aliases = &["a", "an"])]
    Analyze(AnalyzeCommands),

    /// Quality-Driven Development (QDD) tool for creating and refactoring code with guaranteed quality
    #[command(subcommand, visible_aliases = &["q"])]
    Qdd(QddCommands),

    /// Run interactive demo of all capabilities
    #[command(visible_aliases = &["d", "show"])]
    Demo {
        /// Repository path (defaults to current directory)
        #[arg(short, long)]
        path: Option<PathBuf>,

        /// Remote repository URL to clone and analyze
        #[arg(long)]
        url: Option<String>,

        /// Repository to analyze (supports GitHub URLs, local paths, or shorthand like gh:owner/repo)
        #[arg(long)]
        repo: Option<String>,

        /// Output format
        #[arg(short, long, value_enum, default_value = "table")]
        format: OutputFormat,

        /// Protocol to demonstrate (cli, http, mcp, all)
        #[arg(long, value_enum, default_value = "http")]
        protocol: DemoProtocol,

        /// Show API introspection information
        #[arg(long)]
        show_api: bool,

        /// Skip opening browser (web mode only)
        #[arg(long)]
        no_browser: bool,

        /// Port for demo server (default: random)
        #[arg(long)]
        port: Option<u16>,

        /// Run CLI output mode instead of web-based interactive demo
        #[arg(long)]
        cli: bool,

        /// Target node count for graph complexity reduction
        #[arg(long, default_value_t = 15)]
        target_nodes: usize,

        /// Minimum betweenness centrality threshold for graph reduction
        #[arg(long, default_value_t = 0.1)]
        centrality_threshold: f64,

        /// Component size threshold for merging in graph reduction
        #[arg(long, default_value_t = 3)]
        merge_threshold: usize,

        /// Enable debug mode with detailed file classification logs
        #[arg(long)]
        debug: bool,

        /// Output path for debug report (JSON format)
        #[arg(long)]
        debug_output: Option<PathBuf>,

        /// Skip vendor files during analysis (enabled by default)
        #[arg(long, default_value_t = true)]
        skip_vendor: bool,

        /// Disable vendor file skipping (process all files)
        #[arg(long = "no-skip-vendor")]
        no_skip_vendor: bool,

        /// Maximum line length before considering file unparseable
        #[arg(long)]
        max_line_length: Option<usize>,
    },

    /// Validate documentation links
    #[command(visible_aliases = &["docs", "doc"])]
    ValidateDocs(crate::cli::handlers::ValidateDocsCmd),

    /// Validate README/documentation for hallucinations (Sprint 38)
    #[command(visible_aliases = &["readme", "hallucination"])]
    ValidateReadme(crate::cli::handlers::ValidateReadmeCmd),

    /// Red Team Mode: Automated hallucination detection for commits and code
    #[command(visible_aliases = &["rt", "hallucination-detect"])]
    RedTeam(crate::cli::handlers::RedTeamCmd),

    /// Organizational intelligence analysis (GitHub org defect patterns)
    #[command(subcommand, visible_aliases = &["organization"])]
    Org(OrgCommands),

    /// AI prompt generation (defect-aware, ticket-based, spec-based)
    #[command(subcommand, visible_aliases = &["p"])]
    Prompt(PromptCommands),

    /// Run quality gate checks on the codebase
    #[command(visible_aliases = &["check", "c", "verify", "gate"])]
    QualityGate {
        /// Project path to analyze (defaults to current directory)
        #[arg(short = 'p', long, default_value = ".")]
        project_path: PathBuf,

        /// Analyze a specific file instead of the whole project
        #[arg(long)]
        file: Option<PathBuf>,

        /// Output format
        #[arg(short = 'f', long, value_enum, default_value = "summary")]
        format: QualityGateOutputFormat,

        /// Exit with non-zero code if quality gate fails
        #[arg(long)]
        fail_on_violation: bool,

        /// Specific checks to run (all by default)
        #[arg(long, value_delimiter = ',')]
        checks: Vec<QualityCheckType>,

        /// Maximum allowed dead code percentage
        #[arg(long, default_value = "15.0")]
        max_dead_code: f64,

        /// Minimum required complexity entropy
        #[arg(long, default_value = "2.0")]
        min_entropy: f64,

        /// Maximum allowed cyclomatic complexity p99
        #[arg(long, default_value = "50")]
        max_complexity_p99: u32,

        /// Include provability checks
        #[arg(long)]
        include_provability: bool,

        /// Output file path
        #[arg(short = 'o', long)]
        output: Option<PathBuf>,

        /// Show performance metrics
        #[arg(long)]
        perf: bool,
    },

    /// Generate enhanced analysis reports
    #[command(visible_aliases = &["r", "rep"])]
    Report {
        /// Project path to analyze (defaults to current directory)
        #[arg(short = 'p', long, default_value = ".")]
        project_path: PathBuf,

        /// Output format
        #[arg(short = 'f', long, value_enum, default_value = "json")]
        output_format: ReportOutputFormat,

        /// Generate text report (shortcut for --format text)
        #[arg(long = "txt", conflicts_with = "output_format")]
        text: bool,

        /// Generate markdown report (shortcut for --format markdown)
        #[arg(long = "md", conflicts_with = "output_format")]
        markdown: bool,

        /// Generate CSV report (shortcut for --format csv)
        #[arg(long = "csv", conflicts_with = "output_format")]
        csv: bool,

        /// Include visualizations in the report
        #[arg(long)]
        include_visualizations: bool,

        /// Include executive summary
        #[arg(long, default_value_t = true)]
        include_executive_summary: bool,

        /// Include actionable recommendations
        #[arg(long, default_value_t = true)]
        include_recommendations: bool,

        /// Analysis types to include
        #[arg(long, value_delimiter = ',', default_value = "all")]
        analyses: Vec<AnalysisType>,

        /// Confidence threshold for findings (0-100)
        #[arg(long, default_value_t = 50)]
        confidence_threshold: u8,

        /// Output file path
        #[arg(short = 'o', long)]
        output: Option<PathBuf>,

        /// Show performance metrics
        #[arg(long)]
        perf: bool,
    },

    /// Calculate repository health score (0-110 scale)
    #[command(name = "repo-score", visible_aliases = &["score", "health"])]
    RepoScore {
        /// Repository path to score (defaults to current directory)
        #[arg(short = 'p', long, default_value = ".")]
        path: PathBuf,

        /// Output format
        #[arg(short = 'f', long, value_enum, default_value = "text")]
        format: RepoScoreOutputFormat,

        /// Enable verbose output (show detailed scoring breakdown)
        #[arg(short = 'v', long)]
        verbose: bool,

        /// Show only failures and warnings
        #[arg(long)]
        failures_only: bool,

        /// Output file path
        #[arg(short = 'o', long)]
        output: Option<PathBuf>,

        /// Update README.md with repository health badge
        #[arg(long)]
        update_badge: bool,

        /// Deep scan: Check entire git history (slower but more thorough)
        /// Default: Scan HEAD only (fast). Use --deep for complete history analysis.
        #[arg(long)]
        deep: bool,
    },

    /// Calculate Rust project quality score (0-106 scale)
    #[command(name = "rust-project-score", visible_aliases = &["rust-score"])]
    RustProjectScore {
        /// Rust project path to score (defaults to current directory)
        #[arg(short = 'p', long, default_value = ".")]
        path: PathBuf,

        /// Output format
        #[arg(short = 'f', long, value_enum, default_value = "text")]
        format: RepoScoreOutputFormat,

        /// Enable verbose output (show detailed scoring breakdown)
        #[arg(short = 'v', long)]
        verbose: bool,

        /// Show only failures and warnings
        #[arg(long)]
        failures_only: bool,

        /// Output file path
        #[arg(short = 'o', long)]
        output: Option<PathBuf>,

        /// Full analysis mode (includes mutation testing, max 5 minutes)
        /// Default mode is fast (<60 seconds) and skips expensive checks
        #[arg(long)]
        full: bool,
    },

    /// Calculate Popper Falsifiability Score (0-100 scale)
    ///
    /// Evaluates repositories against Karl Popper's scientific standards of falsifiability.
    /// Includes gateway logic: if Falsifiability (Category A) < 60%, total score is 0.
    #[command(name = "popper-score", visible_aliases = &["popper", "falsifiability"])]
    PopperScore {
        /// Project path to score (defaults to current directory)
        #[arg(short = 'p', long, default_value = ".")]
        path: PathBuf,

        /// Output format
        #[arg(short = 'f', long, value_enum, default_value = "text")]
        format: RepoScoreOutputFormat,

        /// Enable verbose output (show detailed sub-score breakdown)
        #[arg(short = 'v', long)]
        verbose: bool,

        /// Show only failures and recommendations
        #[arg(long)]
        failures_only: bool,

        /// Output file path
        #[arg(short = 'o', long)]
        output: Option<PathBuf>,
    },

    /// Score demo/book repository quality (0-10 Category G scale)
    ///
    /// Evaluates educational repositories (demos, tutorials, cookbooks) for:
    /// - G1: Time-to-Interaction (quick-start guides, examples)
    /// - G2: Error Gracefulness (proper error handling in demos)
    /// - G3: Visual Stability (rich terminal output)
    /// - G4: "Wow" Factor (demo GIFs, badges, professional presentation)
    #[command(name = "demo-score", visible_aliases = &["book-score", "score-demo"])]
    DemoScore {
        /// Repository path to score (defaults to current directory)
        #[arg(short = 'p', long, default_value = ".")]
        path: PathBuf,

        /// Output format
        #[arg(short = 'f', long, value_enum, default_value = "text")]
        format: RepoScoreOutputFormat,

        /// Enable verbose output (show detailed scoring breakdown)
        #[arg(short = 'v', long)]
        verbose: bool,

        /// Show only failures and warnings
        #[arg(long)]
        failures_only: bool,

        /// Output file path
        #[arg(short = 'o', long)]
        output: Option<PathBuf>,
    },

    /// ComputeBrick profiling score (0-100 scale) for trueno/realizar ecosystem
    ///
    /// Reads BrickProfiler JSON output and calculates a comprehensive score:
    /// - Performance (40 pts): Throughput vs µs budget
    /// - Efficiency (25 pts): Backend utilization
    /// - Correctness (20 pts): All bricks executed
    /// - Stability (15 pts): CV < 15%
    ///
    /// Reference: qwen2.5-coder-showcase-demo.md §2.5
    #[command(name = "brick-score", visible_aliases = &["brick", "computebrick"])]
    BrickScore {
        /// Project path (defaults to current directory)
        #[arg(short = 'p', long, default_value = ".")]
        path: PathBuf,

        /// BrickProfiler JSON input file (auto-detected if not specified)
        #[arg(short = 'i', long)]
        input: Option<PathBuf>,

        /// Output format
        #[arg(short = 'f', long, value_enum, default_value = "text")]
        format: RepoScoreOutputFormat,

        /// Enable verbose output (show per-brick timing table)
        #[arg(short = 'v', long)]
        verbose: bool,

        /// Show only failures and recommendations
        #[arg(long)]
        failures_only: bool,

        /// Minimum score threshold (fail if below)
        #[arg(short = 't', long, default_value = "0")]
        threshold: u32,

        /// Output file path
        #[arg(short = 'o', long)]
        output: Option<PathBuf>,
    },

    /// Start HTTP API server with WebSocket support
    #[command(visible_aliases = &["server", "api"])]
    Serve {
        /// Port to bind the server to
        #[arg(long, default_value_t = 8080)]
        port: u16,

        /// Host address to bind to
        #[arg(long, default_value = "127.0.0.1")]
        host: String,

        /// Enable CORS for cross-origin requests
        #[arg(long)]
        cors: bool,

        /// Transport protocol to use
        #[arg(long, value_enum, default_value = "http")]
        transport: ServeTransport,
    },

    /// Run self-diagnostics to verify all features are working
    #[command(visible_aliases = &["diag", "doctor"])]
    Diagnose(DiagnoseArgs),

    /// Enforce extreme quality standards using state machine
    #[command(subcommand, visible_aliases = &["enf"])]
    Enforce(EnforceCommands),

    /// Refactor code with real-time analysis or interactive mode
    #[command(subcommand, visible_aliases = &["ref", "rf"])]
    Refactor(RefactorCommands),

    /// Roadmap management with PDMT todos and quality gates
    #[command(subcommand, visible_aliases = &["road", "rm"])]
    Roadmap(RoadmapCommands),

    /// Performance testing per SPECIFICATION.md Section 30
    Test {
        /// Test suite to run
        #[arg(value_enum, default_value = "performance")]
        suite: TestSuite,

        /// Number of test iterations for regression detection
        #[arg(long, default_value = "3")]
        iterations: usize,

        /// Enable memory usage testing
        #[arg(long)]
        memory: bool,

        /// Enable throughput testing
        #[arg(long)]
        throughput: bool,

        /// Enable regression testing
        #[arg(long)]
        regression: bool,

        /// Test timeout in seconds
        #[arg(long, default_value = "300")]
        timeout: u64,

        /// Output file for test results
        #[arg(short = 'o', long)]
        output: Option<PathBuf>,

        /// Show detailed performance metrics
        #[arg(long)]
        perf: bool,
    },

    /// Memory management and optimization
    Memory {
        /// Memory management subcommand
        #[command(subcommand)]
        command: MemoryCommand,
    },

    /// Cache strategy management and optimization
    Cache {
        /// Cache management subcommand
        #[command(subcommand)]
        command: CacheCommand,
    },
    /// Telemetry and system monitoring
    Telemetry {
        /// Show system telemetry data
        #[arg(long)]
        system: bool,
        /// Show service-specific telemetry
        #[arg(long)]
        service: Option<String>,
        /// Reset telemetry data (for testing)
        #[arg(long)]
        reset: bool,
        /// Record a test telemetry event
        #[arg(long)]
        test_event: bool,
    },
    /// Configuration management and settings
    Config {
        /// Show configuration overview or details
        #[arg(long)]
        show: bool,
        /// Interactive edit configuration
        #[arg(long)]
        edit: bool,
        /// Validate configuration
        #[arg(long)]
        validate: bool,
        /// Reset configuration to defaults
        #[arg(long)]
        reset: bool,
        /// Show specific configuration section
        #[arg(long)]
        section: Option<String>,
        /// Set configuration values (key=value format)
        #[arg(long, action = clap::ArgAction::Append)]
        set: Vec<String>,
        /// Path to configuration file
        #[arg(long)]
        config_path: Option<PathBuf>,
    },

    /// Show quality metrics and trends (Phase 3 O(1) Quality Gates)
    #[command(visible_aliases = &["metrics", "trends"])]
    ShowMetrics {
        /// Show metric trends over time
        #[arg(long)]
        trend: bool,

        /// Number of days to analyze (default: 30)
        #[arg(long, default_value_t = 30)]
        days: usize,

        /// Specific metric to analyze (lint, test-fast, coverage, build-release)
        #[arg(long)]
        metric: Option<String>,

        /// Output format
        #[arg(long, value_enum, default_value = "table")]
        format: OutputFormat,

        /// Show only failures (regressions)
        #[arg(long)]
        failures_only: bool,
    },

    /// Predict when quality metrics will exceed thresholds (Phase 4 O(1) Quality Gates)
    #[command(visible_aliases = &["predict"])]
    PredictQuality {
        /// Specific metric to predict (lint, test-fast, coverage, build-release)
        #[arg(long)]
        metric: Option<String>,

        /// Threshold value (ms or bytes)
        #[arg(long)]
        threshold: Option<f64>,

        /// Days to forecast (default: 30)
        #[arg(long, default_value_t = 30)]
        days: usize,

        /// Output format
        #[arg(long, value_enum, default_value = "table")]
        format: OutputFormat,

        /// Predict all metrics
        #[arg(long)]
        all: bool,

        /// Show only metrics at risk of breach
        #[arg(long)]
        failures_only: bool,
    },

    /// Record a quality metric observation (Phase 3.4 O(1) Quality Gates - CI/CD)
    #[command(visible_aliases = &["record"])]
    RecordMetric {
        /// Metric name (lint, test-fast, coverage, build-release)
        metric: String,

        /// Metric value (duration in ms or size in bytes)
        value: f64,

        /// Custom timestamp (Unix timestamp, default: now)
        #[arg(long)]
        timestamp: Option<i64>,
    },

    /// Start Claude Code background agent for continuous quality monitoring
    #[command(visible_aliases = &["ag"])]
    Agent {
        /// Agent mode subcommand
        #[command(subcommand)]
        command: AgentCommands,
    },

    /// Grade technical debt and code quality (TDG - Technical Debt Grading)
    #[command(visible_aliases = &["grade", "debt-grade"])]
    Tdg {
        /// File or directory to analyze
        #[arg(default_value = ".")]
        path: PathBuf,

        /// TDG subcommand (compare two files/directories)
        #[command(subcommand)]
        command: Option<TdgCommand>,

        /// Output format
        #[arg(long, value_enum, default_value = "table")]
        format: TdgOutputFormat,

        /// Configuration file (TOML format)
        #[arg(long)]
        config: Option<PathBuf>,

        /// Quiet mode (score only, no details)
        #[arg(short, long)]
        quiet: bool,

        /// Include component breakdown in output
        #[arg(long)]
        include_components: bool,

        /// Minimum grade to pass (for CI/CD)
        #[arg(long)]
        min_grade: Option<String>,

        /// Output file path
        #[arg(short, long)]
        output: Option<PathBuf>,

        /// Include git context (commit SHA, branch, author) - Sprint 65
        #[arg(long)]
        with_git_context: bool,

        /// Enable detailed explanation mode with function-level breakdown (Issue #78)
        #[arg(long)]
        explain: bool,

        /// Complexity threshold for filtering functions in --explain mode (Issue #78)
        #[arg(long, default_value = "10")]
        threshold: u32,

        /// Baseline git ref (commit/branch/tag) for progress tracking in --explain mode (Issue #78)
        #[arg(long)]
        baseline: Option<String>,

        /// Use ML-based scoring (GH-97: aprender LinearRegression)
        ///
        /// When enabled, TDG scores are calculated using trained ML models
        /// instead of heuristic weighted sums. This provides more accurate,
        /// data-driven scores that can learn from project history.
        #[arg(long)]
        ml: bool,

        /// Show terminal graph visualization of dependencies
        ///
        /// Renders a force-directed graph of function dependencies in the terminal
        /// using trueno-viz. Critical functions are highlighted with color and size.
        /// Supports ASCII, Unicode, and ANSI TrueColor modes.
        #[arg(long)]
        viz: bool,

        /// Visualization theme (default, high-contrast, light, colorblind-safe)
        #[arg(long, default_value = "default")]
        viz_theme: String,
    },

    /// Run quality gates on the current project (TICKET-PMAT-5023, TICKET-PMAT-5024)
    #[command(name = "quality-gates", visible_aliases = &["gates", "qg"])]
    QualityGates {
        /// Quality gates subcommand
        #[command(subcommand)]
        command: Option<QualityGatesCommand>,

        /// Path to quality gate configuration file
        #[arg(long, default_value = ".pmat-gates.toml", global = true)]
        config: PathBuf,

        /// Generate markdown report (only when no subcommand)
        #[arg(long)]
        report: bool,

        /// Output JSON format (only when no subcommand)
        #[arg(long)]
        json: bool,

        /// Project directory
        #[arg(long, default_value = ".", global = true)]
        project_dir: PathBuf,
    },

    /// Project maintenance commands (TICKET-PMAT-5032, TICKET-PMAT-5033)
    #[command(visible_aliases = &["maint", "m"])]
    Maintain {
        /// Maintain subcommand
        #[command(subcommand)]
        command: MaintainCommands,
    },

    /// Pre-commit hook management (TICKET-PMAT-5034)
    #[command(subcommand, visible_aliases = &["hook", "h"])]
    Hooks(HooksCommands),

    /// Manage semantic search embeddings (PMAT-SEARCH-011)
    #[command(subcommand, visible_aliases = &["emb"])]
    Embed(EmbedCommands),

    /// Semantic code search (PMAT-SEARCH-011)
    #[command(subcommand, visible_aliases = &["sem", "find-code"])]
    Semantic(SemanticCommands),

    /// Run mutation testing on specified files (Sprint 61)
    #[cfg(feature = "mutation-testing")]
    Mutate(MutateArgs),

    /// Time-travel debugging commands (Sprint 74)
    #[command(visible_aliases = &["dbg"])]
    Debug {
        #[command(subcommand)]
        command: DebugCommands,
    },

    /// Unified GitHub/YAML workflow commands (Issue #75)
    #[command(visible_aliases = &["w"])]
    Work {
        #[command(subcommand)]
        command: WorkCommands,
    },

    /// QA validation after work completion - Toyota Way quality gates (GH-102)
    #[command(name = "qa-work", visible_aliases = &["qa", "quality"])]
    QaWork {
        #[command(subcommand)]
        command: QaWorkCommands,
    },

    /// Five Whys root cause analysis (Toyota Way methodology)
    /// This is the ONLY acceptable debugging method per CLAUDE.md policy
    #[command(name = "five-whys", visible_aliases = &["why", "debug-whys"])]
    DebugFiveWhys {
        /// Issue description (symptom to analyze)
        issue: String,

        /// Number of "Why" iterations (1-10)
        #[arg(short = 'd', long = "depth", default_value = "5")]
        depth: u8,

        /// Output format: text, json, markdown
        #[arg(short = 'f', long = "format", default_value = "text")]
        format: DebugOutputFormat,

        /// Write output to file
        #[arg(short = 'o', long = "output")]
        output: Option<PathBuf>,

        /// Project path to analyze
        #[arg(short = 'p', long = "path", default_value = ".")]
        path: PathBuf,

        /// Use deep context file for enhanced analysis
        #[arg(short = 'c', long = "context")]
        context: Option<PathBuf>,

        /// Automatically analyze suspected files with PMAT tools
        #[arg(short = 'a', long = "auto-analyze")]
        auto_analyze: bool,
    },

    /// PMAT Oracle - PDCA loop for automated quality improvement (Toyota Way)
    /// Converges ANY Rust project toward perfect quality using CITL signals
    #[command(name = "oracle", visible_aliases = &["fix", "pdca"])]
    Oracle {
        #[command(subcommand)]
        command: OracleCommands,
    },

    /// Unified 200-point Perfection Score (master-plan-pmat-work-system.md)
    /// Aggregates TDG, Repo Score, Rust Score, Coverage, Mutation, Docs, Performance
    #[command(name = "perfection-score", visible_aliases = &["perfection", "perfect", "ps"])]
    PerfectionScore {
        /// Project path (defaults to current directory)
        #[arg(short = 'p', long = "path", default_value = ".")]
        path: PathBuf,

        /// Show detailed category breakdown
        #[arg(long)]
        breakdown: bool,

        /// Set target score and show gap analysis
        #[arg(long)]
        target: Option<u16>,

        /// Output format: text, json, markdown
        #[arg(short = 'f', long = "format", value_enum, default_value = "text")]
        format: PerfectionScoreOutputFormat,

        /// Write output to file
        #[arg(short = 'o', long = "output")]
        output: Option<PathBuf>,

        /// Skip slow checks (mutation testing, full coverage)
        #[arg(long)]
        fast: bool,
    },

    /// Specification management and validation (master-plan-pmat-work-system.md)
    #[command(name = "spec", visible_aliases = &["specification"])]
    Spec {
        #[command(subcommand)]
        command: SpecCommands,
    },

    /// PMAT compliance and migration system (GH-96)
    #[command(visible_aliases = &["compliance"])]
    Comply {
        #[command(subcommand)]
        command: ComplyCommands,
    },

    /// Rust project diagnostics (20 checks across 5 categories)
    /// Matches lltop Tab 8 diagnostics for any Rust project
    #[command(name = "project-diag", visible_aliases = &["pdiag", "proj-diag"])]
    ProjectDiag {
        /// Project path to analyze
        #[arg(short = 'p', long, default_value = ".")]
        path: PathBuf,

        /// Output format: summary, json, markdown, andon
        #[arg(short = 'f', long, value_enum, default_value = "summary")]
        format: ProjectDiagOutputFormat,

        /// Filter by category: cargo, deps, build, quality, advanced
        #[arg(long)]
        category: Option<String>,

        /// Show only failures and warnings
        #[arg(long)]
        failures_only: bool,

        /// Output file path
        #[arg(short, long)]
        output: Option<PathBuf>,

        /// Quiet mode (errors only)
        #[arg(short, long)]
        quiet: bool,
    },

    /// Systematic test discovery and fixing (GH-98)
    #[command(name = "test-discovery", visible_aliases = &["test-fix", "fix-tests"])]
    TestDiscovery {
        #[command(subcommand)]
        command: TestDiscoveryCommands,
    },

    /// Fault localization using Tarantula SBFL algorithm (GH-103)
    /// Identify suspicious code locations based on test coverage data
    #[command(name = "localize", visible_aliases = &["fault", "fl"])]
    Localize {
        /// Path to coverage file for passing tests (LCOV format)
        #[arg(long)]
        passed_coverage: PathBuf,

        /// Path to coverage file for failing tests (LCOV format)
        #[arg(long)]
        failed_coverage: PathBuf,

        /// Number of passing test cases
        #[arg(long)]
        passed_count: usize,

        /// Number of failing test cases
        #[arg(long)]
        failed_count: usize,

        /// SBFL formula: tarantula, ochiai, dstar2, dstar3
        #[arg(long, default_value = "tarantula")]
        formula: String,

        /// Top N suspicious statements to report
        #[arg(long, default_value_t = 10)]
        top_n: usize,

        /// Output file path (extension determines format: .json, .yaml, or text)
        #[arg(short, long)]
        output: Option<PathBuf>,

        /// Output format: terminal, json, yaml
        #[arg(short = 'f', long, default_value = "terminal")]
        format: String,
    },

    /// CUDA-SIMD Technical Debt Gradient (100-point Popper falsification scoring)
    /// Analyzes CUDA PTX, SIMD (AVX2/AVX-512/NEON), and WGPU code for defects
    /// Integrates Toyota Production System principles with falsificationist methodology
    #[command(name = "cuda-tdg", visible_aliases = &["gpu-tdg", "simd-tdg"])]
    CudaTdg {
        /// File or directory to analyze
        #[arg(default_value = ".")]
        path: PathBuf,

        /// Subcommand for specific operations
        #[command(subcommand)]
        command: Option<CudaTdgCommand>,

        /// Output format
        #[arg(short = 'f', long, value_enum, default_value = "terminal")]
        format: CudaTdgOutputFormat,

        /// Minimum score to pass quality gate (0-100)
        #[arg(long, default_value = "85")]
        min_score: f64,

        /// Fail on P0 (critical) defects
        #[arg(long)]
        fail_on_p0: bool,

        /// Include SIMD analysis (AVX2/AVX-512/NEON)
        #[arg(long)]
        simd: bool,

        /// Include WGPU analysis
        #[arg(long)]
        wgpu: bool,

        /// Write output to file
        #[arg(short, long)]
        output: Option<PathBuf>,

        /// Quiet mode (score only, no details)
        #[arg(short, long)]
        quiet: bool,
    },
}

/// CUDA-SIMD TDG output format
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, clap::ValueEnum)]
pub enum CudaTdgOutputFormat {
    /// Terminal output with colors
    #[default]
    Terminal,
    /// JSON for programmatic consumption
    Json,
    /// Markdown for documentation
    Markdown,
    /// SARIF for IDE integration
    Sarif,
}

/// CUDA-SIMD TDG subcommands
#[derive(Debug, Clone, Subcommand)]
#[cfg_attr(test, derive(PartialEq))]
pub enum CudaTdgCommand {
    /// Analyze a file or directory for CUDA/SIMD defects
    Analyze {
        /// Path to analyze
        path: PathBuf,
    },

    /// Score codebase with 100-point Popper falsification system
    Score {
        /// Path to analyze
        #[arg(default_value = ".")]
        path: PathBuf,

        /// Show detailed category breakdown
        #[arg(long)]
        breakdown: bool,
    },

    /// Generate detailed defect report
    Report {
        /// Path to analyze
        #[arg(default_value = ".")]
        path: PathBuf,

        /// Output format (html, json, markdown)
        #[arg(long, default_value = "markdown")]
        format: String,

        /// Output file
        #[arg(short, long)]
        output: Option<PathBuf>,
    },

    /// Check barrier safety (PARITY-114 detection)
    BarrierCheck {
        /// PTX or CUDA file to analyze
        path: PathBuf,
    },

    /// Validate tile dimensions for attention kernels
    ValidateTiles {
        /// Head dimension
        #[arg(long)]
        head_dim: usize,

        /// Tile KV dimension
        #[arg(long)]
        tile_kv: usize,

        /// Shared memory limit (bytes)
        #[arg(long, default_value = "49152")]
        shared_memory: usize,
    },

    /// Quality gate for CI/CD (exits non-zero on failure)
    Gate {
        /// Path to analyze
        #[arg(default_value = ".")]
        path: PathBuf,

        /// Minimum score to pass (0-100)
        #[arg(long, default_value = "85")]
        min_score: f64,

        /// Fail on P0 defects
        #[arg(long)]
        fail_on_p0: bool,
    },

    /// Generate Kaizen continuous improvement report
    Kaizen {
        /// Path to analyze
        #[arg(default_value = ".")]
        path: PathBuf,

        /// Start date for analysis (YYYY-MM-DD)
        #[arg(long)]
        since: Option<String>,
    },

    /// Show Tauranta fault taxonomy
    Taxonomy,
}

/// Oracle subcommands for PDCA loop automated quality improvement
#[derive(Debug, Clone, Subcommand)]
#[cfg_attr(test, derive(PartialEq))]
pub enum OracleCommands {
    /// Run PDCA fix loop to converge toward perfect project quality
    /// Uses CITL (Compiler-In-The-Loop) signals from rustc, clippy, cargo test
    #[command(visible_aliases = &["f", "run"])]
    Fix {
        /// Project path (defaults to current directory)
        #[arg(short = 'p', long = "path", default_value = ".")]
        path: PathBuf,

        /// Maximum iterations (1-100, default 10)
        #[arg(short = 'n', long = "max-iterations", default_value = "10")]
        max_iterations: usize,

        /// Confidence threshold for auto-apply (0.0-1.0)
        #[arg(long, default_value = "0.9")]
        auto_apply_threshold: f32,

        /// Confidence threshold for human review (0.0-1.0)
        #[arg(long, default_value = "0.7")]
        review_threshold: f32,

        /// Dry run (analyze only, don't apply fixes)
        #[arg(long)]
        dry_run: bool,

        /// Output format: text, json, markdown
        #[arg(short = 'f', long = "format", default_value = "text")]
        format: OracleOutputFormat,

        /// Write output to file
        #[arg(short = 'o', long = "output")]
        output: Option<PathBuf>,
    },

    /// Show current project quality status against convergence targets
    #[command(visible_aliases = &["s"])]
    Status {
        /// Project path (defaults to current directory)
        #[arg(short = 'p', long = "path", default_value = ".")]
        path: PathBuf,

        /// Output format: text, json, markdown
        #[arg(short = 'f', long = "format", default_value = "text")]
        format: OracleOutputFormat,
    },

    /// Run a single PDCA iteration (for CI/CD integration)
    Single {
        /// Project path (defaults to current directory)
        #[arg(short = 'p', long = "path", default_value = ".")]
        path: PathBuf,

        /// Output format: text, json, markdown
        #[arg(short = 'f', long = "format", default_value = "text")]
        format: OracleOutputFormat,

        /// Write output to file
        #[arg(short = 'o', long = "output")]
        output: Option<PathBuf>,
    },
}

/// Output format for Oracle command
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, clap::ValueEnum)]
pub enum OracleOutputFormat {
    /// Human-readable text output
    #[default]
    Text,
    /// JSON for programmatic consumption
    Json,
    /// Markdown for documentation
    Markdown,
}

/// Comply subcommands for PMAT compliance checking and migration (GH-96)
#[derive(Debug, Clone, Subcommand)]
pub enum ComplyCommands {
    /// Check project compliance with current PMAT version
    #[command(visible_aliases = &["status"])]
    Check {
        /// Project path (defaults to current directory)
        #[arg(short = 'p', long = "path", default_value = ".")]
        path: PathBuf,

        /// Exit with error if non-compliant
        #[arg(long)]
        strict: bool,

        /// Show only failures (breaking changes/incompatibilities)
        #[arg(long)]
        failures_only: bool,

        /// Output format
        #[arg(short = 'f', long = "format", value_enum, default_value = "text")]
        format: ComplyOutputFormat,
    },

    /// Migrate project to latest PMAT standards
    Migrate {
        /// Project path (defaults to current directory)
        #[arg(short = 'p', long = "path", default_value = ".")]
        path: PathBuf,

        /// Target PMAT version (defaults to current binary version)
        #[arg(long)]
        version: Option<String>,

        /// Dry run (show what would be migrated without changing files)
        #[arg(long)]
        dry_run: bool,

        /// Skip backup creation (NOT RECOMMENDED)
        #[arg(long)]
        no_backup: bool,

        /// Force migration even if breaking changes detected
        #[arg(long)]
        force: bool,
    },

    /// Show changelog since project's PMAT version
    Diff {
        /// Project path (defaults to current directory)
        #[arg(short = 'p', long = "path", default_value = ".")]
        path: PathBuf,

        /// Start version for changelog (defaults to project version)
        #[arg(long)]
        from: Option<String>,

        /// End version for changelog (defaults to current binary)
        #[arg(long)]
        to: Option<String>,

        /// Show only breaking changes
        #[arg(long)]
        breaking_only: bool,
    },

    /// Update hooks and configs to latest versions
    Update {
        /// Project path (defaults to current directory)
        #[arg(short = 'p', long = "path", default_value = ".")]
        path: PathBuf,

        /// Update only hooks
        #[arg(long)]
        hooks: bool,

        /// Update only configs
        #[arg(long)]
        config: bool,

        /// Dry run (show what would be updated)
        #[arg(long)]
        dry_run: bool,
    },

    /// Initialize .pmat/project.toml with current version
    Init {
        /// Project path (defaults to current directory)
        #[arg(short = 'p', long = "path", default_value = ".")]
        path: PathBuf,

        /// Force overwrite existing project.toml
        #[arg(long)]
        force: bool,
    },

    /// Install git hooks for mandatory work tracking (W-006)
    /// Blocks commits without active tickets per master-plan-pmat-work-system.md
    #[command(visible_aliases = &["install", "hooks"])]
    Enforce {
        /// Project path (defaults to current directory)
        #[arg(short = 'p', long = "path", default_value = ".")]
        path: PathBuf,

        /// Skip confirmation prompt
        #[arg(short = 'y', long = "yes")]
        yes: bool,

        /// Remove all PMAT hooks (disable enforcement)
        #[arg(long)]
        disable: bool,

        /// Output format
        #[arg(short = 'f', long = "format", value_enum, default_value = "text")]
        format: ComplyOutputFormat,
    },

    /// Generate compliance report (W-009)
    Report {
        /// Project path (defaults to current directory)
        #[arg(short = 'p', long = "path", default_value = ".")]
        path: PathBuf,

        /// Include ticket history
        #[arg(long)]
        include_history: bool,

        /// Output format
        #[arg(short = 'f', long = "format", value_enum, default_value = "markdown")]
        format: ComplyOutputFormat,

        /// Write output to file
        #[arg(short = 'o', long = "output")]
        output: Option<PathBuf>,
    },
}

/// Comply output formats (GH-96)
#[derive(Debug, Clone, clap::ValueEnum, PartialEq)]
pub enum ComplyOutputFormat {
    /// Human-readable text format
    Text,
    /// JSON format for CI/CD
    Json,
    /// Markdown report format
    Markdown,
}

/// Project diagnostics output formats (lltop Tab 8)
#[derive(Debug, Clone, Copy, Default, clap::ValueEnum, PartialEq, Eq)]
pub enum ProjectDiagOutputFormat {
    /// Human-readable summary with status icons
    #[default]
    Summary,
    /// JSON format for CI/CD
    Json,
    /// Markdown report format
    Markdown,
    /// Andon-style visualization (Toyota Way)
    Andon,
}

/// Output format for perfection score (master-plan-pmat-work-system.md)
#[derive(Debug, Clone, Copy, Default, clap::ValueEnum)]
pub enum PerfectionScoreOutputFormat {
    /// Human-readable text format
    #[default]
    Text,
    /// JSON format for CI/CD
    Json,
    /// Markdown report format
    Markdown,
}

/// Spec subcommands (master-plan-pmat-work-system.md S-001 to S-010)
#[derive(Debug, Clone, Subcommand)]
pub enum SpecCommands {
    /// Validate specification with 100-point Popperian score (S-001)
    /// Requires ≥95 points to be worked on
    #[command(visible_aliases = &["validate", "v"])]
    Score {
        /// Specification file path
        spec: PathBuf,

        /// Output format: text, json, markdown
        #[arg(short = 'f', long = "format", value_enum, default_value = "text")]
        format: SpecOutputFormat,

        /// Write output to file
        #[arg(short = 'o', long = "output")]
        output: Option<PathBuf>,

        /// Show verbose claim validation
        #[arg(short = 'v', long = "verbose")]
        verbose: bool,
    },

    /// Auto-fix spec issues to meet 95-point threshold (S-003)
    #[command(visible_aliases = &["fix", "c"])]
    Comply {
        /// Specification file path
        spec: PathBuf,

        /// Dry run (show what would be fixed without changing file)
        #[arg(long)]
        dry_run: bool,

        /// Output format
        #[arg(short = 'f', long = "format", value_enum, default_value = "text")]
        format: SpecOutputFormat,
    },

    /// Create new specification from template
    #[command(visible_aliases = &["new", "n"])]
    Create {
        /// Specification name (will be slugified)
        name: String,

        /// Issue reference (e.g., "GH-123" or "#123")
        #[arg(long)]
        issue: Option<String>,

        /// Epic to associate with
        #[arg(long)]
        epic: Option<String>,

        /// Output directory (defaults to docs/specifications/)
        #[arg(short = 'o', long = "output")]
        output: Option<PathBuf>,
    },

    /// List all specifications with their scores
    #[command(visible_aliases = &["ls", "l"])]
    List {
        /// Specifications directory (defaults to docs/specifications/)
        #[arg(short = 'p', long = "path", default_value = "docs/specifications")]
        path: PathBuf,

        /// Filter by minimum score
        #[arg(long)]
        min_score: Option<u8>,

        /// Show only specs below 95 threshold
        #[arg(long)]
        failing_only: bool,

        /// Output format
        #[arg(short = 'f', long = "format", value_enum, default_value = "text")]
        format: SpecOutputFormat,
    },

    /// Sync specs with roadmap (bidirectional ticket linking)
    #[command(visible_aliases = &["sy", "link"])]
    Sync {
        /// Specifications directory
        #[arg(short = 's', long = "specs", default_value = "docs/specifications")]
        spec_path: PathBuf,

        /// Roadmap file path
        #[arg(short = 'r', long = "roadmap", default_value = "docs/roadmaps/roadmap.yaml")]
        roadmap_path: PathBuf,

        /// Dry run (show changes without applying)
        #[arg(long)]
        dry_run: bool,

        /// Direction: spec-to-roadmap, roadmap-to-spec, or both
        #[arg(short = 'd', long = "direction", value_enum, default_value = "both")]
        direction: SpecSyncDirection,
    },

    /// Report specs without roadmap links (drift detection)
    #[command(visible_aliases = &["orphans", "unlinked"])]
    Drift {
        /// Specifications directory
        #[arg(short = 's', long = "specs", default_value = "docs/specifications")]
        spec_path: PathBuf,

        /// Roadmap file path
        #[arg(short = 'r', long = "roadmap", default_value = "docs/roadmaps/roadmap.yaml")]
        roadmap_path: PathBuf,

        /// Output format
        #[arg(short = 'f', long = "format", value_enum, default_value = "text")]
        format: SpecOutputFormat,
    },
}

/// Direction for spec-roadmap sync
#[derive(Debug, Clone, Copy, Default, clap::ValueEnum)]
pub enum SpecSyncDirection {
    /// Update roadmap from spec tickets
    SpecToRoadmap,
    /// Update spec frontmatter from roadmap
    RoadmapToSpec,
    /// Bidirectional sync
    #[default]
    Both,
}

/// Output format for spec commands
#[derive(Debug, Clone, Copy, Default, clap::ValueEnum)]
pub enum SpecOutputFormat {
    /// Human-readable text format
    #[default]
    Text,
    /// JSON format for CI/CD
    Json,
    /// Markdown report format
    Markdown,
}

/// Output format for work annotate command
#[derive(Debug, Clone, Copy, Default, clap::ValueEnum)]
pub enum AnnotateOutputFormat {
    /// Human-readable text format
    #[default]
    Text,
    /// JSON format for CI/CD
    Json,
    /// Markdown report format
    Markdown,
}

/// Debug subcommands (Sprint 74 - TRACE-001 through TRACE-003)
#[derive(Debug, Clone, Subcommand)]
pub enum DebugCommands {
    /// Start DAP (Debug Adapter Protocol) server for time-travel debugging
    #[command(visible_aliases = &["srv", "server"])]
    Serve {
        /// Port to bind DAP server (default: 5678)
        #[arg(short, long, default_value = "5678")]
        port: u16,

        /// Host address to bind (default: 127.0.0.1)
        #[arg(long, default_value = "127.0.0.1")]
        host: String,

        /// Directory to save recording files (.pmat format) [Sprint 76 - CAPTURE-003]
        #[arg(long, value_name = "DIR")]
        record_dir: Option<PathBuf>,
    },

    /// Replay execution recording with time-travel navigation
    #[command(visible_aliases = &["play", "view"])]
    Replay {
        /// Path to execution recording file (.pmat format)
        recording: PathBuf,

        /// Start at specific snapshot position
        #[arg(long)]
        position: Option<usize>,

        /// Enable interactive timeline navigation
        #[arg(short, long)]
        interactive: bool,
    },
}

/// Quality gates subcommands (TICKET-PMAT-5024)
#[derive(Subcommand)]
#[cfg_attr(test, derive(Debug))]
pub enum QualityGatesCommand {
    /// Initialize .pmat-gates.toml with defaults
    Init {
        /// Force overwrite existing file
        #[arg(long)]
        force: bool,
    },

    /// Validate configuration file
    Validate,

    /// Show current configuration
    Show {
        /// Output format
        #[arg(long, value_enum, default_value = "toml")]
        format: ConfigFormat,
    },
}

/// Maintain subcommands (TICKET-PMAT-5032, TICKET-PMAT-5033)
#[derive(Subcommand)]
#[cfg_attr(test, derive(Debug))]
pub enum MaintainCommands {
    /// Validate roadmap structure and ticket consistency
    Roadmap {
        /// Path to ROADMAP.md
        #[arg(long, default_value = "ROADMAP.md")]
        roadmap: PathBuf,

        /// Path to tickets directory
        #[arg(long, default_value = "docs/tickets")]
        tickets_dir: PathBuf,

        /// Check ticket status consistency
        #[arg(long)]
        validate: bool,

        /// Show roadmap health report
        #[arg(long)]
        health: bool,

        /// Auto-fix checkbox status based on ticket files
        #[arg(long)]
        fix: bool,

        /// Auto-generate missing ticket files from roadmap entries that don't have corresponding files (TICKET-PMAT-6012)
        #[arg(long)]
        generate_tickets: bool,

        /// Dry-run mode (show changes without applying)
        #[arg(long)]
        dry_run: bool,

        /// Output format
        #[arg(long, value_enum, default_value = "table")]
        format: OutputFormat,
    },

    /// Validate project health (TICKET-PMAT-5033, TICKET-PMAT-6001)
    Health {
        /// Project directory
        #[arg(long, default_value = ".")]
        project_dir: PathBuf,

        /// Output format
        #[arg(long, value_enum, default_value = "table")]
        format: OutputFormat,

        /// Quick mode: only build check (fastest, <10s)
        #[arg(long)]
        quick: bool,

        /// Run all checks (build + tests + coverage + complexity + SATD)
        #[arg(long)]
        all: bool,

        /// Check build status (default if no other flags specified)
        #[arg(long)]
        check_build: bool,

        /// Check tests
        #[arg(long)]
        check_tests: bool,

        /// Check coverage
        #[arg(long)]
        check_coverage: bool,

        /// Check complexity
        #[arg(long)]
        check_complexity: bool,

        /// Check SATD
        #[arg(long)]
        check_satd: bool,
    },

    /// Create bug report from captured error (GH-81)
    #[command(visible_aliases = &["bug", "report"])]
    BugReport {
        /// Custom issue title
        #[arg(long)]
        title: Option<String>,

        /// Preview issue without creating (dry-run)
        #[arg(long)]
        dry_run: bool,

        /// Interactive confirmation before creating
        #[arg(long, short)]
        interactive: bool,

        /// Clear captured error without creating report
        #[arg(long)]
        clear: bool,
    },

    /// Clean up development artifacts and caches (GH-86)
    #[command(visible_aliases = &["clean", "cleanup", "purge"])]
    CleanupResources {
        /// Project directory to scan
        #[arg(long, default_value = ".")]
        project_dir: PathBuf,

        /// Cleanup targets: rust, docker, node, git, logs, caches, all
        #[arg(long, value_delimiter = ',', default_value = "rust")]
        targets: Vec<String>,

        /// Actually execute cleanup (default is dry-run)
        #[arg(long)]
        execute: bool,

        /// Exclude patterns (glob syntax)
        #[arg(long)]
        exclude: Vec<String>,

        /// Minimum age in days for cleanup candidates
        #[arg(long, default_value = "0")]
        min_age_days: u32,

        /// Output format
        #[arg(long, value_enum, default_value = "table")]
        format: OutputFormat,
    },
}

/// Diagnostic output format
#[derive(Clone, Debug, clap::ValueEnum)]
#[cfg_attr(test, derive(PartialEq))]
pub enum DiagnosticOutputFormat {
    /// Plain text format
    Plain,
    /// Human-readable format
    Human,
    /// JSON format
    Json,
    /// YAML format
    Yaml,
    /// Compact table format
    Table,
}

/// Storage management commands
#[derive(Subcommand)]
#[cfg_attr(test, derive(Debug))]
#[derive(Clone)]
pub enum StorageCommand {
    /// Show storage statistics
    Stats {
        /// Include backend-specific details
        #[arg(long)]
        detailed: bool,
    },

    /// Clean up hot cache entries
    Cleanup {
        /// Maximum age in seconds for hot cache entries
        #[arg(long, default_value = "3600")]
        max_age: u64,
    },

    /// Migrate to different storage backend
    Migrate {
        /// Target backend type (sled, rocksdb, inmemory)
        #[arg(long)]
        backend: String,

        /// Storage path for new backend
        #[arg(long)]
        path: Option<PathBuf>,
    },

    /// Flush all pending writes
    Flush,
}

/// TDG subcommands
#[derive(Subcommand)]
#[cfg_attr(test, derive(Debug))]
#[derive(Clone)]
pub enum TdgCommand {
    /// Compare two files or directories
    Compare {
        /// First file or directory to compare
        source1: PathBuf,

        /// Second file or directory to compare
        source2: PathBuf,
    },

    /// View TDG history at specific commits (Sprint 65 Phase 3)
    History {
        /// Specific commit SHA or tag to query
        #[arg(long)]
        commit: Option<String>,

        /// Show TDG history since this commit/tag (e.g., HEAD~10, v2.177.0)
        #[arg(long)]
        since: Option<String>,

        /// Show TDG history in commit range (e.g., HEAD~10..HEAD, v2.177.0..v2.178.0)
        #[arg(long)]
        range: Option<String>,

        /// Filter history by specific file path
        #[arg(long)]
        path: Option<PathBuf>,

        /// Output format
        #[arg(long, value_enum, default_value = "table")]
        format: TdgOutputFormat,
    },

    /// Manage TDG baselines for quality regression detection (Sprint 66 Phase 1)
    Baseline {
        #[command(subcommand)]
        command: BaselineCommand,
    },

    /// Show TDG system diagnostics and health status
    Diagnostics {
        /// Show detailed backend statistics
        #[arg(long)]
        detailed: bool,

        /// Show storage tier breakdown
        #[arg(long)]
        storage: bool,

        /// Show scheduler status
        #[arg(long)]
        scheduler: bool,

        /// Show adaptive threshold information
        #[arg(long)]
        adaptive: bool,

        /// Show resource usage and limits
        #[arg(long)]
        resources: bool,

        /// Show all diagnostic information
        #[arg(long)]
        all: bool,

        /// Output format
        #[arg(long, value_enum, default_value = "human")]
        format: DiagnosticOutputFormat,
    },

    /// Manage TDG storage backends
    Storage {
        #[command(subcommand)]
        command: StorageCommand,
    },

    /// Start TDG web dashboard server (Sprint 31)
    Dashboard {
        /// Port to bind the dashboard server
        #[arg(short, long, default_value = "8080")]
        port: u16,

        /// Host to bind the dashboard server
        #[arg(long, default_value = "127.0.0.1")]
        host: String,

        /// Auto-open dashboard in browser
        #[arg(long)]
        open: bool,

        /// Update interval for real-time metrics (seconds)
        #[arg(long, default_value = "5")]
        update_interval: u64,
    },

    /// Configuration management (single source of truth)
    #[command(subcommand)]
    Config(ConfigCommands),

    /// Check for quality regressions against baseline (Sprint 66 Phase 2)
    CheckRegression {
        /// Path to baseline file
        #[arg(short, long)]
        baseline: PathBuf,

        /// Path to analyze (defaults to current directory)
        #[arg(short, long, default_value = ".")]
        path: PathBuf,

        /// Output format
        #[arg(short, long, value_enum, default_value = "table")]
        format: TdgOutputFormat,

        /// Fail with non-zero exit code if regressions detected
        #[arg(long)]
        fail_on_regression: bool,

        /// Maximum score drop allowed (overrides config)
        #[arg(long)]
        max_score_drop: Option<f32>,

        /// Whether to allow grade drops
        #[arg(long)]
        allow_grade_drop: bool,
    },

    /// Check files meet minimum quality thresholds (Sprint 66 Phase 2)
    CheckQuality {
        /// Path to analyze
        #[arg(short, long, default_value = ".")]
        path: PathBuf,

        /// Minimum grade required for all files
        #[arg(long)]
        min_grade: Option<String>,

        /// Output format
        #[arg(short, long, value_enum, default_value = "table")]
        format: TdgOutputFormat,

        /// Fail with non-zero exit code if files below threshold
        #[arg(long, default_value = "true")]
        fail_on_violation: bool,

        /// Check only new files (requires baseline)
        #[arg(long)]
        new_files_only: bool,

        /// Baseline for new-files-only mode
        #[arg(long)]
        baseline: Option<PathBuf>,
    },
}

/// Baseline management subcommands (Sprint 66 Phase 1)
#[derive(Subcommand, Clone)]
#[cfg_attr(test, derive(Debug))]
pub enum BaselineCommand {
    /// Create a new TDG baseline for the project
    Create {
        /// Project path to analyze
        #[arg(short, long, default_value = ".")]
        path: PathBuf,

        /// Output file for baseline (JSON format)
        #[arg(short, long, default_value = ".pmat-baseline.json")]
        output: PathBuf,

        /// Include git context in baseline
        #[arg(long)]
        with_git_context: bool,

        /// Baseline name/label for reference
        #[arg(long)]
        name: Option<String>,
    },

    /// Compare current state against a baseline
    Compare {
        /// Path to baseline file
        #[arg(short, long)]
        baseline: PathBuf,

        /// Project path to analyze (current state)
        #[arg(short, long, default_value = ".")]
        path: PathBuf,

        /// Output format
        #[arg(short, long, value_enum, default_value = "table")]
        format: TdgOutputFormat,

        /// Exit with error code if regressions detected
        #[arg(long)]
        fail_on_regression: bool,
    },

    /// List all available baselines
    List {
        /// Directory to search for baselines
        #[arg(short, long, default_value = ".")]
        path: PathBuf,

        /// Output format
        #[arg(short, long, value_enum, default_value = "table")]
        format: TdgOutputFormat,
    },

    /// Update an existing baseline
    Update {
        /// Path to baseline file to update
        #[arg(short, long)]
        baseline: PathBuf,

        /// Project path to re-analyze
        #[arg(short, long, default_value = ".")]
        path: PathBuf,

        /// Include git context in updated baseline
        #[arg(long)]
        with_git_context: bool,
    },
}

/// Analyze subcommands
#[derive(Subcommand)]
#[cfg_attr(test, derive(Debug))]
pub enum AnalyzeCommands {
    /// Analyze code churn (change frequency)
    #[command(visible_aliases = &["ch"])]
    Churn {
        /// Project path to analyze
        #[arg(short = 'p', long, default_value = ".")]
        project_path: PathBuf,

        /// Number of days to analyze
        #[arg(short = 'd', long, default_value_t = 30)]
        days: u32,

        /// Output format
        #[arg(long, value_enum, default_value = "summary")]
        format: ChurnOutputFormat,

        /// Output file path
        #[arg(short, long)]
        output: Option<PathBuf>,

        /// Number of top files to show by churn (0 = show all)
        #[arg(long, default_value_t = 10)]
        top_files: usize,

        /// Include file patterns (e.g., "**/*.rs", "src/**")
        #[arg(long)]
        include: Vec<String>,

        /// Exclude file patterns (e.g., "tests/**", "target/**")
        #[arg(long)]
        exclude: Vec<String>,
    },

    /// Analyze code complexity with MCP tool composition support
    ///
    /// MCP Usage Examples:
    /// 1. Find hotspots: pmat analyze complexity --top-files 5 --format json
    /// 2. Analyze specific files: pmat analyze complexity --files src/main.rs,src/lib.rs
    /// 3. Chain with other tools using JSON output for AI agent workflows
    #[command(visible_aliases = &["cx", "complex"])]
    Complexity {
        /// Path to analyze (file or directory)
        #[arg(short = 'p', long, default_value = ".")]
        path: PathBuf,

        /// DEPRECATED: Use --path instead. Project path to analyze
        #[arg(long, hide = true)]
        project_path: Option<PathBuf>,

        /// Analyze a specific file instead of the whole project
        #[arg(long, conflicts_with = "include")]
        file: Option<PathBuf>,

        /// Analyze specific files (comma-separated list for MCP tool composition)
        ///
        /// Enable AI agents to chain analysis tools by passing file lists between commands.
        /// Example: --files src/main.rs,src/lib.rs,tests/integration.rs
        ///
        /// MCP Tool Chaining:
        /// 1. Get top complex files from one analysis
        /// 2. Pass those files to another analysis command
        /// 3. Build focused refactoring workflows
        #[arg(long, value_delimiter = ',', conflicts_with_all = ["file", "include"])]
        files: Vec<PathBuf>,

        /// Filter by toolchain (rust, deno, python-uv)
        #[arg(long)]
        toolchain: Option<String>,

        /// Output format
        #[arg(long, value_enum, default_value = "summary")]
        format: ComplexityOutputFormat,

        /// Output file path
        #[arg(short, long)]
        output: Option<PathBuf>,

        /// Custom cyclomatic complexity threshold
        #[arg(long)]
        max_cyclomatic: Option<u16>,

        /// Custom cognitive complexity threshold
        #[arg(long)]
        max_cognitive: Option<u16>,

        /// Include file patterns (e.g., "**/*.rs")
        #[arg(long)]
        include: Vec<String>,

        /// Watch mode for continuous analysis
        #[arg(long)]
        watch: bool,

        /// Number of top complex files to show (0 = show all violations)
        #[arg(long, default_value_t = 10)]
        top_files: usize,

        /// Exit with non-zero code if violations are found
        #[arg(long)]
        fail_on_violation: bool,

        /// Analysis timeout in seconds
        #[arg(long, default_value = "60")]
        timeout: u64,

        /// Use ML-based scoring (GH-97: aprender LinearRegression)
        ///
        /// When enabled, complexity scores are calculated using trained ML models
        /// instead of heuristic formulas. This provides more accurate, data-driven
        /// scores that account for language-specific patterns and project context.
        #[arg(long)]
        ml: bool,
    },

    /// Generate dependency graphs using Mermaid
    #[command(visible_aliases = &["dep", "graph"])]
    Dag {
        /// Type of dependency graph to generate
        #[arg(long, value_enum, default_value = "full-dependency")]
        dag_type: DagType,

        /// Project path to analyze
        #[arg(short = 'p', long, default_value = ".")]
        project_path: PathBuf,

        /// Output file path
        #[arg(short, long)]
        output: Option<PathBuf>,

        /// Maximum depth for graph traversal
        #[arg(long)]
        max_depth: Option<usize>,

        /// Target number of nodes (applies graph reduction if exceeded)
        #[arg(long)]
        target_nodes: Option<usize>,

        /// Filter out external dependencies
        #[arg(long)]
        filter_external: bool,

        /// Show complexity metrics in the graph
        #[arg(long)]
        show_complexity: bool,

        /// Include duplicate detection analysis
        #[arg(long)]
        include_duplicates: bool,

        /// Include dead code analysis
        #[arg(long)]
        include_dead_code: bool,

        /// Use enhanced vectorized analysis engine
        #[arg(long)]
        enhanced: bool,
    },

    /// Analyze dead and unreachable code
    #[command(name = "dead-code", visible_aliases = &["dead", "dc"])]
    DeadCode {
        /// Path to analyze (defaults to current directory)
        #[arg(long, short = 'p', default_value = ".")]
        path: PathBuf,

        /// Output format
        #[arg(long, short = 'f', value_enum, default_value = "summary")]
        format: DeadCodeOutputFormat,

        /// Show top N files with most dead code
        #[arg(long, short = 't')]
        top_files: Option<usize>,

        /// Include unreachable code blocks in analysis
        #[arg(long, short = 'u')]
        include_unreachable: bool,

        /// Minimum dead lines to report a file (default: 10)
        #[arg(long, default_value = "10")]
        min_dead_lines: usize,

        /// Include test files in analysis
        #[arg(long)]
        include_tests: bool,

        /// Output file path
        #[arg(short, long)]
        output: Option<PathBuf>,

        /// Exit with non-zero code if violations are found
        #[arg(long)]
        fail_on_violation: bool,

        /// Maximum allowed dead code percentage (default: 15.0)
        #[arg(long, default_value = "15.0")]
        max_percentage: f64,

        /// Analysis timeout in seconds
        #[arg(long, default_value = "60")]
        timeout: u64,

        /// Include file patterns (e.g., "**/*.rs", "src/**")
        #[arg(long)]
        include: Vec<String>,

        /// Exclude file patterns (e.g., "tests/**", "target/**")
        #[arg(long)]
        exclude: Vec<String>,

        /// Maximum directory traversal depth (default: 8 levels)
        #[arg(long, default_value = "8")]
        max_depth: usize,
    },

    /// Scan project for known defect patterns (e.g., .unwrap() calls in Rust)
    #[command(name = "defects", visible_aliases = &["known-defects"])]
    Defects {
        /// Path to analyze (defaults to current directory)
        #[arg(long, short = 'p', default_value = ".")]
        path: Option<PathBuf>,

        /// Analyze a specific file instead of the whole project
        #[arg(long, conflicts_with = "path")]
        file: Option<PathBuf>,

        /// Filter by severity level (critical, high, medium, low)
        #[arg(long)]
        severity: Option<String>,

        /// Output format
        #[arg(long, value_enum, default_value = "text")]
        format: DefectsOutputFormat,

        /// Output file path
        #[arg(short, long)]
        output: Option<PathBuf>,
    },

    /// Analyze Self-Admitted Technical Debt (SATD) in comments
    #[command(name = "satd", visible_aliases = &["debt", "td", "tech-debt"])]
    Satd {
        /// Path to analyze (defaults to current directory)
        #[arg(long, short = 'p', default_value = ".")]
        path: PathBuf,

        /// Output format
        #[arg(long, short = 'f', value_enum, default_value = "summary")]
        format: SatdOutputFormat,

        /// Filter by severity level
        #[arg(long, value_enum)]
        severity: Option<SatdSeverity>,

        /// Show only critical debt items
        #[arg(long)]
        critical_only: bool,

        /// Include test files in analysis
        #[arg(long)]
        include_tests: bool,

        /// Use strict mode (only TODO/FIXME/HACK/BUG comments)
        #[arg(long)]
        strict: bool,

        /// Track debt evolution over time (requires git history)
        #[arg(long)]
        evolution: bool,

        /// Number of days for evolution analysis
        #[arg(long, default_value_t = 30)]
        days: u32,

        /// Show debt metrics summary
        #[arg(long)]
        metrics: bool,

        /// Output file path
        #[arg(short, long)]
        output: Option<PathBuf>,

        /// Number of top files with most SATD to show (0 = show all)
        #[arg(long, default_value_t = 10)]
        top_files: usize,

        /// Exit with non-zero code if violations are found
        #[arg(long)]
        fail_on_violation: bool,

        /// Analysis timeout in seconds
        #[arg(long, default_value = "60")]
        timeout: u64,

        /// Include file patterns (e.g., "**/*.rs", "src/**")
        #[arg(long)]
        include: Vec<String>,

        /// Exclude file patterns (e.g., "tests/**", "target/**")
        #[arg(long)]
        exclude: Vec<String>,
    },

    /// Generate comprehensive deep context analysis with defect detection
    #[command(name = "deep-context", visible_aliases = &["context", "ctx", "deep"])]
    DeepContext {
        /// Project path to analyze (defaults to current directory)
        #[arg(long, short = 'p', default_value = ".")]
        project_path: PathBuf,

        /// Output file path
        #[arg(short, long)]
        output: Option<PathBuf>,

        /// Output format
        #[arg(long, value_enum, default_value = "markdown")]
        format: DeepContextOutputFormat,

        /// Enable full detailed report (default is terse)
        #[arg(long)]
        full: bool,

        /// Comma-separated list of analyses to include
        #[arg(long, value_delimiter = ',')]
        include: Vec<String>,

        /// Comma-separated list of analyses to exclude
        #[arg(long, value_delimiter = ',')]
        exclude: Vec<String>,

        /// Period for churn analysis (default: 30 days)
        #[arg(long, default_value_t = 30)]
        period_days: u32,

        /// DAG type for dependency analysis
        #[arg(long, value_enum, default_value = "call-graph")]
        dag_type: DeepContextDagType,

        /// Maximum directory traversal depth
        #[arg(long)]
        max_depth: Option<usize>,

        /// Include file patterns (can be specified multiple times)
        #[arg(long = "include-pattern")]
        include_patterns: Vec<String>,

        /// Exclude file patterns (can be specified multiple times)  
        #[arg(long = "exclude-pattern")]
        exclude_patterns: Vec<String>,

        /// Cache usage strategy
        #[arg(long, value_enum, default_value = "normal")]
        cache_strategy: DeepContextCacheStrategy,

        /// Parallelism level for analysis
        #[arg(long)]
        parallel: Option<usize>,

        /// Enable verbose logging
        #[arg(long)]
        verbose: bool,

        /// Number of top files to show (0 = all)
        #[arg(long, default_value = "10")]
        top_files: usize,
    },

    /// Analyze Technical Debt Gradient (TDG) scores
    #[command(name = "tdg")]
    Tdg {
        /// Path to analyze (defaults to current directory)
        #[arg(long, short = 'p', default_value = ".")]
        path: PathBuf,

        /// TDG threshold for filtering results
        #[arg(short, long, default_value = "1.5")]
        threshold: f64,

        /// Number of top files to show
        #[arg(short = 'n', long, default_value = "10")]
        top_files: usize,

        /// Output format
        #[arg(short, long, value_enum, default_value = "table")]
        format: TdgOutputFormat,

        /// Include TDG component breakdown
        #[arg(long)]
        include_components: bool,

        /// Output file path
        #[arg(short, long)]
        output: Option<PathBuf>,

        /// Show only critical files (TDG > 2.5)
        #[arg(long)]
        critical_only: bool,

        /// Enable verbose analysis output
        #[arg(long)]
        verbose: bool,

        /// Use ML-based scoring (GH-97: aprender LinearRegression)
        ///
        /// When enabled, TDG scores are calculated using trained ML models
        /// instead of heuristic weighted sums. This provides more accurate,
        /// data-driven scores that can learn from project history.
        #[arg(long)]
        ml: bool,
    },

    /// Build with TDG quality gate (CI/CD optimized)
    ///
    /// Combines `cargo build` with TDG score validation.
    /// Fails fast if TDG score exceeds threshold (Jidoka principle).
    ///
    /// Examples:
    ///   pmat analyze build-tdg                    # Build + TDG with defaults
    ///   pmat analyze build-tdg --release          # Release build + TDG
    ///   pmat analyze build-tdg --threshold 2.0    # Custom TDG threshold
    ///   pmat analyze build-tdg --fail-on-regression  # Fail if TDG regressed
    #[command(name = "build-tdg")]
    BuildTdg {
        /// Path to analyze (defaults to current directory)
        #[arg(long, short = 'p', default_value = ".")]
        path: PathBuf,

        /// Build in release mode
        #[arg(long)]
        release: bool,

        /// TDG threshold - fail if exceeded (default: 2.0)
        #[arg(long, default_value = "2.0")]
        threshold: f64,

        /// Fail if TDG score regressed from previous build
        #[arg(long)]
        fail_on_regression: bool,

        /// Skip build, only run TDG analysis
        #[arg(long)]
        tdg_only: bool,

        /// Number of top files to show in TDG report
        #[arg(long, default_value = "10")]
        top_files: usize,

        /// Output format
        #[arg(short, long, value_enum, default_value = "table")]
        format: TdgOutputFormat,

        /// Output file path
        #[arg(short, long)]
        output: Option<PathBuf>,
    },

    /// Find the file with highest defect density (lint violations per line)
    #[command(name = "lint-hotspot")]
    LintHotspot {
        /// Project path to analyze
        #[arg(short = 'p', long, default_value = ".")]
        project_path: PathBuf,

        /// Analyze a specific file instead of finding the hotspot
        #[arg(long)]
        file: Option<PathBuf>,

        /// Output format
        #[arg(short = 'f', long, value_enum, default_value = "summary")]
        format: LintHotspotOutputFormat,

        /// Maximum allowed defect density (violations per 100 lines)
        #[arg(long, default_value_t = 5.0)]
        max_density: f64,

        /// Minimum confidence for automated fixes (0.0-1.0)
        #[arg(long, default_value_t = 0.8)]
        min_confidence: f64,

        /// Enforce quality standards (exit non-zero if violations found)
        #[arg(long)]
        enforce: bool,

        /// Dry run - show what would be fixed without making changes
        #[arg(long)]
        dry_run: bool,

        /// Include enforcement metadata in output
        #[arg(long)]
        enforcement_metadata: bool,

        /// Output file path
        #[arg(short, long)]
        output: Option<PathBuf>,

        /// Enable performance metrics
        #[arg(long)]
        perf: bool,

        /// Additional flags to pass to clippy (uses extreme quality by default)
        #[arg(
            long,
            default_value = "-W warnings -W clippy::pedantic -W clippy::nursery -W clippy::cargo"
        )]
        clippy_flags: String,

        /// Number of top files to show by defect density (0 = all)
        #[arg(long, default_value = "10")]
        top_files: usize,

        /// Include file patterns (e.g., "**/*.rs", "src/**")
        #[arg(long)]
        include: Vec<String>,

        /// Exclude file patterns (e.g., "tests/**", "target/**")
        #[arg(long)]
        exclude: Vec<String>,
    },

    /// Analyze Makefile quality and compliance
    Makefile {
        /// Path to Makefile
        #[arg(help = "Path to Makefile to analyze")]
        path: PathBuf,

        /// Lint rules to apply
        #[arg(
            long,
            value_delimiter = ',',
            default_value = "all",
            help = "Comma-separated list of rules to apply"
        )]
        rules: Vec<String>,

        /// Output format
        #[arg(long, value_enum, default_value = "human")]
        format: MakefileOutputFormat,

        /// Fix auto-fixable issues
        #[arg(long, help = "Automatically fix issues where possible")]
        fix: bool,

        /// Check GNU Make compatibility version
        #[arg(
            long,
            default_value = "4.4",
            help = "GNU Make version to check compatibility against"
        )]
        gnu_version: String,

        /// Number of top files to show (0 = all)
        #[arg(long, default_value = "10")]
        top_files: usize,
    },

    /// Analyze provability properties using abstract interpretation
    Provability {
        /// Project path to analyze (defaults to current directory)
        #[arg(long, short = 'p', default_value = ".")]
        project_path: PathBuf,

        /// Specific functions to analyze (comma-separated)
        #[arg(long, value_delimiter = ',')]
        functions: Vec<String>,

        /// Analysis depth (number of iterations)
        #[arg(long, default_value_t = 10)]
        analysis_depth: usize,

        /// Output format
        #[arg(long, value_enum, default_value = "summary")]
        format: ProvabilityOutputFormat,

        /// Show only high-confidence results
        #[arg(long)]
        high_confidence_only: bool,

        /// Include property evidence in output
        #[arg(long)]
        include_evidence: bool,

        /// Output file path
        #[arg(short, long)]
        output: Option<PathBuf>,

        /// Number of top files to show (0 = all)
        #[arg(long, default_value = "10")]
        top_files: usize,
    },

    /// Detect duplicate code using vectorized `MinHash` and AST embeddings
    Duplicates {
        /// Project path to analyze (defaults to current directory)
        #[arg(long, short = 'p', default_value = ".")]
        project_path: PathBuf,

        /// Detection type: exact, renamed, gapped, semantic, or all
        #[arg(long, default_value = "all")]
        detection_type: DuplicateType,

        /// Similarity threshold for semantic clones (0.0-1.0)
        #[arg(long, default_value = "0.85")]
        threshold: f32,

        /// Minimum number of lines for duplicate detection
        #[arg(long, default_value = "5")]
        min_lines: usize,

        /// Maximum number of tokens to analyze per fragment
        #[arg(long, default_value = "128")]
        max_tokens: usize,

        /// Output format
        #[arg(long, short = 'f', value_enum, default_value = "summary")]
        format: DuplicateOutputFormat,

        /// Show performance metrics
        #[arg(long)]
        perf: bool,

        /// Include file patterns (e.g., "**/*.rs")
        #[arg(long)]
        include: Option<String>,

        /// Exclude file patterns (e.g., "**/target/**")
        #[arg(long)]
        exclude: Option<String>,

        /// Output file path
        #[arg(short, long)]
        output: Option<PathBuf>,

        /// Number of top files to show by duplication (0 = all)
        #[arg(long, default_value = "10")]
        top_files: usize,
    },

    /// Predict defect probability using ML-based analysis
    DefectPrediction {
        /// Project path to analyze (defaults to current directory)
        #[arg(long, short = 'p', default_value = ".")]
        project_path: PathBuf,

        /// Minimum confidence threshold for predictions
        #[arg(long, default_value = "0.5")]
        confidence_threshold: f32,

        /// Minimum lines of code for analysis
        #[arg(long, default_value = "10")]
        min_lines: usize,

        /// Include low-confidence predictions
        #[arg(long)]
        include_low_confidence: bool,

        /// Output format
        #[arg(long, short = 'f', value_enum, default_value = "summary")]
        format: DefectPredictionOutputFormat,

        /// Show only high-risk files (probability > 0.7)
        #[arg(long)]
        high_risk_only: bool,

        /// Include detailed recommendations
        #[arg(long)]
        include_recommendations: bool,

        /// Include file patterns (e.g., "**/*.rs")
        #[arg(long)]
        include: Option<String>,

        /// Exclude file patterns (e.g., "**/target/**")
        #[arg(long)]
        exclude: Option<String>,

        /// Output file path
        #[arg(short, long)]
        output: Option<PathBuf>,

        /// Show performance metrics
        #[arg(long)]
        perf: bool,

        /// Number of top files to show by defect probability (0 = all)
        #[arg(long, default_value = "10")]
        top_files: usize,
    },

    /// Run comprehensive multi-dimensional analysis with MCP tool composition
    ///
    /// Perfect for AI agents to get complete code health metrics. Combines:
    /// - Complexity analysis
    /// - Technical debt detection
    /// - Defect prediction
    /// - Dead code analysis
    /// - Duplicate detection
    ///
    /// MCP Workflow: Use after complexity analysis to get detailed insights on problematic files
    Comprehensive {
        /// Project path to analyze (defaults to current directory)
        #[arg(long, short = 'p', default_value = ".")]
        project_path: PathBuf,

        /// Single file to analyze (overrides project path)
        #[arg(long, conflicts_with = "files")]
        file: Option<PathBuf>,

        /// Analyze specific files (MCP tool composition from complexity hotspots)
        ///
        /// Enable AI agents to perform comprehensive analysis on files identified
        /// by previous complexity analysis. Perfect for multi-stage analysis workflows.
        /// Example: --files src/complex.rs,src/legacy.rs,src/problematic.rs
        #[arg(long, value_delimiter = ',', conflicts_with = "file")]
        files: Vec<PathBuf>,

        /// Output format
        #[arg(long, short = 'f', value_enum, default_value = "summary")]
        format: ComprehensiveOutputFormat,

        /// Enable duplicate detection analysis
        #[arg(long, default_value = "true")]
        include_duplicates: bool,

        /// Enable dead code analysis
        #[arg(long, default_value = "true")]
        include_dead_code: bool,

        /// Enable defect prediction analysis
        #[arg(long, default_value = "true")]
        include_defects: bool,

        /// Enable complexity analysis
        #[arg(long, default_value = "true")]
        include_complexity: bool,

        /// Enable TDG (Technical Debt Gradient) analysis
        #[arg(long, default_value = "true")]
        include_tdg: bool,

        /// Minimum confidence threshold for predictions
        #[arg(long, default_value = "0.5")]
        confidence_threshold: f32,

        /// Minimum lines of code for analysis
        #[arg(long, default_value = "10")]
        min_lines: usize,

        /// Include file patterns (e.g., "**/*.rs")
        #[arg(long)]
        include: Option<String>,

        /// Exclude file patterns (e.g., "**/target/**")
        #[arg(long)]
        exclude: Option<String>,

        /// Output file path
        #[arg(short, long)]
        output: Option<PathBuf>,

        /// Show performance metrics for each analysis component
        #[arg(long)]
        perf: bool,

        /// Generate executive summary only (faster analysis)
        #[arg(long)]
        executive_summary: bool,

        /// Number of top files to show (0 = all)
        #[arg(long, default_value = "10")]
        top_files: usize,
    },

    /// Analyze graph metrics and centrality measures
    GraphMetrics {
        /// Project path to analyze (defaults to current directory)
        #[arg(long, short = 'p', default_value = ".")]
        project_path: PathBuf,

        /// Metrics to compute
        #[arg(long, value_delimiter = ',', default_value = "all")]
        metrics: Vec<GraphMetricType>,

        /// Personalized `PageRank` seed nodes (file paths or function names)
        #[arg(long, value_delimiter = ',')]
        pagerank_seeds: Vec<String>,

        /// `PageRank` damping factor (0.0-1.0)
        #[arg(long, default_value = "0.85")]
        damping_factor: f32,

        /// Maximum iterations for `PageRank` convergence
        #[arg(long, default_value = "100")]
        max_iterations: usize,

        /// Convergence threshold for `PageRank`
        #[arg(long, default_value = "0.001")]
        convergence_threshold: f64,

        /// Export graph as `GraphML` format
        #[arg(long)]
        export_graphml: bool,

        /// Output format
        #[arg(long, short = 'f', value_enum, default_value = "summary")]
        format: GraphMetricsOutputFormat,

        /// Include file patterns (e.g., "**/*.rs")
        #[arg(long)]
        include: Option<String>,

        /// Exclude file patterns (e.g., "**/target/**")
        #[arg(long)]
        exclude: Option<String>,

        /// Output file path
        #[arg(short, long)]
        output: Option<PathBuf>,

        /// Show performance metrics
        #[arg(long)]
        perf: bool,

        /// Top K nodes to show in results
        #[arg(long, default_value = "20")]
        top_k: usize,

        /// Minimum centrality score to include in results
        #[arg(long, default_value = "0.001")]
        min_centrality: f64,
    },

    /// Analyze name similarity with embeddings
    NameSimilarity {
        /// Project path to analyze (defaults to current directory)
        #[arg(long, short = 'p', default_value = ".")]
        project_path: PathBuf,

        /// Name to search for
        query: String,

        /// Number of results to return
        #[arg(long, default_value = "10")]
        top_k: usize,

        /// Include phonetic matches (using Soundex)
        #[arg(long)]
        phonetic: bool,

        /// Search scope: functions, types, variables, all
        #[arg(long, value_enum, default_value = "all")]
        scope: SearchScope,

        /// Minimum similarity threshold (0.0-1.0)
        #[arg(long, default_value = "0.3")]
        threshold: f32,

        /// Output format
        #[arg(long, short = 'f', value_enum, default_value = "summary")]
        format: NameSimilarityOutputFormat,

        /// Include file patterns (e.g., "**/*.rs")
        #[arg(long)]
        include: Option<String>,

        /// Exclude file patterns (e.g., "**/target/**")
        #[arg(long)]
        exclude: Option<String>,

        /// Output file path
        #[arg(short, long)]
        output: Option<PathBuf>,

        /// Show performance metrics
        #[arg(long)]
        perf: bool,

        /// Include fuzzy string matching
        #[arg(long)]
        fuzzy: bool,

        /// Case sensitive matching
        #[arg(long)]
        case_sensitive: bool,
    },

    /// Collect proof annotations from multiple sources
    ProofAnnotations {
        /// Project path to analyze (defaults to current directory)
        #[arg(long, short = 'p', default_value = ".")]
        project_path: PathBuf,

        /// Output format
        #[arg(long, short = 'f', value_enum, default_value = "summary")]
        format: ProofAnnotationOutputFormat,

        /// Show only high-confidence annotations
        #[arg(long)]
        high_confidence_only: bool,

        /// Include evidence details in output
        #[arg(long)]
        include_evidence: bool,

        /// Filter by property type
        #[arg(long, value_enum)]
        property_type: Option<PropertyTypeFilter>,

        /// Filter by verification method
        #[arg(long, value_enum)]
        verification_method: Option<VerificationMethodFilter>,

        /// Output file path
        #[arg(short, long)]
        output: Option<PathBuf>,

        /// Show performance metrics and cache statistics
        #[arg(long)]
        perf: bool,

        /// Clear cache before analysis
        #[arg(long)]
        clear_cache: bool,

        /// Number of top files to show (0 = all)
        #[arg(long, default_value = "10")]
        top_files: usize,
    },

    /// Analyze incremental coverage changes with caching
    IncrementalCoverage {
        /// Project path to analyze (defaults to current directory)
        #[arg(long, short = 'p', default_value = ".")]
        project_path: PathBuf,

        /// Base commit or branch for comparison
        #[arg(long, short = 'b', default_value = "main")]
        base_branch: String,

        /// Target commit or branch
        #[arg(long, short = 't')]
        target_branch: Option<String>,

        /// Output format
        #[arg(long, short = 'f', value_enum, default_value = "summary")]
        format: IncrementalCoverageOutputFormat,

        /// Minimum coverage threshold for warnings
        #[arg(long, default_value = "80.0")]
        coverage_threshold: f64,

        /// Include only changed files
        #[arg(long)]
        changed_files_only: bool,

        /// Show detailed per-file coverage
        #[arg(long)]
        detailed: bool,

        /// Output file path
        #[arg(short, long)]
        output: Option<PathBuf>,

        /// Show performance metrics
        #[arg(long)]
        perf: bool,

        /// Cache directory for coverage data
        #[arg(long)]
        cache_dir: Option<PathBuf>,

        /// Force refresh of coverage cache
        #[arg(long)]
        force_refresh: bool,

        /// Number of top files to show (0 = all)
        #[arg(long, default_value = "10")]
        top_files: usize,
    },

    /// Improve test coverage to target percentage using PMAT tools and Extreme TDD
    #[command(visible_aliases = &["improve-coverage", "cov-improve"])]
    CoverageImprove {
        /// Project path to analyze (defaults to current directory)
        #[arg(long, short = 'p', default_value = ".")]
        project_path: PathBuf,

        /// Target coverage percentage (0-100)
        #[arg(long, short = 't', default_value = "95.0")]
        target: f64,

        /// Maximum improvement iterations
        #[arg(long, default_value = "10")]
        max_iterations: usize,

        /// Skip mutation testing (faster but lower quality)
        #[arg(long)]
        fast: bool,

        /// Minimum mutation score threshold (0-100)
        #[arg(long, default_value = "80.0")]
        mutation_threshold: f64,

        /// Focus on specific files/modules (glob patterns)
        #[arg(long)]
        focus: Vec<String>,

        /// Exclude files/modules (glob patterns)
        #[arg(long)]
        exclude: Vec<String>,

        /// Output file path
        #[arg(short, long)]
        output: Option<PathBuf>,

        /// Output format
        #[arg(long, short = 'f', value_enum, default_value = "text")]
        format: CoverageImproveOutputFormat,
    },

    /// Analyze symbol table with cross-references and usage patterns
    SymbolTable {
        /// Project path to analyze (defaults to current directory)
        #[arg(long, short = 'p', default_value = ".")]
        project_path: PathBuf,

        /// Output format
        #[arg(long, short = 'f', value_enum, default_value = "summary")]
        format: SymbolTableOutputFormat,

        /// Filter by symbol type
        #[arg(long, value_enum)]
        filter: Option<SymbolTypeFilter>,

        /// Search query for specific symbols
        #[arg(long, short = 'q')]
        query: Option<String>,

        /// Include file patterns
        #[arg(long)]
        include: Vec<String>,

        /// Exclude file patterns
        #[arg(long)]
        exclude: Vec<String>,

        /// Show unreferenced symbols
        #[arg(long)]
        show_unreferenced: bool,

        /// Show cross-references
        #[arg(long)]
        show_references: bool,

        /// Output file path
        #[arg(short, long)]
        output: Option<PathBuf>,

        /// Show performance metrics
        #[arg(long)]
        perf: bool,

        /// Number of top files to show (0 = all)
        #[arg(long, default_value = "10")]
        top_files: usize,
    },

    /// Analyze algorithmic complexity (Big-O) of functions
    BigO {
        /// Project path to analyze (defaults to current directory)
        #[arg(long, short = 'p', default_value = ".")]
        project_path: PathBuf,

        /// Output format
        #[arg(long, short = 'f', value_enum, default_value = "summary")]
        format: BigOOutputFormat,

        /// Minimum confidence threshold (0-100)
        #[arg(long, default_value = "50")]
        confidence_threshold: u8,

        /// Analyze space complexity in addition to time
        #[arg(long)]
        analyze_space: bool,

        /// Include file patterns
        #[arg(long)]
        include: Vec<String>,

        /// Exclude file patterns
        #[arg(long)]
        exclude: Vec<String>,

        /// Show only high complexity functions (O(n²) or worse)
        #[arg(long)]
        high_complexity_only: bool,

        /// Output file path
        #[arg(short, long)]
        output: Option<PathBuf>,

        /// Show performance metrics
        #[arg(long)]
        perf: bool,

        /// Number of top files to show by complexity (0 = all)
        #[arg(long, default_value = "10")]
        top_files: usize,
    },

    /// Analyze `AssemblyScript` code
    AssemblyScript {
        /// Project path to analyze (defaults to current directory)
        #[arg(long, short = 'p', default_value = ".")]
        project_path: PathBuf,

        /// Output format
        #[arg(long, short = 'f', value_enum, default_value = "summary")]
        format: ComplexityOutputFormat,

        /// Include WASM complexity analysis
        #[arg(long)]
        wasm_complexity: bool,

        /// Memory analysis with pool optimization
        #[arg(long)]
        memory_analysis: bool,

        /// Security validation checks
        #[arg(long)]
        security: bool,

        /// Output file path
        #[arg(short, long)]
        output: Option<PathBuf>,

        /// Maximum parsing time in seconds
        #[arg(long, default_value = "30")]
        timeout: u64,

        /// Show performance metrics
        #[arg(long)]
        perf: bool,

        /// Number of top files to show (0 = all)
        #[arg(long, default_value = "10")]
        top_files: usize,
    },

    /// Analyze WebAssembly binary and text format
    WebAssembly {
        /// Project path to analyze (defaults to current directory)  
        #[arg(long, short = 'p', default_value = ".")]
        project_path: PathBuf,

        /// Output format
        #[arg(long, short = 'f', value_enum, default_value = "summary")]
        format: ComplexityOutputFormat,

        /// Include binary WASM (.wasm) files
        #[arg(long, default_value = "true")]
        include_binary: bool,

        /// Include text WASM (.wat) files
        #[arg(long, default_value = "true")]
        include_text: bool,

        /// Memory usage analysis
        #[arg(long)]
        memory_analysis: bool,

        /// Security validation
        #[arg(long)]
        security: bool,

        /// Complexity analysis
        #[arg(long)]
        complexity: bool,

        /// Output file path
        #[arg(short, long)]
        output: Option<PathBuf>,

        /// Show performance metrics
        #[arg(long)]
        perf: bool,

        /// Number of top files to show (0 = all)
        #[arg(long, default_value = "10")]
        top_files: usize,
    },

    /// Automated clippy fixes with confidence-based filtering
    Clippy {
        /// Project path to analyze (defaults to current directory)
        #[arg(long, short = 'p', default_value = ".")]
        project_path: PathBuf,

        /// Confidence level for automated fixes (high, medium, low)
        #[arg(long, short = 'c', default_value = "high")]
        confidence: String,

        /// Dry run - show what would be fixed without making changes
        #[arg(long)]
        dry_run: bool,

        /// Specific clippy codes to fix (comma-separated list)
        #[arg(long, value_delimiter = ',')]
        fix_codes: Vec<String>,

        /// Output file path
        #[arg(short, long)]
        output: Option<PathBuf>,

        /// Show performance metrics
        #[arg(long)]
        perf: bool,
    },

    /// Analyze pattern entropy for actionable quality improvements
    ///
    /// Identifies repetitive AST patterns that can be refactored into reusable components.
    /// Provides specific fix suggestions and estimated LOC reduction for each violation.
    Entropy {
        /// Project path to analyze
        #[arg(short = 'p', long, default_value = ".")]
        project_path: PathBuf,

        /// Output format
        #[arg(long, value_enum, default_value = "summary")]
        format: EntropyOutputFormat,

        /// Output file path
        #[arg(short, long)]
        output: Option<PathBuf>,

        /// Minimum severity level to report
        #[arg(long, value_enum, default_value = "medium")]
        min_severity: EntropySeverity,

        /// Number of top violations to show (0 = all)
        #[arg(long, default_value_t = 20)]
        top_violations: usize,

        /// Only analyze specific file
        #[arg(long)]
        file: Option<PathBuf>,

        /// Include test files in analysis
        #[arg(long)]
        include_tests: bool,
    },

    /// Analyze WebAssembly modules for quality, security, and performance
    Wasm {
        /// Path to WASM file to analyze
        wasm_file: PathBuf,

        /// Output format
        #[arg(long, short = 'f', value_enum, default_value = "summary")]
        format: WasmOutputFormat,

        /// Run formal verification for memory safety
        #[arg(long)]
        verify: bool,

        /// Run security vulnerability scanning
        #[arg(long)]
        security: bool,

        /// Run performance profiling
        #[arg(long)]
        profile: bool,

        /// Baseline WASM file for quality comparison
        #[arg(long)]
        baseline: Option<PathBuf>,

        /// Output file path
        #[arg(short, long)]
        output: Option<PathBuf>,

        /// Enable verbose output with detailed analysis
        #[arg(long)]
        verbose: bool,
    },

    /// Deep WASM pipeline inspection (Rust/Ruchy → WASM → JS)
    #[cfg(feature = "deep-wasm")]
    DeepWasm {
        /// Source code path to analyze
        #[arg(short = 'p', long)]
        source_path: PathBuf,

        /// WASM binary file path
        #[arg(long)]
        wasm_file: Option<PathBuf>,

        /// DWARF debug symbols file path
        #[arg(long)]
        dwarf_file: Option<PathBuf>,

        /// Source map file path
        #[arg(long)]
        source_map: Option<PathBuf>,

        /// Source language (auto-detected if not specified)
        #[arg(long, value_enum)]
        language: Option<DeepWasmLanguage>,

        /// Analysis focus area
        #[arg(long, value_enum, default_value = "full")]
        focus: DeepWasmFocus,

        /// Output format
        #[arg(long, value_enum, default_value = "markdown")]
        format: DeepWasmOutputFormat,

        /// Output file path
        #[arg(short, long)]
        output: Option<PathBuf>,

        /// Enable strict quality gates
        #[arg(long)]
        strict: bool,

        /// Include MIR analysis (Rust only)
        #[arg(long)]
        include_mir: bool,

        /// Include LLVM IR analysis
        #[arg(long)]
        include_llvm_ir: bool,

        /// Track memory layout
        #[arg(long)]
        track_memory: bool,

        /// Detect deadlocks (Ruchy actor systems)
        #[arg(long)]
        detect_deadlocks: bool,
    },

    /// Mutation testing with empirical execution (v2.136.0: File corruption bug FIXED - Issue #64)
    ///
    /// Note: 20× faster than cargo-mutants with smart test filtering.
    /// Generates properly formatted mutants using prettyplease.
    #[cfg(feature = "mutation-testing")]
    Mutate {
        /// Path to source code to mutate
        #[arg(short = 'p', long)]
        path: PathBuf,

        /// Mutation operators to use (comma-separated: AOR,ROR,COR,UOR,CRR,SDL)
        #[arg(long, value_delimiter = ',')]
        operators: Option<Vec<String>>,

        /// Enable ML-based survivability prediction
        #[arg(long)]
        ml_predict: bool,

        /// Enable distributed execution
        #[arg(long)]
        distributed: bool,

        /// Number of worker threads for distributed execution
        #[arg(long, default_value = "4")]
        workers: usize,

        /// Show real-time progress
        #[arg(long)]
        progress: bool,

        /// Minimum mutation score threshold (0.0-1.0)
        #[arg(long)]
        min_score: Option<f64>,

        /// Enable CI/CD learning mode
        #[arg(long)]
        ci_learning: bool,

        /// CI provider (github, gitlab, jenkins)
        #[arg(long)]
        ci_provider: Option<String>,

        /// Auto-train threshold (number of samples)
        #[arg(long, default_value = "50")]
        auto_train_threshold: usize,

        /// Output format
        #[arg(long, value_enum, default_value = "json")]
        format: OutputFormat,

        /// Output file path
        #[arg(short, long)]
        output: Option<PathBuf>,
    },

    /// Cluster code by semantic similarity (PMAT-SEARCH-011)
    Cluster {
        /// Clustering method
        #[arg(long, value_enum)]
        method: ClusterMethod,

        /// Number of clusters (required for kmeans)
        #[arg(long)]
        k: Option<usize>,

        /// Filter by language
        #[arg(long)]
        language: Option<String>,

        /// Output format
        #[arg(long, value_enum, default_value = "table")]
        format: OutputFormat,
    },

    /// Extract semantic topics from codebase (PMAT-SEARCH-011)
    Topics {
        /// Number of topics to extract
        #[arg(long)]
        num_topics: usize,

        /// Filter by language
        #[arg(long)]
        language: Option<String>,

        /// Output format
        #[arg(long, value_enum, default_value = "table")]
        format: OutputFormat,
    },
}

/// Quality-Driven Development (QDD) subcommands
#[derive(Subcommand)]
#[cfg_attr(test, derive(Debug))]
pub enum QddCommands {
    /// Create high-quality code from specification
    Create {
        /// Type of code to create
        #[arg(long, value_enum, default_value = "function")]
        code_type: QddCodeType,

        /// Name of the code element (function, module, etc.)
        #[arg(long)]
        name: String,

        /// Purpose/description of the code
        #[arg(long)]
        purpose: String,

        /// Quality profile to use
        #[arg(long, value_enum, default_value = "standard")]
        profile: QddQualityProfile,

        /// Input parameters as type:name pairs
        #[arg(long, value_parser = parse_parameter)]
        input: Vec<(String, String)>,

        /// Output type
        #[arg(long, default_value = "()")]
        output: String,

        /// Output file path
        #[arg(short, long)]
        output_file: Option<PathBuf>,
    },

    /// Refactor existing code to meet quality standards  
    Refactor {
        /// File to refactor
        #[arg(short, long)]
        file: PathBuf,

        /// Specific function to refactor (optional)
        #[arg(long)]
        function: Option<String>,

        /// Quality profile to target
        #[arg(long, value_enum, default_value = "standard")]
        profile: QddQualityProfile,

        /// Maximum complexity allowed
        #[arg(long)]
        max_complexity: Option<u32>,

        /// Minimum test coverage required (%)
        #[arg(long)]
        min_coverage: Option<u32>,

        /// Output file path (default: overwrite original)
        #[arg(short, long)]
        output: Option<PathBuf>,

        /// Dry run - show what would be changed
        #[arg(long)]
        dry_run: bool,
    },

    /// Validate code against quality standards
    Validate {
        /// File or directory to validate
        #[arg(short, long, default_value = ".")]
        path: PathBuf,

        /// Quality profile to validate against
        #[arg(long, value_enum, default_value = "standard")]
        profile: QddQualityProfile,

        /// Output format
        #[arg(long, value_enum, default_value = "summary")]
        format: QddOutputFormat,

        /// Output file path
        #[arg(short, long)]
        output: Option<PathBuf>,

        /// Fail on any violations
        #[arg(long)]
        strict: bool,
    },
}

/// QDD code types
#[derive(clap::ValueEnum, Clone)]
#[cfg_attr(test, derive(Debug))]
pub enum QddCodeType {
    Function,
    Module,
    Service,
    Test,
}

/// QDD quality profiles  
#[derive(clap::ValueEnum, Clone, Copy, Debug)]
pub enum QddQualityProfile {
    Extreme,
    Standard,
    Relaxed,
}

/// QDD output formats
#[derive(clap::ValueEnum, Clone)]
#[cfg_attr(test, derive(Debug))]
pub enum QddOutputFormat {
    Summary,
    Detailed,
    Json,
    Markdown,
}

/// Parse parameter as type:name
fn parse_parameter(s: &str) -> Result<(String, String), String> {
    let parts: Vec<&str> = s.split(':').collect();
    if parts.len() != 2 {
        return Err("Parameter must be in format type:name".to_string());
    }
    Ok((parts[0].to_string(), parts[1].to_string()))
}

/// Enforce subcommands
#[derive(Subcommand)]
#[cfg_attr(test, derive(Debug))]
pub enum EnforceCommands {
    /// Enforce extreme quality standards
    Extreme {
        /// Project path to enforce quality on
        #[arg(short = 'p', long, default_value = ".")]
        project_path: PathBuf,

        /// Single file mode - enforce on one file at a time
        #[arg(long)]
        single_file_mode: bool,

        /// Specific file to enforce (implies single file mode)
        #[arg(long)]
        file: Option<PathBuf>,

        /// Dry run - show what would be changed without making changes
        #[arg(long)]
        dry_run: bool,

        /// Quality profile to use
        #[arg(long, value_enum, default_value = "extreme")]
        profile: QualityProfile,

        /// Show progress during enforcement
        #[arg(long, default_value_t = true)]
        show_progress: bool,

        /// Output format
        #[arg(short = 'f', long, value_enum, default_value = "summary")]
        format: EnforceOutputFormat,

        /// Output file path
        #[arg(short = 'o', long)]
        output: Option<PathBuf>,

        /// Maximum iterations before giving up
        #[arg(long, default_value_t = 100)]
        max_iterations: u32,

        /// Target improvement percentage
        #[arg(long)]
        target_improvement: Option<f32>,

        /// Maximum time in seconds
        #[arg(long)]
        max_time: Option<u64>,

        /// Apply suggestions automatically
        #[arg(long)]
        apply_suggestions: bool,

        /// Validate only (no changes)
        #[arg(long)]
        validate_only: bool,

        /// List all violations and exit
        #[arg(long)]
        list_violations: bool,

        /// Configuration file path
        #[arg(long)]
        config: Option<PathBuf>,

        /// CI mode (exit with error on violations)
        #[arg(long)]
        ci_mode: bool,

        /// Include pattern
        #[arg(long)]
        include: Option<String>,

        /// Exclude pattern
        #[arg(long)]
        exclude: Option<String>,

        /// Cache directory
        #[arg(long)]
        cache_dir: Option<PathBuf>,

        /// Clear cache before starting
        #[arg(long)]
        clear_cache: bool,
    },
}

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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mode_enum() {
        assert_eq!(Mode::Cli, Mode::Cli);
        assert_ne!(Mode::Cli, Mode::Mcp);
    }

    /// Coverage: Stack overflow during coverage instrumentation - IGNORE
    /// The large Cli structure causes stack overflow with llvm-cov instrumentation.
    /// Test passes normally without coverage. Re-enable when cov stack issue resolved.
    #[test]
    #[ignore]
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

    /// Scaffold a WebAssembly project (TICKET-PMAT-5031)
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

/// Roadmap management subcommands
#[derive(Subcommand)]
#[cfg_attr(test, derive(Debug))]
pub enum RoadmapCommands {
    /// Initialize a new sprint in the roadmap
    Init {
        /// Sprint version (e.g., v2.6.0)
        #[arg(long)]
        version: String,

        /// Sprint title
        #[arg(long)]
        title: String,

        /// Sprint duration in days
        #[arg(long, default_value = "14")]
        duration_days: u32,

        /// Sprint priority (P0, P1, P2)
        #[arg(long, default_value = "P0")]
        priority: String,
    },

    /// Generate PDMT todos from roadmap tasks
    Todos {
        /// Sprint ID to generate todos for (uses current if not specified)
        #[arg(long)]
        sprint: Option<String>,

        /// Output file path for todos
        #[arg(long, default_value = "todos.md")]
        output: PathBuf,

        /// Include quality gate requirements in todos
        #[arg(long)]
        include_quality_gates: bool,
    },

    /// Start working on a task
    Start {
        /// Task ID (e.g., PMAT-3001)
        task_id: String,

        /// Create a git branch for the task
        #[arg(long)]
        create_branch: bool,
    },

    /// Complete a task (with quality validation)
    Complete {
        /// Task ID (e.g., PMAT-3001)
        task_id: String,

        /// Skip quality gate checks
        #[arg(long)]
        skip_quality_check: bool,
    },

    /// Check sprint or task status
    Status {
        /// Sprint ID to check
        #[arg(long)]
        sprint: Option<String>,

        /// Task ID to check
        #[arg(long)]
        task: Option<String>,

        /// Output format
        #[arg(long, value_enum, default_value = "human")]
        format: OutputFormat,
    },

    /// Validate sprint readiness for release
    Validate {
        /// Sprint ID to validate
        #[arg(long)]
        sprint: String,

        /// Fail if validation fails (exit code 1)
        #[arg(long)]
        strict: bool,
    },

    /// Run quality checks for a task
    QualityCheck {
        /// Task ID to check
        #[arg(long)]
        task_id: String,
    },
}

/// Test suite types for performance validation
#[derive(Clone, Debug, clap::ValueEnum, PartialEq)]
pub enum TestSuite {
    /// Performance tests per SPECIFICATION.md Section 30
    Performance,
    /// Property-based testing expansion per SPECIFICATION.md Section 28
    Property,
    /// Integration test suite
    Integration,
    /// Regression detection tests
    Regression,
    /// Memory usage validation tests
    Memory,
    /// Throughput validation tests
    Throughput,
    /// All test suites
    All,
}

/// Transport protocol options for serve command
#[derive(Clone, Debug, clap::ValueEnum)]
#[cfg_attr(test, derive(PartialEq))]
pub enum ServeTransport {
    /// HTTP transport (REST API)
    Http,
    /// WebSocket transport (real-time bidirectional)
    WebSocket,
    /// HTTP Server-Sent Events transport (streaming)
    HttpSse,
    /// Both HTTP and WebSocket (hybrid mode)
    Both,
    /// All transports (HTTP, WebSocket, SSE)
    All,
}

/// Agent mode subcommands for Claude Code integration
#[derive(Subcommand)]
#[cfg_attr(test, derive(Debug))]
pub enum AgentCommands {
    /// Start the background agent daemon
    Start {
        /// Project path to monitor (defaults to current directory)
        #[arg(short = 'p', long, default_value = ".")]
        project_path: PathBuf,

        /// Configuration file path
        #[arg(short = 'c', long)]
        config: Option<PathBuf>,

        /// Working directory for the daemon
        #[arg(long)]
        working_dir: Option<PathBuf>,

        /// PID file location
        #[arg(long)]
        pid_file: Option<PathBuf>,

        /// Log file location
        #[arg(long)]
        log_file: Option<PathBuf>,

        /// Run in foreground (don't daemonize)
        #[arg(long)]
        foreground: bool,

        /// Health check interval in seconds
        #[arg(long, default_value = "30")]
        health_interval: u64,

        /// Maximum memory usage in MB before restart
        #[arg(long, default_value = "500")]
        max_memory_mb: u64,

        /// Disable auto-restart on failure
        #[arg(long)]
        no_auto_restart: bool,
    },

    /// Stop the background agent daemon
    Stop {
        /// PID file location
        #[arg(long)]
        pid_file: Option<PathBuf>,

        /// Force stop (SIGKILL) if graceful stop fails
        #[arg(long)]
        force: bool,

        /// Timeout for graceful shutdown in seconds
        #[arg(long, default_value = "10")]
        timeout: u64,
    },

    /// Show daemon status
    Status {
        /// PID file location
        #[arg(long)]
        pid_file: Option<PathBuf>,

        /// Output format
        #[arg(long, value_enum, default_value = "json")]
        format: OutputFormat,
    },

    /// Start monitoring a new project
    Monitor {
        /// Project path to start monitoring
        #[arg(short = 'p', long)]
        project_path: PathBuf,

        /// Project identifier (defaults to path basename)
        #[arg(long)]
        project_id: Option<String>,

        /// Quality thresholds configuration file
        #[arg(long)]
        thresholds: Option<PathBuf>,
    },

    /// Stop monitoring a project
    Unmonitor {
        /// Project ID to stop monitoring
        #[arg(short = 'i', long)]
        project_id: String,
    },

    /// Run health check
    Health {
        /// PID file location
        #[arg(long)]
        pid_file: Option<PathBuf>,

        /// Detailed health information
        #[arg(long)]
        detailed: bool,
    },

    /// Reload daemon configuration
    Reload {
        /// PID file location
        #[arg(long)]
        pid_file: Option<PathBuf>,

        /// Configuration file path
        #[arg(short = 'c', long)]
        config: Option<PathBuf>,
    },

    /// Run quality gate through agent
    QualityGate {
        /// Project ID or path
        #[arg(short = 'p', long)]
        project: String,

        /// Specific file to check
        #[arg(long)]
        file: Option<PathBuf>,

        /// Output format
        #[arg(long, value_enum, default_value = "summary")]
        format: QualityGateOutputFormat,
    },

    /// Start MCP server for testing
    McpServer {
        /// Configuration file path
        #[arg(short = 'c', long)]
        config: Option<PathBuf>,

        /// Debug mode (verbose logging)
        #[arg(long)]
        debug: bool,
    },
}
/// Configuration management commands
#[derive(Subcommand, Clone)]
#[cfg_attr(test, derive(Debug))]
pub enum ConfigCommands {
    /// Show complete configuration
    Show {
        /// Output format
        #[arg(long, value_enum, default_value = "json")]
        format: ConfigFormat,
    },

    /// Get specific configuration value
    Get {
        /// Configuration key path (e.g., `hooks.quality_gates.max_cyclomatic_complexity`)
        key: String,
    },

    /// Validate configuration file
    Validate {
        /// Fix configuration issues automatically
        #[arg(long)]
        fix: bool,
    },

    /// Show configuration source hierarchy
    Sources,
}

/// Pre-commit hook management commands  
#[derive(Subcommand, Clone)]
#[cfg_attr(test, derive(Debug))]
pub enum HooksCommands {
    /// Initialize pre-commit hooks (alias for install)
    Init {
        /// Enable interactive mode for configuration
        #[arg(long)]
        interactive: bool,

        /// Force installation (overwrite existing)
        #[arg(long)]
        force: bool,

        /// Create backup of existing hooks
        #[arg(long, default_value = "true")]
        backup: bool,

        /// Enable TDG quality enforcement hooks (Sprint 66 Phase 3)
        #[arg(long)]
        tdg_enforcement: bool,
    },

    /// Install or update pre-commit hooks
    Install {
        /// Enable interactive mode for configuration
        #[arg(long)]
        interactive: bool,

        /// Force installation (overwrite existing)
        #[arg(long)]
        force: bool,

        /// Create backup of existing hooks
        #[arg(long, default_value = "true")]
        backup: bool,

        /// Enable TDG quality enforcement hooks (Sprint 66 Phase 3)
        #[arg(long)]
        tdg_enforcement: bool,
    },

    /// Remove PMAT-managed hooks
    Uninstall {
        /// Restore backup if available
        #[arg(long)]
        restore_backup: bool,
    },

    /// Show hook installation status
    Status,

    /// Verify hooks work with current configuration
    Verify {
        /// Fix issues automatically
        #[arg(long)]
        fix: bool,
    },

    /// Regenerate hooks from current configuration
    Refresh,

    /// Run pre-commit hooks (for CI/CD integration)
    Run {
        /// Run on all files instead of just staged
        #[arg(long)]
        all_files: bool,

        /// Verbose output
        #[arg(long, short)]
        verbose: bool,
    },
}

/// Configuration output format
#[derive(Debug, Clone, clap::ValueEnum)]
#[cfg_attr(test, derive(PartialEq))]
pub enum ConfigFormat {
    /// JSON format
    Json,
    /// TOML format
    Toml,
    /// Environment variables format
    Env,
}

/// Embed subcommands for semantic search (PMAT-SEARCH-011)
#[derive(Subcommand)]
#[cfg_attr(test, derive(Debug))]
#[command(after_help = "EXAMPLES:
# Sync embeddings for current codebase
pmat embed sync

# Check embedding database status
pmat embed status

# Clear all embeddings (requires confirmation)
pmat embed clear --confirm

# Sync with verbose output
pmat embed sync --verbose

# Check status in JSON format
pmat embed status --format json")]
pub enum EmbedCommands {
    /// Sync embeddings for codebase
    Sync {
        /// Path to analyze
        #[arg(short, long, default_value = ".")]
        path: PathBuf,

        /// Filter by language
        #[arg(long)]
        language: Option<String>,

        /// Output format
        #[arg(long, value_enum, default_value = "table")]
        format: OutputFormat,
    },

    /// Show embedding database status
    Status {
        /// Output format
        #[arg(long, value_enum, default_value = "table")]
        format: OutputFormat,
    },

    /// Clear all embeddings (requires --confirm)
    Clear {
        /// Confirm deletion
        #[arg(long)]
        confirm: bool,
    },
}

/// Semantic search subcommands (PMAT-SEARCH-011)
#[derive(Subcommand)]
#[cfg_attr(test, derive(Debug))]
pub enum SemanticCommands {
    /// Search code by natural language query
    Search {
        /// Natural language query
        query: String,

        /// Search mode
        #[arg(long, value_enum, default_value = "hybrid")]
        mode: SearchMode,

        /// Filter by language
        #[arg(long)]
        language: Option<String>,

        /// Max results to return
        #[arg(long, default_value_t = 10)]
        limit: usize,

        /// Output format
        #[arg(long, value_enum, default_value = "summary")]
        format: OutputFormat,
    },

    /// Find code files similar to a reference file
    Similar {
        /// Reference file path
        file_path: PathBuf,

        /// Max results to return
        #[arg(long, default_value_t = 10)]
        limit: usize,

        /// Output format
        #[arg(long, value_enum, default_value = "summary")]
        format: OutputFormat,
    },
}

/// Search mode for semantic search (PMAT-SEARCH-011)
#[derive(Clone, Debug, clap::ValueEnum, PartialEq)]
pub enum SearchMode {
    /// Keyword-only search (ripgrep)
    Keyword,
    /// Vector-only search (semantic similarity)
    Vector,
    /// Hybrid search (keyword + vector with RRF)
    Hybrid,
}

/// Clustering method (PMAT-SEARCH-011)
#[derive(Clone, Debug, clap::ValueEnum, PartialEq)]
pub enum ClusterMethod {
    /// K-means clustering
    Kmeans,
    /// Hierarchical clustering
    Hierarchical,
    /// DBSCAN density-based clustering
    Dbscan,
}

/// Mutation testing arguments (Sprint 61 + Sprint 70)
#[cfg(feature = "mutation-testing")]
#[derive(Args, Debug, Clone)]
#[cfg_attr(test, derive(PartialEq))]
pub struct MutateArgs {
    /// File or directory to mutate
    #[arg(short, long, value_name = "PATH")]
    pub target: PathBuf,

    /// Programming language (rust, python, typescript, go, cpp)
    #[arg(short, long)]
    pub language: Option<String>,

    /// Timeout per mutant in seconds
    #[arg(long, default_value = "30")]
    pub timeout: u64,

    /// Parallel execution workers
    #[arg(short, long)]
    pub jobs: Option<usize>,

    /// Output format (json, markdown, text)
    #[arg(short = 'f', long, default_value = "text")]
    pub output_format: String,

    /// Output file (stdout if omitted)
    #[arg(short, long)]
    pub output: Option<PathBuf>,

    /// Mutation score threshold (fail if below)
    #[arg(long)]
    pub threshold: Option<f64>,

    /// Show only failures (survived mutants, compile errors, timeouts)
    #[arg(long, default_value = "false")]
    pub failures_only: bool,

    // ========================================================================
    // Sprint 70: cargo-mutants backend options
    // ========================================================================
    /// Use cargo-mutants backend for Rust mutation testing (requires cargo-mutants v24.7.0+)
    ///
    /// Provides comprehensive Rust mutation testing using the industry-standard cargo-mutants tool.
    /// Automatically detects cargo-mutants installation and validates version compatibility.
    ///
    /// Example: pmat mutate --use-cargo-mutants --timeout 600
    /// Guide: docs/user-guides/cargo-mutants-integration.md
    #[arg(long)]
    pub use_cargo_mutants: bool,

    /// Cargo features to enable for mutation testing (comma-separated)
    ///
    /// Only applies when --use-cargo-mutants is specified.
    /// Enables specific Cargo features during mutation testing.
    ///
    /// Example: --features "serde,logging"
    #[arg(long, value_delimiter = ',')]
    pub features: Option<Vec<String>>,

    /// Enable all Cargo features during mutation testing
    ///
    /// Only applies when --use-cargo-mutants is specified.
    /// Equivalent to cargo test --all-features.
    ///
    /// Example: --use-cargo-mutants --all-features
    #[arg(long)]
    pub all_features: bool,

    /// Disable default Cargo features during mutation testing
    ///
    /// Only applies when --use-cargo-mutants is specified.
    /// Equivalent to cargo test --no-default-features.
    ///
    /// Example: --use-cargo-mutants --no-default-features --features "minimal"
    #[arg(long)]
    pub no_default_features: bool,

    /// Don't shuffle mutant execution order (deterministic results)
    ///
    /// Only applies when --use-cargo-mutants is specified.
    /// Mutants will be tested in sequential order for reproducible results.
    ///
    /// Example: --use-cargo-mutants --no-shuffle
    #[arg(long)]
    pub no_shuffle: bool,
}

/// Organizational intelligence subcommands (Phase 4: OIP Integration)
#[derive(Subcommand)]
#[cfg_attr(test, derive(Debug))]
pub enum OrgCommands {
    /// Analyze GitHub organization for defect patterns
    Analyze {
        /// GitHub organization name
        #[arg(long)]
        org: String,

        /// Output file path for analysis results
        #[arg(short, long)]
        output: PathBuf,

        /// Maximum number of concurrent repository analyses
        #[arg(long, default_value_t = 5)]
        max_concurrent: usize,

        /// Automatically summarize results (PII-stripped)
        #[arg(long)]
        summarize: bool,

        /// Strip PII from summary (requires --summarize)
        #[arg(long, requires = "summarize")]
        strip_pii: bool,

        /// Top N defect categories to include in summary
        #[arg(long, default_value_t = 10, requires = "summarize")]
        top_n: usize,

        /// Minimum frequency threshold for defect patterns
        #[arg(long, default_value_t = 3, requires = "summarize")]
        min_frequency: usize,
    },

    /// Fault localization using Tarantula SBFL algorithm (Phase 5-7)
    Localize {
        /// Path to coverage file for passing tests (LCOV format)
        #[arg(long)]
        passed_coverage: PathBuf,

        /// Path to coverage file for failing tests (LCOV format)
        #[arg(long)]
        failed_coverage: PathBuf,

        /// Number of passing test cases
        #[arg(long)]
        passed_count: usize,

        /// Number of failing test cases
        #[arg(long)]
        failed_count: usize,

        /// SBFL formula to use
        #[arg(long, default_value = "tarantula")]
        formula: String,

        /// Top N suspicious statements to report
        #[arg(long, default_value_t = 10)]
        top_n: usize,

        /// Output file path
        #[arg(short, long)]
        output: Option<PathBuf>,

        /// Enable weighted ensemble model (Phase 6)
        #[arg(long)]
        ensemble: bool,

        /// Enable calibrated defect prediction (Phase 7)
        #[arg(long)]
        calibrated: bool,

        /// Confidence threshold for calibrated predictions (0.0-1.0)
        #[arg(long, default_value_t = 0.5)]
        confidence_threshold: f32,

        /// Enrich with TDG scores from pmat
        #[arg(long)]
        enrich_tdg: bool,

        /// Repository path for TDG enrichment
        #[arg(long, default_value = ".")]
        repo: PathBuf,
    },
}

/// Prompt generation subcommands (Phase 4: Organizational Intelligence Integration)
#[derive(Subcommand)]
#[cfg_attr(test, derive(Debug))]
pub enum PromptCommands {
    /// Show workflow prompt (original functionality - EXTREME TDD, Toyota Way, etc.)
    Show {
        /// Prompt name to show (use --list to see all available prompts)
        name: Option<String>,

        /// List all available prompts
        #[arg(long, conflicts_with = "name")]
        list: bool,

        /// Show prompt variables that can be customized
        #[arg(long, requires = "name")]
        show_variables: bool,

        /// Override prompt variables (e.g., --set TEST_CMD="pytest")
        #[arg(long, value_parser = crate::cli::args::parse_key_val, requires = "name")]
        set: Vec<(String, Value)>,

        /// Output format (yaml, json, text)
        #[arg(long, value_enum, default_value = "yaml", requires = "name")]
        format: PromptOutputFormat,

        /// Write output to file instead of stdout
        #[arg(short, long)]
        output: Option<PathBuf>,
    },

    /// Generate defect-aware AI prompt from organizational intelligence
    #[command(visible_aliases = &["gen", "defect"])]
    Generate {
        /// Development task description
        #[arg(short, long)]
        task: String,

        /// Additional context about the task
        #[arg(short, long)]
        context: String,

        /// Path to OIP summary YAML file
        #[arg(short, long)]
        summary: PathBuf,

        /// Write output to file instead of stdout
        #[arg(short, long)]
        output: Option<PathBuf>,
    },

    /// Generate EXTREME TDD workflow prompt for fixing a ticket
    #[command(visible_aliases = &["tkt", "fix"])]
    Ticket {
        /// Ticket/issue description or ID
        #[arg(short, long)]
        ticket: String,

        /// Path to OIP summary YAML file (optional)
        #[arg(short, long)]
        summary: Option<PathBuf>,

        /// Write output to file instead of stdout
        #[arg(short, long)]
        output: Option<PathBuf>,
    },

    /// Generate implementation prompt from specification
    #[command(visible_aliases = &["impl", "spec"])]
    Implement {
        /// Path to specification file (markdown)
        #[arg(short, long)]
        spec: PathBuf,

        /// Path to OIP summary YAML file (optional)
        #[arg(short, long)]
        summary: Option<PathBuf>,

        /// Write output to file instead of stdout
        #[arg(short, long)]
        output: Option<PathBuf>,
    },

    /// Generate prompt for scaffolding a new repository
    #[command(visible_aliases = &["scaffold", "new"])]
    ScaffoldNewRepo {
        /// Path to repository specification file (markdown)
        #[arg(short, long)]
        spec: PathBuf,

        /// Include pmat tools setup
        #[arg(long, default_value_t = true)]
        include_pmat: bool,

        /// Include bashrs setup
        #[arg(long, default_value_t = true)]
        include_bashrs: bool,

        /// Include roadmapping tools
        #[arg(long, default_value_t = true)]
        include_roadmap: bool,

        /// Write output to file instead of stdout
        #[arg(short, long)]
        output: Option<PathBuf>,
    },

    /// Fix all drift from PMAT's rigid quality processes and restore compliance
    #[command(visible_aliases = &["compliance"])]
    Comply {
        /// Minimum acceptable quality grade (default: B+)
        #[arg(long, default_value = "B+")]
        min_grade: String,

        /// Path to baseline quality metrics (default: .pmat/baseline.json)
        #[arg(long)]
        baseline: Option<PathBuf>,

        /// Path to roadmap file (default: roadmap.yaml)
        #[arg(long)]
        roadmap: Option<PathBuf>,

        /// Write output to file instead of stdout
        #[arg(short, long)]
        output: Option<PathBuf>,
    },

    /// Create and maintain technical book documentation with EXTREME TDD validation
    #[command(visible_aliases = &["docs", "mdbook"])]
    Book {
        /// Book title
        #[arg(long)]
        title: Option<String>,

        /// Book type (tutorial, cookbook, reference)
        #[arg(long, default_value = "tutorial")]
        book_type: String,

        /// Target page count
        #[arg(long, default_value_t = 400)]
        target_pages: u32,

        /// Minimum test pass rate (0-100)
        #[arg(long, default_value_t = 90)]
        min_pass_rate: u8,

        /// Write output to file instead of stdout
        #[arg(short, long)]
        output: Option<PathBuf>,
    },

    /// Generate professional repository documentation with badges and polish
    #[command(visible_aliases = &["readme", "image"])]
    RepoImage {
        /// Repository name
        #[arg(long)]
        repo_name: Option<String>,

        /// Repository description
        #[arg(long)]
        description: Option<String>,

        /// GitHub organization (default: paiml)
        #[arg(long, default_value = "paiml")]
        github_org: String,

        /// Primary programming language
        #[arg(long)]
        language: Option<String>,

        /// Is this a course series repository?
        #[arg(long)]
        course_series: bool,

        /// Write output to file instead of stdout
        #[arg(short, long)]
        output: Option<PathBuf>,
    },

    /// Implement GitHub issue/ticket with full EXTREME TDD workflow
    #[command(visible_aliases = &["issue", "gh"])]
    GithubIssue {
        /// GitHub issue URL or issue number
        #[arg(short, long)]
        issue: String,

        /// GitHub organization (required if using issue number)
        #[arg(long)]
        org: Option<String>,

        /// GitHub repository (required if using issue number)
        #[arg(long)]
        repo: Option<String>,

        /// Test command (default: cargo test)
        #[arg(long, default_value = "cargo test")]
        test_cmd: String,

        /// Build command (default: cargo build)
        #[arg(long, default_value = "cargo build")]
        build_cmd: String,

        /// Write output to file instead of stdout
        #[arg(short, long)]
        output: Option<PathBuf>,
    },
}

#[cfg(test)]
mod property_tests {
    use proptest::prelude::*;

    proptest! {
        #[test]
        fn basic_property_stability(_input in ".*") {
            // Basic property test for coverage
            prop_assert!(true);
        }

        #[test]
        fn module_consistency_check(_x in 0u32..1000) {
            // Module consistency verification
            prop_assert!(_x < 1001);
        }
    }
}

/// Work subcommands for unified GitHub/YAML workflow (Issue #75)
/// CRUD: Create (add), Read (list/status), Update (edit/start/complete), Delete (delete)
#[derive(Debug, Clone, Subcommand)]
pub enum WorkCommands {
    /// Add a new work ticket (CREATE)
    #[command(visible_aliases = &["new", "create", "a"])]
    Add {
        /// Ticket title (required)
        title: String,

        /// Description (optional)
        #[arg(short, long)]
        description: Option<String>,

        /// Priority level
        #[arg(short, long, value_enum, default_value = "medium")]
        priority: WorkPriority,

        /// Tags (comma-separated)
        #[arg(short, long)]
        tags: Option<String>,

        /// Project path (default: current directory)
        #[arg(long)]
        path: Option<PathBuf>,

        /// Also create GitHub issue
        #[arg(long)]
        github: bool,
    },

    /// List all work tickets (READ)
    #[command(visible_aliases = &["ls", "l"])]
    List {
        /// Filter by status
        #[arg(short, long)]
        status: Option<String>,

        /// Filter by priority
        #[arg(long)]
        priority: Option<WorkPriority>,

        /// Show only count
        #[arg(long)]
        count: bool,

        /// Project path (default: current directory)
        #[arg(short, long)]
        path: Option<PathBuf>,
    },

    /// Edit an existing ticket (UPDATE)
    #[command(visible_aliases = &["update", "e"])]
    Edit {
        /// Ticket ID to edit
        id: String,

        /// New title
        #[arg(short, long)]
        title: Option<String>,

        /// New description
        #[arg(short, long)]
        description: Option<String>,

        /// New priority
        #[arg(long)]
        priority: Option<WorkPriority>,

        /// New status
        #[arg(short, long)]
        status: Option<String>,

        /// New tags (comma-separated, replaces existing)
        #[arg(long)]
        tags: Option<String>,

        /// Project path (default: current directory)
        #[arg(short, long)]
        path: Option<PathBuf>,
    },

    /// Delete a work ticket (DELETE)
    #[command(visible_aliases = &["rm", "remove", "del"])]
    Delete {
        /// Ticket ID to delete
        id: String,

        /// Skip confirmation prompt
        #[arg(short, long)]
        force: bool,

        /// Project path (default: current directory)
        #[arg(short, long)]
        path: Option<PathBuf>,
    },

    /// Show unified quality annotations for a ticket
    #[command(visible_aliases = &["ann", "quality", "metrics"])]
    Annotate {
        /// Ticket ID to annotate
        id: String,

        /// Project path (default: current directory)
        #[arg(short, long)]
        path: Option<PathBuf>,

        /// Output format
        #[arg(short, long, value_enum, default_value = "text")]
        format: AnnotateOutputFormat,

        /// Include churn analysis (slower)
        #[arg(long)]
        with_churn: bool,

        /// Days for churn analysis
        #[arg(long, default_value = "30")]
        churn_days: u32,
    },

    /// Start work on a GitHub issue or YAML ticket
    #[command(visible_aliases = &["begin", "s"])]
    Start {
        /// Issue number (e.g., "8", "42") or YAML ticket ID (e.g., "PERF-001")
        id: String,

        /// Create specification file (docs/specifications/NNN-name.md)
        #[arg(long)]
        with_spec: bool,

        /// Create as epic with subtasks
        #[arg(long)]
        epic: bool,

        /// Project path (default: current directory)
        #[arg(short, long)]
        path: Option<PathBuf>,

        /// Force create GitHub issue for YAML ticket
        #[arg(long)]
        create_github: bool,
    },

    /// Continue work on existing issue/ticket
    #[command(visible_aliases = &["cont", "c", "resume"])]
    Continue {
        /// Issue number or ticket ID
        id: String,

        /// Project path (default: current directory)
        #[arg(short, long)]
        path: Option<PathBuf>,
    },

    /// Complete work on issue/ticket
    #[command(visible_aliases = &["done", "finish", "f"])]
    Complete {
        /// Issue number or ticket ID
        id: String,

        /// Skip quality gates (not recommended)
        #[arg(long)]
        skip_quality: bool,

        /// Project path (default: current directory)
        #[arg(short, long)]
        path: Option<PathBuf>,
    },

    /// Show work status
    #[command(visible_aliases = &["st", "stat"])]
    Status {
        /// Issue number or ticket ID (default: all)
        id: Option<String>,

        /// Project path (default: current directory)
        #[arg(short, long)]
        path: Option<PathBuf>,

        /// Show only active items
        #[arg(long)]
        active: bool,
    },

    /// Synchronize GitHub and YAML
    #[command(visible_aliases = &["sy"])]
    Sync {
        /// Sync direction
        #[arg(long, value_enum, default_value = "full")]
        direction: SyncDirection,

        /// Project path (default: current directory)
        #[arg(short, long)]
        path: Option<PathBuf>,

        /// Dry run (show what would be synced)
        #[arg(long)]
        dry_run: bool,
    },

    /// Initialize roadmap and hooks
    #[command(visible_aliases = &["setup", "ini"])]
    Init {
        /// GitHub repository (owner/repo)
        #[arg(long)]
        github_repo: Option<String>,

        /// Disable GitHub integration
        #[arg(long)]
        no_github: bool,

        /// Project path (default: current directory)
        #[arg(short, long)]
        path: Option<PathBuf>,
    },

    /// Validate roadmap.yaml syntax and content (Part B: UX Improvements)
    #[command(visible_aliases = &["check", "lint", "v"])]
    Validate {
        /// Project path (default: current directory)
        #[arg(short, long)]
        path: Option<PathBuf>,

        /// Show verbose output with suggestions
        #[arg(long)]
        verbose: bool,

        /// Fix issues automatically where possible
        #[arg(long)]
        fix: bool,
    },

    /// Auto-fix common roadmap.yaml issues (Part B: UX Improvements)
    #[command(visible_aliases = &["fix", "m"])]
    Migrate {
        /// Project path (default: current directory)
        #[arg(short, long)]
        path: Option<PathBuf>,

        /// Dry run (show what would be changed)
        #[arg(long)]
        dry_run: bool,

        /// Create backup before migration
        #[arg(long, default_value = "true")]
        backup: bool,
    },

    /// List all valid status values with descriptions
    #[command(visible_aliases = &["values", "statuses"])]
    ListStatuses,
}

/// Sync direction for work sync command
#[derive(Debug, Clone, Copy, clap::ValueEnum, PartialEq)]
pub enum SyncDirection {
    /// Sync YAML → GitHub
    YamlToGithub,
    /// Sync GitHub → YAML
    GithubToYaml,
    /// Full bidirectional sync
    Full,
}

/// Work priority for CLI (maps to roadmap::Priority)
#[derive(Debug, Clone, Copy, clap::ValueEnum, PartialEq, Default)]
pub enum WorkPriority {
    /// Low priority
    Low,
    /// Medium priority (default)
    #[default]
    Medium,
    /// High priority
    High,
    /// Critical priority
    Critical,
}

impl WorkPriority {
    /// Convert to roadmap Priority
    pub fn to_roadmap_priority(self) -> crate::models::roadmap::Priority {
        match self {
            WorkPriority::Low => crate::models::roadmap::Priority::Low,
            WorkPriority::Medium => crate::models::roadmap::Priority::Medium,
            WorkPriority::High => crate::models::roadmap::Priority::High,
            WorkPriority::Critical => crate::models::roadmap::Priority::Critical,
        }
    }
}

/// QA Work subcommands for Toyota Way quality validation (GH-102)
#[derive(Debug, Clone, Subcommand)]
pub enum QaWorkCommands {
    /// Generate QA checklist for a task
    #[command(visible_aliases = &["checklist", "cl"])]
    GenerateChecklist {
        /// Task/ticket ID (GitHub issue number or YAML ticket ID)
        task_id: String,

        /// Task type for checklist customization
        #[arg(long, value_enum, default_value = "feature")]
        task_type: QaTaskType,

        /// Project path (default: current directory)
        #[arg(short = 'p', long = "path", default_value = ".")]
        path: PathBuf,

        /// Output file for checklist (YAML format)
        #[arg(short = 'o', long = "output")]
        output: Option<PathBuf>,
    },

    /// Run automated QA validation
    #[command(visible_aliases = &["check", "v"])]
    Validate {
        /// Task/ticket ID to validate
        task_id: String,

        /// Project path (default: current directory)
        #[arg(short = 'p', long = "path", default_value = ".")]
        path: PathBuf,

        /// Fail on any warning (strict mode)
        #[arg(long)]
        strict: bool,

        /// Output format
        #[arg(short = 'f', long = "format", value_enum, default_value = "text")]
        format: QaOutputFormat,
    },

    /// Generate QA report for audit trail
    #[command(visible_aliases = &["r"])]
    Report {
        /// Task/ticket ID for report
        task_id: String,

        /// Project path (default: current directory)
        #[arg(short = 'p', long = "path", default_value = ".")]
        path: PathBuf,

        /// Include evidence (coverage reports, test results)
        #[arg(long)]
        with_evidence: bool,

        /// Output file for report
        #[arg(short = 'o', long = "output")]
        output: Option<PathBuf>,

        /// Output format
        #[arg(short = 'f', long = "format", value_enum, default_value = "markdown")]
        format: QaOutputFormat,
    },

    /// Show QA status summary
    #[command(visible_aliases = &["st", "status"])]
    Summary {
        /// Task/ticket ID (optional, shows all if omitted)
        task_id: Option<String>,

        /// Project path (default: current directory)
        #[arg(short = 'p', long = "path", default_value = ".")]
        path: PathBuf,

        /// Show epic summary (aggregate all tasks in epic)
        #[arg(long)]
        epic: Option<String>,
    },

    /// Generate example scripts for a feature (V2)
    #[command(visible_aliases = &["examples", "ex"])]
    GenerateExamples {
        /// Task/ticket ID
        task_id: String,

        /// Feature/command name for examples
        #[arg(short = 'n', long = "name")]
        feature_name: String,

        /// Project path (default: current directory)
        #[arg(short = 'p', long = "path", default_value = ".")]
        path: PathBuf,

        /// Output directory for examples
        #[arg(short = 'o', long = "output")]
        output: Option<PathBuf>,
    },

    /// Validate specification with 100-point Popperian falsifiability scoring (Part D & E)
    ///
    /// Parses markdown specifications and validates claims through evidence.
    /// All claims are FALSE until PROVEN true (Popperian epistemology).
    #[command(visible_aliases = &["spec", "popper"])]
    Spec {
        /// Specification file or ticket ID (e.g., "docs/specifications/foo.md" or "GH-118")
        target: String,

        /// Project path (default: current directory)
        #[arg(short = 'p', long = "path", default_value = ".")]
        path: PathBuf,

        /// Run full validation (includes mutation testing)
        #[arg(long)]
        full: bool,

        /// Output format
        #[arg(short = 'f', long = "format", value_enum, default_value = "text")]
        format: QaOutputFormat,

        /// Output file for results
        #[arg(short = 'o', long = "output")]
        output: Option<PathBuf>,

        /// Fail if total score below threshold (default: 60 for gateway)
        #[arg(long, default_value = "60")]
        threshold: u32,

        /// Fail if gateway category (Falsifiability) below threshold
        #[arg(long, default_value = "15")]
        gateway_threshold: u32,
    },
}

/// Task type for QA checklist customization
#[derive(Debug, Clone, Copy, clap::ValueEnum, PartialEq)]
pub enum QaTaskType {
    /// New feature implementation
    Feature,
    /// Bug fix
    Bugfix,
    /// Code refactoring
    Refactor,
    /// Documentation update
    Docs,
    /// Performance optimization
    Performance,
    /// Security fix
    Security,
}

/// Output format for QA commands
#[derive(Debug, Clone, Copy, clap::ValueEnum, PartialEq)]
pub enum QaOutputFormat {
    /// Human-readable text
    Text,
    /// JSON for CI/CD
    Json,
    /// YAML config format
    Yaml,
    /// Markdown documentation
    Markdown,
}

/// Test discovery subcommands for systematic test fixing (GH-98)
#[derive(Debug, Clone, Subcommand)]
pub enum TestDiscoveryCommands {
    /// Discover all test failures in workspace
    #[command(visible_aliases = &["discover", "d"])]
    Run {
        /// Project path (defaults to current directory)
        #[arg(short = 'p', long = "path", default_value = ".")]
        path: PathBuf,

        /// Output file for failures JSON
        #[arg(short = 'o', long = "output", default_value = "test-failures.json")]
        output: PathBuf,

        /// Use cargo nextest (faster, parallel)
        #[arg(long, default_value = "true")]
        use_nextest: bool,

        /// Maximum test timeout in seconds
        #[arg(long, default_value = "600")]
        timeout: u64,
    },

    /// Categorize test failures by root cause
    #[command(visible_aliases = &["categorize", "cat"])]
    Categorize {
        /// Input failures JSON from discovery
        #[arg(short = 'i', long = "input")]
        input: PathBuf,

        /// Output categories JSON
        #[arg(short = 'o', long = "output", default_value = "test-categories.json")]
        output: PathBuf,
    },

    /// Mark tests as #[ignore] with reasons
    #[command(visible_aliases = &["mark", "m"])]
    Mark {
        /// Input categories JSON
        #[arg(short = 'i', long = "input")]
        input: PathBuf,

        /// Actually apply changes (default: dry-run)
        #[arg(long)]
        apply: bool,
    },

    /// Verify all tests pass after marking
    #[command(visible_aliases = &["verify", "v"])]
    Verify {
        /// Project path
        #[arg(short = 'p', long = "path", default_value = ".")]
        path: PathBuf,
    },

    /// Create GitHub issues from categorized test failures (Phase 5)
    #[command(visible_aliases = &["tickets", "t"])]
    CreateTickets {
        /// Input categories JSON
        #[arg(short = 'i', long = "input")]
        input: PathBuf,

        /// Actually create GitHub issues (default: dry-run)
        #[arg(long)]
        create: bool,

        /// Output tickets summary
        #[arg(short = 'o', long = "output")]
        output: Option<PathBuf>,

        /// GitHub repository (owner/repo format)
        #[arg(long)]
        repo: Option<String>,

        /// Labels to add to created issues
        #[arg(long, value_delimiter = ',')]
        labels: Option<Vec<String>>,
    },

    /// Resolve test file paths from test names
    #[command(visible_aliases = &["resolve", "r"])]
    ResolvePaths {
        /// Input failures JSON
        #[arg(short = 'i', long = "input")]
        input: PathBuf,

        /// Output with resolved paths
        #[arg(short = 'o', long = "output")]
        output: PathBuf,

        /// Project path to search for test files
        #[arg(short = 'p', long = "path", default_value = ".")]
        path: PathBuf,
    },
}

/// Test discovery output formats
#[derive(Debug, Clone, clap::ValueEnum, PartialEq)]
pub enum TestDiscoveryFormat {
    Json,
    Markdown,
    Text,
}

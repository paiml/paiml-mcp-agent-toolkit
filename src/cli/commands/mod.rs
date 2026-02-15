#![cfg_attr(coverage_nightly, coverage(off))]
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
    QualityProfile, QueryOutputFormat, RefactorAutoOutputFormat, RefactorDocsOutputFormat,
    RefactorMode, RefactorOutputFormat, RepoScoreOutputFormat, ReportOutputFormat,
    SatdOutputFormat, SatdSeverity, SearchScope, SymbolTableOutputFormat, SymbolTypeFilter,
    TdgOutputFormat, VerificationMethodFilter, WasmOutputFormat,
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
    about = "PMAT - Professional Multi-language Analysis Toolkit for code quality, complexity, and technical debt",
    version,
    long_about = None,
    after_help = "EXAMPLES:
# Analyze code complexity
pmat analyze complexity --path .

# Calculate Technical Debt Grade (TDG)
pmat tdg .

# Find technical debt markers (TODO, FIXME, HACK)
pmat analyze satd --path .

# Find dead code
pmat analyze dead-code --path .

# Generate AI-ready project context
pmat context --format llm-optimized

# Run quality gates
pmat quality-gate

# Calculate repository health score
pmat repo-score

# Start background agent daemon
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

/// Color output mode
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

    /// Semantic code search with quality annotations (RAG-powered)
    ///
    /// Search functions by intent, returns results ranked by relevance and quality.
    /// Use instead of grep for quality-aware code discovery.
    #[command(visible_aliases = &["q", "search-code"])]
    Query {
        /// Natural language query (e.g., "error handling", "database connection")
        #[arg(default_value = "")]
        query: String,

        /// Maximum number of results
        #[arg(short, long, default_value_t = 10)]
        limit: usize,

        /// Minimum TDG grade filter (A, B, C, D, F)
        #[arg(long, value_name = "GRADE")]
        min_grade: Option<String>,

        /// Maximum cyclomatic complexity filter
        #[arg(long, value_name = "N")]
        max_complexity: Option<u32>,

        /// Filter by language (rust, typescript, python, etc.)
        #[arg(long)]
        language: Option<String>,

        /// Filter by file path pattern
        #[arg(long)]
        path: Option<String>,

        /// Project path to search
        #[arg(short = 'p', long, default_value = ".")]
        project_path: PathBuf,

        /// Output format
        #[arg(short, long, value_enum, default_value = "text")]
        format: QueryOutputFormat,

        /// Include full source code in results (source is shown by default; kept for backward compat)
        #[arg(long, hide = true)]
        include_source: bool,

        /// Build/rebuild the index before querying
        #[arg(long)]
        rebuild_index: bool,

        /// Exclude test functions from results
        #[arg(long)]
        exclude_tests: bool,

        /// Ranking strategy: relevance, pagerank, centrality, indegree
        #[arg(long, value_name = "STRATEGY")]
        rank_by: Option<String>,

        /// Minimum PageRank score filter (0.0-1.0)
        #[arg(long, value_name = "SCORE")]
        min_pagerank: Option<f32>,

        /// Additional project paths to include in search (can be specified multiple times)
        #[arg(long, value_name = "PATH")]
        include_project: Vec<PathBuf>,

        /// Enrich results with git churn data (commit count, volatility)
        #[arg(long)]
        churn: bool,

        /// Enrich results with duplicate code detection (clone count, similarity)
        #[arg(long)]
        duplicates: bool,

        /// Enrich results with entropy/pattern diversity metrics
        #[arg(long)]
        entropy: bool,

        /// Enrich results with batuta fault pattern annotations (mutation targets, boundary conditions)
        #[arg(long)]
        faults: bool,

        /// Enrich with test coverage data (line coverage, missed lines, ROI impact)
        #[arg(long)]
        coverage: bool,

        /// Filter to only uncovered/partially-covered functions (requires --coverage)
        #[arg(long)]
        uncovered_only: bool,

        /// Coverage baseline JSON file for delta comparison (shows coverage changes)
        #[arg(long, value_name = "FILE")]
        coverage_diff: Option<PathBuf>,

        /// Path to pre-existing LLVM coverage JSON file (avoids re-running test suite)
        #[arg(long, value_name = "FILE", env = "PMAT_COVERAGE_FILE")]
        coverage_file: Option<PathBuf>,

        /// Show top coverage gaps ranked by uncovered lines (no query string needed)
        #[arg(long)]
        coverage_gaps: bool,

        /// Include coverage(off) / dead-code / excluded functions in --coverage-gaps results
        #[arg(long)]
        include_excluded: bool,

        /// Filter by definition type (fn, struct, enum, trait, type)
        #[arg(long = "type", value_name = "TYPE")]
        definition_type: Option<String>,

        /// Show compact results without source code (source is shown by default)
        #[arg(long)]
        summary: bool,

        /// Include git commit history in search (RAG fusion)
        #[arg(long, short = 'G', visible_alias = "gh")]
        git_history: bool,

        /// Use regex pattern matching instead of semantic search (like rg -e)
        #[arg(long)]
        regex: bool,

        /// Literal string match, no semantic ranking (like rg -F)
        #[arg(long, visible_alias = "fixed-strings")]
        literal: bool,

        /// Raw file search only, skip index (like rg, searches all file types)
        #[arg(long)]
        raw: bool,

        /// Case-sensitive search (default: smart-case)
        #[arg(long)]
        case_sensitive: bool,

        /// Case-insensitive search (like rg -i)
        #[arg(short = 'i', long)]
        ignore_case: bool,

        /// Exclude results matching this content pattern (like grep -v)
        #[arg(long, value_name = "PATTERN")]
        exclude: Option<String>,

        /// Exclude results from files matching this glob (like rg --glob '!PATTERN')
        #[arg(long, value_name = "GLOB")]
        exclude_file: Option<String>,

        /// Show only file paths with matches (like rg -l / grep -l)
        #[arg(long)]
        files_with_matches: bool,

        /// Count matches per file (like rg -c / grep -c)
        #[arg(long)]
        count: bool,

        /// Show N lines of context after each match (like rg -A)
        #[arg(short = 'A', long = "after-context", value_name = "NUM")]
        after_context: Option<usize>,

        /// Show N lines of context before each match (like rg -B)
        #[arg(short = 'B', long = "before-context", value_name = "NUM")]
        before_context: Option<usize>,

        /// Show N lines of context around each match (like rg -C)
        #[arg(short = 'C', long = "context", value_name = "NUM")]
        context_lines: Option<usize>,

        /// Trace PTX dataflow across projects (emitter → loader → consumer)
        #[arg(long)]
        ptx_flow: bool,

        /// Run PTX quality diagnostics (register pressure, branch density, barriers)
        #[arg(long)]
        ptx_diagnostics: bool,
    },

    /// Analyze code metrics and patterns
    #[command(subcommand, visible_aliases = &["a", "an"])]
    Analyze(AnalyzeCommands),

    /// Quality-Driven Development (QDD) tool for creating and refactoring code with guaranteed quality
    #[command(subcommand, visible_aliases = &["qd"])]
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

    /// Validate README/documentation for factual accuracy and hallucinations
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

        /// Hardware capability TOML file
        /// If not specified, uses ~/.pmat/hardware.toml if it exists.
        /// Use `trueno --detect-hardware` to generate one.
        #[arg(long)]
        hardware: Option<PathBuf>,
    },

    /// Audit dependencies for Sovereign AI stack migration
    ///
    /// Analyzes Cargo.toml and identifies:
    /// - Heavy dependencies that bloat binary size
    /// - Dependencies replaceable with Sovereign stack (trueno, etc.)
    /// - Dev-only dependencies
    /// - Potentially removable dependencies
    #[command(name = "deps-audit", visible_aliases = &["deps", "audit-deps"])]
    DepsAudit {
        /// Project path (defaults to current directory)
        #[arg(short = 'p', long, default_value = ".")]
        path: PathBuf,

        /// Output format (text, json, yaml)
        #[arg(short = 'f', long, default_value = "text")]
        format: String,

        /// Show all dependencies including Core and Sovereign
        #[arg(long)]
        all: bool,

        /// Show 80/20 Pareto analysis: highest ROI removals (transitive deps saved / effort)
        #[arg(long)]
        pareto: bool,

        /// Sort results by metric: transitive (default), size, pagerank, name
        #[arg(long, default_value = "transitive")]
        sort_by: String,
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

        /// Include git context (commit SHA, branch, author)
        #[arg(long)]
        with_git_context: bool,

        /// Enable detailed explanation mode with function-level breakdown
        #[arg(long)]
        explain: bool,

        /// Complexity threshold for filtering functions in --explain mode
        #[arg(long, default_value = "10")]
        threshold: u32,

        /// Baseline git ref (commit/branch/tag) for progress tracking in --explain mode
        #[arg(long)]
        baseline: Option<String>,

        /// Use ML-based scoring (aprender LinearRegression)
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

    /// Run configurable quality gates on the current project
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

    /// Project maintenance commands (cleanup, validation, reports)
    #[command(visible_aliases = &["maint", "m"])]
    Maintain {
        /// Maintain subcommand
        #[command(subcommand)]
        command: MaintainCommands,
    },

    /// Pre-commit hook management and installation
    #[command(subcommand, visible_aliases = &["hook", "h"])]
    Hooks(HooksCommands),

    /// Manage semantic search embeddings for code search
    #[command(subcommand, visible_aliases = &["emb"])]
    Embed(EmbedCommands),

    /// Semantic code search using embeddings
    #[command(subcommand, visible_aliases = &["sem", "find-code"])]
    Semantic(SemanticCommands),

    /// Run mutation testing on specified files
    #[cfg(feature = "mutation-testing")]
    Mutate(MutateArgs),

    /// Time-travel debugging commands for execution traces
    #[command(visible_aliases = &["dbg"])]
    Debug {
        #[command(subcommand)]
        command: DebugCommands,
    },

    /// Unified GitHub/YAML workflow management
    #[command(visible_aliases = &["w"])]
    Work {
        #[command(subcommand)]
        command: WorkCommands,
    },

    /// Falsify claims in a work item, spec, or ticket against the codebase
    ///
    /// Runs the Popperian falsification protocol: extracts testable claims
    /// and attempts to find disconfirming evidence. Produces an immutable receipt.
    #[command(visible_aliases = &["falsify-spec"])]
    Falsify {
        /// Work item ID or spec file path to falsify
        target: String,

        /// Override specific falsification claims (requires --ticket)
        #[arg(long, value_delimiter = ',')]
        override_claims: Option<Vec<String>>,

        /// Ticket ID for override accountability (MANDATORY with --override-claims)
        #[arg(long)]
        ticket: Option<String>,

        /// Project path (default: current directory)
        #[arg(short, long)]
        path: Option<PathBuf>,

        /// Output format (text, json)
        #[arg(long)]
        format: Option<String>,

        /// Only show falsified claims
        #[arg(long, default_value_t = false)]
        failures_only: bool,

        /// Extract claims only, do not run falsification
        #[arg(long, default_value_t = false)]
        dry_run: bool,
    },

    /// QA validation after work completion with Toyota Way quality gates
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

    /// Specification management and validation
    #[command(name = "spec", visible_aliases = &["specification"])]
    Spec {
        #[command(subcommand)]
        command: SpecCommands,
    },

    /// PMAT compliance checking and migration system
    #[command(visible_aliases = &["compliance"])]
    Comply {
        #[command(subcommand)]
        command: ComplyCommands,
    },

    /// Autonomous continuous improvement (Toyota Way Kaizen)
    /// Scans for improvement opportunities, applies safe fixes, and reports remaining issues
    #[command(visible_aliases = &["improve"])]
    Kaizen {
        /// Project path to analyze
        #[arg(short = 'p', long, default_value = ".")]
        path: PathBuf,

        /// Scan only, do not apply any fixes
        #[arg(long)]
        dry_run: bool,

        /// Commit after applying fixes
        #[arg(long)]
        commit: bool,

        /// Push after committing
        #[arg(long)]
        push: bool,

        /// Spawn AI sub-agents for complex fixes
        #[arg(long)]
        agent: bool,

        /// Maximum concurrent AI sub-agents
        #[arg(long, default_value_t = 3)]
        max_agents: usize,

        /// Output format
        #[arg(short = 'f', long, value_enum, default_value = "text")]
        format: KaizenOutputFormat,

        /// Write output to file
        #[arg(short = 'o', long)]
        output: Option<PathBuf>,

        /// Skip clippy analysis
        #[arg(long)]
        skip_clippy: bool,

        /// Skip rustfmt analysis
        #[arg(long)]
        skip_fmt: bool,

        /// Skip comply analysis
        #[arg(long)]
        skip_comply: bool,

        /// Skip GitHub issues analysis
        #[arg(long)]
        skip_github: bool,

        /// Skip batuta defect pattern analysis
        #[arg(long)]
        skip_defects: bool,
    },

    /// Rust project diagnostics (20 checks across 5 categories)
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

    /// Systematic test discovery and fixing
    #[command(name = "test-discovery", visible_aliases = &["test-fix", "fix-tests"])]
    TestDiscovery {
        #[command(subcommand)]
        command: TestDiscoveryCommands,
    },

    /// Fault localization using Spectrum-Based Fault Localization (SBFL)
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

    /// Extract function boundaries from a single file (tree-sitter, no index)
    #[command(visible_aliases = &["ext"])]
    Extract {
        /// List all function/struct/enum/trait boundaries as JSON
        #[arg(long)]
        list: PathBuf,
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

// Misc command types - extracted for file health (CB-040)
include!("misc_commands.rs");

// Analyze commands - extracted for file health (CB-040)
include!("analyze_commands.rs");

// Quality commands (QDD, Enforce) - extracted for file health (CB-040)
include!("quality_commands.rs");

// Refactor and Scaffold commands - extracted for file health (CB-040)
include!("refactor_scaffold.rs");

// Roadmap and Agent commands - extracted for file health (CB-040)
include!("roadmap_agent.rs");

// Config and Hooks commands - extracted for file health (CB-040)
include!("config_hooks.rs");

// Semantic search commands - extracted for file health (CB-040)
include!("semantic_search.rs");

// Org and Prompt commands - extracted for file health (CB-040)
include!("org_prompt.rs");

// Work commands - extracted for file health (CB-040)
include!("work_commands.rs");

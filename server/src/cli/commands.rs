//! CLI command structures
//!
//! This module contains all the command structures used by the CLI for parsing
//! and executing commands. It's separated from the main CLI module to reduce complexity.

use crate::cli::diagnose::DiagnoseArgs;
use crate::cli::{
    AnalysisType, BigOOutputFormat, ComplexityOutputFormat, ComprehensiveOutputFormat,
    ContextFormat, DagType, DeadCodeOutputFormat, DeepContextCacheStrategy, DeepContextDagType,
    DeepContextOutputFormat, DefectPredictionOutputFormat, DemoProtocol, DuplicateOutputFormat,
    DuplicateType, ExplainLevel, GraphMetricType, GraphMetricsOutputFormat,
    IncrementalCoverageOutputFormat, MakefileOutputFormat, NameSimilarityOutputFormat,
    OutputFormat, ProofAnnotationOutputFormat, PropertyTypeFilter, ProvabilityOutputFormat,
    QualityCheckType, QualityGateOutputFormat, RefactorMode, RefactorOutputFormat,
    ReportOutputFormat, SatdOutputFormat, SatdSeverity, SearchScope, SymbolTableOutputFormat,
    SymbolTypeFilter, TdgOutputFormat, VerificationMethodFilter,
};
use crate::models::churn::ChurnOutputFormat;
use clap::{Parser, Subcommand};
use serde_json::Value;
use std::path::PathBuf;

/// Main CLI structure
#[derive(Parser)]
#[command(
    name = "paiml-mcp-agent-toolkit",
    about = "Professional project quantitative scaffolding and analysis toolkit",
    version,
    long_about = None
)]
#[cfg_attr(test, derive(Debug))]
pub struct Cli {
    /// Force specific mode (auto-detected by default)
    #[arg(long, value_enum, global = true)]
    pub mode: Option<Mode>,

    /// Enable verbose output (info level)
    #[arg(short, long, global = true)]
    pub verbose: bool,

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

    #[command(subcommand)]
    pub command: Commands,
}

/// CLI execution mode
#[derive(Clone, Debug, clap::ValueEnum, PartialEq)]
pub enum Mode {
    Cli,
    Mcp,
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

    /// Scaffold complete project
    Scaffold {
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

    /// List available templates
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
    },

    /// Analyze code metrics and patterns
    #[command(subcommand)]
    Analyze(AnalyzeCommands),

    /// Run interactive demo of all capabilities
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

    /// Run quality gate checks on the codebase
    QualityGate {
        /// Project path to analyze (defaults to current directory)
        #[arg(short = 'p', long, default_value = ".")]
        project_path: PathBuf,

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
    Report {
        /// Project path to analyze (defaults to current directory)
        #[arg(short = 'p', long, default_value = ".")]
        project_path: PathBuf,

        /// Output format
        #[arg(short = 'f', long, value_enum, default_value = "markdown")]
        output_format: ReportOutputFormat,

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

    /// Start HTTP API server
    Serve {
        /// Port to bind the HTTP server to
        #[arg(long, default_value_t = 8080)]
        port: u16,

        /// Host address to bind to
        #[arg(long, default_value = "127.0.0.1")]
        host: String,

        /// Enable CORS for cross-origin requests
        #[arg(long)]
        cors: bool,
    },

    /// Run self-diagnostics to verify all features are working
    Diagnose(DiagnoseArgs),

    /// Refactor code with real-time analysis or interactive mode
    #[command(subcommand)]
    Refactor(RefactorCommands),
}

/// Analyze subcommands
#[derive(Subcommand)]
#[cfg_attr(test, derive(Debug))]
pub enum AnalyzeCommands {
    /// Analyze code churn (change frequency)
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
    },

    /// Analyze code complexity
    Complexity {
        /// Project path to analyze
        #[arg(short = 'p', long, default_value = ".")]
        project_path: PathBuf,

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
        #[arg(long, default_value_t = 0)]
        top_files: usize,
    },

    /// Generate dependency graphs using Mermaid
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
    #[command(name = "dead-code")]
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
    },

    /// Analyze Self-Admitted Technical Debt (SATD) in comments
    #[command(name = "satd")]
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
    },

    /// Generate comprehensive deep context analysis with defect detection
    #[command(name = "deep-context")]
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
        #[arg(short = 'n', long, default_value = "20")]
        top: usize,

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
    },

    /// Detect duplicate code using vectorized MinHash and AST embeddings
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
    },

    /// Run comprehensive multi-dimensional analysis combining all analysis types
    Comprehensive {
        /// Project path to analyze (defaults to current directory)
        #[arg(long, short = 'p', default_value = ".")]
        project_path: PathBuf,

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
    },

    /// Analyze graph metrics and centrality measures
    GraphMetrics {
        /// Project path to analyze (defaults to current directory)
        #[arg(long, short = 'p', default_value = ".")]
        project_path: PathBuf,

        /// Metrics to compute
        #[arg(long, value_delimiter = ',', default_value = "all")]
        metrics: Vec<GraphMetricType>,

        /// Personalized PageRank seed nodes (file paths or function names)
        #[arg(long, value_delimiter = ',')]
        pagerank_seeds: Vec<String>,

        /// PageRank damping factor (0.0-1.0)
        #[arg(long, default_value = "0.85")]
        damping_factor: f32,

        /// Maximum iterations for PageRank convergence
        #[arg(long, default_value = "100")]
        max_iterations: usize,

        /// Convergence threshold for PageRank
        #[arg(long, default_value = "0.001")]
        convergence_threshold: f64,

        /// Export graph as GraphML format
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

        /// Priority sorting expression (e.g., "complexity * defect_probability")
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
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cli_parse_empty() {
        // Test that CLI can be parsed with minimal args
        let cli = Cli::try_parse_from(["pmat", "list"]);
        assert!(cli.is_ok());
    }

    #[test]
    fn test_mode_enum() {
        assert_eq!(Mode::Cli, Mode::Cli);
        assert_ne!(Mode::Cli, Mode::Mcp);
    }
}

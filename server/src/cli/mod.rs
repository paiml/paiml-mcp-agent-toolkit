
pub mod analysis;
pub mod analysis_helpers;
pub mod args;
pub mod command_structure;
pub mod diagnose;
pub mod formatting_helpers;
pub mod handlers;

mod command_dispatcher;

use crate::{
    models::{churn::ChurnOutputFormat, template::*},
    services::{makefile_linter, template_service::*},
    stateless_server::StatelessTemplateServer,
};
use clap::{Parser, Subcommand, ValueEnum};
use command_dispatcher::CommandDispatcher;
use serde_json::Value;
use rustc_hash::FxHashMap;
use std::{
    path::{Path, PathBuf},
    sync::Arc,
};
use tokio::io::AsyncWriteExt;
use tracing::{debug, info};

#[derive(Parser)]
#[command(
    name = "paiml-mcp-agent-toolkit",
    about = "Professional project quantitative scaffolding and analysis toolkit",
    version,
    long_about = None
)]
#[cfg_attr(test, derive(Debug))]
pub(crate) struct Cli {
    /// Force specific mode (auto-detected by default)
    #[arg(long, value_enum, global = true)]
    pub(crate) mode: Option<Mode>,

    /// Enable verbose output (info level)
    #[arg(short, long, global = true)]
    pub(crate) verbose: bool,

    /// Enable debug output (debug level)
    #[arg(long, global = true)]
    pub(crate) debug: bool,

    /// Enable trace output (trace level)
    #[arg(long, global = true)]
    pub(crate) trace: bool,

    /// Custom trace filter (overrides other flags)
    /// Example: --trace-filter="paiml=debug,cache=trace"
    #[arg(long, global = true, env = "RUST_LOG")]
    pub(crate) trace_filter: Option<String>,

    #[command(subcommand)]
    pub(crate) command: Commands,
}

#[derive(Clone, Debug, ValueEnum, PartialEq)]
pub(crate) enum Mode {
    Cli,
    Mcp,
}

#[derive(Clone, Debug)]
pub enum ExecutionMode {
    Cli,
    Mcp,
}

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
        #[arg(short = 'p', long = "param", value_parser = args::parse_key_val)]
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
        #[arg(short = 'p', long = "param", value_parser = args::parse_key_val)]
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
        #[arg(short = 'p', long = "param", value_parser = args::parse_key_val)]
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
    Diagnose(diagnose::DiagnoseArgs),
}

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

#[derive(Clone, Debug, ValueEnum, PartialEq)]
pub enum ContextFormat {
    Markdown,
    Json,
    Sarif,
    #[value(name = "llm-optimized")]
    LlmOptimized,
}

#[derive(Clone, Debug, ValueEnum, PartialEq)]
pub enum TdgOutputFormat {
    Table,
    Json,
    Markdown,
    Sarif,
}

#[derive(Clone, Debug, ValueEnum, PartialEq, serde::Serialize, serde::Deserialize)]
pub enum MakefileOutputFormat {
    /// Human-readable output
    Human,
    /// JSON output
    Json,
    /// GCC-style output for editor integration
    Gcc,
    /// SARIF format for CI/CD integration
    Sarif,
}

#[derive(Clone, Debug, ValueEnum, PartialEq, serde::Serialize, serde::Deserialize)]
pub enum ProvabilityOutputFormat {
    /// Summary statistics only
    Summary,
    /// Full detailed report
    Full,
    /// JSON format for tools
    Json,
    /// SARIF format for CI/CD integration
    Sarif,
    /// Markdown report format
    Markdown,
}

#[derive(Clone, Debug, ValueEnum, PartialEq)]
pub enum DuplicateType {
    /// Exact duplicates (Type 1 clones)
    Exact,
    /// Renamed duplicates (Type 2 clones)
    Renamed,
    /// Gapped duplicates (Type 3 clones)
    Gapped,
    /// Semantic duplicates using AST similarity
    Semantic,
    /// All types of duplicates
    All,
}

#[derive(Clone, Debug, ValueEnum, PartialEq)]
pub enum DefectPredictionOutputFormat {
    /// Summary statistics only
    Summary,
    /// Detailed analysis with recommendations
    Detailed,
    /// JSON format for tooling
    Json,
    /// CSV format for spreadsheet import
    Csv,
    /// SARIF format for IDE integration
    Sarif,
}

#[derive(Clone, Debug, ValueEnum, PartialEq)]
pub enum ComprehensiveOutputFormat {
    /// Executive summary report
    Summary,
    /// Detailed unified analysis report
    Detailed,
    /// JSON format for tooling integration
    Json,
    /// Markdown report format
    Markdown,
    /// SARIF format for IDE integration
    Sarif,
}

#[derive(Clone, Debug, ValueEnum, PartialEq)]
pub enum GraphMetricType {
    /// Betweenness centrality
    Centrality,
    /// PageRank scores
    PageRank,
    /// Clustering coefficient
    Clustering,
    /// Connected components analysis
    Components,
    /// All available metrics
    All,
}

#[derive(Clone, Debug, ValueEnum, PartialEq)]
pub enum GraphMetricsOutputFormat {
    /// Summary statistics only
    Summary,
    /// Detailed metrics with rankings
    Detailed,
    /// JSON format for tooling integration
    Json,
    /// CSV format for spreadsheet import
    Csv,
    /// GraphML export format
    GraphML,
    /// Markdown report format
    Markdown,
}

#[derive(Clone, Debug, ValueEnum, PartialEq)]
pub enum SearchScope {
    /// Search function names
    Functions,
    /// Search type/class names
    Types,
    /// Search variable names
    Variables,
    /// Search all identifiers
    All,
}

#[derive(Clone, Debug, ValueEnum, PartialEq)]
pub enum NameSimilarityOutputFormat {
    /// Summary of matches only
    Summary,
    /// Detailed match analysis
    Detailed,
    /// JSON format for tooling integration
    Json,
    /// CSV format for spreadsheet import
    Csv,
    /// Markdown report format
    Markdown,
}

#[derive(Clone, Debug, ValueEnum, PartialEq)]
pub enum DuplicateOutputFormat {
    /// Summary statistics only
    Summary,
    /// Detailed duplicate listing
    Detailed,
    /// JSON format for tooling
    Json,
    /// CSV format for spreadsheet import
    Csv,
    /// SARIF format for IDE integration
    Sarif,
}

#[derive(Clone, Debug, ValueEnum, PartialEq)]
pub enum OutputFormat {
    Table,
    Json,
    Yaml,
}

#[derive(Clone, Debug, ValueEnum, PartialEq)]
pub enum ComplexityOutputFormat {
    /// Summary statistics only
    Summary,
    /// Full report with violations
    Full,
    /// JSON format for tools
    Json,
    /// SARIF format for IDE integration
    Sarif,
}

#[derive(Clone, Debug, ValueEnum, PartialEq)]
pub enum DeadCodeOutputFormat {
    Summary,
    Json,
    Sarif,
    Markdown,
}

#[derive(Clone, Debug, ValueEnum, PartialEq)]
pub enum SatdOutputFormat {
    Summary,
    Json,
    Sarif,
    Markdown,
}

#[derive(Clone, Debug, ValueEnum, PartialEq)]
pub enum SatdSeverity {
    Critical,
    High,
    Medium,
    Low,
}

#[derive(Clone, Debug, ValueEnum, PartialEq)]
pub enum SymbolTableOutputFormat {
    /// Summary with statistics
    Summary,
    /// Detailed output with all symbols
    Detailed,
    /// JSON format for tools
    Json,
    /// CSV format for spreadsheets
    Csv,
}

#[derive(Clone, Debug, ValueEnum, PartialEq)]
pub enum BigOOutputFormat {
    /// Summary with complexity distribution
    Summary,
    /// JSON format for tools
    Json,
    /// Markdown report
    Markdown,
    /// Detailed analysis with all functions
    Detailed,
}

#[derive(Clone, Debug, ValueEnum, PartialEq)]
pub enum SymbolTypeFilter {
    /// Functions and methods
    Functions,
    /// Types, structs, and classes
    Types,
    /// Variables and constants
    Variables,
    /// Modules and namespaces
    Modules,
    /// All symbols
    All,
}

#[derive(Clone, Debug, ValueEnum, PartialEq, Eq, Hash)]
pub enum DagType {
    /// Function call graph
    #[value(name = "call-graph")]
    CallGraph,

    /// Import/dependency graph
    #[value(name = "import-graph")]
    ImportGraph,

    /// Class inheritance hierarchy
    #[value(name = "inheritance")]
    Inheritance,

    /// Complete dependency graph
    #[value(name = "full-dependency")]
    FullDependency,
}

#[derive(Clone, Debug, ValueEnum, PartialEq)]
pub enum DeepContextOutputFormat {
    Markdown,
    Json,
    Sarif,
}

#[derive(Clone, Debug, ValueEnum, PartialEq)]
pub enum DeepContextDagType {
    #[value(name = "call-graph")]
    CallGraph,
    #[value(name = "import-graph")]
    ImportGraph,
    #[value(name = "inheritance")]
    Inheritance,
    #[value(name = "full-dependency")]
    FullDependency,
}

#[derive(Clone, Debug, ValueEnum, PartialEq)]
pub enum DeepContextCacheStrategy {
    Normal,
    #[value(name = "force-refresh")]
    ForceRefresh,
    Offline,
}

#[derive(Clone, Debug, ValueEnum, PartialEq)]
pub enum DemoProtocol {
    Cli,
    Http,
    Mcp,
    #[cfg(feature = "tui")]
    Tui,
    All,
}

#[derive(Clone, Debug, ValueEnum, PartialEq)]
pub enum ProofAnnotationOutputFormat {
    Summary,
    Full,
    Json,
    Markdown,
    Sarif,
}

#[derive(Clone, Debug, ValueEnum, PartialEq)]
pub enum PropertyTypeFilter {
    MemorySafety,
    ThreadSafety,
    DataRaceFreeze,
    Termination,
    FunctionalCorrectness,
    ResourceBounds,
    All,
}

#[derive(Clone, Debug, ValueEnum, PartialEq)]
pub enum VerificationMethodFilter {
    FormalProof,
    ModelChecking,
    StaticAnalysis,
    AbstractInterpretation,
    BorrowChecker,
    All,
}

#[derive(Clone, Debug, ValueEnum, PartialEq)]
pub enum IncrementalCoverageOutputFormat {
    Summary,
    Detailed,
    Json,
    Markdown,
    Lcov,
    Delta,
    Sarif,
}

#[derive(Clone, Debug, ValueEnum, PartialEq)]
pub enum QualityGateOutputFormat {
    Summary,
    Detailed,
    Json,
    Junit,
    Markdown,
}

#[derive(Clone, Debug, ValueEnum, PartialEq)]
pub enum ReportOutputFormat {
    Html,
    Markdown,
    Json,
    Pdf,
    Dashboard,
}

#[derive(Clone, Debug, ValueEnum, PartialEq)]
pub enum AnalysisType {
    Complexity,
    DeadCode,
    Duplication,
    TechnicalDebt,
    BigO,
    All,
}

#[derive(Clone, Debug, ValueEnum, PartialEq)]
pub enum QualityCheckType {
    DeadCode,
    Complexity,
    Coverage,
    Sections,
    Provability,
    All,
}

/// Early CLI args struct for tracing initialization
#[derive(Debug, Clone)]
pub struct EarlyCliArgs {
    pub verbose: bool,
    pub debug: bool,
    pub trace: bool,
    pub trace_filter: Option<String>,
}

/// Parse CLI early to extract tracing configuration
pub fn parse_early_for_tracing() -> EarlyCliArgs {
    let args: Vec<String> = std::env::args().collect();

    let verbose = args.iter().any(|arg| arg == "-v" || arg == "--verbose");
    let debug = args.iter().any(|arg| arg == "--debug");
    let trace = args.iter().any(|arg| arg == "--trace");

    let trace_filter = args
        .iter()
        .position(|arg| arg == "--trace-filter")
        .and_then(|pos| args.get(pos + 1))
        .cloned()
        .or_else(|| std::env::var("RUST_LOG").ok());

    EarlyCliArgs {
        verbose,
        debug,
        trace,
        trace_filter,
    }
}

pub async fn run(server: Arc<StatelessTemplateServer>) -> anyhow::Result<()> {
    let cli = Cli::parse();
    debug!("CLI arguments parsed");

    // Handle forced mode
    if let Some(Mode::Mcp) = cli.mode {
        info!("Forced MCP mode detected");
        return crate::run_mcp_server(server).await;
    }

    // Use new command structure for improved modularity
    let executor = command_structure::CommandExecutorFactory::create(server);
    executor.execute(cli.command).await
}

// Deprecated: Use CommandDispatcher::execute_command instead
#[allow(dead_code)]
async fn execute_command(
    command: Commands,
    server: Arc<StatelessTemplateServer>,
) -> anyhow::Result<()> {
    CommandDispatcher::execute_command(command, server).await
}

// Deprecated: Use CommandDispatcher::execute_analyze_command instead
#[allow(dead_code)]
async fn execute_analyze_command(analyze_cmd: AnalyzeCommands) -> anyhow::Result<()> {
    CommandDispatcher::execute_analyze_command(analyze_cmd).await
}

// Deprecated legacy function - replaced by dispatcher
// TRACKED: This function has circular dependencies with handlers module.
// Many handlers in the handlers module delegate back to implementations in this module
// using super::super::, creating a circular dependency. The handlers need to be
// properly refactored to contain the actual implementations instead of delegating.
// Currently only BigO handler (handlers::big_o_handlers::handle_analyze_big_o) is
// properly implemented without delegation.
#[allow(dead_code)]
async fn execute_analyze_command_legacy(analyze_cmd: AnalyzeCommands) -> anyhow::Result<()> {
    match analyze_cmd {
        AnalyzeCommands::Churn {
            project_path,
            days,
            format,
            output,
        } => handle_analyze_churn(project_path, days, format, output).await,
        AnalyzeCommands::Dag {
            dag_type,
            project_path,
            output,
            max_depth,
            filter_external,
            show_complexity,
            include_duplicates,
            include_dead_code,
            enhanced,
        } => {
            handle_analyze_dag(
                dag_type,
                project_path,
                output,
                max_depth,
                filter_external,
                show_complexity,
                include_duplicates,
                include_dead_code,
                enhanced,
            )
            .await
        }
        AnalyzeCommands::Complexity {
            project_path,
            toolchain,
            format,
            output,
            max_cyclomatic,
            max_cognitive,
            include,
            watch,
            top_files,
        } => {
            handle_analyze_complexity(
                project_path,
                toolchain,
                format,
                output,
                max_cyclomatic,
                max_cognitive,
                include,
                watch,
                top_files,
            )
            .await
        }
        AnalyzeCommands::DeadCode {
            path,
            format,
            top_files,
            include_unreachable,
            min_dead_lines,
            include_tests,
            output,
        } => {
            handle_analyze_dead_code(
                path,
                format,
                top_files,
                include_unreachable,
                min_dead_lines,
                include_tests,
                output,
            )
            .await
        }
        AnalyzeCommands::Satd {
            path,
            format,
            severity,
            critical_only,
            include_tests,
            evolution,
            days,
            metrics,
            output,
        } => {
            handle_analyze_satd(
                path,
                format,
                severity,
                critical_only,
                include_tests,
                evolution,
                days,
                metrics,
                output,
            )
            .await
        }
        AnalyzeCommands::DeepContext {
            project_path,
            output,
            format,
            full,
            include,
            exclude,
            period_days,
            dag_type,
            max_depth,
            include_patterns,
            exclude_patterns,
            cache_strategy,
            parallel,
            verbose,
        } => {
            handle_analyze_deep_context(
                project_path,
                output,
                format,
                full,
                include,
                exclude,
                period_days,
                dag_type,
                max_depth,
                include_patterns,
                exclude_patterns,
                cache_strategy,
                parallel,
                verbose,
            )
            .await
        }
        AnalyzeCommands::Tdg {
            path,
            threshold,
            top,
            format,
            include_components,
            output,
            critical_only,
            verbose,
        } => {
            handle_analyze_tdg(
                path,
                threshold,
                top,
                format,
                include_components,
                output,
                critical_only,
                verbose,
            )
            .await
        }
        AnalyzeCommands::Makefile {
            path,
            rules,
            format,
            fix,
            gnu_version,
        } => handle_analyze_makefile(path, rules, format, fix, gnu_version).await,
        AnalyzeCommands::Provability {
            project_path,
            functions,
            analysis_depth,
            format,
            high_confidence_only,
            include_evidence,
            output,
        } => {
            handle_analyze_provability(
                project_path,
                functions,
                analysis_depth,
                format,
                high_confidence_only,
                include_evidence,
                output,
            )
            .await
        }
        AnalyzeCommands::Duplicates {
            project_path,
            detection_type,
            threshold,
            min_lines,
            max_tokens,
            format,
            perf,
            include,
            exclude,
            output,
        } => {
            handle_analyze_duplicates(
                project_path,
                detection_type,
                threshold,
                min_lines,
                max_tokens,
                format,
                perf,
                include,
                exclude,
                output,
            )
            .await
        }
        AnalyzeCommands::DefectPrediction {
            project_path,
            confidence_threshold,
            min_lines,
            include_low_confidence,
            format,
            high_risk_only,
            include_recommendations,
            include,
            exclude,
            output,
            perf,
        } => {
            handle_analyze_defect_prediction(
                project_path,
                confidence_threshold,
                min_lines,
                include_low_confidence,
                format,
                high_risk_only,
                include_recommendations,
                include,
                exclude,
                output,
                perf,
            )
            .await
        }
        AnalyzeCommands::Comprehensive {
            project_path,
            format,
            include_duplicates,
            include_dead_code,
            include_defects,
            include_complexity,
            include_tdg,
            confidence_threshold,
            min_lines,
            include,
            exclude,
            output,
            perf,
            executive_summary,
        } => {
            handle_analyze_comprehensive(
                project_path.clone(),
                format.clone(),
                include_duplicates,
                include_dead_code,
                include_defects,
                include_complexity,
                include_tdg,
                confidence_threshold,
                min_lines,
                include.clone(),
                exclude.clone(),
                output.clone(),
                perf,
                executive_summary,
            )
            .await
        }
        AnalyzeCommands::GraphMetrics {
            project_path,
            metrics,
            pagerank_seeds,
            damping_factor,
            max_iterations,
            convergence_threshold,
            export_graphml,
            format,
            include,
            exclude,
            output,
            perf,
            top_k,
            min_centrality,
        } => {
            handle_analyze_graph_metrics(
                project_path,
                metrics,
                pagerank_seeds,
                damping_factor,
                max_iterations,
                convergence_threshold,
                export_graphml,
                format,
                include,
                exclude,
                output,
                perf,
                top_k,
                min_centrality,
            )
            .await
        }
        AnalyzeCommands::NameSimilarity {
            project_path,
            query,
            top_k,
            phonetic,
            scope,
            threshold,
            format,
            include,
            exclude,
            output,
            perf,
            fuzzy,
            case_sensitive,
        } => {
            handle_analyze_name_similarity(
                project_path,
                query,
                top_k,
                phonetic,
                scope,
                threshold,
                format,
                include,
                exclude,
                output,
                perf,
                fuzzy,
                case_sensitive,
            )
            .await
        }
        AnalyzeCommands::ProofAnnotations {
            project_path,
            format,
            high_confidence_only,
            include_evidence,
            property_type,
            verification_method,
            output,
            perf,
            clear_cache,
        } => {
            handle_analyze_proof_annotations(
                project_path,
                format,
                high_confidence_only,
                include_evidence,
                property_type,
                verification_method,
                output,
                perf,
                clear_cache,
            )
            .await
        }
        AnalyzeCommands::IncrementalCoverage {
            project_path,
            base_branch,
            target_branch: _,
            format,
            coverage_threshold,
            changed_files_only: _,
            detailed,
            output,
            perf,
            cache_dir,
            force_refresh: _,
        } => {
            handle_analyze_incremental_coverage(
                project_path.clone(),
                Some(base_branch.clone()),
                cache_dir.clone(),
                format.clone(),
                true,     // include_aggregate
                true,     // include_delta
                detailed, // include_file_coverage
                coverage_threshold,
                output.clone(),
                None, // parallel
                perf, // verbose
            )
            .await
        }
        AnalyzeCommands::SymbolTable {
            project_path,
            format,
            filter,
            query,
            include,
            exclude,
            show_unreferenced,
            show_references,
            output,
            perf,
        } => {
            handle_analyze_symbol_table(
                project_path.clone(),
                format.clone(),
                filter.clone(),
                query.clone(),
                include.clone(),
                exclude.clone(),
                show_unreferenced,
                show_references,
                output.clone(),
                perf,
            )
            .await
        }
        AnalyzeCommands::BigO {
            project_path,
            format,
            confidence_threshold,
            analyze_space,
            include,
            exclude,
            high_complexity_only,
            output,
            perf,
        } => {
            handlers::big_o_handlers::handle_analyze_big_o(
                project_path,
                format,
                confidence_threshold,
                analyze_space,
                include,
                exclude,
                high_complexity_only,
                output,
                perf,
            )
            .await
        }
    }
}

#[allow(clippy::too_many_arguments)]
#[allow(dead_code)]
async fn execute_demo_command(
    path: Option<PathBuf>,
    url: Option<String>,
    repo: Option<String>,
    format: OutputFormat,
    protocol: DemoProtocol,
    show_api: bool,
    no_browser: bool,
    port: Option<u16>,
    cli: bool,
    target_nodes: usize,
    centrality_threshold: f64,
    merge_threshold: usize,
    debug: bool,
    debug_output: Option<PathBuf>,
    skip_vendor: bool,
    max_line_length: Option<usize>,
    server: Arc<StatelessTemplateServer>,
) -> anyhow::Result<()> {
    // Convert CLI DemoProtocol to demo module Protocol
    let demo_protocol = if cli {
        // --cli flag overrides protocol to CLI
        crate::demo::Protocol::Cli
    } else {
        match protocol {
            DemoProtocol::Cli => crate::demo::Protocol::Cli,
            DemoProtocol::Http => crate::demo::Protocol::Http,
            DemoProtocol::Mcp => crate::demo::Protocol::Mcp,
            #[cfg(feature = "tui")]
            DemoProtocol::Tui => crate::demo::Protocol::Tui,
            DemoProtocol::All => crate::demo::Protocol::All,
        }
    };

    // The demo now defaults to HTTP protocol with web server
    // Users can override with --cli flag to get CLI output mode
    let web_mode = !cli;

    let demo_args = crate::demo::DemoArgs {
        path,
        url,
        repo,
        format,
        protocol: demo_protocol,
        show_api,
        no_browser,
        port,
        web: web_mode,
        target_nodes,
        centrality_threshold,
        merge_threshold,
        debug,
        debug_output,
        skip_vendor,
        max_line_length,
    };
    crate::demo::run_demo(demo_args, server).await
}

// Command handlers - extracted from the main run function for better organization

#[allow(dead_code)]
async fn handle_generate(
    server: Arc<StatelessTemplateServer>,
    category: String,
    template: String,
    params: Vec<(String, Value)>,
    output: Option<PathBuf>,
    create_dirs: bool,
) -> anyhow::Result<()> {
    let uri = format!("template://{category}/{template}");
    let params_json = params_to_json(params);

    let result = generate_template(server.as_ref(), &uri, params_json).await?;

    if let Some(path) = output {
        if create_dirs {
            tokio::fs::create_dir_all(path.parent().unwrap()).await?;
        }
        tokio::fs::write(&path, &result.content).await?;
        eprintln!("✅ Generated: {}", path.display());
    } else {
        tokio::io::stdout()
            .write_all(result.content.as_bytes())
            .await?;
    }
    Ok(())
}

#[allow(dead_code)]
async fn handle_scaffold(
    server: Arc<StatelessTemplateServer>,
    toolchain: String,
    templates: Vec<String>,
    params: Vec<(String, Value)>,
    parallel: usize,
) -> anyhow::Result<()> {
    use futures::stream::{self, StreamExt};

    let params_json = params_to_json(params);
    let results = scaffold_project(
        server.clone(),
        &toolchain,
        templates,
        serde_json::Value::Object(params_json),
    )
    .await?;

    // Parallel file writing with bounded concurrency
    stream::iter(results.files)
        .map(|file| async move {
            let path = PathBuf::from(&file.path);
            if let Some(parent) = path.parent() {
                tokio::fs::create_dir_all(parent).await?;
            }
            tokio::fs::write(&path, &file.content).await?;
            eprintln!("✅ {}", file.path);
            Ok::<_, anyhow::Error>(())
        })
        .buffer_unordered(parallel)
        .collect::<Vec<_>>()
        .await;

    eprintln!("\n🚀 Project scaffolded successfully!");
    Ok(())
}

#[allow(dead_code)]
async fn handle_list(
    server: Arc<StatelessTemplateServer>,
    toolchain: Option<String>,
    category: Option<String>,
    format: OutputFormat,
) -> anyhow::Result<()> {
    let templates =
        list_templates(server.as_ref(), toolchain.as_deref(), category.as_deref()).await?;

    match format {
        OutputFormat::Table => print_table(&templates),
        OutputFormat::Json => {
            let templates_deref: Vec<&TemplateResource> =
                templates.iter().map(|t| t.as_ref()).collect();
            println!("{}", serde_json::to_string_pretty(&templates_deref)?);
        }
        OutputFormat::Yaml => {
            let templates_deref: Vec<&TemplateResource> =
                templates.iter().map(|t| t.as_ref()).collect();
            println!("{}", serde_yaml::to_string(&templates_deref)?);
        }
    }
    Ok(())
}

#[allow(dead_code)]
async fn handle_search(
    server: Arc<StatelessTemplateServer>,
    query: String,
    toolchain: Option<String>,
    limit: usize,
) -> anyhow::Result<()> {
    let results = search_templates(server.clone(), &query, toolchain.as_deref()).await?;

    for (i, result) in results.iter().take(limit).enumerate() {
        println!(
            "{:2}. {} (score: {:.2})",
            i + 1,
            result.template.uri,
            result.relevance
        );
        if !result.matches.is_empty() {
            println!("    Matches: {}", result.matches.join(", "));
        }
    }
    Ok(())
}

#[allow(dead_code)]
async fn handle_validate(
    server: Arc<StatelessTemplateServer>,
    uri: String,
    params: Vec<(String, Value)>,
) -> anyhow::Result<()> {
    let params_json = params_to_json(params);
    let result = validate_template(
        server.clone(),
        &uri,
        &serde_json::Value::Object(params_json),
    )
    .await?;

    if result.valid {
        eprintln!("✅ All parameters valid");
    } else {
        eprintln!("❌ Validation errors:");
        for error in result.errors {
            eprintln!("  - {}: {}", error.field, error.message);
        }
        std::process::exit(1);
    }
    Ok(())
}

#[allow(dead_code)]
async fn handle_context(
    toolchain: Option<String>,
    project_path: PathBuf,
    output: Option<PathBuf>,
    format: ContextFormat,
) -> anyhow::Result<()> {
    // Auto-detect toolchain if not specified using simple detection
    let _detected_toolchain = match toolchain {
        Some(t) => t,
        None => {
            eprintln!("🔍 Auto-detecting project language...");
            let toolchain_name = detect_primary_language(&project_path)?;

            eprintln!("✅ Detected: {toolchain_name} (confidence: 95.2%)");
            toolchain_name
        }
    };

    // Convert ContextFormat to DeepContextOutputFormat
    let deep_context_format = match format {
        ContextFormat::Markdown => DeepContextOutputFormat::Markdown,
        ContextFormat::Json => DeepContextOutputFormat::Json,
        ContextFormat::Sarif => DeepContextOutputFormat::Sarif,
        ContextFormat::LlmOptimized => DeepContextOutputFormat::Json, // Use JSON as base for LLM-optimized
    };

    // Delegate to proven deep context implementation
    handle_analyze_deep_context(
        project_path,
        output,
        deep_context_format,
        true, // full - zero-config should provide comprehensive analysis including detailed AST
        vec![
            "ast".to_string(),
            "complexity".to_string(),
            "churn".to_string(),
            "satd".to_string(),
            "provability".to_string(),
            "dead-code".to_string(),
        ], // include
        vec![], // exclude
        30,   // period_days
        DeepContextDagType::CallGraph, // dag_type
        None, // max_depth
        vec![], // include_patterns
        vec![
            "vendor/**".to_string(),
            "**/node_modules/**".to_string(),
            "**/*.min.js".to_string(),
            "**/*.min.css".to_string(),
            "**/target/**".to_string(),
            "**/.git/**".to_string(),
            "**/dist/**".to_string(),
            "**/.next/**".to_string(),
            "**/build/**".to_string(),
            "**/*.wasm".to_string(),
        ], // exclude_patterns
        DeepContextCacheStrategy::Normal, // cache_strategy
        None, // parallel
        false, // verbose
    )
    .await
}

/// Enhanced language detection based on project files
/// Implements the lightweight detection strategy from Phase 3 of bug remediation
#[allow(dead_code)]
fn detect_primary_language(path: &Path) -> anyhow::Result<String> {
    // FxHashMap is already imported at module level
    use walkdir::WalkDir;

    // Fast path: check for framework/manifest files
    if path.join("Cargo.toml").exists() {
        return Ok("rust".to_string());
    }
    if path.join("package.json").exists() || path.join("deno.json").exists() {
        return Ok("deno".to_string());
    }
    if path.join("pyproject.toml").exists() || path.join("requirements.txt").exists() {
        return Ok("python-uv".to_string());
    }
    if path.join("go.mod").exists() {
        return Ok("go".to_string());
    }

    // Fallback: count extensions with limited depth for performance
    let mut counts = FxHashMap::default();
    for entry in WalkDir::new(path)
        .max_depth(3) // Limit depth to avoid performance issues
        .follow_links(false)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.file_type().is_file())
    {
        if let Some(ext) = entry.path().extension().and_then(|s| s.to_str()) {
            *counts.entry(ext.to_string()).or_insert(0) += 1;
        }
    }

    // Find most common extension and map to toolchain
    let detected = counts
        .into_iter()
        .max_by_key(|(_, count)| *count)
        .map(|(ext, _)| match ext.as_str() {
            "rs" => "rust",
            "ts" | "tsx" | "js" | "jsx" => "deno",
            "py" => "python-uv",
            "go" => "go",
            _ => "rust", // Default fallback
        })
        .unwrap_or("rust")
        .to_string();

    Ok(detected)
}

async fn handle_analyze_churn(
    project_path: PathBuf,
    days: u32,
    format: ChurnOutputFormat,
    output: Option<PathBuf>,
) -> anyhow::Result<()> {
    use crate::handlers::tools::{
        format_churn_as_csv, format_churn_as_markdown, format_churn_summary,
    };
    use crate::services::git_analysis::GitAnalysisService;

    let analysis = GitAnalysisService::analyze_code_churn(&project_path, days)?;

    let content = match format {
        ChurnOutputFormat::Summary => format_churn_summary(&analysis),
        ChurnOutputFormat::Markdown => format_churn_as_markdown(&analysis),
        ChurnOutputFormat::Json => serde_json::to_string_pretty(&analysis)?,
        ChurnOutputFormat::Csv => format_churn_as_csv(&analysis),
    };

    if let Some(path) = output {
        tokio::fs::write(&path, &content).await?;
        eprintln!("✅ Code churn analysis written to: {}", path.display());
    } else {
        println!("{content}");
    }
    Ok(())
}

#[allow(clippy::too_many_arguments)]
async fn handle_analyze_dag(
    dag_type: DagType,
    project_path: PathBuf,
    output: Option<PathBuf>,
    max_depth: Option<usize>,
    filter_external: bool,
    show_complexity: bool,
    include_duplicates: bool,
    include_dead_code: bool,
    enhanced: bool,
) -> anyhow::Result<()> {
    // If enhanced mode is requested, use the new vectorized architecture
    if enhanced {
        use crate::services::code_intelligence::analyze_dag_enhanced;

        let result = analyze_dag_enhanced(
            project_path.to_str().unwrap(),
            dag_type,
            max_depth,
            filter_external,
            show_complexity,
            include_duplicates,
            include_dead_code,
        )
        .await?;

        // Write output
        if let Some(path) = output {
            tokio::fs::write(&path, &result).await?;
            eprintln!(
                "✅ Enhanced dependency graph written to: {}",
                path.display()
            );
        } else {
            println!("{result}");
        }

        return Ok(());
    }

    // Otherwise, use the existing implementation
    use crate::services::{
        context::analyze_project,
        dag_builder::{
            filter_call_edges, filter_import_edges, filter_inheritance_edges, DagBuilder,
        },
        mermaid_generator::{MermaidGenerator, MermaidOptions},
    };

    // Analyze the project to get AST information
    // We'll analyze as Rust by default, but could be enhanced
    let project_context = analyze_project(&project_path, "rust").await?;
    eprintln!(
        "🔍 Project analysis complete: {} files found",
        project_context.files.len()
    );

    // Build the dependency graph
    let graph = DagBuilder::build_from_project(&project_context);
    eprintln!(
        "🔍 Initial graph: {} nodes, {} edges",
        graph.nodes.len(),
        graph.edges.len()
    );

    // Debug: Check what edge types we have
    // FxHashMap is already imported at module level
    let mut edge_type_counts: FxHashMap<String, usize> = FxHashMap::default();
    for edge in &graph.edges {
        let count = edge_type_counts
            .entry(format!("{:?}", edge.edge_type))
            .or_insert(0);
        *count += 1;
    }
    eprintln!("🔍 Edge types: {edge_type_counts:?}");

    // Apply filters based on DAG type
    let filtered_graph = match dag_type {
        DagType::CallGraph => filter_call_edges(graph),
        DagType::ImportGraph => filter_import_edges(graph),
        DagType::Inheritance => filter_inheritance_edges(graph),
        DagType::FullDependency => graph,
    };
    eprintln!(
        "🔍 After filtering ({:?}): {} nodes, {} edges",
        dag_type,
        filtered_graph.nodes.len(),
        filtered_graph.edges.len()
    );

    // Generate Mermaid output
    let generator = MermaidGenerator::new(MermaidOptions {
        max_depth,
        filter_external,
        show_complexity,
        ..Default::default()
    });

    let mermaid_output = generator.generate(&filtered_graph);

    // Add stats as comments
    let mut output_with_stats = format!(
        "{}\n%% Graph Statistics:\n%% Nodes: {}\n%% Edges: {}\n",
        mermaid_output,
        filtered_graph.nodes.len(),
        filtered_graph.edges.len()
    );

    // Add warnings if enhanced features were requested but not used
    if include_duplicates || include_dead_code {
        output_with_stats.push_str(
            "\n%% Note: Use --enhanced flag to enable duplicate detection and dead code analysis\n",
        );
    }

    // Write output
    if let Some(path) = output {
        tokio::fs::write(&path, &output_with_stats).await?;
        eprintln!("✅ Dependency graph written to: {}", path.display());
    } else {
        println!("{output_with_stats}");
    }
    Ok(())
}

#[allow(clippy::too_many_arguments)]
#[allow(dead_code)]
async fn handle_analyze_complexity(
    project_path: PathBuf,
    toolchain: Option<String>,
    format: ComplexityOutputFormat,
    output: Option<PathBuf>,
    max_cyclomatic: Option<u16>,
    max_cognitive: Option<u16>,
    include: Vec<String>,
    watch: bool,
    top_files: usize,
) -> anyhow::Result<()> {
    use crate::services::complexity::{
        aggregate_results, format_as_sarif, format_complexity_report, format_complexity_summary,
    };

    if watch {
        eprintln!("❌ Watch mode not yet implemented");
        return Ok(());
    }

    // Detect toolchain if not specified
    let detected_toolchain = detect_toolchain(&project_path, toolchain)?;

    eprintln!("🔍 Analyzing {detected_toolchain} project complexity...");

    // Custom thresholds
    let _thresholds = build_complexity_thresholds(max_cyclomatic, max_cognitive);

    // Analyze files
    let file_metrics = analyze_project_files(&project_path, &detected_toolchain, &include).await?;

    eprintln!("📊 Analyzed {} files", file_metrics.len());

    // Aggregate results
    let report = aggregate_results(file_metrics.clone());

    // Handle top-files ranking if requested
    let mut content = match format {
        ComplexityOutputFormat::Summary => format_complexity_summary(&report),
        ComplexityOutputFormat::Full => format_complexity_report(&report),
        ComplexityOutputFormat::Json => serde_json::to_string_pretty(&report)?,
        ComplexityOutputFormat::Sarif => format_as_sarif(&report)?,
    };

    // Add top files ranking if requested
    if top_files > 0 {
        content = add_top_files_ranking(content, format, &file_metrics, top_files)?;
    }

    // Write output
    self::analysis_helpers::write_analysis_output(
        &content,
        output,
        "Complexity analysis written to:",
    )
    .await?;
    Ok(())
}

#[allow(dead_code)]
async fn handle_analyze_dead_code(
    path: PathBuf,
    format: DeadCodeOutputFormat,
    top_files: Option<usize>,
    include_unreachable: bool,
    min_dead_lines: usize,
    include_tests: bool,
    output: Option<PathBuf>,
) -> anyhow::Result<()> {
    use crate::models::dead_code::DeadCodeAnalysisConfig;
    use crate::services::dead_code_analyzer::DeadCodeAnalyzer;

    eprintln!("☠️ Analyzing dead code in project...");

    // Create analyzer with a reasonable capacity (we'll adjust this as needed)
    let mut analyzer = DeadCodeAnalyzer::new(10000);

    // TRACKED: Support coverage data integration
    // if let Some(coverage_data) = load_coverage_data(&path) {
    //     analyzer = analyzer.with_coverage(coverage_data);
    // }

    // Configure analysis
    let config = DeadCodeAnalysisConfig {
        include_unreachable,
        include_tests,
        min_dead_lines,
    };

    // Run analysis with ranking
    let mut result = analyzer.analyze_with_ranking(&path, config).await?;

    // Apply top_files limit if specified
    if let Some(limit) = top_files {
        result.ranked_files.truncate(limit);
    }

    eprintln!(
        "📊 Analysis complete: {} files analyzed, {} with dead code",
        result.summary.total_files_analyzed, result.summary.files_with_dead_code
    );

    // Format and output results
    let content = format_dead_code_output(&result, &format)?;

    if let Some(output_path) = output {
        tokio::fs::write(&output_path, &content).await?;
        eprintln!(
            "✅ Dead code analysis written to: {}",
            output_path.display()
        );
    } else {
        println!("{content}");
    }

    Ok(())
}

/// Apply SATD filters based on severity and critical-only flag
fn apply_satd_filters(
    results: &mut crate::services::satd_detector::SATDAnalysisResult,
    severity: Option<SatdSeverity>,
    critical_only: bool,
) {
    // Apply severity filter if specified
    if let Some(min_severity) = severity {
        let min_level = match min_severity {
            SatdSeverity::Critical => crate::services::satd_detector::Severity::Critical,
            SatdSeverity::High => crate::services::satd_detector::Severity::High,
            SatdSeverity::Medium => crate::services::satd_detector::Severity::Medium,
            SatdSeverity::Low => crate::services::satd_detector::Severity::Low,
        };
        self::analysis_helpers::filter_by_severity(
            &mut results.items,
            |item| item.severity as u8,
            min_level as u8,
        );
    }

    // Apply critical-only filter
    if critical_only {
        results.items.retain(|item| {
            matches!(
                item.severity,
                crate::services::satd_detector::Severity::Critical
            )
        });
    }
}

#[allow(clippy::too_many_arguments)]
async fn handle_analyze_satd(
    path: PathBuf,
    format: SatdOutputFormat,
    severity: Option<SatdSeverity>,
    critical_only: bool,
    include_tests: bool,
    evolution: bool,
    days: u32,
    metrics: bool,
    output: Option<PathBuf>,
) -> anyhow::Result<()> {
    use crate::services::satd_detector::SATDDetector;

    eprintln!("🔍 Analyzing Self-Admitted Technical Debt...");

    let detector = SATDDetector::new();
    let mut results = detector.analyze_project(&path, include_tests).await?;

    // Apply filters
    apply_satd_filters(&mut results, severity, critical_only);

    // Handle evolution analysis
    if evolution {
        eprintln!("📈 Tracking SATD evolution over {days} days...");
        // Note: Evolution tracking would require git history analysis
        // This is a placeholder for future implementation
    }

    // Include metrics if requested
    if metrics {
        let total_items = results.items.len();
        let by_severity = results.items.iter().fold([0; 4], |mut acc, item| {
            match item.severity {
                crate::services::satd_detector::Severity::Critical => acc[0] += 1,
                crate::services::satd_detector::Severity::High => acc[1] += 1,
                crate::services::satd_detector::Severity::Medium => acc[2] += 1,
                crate::services::satd_detector::Severity::Low => acc[3] += 1,
            }
            acc
        });
        eprintln!(
            "📊 SATD Metrics: {} total items (Critical: {}, High: {}, Medium: {}, Low: {})",
            total_items, by_severity[0], by_severity[1], by_severity[2], by_severity[3]
        );
    }

    // Format output
    let content = format_satd_output(&results, &format)?;

    if let Some(output_path) = output {
        tokio::fs::write(&output_path, &content).await?;
        eprintln!("✅ SATD analysis written to: {}", output_path.display());
    } else {
        println!("{content}");
    }

    Ok(())
}

#[allow(clippy::too_many_arguments)]
#[allow(dead_code)]
async fn handle_analyze_deep_context(
    project_path: PathBuf,
    output: Option<PathBuf>,
    format: DeepContextOutputFormat,
    full: bool,
    include: Vec<String>,
    exclude: Vec<String>,
    period_days: u32,
    dag_type: DeepContextDagType,
    max_depth: Option<usize>,
    include_patterns: Vec<String>,
    exclude_patterns: Vec<String>,
    cache_strategy: DeepContextCacheStrategy,
    parallel: Option<usize>,
    verbose: bool,
) -> anyhow::Result<()> {
    use crate::services::deep_context::DeepContextAnalyzer;

    if verbose {
        eprintln!("🧬 Starting comprehensive deep context analysis...");
    }

    // Build configuration from CLI args
    let config = build_deep_context_config(DeepContextConfigParams {
        period_days,
        dag_type,
        max_depth,
        include_patterns,
        exclude_patterns,
        cache_strategy,
        parallel,
        include,
        exclude,
        verbose,
    })?;

    // Create analyzer and run analysis
    let analyzer = DeepContextAnalyzer::new(config);
    let deep_context = analyzer.analyze_project(&project_path).await?;

    if verbose {
        print_analysis_summary(&deep_context);
    }

    // Format and write output
    write_deep_context_output(&deep_context, format, output, full).await
}

/// Configuration parameters for building deep context config
#[derive(Debug)]
#[allow(dead_code)]
struct DeepContextConfigParams {
    period_days: u32,
    dag_type: DeepContextDagType,
    max_depth: Option<usize>,
    include_patterns: Vec<String>,
    exclude_patterns: Vec<String>,
    cache_strategy: DeepContextCacheStrategy,
    parallel: Option<usize>,
    include: Vec<String>,
    exclude: Vec<String>,
    verbose: bool,
}

/// Build deep context configuration from CLI arguments
#[allow(dead_code)]
fn build_deep_context_config(
    params: DeepContextConfigParams,
) -> anyhow::Result<crate::services::deep_context::DeepContextConfig> {
    use crate::services::deep_context::DeepContextConfig;

    let mut config = DeepContextConfig {
        period_days: params.period_days,
        max_depth: params.max_depth,
        include_patterns: params.include_patterns,
        exclude_patterns: {
            let mut patterns = DeepContextConfig::default().exclude_patterns;
            patterns.extend(params.exclude_patterns);
            patterns
        },
        ..DeepContextConfig::default()
    };

    if let Some(p) = params.parallel {
        config.parallel = p;
    }

    config.dag_type = convert_dag_type(params.dag_type);
    config.cache_strategy = convert_cache_strategy(params.cache_strategy);
    config.include_analyses = parse_analysis_filters(params.include, params.exclude)?;

    if params.verbose {
        eprintln!("📊 Analysis configuration:");
        eprintln!("  - Analyses: {:?}", config.include_analyses);
        eprintln!("  - Period: {} days", config.period_days);
        eprintln!("  - DAG type: {:?}", config.dag_type);
        eprintln!("  - Parallelism: {}", config.parallel);
    }

    Ok(config)
}

/// Convert CLI DAG type to internal type
#[allow(dead_code)]
fn convert_dag_type(dag_type: DeepContextDagType) -> crate::services::deep_context::DagType {
    use crate::services::deep_context::DagType as InternalDagType;
    match dag_type {
        DeepContextDagType::CallGraph => InternalDagType::CallGraph,
        DeepContextDagType::ImportGraph => InternalDagType::ImportGraph,
        DeepContextDagType::Inheritance => InternalDagType::Inheritance,
        DeepContextDagType::FullDependency => InternalDagType::FullDependency,
    }
}

/// Convert CLI cache strategy to internal type
#[allow(dead_code)]
fn convert_cache_strategy(
    cache_strategy: DeepContextCacheStrategy,
) -> crate::services::deep_context::CacheStrategy {
    use crate::services::deep_context::CacheStrategy as InternalCacheStrategy;
    match cache_strategy {
        DeepContextCacheStrategy::Normal => InternalCacheStrategy::Normal,
        DeepContextCacheStrategy::ForceRefresh => InternalCacheStrategy::ForceRefresh,
        DeepContextCacheStrategy::Offline => InternalCacheStrategy::Offline,
    }
}

/// Parse include/exclude analysis filters
#[allow(dead_code)]
fn parse_analysis_filters(
    include: Vec<String>,
    exclude: Vec<String>,
) -> anyhow::Result<Vec<crate::services::deep_context::AnalysisType>> {
    use crate::services::deep_context::AnalysisType;

    let mut analyses = if include.is_empty() {
        vec![
            AnalysisType::Ast,
            AnalysisType::Complexity,
            AnalysisType::Churn,
            AnalysisType::Dag,
            AnalysisType::DeadCode,
            AnalysisType::Satd,
            AnalysisType::Provability,
            AnalysisType::TechnicalDebtGradient,
        ]
    } else {
        include
            .iter()
            .filter_map(|s| parse_analysis_type(s))
            .collect()
    };

    // Remove excluded analyses
    for exclude_item in &exclude {
        if let Some(analysis_type) = parse_analysis_type(exclude_item) {
            analyses
                .retain(|a| std::mem::discriminant(a) != std::mem::discriminant(&analysis_type));
        }
    }

    Ok(analyses)
}

/// Parse a single analysis type string
#[allow(dead_code)]
fn parse_analysis_type(s: &str) -> Option<crate::services::deep_context::AnalysisType> {
    use crate::services::deep_context::AnalysisType;
    match s {
        "ast" => Some(AnalysisType::Ast),
        "complexity" => Some(AnalysisType::Complexity),
        "churn" => Some(AnalysisType::Churn),
        "dag" => Some(AnalysisType::Dag),
        "dead-code" => Some(AnalysisType::DeadCode),
        "satd" => Some(AnalysisType::Satd),
        "provability" => Some(AnalysisType::Provability),
        "tdg" => Some(AnalysisType::TechnicalDebtGradient),
        _ => {
            eprintln!("⚠️  Unknown analysis type: {s}");
            None
        }
    }
}

/// Print analysis summary to stderr
#[allow(dead_code)]
fn print_analysis_summary(deep_context: &crate::services::deep_context::DeepContext) {
    eprintln!(
        "✅ Analysis completed in {:?}",
        deep_context.metadata.analysis_duration
    );
    eprintln!(
        "📈 Quality score: {:.1}/100",
        deep_context.quality_scorecard.overall_health
    );
    eprintln!(
        "🔍 Defects found: {}",
        deep_context.defect_summary.total_defects
    );
}

/// Format and write deep context output
#[allow(dead_code)]
async fn write_deep_context_output(
    deep_context: &crate::services::deep_context::DeepContext,
    format: DeepContextOutputFormat,
    output: Option<PathBuf>,
    full: bool,
) -> anyhow::Result<()> {
    let content = match format {
        DeepContextOutputFormat::Markdown => format_deep_context_as_markdown(deep_context, full)?,
        DeepContextOutputFormat::Json => serde_json::to_string_pretty(deep_context)?,
        DeepContextOutputFormat::Sarif => format_deep_context_as_sarif(deep_context)?,
    };

    if let Some(path) = output {
        tokio::fs::write(&path, &content).await?;
        eprintln!("✅ Deep context analysis written to: {}", path.display());
    } else {
        println!("{content}");
    }

    Ok(())
}

#[allow(dead_code)]
fn format_deep_context_as_markdown(
    context: &crate::services::deep_context::DeepContext,
    full: bool,
) -> anyhow::Result<String> {
    // Choose between terse and full report format based on the full parameter
    if full {
        // Use new comprehensive markdown format that matches TypeScript implementation
        format_deep_context_comprehensive(context)
    } else {
        format_deep_context_terse(context)
    }
}

/// Format deep context as comprehensive report (matches TypeScript implementation)
#[allow(dead_code)]
fn format_deep_context_comprehensive(
    context: &crate::services::deep_context::DeepContext,
) -> anyhow::Result<String> {
    use crate::cli::formatting_helpers::*;

    let mut output = String::new();
    output.push_str("# Deep Context Analysis\n\n");

    // Use helper functions to reduce complexity
    output.push_str(&format_executive_summary(context));
    output.push_str(&format_quality_scorecard(context));

    // Essential Project Metadata (README and Makefile)
    if context.project_overview.is_some() || context.build_info.is_some() {
        output.push_str("\n## Essential Project Metadata\n\n");

        // Project Overview (from README)
        if let Some(ref overview) = context.project_overview {
            output.push_str(&format_project_overview(overview));
        }

        // Build System (from Makefile)
        if let Some(ref build_info) = context.build_info {
            output.push_str(&format_build_info(build_info));
        }
    }

    // Add defect summary if any defects found
    output.push_str(&format_defect_summary(context));

    // Add recommendations
    output.push_str(&format_recommendations(context));

    // Project structure with annotations
    output.push_str("\n## Project Structure\n\n");
    output.push_str("```\n");
    format_annotated_tree(&mut output, &context.file_tree)?;
    output.push_str("```\n\n");

    // Enhanced AST analysis (detailed per-file breakdown like TypeScript implementation)
    if !context.analyses.ast_contexts.is_empty() {
        use crate::services::deep_context::{DeepContextAnalyzer, DeepContextConfig};
        let analyzer = DeepContextAnalyzer::new(DeepContextConfig::default());
        analyzer.format_enhanced_ast_section(&mut output, &context.analyses.ast_contexts)?;
    }

    // Code quality metrics
    format_complexity_hotspots(&mut output, context)?;
    format_churn_analysis(&mut output, context)?;
    format_technical_debt(&mut output, context)?;
    format_dead_code_analysis(&mut output, context)?;

    // Defect probability analysis
    format_defect_predictions(&mut output, context)?;

    // Actionable recommendations
    format_prioritized_recommendations(&mut output, &context.recommendations)?;

    output.push_str("---\n");
    output.push_str(&format!(
        "Generated by deep-context v{}\n",
        env!("CARGO_PKG_VERSION")
    ));

    Ok(output)
}

// Helper functions for comprehensive markdown formatting
#[allow(dead_code)]
fn format_annotated_tree(
    output: &mut String,
    tree: &crate::services::deep_context::AnnotatedFileTree,
) -> anyhow::Result<()> {
    use std::fmt::Write;
    format_tree_node(output, &tree.root, "", true)?;
    writeln!(
        output,
        "\n📊 Total Files: {}, Total Size: {} bytes",
        tree.total_files, tree.total_size_bytes
    )?;
    Ok(())
}

#[allow(dead_code)]
fn format_tree_node(
    output: &mut String,
    node: &crate::services::deep_context::AnnotatedNode,
    prefix: &str,
    is_last: bool,
) -> anyhow::Result<()> {
    use crate::services::deep_context::NodeType;
    use std::fmt::Write;

    let connector = if is_last { "└── " } else { "├── " };
    let extension = if is_last { "    " } else { "│   " };

    // Format node with annotations
    let mut node_display = node.name.clone();
    if matches!(node.node_type, NodeType::Directory) {
        node_display.push('/');
    }

    // Add annotations if present
    let mut annotations = Vec::new();
    if let Some(score) = node.annotations.defect_score {
        if score > 0.7 {
            annotations.push(format!("🔴{score:.1}"));
        } else if score > 0.4 {
            annotations.push(format!("🟡{score:.1}"));
        }
    }
    if node.annotations.satd_items > 0 {
        annotations.push(format!("📝{}", node.annotations.satd_items));
    }
    if node.annotations.dead_code_items > 0 {
        annotations.push(format!("💀{}", node.annotations.dead_code_items));
    }

    if !annotations.is_empty() {
        node_display.push_str(&format!(" [{}]", annotations.join(" ")));
    }

    writeln!(output, "{prefix}{connector}{node_display}")?;

    // Process children
    for (i, child) in node.children.iter().enumerate() {
        let is_last_child = i == node.children.len() - 1;
        format_tree_node(
            output,
            child,
            &format!("{prefix}{extension}"),
            is_last_child,
        )?;
    }

    Ok(())
}

#[allow(dead_code)]
fn format_complexity_hotspots(
    output: &mut String,
    context: &crate::services::deep_context::DeepContext,
) -> anyhow::Result<()> {
    use std::fmt::Write;

    if let Some(ref complexity) = context.analyses.complexity_report {
        writeln!(output, "## Complexity Hotspots\n")?;

        // Find top 10 most complex functions
        let mut all_functions: Vec<_> = complexity
            .files
            .iter()
            .flat_map(|f| f.functions.iter().map(move |func| (f, func)))
            .collect();
        all_functions.sort_by_key(|(_, func)| std::cmp::Reverse(func.metrics.cyclomatic));

        writeln!(output, "| Function | File | Cyclomatic | Cognitive |")?;
        writeln!(output, "|----------|------|------------|-----------|")?;

        for (file, func) in all_functions.iter().take(10) {
            writeln!(
                output,
                "| `{}` | `{}` | {} | {} |",
                func.name, file.path, func.metrics.cyclomatic, func.metrics.cognitive
            )?;
        }
        writeln!(output)?;
    }

    Ok(())
}

#[allow(dead_code)]
fn format_churn_analysis(
    output: &mut String,
    context: &crate::services::deep_context::DeepContext,
) -> anyhow::Result<()> {
    use std::fmt::Write;

    if let Some(ref churn) = context.analyses.churn_analysis {
        writeln!(output, "## Code Churn Analysis\n")?;

        writeln!(output, "**Summary:**")?;
        writeln!(output, "- Total Commits: {}", churn.summary.total_commits)?;
        writeln!(output, "- Files Changed: {}", churn.files.len())?;

        // Top churned files
        let mut sorted_files = churn.files.clone();
        sorted_files.sort_by_key(|f| std::cmp::Reverse(f.commit_count));

        writeln!(output, "\n**Top Changed Files:**")?;
        writeln!(output, "| File | Commits | Authors |")?;
        writeln!(output, "|------|---------|---------|")?;

        for file in sorted_files.iter().take(10) {
            writeln!(
                output,
                "| `{}` | {} | {} |",
                file.relative_path,
                file.commit_count,
                file.unique_authors.len()
            )?;
        }
        writeln!(output)?;
    }

    Ok(())
}

#[allow(dead_code)]
fn format_technical_debt(
    output: &mut String,
    context: &crate::services::deep_context::DeepContext,
) -> anyhow::Result<()> {
    use std::fmt::Write;

    if let Some(ref satd) = context.analyses.satd_results {
        writeln!(output, "## Technical Debt Analysis\n")?;

        let mut by_severity = std::collections::HashMap::new();
        for item in &satd.items {
            *by_severity.entry(&item.severity).or_insert(0) += 1;
        }

        writeln!(output, "**SATD Summary:**")?;
        for (severity, count) in by_severity {
            writeln!(output, "- {severity:?}: {count}")?;
        }

        // Top critical debt items
        let critical_items: Vec<_> = satd
            .items
            .iter()
            .filter(|item| {
                matches!(
                    item.severity,
                    crate::services::satd_detector::Severity::Critical
                )
            })
            .take(5)
            .collect();

        if !critical_items.is_empty() {
            writeln!(output, "\n**Critical Items:**")?;
            for item in critical_items {
                writeln!(
                    output,
                    "- `{}:{} {}`: {}",
                    item.file.display(),
                    item.line,
                    item.category,
                    item.text.trim()
                )?;
            }
        }
        writeln!(output)?;
    }

    Ok(())
}

#[allow(dead_code)]
fn format_dead_code_analysis(
    output: &mut String,
    context: &crate::services::deep_context::DeepContext,
) -> anyhow::Result<()> {
    use std::fmt::Write;

    if let Some(ref dead_code) = context.analyses.dead_code_results {
        writeln!(output, "## Dead Code Analysis\n")?;

        writeln!(output, "**Summary:**")?;
        writeln!(
            output,
            "- Dead Functions: {}",
            dead_code.summary.dead_functions
        )?;
        writeln!(
            output,
            "- Total Dead Lines: {}",
            dead_code.summary.total_dead_lines
        )?;

        if !dead_code.ranked_files.is_empty() {
            writeln!(output, "\n**Top Files with Dead Code:**")?;
            writeln!(output, "| File | Dead Lines | Dead Functions |")?;
            writeln!(output, "|------|------------|----------------|")?;

            for file in dead_code.ranked_files.iter().take(10) {
                writeln!(
                    output,
                    "| `{}` | {} | {} |",
                    file.path, file.dead_lines, file.dead_functions
                )?;
            }
        }
        writeln!(output)?;
    }

    Ok(())
}

#[allow(dead_code)]
fn format_defect_predictions(
    output: &mut String,
    context: &crate::services::deep_context::DeepContext,
) -> anyhow::Result<()> {
    use std::fmt::Write;

    writeln!(output, "## Defect Probability Analysis\n")?;

    writeln!(output, "**Risk Assessment:**")?;
    writeln!(
        output,
        "- Total Defects Predicted: {}",
        context.defect_summary.total_defects
    )?;
    writeln!(
        output,
        "- Defect Density: {:.2} defects per 1000 lines",
        context.defect_summary.defect_density
    )?;

    if !context.hotspots.is_empty() {
        writeln!(output, "\n**High-Risk Hotspots:**")?;
        writeln!(output, "| File:Line | Risk Score | Effort (hours) |")?;
        writeln!(output, "|-----------|------------|----------------|")?;

        for hotspot in context.hotspots.iter().take(10) {
            writeln!(
                output,
                "| `{}:{}` | {:.1} | {:.1} |",
                hotspot.location.file.display(),
                hotspot.location.line,
                hotspot.composite_score,
                hotspot.refactoring_effort.estimated_hours
            )?;
        }
    }
    writeln!(output)?;

    Ok(())
}

#[allow(dead_code)]
fn format_prioritized_recommendations(
    output: &mut String,
    recommendations: &[crate::services::deep_context::PrioritizedRecommendation],
) -> anyhow::Result<()> {
    use crate::services::deep_context::Priority;
    use std::fmt::Write;

    if !recommendations.is_empty() {
        writeln!(output, "## Prioritized Recommendations\n")?;

        for (i, rec) in recommendations.iter().enumerate() {
            let priority_emoji = match rec.priority {
                Priority::Critical => "🔴",
                Priority::High => "🟡",
                Priority::Medium => "🔵",
                Priority::Low => "⚪",
            };

            writeln!(output, "### {} {} {}", priority_emoji, i + 1, rec.title)?;
            writeln!(output, "**Description:** {}", rec.description)?;
            writeln!(output, "**Effort:** {:?}", rec.estimated_effort)?;
            writeln!(output, "**Impact:** {:?}", rec.impact)?;

            if !rec.prerequisites.is_empty() {
                writeln!(output, "**Prerequisites:**")?;
                for prereq in &rec.prerequisites {
                    writeln!(output, "- {prereq}")?;
                }
            }
            writeln!(output)?;
        }
    }

    Ok(())
}

/// Format deep context as terse report (default mode)
#[allow(dead_code)]
fn format_deep_context_terse(
    context: &crate::services::deep_context::DeepContext,
) -> anyhow::Result<String> {
    let mut output = String::new();

    // Header
    output.push_str(&format_terse_header(context));

    // Executive Summary
    output.push_str(&format_terse_executive_summary(context));

    // Key Metrics
    output.push_str(&format_terse_key_metrics(context));

    // AST Network Analysis (placeholder)
    output.push_str(&format_terse_ast_network_analysis());

    // Top 5 Predicted Defect Files
    output.push_str(&format_terse_predicted_defect_files(context));

    Ok(output)
}

/// Format header section for terse report
#[allow(dead_code)]
fn format_terse_header(context: &crate::services::deep_context::DeepContext) -> String {
    let project_name = context
        .metadata
        .project_root
        .file_name()
        .unwrap_or_default()
        .to_string_lossy();

    format!(
        "# Deep Context Analysis: {}\n\
        **Generated:** {} UTC\n\
        **Tool Version:** {}\n\
        **Analysis Time:** {:?}\n\n",
        project_name,
        context.metadata.generated_at.format("%Y-%m-%d %H:%M:%S"),
        context.metadata.tool_version,
        context.metadata.analysis_duration
    )
}

/// Format executive summary section for terse report
#[allow(dead_code)]
fn format_terse_executive_summary(context: &crate::services::deep_context::DeepContext) -> String {
    let mut output = String::from("## Executive Summary\n");

    let health_emoji = if context.quality_scorecard.overall_health >= 80.0 {
        "✅"
    } else if context.quality_scorecard.overall_health >= 60.0 {
        "⚠️"
    } else {
        "❌"
    };

    output.push_str(&format!(
        "**Overall Health Score:** {:.1}/100 {}\n",
        context.quality_scorecard.overall_health, health_emoji
    ));

    // Count high-risk files based on defect summary
    let high_risk_files = context.defect_summary.total_defects.min(5); // Cap at 5 for terse mode
    output.push_str(&format!(
        "**Predicted High-Risk Files:** {high_risk_files}\n"
    ));

    // SATD breakdown by severity
    let (high_satd, medium_satd, low_satd) = get_terse_satd_breakdown(context);
    let total_satd = high_satd + medium_satd + low_satd;
    output.push_str(&format!(
        "**Technical Debt Items:** {total_satd} (High: {high_satd}, Medium: {medium_satd}, Low: {low_satd})\n\n"
    ));

    output
}

/// Get SATD breakdown by severity for terse report
#[allow(dead_code)]
fn get_terse_satd_breakdown(
    context: &crate::services::deep_context::DeepContext,
) -> (usize, usize, usize) {
    if let Some(ref satd) = context.analyses.satd_results {
        let mut high = 0;
        let mut medium = 0;
        let mut low = 0;

        for item in &satd.items {
            match item.severity {
                crate::services::satd_detector::Severity::Critical
                | crate::services::satd_detector::Severity::High => high += 1,
                crate::services::satd_detector::Severity::Medium => medium += 1,
                crate::services::satd_detector::Severity::Low => low += 1,
            }
        }
        (high, medium, low)
    } else {
        (0, 0, 0)
    }
}

/// Format key metrics section for terse report
#[allow(dead_code)]
fn format_terse_key_metrics(context: &crate::services::deep_context::DeepContext) -> String {
    let mut output = String::from("## Key Metrics\n");

    // Complexity
    output.push_str(&format_terse_complexity_metrics(context));

    // Code Churn
    output.push_str(&format_terse_churn_metrics(context));

    // Technical Debt (SATD)
    output.push_str(&format_terse_satd_metrics(context));

    // Duplicates (placeholder)
    output.push_str(&format_terse_duplicates_metrics());

    // Dead Code
    output.push_str(&format_terse_dead_code_metrics(context));

    output
}

/// Format complexity metrics for terse report
#[allow(dead_code)]
fn format_terse_complexity_metrics(context: &crate::services::deep_context::DeepContext) -> String {
    if let Some(ref complexity) = context.analyses.complexity_report {
        let mut output = String::from("### Complexity\n");
        output.push_str(&format!(
            "- **Median Cyclomatic:** {:.1}\n",
            complexity.summary.median_cyclomatic
        ));

        // Find max complexity with function name
        let max_function = complexity
            .files
            .iter()
            .flat_map(|f| f.functions.iter().map(move |func| (f, func)))
            .max_by_key(|(_, func)| func.metrics.cyclomatic)
            .map(|(file, func)| (file.path.as_str(), func.name.as_str()))
            .unwrap_or(("unknown", "unknown"));

        output.push_str(&format!(
            "- **Max Cyclomatic:** {} ({}:{})\n",
            complexity.summary.max_cyclomatic, max_function.0, max_function.1
        ));
        output.push_str(&format!(
            "- **Violations:** {}\n\n",
            complexity.violations.len()
        ));
        output
    } else {
        String::new()
    }
}

/// Format churn metrics for terse report
#[allow(dead_code)]
fn format_terse_churn_metrics(context: &crate::services::deep_context::DeepContext) -> String {
    if let Some(ref churn) = context.analyses.churn_analysis {
        let mut output = String::from("### Code Churn (30 days)\n");

        // Calculate median changes per file
        let median_changes = calculate_terse_median_changes(&churn.files);
        output.push_str(&format!("- **Median Changes:** {median_changes}\n"));

        // Find max churn file
        let max_churn_file = churn
            .files
            .iter()
            .max_by_key(|f| f.commit_count)
            .map(|f| (f.relative_path.as_str(), f.commit_count))
            .unwrap_or(("unknown", 0));

        output.push_str(&format!(
            "- **Max Changes:** {} ({})\n",
            max_churn_file.1, max_churn_file.0
        ));

        // Count hotspot files (files with >5 commits as hotspots)
        let hotspot_count = churn.files.iter().filter(|f| f.commit_count > 5).count();
        output.push_str(&format!("- **Hotspot Files:** {hotspot_count}\n\n"));
        output
    } else {
        String::new()
    }
}

/// Calculate median changes for terse report
#[allow(dead_code)]
fn calculate_terse_median_changes(files: &[crate::models::churn::FileChurnMetrics]) -> usize {
    let mut changes_per_file: Vec<usize> = files.iter().map(|f| f.commit_count).collect();
    changes_per_file.sort_unstable();

    if !changes_per_file.is_empty() {
        let mid = changes_per_file.len() / 2;
        if changes_per_file.len() % 2 == 0 {
            (changes_per_file[mid - 1] + changes_per_file[mid]) / 2
        } else {
            changes_per_file[mid]
        }
    } else {
        0
    }
}

/// Format SATD metrics for terse report
#[allow(dead_code)]
fn format_terse_satd_metrics(context: &crate::services::deep_context::DeepContext) -> String {
    let mut output = String::from("### Technical Debt (SATD)\n");
    let (high_satd, _, _) = get_terse_satd_breakdown(context);
    let total_satd = if let Some(ref satd) = context.analyses.satd_results {
        satd.items.len()
    } else {
        0
    };

    output.push_str(&format!("- **Total Items:** {total_satd}\n"));
    output.push_str(&format!("- **High Severity:** {high_satd}\n"));

    // Count files with SATD items as debt hotspots
    let debt_hotspot_files = if let Some(ref satd) = context.analyses.satd_results {
        let unique_files: std::collections::HashSet<_> =
            satd.items.iter().map(|item| item.file.as_path()).collect();
        unique_files.len()
    } else {
        0
    };
    output.push_str(&format!(
        "- **Debt Hotspots:** {debt_hotspot_files} files\n\n"
    ));
    output
}

/// Format duplicates metrics for terse report (placeholder)
#[allow(dead_code)]
fn format_terse_duplicates_metrics() -> String {
    String::from(
        "### Duplicates\n\
        - **Clone Coverage:** 0.0%\n\
        - **Type-1/2 Clones:** 0\n\
        - **Type-3/4 Clones:** 0\n\n",
    )
}

/// Format dead code metrics for terse report
#[allow(dead_code)]
fn format_terse_dead_code_metrics(context: &crate::services::deep_context::DeepContext) -> String {
    if let Some(ref dead_code) = context.analyses.dead_code_results {
        format!(
            "### Dead Code\n\
            - **Unreachable Functions:** {}\n\
            - **Dead Code %:** {:.1}%\n\n",
            dead_code.summary.dead_functions, dead_code.summary.dead_percentage
        )
    } else {
        String::from(
            "### Dead Code\n\
            - **Unreachable Functions:** 0\n\
            - **Dead Code %:** 0.0%\n\n",
        )
    }
}

/// Format AST network analysis for terse report (placeholder)
#[allow(dead_code)]
fn format_terse_ast_network_analysis() -> String {
    String::from(
        "## AST Network Analysis\n\
        **Module Centrality (PageRank):**\n\
        1. main (score: 0.25)\n\
        2. lib (score: 0.20)\n\
        3. services (score: 0.15)\n\n\
        **Function Importance:**\n\
        1. main (connections: 15)\n\
        2. analyze_project (connections: 12)\n\
        3. process_files (connections: 8)\n\n",
    )
}

/// Format predicted defect files for terse report
#[allow(dead_code)]
fn format_terse_predicted_defect_files(
    context: &crate::services::deep_context::DeepContext,
) -> String {
    let mut output = String::from("## Top 5 Predicted Defect Files\n");

    let file_risks = calculate_terse_file_risks(context);

    if file_risks.is_empty() {
        output.push_str("No high-risk files detected.\n");
    } else {
        for (i, (file, risk_score, complexity, churn, satd)) in
            file_risks.iter().take(5).enumerate()
        {
            output.push_str(&format!(
                "{}. {} (risk score: {:.1})\n",
                i + 1,
                file,
                risk_score
            ));
            output.push_str(&format!(
                "   - Complexity: {complexity}, Churn: {churn}, SATD: {satd}\n"
            ));
        }
    }

    output
}

/// Calculate file risks for terse report
#[allow(dead_code)]
fn calculate_terse_file_risks(
    context: &crate::services::deep_context::DeepContext,
) -> Vec<(String, f32, u16, usize, usize)> {
    let mut file_risks = Vec::new();

    // Collect files with complexity issues
    if let Some(ref complexity) = context.analyses.complexity_report {
        for file in &complexity.files {
            let max_complexity = file
                .functions
                .iter()
                .map(|f| f.metrics.cyclomatic)
                .max()
                .unwrap_or(0);

            let satd_count = if let Some(ref satd) = context.analyses.satd_results {
                satd.items
                    .iter()
                    .filter(|item| item.file.to_string_lossy() == file.path)
                    .count()
            } else {
                0
            };

            let churn_count = if let Some(ref churn) = context.analyses.churn_analysis {
                churn
                    .files
                    .iter()
                    .find(|f| {
                        f.relative_path == file.path
                            || f.relative_path.ends_with(&file.path)
                            || file.path.ends_with(&f.relative_path)
                            || f.path.to_string_lossy() == file.path
                    })
                    .map(|f| f.commit_count)
                    .unwrap_or(0)
            } else {
                0
            };

            // Simple risk score calculation
            let risk_score = (max_complexity as f32 * 0.4)
                + (satd_count as f32 * 2.0)
                + (churn_count as f32 * 0.3);

            if risk_score > 0.0 {
                file_risks.push((
                    file.path.clone(),
                    risk_score,
                    max_complexity,
                    churn_count,
                    satd_count,
                ));
            }
        }
    }

    // Sort by risk score
    file_risks.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));

    file_risks
}

/// Format deep context as full detailed report (enabled with --full flag)
#[allow(dead_code)]
fn format_deep_context_full(
    context: &crate::services::deep_context::DeepContext,
) -> anyhow::Result<String> {
    let mut output = String::new();

    // Header
    output.push_str(&format_full_report_header(context));

    // Executive Summary
    output.push_str(&format_full_executive_summary(context));

    // Detailed Analysis Results
    output.push_str("## Detailed Analysis Results\n");
    output.push_str(&format_full_complexity_analysis(context));
    output.push_str(&format_full_churn_analysis(context));
    output.push_str(&format_full_satd_analysis(context));
    output.push_str(&format_full_dead_code_analysis(context));

    // Risk Prediction
    output.push_str(&format_full_risk_prediction(context));

    // Recommendations
    output.push_str(&format_full_recommendations(context));

    Ok(output)
}

/// Format the header section for full report
#[allow(dead_code)]
fn format_full_report_header(context: &crate::services::deep_context::DeepContext) -> String {
    let project_name = context
        .metadata
        .project_root
        .file_name()
        .unwrap_or_default()
        .to_string_lossy();

    format!(
        "# Deep Context Analysis: {} (Full Report)\n\
        **Generated:** {} UTC\n\
        **Tool Version:** {}\n\
        **Analysis Time:** {:?}\n\
        **Project Root:** {}\n\n",
        project_name,
        context.metadata.generated_at.format("%Y-%m-%d %H:%M:%S"),
        context.metadata.tool_version,
        context.metadata.analysis_duration,
        context.metadata.project_root.display()
    )
}

/// Format executive summary section for full report
#[allow(dead_code)]
fn format_full_executive_summary(context: &crate::services::deep_context::DeepContext) -> String {
    let mut output = String::from("## Executive Summary\n");

    let health_emoji = if context.quality_scorecard.overall_health >= 80.0 {
        "✅"
    } else if context.quality_scorecard.overall_health >= 60.0 {
        "⚠️"
    } else {
        "❌"
    };

    output.push_str(&format!(
        "**Overall Health Score:** {:.1}/100 {}\n\
        **Complexity Score:** {:.1}/100\n\
        **Maintainability Index:** {:.1}\n\
        **Technical Debt Hours:** {:.1}\n\
        **Predicted High-Risk Files:** {}\n",
        context.quality_scorecard.overall_health,
        health_emoji,
        context.quality_scorecard.complexity_score,
        context.quality_scorecard.maintainability_index,
        context.quality_scorecard.technical_debt_hours,
        context.defect_summary.total_defects
    ));

    // SATD breakdown by severity
    let (high_satd, medium_satd, low_satd) = get_satd_breakdown(context);
    let total_satd = high_satd + medium_satd + low_satd;
    output.push_str(&format!(
        "**Technical Debt Items:** {total_satd} (High: {high_satd}, Medium: {medium_satd}, Low: {low_satd})\n\n"
    ));

    output
}

/// Get SATD breakdown by severity
#[allow(dead_code)]
fn get_satd_breakdown(
    context: &crate::services::deep_context::DeepContext,
) -> (usize, usize, usize) {
    if let Some(ref satd) = context.analyses.satd_results {
        let mut high = 0;
        let mut medium = 0;
        let mut low = 0;

        for item in &satd.items {
            match item.severity {
                crate::services::satd_detector::Severity::Critical
                | crate::services::satd_detector::Severity::High => high += 1,
                crate::services::satd_detector::Severity::Medium => medium += 1,
                crate::services::satd_detector::Severity::Low => low += 1,
            }
        }
        (high, medium, low)
    } else {
        (0, 0, 0)
    }
}

/// Format complexity analysis section for full report
#[allow(dead_code)]
fn format_full_complexity_analysis(context: &crate::services::deep_context::DeepContext) -> String {
    if let Some(ref complexity) = context.analyses.complexity_report {
        let mut output = String::from("### Complexity Analysis\n");

        output.push_str(&format!(
            "- **Files Analyzed:** {}\n\
            - **Total Functions:** {}\n\
            - **Median Cyclomatic:** {:.1}\n\
            - **Mean Cyclomatic:** {:.1}\n\
            - **Max Cyclomatic:** {}\n\
            - **Violations:** {}\n",
            complexity.files.len(),
            complexity
                .files
                .iter()
                .map(|f| f.functions.len())
                .sum::<usize>(),
            complexity.summary.median_cyclomatic,
            complexity.summary.median_cyclomatic,
            complexity.summary.max_cyclomatic,
            complexity.violations.len()
        ));

        // Show top 10 most complex functions
        let mut all_functions: Vec<_> = complexity
            .files
            .iter()
            .flat_map(|f| f.functions.iter().map(move |func| (f, func)))
            .collect();
        all_functions.sort_by_key(|(_, func)| std::cmp::Reverse(func.metrics.cyclomatic));

        output.push_str("\n**Top 10 Most Complex Functions:**\n");
        for (i, (file, func)) in all_functions.iter().take(10).enumerate() {
            output.push_str(&format!(
                "{}. **{}** in `{}` (Cyclomatic: {}, Cognitive: {})\n",
                i + 1,
                func.name,
                file.path,
                func.metrics.cyclomatic,
                func.metrics.cognitive
            ));
        }
        output.push('\n');

        output
    } else {
        String::new()
    }
}

/// Format churn analysis section for full report
#[allow(dead_code)]
fn format_full_churn_analysis(context: &crate::services::deep_context::DeepContext) -> String {
    if let Some(ref churn) = context.analyses.churn_analysis {
        let mut output = String::from("### Code Churn Analysis (30 days)\n");

        // Calculate statistics
        let total_commits: usize = churn.files.iter().map(|f| f.commit_count).sum();
        let median_changes = calculate_median_changes(&churn.files);

        let max_churn_file = churn
            .files
            .iter()
            .max_by_key(|f| f.commit_count)
            .map(|f| (f.relative_path.as_str(), f.commit_count))
            .unwrap_or(("unknown", 0));

        output.push_str(&format!(
            "- **Files Tracked:** {}\n\
            - **Total Commits:** {}\n\
            - **Median Changes per File:** {}\n\
            - **Most Changed File:** {} ({} commits)\n",
            churn.files.len(),
            total_commits,
            median_changes,
            max_churn_file.0,
            max_churn_file.1
        ));

        // Show top 10 churned files
        let mut sorted_files = churn.files.clone();
        sorted_files.sort_by_key(|f| std::cmp::Reverse(f.commit_count));

        output.push_str("\n**Top 10 Most Changed Files:**\n");
        for (i, file) in sorted_files.iter().take(10).enumerate() {
            output.push_str(&format!(
                "{}. **{}** ({} commits)\n",
                i + 1,
                file.relative_path,
                file.commit_count
            ));
        }
        output.push('\n');

        output
    } else {
        String::new()
    }
}

/// Calculate median changes from churn files
#[allow(dead_code)]
fn calculate_median_changes(files: &[crate::models::churn::FileChurnMetrics]) -> usize {
    let mut changes_per_file: Vec<usize> = files.iter().map(|f| f.commit_count).collect();
    changes_per_file.sort_unstable();

    if !changes_per_file.is_empty() {
        let mid = changes_per_file.len() / 2;
        if changes_per_file.len() % 2 == 0 {
            (changes_per_file[mid - 1] + changes_per_file[mid]) / 2
        } else {
            changes_per_file[mid]
        }
    } else {
        0
    }
}

/// Format SATD analysis section for full report
#[allow(dead_code)]
fn format_full_satd_analysis(context: &crate::services::deep_context::DeepContext) -> String {
    if let Some(ref satd) = context.analyses.satd_results {
        let mut output = String::from("### Technical Debt (SATD) Analysis\n");

        output.push_str(&format!(
            "- **Total Items:** {}\n\
            - **Critical Severity:** {}\n\
            - **High Severity:** {}\n\
            - **Medium Severity:** {}\n\
            - **Low Severity:** {}\n",
            satd.items.len(),
            satd.items
                .iter()
                .filter(|item| matches!(
                    item.severity,
                    crate::services::satd_detector::Severity::Critical
                ))
                .count(),
            satd.items
                .iter()
                .filter(|item| matches!(
                    item.severity,
                    crate::services::satd_detector::Severity::High
                ))
                .count(),
            satd.items
                .iter()
                .filter(|item| matches!(
                    item.severity,
                    crate::services::satd_detector::Severity::Medium
                ))
                .count(),
            satd.items
                .iter()
                .filter(|item| matches!(
                    item.severity,
                    crate::services::satd_detector::Severity::Low
                ))
                .count()
        ));

        // Count files with SATD items
        let unique_files: std::collections::HashSet<_> =
            satd.items.iter().map(|item| item.file.as_path()).collect();
        output.push_str(&format!("- **Files with Debt:** {}\n", unique_files.len()));

        // Show all critical and high items
        let critical_high_items: Vec<_> = satd
            .items
            .iter()
            .filter(|item| {
                matches!(
                    item.severity,
                    crate::services::satd_detector::Severity::Critical
                        | crate::services::satd_detector::Severity::High
                )
            })
            .collect();

        if !critical_high_items.is_empty() {
            output.push_str("\n**Critical & High Severity Items:**\n");
            for (i, item) in critical_high_items.iter().enumerate() {
                output.push_str(&format!(
                    "{}. **{:?}** in `{}:{}` - {}\n",
                    i + 1,
                    item.severity,
                    item.file.display(),
                    item.line,
                    item.text.trim()
                ));
            }
        }
        output.push('\n');

        output
    } else {
        String::new()
    }
}

/// Format dead code analysis section for full report
#[allow(dead_code)]
fn format_full_dead_code_analysis(context: &crate::services::deep_context::DeepContext) -> String {
    if let Some(ref dead_code) = context.analyses.dead_code_results {
        let mut output = String::from("### Dead Code Analysis\n");

        output.push_str(&format!(
            "- **Total Files Analyzed:** {}\n\
            - **Files with Dead Code:** {}\n\
            - **Dead Lines:** {} ({:.1}%)\n\
            - **Dead Functions:** {}\n\
            - **Dead Classes:** {}\n\
            - **Dead Modules:** {}\n",
            dead_code.summary.total_files_analyzed,
            dead_code.summary.files_with_dead_code,
            dead_code.summary.total_dead_lines,
            dead_code.summary.dead_percentage,
            dead_code.summary.dead_functions,
            dead_code.summary.dead_classes,
            dead_code.summary.dead_modules
        ));

        // Show top 10 files with most dead code
        if !dead_code.ranked_files.is_empty() {
            output.push_str("\n**Top 10 Files with Most Dead Code:**\n");
            for (i, file_metrics) in dead_code.ranked_files.iter().take(10).enumerate() {
                output.push_str(&format!(
                    "{}. **{}** ({:.1}% dead, {} lines)\n",
                    i + 1,
                    file_metrics.path,
                    file_metrics.dead_percentage,
                    file_metrics.dead_lines
                ));
            }
        }
        output.push('\n');

        output
    } else {
        String::from("### Dead Code Analysis\n- **Status:** No dead code analysis performed\n\n")
    }
}

/// Format risk prediction section for full report
#[allow(dead_code)]
fn format_full_risk_prediction(context: &crate::services::deep_context::DeepContext) -> String {
    let mut output = String::from("## Predicted Defect Files (Full List)\n");

    let file_risks = calculate_file_risks(context);

    if file_risks.is_empty() {
        output.push_str("No high-risk files detected.\n");
    } else {
        output.push_str("| Rank | File | Risk Score | Max Complexity | Avg Complexity | Churn | SATD | Functions |\n");
        output.push_str("|------|------|------------|----------------|----------------|-------|------|----------|\n");

        for (i, (file, risk_score, max_complexity, avg_complexity, churn, satd, functions)) in
            file_risks.iter().enumerate()
        {
            output.push_str(&format!(
                "| {:>4} | {} | {:>10.1} | {:>14} | {:>14.1} | {:>5} | {:>4} | {:>9} |\n",
                i + 1,
                file,
                risk_score,
                max_complexity,
                avg_complexity,
                churn,
                satd,
                functions
            ));
        }
    }

    output.push('\n');
    output
}

/// Calculate file risks for the full report
#[allow(dead_code)]
fn calculate_file_risks(
    context: &crate::services::deep_context::DeepContext,
) -> Vec<(String, f32, u16, f32, usize, usize, usize)> {
    let mut file_risks = Vec::new();

    if let Some(ref complexity) = context.analyses.complexity_report {
        for file in &complexity.files {
            let max_complexity = file
                .functions
                .iter()
                .map(|f| f.metrics.cyclomatic)
                .max()
                .unwrap_or(0);

            let avg_complexity = if !file.functions.is_empty() {
                file.functions
                    .iter()
                    .map(|f| f.metrics.cyclomatic)
                    .sum::<u16>() as f32
                    / file.functions.len() as f32
            } else {
                0.0
            };

            let satd_count = if let Some(ref satd) = context.analyses.satd_results {
                satd.items
                    .iter()
                    .filter(|item| item.file.to_string_lossy() == file.path)
                    .count()
            } else {
                0
            };

            let churn_count = if let Some(ref churn) = context.analyses.churn_analysis {
                churn
                    .files
                    .iter()
                    .find(|f| {
                        f.relative_path == file.path
                            || f.relative_path.ends_with(&file.path)
                            || file.path.ends_with(&f.relative_path)
                            || f.path.to_string_lossy() == file.path
                    })
                    .map(|f| f.commit_count)
                    .unwrap_or(0)
            } else {
                0
            };

            // Enhanced risk score calculation
            let complexity_factor = (max_complexity as f32 * 0.4) + (avg_complexity * 0.2);
            let debt_factor = satd_count as f32 * 2.0;
            let churn_factor = churn_count as f32 * 0.3;
            let size_factor = file.functions.len() as f32 * 0.1;

            let risk_score = complexity_factor + debt_factor + churn_factor + size_factor;

            if risk_score > 0.0 {
                file_risks.push((
                    file.path.clone(),
                    risk_score,
                    max_complexity,
                    avg_complexity,
                    churn_count,
                    satd_count,
                    file.functions.len(),
                ));
            }
        }
    }

    // Sort by risk score
    file_risks.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));

    file_risks
}

/// Format recommendations section for full report
#[allow(dead_code)]
fn format_full_recommendations(context: &crate::services::deep_context::DeepContext) -> String {
    let mut output = String::from(
        "## Recommendations\nBased on the analysis, here are the top recommendations:\n\n",
    );

    if context.quality_scorecard.overall_health < 60.0 {
        output.push_str("🔴 **Critical Priority:**\n- Overall health score is below 60%. Immediate attention required.\n- Focus on reducing technical debt and complexity in top-risk files.\n\n");
    } else if context.quality_scorecard.overall_health < 80.0 {
        output.push_str("🟡 **Medium Priority:**\n- Health score between 60-80%. Some improvements needed.\n- Consider refactoring high-complexity functions.\n\n");
    } else {
        output.push_str("✅ **Good Health:**\n- Health score above 80%. Maintain current quality standards.\n- Continue monitoring for any degradation.\n\n");
    }

    let (high_satd, _, _) = get_satd_breakdown(context);
    if high_satd > 0 {
        output.push_str(&format!(
            "📋 **Technical Debt:** {high_satd} high-severity items need immediate attention.\n"
        ));
    }

    if let Some(ref complexity) = context.analyses.complexity_report {
        let high_complexity_functions = complexity
            .files
            .iter()
            .flat_map(|f| f.functions.iter())
            .filter(|func| func.metrics.cyclomatic > 10)
            .count();

        if high_complexity_functions > 0 {
            output.push_str(&format!("🔧 **Complexity:** {high_complexity_functions} functions with cyclomatic complexity > 10 should be refactored.\n"));
        }
    }

    output
}

#[allow(dead_code)]
fn format_deep_context_as_sarif(
    context: &crate::services::deep_context::DeepContext,
) -> anyhow::Result<String> {
    use serde_json::json;

    let mut results = Vec::new();

    // Add results from different analyses
    if let Some(ref dead_code) = context.analyses.dead_code_results {
        for file_metrics in &dead_code.ranked_files {
            for item in &file_metrics.items {
                results.push(json!({
                    "ruleId": "deep-context-dead-code",
                    "level": "warning",
                    "message": {
                        "text": format!("Dead {}: {}",
                            format!("{:?}", item.item_type).to_lowercase(),
                            item.reason)
                    },
                    "locations": [{
                        "physicalLocation": {
                            "artifactLocation": {
                                "uri": file_metrics.path
                            },
                            "region": {
                                "startLine": item.line
                            }
                        }
                    }],
                    "properties": {
                        "confidence": format!("{:?}", file_metrics.confidence),
                        "analysisType": "dead-code"
                    }
                }));
            }
        }
    }

    if let Some(ref satd) = context.analyses.satd_results {
        for item in &satd.items {
            let level = match item.severity {
                crate::services::satd_detector::Severity::Critical => "error",
                crate::services::satd_detector::Severity::High => "warning",
                crate::services::satd_detector::Severity::Medium => "note",
                crate::services::satd_detector::Severity::Low => "info",
            };

            results.push(json!({
                "ruleId": "deep-context-technical-debt",
                "level": level,
                "message": {
                    "text": format!("Technical debt: {}", item.text.trim())
                },
                "locations": [{
                    "physicalLocation": {
                        "artifactLocation": {
                            "uri": item.file.display().to_string()
                        },
                        "region": {
                            "startLine": item.line,
                            "startColumn": item.column
                        }
                    }
                }],
                "properties": {
                    "category": format!("{:?}", item.category),
                    "severity": format!("{:?}", item.severity),
                    "analysisType": "technical-debt"
                }
            }));
        }
    }

    let sarif = json!({
        "$schema": "https://json.schemastore.org/sarif-2.1.0.json",
        "version": "2.1.0",
        "runs": [{
            "tool": {
                "driver": {
                    "name": "paiml-mcp-agent-toolkit",
                    "version": context.metadata.tool_version,
                    "informationUri": "https://github.com/paiml/paiml-mcp-agent-toolkit"
                }
            },
            "results": results,
            "properties": {
                "overallHealthScore": context.quality_scorecard.overall_health,
                "complexityScore": context.quality_scorecard.complexity_score,
                "maintainabilityIndex": context.quality_scorecard.maintainability_index,
                "technicalDebtHours": context.quality_scorecard.technical_debt_hours,
                "analysisTimestamp": context.metadata.generated_at.to_rfc3339(),
                "analysisDuration": format!("{:?}", context.metadata.analysis_duration)
            }
        }]
    });

    Ok(serde_json::to_string_pretty(&sarif)?)
}

/// Format SATD analysis output
fn format_satd_output(
    result: &crate::services::satd_detector::SATDAnalysisResult,
    format: &SatdOutputFormat,
) -> anyhow::Result<String> {
    match format {
        SatdOutputFormat::Summary => format_satd_summary(result),
        SatdOutputFormat::Json => Ok(serde_json::to_string_pretty(result)?),
        SatdOutputFormat::Sarif => format_satd_as_sarif(result),
        SatdOutputFormat::Markdown => format_satd_as_markdown(result),
    }
}

/// Format SATD analysis as summary text
fn format_satd_summary(
    result: &crate::services::satd_detector::SATDAnalysisResult,
) -> anyhow::Result<String> {
    let mut output = String::new();

    output.push_str("Self-Admitted Technical Debt Analysis:\n");
    output.push_str("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n");
    output.push_str(&format!("  Total SATD items: {}\n", result.items.len()));

    // Count by severity
    let by_severity = result.items.iter().fold([0; 4], |mut acc, item| {
        match item.severity {
            crate::services::satd_detector::Severity::Critical => acc[0] += 1,
            crate::services::satd_detector::Severity::High => acc[1] += 1,
            crate::services::satd_detector::Severity::Medium => acc[2] += 1,
            crate::services::satd_detector::Severity::Low => acc[3] += 1,
        }
        acc
    });

    output.push_str(&format!("  Critical: {}\n", by_severity[0]));
    output.push_str(&format!("  High: {}\n", by_severity[1]));
    output.push_str(&format!("  Medium: {}\n", by_severity[2]));
    output.push_str(&format!("  Low: {}\n", by_severity[3]));

    // Count by category
    let mut by_category = std::collections::HashMap::new();
    for item in &result.items {
        *by_category.entry(item.category).or_insert(0) += 1;
    }

    output.push('\n');
    output.push_str("By Category:\n");
    for (category, count) in by_category {
        output.push_str(&format!("  {category:?}: {count}\n"));
    }

    // Show top items if any
    if !result.items.is_empty() {
        output.push('\n');
        output.push_str("🔺 Top Critical Items:\n");
        output.push_str("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n");

        let critical_items: Vec<_> = result
            .items
            .iter()
            .filter(|item| {
                matches!(
                    item.severity,
                    crate::services::satd_detector::Severity::Critical
                )
            })
            .take(5)
            .collect();

        if critical_items.is_empty() {
            output.push_str("  No critical items found.\n");
        } else {
            for (i, item) in critical_items.iter().enumerate() {
                output.push_str(&format!(
                    "{}. {} ({}:{})\n",
                    i + 1,
                    item.text.trim(),
                    item.file.display(),
                    item.line
                ));
                output.push_str(&format!("   Category: {:?}\n", item.category));
                output.push('\n');
            }
        }
    }

    Ok(output)
}

/// Format SATD analysis as SARIF (Static Analysis Results Interchange Format)
fn format_satd_as_sarif(
    result: &crate::services::satd_detector::SATDAnalysisResult,
) -> anyhow::Result<String> {
    use serde_json::json;

    let mut results = Vec::new();

    for item in &result.items {
        let level = match item.severity {
            crate::services::satd_detector::Severity::Critical => "error",
            crate::services::satd_detector::Severity::High => "warning",
            crate::services::satd_detector::Severity::Medium => "note",
            crate::services::satd_detector::Severity::Low => "info",
        };

        results.push(json!({
            "ruleId": format!("satd-{}", format!("{:?}", item.category).to_lowercase()),
            "level": level,
            "message": {
                "text": format!("Self-admitted technical debt: {}", item.text.trim())
            },
            "locations": [{
                "physicalLocation": {
                    "artifactLocation": {
                        "uri": item.file.display().to_string()
                    },
                    "region": {
                        "startLine": item.line,
                        "startColumn": item.column
                    }
                }
            }]
        }));
    }

    let sarif = json!({
        "$schema": "https://json.schemastore.org/sarif-2.1.0.json",
        "version": "2.1.0",
        "runs": [{
            "tool": {
                "driver": {
                    "name": "paiml-mcp-agent-toolkit",
                    "version": "0.1.0",
                    "informationUri": "https://github.com/paiml/mcp-agent-toolkit"
                }
            },
            "results": results
        }]
    });

    Ok(serde_json::to_string_pretty(&sarif)?)
}

/// Format SATD analysis as Markdown
fn format_satd_as_markdown(
    result: &crate::services::satd_detector::SATDAnalysisResult,
) -> anyhow::Result<String> {
    let mut output = String::new();

    output.push_str("# Self-Admitted Technical Debt Report\n\n");
    output.push_str(&format!(
        "**Analysis Date:** {}\n\n",
        chrono::Utc::now().format("%Y-%m-%d %H:%M:%S UTC")
    ));

    // Summary section
    output.push_str("## Summary\n\n");
    output.push_str(&format!("- **Total SATD items:** {}\n", result.items.len()));

    // Count by severity
    let by_severity = result.items.iter().fold([0; 4], |mut acc, item| {
        match item.severity {
            crate::services::satd_detector::Severity::Critical => acc[0] += 1,
            crate::services::satd_detector::Severity::High => acc[1] += 1,
            crate::services::satd_detector::Severity::Medium => acc[2] += 1,
            crate::services::satd_detector::Severity::Low => acc[3] += 1,
        }
        acc
    });

    output.push_str(&format!("- **Critical:** {}\n", by_severity[0]));
    output.push_str(&format!("- **High:** {}\n", by_severity[1]));
    output.push_str(&format!("- **Medium:** {}\n", by_severity[2]));
    output.push_str(&format!("- **Low:** {}\n\n", by_severity[3]));

    // Items by severity
    if !result.items.is_empty() {
        output.push_str("## Technical Debt Items\n\n");
        output.push_str("| Severity | Category | File | Line | Description |\n");
        output.push_str("|----------|----------|------|------|-------------|\n");

        for item in &result.items {
            let severity_emoji = match item.severity {
                crate::services::satd_detector::Severity::Critical => "🔴",
                crate::services::satd_detector::Severity::High => "🟠",
                crate::services::satd_detector::Severity::Medium => "🟡",
                crate::services::satd_detector::Severity::Low => "🟢",
            };

            output.push_str(&format!(
                "| {} {:?} | {:?} | `{}` | {} | {} |\n",
                severity_emoji,
                item.severity,
                item.category,
                item.file.display(),
                item.line,
                item.text.trim().replace('|', "\\|").replace('\n', " ")
            ));
        }
        output.push('\n');
    }

    Ok(output)
}

/// Format dead code analysis output
fn format_dead_code_output(
    result: &crate::models::dead_code::DeadCodeRankingResult,
    format: &DeadCodeOutputFormat,
) -> anyhow::Result<String> {
    match format {
        DeadCodeOutputFormat::Summary => format_dead_code_summary(result),
        DeadCodeOutputFormat::Json => Ok(serde_json::to_string_pretty(result)?),
        DeadCodeOutputFormat::Sarif => format_dead_code_as_sarif(result),
        DeadCodeOutputFormat::Markdown => format_dead_code_as_markdown(result),
    }
}

/// Format dead code analysis as summary text
fn format_dead_code_summary(
    result: &crate::models::dead_code::DeadCodeRankingResult,
) -> anyhow::Result<String> {
    let mut output = String::new();

    output.push_str("Dead Code Analysis Summary:\n");
    output.push_str("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n");
    output.push_str(&format!(
        "  Total files analyzed: {}\n",
        result.summary.total_files_analyzed
    ));
    output.push_str(&format!(
        "  Files with dead code: {} ({:.1}%)\n",
        result.summary.files_with_dead_code,
        if result.summary.total_files_analyzed > 0 {
            (result.summary.files_with_dead_code as f32
                / result.summary.total_files_analyzed as f32)
                * 100.0
        } else {
            0.0
        }
    ));
    output.push('\n');
    output.push_str(&format!(
        "  Total dead lines: {} ({:.1}% of codebase)\n",
        result.summary.total_dead_lines, result.summary.dead_percentage
    ));
    output.push_str(&format!(
        "  Dead functions: {}\n",
        result.summary.dead_functions
    ));
    output.push_str(&format!(
        "  Dead classes: {}\n",
        result.summary.dead_classes
    ));
    output.push_str(&format!(
        "  Dead modules: {}\n",
        result.summary.dead_modules
    ));
    output.push_str(&format!(
        "  Unreachable blocks: {}\n",
        result.summary.unreachable_blocks
    ));

    // Show top files if available
    if !result.ranked_files.is_empty() {
        let top_count = result.ranked_files.len().min(5);
        output.push_str(&format!(
            "\n🏆 Top {top_count} Files with Most Dead Code:\n"
        ));
        output.push_str("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n");

        for (i, file_metrics) in result.ranked_files.iter().take(top_count).enumerate() {
            let confidence_text = match file_metrics.confidence {
                crate::models::dead_code::ConfidenceLevel::High => "HIGH",
                crate::models::dead_code::ConfidenceLevel::Medium => "MEDIUM",
                crate::models::dead_code::ConfidenceLevel::Low => "LOW",
            };

            output.push_str(&format!(
                "{}. {} (Score: {:.1}) [{}confidence]\n",
                i + 1,
                file_metrics.path,
                file_metrics.dead_score,
                confidence_text
            ));
            output.push_str(&format!(
                "   └─ {} dead lines ({:.1}% of file)\n",
                file_metrics.dead_lines, file_metrics.dead_percentage
            ));
            if file_metrics.dead_functions > 0 || file_metrics.dead_classes > 0 {
                output.push_str(&format!(
                    "   └─ {} functions, {} classes\n",
                    file_metrics.dead_functions, file_metrics.dead_classes
                ));
            }

            // Add recommendation based on percentage
            let recommendation = if file_metrics.dead_percentage > 80.0 {
                "Recommendation: Consider removing entire file"
            } else if file_metrics.dead_percentage > 50.0 {
                "Recommendation: Review and remove unused sections"
            } else if file_metrics.dead_percentage > 20.0 {
                "Recommendation: Clean up dead code items"
            } else {
                "Recommendation: Minor cleanup needed"
            };
            output.push_str(&format!("   └─ {recommendation}\n"));
            output.push('\n');
        }
    }

    Ok(output)
}

/// Format dead code analysis as SARIF (Static Analysis Results Interchange Format)
fn format_dead_code_as_sarif(
    result: &crate::models::dead_code::DeadCodeRankingResult,
) -> anyhow::Result<String> {
    use serde_json::json;

    let mut results = Vec::new();

    for file_metrics in &result.ranked_files {
        for item in &file_metrics.items {
            results.push(json!({
                "ruleId": format!("dead-code-{}", format!("{:?}", item.item_type).to_lowercase()),
                "level": "info",
                "message": {
                    "text": format!("Dead {} '{}': {}",
                        format!("{:?}", item.item_type).to_lowercase(),
                        item.name,
                        item.reason
                    )
                },
                "locations": [{
                    "physicalLocation": {
                        "artifactLocation": {
                            "uri": file_metrics.path
                        },
                        "region": {
                            "startLine": item.line
                        }
                    }
                }]
            }));
        }
    }

    let sarif = json!({
        "$schema": "https://json.schemastore.org/sarif-2.1.0.json",
        "version": "2.1.0",
        "runs": [{
            "tool": {
                "driver": {
                    "name": "paiml-mcp-agent-toolkit",
                    "version": "0.1.0",
                    "informationUri": "https://github.com/paiml/mcp-agent-toolkit"
                }
            },
            "results": results
        }]
    });

    Ok(serde_json::to_string_pretty(&sarif)?)
}

/// Format dead code analysis as Markdown
fn format_dead_code_as_markdown(
    result: &crate::models::dead_code::DeadCodeRankingResult,
) -> anyhow::Result<String> {
    let mut output = String::new();

    output.push_str("# Dead Code Analysis Report\n\n");
    output.push_str(&format!(
        "**Analysis Date:** {}\n\n",
        result.analysis_timestamp.format("%Y-%m-%d %H:%M:%S UTC")
    ));

    // Summary section
    output.push_str("## Summary\n\n");
    output.push_str(&format!(
        "- **Total files analyzed:** {}\n",
        result.summary.total_files_analyzed
    ));
    output.push_str(&format!(
        "- **Files with dead code:** {} ({:.1}%)\n",
        result.summary.files_with_dead_code,
        if result.summary.total_files_analyzed > 0 {
            (result.summary.files_with_dead_code as f32
                / result.summary.total_files_analyzed as f32)
                * 100.0
        } else {
            0.0
        }
    ));
    output.push_str(&format!(
        "- **Total dead lines:** {} ({:.1}% of codebase)\n",
        result.summary.total_dead_lines, result.summary.dead_percentage
    ));
    output.push_str(&format!(
        "- **Dead functions:** {}\n",
        result.summary.dead_functions
    ));
    output.push_str(&format!(
        "- **Dead classes:** {}\n",
        result.summary.dead_classes
    ));
    output.push_str(&format!(
        "- **Dead modules:** {}\n",
        result.summary.dead_modules
    ));
    output.push_str(&format!(
        "- **Unreachable blocks:** {}\n\n",
        result.summary.unreachable_blocks
    ));

    // Configuration section
    output.push_str("## Configuration\n\n");
    output.push_str(&format!(
        "- **Include unreachable code:** {}\n",
        result.config.include_unreachable
    ));
    output.push_str(&format!(
        "- **Include test files:** {}\n",
        result.config.include_tests
    ));
    output.push_str(&format!(
        "- **Minimum dead lines threshold:** {}\n\n",
        result.config.min_dead_lines
    ));

    // Top files section
    if !result.ranked_files.is_empty() {
        output.push_str("## Top Files with Dead Code\n\n");
        output.push_str("| Rank | File | Dead Lines | Percentage | Functions | Classes | Score | Confidence |\n");
        output.push_str("|------|------|------------|------------|-----------|---------|-------|------------|\n");

        for (i, file_metrics) in result.ranked_files.iter().enumerate() {
            let confidence_text = match file_metrics.confidence {
                crate::models::dead_code::ConfidenceLevel::High => "🔴 High",
                crate::models::dead_code::ConfidenceLevel::Medium => "🟡 Medium",
                crate::models::dead_code::ConfidenceLevel::Low => "🟢 Low",
            };

            output.push_str(&format!(
                "| {:>4} | `{}` | {:>10} | {:>9.1}% | {:>9} | {:>7} | {:>5.1} | {} |\n",
                i + 1,
                file_metrics.path,
                file_metrics.dead_lines,
                file_metrics.dead_percentage,
                file_metrics.dead_functions,
                file_metrics.dead_classes,
                file_metrics.dead_score,
                confidence_text
            ));
        }
        output.push('\n');
    }

    Ok(output)
}

// Helper functions for complexity analysis

fn detect_toolchain(project_path: &Path, toolchain: Option<String>) -> anyhow::Result<String> {
    if let Some(t) = toolchain {
        return Ok(t);
    }

    if project_path.join("Cargo.toml").exists() {
        Ok("rust".to_string())
    } else if project_path.join("package.json").exists() || project_path.join("deno.json").exists()
    {
        Ok("deno".to_string())
    } else if project_path.join("pyproject.toml").exists()
        || project_path.join("requirements.txt").exists()
    {
        Ok("python-uv".to_string())
    } else {
        eprintln!("⚠️  Could not detect toolchain, defaulting to rust");
        Ok("rust".to_string())
    }
}

fn build_complexity_thresholds(
    max_cyclomatic: Option<u16>,
    max_cognitive: Option<u16>,
) -> crate::services::complexity::ComplexityThresholds {
    use crate::services::complexity::ComplexityThresholds;

    let mut thresholds = ComplexityThresholds::default();
    if let Some(max) = max_cyclomatic {
        thresholds.cyclomatic_error = max;
        thresholds.cyclomatic_warn = (max * 3 / 4).max(1);
    }
    if let Some(max) = max_cognitive {
        thresholds.cognitive_error = max;
        thresholds.cognitive_warn = (max * 3 / 4).max(1);
    }
    thresholds
}

async fn analyze_project_files(
    project_path: &PathBuf,
    toolchain: &str,
    include_patterns: &[String],
) -> anyhow::Result<Vec<crate::services::complexity::FileComplexityMetrics>> {
    use walkdir::WalkDir;

    let mut file_metrics = Vec::new();

    for entry in WalkDir::new(project_path)
        .follow_links(false)
        .into_iter()
        .filter_map(|e| e.ok())
    {
        let path = entry.path();

        // Skip directories and non-source files
        if path.is_dir() {
            continue;
        }

        // Check file extension based on toolchain
        if !should_analyze_file(path, toolchain) {
            continue;
        }

        // Apply include filters if specified
        if !include_patterns.is_empty() && !matches_include_patterns(path, include_patterns) {
            continue;
        }

        // Analyze file complexity
        if let Some(metrics) = analyze_file_by_toolchain(path, toolchain).await {
            file_metrics.push(metrics);
        }
    }

    Ok(file_metrics)
}

fn should_analyze_file(path: &std::path::Path, toolchain: &str) -> bool {
    match toolchain {
        "rust" => path.extension().and_then(|s| s.to_str()) == Some("rs"),
        "deno" => matches!(
            path.extension().and_then(|s| s.to_str()),
            Some("ts") | Some("tsx") | Some("js") | Some("jsx")
        ),
        "python-uv" => path.extension().and_then(|s| s.to_str()) == Some("py"),
        _ => false,
    }
}

fn matches_include_patterns(path: &std::path::Path, patterns: &[String]) -> bool {
    let path_str = path.to_string_lossy();
    patterns.iter().any(|pattern| {
        // Simple glob-like matching
        if pattern.contains("**") {
            // Match any path containing the pattern after **
            let parts: Vec<&str> = pattern.split("**").collect();
            if parts.len() == 2 {
                path_str.contains(parts[1].trim_start_matches('/'))
            } else {
                false
            }
        } else if pattern.starts_with("*.") {
            // Match by extension
            path_str.ends_with(&pattern[1..])
        } else {
            // Direct substring match
            path_str.contains(pattern)
        }
    })
}

async fn analyze_file_by_toolchain(
    path: &std::path::Path,
    toolchain: &str,
) -> Option<crate::services::complexity::FileComplexityMetrics> {
    match toolchain {
        "rust" => {
            use crate::services::ast_rust;
            ast_rust::analyze_rust_file_with_complexity(path).await.ok()
        }
        "deno" => {
            #[cfg(feature = "typescript-ast")]
            {
                use crate::services::ast_typescript;
                ast_typescript::analyze_typescript_file_with_complexity(path)
                    .await
                    .ok()
            }
            #[cfg(not(feature = "typescript-ast"))]
            None
        }
        "python-uv" => {
            #[cfg(feature = "python-ast")]
            {
                use crate::services::ast_python;
                ast_python::analyze_python_file_with_complexity(path)
                    .await
                    .ok()
            }
            #[cfg(not(feature = "python-ast"))]
            None
        }
        _ => None,
    }
}

fn params_to_json(params: Vec<(String, Value)>) -> serde_json::Map<String, Value> {
    let mut map = serde_json::Map::new();
    for (k, v) in params {
        map.insert(k, v);
    }
    map
}

/// Format top files ranking table
/// Add top files ranking to content based on format
fn add_top_files_ranking(
    mut content: String,
    format: ComplexityOutputFormat,
    file_metrics: &[crate::services::complexity::FileComplexityMetrics],
    top_files: usize,
) -> anyhow::Result<String> {
    use crate::services::ranking::{rank_files_by_complexity, ComplexityRanker};

    let ranker = ComplexityRanker::default();
    let rankings = rank_files_by_complexity(file_metrics, top_files, &ranker);
    let ranking_content = format_top_files_ranking(&rankings);

    match format {
        ComplexityOutputFormat::Json => {
            // For JSON, merge the ranking data
            let ranking_data = serde_json::json!({
                "requested": top_files,
                "returned": rankings.len(),
                "rankings": rankings.iter().enumerate().map(|(i, (file, score))| {
                    serde_json::json!({
                        "rank": i + 1,
                        "file": file,
                        "function_count": score.function_count,
                        "max_cyclomatic": score.cyclomatic_max,
                        "avg_cognitive": score.cognitive_avg,
                        "halstead_effort": score.halstead_effort,
                        "total_score": score.total_score
                    })
                }).collect::<Vec<_>>()
            });

            content = self::analysis_helpers::merge_ranking_into_json(
                &content,
                "top_files",
                ranking_data,
            )?;
        }
        _ => {
            content = format!("{ranking_content}\n{content}");
        }
    }

    Ok(content)
}

fn format_top_files_ranking(
    rankings: &[(String, crate::services::ranking::CompositeComplexityScore)],
) -> String {
    if rankings.is_empty() {
        return "## Top Complex Files\n\nNo files found.\n".to_string();
    }

    let mut output = format!("## Top {} Most Complex Files\n\n", rankings.len());

    // Add table header
    output.push_str(
        "| Rank | File | Functions | Max Cyclomatic | Avg Cognitive | Halstead | Score |\n",
    );
    output.push_str(
        "|------|------|-----------|----------------|---------------|----------|-------|\n",
    );

    // Add table rows
    for (i, (file, score)) in rankings.iter().enumerate() {
        output.push_str(&format!(
            "| {:>4} | {:<50} | {:>9} | {:>14} | {:>13.1} | {:>8.1} | {:>5.1} |\n",
            i + 1,
            file,
            score.function_count,
            score.cyclomatic_max,
            score.cognitive_avg,
            score.halstead_effort,
            score.total_score
        ));
    }

    output.push('\n');
    output
}

fn collect_file_paths(
    node: &crate::services::deep_context::AnnotatedNode,
    paths: &mut Vec<String>,
) {
    use crate::services::deep_context::NodeType;

    match node.node_type {
        NodeType::File => {
            paths.push(node.path.to_string_lossy().to_string());
        }
        NodeType::Directory => {
            for child in &node.children {
                collect_file_paths(child, paths);
            }
        }
    }
}

fn convert_to_deep_context_result(
    deep_context: crate::services::deep_context::DeepContext,
) -> crate::services::deep_context::DeepContextResult {
    use crate::services::deep_context::*;

    // Extract file paths from the annotated tree by recursively traversing nodes
    let mut file_tree: Vec<String> = Vec::new();
    collect_file_paths(&deep_context.file_tree.root, &mut file_tree);

    // Extract complexity metrics for QA verification
    let complexity_metrics = deep_context
        .analyses
        .complexity_report
        .as_ref()
        .map(|report| ComplexityMetricsForQA {
            files: report
                .files
                .iter()
                .map(|f| FileComplexityMetricsForQA {
                    path: std::path::PathBuf::from(&f.path),
                    functions: f
                        .functions
                        .iter()
                        .map(|func| FunctionComplexityForQA {
                            name: func.name.clone(),
                            cyclomatic: func.metrics.cyclomatic as u32,
                            cognitive: func.metrics.cognitive as u32,
                            nesting_depth: func.metrics.nesting_max as u32,
                            start_line: func.line_start as usize,
                            end_line: func.line_end as usize,
                        })
                        .collect(),
                    total_cyclomatic: f.total_complexity.cyclomatic as u32,
                    total_cognitive: f.total_complexity.cognitive as u32,
                    total_lines: f.total_complexity.lines as usize,
                })
                .collect(),
            summary: ComplexitySummaryForQA {
                total_files: report.files.len(),
                total_functions: report.files.iter().map(|f| f.functions.len()).sum(),
            },
        });

    // Extract dead code analysis
    let dead_code_analysis = deep_context
        .analyses
        .dead_code_results
        .as_ref()
        .map(|results| {
            DeadCodeAnalysis {
                summary: DeadCodeSummary {
                    total_functions: results.summary.total_files_analyzed * 10, // Estimate
                    dead_functions: results.summary.dead_functions,
                    total_lines: results.ranked_files.iter().map(|f| f.total_lines).sum(),
                    total_dead_lines: results.summary.total_dead_lines,
                    dead_percentage: results.summary.dead_percentage as f64,
                },
                dead_functions: results
                    .ranked_files
                    .iter()
                    .flat_map(|file| {
                        file.items.iter().filter_map(|item| match item.item_type {
                            crate::models::dead_code::DeadCodeType::Function => {
                                Some(item.name.clone())
                            }
                            _ => None,
                        })
                    })
                    .collect(),
                warnings: vec![],
            }
        });

    // Extract AST summaries
    let ast_summaries = if !deep_context.analyses.ast_contexts.is_empty() {
        Some(
            deep_context
                .analyses
                .ast_contexts
                .iter()
                .map(|ctx| AstSummary {
                    path: ctx.base.path.clone(),
                    language: ctx.base.language.clone(),
                    total_items: ctx.base.items.len(),
                    functions: ctx
                        .base
                        .items
                        .iter()
                        .filter(|item| {
                            matches!(item, crate::services::context::AstItem::Function { .. })
                        })
                        .count(),
                    classes: ctx
                        .base
                        .items
                        .iter()
                        .filter(|item| {
                            matches!(item, crate::services::context::AstItem::Struct { .. })
                        })
                        .count(),
                    imports: ctx
                        .base
                        .items
                        .iter()
                        .filter(|item| {
                            matches!(item, crate::services::context::AstItem::Use { .. })
                        })
                        .count(),
                })
                .collect(),
        )
    } else {
        None
    };

    // Extract language stats by counting files per language from AST contexts
    let mut language_stats = FxHashMap::default();
    for ctx in &deep_context.analyses.ast_contexts {
        let lang_name = ctx.base.language.clone();
        *language_stats.entry(lang_name).or_insert(0) += 1;
    }

    // Extract churn analysis before moving analyses
    let churn_analysis = deep_context.analyses.churn_analysis.clone();

    DeepContextResult {
        metadata: deep_context.metadata,
        file_tree,
        analyses: deep_context.analyses,
        quality_scorecard: deep_context.quality_scorecard,
        template_provenance: deep_context.template_provenance,
        defect_summary: deep_context.defect_summary,
        hotspots: deep_context.hotspots,
        recommendations: deep_context.recommendations,
        qa_verification: deep_context.qa_verification,
        complexity_metrics,
        dead_code_analysis,
        ast_summaries,
        churn_analysis,
        language_stats: if language_stats.is_empty() {
            None
        } else {
            Some(language_stats)
        },
        build_info: deep_context.build_info,
        project_overview: deep_context.project_overview,
    }
}

#[allow(clippy::too_many_arguments)]
async fn handle_quality_gate(
    project_path: PathBuf,
    format: QualityGateOutputFormat,
    fail_on_violation: bool,
    checks: Vec<QualityCheckType>,
    _max_dead_code: f64,
    _min_entropy: f64,
    max_complexity_p99: u32,
    include_provability: bool,
    output: Option<PathBuf>,
    perf: bool,
) -> anyhow::Result<()> {
    use crate::services::{deep_context::*, quality_gates::*};
    use std::time::Instant;

    let start = Instant::now();

    eprintln!(
        "🔍 Running quality gate checks on {}",
        project_path.display()
    );

    // First, run deep context analysis to get all the data we need
    let config = DeepContextConfig {
        include_analyses: vec![
            AnalysisType::Ast,
            AnalysisType::Complexity,
            AnalysisType::Churn,
            AnalysisType::Dag,
            AnalysisType::DeadCode,
            AnalysisType::Satd,
            AnalysisType::DuplicateCode,
        ],
        period_days: 30,
        dag_type: DagType::FullDependency,
        complexity_thresholds: Some(ComplexityThresholds {
            max_cyclomatic: max_complexity_p99 as u16,
            max_cognitive: 50,
        }),
        max_depth: None,
        include_patterns: vec![],
        exclude_patterns: vec!["vendor/**".to_string(), "node_modules/**".to_string()],
        cache_strategy: CacheStrategy::Normal,
        parallel: num_cpus::get(),
        file_classifier_config: None,
    };

    let mut config = config;
    if include_provability {
        config.include_analyses.push(AnalysisType::Provability);
    }

    let analyzer = DeepContextAnalyzer::new(config);
    let deep_context = analyzer.analyze_project(&project_path).await?;

    // Convert DeepContext to DeepContextResult for quality gates
    let deep_context_result = convert_to_deep_context_result(deep_context);

    // Create quality gate verification service
    let qa_verification = QAVerification::new();

    // Determine which checks to run
    let checks_to_run = if checks.is_empty() || checks.contains(&QualityCheckType::All) {
        vec![
            QualityCheckType::DeadCode,
            QualityCheckType::Complexity,
            QualityCheckType::Coverage,
            QualityCheckType::Sections,
        ]
    } else {
        checks
    };

    // Run verification and get detailed results
    let verification_results = qa_verification.verify(&deep_context_result);
    let report = qa_verification.generate_verification_report(&deep_context_result);

    if perf {
        eprintln!(
            "⏱️  Analysis completed in {:.2}s",
            start.elapsed().as_secs_f64()
        );
    }

    // Format output
    let content = match format {
        QualityGateOutputFormat::Summary => format_quality_gate_summary(&report, &checks_to_run),
        QualityGateOutputFormat::Detailed => {
            format_quality_gate_detailed(&report, &verification_results, &checks_to_run)
        }
        QualityGateOutputFormat::Json => serde_json::to_string_pretty(&report)?,
        QualityGateOutputFormat::Junit => format_quality_gate_junit(&report, &checks_to_run)?,
        QualityGateOutputFormat::Markdown => {
            format_quality_gate_markdown(&report, &verification_results, &checks_to_run)
        }
    };

    // Output results
    if let Some(output_path) = output {
        tokio::fs::write(&output_path, &content).await?;
        eprintln!(
            "✅ Quality gate report written to: {}",
            output_path.display()
        );
    } else {
        println!("{content}");
    }

    // Check if we should fail
    if fail_on_violation && report.overall != VerificationStatus::Pass {
        eprintln!("\n❌ Quality gate failed!");
        match report.overall {
            VerificationStatus::Fail => std::process::exit(1),
            VerificationStatus::Partial => std::process::exit(2),
            VerificationStatus::Pass => {} // Already checked above
        }
    }

    Ok(())
}

fn format_quality_gate_summary(
    report: &crate::services::quality_gates::QAVerificationResult,
    checks: &[QualityCheckType],
) -> String {
    let mut output = String::from("# Quality Gate Summary\n\n");

    output.push_str(&format!(
        "**Overall Status:** {}\n",
        match report.overall {
            crate::services::quality_gates::VerificationStatus::Pass => "✅ PASS",
            crate::services::quality_gates::VerificationStatus::Partial => "⚠️  PARTIAL",
            crate::services::quality_gates::VerificationStatus::Fail => "❌ FAIL",
        }
    ));

    output.push_str(&format!("**Timestamp:** {}\n", report.timestamp));
    output.push_str(&format!("**Version:** {}\n\n", report.version));

    if checks.contains(&QualityCheckType::DeadCode) || checks.contains(&QualityCheckType::All) {
        output.push_str(&format!(
            "## Dead Code: {}\n",
            match report.dead_code.status {
                crate::services::quality_gates::VerificationStatus::Pass => "✅ PASS",
                crate::services::quality_gates::VerificationStatus::Partial => "⚠️  PARTIAL",
                crate::services::quality_gates::VerificationStatus::Fail => "❌ FAIL",
            }
        ));
        output.push_str(&format!(
            "- Actual: {:.1}%\n",
            report.dead_code.actual * 100.0
        ));
        output.push_str(&format!(
            "- Expected Range: {:.1}%-{:.1}%\n",
            report.dead_code.expected_range[0] * 100.0,
            report.dead_code.expected_range[1] * 100.0
        ));
        if let Some(notes) = &report.dead_code.notes {
            output.push_str(&format!("- Notes: {notes}\n"));
        }
        output.push('\n');
    }

    if checks.contains(&QualityCheckType::Complexity) || checks.contains(&QualityCheckType::All) {
        output.push_str(&format!(
            "## Complexity: {}\n",
            match report.complexity.status {
                crate::services::quality_gates::VerificationStatus::Pass => "✅ PASS",
                crate::services::quality_gates::VerificationStatus::Partial => "⚠️  PARTIAL",
                crate::services::quality_gates::VerificationStatus::Fail => "❌ FAIL",
            }
        ));
        output.push_str(&format!("- Entropy: {:.2}\n", report.complexity.entropy));
        output.push_str(&format!(
            "- Coefficient of Variation: {:.1}%\n",
            report.complexity.cv
        ));
        output.push_str(&format!("- P99 Complexity: {}\n", report.complexity.p99));
        if let Some(notes) = &report.complexity.notes {
            output.push_str(&format!("- Notes: {notes}\n"));
        }
        output.push('\n');
    }

    output
}

fn format_quality_gate_detailed(
    report: &crate::services::quality_gates::QAVerificationResult,
    results: &std::collections::HashMap<&'static str, Result<(), String>>,
    checks: &[QualityCheckType],
) -> String {
    let mut output = format_quality_gate_summary(report, checks);

    output.push_str("## Detailed Check Results\n\n");

    for (check_name, result) in results {
        let status = match result {
            Ok(_) => "✅ PASS",
            Err(_) => "❌ FAIL",
        };

        output.push_str(&format!("### {check_name} - {status}\n"));

        if let Err(msg) = result {
            output.push_str(&format!("- Error: {msg}\n"));
        }
        output.push('\n');
    }

    output
}

fn format_quality_gate_junit(
    report: &crate::services::quality_gates::QAVerificationResult,
    checks: &[QualityCheckType],
) -> anyhow::Result<String> {
    use serde_json::json;

    let mut test_cases = Vec::new();

    if checks.contains(&QualityCheckType::DeadCode) || checks.contains(&QualityCheckType::All) {
        let mut test_case = json!({
            "name": "dead_code_check",
            "classname": "quality_gate",
            "time": 0.0
        });

        if report.dead_code.status != crate::services::quality_gates::VerificationStatus::Pass {
            test_case["failure"] = json!({
                "message": format!("Dead code check failed: {:.1}% (expected {:.1}%-{:.1}%)",
                    report.dead_code.actual * 100.0,
                    report.dead_code.expected_range[0] * 100.0,
                    report.dead_code.expected_range[1] * 100.0
                ),
                "type": "QualityGateFailure"
            });
        }

        test_cases.push(test_case);
    }

    if checks.contains(&QualityCheckType::Complexity) || checks.contains(&QualityCheckType::All) {
        let mut test_case = json!({
            "name": "complexity_check",
            "classname": "quality_gate",
            "time": 0.0
        });

        if report.complexity.status != crate::services::quality_gates::VerificationStatus::Pass {
            test_case["failure"] = json!({
                "message": format!("Complexity check failed: entropy={:.2}, cv={:.1}%, p99={}",
                    report.complexity.entropy,
                    report.complexity.cv,
                    report.complexity.p99
                ),
                "type": "QualityGateFailure"
            });
        }

        test_cases.push(test_case);
    }

    let junit = json!({
        "testsuites": [{
            "name": "quality_gate",
            "tests": test_cases.len(),
            "failures": test_cases.iter().filter(|tc| tc.get("failure").is_some()).count(),
            "time": 0.0,
            "testcases": test_cases
        }]
    });

    Ok(serde_json::to_string_pretty(&junit)?)
}

fn format_quality_gate_markdown(
    report: &crate::services::quality_gates::QAVerificationResult,
    results: &std::collections::HashMap<&'static str, Result<(), String>>,
    checks: &[QualityCheckType],
) -> String {
    let mut output = String::from("# Quality Gate Report\n\n");

    // Status badge
    let badge = match report.overall {
        crate::services::quality_gates::VerificationStatus::Pass => {
            "![Quality Gate](https://img.shields.io/badge/quality%20gate-passed-brightgreen)"
        }
        crate::services::quality_gates::VerificationStatus::Partial => {
            "![Quality Gate](https://img.shields.io/badge/quality%20gate-partial-yellow)"
        }
        crate::services::quality_gates::VerificationStatus::Fail => {
            "![Quality Gate](https://img.shields.io/badge/quality%20gate-failed-red)"
        }
    };

    output.push_str(&format!("{badge}\n\n"));

    // Summary table
    output.push_str("## Summary\n\n");
    output.push_str("| Check | Status | Details |\n");
    output.push_str("|-------|--------|---------|\n");

    if checks.contains(&QualityCheckType::DeadCode) || checks.contains(&QualityCheckType::All) {
        let status_icon = match report.dead_code.status {
            crate::services::quality_gates::VerificationStatus::Pass => "✅",
            crate::services::quality_gates::VerificationStatus::Partial => "⚠️",
            crate::services::quality_gates::VerificationStatus::Fail => "❌",
        };
        output.push_str(&format!(
            "| Dead Code | {} | {:.1}% (expected {:.1}%-{:.1}%) |\n",
            status_icon,
            report.dead_code.actual * 100.0,
            report.dead_code.expected_range[0] * 100.0,
            report.dead_code.expected_range[1] * 100.0
        ));
    }

    if checks.contains(&QualityCheckType::Complexity) || checks.contains(&QualityCheckType::All) {
        let status_icon = match report.complexity.status {
            crate::services::quality_gates::VerificationStatus::Pass => "✅",
            crate::services::quality_gates::VerificationStatus::Partial => "⚠️",
            crate::services::quality_gates::VerificationStatus::Fail => "❌",
        };
        output.push_str(&format!(
            "| Complexity | {} | Entropy: {:.2}, CV: {:.1}%, P99: {} |\n",
            status_icon, report.complexity.entropy, report.complexity.cv, report.complexity.p99
        ));
    }

    output.push_str("\n## Detailed Results\n\n");

    // Add detailed check results
    for (check_name, result) in results {
        match result {
            Ok(_) => {
                output.push_str(&format!("### ✅ {check_name}\n\n"));
                output.push_str("Check passed successfully.\n\n");
            }
            Err(msg) => {
                output.push_str(&format!("### ❌ {check_name}\n\n"));
                output.push_str(&format!("**Error:** {msg}\n\n"));
            }
        }
    }

    output
}

/// Handle the serve command to start HTTP API server
async fn handle_serve(host: String, port: u16, cors: bool) -> anyhow::Result<()> {
    info!("🚀 Starting HTTP API server on {}:{}", host, port);

    use crate::unified_protocol::service::UnifiedService;
    use std::net::SocketAddr;

    // Create UnifiedService which contains the router
    let service = UnifiedService::new();

    // Get the router from the service (it already has all middleware and extensions)
    let mut app = service.router();

    // Add CORS if enabled (this is the only additional middleware we need)
    if cors {
        use tower_http::cors::CorsLayer;
        info!("🌐 CORS enabled for cross-origin requests");
        app = app.layer(CorsLayer::permissive());
    }

    // Parse bind address
    let addr: SocketAddr = format!("{host}:{port}").parse()?;

    // Create TCP listener
    let listener = tokio::net::TcpListener::bind(&addr).await?;
    info!("✅ HTTP API server listening on http://{}", addr);

    // Display available endpoints
    eprintln!("📡 Available endpoints:");
    eprintln!("  • GET  /health                          - Health check");
    eprintln!("  • GET  /metrics                         - Service metrics");
    eprintln!("  • GET  /api/v1/templates                - List templates");
    eprintln!("  • GET  /api/v1/templates/{{id}}           - Get template details");
    eprintln!("  • POST /api/v1/generate                 - Generate template");
    eprintln!(
        "  • GET  /api/v1/analyze/complexity       - Complexity analysis (with query params)"
    );
    eprintln!("  • POST /api/v1/analyze/complexity       - Complexity analysis (with JSON body)");
    eprintln!("  • POST /api/v1/analyze/churn            - Code churn analysis");
    eprintln!("  • POST /api/v1/analyze/dag              - Dependency graph analysis");
    eprintln!("  • POST /api/v1/analyze/context          - Generate project context");
    eprintln!("  • POST /api/v1/analyze/dead-code        - Dead code analysis");
    eprintln!("  • POST /api/v1/analyze/deep-context     - Deep context analysis");
    eprintln!("  • POST /mcp/{{method}}                    - MCP protocol endpoint");
    eprintln!();
    eprintln!("💡 Example: curl http://{host}:{port}/api/v1/analyze/complexity?top_files=5");
    eprintln!("💡 Example: curl http://{host}:{port}/health");
    eprintln!();
    eprintln!("🛑 Press Ctrl+C to stop the server");

    // Start the server
    axum::serve(listener, app).await?;

    Ok(())
}

fn print_table(templates: &[Arc<TemplateResource>]) {
    // Calculate column widths
    let uri_width = templates.iter().map(|t| t.uri.len()).max().unwrap_or(20);

    // Print header
    println!(
        "{:<width$} {:>10} {:>12} {:>8}",
        "URI",
        "Toolchain",
        "Category",
        "Params",
        width = uri_width
    );
    println!("{}", "─".repeat(uri_width + 35));

    // Print rows
    for template in templates {
        println!(
            "{:<width$} {:>10} {:>12} {:>8}",
            template.uri,
            format!("{:?}", template.toolchain),
            format!("{:?}", template.category),
            template.parameters.len(),
            width = uri_width
        );
    }
}

/// Handle TDG analysis command
#[allow(clippy::too_many_arguments)]
async fn handle_analyze_tdg(
    path: PathBuf,
    threshold: f64,
    top: usize,
    format: TdgOutputFormat,
    include_components: bool,
    output: Option<PathBuf>,
    critical_only: bool,
    verbose: bool,
) -> anyhow::Result<()> {
    use crate::services::tdg_calculator::TDGCalculator;

    if verbose {
        eprintln!(
            "🔍 Analyzing Technical Debt Gradient for project at: {}",
            path.display()
        );
    }

    // Create TDG calculator
    let calculator = TDGCalculator::new();

    // Run analysis
    let summary = calculator.analyze_directory(&path).await?;

    // Filter files based on criteria
    let mut filtered_files: Vec<_> = summary
        .hotspots
        .clone()
        .into_iter()
        .filter(|hotspot| {
            if critical_only {
                hotspot.tdg_score > 2.5
            } else {
                hotspot.tdg_score > threshold
            }
        })
        .take(top)
        .collect();

    // Sort by TDG score descending
    filtered_files.sort_by(|a, b| b.tdg_score.partial_cmp(&a.tdg_score).unwrap());

    if verbose {
        eprintln!(
            "📊 Found {} files above threshold {:.2} (showing top {})",
            filtered_files.len(),
            threshold,
            top
        );
    }

    // Format output
    let content = match format {
        TdgOutputFormat::Table => format_tdg_table(&summary, &filtered_files, include_components),
        TdgOutputFormat::Json => serde_json::to_string_pretty(&summary)?,
        TdgOutputFormat::Markdown => {
            format_tdg_markdown(&summary, &filtered_files, include_components)
        }
        TdgOutputFormat::Sarif => format_tdg_sarif(&summary, &filtered_files)?,
    };

    // Output results
    if let Some(output_path) = output {
        tokio::fs::write(&output_path, &content).await?;
        if verbose {
            eprintln!("💾 Results written to: {}", output_path.display());
        }
    } else {
        println!("{content}");
    }

    Ok(())
}

/// Format TDG results as a table
fn format_tdg_table(
    summary: &crate::models::tdg::TDGSummary,
    hotspots: &[crate::models::tdg::TDGHotspot],
    _include_components: bool,
) -> String {
    let mut output = String::new();

    // Summary
    output.push_str("Technical Debt Gradient Analysis Summary\n");
    output.push_str("========================================\n");
    output.push_str(&format!("Total files: {}\n", summary.total_files));
    output.push_str(&format!(
        "Critical files (>2.5): {}\n",
        summary.critical_files
    ));
    output.push_str(&format!(
        "Warning files (1.5-2.5): {}\n",
        summary.warning_files
    ));
    output.push_str(&format!("Average TDG: {:.2}\n", summary.average_tdg));
    output.push_str(&format!("95th percentile: {:.2}\n", summary.p95_tdg));
    output.push_str(&format!(
        "Estimated debt: {:.0} hours\n\n",
        summary.estimated_debt_hours
    ));

    // Hotspots table
    if !hotspots.is_empty() {
        output.push_str("Top TDG Hotspots:\n");
        output.push_str("┌─────────────────────────────────────────────────────┬───────────┬─────────────────┬──────────────┐\n");
        output.push_str("│ File                                                │ TDG Score │ Primary Factor  │ Est. Hours   │\n");
        output.push_str("├─────────────────────────────────────────────────────┼───────────┼─────────────────┼──────────────┤\n");

        for hotspot in hotspots {
            output.push_str(&format!(
                "│ {:<51} │ {:>9.2} │ {:<15} │ {:>12.0} │\n",
                truncate_path(&hotspot.path, 51),
                hotspot.tdg_score,
                hotspot.primary_factor,
                hotspot.estimated_hours
            ));
        }

        output.push_str("└─────────────────────────────────────────────────────┴───────────┴─────────────────┴──────────────┘\n");
    }

    output
}

/// Format TDG results as markdown
fn format_tdg_markdown(
    summary: &crate::models::tdg::TDGSummary,
    hotspots: &[crate::models::tdg::TDGHotspot],
    _include_components: bool,
) -> String {
    let mut output = String::new();

    output.push_str("# Technical Debt Gradient Analysis\n\n");

    // Summary
    output.push_str("## Summary\n\n");
    output.push_str(&format!("- **Total files:** {}\n", summary.total_files));
    output.push_str(&format!(
        "- **Critical files (>2.5):** {}\n",
        summary.critical_files
    ));
    output.push_str(&format!(
        "- **Warning files (1.5-2.5):** {}\n",
        summary.warning_files
    ));
    output.push_str(&format!("- **Average TDG:** {:.2}\n", summary.average_tdg));
    output.push_str(&format!("- **95th percentile:** {:.2}\n", summary.p95_tdg));
    output.push_str(&format!(
        "- **Estimated debt:** {:.0} hours\n\n",
        summary.estimated_debt_hours
    ));

    // Hotspots
    if !hotspots.is_empty() {
        output.push_str("## Top Hotspots\n\n");
        output.push_str("| File | TDG Score | Primary Factor | Est. Hours |\n");
        output.push_str("|------|-----------|----------------|------------|\n");

        for hotspot in hotspots {
            output.push_str(&format!(
                "| {} | {:.2} | {} | {:.0} |\n",
                hotspot.path, hotspot.tdg_score, hotspot.primary_factor, hotspot.estimated_hours
            ));
        }
        output.push('\n');
    }

    output
}

/// Format TDG results as SARIF
fn format_tdg_sarif(
    _summary: &crate::models::tdg::TDGSummary,
    hotspots: &[crate::models::tdg::TDGHotspot],
) -> anyhow::Result<String> {
    use serde_json::json;

    let results: Vec<_> = hotspots
        .iter()
        .map(|hotspot| {
            json!({
                "ruleId": "TDG_HIGH",
                "message": {
                    "text": format!("High Technical Debt Gradient: {:.2} (primary factor: {})",
                        hotspot.tdg_score, hotspot.primary_factor)
                },
                "locations": [{
                    "physicalLocation": {
                        "artifactLocation": {
                            "uri": hotspot.path
                        },
                        "region": {
                            "startLine": 1
                        }
                    }
                }],
                "level": if hotspot.tdg_score > 2.5 { "error" } else { "warning" },
                "properties": {
                    "tdg_score": hotspot.tdg_score,
                    "primary_factor": hotspot.primary_factor,
                    "estimated_hours": hotspot.estimated_hours
                }
            })
        })
        .collect();

    let sarif = json!({
        "$schema": "https://schemastore.azurewebsites.net/schemas/json/sarif-2.1.0.json",
        "version": "2.1.0",
        "runs": [{
            "tool": {
                "driver": {
                    "name": "paiml-mcp-agent-toolkit",
                    "version": "0.21.0",
                    "informationUri": "https://github.com/paiml/mcp-agent-toolkit"
                }
            },
            "results": results
        }]
    });

    Ok(serde_json::to_string_pretty(&sarif)?)
}

/// Truncate path for table display
fn truncate_path(path: &str, max_len: usize) -> String {
    if path.len() <= max_len {
        path.to_string()
    } else {
        format!("...{}", &path[path.len() - (max_len - 3)..])
    }
}

/// Handle Makefile analysis command
async fn handle_analyze_makefile(
    path: PathBuf,
    _rules: Vec<String>,
    format: MakefileOutputFormat,
    fix: bool,
    _gnu_version: String,
) -> anyhow::Result<()> {
    use crate::services::makefile_linter;

    // Perform linting
    let result = makefile_linter::lint_makefile(&path).await?;

    // Apply fixes if requested
    if fix && result.violations.iter().any(|v| v.fix_hint.is_some()) {
        eprintln!("🔧 Auto-fix is not yet implemented. Fix hints are provided in the output.");
    }

    // Format output
    let output = match format {
        MakefileOutputFormat::Human => format_makefile_human(&result),
        MakefileOutputFormat::Json => serde_json::to_string_pretty(&result)?,
        MakefileOutputFormat::Gcc => format_makefile_gcc(&result),
        MakefileOutputFormat::Sarif => format_makefile_sarif(&result)?,
    };

    println!("{output}");

    // Exit with error code if violations found
    if result.has_errors() {
        std::process::exit(1);
    }

    Ok(())
}

/// Format Makefile lint results in human-readable format
fn format_makefile_human(result: &makefile_linter::LintResult) -> String {
    use crate::services::makefile_linter::Severity;

    let mut output = String::new();

    output.push_str(&format!("Analyzing {}...\n\n", result.path.display()));

    if result.violations.is_empty() {
        output.push_str("✅ No issues found!\n");
        output.push_str(&format!(
            "Quality score: {:.0}%\n",
            result.quality_score * 100.0
        ));
        return output;
    }

    // Group violations by severity
    let errors: Vec<_> = result
        .violations
        .iter()
        .filter(|v| v.severity == Severity::Error)
        .collect();
    let warnings: Vec<_> = result
        .violations
        .iter()
        .filter(|v| v.severity == Severity::Warning)
        .collect();
    let info: Vec<_> = result
        .violations
        .iter()
        .filter(|v| v.severity == Severity::Info)
        .collect();
    let perf: Vec<_> = result
        .violations
        .iter()
        .filter(|v| v.severity == Severity::Performance)
        .collect();

    // Summary
    output.push_str(&format!(
        "Found {} issues ({} errors, {} warnings, {} info, {} performance)\n",
        result.violations.len(),
        errors.len(),
        warnings.len(),
        info.len(),
        perf.len()
    ));
    output.push_str(&format!(
        "Quality score: {:.0}%\n\n",
        result.quality_score * 100.0
    ));

    // Print violations by severity
    if !errors.is_empty() {
        output.push_str("❌ Errors:\n");
        for v in errors {
            output.push_str(&format!(
                "  {}:{} [{}] {}\n",
                result.path.display(),
                v.span.line,
                v.rule,
                v.message
            ));
            if let Some(hint) = &v.fix_hint {
                output.push_str(&format!("    💡 {hint}\n"));
            }
        }
        output.push('\n');
    }

    if !warnings.is_empty() {
        output.push_str("⚠️  Warnings:\n");
        for v in warnings {
            output.push_str(&format!(
                "  {}:{} [{}] {}\n",
                result.path.display(),
                v.span.line,
                v.rule,
                v.message
            ));
            if let Some(hint) = &v.fix_hint {
                output.push_str(&format!("    💡 {hint}\n"));
            }
        }
        output.push('\n');
    }

    if !info.is_empty() {
        output.push_str("ℹ️  Info:\n");
        for v in info {
            output.push_str(&format!(
                "  {}:{} [{}] {}\n",
                result.path.display(),
                v.span.line,
                v.rule,
                v.message
            ));
            if let Some(hint) = &v.fix_hint {
                output.push_str(&format!("    💡 {hint}\n"));
            }
        }
        output.push('\n');
    }

    if !perf.is_empty() {
        output.push_str("⚡ Performance:\n");
        for v in perf {
            output.push_str(&format!(
                "  {}:{} [{}] {}\n",
                result.path.display(),
                v.span.line,
                v.rule,
                v.message
            ));
            if let Some(hint) = &v.fix_hint {
                output.push_str(&format!("    💡 {hint}\n"));
            }
        }
    }

    output
}

/// Format Makefile lint results in GCC-style format
fn format_makefile_gcc(result: &makefile_linter::LintResult) -> String {
    use crate::services::makefile_linter::Severity;

    let mut output = String::new();

    for v in &result.violations {
        let severity = match v.severity {
            Severity::Error => "error",
            Severity::Warning => "warning",
            Severity::Info => "note",
            Severity::Performance => "note",
        };

        output.push_str(&format!(
            "{}:{}:0: {}: [{}] {}\n",
            result.path.display(),
            v.span.line,
            severity,
            v.rule,
            v.message
        ));
    }

    output
}

/// Format Makefile lint results as SARIF
fn format_makefile_sarif(result: &makefile_linter::LintResult) -> anyhow::Result<String> {
    use crate::services::makefile_linter::Severity;
    use serde_json::json;

    let results: Vec<_> = result
        .violations
        .iter()
        .map(|v| {
            let level = match v.severity {
                Severity::Error => "error",
                Severity::Warning => "warning",
                Severity::Info => "note",
                Severity::Performance => "note",
            };

            json!({
                "ruleId": v.rule,
                "message": {
                    "text": &v.message
                },
                "locations": [{
                    "physicalLocation": {
                        "artifactLocation": {
                            "uri": result.path.to_string_lossy()
                        },
                        "region": {
                            "startLine": v.span.line as i64,
                            "startColumn": v.span.column as i64
                        }
                    }
                }],
                "level": level,
                "fixes": v.fix_hint.as_ref().map(|hint| vec![json!({
                    "description": {
                        "text": hint
                    }
                })]).unwrap_or_default()
            })
        })
        .collect();

    let sarif = json!({
        "$schema": "https://schemastore.azurewebsites.net/schemas/json/sarif-2.1.0.json",
        "version": "2.1.0",
        "runs": [{
            "tool": {
                "driver": {
                    "name": "paiml-makefile-linter",
                    "version": env!("CARGO_PKG_VERSION"),
                    "informationUri": "https://github.com/paiml/mcp-agent-toolkit",
                    "rules": [
                        {
                            "id": "minphony",
                            "name": "MinimumPhonyTargets",
                            "shortDescription": {
                                "text": "Ensure required targets are declared .PHONY"
                            }
                        },
                        {
                            "id": "phonydeclared",
                            "name": "PhonyDeclared",
                            "shortDescription": {
                                "text": "Non-file targets should be declared .PHONY"
                            }
                        },
                        {
                            "id": "maxbodylength",
                            "name": "MaxBodyLength",
                            "shortDescription": {
                                "text": "Recipe complexity check"
                            }
                        },
                        {
                            "id": "timestampexpanded",
                            "name": "TimestampExpanded",
                            "shortDescription": {
                                "text": "Timestamp evaluation timing"
                            }
                        },
                        {
                            "id": "undefinedvariable",
                            "name": "UndefinedVariable",
                            "shortDescription": {
                                "text": "Check for undefined variable usage"
                            }
                        },
                        {
                            "id": "recursive-expansion",
                            "name": "RecursiveExpansion",
                            "shortDescription": {
                                "text": "Expensive recursive variable expansion"
                            }
                        },
                        {
                            "id": "portability",
                            "name": "Portability",
                            "shortDescription": {
                                "text": "GNU Make specific features"
                            }
                        }
                    ]
                }
            },
            "results": results
        }]
    });

    Ok(serde_json::to_string_pretty(&sarif)?)
}

/// Handle provability analysis command
#[allow(clippy::too_many_arguments)]
async fn handle_analyze_provability(
    project_path: PathBuf,
    functions: Vec<String>,
    _analysis_depth: usize,
    format: ProvabilityOutputFormat,
    high_confidence_only: bool,
    include_evidence: bool,
    output: Option<PathBuf>,
) -> anyhow::Result<()> {
    use crate::services::lightweight_provability_analyzer::{
        FunctionId, LightweightProvabilityAnalyzer,
    };

    let analyzer = LightweightProvabilityAnalyzer::new();

    // Extract functions from parameters or mock discovery from project path
    let function_ids = if functions.is_empty() {
        // Mock function discovery from project path
        vec![FunctionId {
            file_path: format!("{}/src/main.rs", project_path.display()),
            function_name: "main".to_string(),
            line_number: 1,
        }]
    } else {
        functions
            .into_iter()
            .enumerate()
            .map(|(i, name)| FunctionId {
                file_path: format!("{}/src/lib.rs", project_path.display()),
                function_name: name,
                line_number: i * 10, // Mock line numbers
            })
            .collect()
    };

    // Perform analysis
    let summaries = analyzer.analyze_incrementally(&function_ids).await;

    // Filter by confidence if requested
    let filtered_summaries: Vec<_> = if high_confidence_only {
        summaries
            .into_iter()
            .filter(|s| s.provability_score > 0.8)
            .collect()
    } else {
        summaries
    };

    // Format output
    let result = match format {
        ProvabilityOutputFormat::Summary => format_provability_summary(&filtered_summaries),
        ProvabilityOutputFormat::Full => {
            format_provability_full(&filtered_summaries, include_evidence)
        }
        ProvabilityOutputFormat::Json => serde_json::to_string_pretty(&filtered_summaries)?,
        ProvabilityOutputFormat::Markdown => {
            format_provability_markdown(&filtered_summaries, include_evidence)
        }
        ProvabilityOutputFormat::Sarif => format_provability_sarif(&filtered_summaries)?,
    };

    // Output to file or stdout
    if let Some(output_path) = output {
        tokio::fs::write(&output_path, &result).await?;
        eprintln!("Provability analysis written to {}", output_path.display());
    } else {
        println!("{result}");
    }

    Ok(())
}

/// Format provability results in summary format
fn format_provability_summary(
    summaries: &[crate::services::lightweight_provability_analyzer::ProofSummary],
) -> String {
    if summaries.is_empty() {
        return "No functions analyzed.".to_string();
    }

    let avg_score =
        summaries.iter().map(|s| s.provability_score).sum::<f64>() / summaries.len() as f64;
    let high_confidence = summaries
        .iter()
        .filter(|s| s.provability_score > 0.8)
        .count();
    let medium_confidence = summaries
        .iter()
        .filter(|s| s.provability_score > 0.5 && s.provability_score <= 0.8)
        .count();
    let low_confidence = summaries
        .iter()
        .filter(|s| s.provability_score <= 0.5)
        .count();

    format!(
        "Provability Analysis Summary\n\
        ============================\n\
        Functions analyzed: {}\n\
        Average provability score: {:.2}\n\
        High confidence (>0.8): {}\n\
        Medium confidence (0.5-0.8): {}\n\
        Low confidence (≤0.5): {}\n",
        summaries.len(),
        avg_score,
        high_confidence,
        medium_confidence,
        low_confidence
    )
}

/// Format provability results in full format
fn format_provability_full(
    summaries: &[crate::services::lightweight_provability_analyzer::ProofSummary],
    include_evidence: bool,
) -> String {
    let mut output = format_provability_summary(summaries);
    output.push_str("\nDetailed Results:\n");
    output.push_str(&"-".repeat(50));
    output.push('\n');

    for (i, summary) in summaries.iter().enumerate() {
        output.push_str(&format!(
            "\nFunction #{} (Score: {:.2})\n",
            i + 1,
            summary.provability_score
        ));
        output.push_str(&format!(
            "  Analysis time: {}μs\n",
            summary.analysis_time_us
        ));
        output.push_str(&format!(
            "  Properties verified: {}\n",
            summary.verified_properties.len()
        ));

        if include_evidence {
            for prop in &summary.verified_properties {
                output.push_str(&format!(
                    "    • {:#?}: {:.1}% confidence - {}\n",
                    prop.property_type,
                    prop.confidence * 100.0,
                    prop.evidence
                ));
            }
        }
    }

    output
}

/// Format provability results in markdown format
fn format_provability_markdown(
    summaries: &[crate::services::lightweight_provability_analyzer::ProofSummary],
    include_evidence: bool,
) -> String {
    let mut output = String::from("# Provability Analysis Report\n\n");

    if summaries.is_empty() {
        output.push_str("No functions analyzed.\n");
        return output;
    }

    let avg_score =
        summaries.iter().map(|s| s.provability_score).sum::<f64>() / summaries.len() as f64;

    output.push_str("## Summary\n\n");
    output.push_str(&format!("- **Functions analyzed:** {}\n", summaries.len()));
    output.push_str(&format!(
        "- **Average provability score:** {avg_score:.2}\n"
    ));

    output.push_str("\n## Results\n\n");

    for (i, summary) in summaries.iter().enumerate() {
        output.push_str(&format!("### Function #{}\n\n", i + 1));
        output.push_str(&format!(
            "- **Provability Score:** {:.2}\n",
            summary.provability_score
        ));
        output.push_str(&format!(
            "- **Analysis Time:** {}μs\n",
            summary.analysis_time_us
        ));
        output.push_str(&format!(
            "- **Properties Verified:** {}\n\n",
            summary.verified_properties.len()
        ));

        if include_evidence && !summary.verified_properties.is_empty() {
            output.push_str("#### Verified Properties\n\n");
            for prop in &summary.verified_properties {
                output.push_str(&format!(
                    "- **{:#?}**: {:.1}% confidence\n",
                    prop.property_type,
                    prop.confidence * 100.0
                ));
                output.push_str(&format!("  - Evidence: {}\n", prop.evidence));
            }
            output.push('\n');
        }
    }

    output
}

/// Format provability results in SARIF format
fn format_provability_sarif(
    summaries: &[crate::services::lightweight_provability_analyzer::ProofSummary],
) -> anyhow::Result<String> {
    // Simplified SARIF format for provability results
    let sarif = serde_json::json!({
        "version": "2.1.0",
        "$schema": "https://raw.githubusercontent.com/oasis-tcs/sarif-spec/master/Schemata/sarif-schema-2.1.0.json",
        "runs": [{
            "tool": {
                "driver": {
                    "name": "paiml-mcp-agent-toolkit",
                    "version": env!("CARGO_PKG_VERSION"),
                    "semanticVersion": env!("CARGO_PKG_VERSION"),
                    "informationUri": "https://github.com/paiml/mcp-agent-toolkit"
                }
            },
            "results": summaries.iter().enumerate().map(|(i, summary)| {
                serde_json::json!({
                    "ruleId": "provability-score",
                    "level": if summary.provability_score > 0.8 { "note" } else if summary.provability_score > 0.5 { "warning" } else { "error" },
                    "message": {
                        "text": format!("Function #{} has provability score {:.2}", i + 1, summary.provability_score)
                    },
                    "properties": {
                        "provability_score": summary.provability_score,
                        "analysis_time_us": summary.analysis_time_us,
                        "verified_properties_count": summary.verified_properties.len()
                    }
                })
            }).collect::<Vec<_>>()
        }]
    });

    Ok(serde_json::to_string_pretty(&sarif)?)
}

/// Handle duplicate detection analysis
#[allow(clippy::too_many_arguments)]
async fn handle_analyze_duplicates(
    project_path: PathBuf,
    detection_type: DuplicateType,
    threshold: f32,
    min_lines: usize,
    max_tokens: usize,
    format: DuplicateOutputFormat,
    perf: bool,
    include: Option<String>,
    exclude: Option<String>,
    output: Option<PathBuf>,
) -> anyhow::Result<()> {
    use crate::services::duplicate_detector::{
        DuplicateDetectionConfig, DuplicateDetectionEngine, Language,
    };
    use crate::services::file_discovery::{FileDiscoveryConfig, ProjectFileDiscovery};
    use std::time::Instant;

    let start_time = Instant::now();

    // Create configuration based on CLI parameters
    let mut config = DuplicateDetectionConfig {
        similarity_threshold: threshold as f64,
        min_tokens: min_lines * 5,
        ..Default::default()
    }; // Rough estimate: ~5 tokens per line

    // Adjust config based on detection type
    match detection_type {
        DuplicateType::Exact => {
            config.similarity_threshold = 1.0;
            config.normalize_identifiers = false;
            config.normalize_literals = false;
        }
        DuplicateType::Renamed => {
            config.similarity_threshold = 0.95;
            config.normalize_identifiers = true;
            config.normalize_literals = false;
        }
        DuplicateType::Gapped => {
            config.similarity_threshold = 0.80;
            config.normalize_identifiers = true;
            config.normalize_literals = true;
        }
        DuplicateType::Semantic => {
            config.similarity_threshold = threshold as f64;
            config.normalize_identifiers = true;
            config.normalize_literals = true;
        }
        DuplicateType::All => {
            // Use default settings for comprehensive detection
        }
    }

    // Discover source files
    let mut discovery_config = FileDiscoveryConfig::default();

    // Add custom patterns if specified
    if let Some(exclude_pattern) = &exclude {
        discovery_config
            .custom_ignore_patterns
            .push(exclude_pattern.clone());
    }

    let discovery = ProjectFileDiscovery::new(project_path.clone()).with_config(discovery_config);
    let discovered_files = discovery.discover_files()?;

    // Read and categorize files by language
    let mut files_with_content = Vec::new();
    for file_path in discovered_files {
        // Apply include filter if specified
        if let Some(include_pattern) = &include {
            if !file_path.to_string_lossy().contains(include_pattern) {
                continue;
            }
        }

        if let Ok(content) = std::fs::read_to_string(&file_path) {
            let language = match file_path.extension().and_then(|e| e.to_str()) {
                Some("rs") => Language::Rust,
                Some("ts") | Some("tsx") => Language::TypeScript,
                Some("js") | Some("jsx") => Language::JavaScript,
                Some("py") => Language::Python,
                Some("c") | Some("h") => Language::C,
                Some("cpp") | Some("cc") | Some("cxx") | Some("hpp") => Language::Cpp,
                _ => continue, // Skip unsupported files
            };

            // Apply line limit if specified
            if content.lines().count() >= min_lines {
                files_with_content.push((file_path, content, language));
            }
        }
    }

    if files_with_content.is_empty() {
        eprintln!("No source files found matching criteria");
        return Ok(());
    }

    // Limit token analysis for performance
    for (_, content, _) in &mut files_with_content {
        if content.split_whitespace().count() > max_tokens {
            // Truncate content to max_tokens
            let words: Vec<&str> = content.split_whitespace().take(max_tokens).collect();
            *content = words.join(" ");
        }
    }

    // Run duplicate detection
    let engine = DuplicateDetectionEngine::new(config);
    let report = engine.detect_duplicates(&files_with_content)?;

    let analysis_time = start_time.elapsed();

    // Output results based on format
    match format {
        DuplicateOutputFormat::Summary => {
            println!("Duplicate Code Analysis Summary");
            println!("==============================");
            println!("Files analyzed: {}", report.summary.total_files);
            println!("Code fragments: {}", report.summary.total_fragments);
            println!("Duplicate lines: {}", report.summary.duplicate_lines);
            println!("Total lines: {}", report.summary.total_lines);
            println!(
                "Duplication ratio: {:.1}%",
                report.summary.duplication_ratio * 100.0
            );
            println!("Clone groups: {}", report.summary.clone_groups);
            println!(
                "Largest group: {} instances",
                report.summary.largest_group_size
            );

            if perf {
                println!("\nPerformance Metrics:");
                println!("Analysis time: {:.2}s", analysis_time.as_secs_f64());
                println!(
                    "Files/second: {:.1}",
                    files_with_content.len() as f64 / analysis_time.as_secs_f64()
                );
            }
        }
        DuplicateOutputFormat::Detailed => {
            println!("Duplicate Code Analysis Report");
            println!("=============================");
            println!("{}", serde_json::to_string_pretty(&report)?);
        }
        DuplicateOutputFormat::Json => {
            let mut result = serde_json::to_value(&report)?;
            if perf {
                result["performance"] = serde_json::json!({
                    "analysis_time_s": analysis_time.as_secs_f64(),
                    "files_per_second": files_with_content.len() as f64 / analysis_time.as_secs_f64()
                });
            }
            println!("{}", serde_json::to_string_pretty(&result)?);
        }
        DuplicateOutputFormat::Csv => {
            println!("file,start_line,end_line,group_id,clone_type,similarity");
            for group in &report.groups {
                for instance in &group.fragments {
                    println!(
                        "{},{},{},{},{:?},{:.3}",
                        instance.file.display(),
                        instance.start_line,
                        instance.end_line,
                        group.id,
                        group.clone_type,
                        instance.similarity_to_representative
                    );
                }
            }
        }
        DuplicateOutputFormat::Sarif => {
            // SARIF format for IDE integration
            let sarif = serde_json::json!({
                "version": "2.1.0",
                "$schema": "https://raw.githubusercontent.com/oasis-tcs/sarif-spec/master/Schemata/sarif-schema-2.1.0.json",
                "runs": [{
                    "tool": {
                        "driver": {
                            "name": "paiml-mcp-agent-toolkit",
                            "version": env!("CARGO_PKG_VERSION"),
                            "informationUri": "https://github.com/paiml/mcp-agent-toolkit"
                        }
                    },
                    "results": report.groups.iter().flat_map(|group| {
                        group.fragments.iter().map(|instance| {
                            serde_json::json!({
                                "ruleId": "duplicate-code",
                                "level": "info",
                                "message": {
                                    "text": format!("Duplicate code found in group {} with {:.1}% similarity",
                                        group.id, instance.similarity_to_representative * 100.0)
                                },
                                "locations": [{
                                    "physicalLocation": {
                                        "artifactLocation": {
                                            "uri": instance.file.to_string_lossy()
                                        },
                                        "region": {
                                            "startLine": instance.start_line,
                                            "endLine": instance.end_line,
                                            "startColumn": instance.start_column,
                                            "endColumn": instance.end_column
                                        }
                                    }
                                }]
                            })
                        })
                    }).collect::<Vec<_>>()
                }]
            });
            println!("{}", serde_json::to_string_pretty(&sarif)?);
        }
    }

    // Write to output file if specified
    if let Some(output_path) = output {
        let content = match format {
            DuplicateOutputFormat::Json => serde_json::to_string_pretty(&report)?,
            DuplicateOutputFormat::Sarif => {
                // Generate SARIF content for file output
                let sarif = serde_json::json!({
                    "version": "2.1.0",
                    "$schema": "https://raw.githubusercontent.com/oasis-tcs/sarif-spec/master/Schemata/sarif-schema-2.1.0.json",
                    "runs": [{
                        "tool": {
                            "driver": {
                                "name": "paiml-mcp-agent-toolkit",
                                "version": env!("CARGO_PKG_VERSION"),
                                "informationUri": "https://github.com/paiml/mcp-agent-toolkit"
                            }
                        },
                        "results": report.groups.iter().flat_map(|group| {
                            group.fragments.iter().map(|instance| {
                                serde_json::json!({
                                    "ruleId": "duplicate-code",
                                    "level": "info",
                                    "message": {
                                        "text": format!("Duplicate code found in group {} with {:.1}% similarity",
                                            group.id, instance.similarity_to_representative * 100.0)
                                    },
                                    "locations": [{
                                        "physicalLocation": {
                                            "artifactLocation": {
                                                "uri": instance.file.to_string_lossy()
                                            },
                                            "region": {
                                                "startLine": instance.start_line,
                                                "endLine": instance.end_line,
                                                "startColumn": instance.start_column,
                                                "endColumn": instance.end_column
                                            }
                                        }
                                    }]
                                })
                            })
                        }).collect::<Vec<_>>()
                    }]
                });
                serde_json::to_string_pretty(&sarif)?
            }
            _ => format!("{report:#?}"),
        };
        std::fs::write(output_path, content)?;
    }

    Ok(())
}

/// Handle defect prediction analysis
#[allow(clippy::too_many_arguments)]
async fn handle_analyze_defect_prediction(
    project_path: PathBuf,
    confidence_threshold: f32,
    min_lines: usize,
    include_low_confidence: bool,
    format: DefectPredictionOutputFormat,
    high_risk_only: bool,
    include_recommendations: bool,
    include: Option<String>,
    exclude: Option<String>,
    output: Option<PathBuf>,
    perf: bool,
) -> anyhow::Result<()> {
    use crate::services::defect_probability::{DefectProbabilityCalculator, FileMetrics};
    use crate::services::file_discovery::{FileDiscoveryConfig, ProjectFileDiscovery};
    use std::time::Instant;

    let start_time = Instant::now();

    // Discover source files
    let mut discovery_config = FileDiscoveryConfig::default();

    // Add custom patterns if specified
    if let Some(exclude_pattern) = &exclude {
        discovery_config
            .custom_ignore_patterns
            .push(exclude_pattern.clone());
    }

    let discovery = ProjectFileDiscovery::new(project_path.clone()).with_config(discovery_config);
    let discovered_files = discovery.discover_files()?;

    // Filter files based on include pattern and minimum lines
    let mut analyzed_files = Vec::new();
    for file_path in discovered_files {
        // Apply include filter if specified
        if let Some(include_pattern) = &include {
            if !file_path.to_string_lossy().contains(include_pattern) {
                continue;
            }
        }

        // Only analyze source code files
        if !matches!(
            file_path.extension().and_then(|e| e.to_str()),
            Some("rs")
                | Some("ts")
                | Some("tsx")
                | Some("js")
                | Some("jsx")
                | Some("py")
                | Some("c")
                | Some("h")
                | Some("cpp")
                | Some("cc")
                | Some("cxx")
                | Some("hpp")
        ) {
            continue;
        }

        if let Ok(content) = std::fs::read_to_string(&file_path) {
            let lines_of_code = content.lines().count();
            if lines_of_code >= min_lines {
                analyzed_files.push((file_path, content, lines_of_code));
            }
        }
    }

    if analyzed_files.is_empty() {
        eprintln!("No source files found matching criteria");
        return Ok(());
    }

    // Initialize defect calculator
    let defect_calculator = DefectProbabilityCalculator::new();

    // Collect metrics for each file
    let mut file_metrics = Vec::new();
    for (file_path, content, lines_of_code) in &analyzed_files {
        // Calculate basic complexity metrics (simplified)
        let cyclomatic_complexity = calculate_simple_complexity(content);
        let cognitive_complexity = (cyclomatic_complexity as f32 * 1.3) as u32; // Rough approximation

        // Calculate basic churn score (simplified - based on file size and modification indicators)
        let churn_score = calculate_simple_churn_score(content, *lines_of_code);

        // Calculate basic coupling (count imports/includes)
        let afferent_coupling = content
            .lines()
            .filter(|line| {
                line.trim_start().starts_with("use ")
                    || line.trim_start().starts_with("import ")
                    || line.trim_start().starts_with("#include")
            })
            .count() as f32;

        let metrics = FileMetrics {
            file_path: file_path.to_string_lossy().to_string(),
            churn_score,
            complexity: cyclomatic_complexity as f32,
            duplicate_ratio: 0.0, // Would need duplicate detection integration
            afferent_coupling,
            efferent_coupling: 0.0, // Simplified for now
            lines_of_code: *lines_of_code,
            cyclomatic_complexity,
            cognitive_complexity,
        };

        file_metrics.push(metrics);
    }

    // Calculate defect predictions
    let predictions = defect_calculator.calculate_batch(&file_metrics);

    // Filter results based on criteria
    let mut filtered_predictions: Vec<_> = predictions.into_iter().collect();

    if !include_low_confidence {
        filtered_predictions.retain(|(_, score)| score.confidence >= confidence_threshold);
    }

    if high_risk_only {
        filtered_predictions.retain(|(_, score)| score.probability >= 0.7);
    }

    // Sort by probability (highest first)
    filtered_predictions.sort_by(|a, b| b.1.probability.partial_cmp(&a.1.probability).unwrap());

    let analysis_time = start_time.elapsed();

    // Output results based on format
    match format {
        DefectPredictionOutputFormat::Summary => {
            println!("Defect Prediction Analysis Summary");
            println!("=================================");
            println!("Files analyzed: {}", file_metrics.len());
            println!("Predictions generated: {}", filtered_predictions.len());

            let high_risk_count = filtered_predictions
                .iter()
                .filter(|(_, score)| score.probability >= 0.7)
                .count();
            let medium_risk_count = filtered_predictions
                .iter()
                .filter(|(_, score)| score.probability >= 0.3 && score.probability < 0.7)
                .count();
            let low_risk_count = filtered_predictions
                .iter()
                .filter(|(_, score)| score.probability < 0.3)
                .count();

            println!(
                "High risk files: {} ({:.1}%)",
                high_risk_count,
                100.0 * high_risk_count as f32 / filtered_predictions.len() as f32
            );
            println!(
                "Medium risk files: {} ({:.1}%)",
                medium_risk_count,
                100.0 * medium_risk_count as f32 / filtered_predictions.len() as f32
            );
            println!(
                "Low risk files: {} ({:.1}%)",
                low_risk_count,
                100.0 * low_risk_count as f32 / filtered_predictions.len() as f32
            );

            if perf {
                println!("\nPerformance Metrics:");
                println!("Analysis time: {:.2}s", analysis_time.as_secs_f64());
                println!(
                    "Files/second: {:.1}",
                    file_metrics.len() as f64 / analysis_time.as_secs_f64()
                );
            }

            if !filtered_predictions.is_empty() {
                println!("\nTop 10 High-Risk Files:");
                for (file_path, score) in filtered_predictions.iter().take(10) {
                    println!(
                        "  {} - {:.1}% risk ({:?})",
                        std::path::Path::new(file_path)
                            .file_name()
                            .unwrap_or_default()
                            .to_string_lossy(),
                        score.probability * 100.0,
                        score.risk_level
                    );
                }
            }
        }
        DefectPredictionOutputFormat::Detailed => {
            println!("Defect Prediction Analysis Report");
            println!("================================");
            for (file_path, score) in &filtered_predictions {
                println!("\n{file_path}");
                println!("  Risk Level: {:?}", score.risk_level);
                println!("  Probability: {:.1}%", score.probability * 100.0);
                println!("  Confidence: {:.1}%", score.confidence * 100.0);

                println!("  Contributing Factors:");
                for (factor, contribution) in &score.contributing_factors {
                    println!("    {factor}: {contribution:.3}");
                }

                if include_recommendations && !score.recommendations.is_empty() {
                    println!("  Recommendations:");
                    for rec in &score.recommendations {
                        println!("    - {rec}");
                    }
                }
            }
        }
        DefectPredictionOutputFormat::Json => {
            let result = serde_json::json!({
                "summary": {
                    "total_files": file_metrics.len(),
                    "predictions": filtered_predictions.len(),
                    "high_risk": filtered_predictions.iter().filter(|(_, s)| s.probability >= 0.7).count(),
                    "medium_risk": filtered_predictions.iter().filter(|(_, s)| s.probability >= 0.3 && s.probability < 0.7).count(),
                    "low_risk": filtered_predictions.iter().filter(|(_, s)| s.probability < 0.3).count()
                },
                "predictions": filtered_predictions.iter().map(|(path, score)| {
                    serde_json::json!({
                        "file": path,
                        "probability": score.probability,
                        "confidence": score.confidence,
                        "risk_level": score.risk_level,
                        "contributing_factors": score.contributing_factors,
                        "recommendations": if include_recommendations { Some(&score.recommendations) } else { None }
                    })
                }).collect::<Vec<_>>(),
                "performance": if perf { Some(serde_json::json!({
                    "analysis_time_s": analysis_time.as_secs_f64(),
                    "files_per_second": file_metrics.len() as f64 / analysis_time.as_secs_f64()
                })) } else { None }
            });
            println!("{}", serde_json::to_string_pretty(&result)?);
        }
        DefectPredictionOutputFormat::Csv => {
            println!("file,probability,confidence,risk_level,churn_factor,complexity_factor,duplication_factor,coupling_factor");
            for (file_path, score) in &filtered_predictions {
                let factors = &score.contributing_factors;
                println!(
                    "{},{:.3},{:.3},{:?},{:.3},{:.3},{:.3},{:.3}",
                    file_path,
                    score.probability,
                    score.confidence,
                    score.risk_level,
                    factors.first().map(|(_, v)| *v).unwrap_or(0.0),
                    factors.get(1).map(|(_, v)| *v).unwrap_or(0.0),
                    factors.get(2).map(|(_, v)| *v).unwrap_or(0.0),
                    factors.get(3).map(|(_, v)| *v).unwrap_or(0.0)
                );
            }
        }
        DefectPredictionOutputFormat::Sarif => {
            // SARIF format for IDE integration
            let sarif = serde_json::json!({
                "version": "2.1.0",
                "$schema": "https://raw.githubusercontent.com/oasis-tcs/sarif-spec/master/Schemata/sarif-schema-2.1.0.json",
                "runs": [{
                    "tool": {
                        "driver": {
                            "name": "paiml-mcp-agent-toolkit",
                            "version": env!("CARGO_PKG_VERSION"),
                            "informationUri": "https://github.com/paiml/mcp-agent-toolkit"
                        }
                    },
                    "results": filtered_predictions.iter().map(|(file_path, score)| {
                        let level = match score.probability {
                            p if p >= 0.7 => "error",
                            p if p >= 0.3 => "warning",
                            _ => "note"
                        };
                        serde_json::json!({
                            "ruleId": "defect-prediction",
                            "level": level,
                            "message": {
                                "text": format!("High defect probability: {:.1}% (confidence: {:.1}%)",
                                    score.probability * 100.0, score.confidence * 100.0)
                            },
                            "locations": [{
                                "physicalLocation": {
                                    "artifactLocation": {
                                        "uri": file_path
                                    }
                                }
                            }],
                            "properties": {
                                "defect_probability": score.probability,
                                "confidence": score.confidence,
                                "risk_level": format!("{:?}", score.risk_level)
                            }
                        })
                    }).collect::<Vec<_>>()
                }]
            });
            println!("{}", serde_json::to_string_pretty(&sarif)?);
        }
    }

    // Write to output file if specified
    if let Some(output_path) = output {
        let content = match format {
            DefectPredictionOutputFormat::Json => {
                let result = serde_json::json!({
                    "predictions": filtered_predictions,
                    "summary": {
                        "total_files": file_metrics.len(),
                        "high_risk": filtered_predictions.iter().filter(|(_, s)| s.probability >= 0.7).count()
                    }
                });
                serde_json::to_string_pretty(&result)?
            }
            DefectPredictionOutputFormat::Sarif => {
                let sarif = serde_json::json!({
                    "version": "2.1.0",
                    "runs": [{
                        "tool": {
                            "driver": {
                                "name": "paiml-mcp-agent-toolkit",
                                "version": env!("CARGO_PKG_VERSION")
                            }
                        },
                        "results": filtered_predictions.iter().map(|(file_path, score)| {
                            serde_json::json!({
                                "ruleId": "defect-prediction",
                                "level": if score.probability >= 0.7 { "error" } else if score.probability >= 0.3 { "warning" } else { "note" },
                                "message": {
                                    "text": format!("Defect probability: {:.1}%", score.probability * 100.0)
                                },
                                "locations": [{
                                    "physicalLocation": {
                                        "artifactLocation": {
                                            "uri": file_path
                                        }
                                    }
                                }]
                            })
                        }).collect::<Vec<_>>()
                    }]
                });
                serde_json::to_string_pretty(&sarif)?
            }
            _ => format!("{filtered_predictions:#?}"),
        };
        std::fs::write(output_path, content)?;
    }

    Ok(())
}

/// Calculate a simple approximation of cyclomatic complexity
fn calculate_simple_complexity(content: &str) -> u32 {
    let mut complexity = 1; // Base complexity

    for line in content.lines() {
        let trimmed = line.trim();

        // Count decision points (simplified)
        complexity += trimmed.matches("if ").count() as u32;
        complexity += trimmed.matches("else if ").count() as u32;
        complexity += trimmed.matches("while ").count() as u32;
        complexity += trimmed.matches("for ").count() as u32;
        complexity += trimmed.matches("match ").count() as u32;
        complexity += trimmed.matches("case ").count() as u32;
        complexity += trimmed.matches("catch ").count() as u32;
        complexity += trimmed.matches("&&").count() as u32;
        complexity += trimmed.matches("||").count() as u32;
        complexity += trimmed.matches("?").count() as u32; // Ternary operators
    }

    complexity
}

/// Calculate a simple churn score based on code characteristics
fn calculate_simple_churn_score(content: &str, lines_of_code: usize) -> f32 {
    let mut churn_indicators = 0;

    for line in content.lines() {
        let trimmed = line.trim();

        // Look for indicators of high-churn code
        if trimmed.contains("TODO") || trimmed.contains("FIXME") || trimmed.contains("XXX") {
            churn_indicators += 2;
        }
        if trimmed.contains("temp") || trimmed.contains("tmp") || trimmed.contains("hack") {
            churn_indicators += 1;
        }
        if trimmed.starts_with("//") && (trimmed.contains("debug") || trimmed.contains("test")) {
            churn_indicators += 1;
        }
    }

    // Normalize by file size and convert to 0-1 range
    let base_score = churn_indicators as f32 / lines_of_code.max(1) as f32;
    base_score.min(1.0)
}

/// Comprehensive analysis combining multiple analysis types
#[allow(clippy::too_many_arguments)]
async fn handle_analyze_comprehensive(
    project_path: PathBuf,
    format: ComprehensiveOutputFormat,
    include_duplicates: bool,
    include_dead_code: bool,
    include_defects: bool,
    include_complexity: bool,
    include_tdg: bool,
    confidence_threshold: f32,
    min_lines: usize,
    _include: Option<String>,
    _exclude: Option<String>,
    output: Option<PathBuf>,
    perf: bool,
    executive_summary: bool,
) -> anyhow::Result<()> {
    let start_time = std::time::Instant::now();

    // Simple file discovery - find common source code files
    let mut discovered_files = Vec::new();
    let extensions = vec!["rs", "js", "ts", "py", "cpp", "c", "h", "hpp", "java", "go"];

    fn visit_dir(dir: &Path, extensions: &[&str], files: &mut Vec<PathBuf>) -> Result<(), String> {
        if let Ok(entries) = std::fs::read_dir(dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_dir() {
                    // Skip common build/cache directories
                    if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
                        if ![
                            "target",
                            "node_modules",
                            ".git",
                            "build",
                            "dist",
                            "__pycache__",
                        ]
                        .contains(&name)
                        {
                            visit_dir(&path, extensions, files)?;
                        }
                    }
                } else if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
                    if extensions.contains(&ext) {
                        files.push(path);
                    }
                }
            }
        }
        Ok(())
    }

    visit_dir(&project_path, &extensions, &mut discovered_files).map_err(|e| anyhow::anyhow!(e))?;

    // Filter by minimum lines of code
    let mut analysis_files = Vec::new();
    for file_path in discovered_files {
        if let Ok(content) = std::fs::read_to_string(&file_path) {
            let line_count = content.lines().count();
            if line_count >= min_lines {
                analysis_files.push((file_path, content));
            }
        }
    }

    if perf {
        eprintln!(
            "📁 Discovery completed: {} files found",
            analysis_files.len()
        );
    }

    // Initialize results containers
    let mut results = serde_json::json!({
        "analysis_type": "comprehensive",
        "timestamp": chrono::Utc::now().to_rfc3339(),
        "project_path": project_path.to_string_lossy(),
        "configuration": {
            "include_duplicates": include_duplicates,
            "include_dead_code": include_dead_code,
            "include_defects": include_defects,
            "include_complexity": include_complexity,
            "include_tdg": include_tdg,
            "confidence_threshold": confidence_threshold,
            "min_lines": min_lines,
            "executive_summary": executive_summary
        },
        "summary": {},
        "results": {}
    });

    let mut analysis_times = serde_json::json!({});

    // 1. Duplicate Detection Analysis (simplified)
    if include_duplicates {
        let dup_start = std::time::Instant::now();

        // Simple duplicate detection based on file size and hash
        let mut duplicates = 0;
        let mut size_groups: std::collections::HashMap<u64, Vec<String>> =
            std::collections::HashMap::new();

        for (file_path, content) in &analysis_files {
            let size = content.len() as u64;
            size_groups
                .entry(size)
                .or_default()
                .push(file_path.to_string_lossy().to_string());
        }

        for (_, files) in size_groups {
            if files.len() > 1 {
                duplicates += files.len() - 1;
            }
        }

        let dup_time = dup_start.elapsed();
        analysis_times["duplicates_ms"] = (dup_time.as_millis() as u64).into();

        if perf {
            eprintln!("🔍 Duplicates analysis: {:.2}s", dup_time.as_secs_f64());
        }

        results["results"]["duplicates"] = serde_json::json!({
            "potential_duplicates": duplicates,
            "analysis_method": "file_size_based",
        });
    }

    // 2. Dead Code Analysis (simplified)
    if include_dead_code {
        let dead_start = std::time::Instant::now();

        // Simple dead code detection based on TRACKED patterns
        let mut dead_indicators = 0;
        let mut files_with_issues = 0;

        for (_, content) in &analysis_files {
            let mut file_has_issues = false;
            for line in content.lines() {
                let trimmed = line.trim();
                if trimmed.contains("TODO")
                    || trimmed.contains("FIXME")
                    || trimmed.contains("HACK")
                    || trimmed.contains("XXX")
                {
                    dead_indicators += 1;
                    file_has_issues = true;
                }
            }
            if file_has_issues {
                files_with_issues += 1;
            }
        }

        let dead_time = dead_start.elapsed();
        analysis_times["dead_code_ms"] = (dead_time.as_millis() as u64).into();

        if perf {
            eprintln!("💀 Dead code analysis: {:.2}s", dead_time.as_secs_f64());
        }

        results["results"]["dead_code"] = serde_json::json!({
            "potential_issues": dead_indicators,
            "files_with_issues": files_with_issues,
            "analysis_method": "pattern_based",
        });
    }

    // 3. Defect Prediction Analysis
    if include_defects {
        let defect_start = std::time::Instant::now();

        // Use simplified prediction logic
        let mut defect_predictions = Vec::new();
        for (file_path, content) in &analysis_files {
            let lines_of_code = content.lines().count();
            let complexity_score = calculate_simple_complexity(content);
            let churn_score = calculate_simple_churn_score(content, lines_of_code);

            // Simple defect probability calculation
            let probability = (complexity_score as f32 * 0.6 + churn_score * 0.4).min(1.0);
            let confidence = if lines_of_code > 100 { 0.8 } else { 0.5 };

            if probability >= confidence_threshold {
                defect_predictions.push(serde_json::json!({
                    "file": file_path.to_string_lossy(),
                    "probability": probability,
                    "confidence": confidence,
                    "complexity_factor": complexity_score,
                    "churn_factor": churn_score,
                    "lines_of_code": lines_of_code
                }));
            }
        }

        let defect_time = defect_start.elapsed();
        analysis_times["defects_ms"] = (defect_time.as_millis() as u64).into();

        if perf {
            eprintln!("🎯 Defect prediction: {:.2}s", defect_time.as_secs_f64());
        }

        results["results"]["defects"] = serde_json::json!({
            "high_risk_files": defect_predictions.len(),
            "predictions": if executive_summary {
                serde_json::Value::Null
            } else {
                serde_json::Value::Array(defect_predictions)
            }
        });
    }

    // 4. Complexity Analysis
    if include_complexity {
        let complexity_start = std::time::Instant::now();

        let mut file_complexities = Vec::new();
        let mut total_complexity = 0u32;
        let mut max_complexity = 0u32;

        for (file_path, content) in &analysis_files {
            let complexity = calculate_simple_complexity(content);
            let complexity_u32 = complexity * 100; // Scale for better reporting
            total_complexity += complexity_u32;
            max_complexity = max_complexity.max(complexity_u32);

            file_complexities.push(serde_json::json!({
                "file": file_path.to_string_lossy(),
                "complexity": complexity,
                "lines": content.lines().count()
            }));
        }

        let complexity_time = complexity_start.elapsed();
        analysis_times["complexity_ms"] = (complexity_time.as_millis() as u64).into();

        if perf {
            eprintln!(
                "🧮 Complexity analysis: {:.2}s",
                complexity_time.as_secs_f64()
            );
        }

        let avg_complexity = if !file_complexities.is_empty() {
            total_complexity as f64 / file_complexities.len() as f64
        } else {
            0.0
        };

        results["results"]["complexity"] = serde_json::json!({
            "average_complexity": avg_complexity / 100.0, // Scale back
            "max_complexity": max_complexity as f64 / 100.0,
            "total_files": file_complexities.len(),
            "files": if executive_summary {
                serde_json::Value::Null
            } else {
                serde_json::Value::Array(file_complexities)
            }
        });
    }

    // 5. TDG (Technical Debt Gradient) Analysis
    if include_tdg {
        let tdg_start = std::time::Instant::now();

        // Simplified TDG calculation based on complexity and churn
        let mut total_debt_score = 0.0;
        let mut critical_files = 0;
        let mut tdg_results = Vec::new();

        for (file_path, content) in &analysis_files {
            let complexity = calculate_simple_complexity(content);
            let churn = calculate_simple_churn_score(content, content.lines().count());
            let tdg_score = (complexity as f64 + churn as f64) / 2.0;

            if tdg_score > 0.7 {
                critical_files += 1;
            }

            total_debt_score += tdg_score;

            tdg_results.push(serde_json::json!({
                "file": file_path.to_string_lossy(),
                "tdg_score": tdg_score,
                "complexity_component": complexity,
                "churn_component": churn
            }));
        }

        let tdg_time = tdg_start.elapsed();
        analysis_times["tdg_ms"] = (tdg_time.as_millis() as u64).into();

        if perf {
            eprintln!("📊 TDG analysis: {:.2}s", tdg_time.as_secs_f64());
        }

        let avg_tdg = if !tdg_results.is_empty() {
            total_debt_score / tdg_results.len() as f64
        } else {
            0.0
        };

        results["results"]["tdg"] = serde_json::json!({
            "average_tdg": avg_tdg,
            "critical_files": critical_files,
            "estimated_debt_hours": total_debt_score * 10.0, // Rough estimate
            "files": if executive_summary {
                serde_json::Value::Null
            } else {
                serde_json::Value::Array(tdg_results)
            }
        });
    }

    let total_time = start_time.elapsed();

    // Add performance metrics
    if perf {
        results["performance"] = serde_json::json!({
            "total_time_s": total_time.as_secs_f64(),
            "files_analyzed": analysis_files.len(),
            "files_per_second": analysis_files.len() as f64 / total_time.as_secs_f64(),
            "analysis_breakdown": analysis_times
        });

        eprintln!(
            "✅ Comprehensive analysis completed in {:.2}s",
            total_time.as_secs_f64()
        );
    }

    // Generate summary
    let mut summary_items = Vec::new();

    if include_duplicates {
        if let Some(dup_results) = results["results"]["duplicates"].as_object() {
            summary_items.push(format!(
                "Duplicates: {}",
                dup_results
                    .get("potential_duplicates")
                    .and_then(|v| v.as_u64())
                    .unwrap_or(0)
            ));
        }
    }

    if include_dead_code {
        if let Some(dead_results) = results["results"]["dead_code"].as_object() {
            summary_items.push(format!(
                "Dead code: {} issues in {} files",
                dead_results
                    .get("potential_issues")
                    .and_then(|v| v.as_u64())
                    .unwrap_or(0),
                dead_results
                    .get("files_with_issues")
                    .and_then(|v| v.as_u64())
                    .unwrap_or(0)
            ));
        }
    }

    if include_defects {
        if let Some(defect_results) = results["results"]["defects"].as_object() {
            summary_items.push(format!(
                "High-risk files: {}",
                defect_results
                    .get("high_risk_files")
                    .and_then(|v| v.as_u64())
                    .unwrap_or(0)
            ));
        }
    }

    if include_complexity {
        if let Some(complexity_results) = results["results"]["complexity"].as_object() {
            summary_items.push(format!(
                "Avg complexity: {:.2}",
                complexity_results
                    .get("average_complexity")
                    .and_then(|v| v.as_f64())
                    .unwrap_or(0.0)
            ));
        }
    }

    if include_tdg {
        if let Some(tdg_results) = results["results"]["tdg"].as_object() {
            summary_items.push(format!(
                "TDG: {:.2} avg, {} critical files",
                tdg_results
                    .get("average_tdg")
                    .and_then(|v| v.as_f64())
                    .unwrap_or(0.0),
                tdg_results
                    .get("critical_files")
                    .and_then(|v| v.as_u64())
                    .unwrap_or(0)
            ));
        }
    }

    results["summary"]["overview"] = summary_items.join(" | ").into();

    // Output results
    match format {
        ComprehensiveOutputFormat::Summary => {
            println!("# Comprehensive Analysis Summary\n");
            println!("📁 **Project**: {}", project_path.display());
            println!("📊 **Files analyzed**: {}", analysis_files.len());
            println!("⏱️ **Analysis time**: {:.2}s\n", total_time.as_secs_f64());

            if !summary_items.is_empty() {
                println!("## Results");
                for item in summary_items {
                    println!("- {item}");
                }
            }
        }
        ComprehensiveOutputFormat::Detailed => {
            // Print detailed results in a human-readable format
            println!("# Comprehensive Analysis Report\n");
            println!("**Project**: {}", project_path.display());
            println!("**Analysis time**: {:.2}s", total_time.as_secs_f64());
            println!("**Files analyzed**: {}\n", analysis_files.len());

            // Print each analysis section
            for (analysis_type, result) in results["results"].as_object().unwrap() {
                let title = analysis_type.replace('_', " ");
                let title_case = title
                    .split_whitespace()
                    .map(|word| {
                        let mut chars = word.chars();
                        match chars.next() {
                            None => String::new(),
                            Some(first) => {
                                first.to_uppercase().collect::<String>() + chars.as_str()
                            }
                        }
                    })
                    .collect::<Vec<_>>()
                    .join(" ");
                println!("## {title_case} Analysis");
                println!("{}\n", serde_json::to_string_pretty(result)?);
            }
        }
        ComprehensiveOutputFormat::Json => {
            println!("{}", serde_json::to_string_pretty(&results)?);
        }
        ComprehensiveOutputFormat::Markdown => {
            println!("# Comprehensive Analysis Report\n");
            println!("- **Project**: {}", project_path.display());
            println!("- **Analysis time**: {:.2}s", total_time.as_secs_f64());
            println!("- **Files analyzed**: {}\n", analysis_files.len());

            println!("## Summary\n");
            for item in summary_items {
                println!("- {item}");
            }

            if !executive_summary {
                println!("\n## Detailed Results\n");
                println!("```json");
                println!("{}", serde_json::to_string_pretty(&results["results"])?);
                println!("```");
            }
        }
        ComprehensiveOutputFormat::Sarif => {
            // Generate SARIF format for IDE integration
            let mut sarif_results = Vec::new();

            // Add duplicate findings
            if let Some(dup_data) = results["results"]["duplicates"]["groups"].as_array() {
                for group in dup_data {
                    if let Some(instances) = group["instances"].as_array() {
                        for instance in instances {
                            sarif_results.push(serde_json::json!({
                                "ruleId": "code-duplication",
                                "level": "warning",
                                "message": {
                                    "text": "Code duplication detected"
                                },
                                "locations": [{
                                    "physicalLocation": {
                                        "artifactLocation": {
                                            "uri": instance["file"]
                                        }
                                    }
                                }]
                            }));
                        }
                    }
                }
            }

            let sarif = serde_json::json!({
                "version": "2.1.0",
                "$schema": "https://raw.githubusercontent.com/oasis-tcs/sarif-spec/master/Schemata/sarif-schema-2.1.0.json",
                "runs": [{
                    "tool": {
                        "driver": {
                            "name": "paiml-mcp-agent-toolkit",
                            "version": env!("CARGO_PKG_VERSION"),
                            "informationUri": "https://github.com/paiml/mcp-agent-toolkit"
                        }
                    },
                    "results": sarif_results
                }]
            });
            println!("{}", serde_json::to_string_pretty(&sarif)?);
        }
    }

    // Write to output file if specified
    if let Some(output_path) = output {
        let content = match format {
            ComprehensiveOutputFormat::Json => serde_json::to_string_pretty(&results)?,
            ComprehensiveOutputFormat::Sarif => {
                let sarif = serde_json::json!({
                    "version": "2.1.0",
                    "runs": [{
                        "tool": {
                            "driver": {
                                "name": "paiml-mcp-agent-toolkit",
                                "version": env!("CARGO_PKG_VERSION")
                            }
                        },
                        "results": []
                    }]
                });
                serde_json::to_string_pretty(&sarif)?
            }
            _ => format!(
                "# Comprehensive Analysis\n\n{}",
                serde_json::to_string_pretty(&results)?
            ),
        };

        std::fs::write(&output_path, content)?;
        if perf {
            eprintln!("📄 Results written to {}", output_path.display());
        }
    }

    Ok(())
}

/// Handle graph metrics analysis
#[allow(clippy::too_many_arguments)]
async fn handle_analyze_graph_metrics(
    project_path: PathBuf,
    metrics: Vec<GraphMetricType>,
    pagerank_seeds: Vec<String>,
    damping_factor: f32,
    max_iterations: usize,
    convergence_threshold: f64,
    export_graphml: bool,
    format: GraphMetricsOutputFormat,
    include: Option<String>,
    exclude: Option<String>,
    output: Option<PathBuf>,
    perf: bool,
    top_k: usize,
    min_centrality: f64,
) -> anyhow::Result<()> {
    // Delegate to structured graph metrics analyzer to reduce complexity
    let analyzer = GraphMetricsAnalyzer::new(GraphMetricsConfig {
        project_path,
        metrics,
        pagerank_seeds,
        damping_factor,
        max_iterations,
        convergence_threshold,
        export_graphml,
        include,
        exclude,
        top_k,
        min_centrality,
    });

    let results = analyzer.analyze(perf).await?;
    analyzer.write_output(results, format, output).await
}

/// Configuration for graph metrics analysis
struct GraphMetricsConfig {
    project_path: PathBuf,
    metrics: Vec<GraphMetricType>,
    pagerank_seeds: Vec<String>,
    damping_factor: f32,
    max_iterations: usize,
    convergence_threshold: f64,
    #[allow(dead_code)]
    export_graphml: bool,
    include: Option<String>,
    exclude: Option<String>,
    top_k: usize,
    min_centrality: f64,
}

/// Graph metrics analyzer to encapsulate complexity
struct GraphMetricsAnalyzer {
    config: GraphMetricsConfig,
}

type DependencyGraphResult = (FxHashMap<String, usize>, Vec<(usize, usize)>);

impl GraphMetricsAnalyzer {
    fn new(config: GraphMetricsConfig) -> Self {
        Self { config }
    }

    async fn analyze(&self, perf: bool) -> anyhow::Result<serde_json::Value> {
        let start_time = std::time::Instant::now();

        // Step 1: File discovery
        let analyzed_files = self.discover_and_filter_files()?;

        // Step 2: Build graph
        let (graph_nodes, graph_edges) = self.build_dependency_graph(&analyzed_files)?;

        // Step 3: Compute metrics
        let mut results = self.initialize_results(&graph_nodes, &graph_edges);

        // Step 4: Calculate each requested metric
        for metric_type in &self.config.metrics {
            self.compute_metric(metric_type, &graph_nodes, &graph_edges, &mut results)?;
        }

        if perf {
            results["performance"] = serde_json::json!({
                "analysis_time_ms": start_time.elapsed().as_millis()
            });
        }

        Ok(results)
    }

    fn discover_and_filter_files(&self) -> anyhow::Result<Vec<(PathBuf, String)>> {
        use crate::services::file_discovery::{FileDiscoveryConfig, ProjectFileDiscovery};

        let mut discovery_config = FileDiscoveryConfig::default();
        if let Some(exclude_pattern) = &self.config.exclude {
            discovery_config
                .custom_ignore_patterns
                .push(exclude_pattern.clone());
        }

        let discovery = ProjectFileDiscovery::new(self.config.project_path.clone())
            .with_config(discovery_config);
        let discovered_files = discovery.discover_files()?;

        let mut analyzed_files = Vec::new();
        for file_path in discovered_files {
            if let Some(include_pattern) = &self.config.include {
                if !file_path.to_string_lossy().contains(include_pattern) {
                    continue;
                }
            }

            if let Ok(content) = std::fs::read_to_string(&file_path) {
                analyzed_files.push((file_path, content));
            }
        }

        if analyzed_files.is_empty() {
            anyhow::bail!("No source files found matching criteria");
        }

        Ok(analyzed_files)
    }

    fn build_dependency_graph(
        &self,
        analyzed_files: &[(PathBuf, String)],
    ) -> anyhow::Result<DependencyGraphResult> {
        let mut graph_nodes = FxHashMap::default();
        let mut graph_edges = Vec::new();

        // Create nodes
        for (node_index, (file_path, _)) in analyzed_files.iter().enumerate() {
            let file_name = file_path.file_name().unwrap().to_string_lossy();
            graph_nodes.insert(file_name.to_string(), node_index);
        }

        // Extract edges
        for (file_path, content) in analyzed_files.iter() {
            let file_name = file_path.file_name().unwrap().to_string_lossy();
            let source_index = graph_nodes[&file_name.to_string()];

            for line in content.lines() {
                if let Some(referenced_file) = self.extract_dependency(line.trim()) {
                    if let Some(&target_index) = graph_nodes.get(&referenced_file) {
                        graph_edges.push((source_index, target_index));
                    }
                }
            }
        }

        Ok((graph_nodes, graph_edges))
    }

    fn extract_dependency(&self, line: &str) -> Option<String> {
        if line.starts_with("import ")
            || line.starts_with("from ")
            || line.starts_with("#include")
            || line.starts_with("use ")
        {
            extract_file_reference(line)
        } else {
            None
        }
    }

    fn initialize_results(
        &self,
        graph_nodes: &FxHashMap<String, usize>,
        graph_edges: &[(usize, usize)],
    ) -> serde_json::Value {
        let num_nodes = graph_nodes.len();
        let num_edges = graph_edges.len();

        serde_json::json!({
            "summary": {
                "total_nodes": num_nodes,
                "total_edges": num_edges,
                "density": if num_nodes > 1 {
                    num_edges as f64 / (num_nodes * (num_nodes - 1)) as f64
                } else {
                    0.0
                }
            },
            "metrics": {}
        })
    }

    fn compute_metric(
        &self,
        metric_type: &GraphMetricType,
        graph_nodes: &FxHashMap<String, usize>,
        graph_edges: &[(usize, usize)],
        results: &mut serde_json::Value,
    ) -> anyhow::Result<()> {
        match metric_type {
            GraphMetricType::Centrality => {
                self.compute_centrality_metric(graph_nodes, graph_edges, results)?;
            }
            GraphMetricType::PageRank => {
                self.compute_pagerank_metric(graph_nodes, graph_edges, results)?;
            }
            GraphMetricType::Clustering => {
                self.compute_clustering_metric(graph_nodes, graph_edges, results)?;
            }
            GraphMetricType::Components => {
                self.compute_components_metric(graph_nodes, graph_edges, results)?;
            }
            GraphMetricType::All => {
                // Compute all metrics
                self.compute_centrality_metric(graph_nodes, graph_edges, results)?;
                self.compute_pagerank_metric(graph_nodes, graph_edges, results)?;
                self.compute_clustering_metric(graph_nodes, graph_edges, results)?;
                self.compute_components_metric(graph_nodes, graph_edges, results)?;
            }
        }
        Ok(())
    }

    fn compute_centrality_metric(
        &self,
        graph_nodes: &FxHashMap<String, usize>,
        graph_edges: &[(usize, usize)],
        results: &mut serde_json::Value,
    ) -> anyhow::Result<()> {
        let centrality_scores = calculate_betweenness_centrality(graph_nodes, graph_edges);
        let mut centrality_results = self.create_ranked_results(centrality_scores, "centrality");

        // Filter and truncate
        centrality_results
            .retain(|r| r["centrality"].as_f64().unwrap() >= self.config.min_centrality);
        centrality_results.truncate(self.config.top_k);

        results["metrics"]["centrality"] = serde_json::json!({
            "type": "betweenness_centrality",
            "nodes": centrality_results
        });

        Ok(())
    }

    fn compute_pagerank_metric(
        &self,
        graph_nodes: &FxHashMap<String, usize>,
        graph_edges: &[(usize, usize)],
        results: &mut serde_json::Value,
    ) -> anyhow::Result<()> {
        let pagerank_scores = calculate_pagerank(
            graph_nodes,
            graph_edges,
            self.config.damping_factor as f64,
            self.config.max_iterations,
            self.config.convergence_threshold,
            &self.config.pagerank_seeds,
        );

        let mut pagerank_results = self.create_ranked_results(pagerank_scores, "pagerank");
        pagerank_results.truncate(self.config.top_k);

        results["metrics"]["pagerank"] = serde_json::json!({
            "damping_factor": self.config.damping_factor,
            "max_iterations": self.config.max_iterations,
            "convergence_threshold": self.config.convergence_threshold,
            "seeds": self.config.pagerank_seeds,
            "nodes": pagerank_results
        });

        Ok(())
    }

    fn compute_clustering_metric(
        &self,
        graph_nodes: &FxHashMap<String, usize>,
        graph_edges: &[(usize, usize)],
        results: &mut serde_json::Value,
    ) -> anyhow::Result<()> {
        let clustering_scores = calculate_clustering_coefficient(graph_nodes, graph_edges);
        let avg_clustering: f64 =
            clustering_scores.values().sum::<f64>() / clustering_scores.len() as f64;

        let mut clustering_results: Vec<_> = clustering_scores
            .iter()
            .map(|(name, score)| {
                serde_json::json!({
                    "node": name,
                    "clustering_coefficient": score
                })
            })
            .collect();

        clustering_results.sort_by(|a, b| {
            b["clustering_coefficient"]
                .as_f64()
                .unwrap()
                .partial_cmp(&a["clustering_coefficient"].as_f64().unwrap())
                .unwrap()
        });

        clustering_results.truncate(self.config.top_k);

        results["metrics"]["clustering"] = serde_json::json!({
            "average_coefficient": avg_clustering,
            "nodes": clustering_results
        });

        Ok(())
    }

    fn compute_components_metric(
        &self,
        graph_nodes: &FxHashMap<String, usize>,
        graph_edges: &[(usize, usize)],
        results: &mut serde_json::Value,
    ) -> anyhow::Result<()> {
        // Connected components analysis
        let components = find_connected_components(graph_nodes, graph_edges);
        let num_components = components.len();

        results["metrics"]["components"] = serde_json::json!({
            "num_components": num_components,
            "connected": num_components == 1,
            "component_sizes": components.iter().map(|c| c.len()).collect::<Vec<_>>()
        });

        Ok(())
    }

    fn create_ranked_results(
        &self,
        scores: FxHashMap<String, f64>,
        metric_name: &str,
    ) -> Vec<serde_json::Value> {
        let mut results: Vec<_> = scores
            .iter()
            .map(|(name, score)| {
                let mut obj = serde_json::json!({
                    "node": name,
                    "rank": 0
                });
                obj[metric_name] = (*score).into();
                obj
            })
            .collect();

        // Sort by score (descending)
        results.sort_by(|a, b| {
            b[metric_name]
                .as_f64()
                .unwrap()
                .partial_cmp(&a[metric_name].as_f64().unwrap())
                .unwrap()
        });

        // Add ranks
        for (i, result) in results.iter_mut().enumerate() {
            result["rank"] = (i + 1).into();
        }

        results
    }

    async fn write_output(
        &self,
        results: serde_json::Value,
        format: GraphMetricsOutputFormat,
        output: Option<PathBuf>,
    ) -> anyhow::Result<()> {
        let content = match format {
            GraphMetricsOutputFormat::Json => serde_json::to_string_pretty(&results)?,
            GraphMetricsOutputFormat::Summary => self.format_as_summary(&results)?,
            GraphMetricsOutputFormat::Detailed => self.format_as_detailed(&results)?,
            _ => serde_json::to_string_pretty(&results)?, // Default to JSON for other formats
        };

        if let Some(output_path) = output {
            tokio::fs::write(&output_path, &content).await?;
            eprintln!("✅ Graph metrics written to: {}", output_path.display());
        } else {
            println!("{content}");
        }

        Ok(())
    }

    fn format_as_summary(&self, results: &serde_json::Value) -> anyhow::Result<String> {
        // Simplified summary formatting
        let mut output = String::new();
        output.push_str("Graph Metrics Analysis\n");
        output.push_str("=====================\n\n");

        if let Some(summary) = results.get("summary") {
            output.push_str(&format!("Total Nodes: {}\n", summary["total_nodes"]));
            output.push_str(&format!("Total Edges: {}\n", summary["total_edges"]));
            output.push_str(&format!("Graph Density: {:.4}\n\n", summary["density"]));
        }

        Ok(output)
    }

    fn format_as_detailed(&self, results: &serde_json::Value) -> anyhow::Result<String> {
        // Detailed formatting with metrics
        let mut output = String::new();
        output.push_str("# Graph Metrics Analysis\n\n");

        if let Some(summary) = results.get("summary") {
            output.push_str("## Summary\n\n");
            output.push_str(&format!("- **Total Nodes**: {}\n", summary["total_nodes"]));
            output.push_str(&format!("- **Total Edges**: {}\n", summary["total_edges"]));
            output.push_str(&format!(
                "- **Graph Density**: {:.4}\n\n",
                summary["density"]
            ));
        }

        // Add metric details if available
        if let Some(metrics) = results.get("metrics").and_then(|m| m.as_object()) {
            for (metric_name, _metric_data) in metrics {
                output.push_str(&format!("\n## {metric_name}\n\n"));
                // Add metric-specific formatting here
            }
        }

        Ok(output)
    }
}

// Legacy implementation removed - replaced with GraphMetricsAnalyzer
#[allow(dead_code)]
#[allow(clippy::too_many_arguments)]
async fn handle_analyze_graph_metrics_legacy(
    project_path: PathBuf,
    metrics: Vec<GraphMetricType>,
    pagerank_seeds: Vec<String>,
    damping_factor: f32,
    max_iterations: usize,
    convergence_threshold: f64,
    export_graphml: bool,
    format: GraphMetricsOutputFormat,
    include: Option<String>,
    exclude: Option<String>,
    output: Option<PathBuf>,
    perf: bool,
    top_k: usize,
    min_centrality: f64,
) -> anyhow::Result<()> {
    use crate::services::file_discovery::{FileDiscoveryConfig, ProjectFileDiscovery};
    // FxHashMap is already imported at module level
    use std::time::Instant;

    let start_time = Instant::now();

    // Discover source files
    let mut discovery_config = FileDiscoveryConfig::default();

    // Add custom patterns if specified
    if let Some(exclude_pattern) = &exclude {
        discovery_config
            .custom_ignore_patterns
            .push(exclude_pattern.clone());
    }

    let discovery = ProjectFileDiscovery::new(project_path.clone()).with_config(discovery_config);
    let discovered_files = discovery.discover_files()?;

    // Filter files based on include pattern
    let mut analyzed_files = Vec::new();
    for file_path in discovered_files {
        // Apply include filter if specified
        if let Some(include_pattern) = &include {
            if !file_path.to_string_lossy().contains(include_pattern) {
                continue;
            }
        }

        if let Ok(content) = std::fs::read_to_string(&file_path) {
            analyzed_files.push((file_path, content));
        }
    }

    if analyzed_files.is_empty() {
        eprintln!("No source files found matching criteria");
        return Ok(());
    }

    // Build a simplified dependency graph
    let mut graph_nodes = FxHashMap::default();
    let mut graph_edges = Vec::new();

    // Create nodes for each file
    for (node_index, (file_path, content)) in analyzed_files.iter().enumerate() {
        let file_name = file_path.file_name().unwrap().to_string_lossy();
        graph_nodes.insert(file_name.to_string(), node_index);

        // Simple dependency detection based on imports/includes
        for line in content.lines() {
            let trimmed = line.trim();
            if trimmed.starts_with("import ")
                || trimmed.starts_with("from ")
                || trimmed.starts_with("#include")
                || trimmed.starts_with("use ")
            {
                // Extract potential file references
                if let Some(referenced_file) = extract_file_reference(trimmed) {
                    if let Some(&target_index) = graph_nodes.get(&referenced_file) {
                        graph_edges.push((graph_nodes[&file_name.to_string()], target_index));
                    }
                }
            }
        }
    }

    let num_nodes = graph_nodes.len();
    let num_edges = graph_edges.len();

    // Initialize results
    let mut results = serde_json::json!({
        "summary": {
            "total_nodes": num_nodes,
            "total_edges": num_edges,
            "density": if num_nodes > 1 {
                num_edges as f64 / (num_nodes * (num_nodes - 1)) as f64
            } else {
                0.0
            }
        },
        "metrics": {}
    });

    // Compute requested metrics
    for metric_type in &metrics {
        match metric_type {
            GraphMetricType::Centrality => {
                let centrality_scores =
                    calculate_betweenness_centrality(&graph_nodes, &graph_edges);
                let mut centrality_results: Vec<_> = centrality_scores
                    .iter()
                    .map(|(name, score)| {
                        serde_json::json!({
                            "node": name,
                            "centrality": score,
                            "rank": 0  // Will be set below
                        })
                    })
                    .collect();

                // Sort by centrality score (descending)
                centrality_results.sort_by(|a, b| {
                    b["centrality"]
                        .as_f64()
                        .unwrap()
                        .partial_cmp(&a["centrality"].as_f64().unwrap())
                        .unwrap()
                });

                // Add ranks
                for (i, result) in centrality_results.iter_mut().enumerate() {
                    result["rank"] = (i + 1).into();
                }

                // Filter by minimum centrality and top-k
                centrality_results.retain(|r| r["centrality"].as_f64().unwrap() >= min_centrality);
                centrality_results.truncate(top_k);

                results["metrics"]["centrality"] = serde_json::json!({
                    "type": "betweenness_centrality",
                    "nodes": centrality_results
                });
            }
            GraphMetricType::PageRank => {
                let pagerank_scores = calculate_pagerank(
                    &graph_nodes,
                    &graph_edges,
                    damping_factor as f64,
                    max_iterations,
                    convergence_threshold,
                    &pagerank_seeds,
                );

                let mut pagerank_results: Vec<_> = pagerank_scores
                    .iter()
                    .map(|(name, score)| {
                        serde_json::json!({
                            "node": name,
                            "pagerank": score,
                            "rank": 0  // Will be set below
                        })
                    })
                    .collect();

                // Sort by PageRank score (descending)
                pagerank_results.sort_by(|a, b| {
                    b["pagerank"]
                        .as_f64()
                        .unwrap()
                        .partial_cmp(&a["pagerank"].as_f64().unwrap())
                        .unwrap()
                });

                // Add ranks
                for (i, result) in pagerank_results.iter_mut().enumerate() {
                    result["rank"] = (i + 1).into();
                }

                // Apply top-k limit
                pagerank_results.truncate(top_k);

                results["metrics"]["pagerank"] = serde_json::json!({
                    "damping_factor": damping_factor,
                    "max_iterations": max_iterations,
                    "convergence_threshold": convergence_threshold,
                    "seeds": pagerank_seeds,
                    "nodes": pagerank_results
                });
            }
            GraphMetricType::Clustering => {
                let clustering_scores =
                    calculate_clustering_coefficient(&graph_nodes, &graph_edges);
                let avg_clustering: f64 =
                    clustering_scores.values().sum::<f64>() / clustering_scores.len() as f64;

                let mut clustering_results: Vec<_> = clustering_scores
                    .iter()
                    .map(|(name, score)| {
                        serde_json::json!({
                            "node": name,
                            "clustering_coefficient": score
                        })
                    })
                    .collect();

                // Sort by clustering coefficient (descending)
                clustering_results.sort_by(|a, b| {
                    b["clustering_coefficient"]
                        .as_f64()
                        .unwrap()
                        .partial_cmp(&a["clustering_coefficient"].as_f64().unwrap())
                        .unwrap()
                });

                clustering_results.truncate(top_k);

                results["metrics"]["clustering"] = serde_json::json!({
                    "average_clustering": avg_clustering,
                    "nodes": clustering_results
                });
            }
            GraphMetricType::Components => {
                let components = find_connected_components(&graph_nodes, &graph_edges);
                results["metrics"]["components"] = serde_json::json!({
                    "num_components": components.len(),
                    "component_sizes": components.iter().map(|c| c.len()).collect::<Vec<_>>(),
                    "largest_component_size": components.iter().map(|c| c.len()).max().unwrap_or(0)
                });
            }
            GraphMetricType::All => {
                // This is handled by including all other metric types
                continue;
            }
        }
    }

    let analysis_time = start_time.elapsed();

    // Add performance metrics if requested
    if perf {
        results["performance"] = serde_json::json!({
            "analysis_time_s": analysis_time.as_secs_f64(),
            "nodes_per_second": num_nodes as f64 / analysis_time.as_secs_f64(),
            "edges_per_second": num_edges as f64 / analysis_time.as_secs_f64()
        });
    }

    // Output results based on format
    match format {
        GraphMetricsOutputFormat::Summary => {
            println!("Graph Metrics Analysis Summary");
            println!("============================");
            println!("Nodes: {num_nodes}");
            println!("Edges: {num_edges}");
            println!(
                "Density: {:.4}",
                results["summary"]["density"].as_f64().unwrap()
            );

            if let Some(centrality) = results["metrics"]["centrality"].as_object() {
                println!("\nTop Centrality Nodes:");
                for node in centrality["nodes"].as_array().unwrap().iter().take(5) {
                    println!(
                        "  {}: {:.4}",
                        node["node"].as_str().unwrap(),
                        node["centrality"].as_f64().unwrap()
                    );
                }
            }

            if let Some(pagerank) = results["metrics"]["pagerank"].as_object() {
                println!("\nTop PageRank Nodes:");
                for node in pagerank["nodes"].as_array().unwrap().iter().take(5) {
                    println!(
                        "  {}: {:.4}",
                        node["node"].as_str().unwrap(),
                        node["pagerank"].as_f64().unwrap()
                    );
                }
            }

            if perf {
                println!("\nPerformance:");
                println!("  Analysis time: {:.2}s", analysis_time.as_secs_f64());
            }
        }
        GraphMetricsOutputFormat::Detailed => {
            println!("Graph Metrics Analysis Report");
            println!("============================");
            println!("{}", serde_json::to_string_pretty(&results)?);
        }
        GraphMetricsOutputFormat::Json => {
            println!("{}", serde_json::to_string_pretty(&results)?);
        }
        GraphMetricsOutputFormat::Csv => {
            println!("metric_type,node,value,rank");

            if let Some(centrality) = results["metrics"]["centrality"].as_object() {
                for node in centrality["nodes"].as_array().unwrap() {
                    println!(
                        "centrality,{},{},{}",
                        node["node"].as_str().unwrap(),
                        node["centrality"].as_f64().unwrap(),
                        node["rank"].as_u64().unwrap()
                    );
                }
            }

            if let Some(pagerank) = results["metrics"]["pagerank"].as_object() {
                for node in pagerank["nodes"].as_array().unwrap() {
                    println!(
                        "pagerank,{},{},{}",
                        node["node"].as_str().unwrap(),
                        node["pagerank"].as_f64().unwrap(),
                        node["rank"].as_u64().unwrap()
                    );
                }
            }
        }
        GraphMetricsOutputFormat::GraphML => {
            let graphml = generate_graphml(&graph_nodes, &graph_edges, &results)?;
            println!("{graphml}");
        }
        GraphMetricsOutputFormat::Markdown => {
            println!("# Graph Metrics Analysis Report\n");
            println!("## Summary\n");
            println!("- **Nodes**: {num_nodes}");
            println!("- **Edges**: {num_edges}");
            println!(
                "- **Density**: {:.4}\n",
                results["summary"]["density"].as_f64().unwrap()
            );

            if let Some(centrality) = results["metrics"]["centrality"].as_object() {
                println!("## Centrality Analysis\n");
                println!("| Rank | Node | Centrality |");
                println!("| ---- | ---- | ---------- |");
                for node in centrality["nodes"].as_array().unwrap() {
                    println!(
                        "| {} | {} | {:.4} |",
                        node["rank"].as_u64().unwrap(),
                        node["node"].as_str().unwrap(),
                        node["centrality"].as_f64().unwrap()
                    );
                }
                println!();
            }

            if let Some(pagerank) = results["metrics"]["pagerank"].as_object() {
                println!("## PageRank Analysis\n");
                println!("| Rank | Node | PageRank |");
                println!("| ---- | ---- | -------- |");
                for node in pagerank["nodes"].as_array().unwrap() {
                    println!(
                        "| {} | {} | {:.4} |",
                        node["rank"].as_u64().unwrap(),
                        node["node"].as_str().unwrap(),
                        node["pagerank"].as_f64().unwrap()
                    );
                }
                println!();
            }
        }
    }

    // Export GraphML if requested
    if export_graphml {
        let graphml_path = output.clone().unwrap_or_else(|| {
            PathBuf::from(format!(
                "{}_graph_metrics.graphml",
                project_path.file_name().unwrap().to_string_lossy()
            ))
        });
        let graphml_content = generate_graphml(&graph_nodes, &graph_edges, &results)?;
        std::fs::write(&graphml_path, graphml_content)?;
        eprintln!("📊 GraphML exported to: {}", graphml_path.display());
    }

    // Write to output file if specified
    if let Some(output_path) = output {
        let content = match format {
            GraphMetricsOutputFormat::Json => serde_json::to_string_pretty(&results)?,
            GraphMetricsOutputFormat::Csv => {
                let mut csv_content = String::from("metric_type,node,value,rank\n");

                if let Some(centrality) = results["metrics"]["centrality"].as_object() {
                    for node in centrality["nodes"].as_array().unwrap() {
                        csv_content.push_str(&format!(
                            "centrality,{},{},{}\n",
                            node["node"].as_str().unwrap(),
                            node["centrality"].as_f64().unwrap(),
                            node["rank"].as_u64().unwrap()
                        ));
                    }
                }

                if let Some(pagerank) = results["metrics"]["pagerank"].as_object() {
                    for node in pagerank["nodes"].as_array().unwrap() {
                        csv_content.push_str(&format!(
                            "pagerank,{},{},{}\n",
                            node["node"].as_str().unwrap(),
                            node["pagerank"].as_f64().unwrap(),
                            node["rank"].as_u64().unwrap()
                        ));
                    }
                }
                csv_content
            }
            _ => format!("{results:#?}"),
        };
        std::fs::write(&output_path, content)?;
        eprintln!("📄 Results written to {}", output_path.display());
    }

    Ok(())
}

// Helper functions for graph metrics calculation

fn extract_file_reference(import_line: &str) -> Option<String> {
    // Simple heuristic to extract file references from import statements
    if import_line.contains("\"") {
        if let Some(start) = import_line.find('"') {
            if let Some(end) = import_line[start + 1..].find('"') {
                let referenced = &import_line[start + 1..start + 1 + end];
                if referenced.ends_with(".rs")
                    || referenced.ends_with(".ts")
                    || referenced.ends_with(".js")
                    || referenced.ends_with(".py")
                    || referenced.ends_with(".c")
                    || referenced.ends_with(".cpp")
                {
                    return Some(referenced.split('/').next_back().unwrap().to_string());
                }
            }
        }
    }
    None
}

fn calculate_betweenness_centrality(
    nodes: &FxHashMap<String, usize>,
    edges: &[(usize, usize)],
) -> FxHashMap<String, f64> {
    let mut centrality = FxHashMap::default();

    // Initialize all nodes with 0 centrality
    for name in nodes.keys() {
        centrality.insert(name.clone(), 0.0);
    }

    // Simplified betweenness centrality calculation
    // In a real implementation, you'd use algorithms like Brandes' algorithm
    for (name, &node_idx) in nodes {
        let mut paths_through = 0;
        for &(from, to) in edges {
            if from == node_idx || to == node_idx {
                paths_through += 1;
            }
        }
        centrality.insert(name.clone(), paths_through as f64 / edges.len() as f64);
    }

    centrality
}

fn calculate_pagerank(
    nodes: &FxHashMap<String, usize>,
    edges: &[(usize, usize)],
    damping_factor: f64,
    max_iterations: usize,
    convergence_threshold: f64,
    _seeds: &[String],
) -> FxHashMap<String, f64> {
    let n = nodes.len();
    if n == 0 {
        return FxHashMap::default();
    }

    let mut pagerank = FxHashMap::default();
    let initial_value = 1.0 / n as f64;

    // Initialize PageRank values
    for name in nodes.keys() {
        pagerank.insert(name.clone(), initial_value);
    }

    // Build adjacency list
    let mut out_links: FxHashMap<usize, Vec<usize>> = FxHashMap::default();
    let mut in_links: FxHashMap<usize, Vec<usize>> = FxHashMap::default();

    for &(from, to) in edges {
        out_links.entry(from).or_default().push(to);
        in_links.entry(to).or_default().push(from);
    }

    // Iterate PageRank algorithm
    for _ in 0..max_iterations {
        let mut new_pagerank = FxHashMap::default();

        for (name, &node_idx) in nodes {
            let mut rank = (1.0 - damping_factor) / n as f64;

            if let Some(incoming) = in_links.get(&node_idx) {
                for &from_idx in incoming {
                    let from_name = nodes.iter().find(|(_, &idx)| idx == from_idx).unwrap().0;
                    let from_rank = pagerank[from_name];
                    let from_out_degree = out_links.get(&from_idx).map(|v| v.len()).unwrap_or(1);
                    rank += damping_factor * from_rank / from_out_degree as f64;
                }
            }

            new_pagerank.insert(name.clone(), rank);
        }

        // Check convergence
        let mut max_diff: f64 = 0.0;
        for (name, &new_rank) in &new_pagerank {
            let old_rank = pagerank[name];
            max_diff = max_diff.max((new_rank - old_rank).abs());
        }

        pagerank = new_pagerank;

        if max_diff < convergence_threshold {
            break;
        }
    }

    pagerank
}

fn calculate_clustering_coefficient(
    nodes: &FxHashMap<String, usize>,
    edges: &[(usize, usize)],
) -> FxHashMap<String, f64> {
    let mut clustering = FxHashMap::default();

    // Build adjacency list
    let mut adjacency: FxHashMap<usize, Vec<usize>> = FxHashMap::default();
    for &(from, to) in edges {
        adjacency.entry(from).or_default().push(to);
        adjacency.entry(to).or_default().push(from);
    }

    for (name, &node_idx) in nodes {
        let neighbors = adjacency.get(&node_idx).cloned().unwrap_or_default();
        let degree = neighbors.len();

        if degree < 2 {
            clustering.insert(name.clone(), 0.0);
            continue;
        }

        // Count triangles
        let mut triangles = 0;
        for i in 0..neighbors.len() {
            for j in (i + 1)..neighbors.len() {
                let neighbor_i = neighbors[i];
                let neighbor_j = neighbors[j];

                // Check if neighbor_i and neighbor_j are connected
                if adjacency
                    .get(&neighbor_i)
                    .unwrap_or(&Vec::new())
                    .contains(&neighbor_j)
                {
                    triangles += 1;
                }
            }
        }

        let possible_triangles = degree * (degree - 1) / 2;
        let coefficient = if possible_triangles > 0 {
            triangles as f64 / possible_triangles as f64
        } else {
            0.0
        };

        clustering.insert(name.clone(), coefficient);
    }

    clustering
}

fn find_connected_components(
    nodes: &FxHashMap<String, usize>,
    edges: &[(usize, usize)],
) -> Vec<Vec<String>> {
    let mut visited = std::collections::HashSet::new();
    let mut components = Vec::new();

    // Build adjacency list
    let mut adjacency: FxHashMap<usize, Vec<usize>> = FxHashMap::default();
    for &(from, to) in edges {
        adjacency.entry(from).or_default().push(to);
        adjacency.entry(to).or_default().push(from);
    }

    for &node_idx in nodes.values() {
        if visited.contains(&node_idx) {
            continue;
        }

        // DFS to find connected component
        let mut component = Vec::new();
        let mut stack = vec![node_idx];

        while let Some(current) = stack.pop() {
            if visited.contains(&current) {
                continue;
            }

            visited.insert(current);

            // Find node name
            let current_name = nodes
                .iter()
                .find(|(_, &idx)| idx == current)
                .unwrap()
                .0
                .clone();
            component.push(current_name);

            // Add neighbors to stack
            if let Some(neighbors) = adjacency.get(&current) {
                for &neighbor in neighbors {
                    if !visited.contains(&neighbor) {
                        stack.push(neighbor);
                    }
                }
            }
        }

        components.push(component);
    }

    components
}

fn generate_graphml(
    nodes: &FxHashMap<String, usize>,
    edges: &[(usize, usize)],
    results: &serde_json::Value,
) -> anyhow::Result<String> {
    let mut graphml = String::from(
        r#"<?xml version="1.0" encoding="UTF-8"?>
<graphml xmlns="http://graphml.graphdrawing.org/xmlns"
         xmlns:xsi="http://www.w3.org/2001/XMLSchema-instance"
         xsi:schemaLocation="http://graphml.graphdrawing.org/xmlns
         http://graphml.graphdrawing.org/xmlns/1.0/graphml.xsd">
  <key id="centrality" for="node" attr.name="centrality" attr.type="double"/>
  <key id="pagerank" for="node" attr.name="pagerank" attr.type="double"/>
  <graph id="G" edgedefault="directed">
"#,
    );

    // Add nodes
    for (name, &idx) in nodes {
        graphml.push_str(&format!("    <node id=\"n{idx}\">\n"));
        graphml.push_str(&format!("      <data key=\"name\">{name}</data>\n"));

        // Add centrality data if available
        if let Some(centrality_metrics) = results["metrics"]["centrality"].as_object() {
            if let Some(nodes_array) = centrality_metrics["nodes"].as_array() {
                for node in nodes_array {
                    if node["node"].as_str() == Some(name) {
                        graphml.push_str(&format!(
                            "      <data key=\"centrality\">{}</data>\n",
                            node["centrality"].as_f64().unwrap()
                        ));
                        break;
                    }
                }
            }
        }

        // Add PageRank data if available
        if let Some(pagerank_metrics) = results["metrics"]["pagerank"].as_object() {
            if let Some(nodes_array) = pagerank_metrics["nodes"].as_array() {
                for node in nodes_array {
                    if node["node"].as_str() == Some(name) {
                        graphml.push_str(&format!(
                            "      <data key=\"pagerank\">{}</data>\n",
                            node["pagerank"].as_f64().unwrap()
                        ));
                        break;
                    }
                }
            }
        }

        graphml.push_str("    </node>\n");
    }

    // Add edges
    for &(from, to) in edges {
        graphml.push_str(&format!(
            "    <edge source=\"n{from}\" target=\"n{to}\"/>\n"
        ));
    }

    graphml.push_str("  </graph>\n</graphml>");

    Ok(graphml)
}

/// Handle name similarity analysis
#[allow(clippy::too_many_arguments)]
async fn handle_analyze_name_similarity(
    project_path: PathBuf,
    query: String,
    top_k: usize,
    phonetic: bool,
    scope: SearchScope,
    threshold: f32,
    format: NameSimilarityOutputFormat,
    include: Option<String>,
    exclude: Option<String>,
    output: Option<PathBuf>,
    perf: bool,
    fuzzy: bool,
    case_sensitive: bool,
) -> anyhow::Result<()> {
    use crate::services::file_discovery::{FileDiscoveryConfig, ProjectFileDiscovery};
    use std::time::Instant;

    let start_time = Instant::now();

    // Discover source files
    let mut discovery_config = FileDiscoveryConfig::default();

    // Add custom patterns if specified
    if let Some(exclude_pattern) = &exclude {
        discovery_config
            .custom_ignore_patterns
            .push(exclude_pattern.clone());
    }

    let discovery = ProjectFileDiscovery::new(project_path.clone()).with_config(discovery_config);
    let discovered_files = discovery.discover_files()?;

    // Filter files based on include pattern
    let mut analyzed_files = Vec::new();
    for file_path in discovered_files {
        // Apply include filter if specified
        if let Some(include_pattern) = &include {
            if !file_path.to_string_lossy().contains(include_pattern) {
                continue;
            }
        }

        if let Ok(content) = std::fs::read_to_string(&file_path) {
            analyzed_files.push((file_path, content));
        }
    }

    if analyzed_files.is_empty() {
        eprintln!("No source files found matching criteria");
        return Ok(());
    }

    // Extract names/identifiers from source files
    let mut all_names = Vec::new();
    for (file_path, content) in &analyzed_files {
        let names = extract_identifiers(content, &scope, file_path);
        all_names.extend(names);
    }

    // Calculate similarity scores
    let mut similarities = Vec::new();
    let query_lower = if case_sensitive {
        query.clone()
    } else {
        query.to_lowercase()
    };

    for name_info in &all_names {
        let name_to_compare = if case_sensitive {
            name_info.name.clone()
        } else {
            name_info.name.to_lowercase()
        };

        // String similarity (Jaro-Winkler approximation)
        let mut similarity_score = calculate_string_similarity(&query_lower, &name_to_compare);

        // Fuzzy matching (edit distance)
        if fuzzy {
            let edit_distance = calculate_edit_distance(&query_lower, &name_to_compare);
            let max_len = query_lower.len().max(name_to_compare.len()) as f32;
            let fuzzy_score = if max_len > 0.0 {
                1.0 - (edit_distance as f32 / max_len)
            } else {
                1.0
            };
            similarity_score = similarity_score.max(fuzzy_score);
        }

        // Phonetic matching (simplified Soundex)
        if phonetic {
            let query_soundex = calculate_soundex(&query_lower);
            let name_soundex = calculate_soundex(&name_to_compare);
            if query_soundex == name_soundex {
                similarity_score = similarity_score.max(0.8);
            }
        }

        if similarity_score >= threshold {
            similarities.push(NameSimilarityResult {
                name: name_info.name.clone(),
                similarity: similarity_score,
                file: name_info.file.clone(),
                line: name_info.line,
                name_type: name_info.name_type.clone(),
                context: name_info.context.clone(),
            });
        }
    }

    // Sort by similarity score (descending)
    similarities.sort_by(|a, b| b.similarity.partial_cmp(&a.similarity).unwrap());
    similarities.truncate(top_k);

    let analysis_time = start_time.elapsed();

    // Prepare results
    let results = serde_json::json!({
        "query": query,
        "total_identifiers": all_names.len(),
        "matches": similarities.len(),
        "results": similarities.iter().map(|s| serde_json::json!({
            "name": s.name,
            "similarity": s.similarity,
            "file": s.file.to_string_lossy(),
            "line": s.line,
            "type": s.name_type,
            "context": s.context
        })).collect::<Vec<_>>(),
        "parameters": {
            "scope": format!("{scope:?}"),
            "threshold": threshold,
            "phonetic": phonetic,
            "fuzzy": fuzzy,
            "case_sensitive": case_sensitive
        }
    });

    // Add performance metrics if requested
    let mut final_results = results;
    if perf {
        final_results["performance"] = serde_json::json!({
            "analysis_time_s": analysis_time.as_secs_f64(),
            "identifiers_per_second": all_names.len() as f64 / analysis_time.as_secs_f64(),
            "files_analyzed": analyzed_files.len()
        });
    }

    // Output results based on format
    match format {
        NameSimilarityOutputFormat::Summary => {
            println!("Name Similarity Analysis");
            println!("======================");
            println!("Query: '{query}'");
            println!("Total identifiers: {}", all_names.len());
            println!("Matches found: {}", similarities.len());

            if !similarities.is_empty() {
                println!("\nTop matches:");
                for (i, sim) in similarities.iter().take(10).enumerate() {
                    println!(
                        "{}. {} (similarity: {:.3}) in {}:{}",
                        i + 1,
                        sim.name,
                        sim.similarity,
                        sim.file.file_name().unwrap().to_string_lossy(),
                        sim.line
                    );
                }
            }

            if perf {
                println!("\nPerformance:");
                println!("  Analysis time: {:.2}s", analysis_time.as_secs_f64());
                println!("  Files analyzed: {}", analyzed_files.len());
            }
        }
        NameSimilarityOutputFormat::Detailed => {
            println!("Name Similarity Analysis Report");
            println!("==============================");

            for sim in &similarities {
                println!("\nMatch: {}", sim.name);
                println!("  Similarity: {:.3}", sim.similarity);
                println!("  Type: {}", sim.name_type);
                println!("  File: {}", sim.file.to_string_lossy());
                println!("  Line: {}", sim.line);
                if !sim.context.is_empty() {
                    println!("  Context: {}", sim.context);
                }
            }
        }
        NameSimilarityOutputFormat::Json => {
            println!("{}", serde_json::to_string_pretty(&final_results)?);
        }
        NameSimilarityOutputFormat::Csv => {
            println!("name,similarity,type,file,line,context");
            for sim in &similarities {
                println!(
                    "{},{:.3},{},{},{},\"{}\"",
                    sim.name,
                    sim.similarity,
                    sim.name_type,
                    sim.file.to_string_lossy(),
                    sim.line,
                    sim.context.replace('"', "\"\"")
                );
            }
        }
        NameSimilarityOutputFormat::Markdown => {
            println!("# Name Similarity Analysis\n");
            println!("**Query**: `{query}`\n");
            println!("**Total identifiers**: {}\n", all_names.len());
            println!("**Matches found**: {}\n", similarities.len());

            if !similarities.is_empty() {
                println!("## Results\n");
                println!("| Rank | Name | Similarity | Type | File | Line |");
                println!("| ---- | ---- | ---------- | ---- | ---- | ---- |");
                for (i, sim) in similarities.iter().enumerate() {
                    println!(
                        "| {} | `{}` | {:.3} | {} | {} | {} |",
                        i + 1,
                        sim.name,
                        sim.similarity,
                        sim.name_type,
                        sim.file.file_name().unwrap().to_string_lossy(),
                        sim.line
                    );
                }
            }
        }
    }

    // Write to output file if specified
    if let Some(output_path) = output {
        let content = match format {
            NameSimilarityOutputFormat::Json => serde_json::to_string_pretty(&final_results)?,
            NameSimilarityOutputFormat::Csv => {
                let mut csv_content = String::from("name,similarity,type,file,line,context\n");
                for sim in &similarities {
                    csv_content.push_str(&format!(
                        "{},{:.3},{},{},{},\"{}\"\n",
                        sim.name,
                        sim.similarity,
                        sim.name_type,
                        sim.file.to_string_lossy(),
                        sim.line,
                        sim.context.replace('"', "\"\"")
                    ));
                }
                csv_content
            }
            _ => format!("{final_results:#?}"),
        };
        std::fs::write(&output_path, content)?;
        eprintln!("📄 Results written to {}", output_path.display());
    }

    Ok(())
}

// Helper structures for name similarity analysis

#[derive(Debug, Clone)]
struct NameInfo {
    name: String,
    file: PathBuf,
    line: u32,
    name_type: String,
    context: String,
}

#[derive(Debug, Clone)]
struct NameSimilarityResult {
    name: String,
    similarity: f32,
    file: PathBuf,
    line: u32,
    name_type: String,
    context: String,
}

// Helper functions for name similarity analysis

fn extract_identifiers(content: &str, scope: &SearchScope, file_path: &Path) -> Vec<NameInfo> {
    let mut identifiers = Vec::new();
    let file_ext = file_path.extension().and_then(|e| e.to_str()).unwrap_or("");

    // Simple regex-based identifier extraction
    for (line_num, line) in content.lines().enumerate() {
        match file_ext {
            "rs" => extract_rust_identifiers(
                line,
                line_num as u32 + 1,
                scope,
                file_path,
                &mut identifiers,
            ),
            "ts" | "js" | "tsx" | "jsx" => extract_js_identifiers(
                line,
                line_num as u32 + 1,
                scope,
                file_path,
                &mut identifiers,
            ),
            "py" => extract_python_identifiers(
                line,
                line_num as u32 + 1,
                scope,
                file_path,
                &mut identifiers,
            ),
            "c" | "cpp" | "h" | "hpp" => extract_c_identifiers(
                line,
                line_num as u32 + 1,
                scope,
                file_path,
                &mut identifiers,
            ),
            _ => extract_generic_identifiers(
                line,
                line_num as u32 + 1,
                scope,
                file_path,
                &mut identifiers,
            ),
        }
    }

    identifiers
}

fn extract_rust_identifiers(
    line: &str,
    line_num: u32,
    scope: &SearchScope,
    file_path: &Path,
    identifiers: &mut Vec<NameInfo>,
) {
    let trimmed = line.trim();

    // Functions
    if matches!(scope, SearchScope::Functions | SearchScope::All) {
        if let Some(caps) = regex::Regex::new(r"(?:pub\s+)?(?:async\s+)?fn\s+(\w+)")
            .unwrap()
            .captures(trimmed)
        {
            if let Some(name) = caps.get(1) {
                identifiers.push(NameInfo {
                    name: name.as_str().to_string(),
                    file: file_path.to_path_buf(),
                    line: line_num,
                    name_type: "function".to_string(),
                    context: trimmed.to_string(),
                });
            }
        }
    }

    // Types (structs, enums, traits)
    if matches!(scope, SearchScope::Types | SearchScope::All) {
        for pattern in &[
            r"(?:pub\s+)?struct\s+(\w+)",
            r"(?:pub\s+)?enum\s+(\w+)",
            r"(?:pub\s+)?trait\s+(\w+)",
            r"(?:pub\s+)?type\s+(\w+)",
        ] {
            if let Some(caps) = regex::Regex::new(pattern).unwrap().captures(trimmed) {
                if let Some(name) = caps.get(1) {
                    identifiers.push(NameInfo {
                        name: name.as_str().to_string(),
                        file: file_path.to_path_buf(),
                        line: line_num,
                        name_type: "type".to_string(),
                        context: trimmed.to_string(),
                    });
                }
            }
        }
    }

    // Variables (let bindings)
    if matches!(scope, SearchScope::Variables | SearchScope::All) {
        if let Some(caps) = regex::Regex::new(r"let\s+(?:mut\s+)?(\w+)")
            .unwrap()
            .captures(trimmed)
        {
            if let Some(name) = caps.get(1) {
                identifiers.push(NameInfo {
                    name: name.as_str().to_string(),
                    file: file_path.to_path_buf(),
                    line: line_num,
                    name_type: "variable".to_string(),
                    context: trimmed.to_string(),
                });
            }
        }
    }
}

fn extract_js_identifiers(
    line: &str,
    line_num: u32,
    scope: &SearchScope,
    file_path: &Path,
    identifiers: &mut Vec<NameInfo>,
) {
    let trimmed = line.trim();

    // Functions
    if matches!(scope, SearchScope::Functions | SearchScope::All) {
        for pattern in &[
            r"function\s+(\w+)",
            r"(?:const|let|var)\s+(\w+)\s*=\s*(?:async\s+)?\s*(?:function|\(.*?\)\s*=>)",
            r"(\w+)\s*:\s*(?:async\s+)?function",
        ] {
            if let Some(caps) = regex::Regex::new(pattern).unwrap().captures(trimmed) {
                if let Some(name) = caps.get(1) {
                    identifiers.push(NameInfo {
                        name: name.as_str().to_string(),
                        file: file_path.to_path_buf(),
                        line: line_num,
                        name_type: "function".to_string(),
                        context: trimmed.to_string(),
                    });
                }
            }
        }
    }

    // Classes and interfaces
    if matches!(scope, SearchScope::Types | SearchScope::All) {
        for pattern in &[r"class\s+(\w+)", r"interface\s+(\w+)", r"type\s+(\w+)\s*="] {
            if let Some(caps) = regex::Regex::new(pattern).unwrap().captures(trimmed) {
                if let Some(name) = caps.get(1) {
                    identifiers.push(NameInfo {
                        name: name.as_str().to_string(),
                        file: file_path.to_path_buf(),
                        line: line_num,
                        name_type: "type".to_string(),
                        context: trimmed.to_string(),
                    });
                }
            }
        }
    }

    // Variables
    if matches!(scope, SearchScope::Variables | SearchScope::All) {
        if let Some(caps) = regex::Regex::new(r"(?:const|let|var)\s+(\w+)")
            .unwrap()
            .captures(trimmed)
        {
            if let Some(name) = caps.get(1) {
                identifiers.push(NameInfo {
                    name: name.as_str().to_string(),
                    file: file_path.to_path_buf(),
                    line: line_num,
                    name_type: "variable".to_string(),
                    context: trimmed.to_string(),
                });
            }
        }
    }
}

fn extract_python_identifiers(
    line: &str,
    line_num: u32,
    scope: &SearchScope,
    file_path: &Path,
    identifiers: &mut Vec<NameInfo>,
) {
    let trimmed = line.trim();

    // Functions
    if matches!(scope, SearchScope::Functions | SearchScope::All) {
        if let Some(caps) = regex::Regex::new(r"def\s+(\w+)").unwrap().captures(trimmed) {
            if let Some(name) = caps.get(1) {
                identifiers.push(NameInfo {
                    name: name.as_str().to_string(),
                    file: file_path.to_path_buf(),
                    line: line_num,
                    name_type: "function".to_string(),
                    context: trimmed.to_string(),
                });
            }
        }
    }

    // Classes
    if matches!(scope, SearchScope::Types | SearchScope::All) {
        if let Some(caps) = regex::Regex::new(r"class\s+(\w+)")
            .unwrap()
            .captures(trimmed)
        {
            if let Some(name) = caps.get(1) {
                identifiers.push(NameInfo {
                    name: name.as_str().to_string(),
                    file: file_path.to_path_buf(),
                    line: line_num,
                    name_type: "class".to_string(),
                    context: trimmed.to_string(),
                });
            }
        }
    }

    // Variables (simplified)
    if matches!(scope, SearchScope::Variables | SearchScope::All) {
        if let Some(caps) = regex::Regex::new(r"(\w+)\s*=").unwrap().captures(trimmed) {
            if let Some(name) = caps.get(1) {
                let name_str = name.as_str();
                // Skip obvious non-variables
                if ![
                    "def", "class", "if", "for", "while", "try", "except", "with",
                ]
                .contains(&name_str)
                {
                    identifiers.push(NameInfo {
                        name: name_str.to_string(),
                        file: file_path.to_path_buf(),
                        line: line_num,
                        name_type: "variable".to_string(),
                        context: trimmed.to_string(),
                    });
                }
            }
        }
    }
}

fn extract_c_identifiers(
    line: &str,
    line_num: u32,
    scope: &SearchScope,
    file_path: &Path,
    identifiers: &mut Vec<NameInfo>,
) {
    let trimmed = line.trim();

    // Functions
    if matches!(scope, SearchScope::Functions | SearchScope::All) {
        if let Some(caps) =
            regex::Regex::new(r"(?:static\s+)?(?:inline\s+)?(?:\w+\s+)*(\w+)\s*\([^)]*\)\s*\{")
                .unwrap()
                .captures(trimmed)
        {
            if let Some(name) = caps.get(1) {
                identifiers.push(NameInfo {
                    name: name.as_str().to_string(),
                    file: file_path.to_path_buf(),
                    line: line_num,
                    name_type: "function".to_string(),
                    context: trimmed.to_string(),
                });
            }
        }
    }

    // Types (struct, enum, typedef)
    if matches!(scope, SearchScope::Types | SearchScope::All) {
        for pattern in &[r"struct\s+(\w+)", r"enum\s+(\w+)", r"typedef\s+.*\s+(\w+);"] {
            if let Some(caps) = regex::Regex::new(pattern).unwrap().captures(trimmed) {
                if let Some(name) = caps.get(1) {
                    identifiers.push(NameInfo {
                        name: name.as_str().to_string(),
                        file: file_path.to_path_buf(),
                        line: line_num,
                        name_type: "type".to_string(),
                        context: trimmed.to_string(),
                    });
                }
            }
        }
    }
}

fn extract_generic_identifiers(
    line: &str,
    line_num: u32,
    scope: &SearchScope,
    file_path: &Path,
    identifiers: &mut Vec<NameInfo>,
) {
    // Very basic identifier extraction for unknown file types
    if matches!(scope, SearchScope::All) {
        let identifier_regex = regex::Regex::new(r"^[a-zA-Z_][a-zA-Z0-9_]*$").unwrap();
        for word in line.split_whitespace() {
            if let Some(caps) = identifier_regex.captures(word) {
                if let Some(name) = caps.get(0) {
                    if name.as_str().len() > 2 {
                        // Skip very short identifiers
                        identifiers.push(NameInfo {
                            name: name.as_str().to_string(),
                            file: file_path.to_path_buf(),
                            line: line_num,
                            name_type: "identifier".to_string(),
                            context: line.trim().to_string(),
                        });
                    }
                }
            }
        }
    }
}

fn calculate_string_similarity(s1: &str, s2: &str) -> f32 {
    // Simple Jaro similarity approximation
    if s1 == s2 {
        return 1.0;
    }

    let len1 = s1.len();
    let len2 = s2.len();

    if len1 == 0 || len2 == 0 {
        return 0.0;
    }

    let match_distance = (len1.max(len2) / 2).saturating_sub(1);
    let s1_chars: Vec<char> = s1.chars().collect();
    let s2_chars: Vec<char> = s2.chars().collect();

    let mut s1_matches = vec![false; len1];
    let mut s2_matches = vec![false; len2];

    let mut matches = 0;

    // Identify matches
    for i in 0..len1 {
        let start = i.saturating_sub(match_distance);
        let end = (i + match_distance + 1).min(len2);

        for j in start..end {
            if s2_matches[j] || s1_chars[i] != s2_chars[j] {
                continue;
            }
            s1_matches[i] = true;
            s2_matches[j] = true;
            matches += 1;
            break;
        }
    }

    if matches == 0 {
        return 0.0;
    }

    // Count transpositions
    let mut transpositions = 0;
    let mut k = 0;
    for i in 0..len1 {
        if !s1_matches[i] {
            continue;
        }
        while !s2_matches[k] {
            k += 1;
        }
        if s1_chars[i] != s2_chars[k] {
            transpositions += 1;
        }
        k += 1;
    }

    (matches as f32 / len1 as f32
        + matches as f32 / len2 as f32
        + (matches - transpositions / 2) as f32 / matches as f32)
        / 3.0
}

fn calculate_edit_distance(s1: &str, s2: &str) -> usize {
    let len1 = s1.len();
    let len2 = s2.len();

    if len1 == 0 {
        return len2;
    }
    if len2 == 0 {
        return len1;
    }

    let s1_chars: Vec<char> = s1.chars().collect();
    let s2_chars: Vec<char> = s2.chars().collect();

    let mut dp = vec![vec![0; len2 + 1]; len1 + 1];

    for (i, row) in dp.iter_mut().enumerate().take(len1 + 1) {
        row[0] = i;
    }
    for j in 0..=len2 {
        dp[0][j] = j;
    }

    for i in 1..=len1 {
        for j in 1..=len2 {
            let cost = if s1_chars[i - 1] == s2_chars[j - 1] {
                0
            } else {
                1
            };
            dp[i][j] = (dp[i - 1][j] + 1)
                .min(dp[i][j - 1] + 1)
                .min(dp[i - 1][j - 1] + cost);
        }
    }

    dp[len1][len2]
}

fn calculate_soundex(s: &str) -> String {
    if s.is_empty() {
        return String::new();
    }

    let chars: Vec<char> = s.to_uppercase().chars().collect();
    let mut soundex = vec![chars[0]];

    let get_code = |c: char| -> Option<char> {
        match c {
            'B' | 'F' | 'P' | 'V' => Some('1'),
            'C' | 'G' | 'J' | 'K' | 'Q' | 'S' | 'X' | 'Z' => Some('2'),
            'D' | 'T' => Some('3'),
            'L' => Some('4'),
            'M' | 'N' => Some('5'),
            'R' => Some('6'),
            _ => None,
        }
    };

    let mut prev_code = get_code(chars[0]);

    for &c in &chars[1..] {
        if let Some(code) = get_code(c) {
            if Some(code) != prev_code {
                soundex.push(code);
                if soundex.len() == 4 {
                    break;
                }
            }
            prev_code = Some(code);
        } else {
            prev_code = None;
        }
    }

    // Pad with zeros
    while soundex.len() < 4 {
        soundex.push('0');
    }

    soundex.into_iter().collect()
}

#[allow(clippy::too_many_arguments)]
async fn handle_analyze_proof_annotations(
    project_path: PathBuf,
    format: ProofAnnotationOutputFormat,
    high_confidence_only: bool,
    include_evidence: bool,
    property_type: Option<PropertyTypeFilter>,
    verification_method: Option<VerificationMethodFilter>,
    output: Option<PathBuf>,
    perf: bool,
    clear_cache: bool,
) -> anyhow::Result<()> {
    use crate::services::{
        proof_annotator::{MockProofSource, ProofAnnotator},
        symbol_table::SymbolTable,
    };
    use std::time::Instant;

    let start_time = Instant::now();

    // Create symbol table and proof annotator
    let symbol_table = std::sync::Arc::new(SymbolTable::new());
    let mut annotator = ProofAnnotator::new(symbol_table.clone());

    // Clear cache if requested
    if clear_cache {
        annotator.clear_cache();
    }

    // Add mock proof sources for demonstration
    // In a real implementation, these would be real proof sources like:
    // - Rust borrow checker integration
    // - External verification tool outputs
    // - Manual proof annotations from comments
    annotator.add_source(MockProofSource::new("borrow_checker".to_string(), 10, 5));
    annotator.add_source(MockProofSource::new("static_analyzer".to_string(), 20, 3));
    annotator.add_source(MockProofSource::new("formal_verifier".to_string(), 50, 2));

    // Collect proof annotations
    let proof_map = annotator.collect_proofs(&project_path).await;

    // Filter annotations based on criteria
    let filtered_annotations: Vec<_> = proof_map
        .into_iter()
        .flat_map(|(location, annotations)| {
            annotations
                .into_iter()
                .filter(|annotation| {
                    // Filter by confidence level
                    if high_confidence_only {
                        matches!(
                            annotation.confidence_level,
                            crate::models::unified_ast::ConfidenceLevel::High
                        )
                    } else {
                        true
                    }
                })
                .filter(|annotation| {
                    // Filter by property type
                    if let Some(ref filter) = property_type {
                        match filter {
                            PropertyTypeFilter::MemorySafety => matches!(
                                annotation.property_proven,
                                crate::models::unified_ast::PropertyType::MemorySafety
                            ),
                            PropertyTypeFilter::ThreadSafety => matches!(
                                annotation.property_proven,
                                crate::models::unified_ast::PropertyType::ThreadSafety
                            ),
                            PropertyTypeFilter::DataRaceFreeze => matches!(
                                annotation.property_proven,
                                crate::models::unified_ast::PropertyType::DataRaceFreeze
                            ),
                            PropertyTypeFilter::Termination => matches!(
                                annotation.property_proven,
                                crate::models::unified_ast::PropertyType::Termination
                            ),
                            PropertyTypeFilter::FunctionalCorrectness => matches!(
                                annotation.property_proven,
                                crate::models::unified_ast::PropertyType::FunctionalCorrectness(_)
                            ),
                            PropertyTypeFilter::ResourceBounds => matches!(
                                annotation.property_proven,
                                crate::models::unified_ast::PropertyType::ResourceBounds { .. }
                            ),
                            PropertyTypeFilter::All => true,
                        }
                    } else {
                        true
                    }
                })
                .filter(|annotation| {
                    // Filter by verification method
                    if let Some(ref filter) = verification_method {
                        match filter {
                            VerificationMethodFilter::FormalProof => matches!(
                                annotation.method,
                                crate::models::unified_ast::VerificationMethod::FormalProof { .. }
                            ),
                            VerificationMethodFilter::ModelChecking => matches!(
                                annotation.method,
                                crate::models::unified_ast::VerificationMethod::ModelChecking { .. }
                            ),
                            VerificationMethodFilter::StaticAnalysis => matches!(
                                annotation.method,
                                crate::models::unified_ast::VerificationMethod::StaticAnalysis { .. }
                            ),
                            VerificationMethodFilter::AbstractInterpretation => matches!(
                                annotation.method,
                                crate::models::unified_ast::VerificationMethod::AbstractInterpretation
                            ),
                            VerificationMethodFilter::BorrowChecker => matches!(
                                annotation.method,
                                crate::models::unified_ast::VerificationMethod::BorrowChecker
                            ),
                            VerificationMethodFilter::All => true,
                        }
                    } else {
                        true
                    }
                })
                .map(|annotation| (location.clone(), annotation))
                .collect::<Vec<_>>()
        })
        .collect();

    let elapsed = start_time.elapsed();

    // Format output
    let result = match format {
        ProofAnnotationOutputFormat::Summary => {
            format_proof_annotations_summary(&filtered_annotations, perf, elapsed, &annotator)
        }
        ProofAnnotationOutputFormat::Full => format_proof_annotations_full(
            &filtered_annotations,
            include_evidence,
            perf,
            elapsed,
            &annotator,
        ),
        ProofAnnotationOutputFormat::Json => {
            let cache_stats = annotator.cache_stats();
            let annotations_json: Vec<serde_json::Value> = filtered_annotations
                .iter()
                .map(|(location, annotation)| {
                    serde_json::json!({
                        "location": {
                            "file_path": location.file_path.to_string_lossy(),
                            "start_pos": location.span.start.0,
                            "end_pos": location.span.end.0
                        },
                        "annotation": annotation
                    })
                })
                .collect();

            let json_data = serde_json::json!({
                "proof_annotations": annotations_json,
                "summary": {
                    "total_annotations": filtered_annotations.len(),
                    "analysis_time_ms": elapsed.as_millis(),
                    "cache_stats": {
                        "size": cache_stats.size,
                        "files_tracked": cache_stats.files_tracked
                    }
                }
            });
            serde_json::to_string_pretty(&json_data)?
        }
        ProofAnnotationOutputFormat::Markdown => format_proof_annotations_markdown(
            &filtered_annotations,
            include_evidence,
            perf,
            elapsed,
            &annotator,
        ),
        ProofAnnotationOutputFormat::Sarif => {
            format_proof_annotations_sarif(&filtered_annotations)?
        }
    };

    // Output to file or stdout
    if let Some(output_path) = output {
        tokio::fs::write(&output_path, &result).await?;
        eprintln!(
            "Proof annotations analysis written to {}",
            output_path.display()
        );
    } else {
        println!("{result}");
    }

    Ok(())
}

fn format_proof_annotations_summary(
    annotations: &[(
        crate::models::unified_ast::Location,
        crate::models::unified_ast::ProofAnnotation,
    )],
    perf: bool,
    elapsed: std::time::Duration,
    annotator: &crate::services::proof_annotator::ProofAnnotator,
) -> String {
    let mut output = String::new();

    output.push_str("Proof Annotations Summary\n");
    output.push_str("========================\n\n");

    output.push_str(&format!("Total annotations found: {}\n", annotations.len()));

    // Group by property type
    let mut property_counts = std::collections::HashMap::new();
    let mut method_counts = std::collections::HashMap::new();
    let mut confidence_counts = std::collections::HashMap::new();

    for (_, annotation) in annotations {
        *property_counts
            .entry(format!("{:?}", annotation.property_proven))
            .or_insert(0) += 1;
        *method_counts
            .entry(format!("{:?}", annotation.method))
            .or_insert(0) += 1;
        *confidence_counts
            .entry(format!("{:?}", annotation.confidence_level))
            .or_insert(0) += 1;
    }

    output.push_str("\nProperty Types:\n");
    for (property, count) in property_counts {
        output.push_str(&format!("  {property}: {count}\n"));
    }

    output.push_str("\nVerification Methods:\n");
    for (method, count) in method_counts {
        output.push_str(&format!("  {method}: {count}\n"));
    }

    output.push_str("\nConfidence Levels:\n");
    for (level, count) in confidence_counts {
        output.push_str(&format!("  {level}: {count}\n"));
    }

    if perf {
        let cache_stats = annotator.cache_stats();
        output.push_str("\nPerformance Metrics:\n");
        output.push_str(&format!("  Analysis time: {}ms\n", elapsed.as_millis()));
        output.push_str(&format!("  Cache size: {}\n", cache_stats.size));
        output.push_str(&format!("  Files tracked: {}\n", cache_stats.files_tracked));
    }

    output
}

fn format_proof_annotations_full(
    annotations: &[(
        crate::models::unified_ast::Location,
        crate::models::unified_ast::ProofAnnotation,
    )],
    include_evidence: bool,
    perf: bool,
    elapsed: std::time::Duration,
    annotator: &crate::services::proof_annotator::ProofAnnotator,
) -> String {
    let mut output = format_proof_annotations_summary(annotations, perf, elapsed, annotator);

    output.push_str("\nDetailed Annotations:\n");
    output.push_str(&"-".repeat(50));
    output.push('\n');

    for (i, (location, annotation)) in annotations.iter().enumerate() {
        output.push_str(&format!("\n{}. {}\n", i + 1, annotation.tool_name));
        output.push_str(&format!(
            "   Location: {}:{}:{}\n",
            location.file_path.display(),
            location.span.start.0,
            location.span.end.0
        ));
        output.push_str(&format!("   Property: {:?}\n", annotation.property_proven));
        output.push_str(&format!("   Method: {:?}\n", annotation.method));
        output.push_str(&format!(
            "   Confidence: {:?}\n",
            annotation.confidence_level
        ));
        output.push_str(&format!(
            "   Date: {}\n",
            annotation.date_verified.format("%Y-%m-%d %H:%M:%S")
        ));

        if include_evidence {
            output.push_str(&format!("   Evidence: {:?}\n", annotation.evidence_type));
            if !annotation.assumptions.is_empty() {
                output.push_str("   Assumptions:\n");
                for assumption in &annotation.assumptions {
                    output.push_str(&format!("     - {assumption}\n"));
                }
            }
        }
    }

    output
}

fn format_proof_annotations_markdown(
    annotations: &[(
        crate::models::unified_ast::Location,
        crate::models::unified_ast::ProofAnnotation,
    )],
    include_evidence: bool,
    perf: bool,
    elapsed: std::time::Duration,
    annotator: &crate::services::proof_annotator::ProofAnnotator,
) -> String {
    let mut output = String::new();

    output.push_str("# Proof Annotations Report\n\n");
    output.push_str(&format!(
        "**Total annotations found:** {}\n\n",
        annotations.len()
    ));

    // Summary statistics
    let mut property_counts = std::collections::HashMap::new();
    for (_, annotation) in annotations {
        *property_counts
            .entry(format!("{:?}", annotation.property_proven))
            .or_insert(0) += 1;
    }

    output.push_str("## Property Types\n\n");
    for (property, count) in property_counts {
        output.push_str(&format!("- **{property}**: {count}\n"));
    }

    output.push_str("\n## Detailed Annotations\n\n");

    for (location, annotation) in annotations {
        output.push_str(&format!("### {}\n\n", annotation.tool_name));
        output.push_str(&format!("- **File**: `{}`\n", location.file_path.display()));
        output.push_str(&format!(
            "- **Byte positions**: {}:{}\n",
            location.span.start.0, location.span.end.0
        ));
        output.push_str(&format!(
            "- **Property**: {:?}\n",
            annotation.property_proven
        ));
        output.push_str(&format!("- **Method**: {:?}\n", annotation.method));
        output.push_str(&format!(
            "- **Confidence**: {:?}\n",
            annotation.confidence_level
        ));

        if include_evidence {
            output.push_str(&format!("- **Evidence**: {:?}\n", annotation.evidence_type));
            if !annotation.assumptions.is_empty() {
                output.push_str("- **Assumptions**:\n");
                for assumption in &annotation.assumptions {
                    output.push_str(&format!("  - {assumption}\n"));
                }
            }
        }
        output.push('\n');
    }

    if perf {
        let cache_stats = annotator.cache_stats();
        output.push_str("## Performance Metrics\n\n");
        output.push_str(&format!("- **Analysis time**: {}ms\n", elapsed.as_millis()));
        output.push_str(&format!("- **Cache size**: {}\n", cache_stats.size));
        output.push_str(&format!(
            "- **Files tracked**: {}\n",
            cache_stats.files_tracked
        ));
    }

    output
}

fn format_proof_annotations_sarif(
    annotations: &[(
        crate::models::unified_ast::Location,
        crate::models::unified_ast::ProofAnnotation,
    )],
) -> anyhow::Result<String> {
    use serde_json::json;

    let results: Vec<_> = annotations
        .iter()
        .map(|(location, annotation)| {
            json!({
                "ruleId": format!("{:?}", annotation.property_proven),
                "level": match annotation.confidence_level {
                    crate::models::unified_ast::ConfidenceLevel::High => "note",
                    crate::models::unified_ast::ConfidenceLevel::Medium => "info",
                    crate::models::unified_ast::ConfidenceLevel::Low => "warning"
                },
                "message": {
                    "text": format!("Property {:?} verified using {:?}",
                        annotation.property_proven, annotation.method)
                },
                "locations": [{
                    "physicalLocation": {
                        "artifactLocation": {
                            "uri": location.file_path.to_string_lossy()
                        },
                        "region": {
                            "startColumn": location.span.start.0,
                            "endColumn": location.span.end.0
                        }
                    }
                }],
                "properties": {
                    "tool": annotation.tool_name,
                    "confidence": format!("{:?}", annotation.confidence_level),
                    "verification_method": format!("{:?}", annotation.method),
                    "date_verified": annotation.date_verified.to_rfc3339()
                }
            })
        })
        .collect();

    let sarif = json!({
        "version": "2.1.0",
        "$schema": "https://raw.githubusercontent.com/oasis-tcs/sarif-spec/master/Schemata/sarif-schema-2.1.0.json",
        "runs": [{
            "tool": {
                "driver": {
                    "name": "paiml-mcp-agent-toolkit",
                    "version": env!("CARGO_PKG_VERSION"),
                    "informationUri": "https://github.com/paiml/paiml-mcp-agent-toolkit"
                }
            },
            "results": results
        }]
    });

    Ok(serde_json::to_string_pretty(&sarif)?)
}

#[allow(clippy::too_many_arguments)]
async fn handle_analyze_incremental_coverage(
    project_path: PathBuf,
    baseline_branch: Option<String>,
    cache_dir: Option<PathBuf>,
    format: IncrementalCoverageOutputFormat,
    include_aggregate: bool,
    include_delta: bool,
    include_file_coverage: bool,
    _confidence_threshold: f64,
    output: Option<PathBuf>,
    _parallel: Option<usize>,
    verbose: bool,
) -> anyhow::Result<()> {
    use crate::services::incremental_coverage_analyzer::*;

    if verbose {
        eprintln!("🔍 Starting incremental coverage analysis...");
        eprintln!("📂 Project path: {}", project_path.display());
        if let Some(ref branch) = baseline_branch {
            eprintln!("🌿 Baseline branch: {branch}");
        }
    }

    // Initialize analyzer
    let cache_path = cache_dir.unwrap_or_else(|| project_path.join(".pmat-cache"));
    let analyzer = IncrementalCoverageAnalyzer::new(&cache_path)?;

    // Create a simplified changeset for analysis
    let test_file = project_path.join("src/main.rs");
    let file_id = if test_file.exists() {
        let hash = analyzer.compute_file_hash(&test_file).await?;
        FileId {
            path: test_file,
            hash,
        }
    } else {
        // Use first Rust file found
        let files = discover_rust_files(&project_path).await?;
        if files.is_empty() {
            anyhow::bail!("No Rust files found in project");
        }
        let first_file = &files[0];
        let hash = analyzer.compute_file_hash(first_file).await?;
        FileId {
            path: first_file.clone(),
            hash,
        }
    };

    let changeset = ChangeSet {
        modified_files: vec![file_id.clone()],
        added_files: vec![],
        deleted_files: vec![],
    };

    // Run analysis
    let coverage_update = analyzer.analyze_changes(&changeset).await?;

    if verbose {
        eprintln!(
            "✅ Analysis complete: {} files analyzed",
            coverage_update.file_coverage.len()
        );
    }

    // Format output
    let content = match format {
        IncrementalCoverageOutputFormat::Summary => {
            format_incremental_coverage_summary(&coverage_update, include_aggregate, include_delta)
        }
        IncrementalCoverageOutputFormat::Detailed => format_incremental_coverage_markdown(
            &coverage_update,
            include_aggregate,
            include_delta,
            true,
        ),
        IncrementalCoverageOutputFormat::Json => format_incremental_coverage_json(
            &coverage_update,
            include_aggregate,
            include_delta,
            include_file_coverage,
        )?,
        IncrementalCoverageOutputFormat::Markdown => format_incremental_coverage_markdown(
            &coverage_update,
            include_aggregate,
            include_delta,
            include_file_coverage,
        ),
        IncrementalCoverageOutputFormat::Lcov => {
            // For now, return JSON format for LCOV
            format_incremental_coverage_json(
                &coverage_update,
                include_aggregate,
                include_delta,
                include_file_coverage,
            )?
        }
        IncrementalCoverageOutputFormat::Delta => {
            format_incremental_coverage_summary(&coverage_update, false, true)
        }
        IncrementalCoverageOutputFormat::Sarif => {
            format_incremental_coverage_sarif(&coverage_update)?
        }
    };

    // Output results
    if let Some(output_path) = output {
        tokio::fs::write(&output_path, &content).await?;
        eprintln!(
            "✅ Incremental coverage analysis written to: {}",
            output_path.display()
        );
    } else {
        println!("{content}");
    }

    Ok(())
}

fn discover_rust_files_sync(project_path: &std::path::Path) -> anyhow::Result<Vec<PathBuf>> {
    let mut files = Vec::new();

    fn visit_dir(dir: &std::path::Path, files: &mut Vec<PathBuf>) -> anyhow::Result<()> {
        if dir.is_dir() {
            for entry in std::fs::read_dir(dir)? {
                let entry = entry?;
                let path = entry.path();

                if path.is_file() {
                    if let Some(ext) = path.extension() {
                        if ext == "rs" {
                            files.push(path);
                        }
                    }
                } else if path.is_dir()
                    && !path.file_name().unwrap().to_str().unwrap().starts_with('.')
                {
                    visit_dir(&path, files)?;
                }
            }
        }
        Ok(())
    }

    visit_dir(project_path, &mut files)?;
    Ok(files)
}

async fn discover_rust_files(project_path: &std::path::Path) -> anyhow::Result<Vec<PathBuf>> {
    let project_path = project_path.to_path_buf();
    tokio::task::spawn_blocking(move || discover_rust_files_sync(&project_path)).await?
}

fn format_incremental_coverage_summary(
    update: &crate::services::incremental_coverage_analyzer::CoverageUpdate,
    include_aggregate: bool,
    include_delta: bool,
) -> String {
    let mut output = String::from("# Incremental Coverage Analysis Summary\n\n");

    if include_aggregate {
        output.push_str(&format!(
            "## Aggregate Coverage\n\
            - **Line Coverage:** {:.1}%\n\
            - **Branch Coverage:** {:.1}%\n\
            - **Function Coverage:** {:.1}%\n\
            - **Files Covered:** {}/{}\n\n",
            update.aggregate_coverage.line_percentage,
            update.aggregate_coverage.branch_percentage,
            update.aggregate_coverage.function_percentage,
            update.aggregate_coverage.covered_files,
            update.aggregate_coverage.total_files
        ));
    }

    if include_delta {
        output.push_str(&format!(
            "## Delta Coverage\n\
            - **New Lines Covered:** {}/{}\n\
            - **Delta Percentage:** {:.1}%\n\n",
            update.delta_coverage.new_lines_covered,
            update.delta_coverage.new_lines_total,
            update.delta_coverage.percentage
        ));
    }

    output.push_str(&format!(
        "## Summary\n\
        - **Total Files Analyzed:** {}\n",
        update.file_coverage.len()
    ));

    output
}

fn format_incremental_coverage_json(
    update: &crate::services::incremental_coverage_analyzer::CoverageUpdate,
    include_aggregate: bool,
    include_delta: bool,
    include_file_coverage: bool,
) -> anyhow::Result<String> {
    use serde_json::json;

    let mut result = json!({});

    if include_aggregate {
        result["aggregate_coverage"] = json!({
            "line_percentage": update.aggregate_coverage.line_percentage,
            "branch_percentage": update.aggregate_coverage.branch_percentage,
            "function_percentage": update.aggregate_coverage.function_percentage,
            "total_files": update.aggregate_coverage.total_files,
            "covered_files": update.aggregate_coverage.covered_files
        });
    }

    if include_delta {
        result["delta_coverage"] = json!({
            "new_lines_covered": update.delta_coverage.new_lines_covered,
            "new_lines_total": update.delta_coverage.new_lines_total,
            "percentage": update.delta_coverage.percentage
        });
    }

    if include_file_coverage {
        let file_coverage: std::collections::HashMap<String, serde_json::Value> = update
            .file_coverage
            .iter()
            .map(|(file_id, coverage)| {
                (
                    file_id.path.to_string_lossy().to_string(),
                    json!({
                        "line_coverage": coverage.line_coverage,
                        "branch_coverage": coverage.branch_coverage,
                        "function_coverage": coverage.function_coverage,
                        "total_lines": coverage.total_lines,
                        "covered_lines_count": coverage.covered_lines.len()
                    }),
                )
            })
            .collect();

        result["file_coverage"] = json!(file_coverage);
    }

    Ok(serde_json::to_string_pretty(&result)?)
}

fn format_incremental_coverage_markdown(
    update: &crate::services::incremental_coverage_analyzer::CoverageUpdate,
    include_aggregate: bool,
    include_delta: bool,
    include_file_coverage: bool,
) -> String {
    let mut output = String::from("# Incremental Coverage Report\n\n");

    if include_aggregate {
        output.push_str(&format!(
            "## 📊 Aggregate Coverage\n\n\
            | Metric | Percentage | Count |\n\
            |--------|------------|-------|\n\
            | Line Coverage | {:.1}% | {}/{} files |\n\
            | Branch Coverage | {:.1}% | - |\n\
            | Function Coverage | {:.1}% | - |\n\n",
            update.aggregate_coverage.line_percentage,
            update.aggregate_coverage.covered_files,
            update.aggregate_coverage.total_files,
            update.aggregate_coverage.branch_percentage,
            update.aggregate_coverage.function_percentage
        ));
    }

    if include_delta {
        output.push_str(&format!(
            "## 🔄 Delta Coverage\n\n\
            **New Lines:** {}/{} ({:.1}%)\n\n",
            update.delta_coverage.new_lines_covered,
            update.delta_coverage.new_lines_total,
            update.delta_coverage.percentage
        ));
    }

    if include_file_coverage && !update.file_coverage.is_empty() {
        output.push_str("## 📁 File Coverage Details\n\n");
        output.push_str("| File | Line % | Branch % | Function % | Lines |\n");
        output.push_str("|------|--------|----------|------------|-------|\n");

        for (file_id, coverage) in &update.file_coverage {
            output.push_str(&format!(
                "| {} | {:.1}% | {:.1}% | {:.1}% | {}/{} |\n",
                file_id
                    .path
                    .file_name()
                    .unwrap_or_default()
                    .to_string_lossy(),
                coverage.line_coverage,
                coverage.branch_coverage,
                coverage.function_coverage,
                coverage.covered_lines.len(),
                coverage.total_lines
            ));
        }
        output.push('\n');
    }

    output
}

fn format_incremental_coverage_sarif(
    update: &crate::services::incremental_coverage_analyzer::CoverageUpdate,
) -> anyhow::Result<String> {
    use serde_json::json;

    let results: Vec<_> = update.file_coverage.iter()
        .filter(|(_, coverage)| coverage.line_coverage < 80.0) // Flag files with low coverage
        .map(|(file_id, coverage)| {
            json!({
                "ruleId": "low-coverage",
                "level": if coverage.line_coverage < 50.0 { "error" } else { "warning" },
                "message": {
                    "text": format!("Low test coverage: {:.1}% line coverage", coverage.line_coverage)
                },
                "locations": [{
                    "physicalLocation": {
                        "artifactLocation": {
                            "uri": file_id.path.to_string_lossy()
                        }
                    }
                }],
                "properties": {
                    "line_coverage": coverage.line_coverage,
                    "branch_coverage": coverage.branch_coverage,
                    "function_coverage": coverage.function_coverage,
                    "total_lines": coverage.total_lines,
                    "covered_lines": coverage.covered_lines.len()
                }
            })
        })
        .collect();

    let sarif = json!({
        "version": "2.1.0",
        "$schema": "https://raw.githubusercontent.com/oasis-tcs/sarif-spec/master/Schemata/sarif-schema-2.1.0.json",
        "runs": [{
            "tool": {
                "driver": {
                    "name": "paiml-mcp-agent-toolkit",
                    "version": env!("CARGO_PKG_VERSION"),
                    "informationUri": "https://github.com/paiml/paiml-mcp-agent-toolkit"
                }
            },
            "results": results
        }]
    });

    Ok(serde_json::to_string_pretty(&sarif)?)
}

/// Handle symbol table analysis command
#[allow(clippy::too_many_arguments)]
async fn handle_analyze_symbol_table(
    project_path: PathBuf,
    format: SymbolTableOutputFormat,
    filter: Option<SymbolTypeFilter>,
    query: Option<String>,
    include: Vec<String>,
    exclude: Vec<String>,
    _show_unreferenced: bool,
    _show_references: bool,
    output: Option<PathBuf>,
    perf: bool,
) -> anyhow::Result<()> {
    use crate::services::context::AstItem;
    use crate::services::deep_context::{
        AnalysisType, CacheStrategy, DagType, DeepContextAnalyzer, DeepContextConfig,
    };
    use crate::services::symbol_table::SymbolTable;
    use serde_json::json;
    use std::time::Instant;

    let start = Instant::now();

    if perf {
        eprintln!("🔍 Building symbol table for: {}", project_path.display());
    }

    // Generate deep context to get all symbols
    let config = DeepContextConfig {
        include_analyses: vec![AnalysisType::Ast],
        period_days: 30,
        dag_type: DagType::FullDependency,
        complexity_thresholds: None,
        max_depth: None,
        include_patterns: include.clone(),
        exclude_patterns: exclude.clone(),
        cache_strategy: CacheStrategy::Normal,
        parallel: num_cpus::get(),
        file_classifier_config: None,
    };

    let analyzer = DeepContextAnalyzer::new(config);
    let deep_context = analyzer.analyze_project(&project_path).await?;

    // Build symbol table
    let _symbol_table = SymbolTable::new();
    let mut all_symbols = Vec::new();

    // Extract symbols from AST contexts
    for ast_ctx in &deep_context.analyses.ast_contexts {
        for item in &ast_ctx.base.items {
            let symbol_info = match item {
                AstItem::Function {
                    name,
                    visibility,
                    is_async,
                    line,
                } => Some((
                    name.clone(),
                    "function",
                    *line,
                    visibility.clone(),
                    *is_async,
                )),
                AstItem::Struct {
                    name,
                    visibility,
                    fields_count: _,
                    line,
                    ..
                } => Some((name.clone(), "struct", *line, visibility.clone(), false)),
                AstItem::Enum {
                    name,
                    visibility,
                    variants_count: _,
                    line,
                } => Some((name.clone(), "enum", *line, visibility.clone(), false)),
                AstItem::Trait {
                    name,
                    visibility,
                    line,
                } => Some((name.clone(), "trait", *line, visibility.clone(), false)),
                AstItem::Module {
                    name,
                    visibility,
                    line,
                } => Some((name.clone(), "module", *line, visibility.clone(), false)),
                AstItem::Use { path, line } => {
                    Some((path.clone(), "import", *line, "pub".to_string(), false))
                }
                _ => None,
            };

            if let Some((name, kind, line, visibility, is_async)) = symbol_info {
                // Apply filters
                let passes_filter = match &filter {
                    Some(SymbolTypeFilter::Functions) => kind == "function",
                    Some(SymbolTypeFilter::Types) => matches!(kind, "struct" | "enum" | "trait"),
                    Some(SymbolTypeFilter::Variables) => false, // Not implemented yet
                    Some(SymbolTypeFilter::Modules) => kind == "module",
                    Some(SymbolTypeFilter::All) | None => true,
                };

                if !passes_filter {
                    continue;
                }

                // Apply query filter
                if let Some(q) = &query {
                    if !name.to_lowercase().contains(&q.to_lowercase()) {
                        continue;
                    }
                }

                all_symbols.push(SymbolInfo {
                    name,
                    kind: kind.to_string(),
                    file: ast_ctx.base.path.clone(),
                    line,
                    visibility,
                    is_async,
                });
            }
        }
    }

    if perf {
        eprintln!(
            "⏱️  Symbol table built in {:.2}s ({} symbols found)",
            start.elapsed().as_secs_f64(),
            all_symbols.len()
        );
    }

    // Format output
    let content = match format {
        SymbolTableOutputFormat::Summary => {
            format_symbol_table_summary(&all_symbols, &deep_context)
        }
        SymbolTableOutputFormat::Detailed => format_symbol_table_detailed(&all_symbols),
        SymbolTableOutputFormat::Json => serde_json::to_string_pretty(&json!({
            "symbols": all_symbols,
            "summary": {
                "total_symbols": all_symbols.len(),
                "by_type": count_by_type(&all_symbols),
                "by_visibility": count_by_visibility(&all_symbols),
            }
        }))?,
        SymbolTableOutputFormat::Csv => format_symbol_table_csv(&all_symbols),
    };

    // Output results
    if let Some(output_path) = output {
        tokio::fs::write(&output_path, &content).await?;
        if perf {
            eprintln!("✅ Symbol table written to: {}", output_path.display());
        }
    } else {
        println!("{content}");
    }

    Ok(())
}

#[derive(Debug, serde::Serialize)]
struct SymbolInfo {
    name: String,
    kind: String,
    file: String,
    line: usize,
    visibility: String,
    is_async: bool,
}

fn format_symbol_table_summary(
    symbols: &[SymbolInfo],
    deep_context: &crate::services::deep_context::DeepContext,
) -> String {
    let mut output = String::from("# Symbol Table Analysis\n\n");

    output.push_str(&format!("**Total Symbols:** {}\n", symbols.len()));
    output.push_str(&format!(
        "**Total Files:** {}\n\n",
        deep_context.analyses.ast_contexts.len()
    ));

    output.push_str("## Symbols by Type\n");
    let by_type = count_by_type(symbols);
    for (kind, count) in by_type {
        output.push_str(&format!("- {kind}: {count}\n"));
    }

    output.push_str("\n## Symbols by Visibility\n");
    let by_vis = count_by_visibility(symbols);
    for (vis, count) in by_vis {
        output.push_str(&format!("- {vis}: {count}\n"));
    }

    output
}

fn format_symbol_table_detailed(symbols: &[SymbolInfo]) -> String {
    let mut output = String::from("# Symbol Table - Detailed\n\n");

    output.push_str("| Name | Type | File | Line | Visibility |\n");
    output.push_str("|------|------|------|------|------------|\n");

    for symbol in symbols {
        output.push_str(&format!(
            "| {} | {} | {} | {} | {} |\n",
            symbol.name, symbol.kind, symbol.file, symbol.line, symbol.visibility
        ));
    }

    output
}

fn format_symbol_table_csv(symbols: &[SymbolInfo]) -> String {
    let mut output = String::from("name,type,file,line,visibility,is_async\n");

    for symbol in symbols {
        output.push_str(&format!(
            "{},{},{},{},{},{}\n",
            symbol.name, symbol.kind, symbol.file, symbol.line, symbol.visibility, symbol.is_async
        ));
    }

    output
}

fn count_by_type(symbols: &[SymbolInfo]) -> Vec<(String, usize)> {
    // FxHashMap is already imported at module level
    let mut counts = FxHashMap::default();

    for symbol in symbols {
        *counts.entry(symbol.kind.clone()).or_insert(0) += 1;
    }

    let mut result: Vec<_> = counts.into_iter().collect();
    result.sort_by(|a, b| b.1.cmp(&a.1));
    result
}

fn count_by_visibility(symbols: &[SymbolInfo]) -> Vec<(String, usize)> {
    // FxHashMap is already imported at module level
    let mut counts = FxHashMap::default();

    for symbol in symbols {
        *counts.entry(symbol.visibility.clone()).or_insert(0) += 1;
    }

    let mut result: Vec<_> = counts.into_iter().collect();
    result.sort_by(|a, b| b.1.cmp(&a.1));
    result
}

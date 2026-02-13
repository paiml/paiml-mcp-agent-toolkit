// Analyze commands - extracted for file health (CB-040)

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

        /// Use ML-based scoring (aprender LinearRegression)
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

        /// Use extended mode - detect euphemisms like 'placeholder', 'stub', 'for now'
        /// that hide technical debt (addresses issue #149)
        #[arg(long)]
        extended: bool,

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

        /// Use ML-based scoring (aprender LinearRegression)
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

    /// Cluster code by semantic similarity
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

    /// Extract semantic topics from codebase
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

    /// Analyze ML model files (GGUF, APR, SafeTensors)
    #[command(visible_aliases = &["model", "mlops"])]
    Models {
        /// Path to scan for model files
        #[arg(long, default_value = ".")]
        path: std::path::PathBuf,

        /// Output format (table, json)
        #[arg(long, value_enum, default_value = "table")]
        format: OutputFormat,

        /// Run compliance checks on model files
        #[arg(long)]
        check: bool,
    },
}

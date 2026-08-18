
/// Main command enum
//
// Note: `Query` variant is the largest (50+ fields covering enrichment flags,
// search modes, output formats). The clippy::large_enum_variant warning is
// allowed because the alternative (boxing fields) would degrade ergonomics
// across the entire dispatcher.
#[allow(clippy::large_enum_variant)]
#[derive(Subcommand)]
#[cfg_attr(test, derive(Debug))]
pub enum Commands {
    // ── Template commands ─────────────────────────────────────────────
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

    // ── Context & Query commands ────────────────────────────────────
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

        /// Ranking strategy: relevance, pagerank, centrality, indegree, impact, cross-project, priority
        #[arg(long, value_name = "STRATEGY", value_parser = parse_rank_by_value)]
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
        #[arg(long = "type", value_name = "TYPE", value_parser = parse_definition_type_value)]
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

        /// Search mode: semantic (default, auto-blended), lexical (case-insensitive
        /// substring/regex over name+signature+source, no embeddings), or hybrid
        /// (RRF k=60 over both). Issue #562: makes lexical-vs-semantic comparison
        /// teachable in one command without the `pmat semantic` config gate.
        #[arg(long, value_name = "MODE", value_parser = ["semantic", "lexical", "hybrid"])]
        search_mode: Option<String>,

        /// Raw file search only, skip index (like rg, searches all file types)
        #[arg(long)]
        raw: bool,

        /// Case-sensitive search (default: smart-case)
        #[arg(long)]
        case_sensitive: bool,

        /// Case-insensitive search (like rg -i)
        #[arg(short = 'i', long)]
        ignore_case: bool,

        /// Exclude results matching content patterns (like grep -v, repeatable)
        #[arg(long, value_name = "PATTERN")]
        exclude: Vec<String>,

        /// Exclude results from files matching this glob (like rg --glob '!PATTERN', repeatable)
        #[arg(long, value_name = "GLOB")]
        exclude_file: Vec<String>,

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

        /// Suggest semantic renames for _part_ files based on function index analysis
        #[arg(long)]
        suggest_rename: bool,

        /// Apply suggested renames (git mv + update parent declarations). Requires --suggest-rename
        #[arg(long, requires = "suggest_rename")]
        apply: bool,

        /// Disable document search results (documents are included by default)
        #[arg(long)]
        no_docs: bool,

        /// Search only documents, skip code index
        #[arg(long)]
        docs_only: bool,

        /// Scan functions for I/O patterns and suggest module extractions
        #[arg(long)]
        extract_candidates: bool,

        /// Maximum lines of code per suggested extraction module (used with --extract-candidates)
        #[arg(long, value_name = "LINES", default_value_t = 500)]
        max_module_lines: usize,

        /// Delegate to `pv query` for cross-project contract search
        #[arg(long, help = "Search provable-contracts YAML via pv query")]
        contracts: bool,

        /// Show functions WITHOUT contract bindings, ranked by importance
        #[arg(long, help = "Find undercontracted functions (no binding in binding-index.json)")]
        contract_gaps: bool,

        /// Filter: only functions at or above this verification level (L0-L5)
        #[arg(long, value_name = "LEVEL", help = "e.g. --min-level L3")]
        min_level: Option<String>,

        /// Filter: only functions at or below this verification level (L0-L5)
        #[arg(long, value_name = "LEVEL", help = "e.g. --max-level L1")]
        max_level: Option<String>,

        /// Sort results by contract quality score (requires binding-index.json)
        #[arg(long)]
        contract_score: bool,

        /// Include non-code assets (README, Dockerfile, etc.) in results
        #[arg(long)]
        asset_contracts: bool,
    },

    // ── Analysis & Demo commands ───────────────────────────────────
    /// Analyze code metrics and patterns
    #[command(subcommand, visible_aliases = &["a", "an"])]
    Analyze(AnalyzeCommands),

    /// Quality-Driven Development (QDD) tool for creating and refactoring code with guaranteed quality
    #[command(subcommand, visible_aliases = &["qd"])]
    Qdd(QddCommands),

    /// Interactive demo of pmat's analysis output
    ///
    /// `cargo install pmat` produces a build in which every demo mode (including
    /// `--cli`) exits 1 with "Demo feature not enabled", so all the flags below
    /// are inert. Listed the same way `serve` is: an entry that cannot run must
    /// say so in the command list rather than advertise itself as working.
    ///
    /// The label is CONDITIONAL, matching `Org` below. It used to be an
    /// unconditional doc comment, so a binary built with `--features demo` —
    /// one in which the command works — still told its user
    /// "[NOT AVAILABLE in the default build]". A disclosure that is wrong in
    /// the build that enables the feature is a new defect, not a fix for the
    /// old one.
    #[cfg_attr(
        not(feature = "demo"),
        command(
            about = "[NOT AVAILABLE in the default build] Interactive demo — needs --features demo"
        )
    )]
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
    ///
    /// `cargo install pmat` produces a build in which every `org` subcommand
    /// exits with an error: `org localize` is compiled out (its whole handler is
    /// `#[cfg(feature = "org-intelligence")]`) and `org analyze` is gone for
    /// good, its upstream API having been removed in aprender-orchestrate 0.41.
    /// Labelled the way `serve` and `demo` are: an entry the shipped binary
    /// cannot run must say so in the command list rather than advertise itself
    /// as working.
    #[cfg_attr(
        not(feature = "org-intelligence"),
        command(
            about = "[NOT AVAILABLE in the default build] Organizational intelligence — needs --features org-intelligence"
        )
    )]
    #[command(subcommand, visible_aliases = &["organization"])]
    Org(OrgCommands),

    /// MCP manifest and schema management
    #[command(subcommand)]
    Mcp(McpCommands),

    /// Google Anti-Gravity customizations translator
    #[command(subcommand, visible_aliases = &["antigravity"])]
    Agy(AgyCommands),

    /// Bootstrap an agent-ready workspace: quality hook, MCP registration,
    /// skill and root rules file
    ///
    /// Existing files are never overwritten without `--force`; the report says
    /// what was skipped and why. Artifacts whose format nobody has defined are
    /// refused with a reason rather than filled in with a guess.
    #[command(visible_aliases = &["bootstrap"])]
    Init {
        /// Which client's layout to write
        #[arg(long, value_enum, default_value = "agy")]
        target: InitTarget,

        /// Workspace root to write into (must already exist)
        #[arg(long, default_value = ".")]
        path: PathBuf,

        /// Replace files whose contents differ from the template. Your version
        /// is not backed up.
        #[arg(long)]
        force: bool,

        /// Report format
        #[arg(long, value_enum, default_value = "human")]
        format: InitFormat,
    },

    /// AI prompt generation (defect-aware, ticket-based, spec-based)
    #[command(subcommand, visible_aliases = &["p"])]
    Prompt(PromptCommands),

    // ── Scoring commands ───────────────────────────────────────────
    /// Run quality gate checks on the codebase
    // `verify` is now the dedicated CI-faithful pre-flight command (see `Verify`).
    #[command(visible_aliases = &["check", "c", "gate"])]
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

        // A blocking violation that does not block is a contradiction in terms.
        // Exit 1 on a failed verdict used to be opt-in behind
        // `--fail-on-violation`, so the default invocation printed "Quality
        // gate found 35 blocking violations" and exited 0 — indistinguishable,
        // to any caller that reads only the exit code, from a clean tree. This
        // repo's own `make dogfood-all` ran
        // `pmat quality-gate --perf --max-complexity-p99 20 || (echo "❌ …" &&
        // exit 1)`, whose `||` arm was therefore unreachable. The gate now
        // gates by default and `--report-only` is the opt-out.
        /// Report findings without failing: exit 0 even when the gate finds
        /// blocking violations
        ///
        /// For dashboards and drift tracking, where the report is the product
        /// and the verdict is not. Without it, blocking violations exit 1.
        #[arg(long, visible_alias = "no-fail", conflicts_with = "fail_on_violation")]
        report_only: bool,

        /// Accepted for compatibility; has no effect since 3.32.0, when
        /// blocking violations began exiting non-zero by default
        ///
        /// Pass `--report-only` for the old default (report, always exit 0).
        #[arg(long)]
        fail_on_violation: bool,

        /// Specific checks to run (all by default)
        #[arg(long, value_delimiter = ',')]
        checks: Vec<QualityCheckType>,

        /// Maximum allowed dead code percentage
        #[arg(long, default_value = "15.0")]
        max_dead_code: f64,

        // #683: this was `min_entropy: f64` with `default_value = "2.0"`. Pattern
        // diversity is measured on a 0.0-1.0 scale, so 2.0 was clamped to
        // "require 100% diversity" — unreachable, so the check reported FAILED
        // whatever the user asked for. As an Option, an explicit value is
        // distinguishable from "unset" and can outrank project config.
        /// Minimum required pattern diversity, 0.0-1.0 (0.0 = no diversity
        /// required, always passes) [default: `[entropy] min_pattern_diversity`
        /// from .pmat-gates.toml / .pmat-metrics.toml / pmat.toml, else 0.3]
        #[arg(long)]
        min_entropy: Option<f64>,

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
        #[arg(short = 'f', long, visible_alias = "format", value_enum, default_value = "json")]
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

        /// Output file path (default: stdout; no file is created)
        #[arg(short = 'o', long)]
        output: Option<PathBuf>,

        /// Show performance metrics
        #[arg(long)]
        perf: bool,
    },

    /// Unified quality score — geometric composite (0-100)
    #[command(name = "score")]
    Score {
        #[arg(short = 'p', long, default_value = ".", help = "Project path to score")]
        path: PathBuf,
        #[arg(long, help = "Minimum score threshold (exit 1 if below)")]
        gate: Option<f64>,
        #[arg(short = 'f', long, value_enum, default_value = "text", help = "Output format")]
        format: RepoScoreOutputFormat,
        #[arg(short = 'o', long, help = "Write output to file")]
        output: Option<PathBuf>,
        #[arg(long, help = "Show score trend over recent commits")]
        trend: bool,
        #[arg(long, help = "Check for score regression vs previous commit")]
        regression_check: bool,
        #[arg(long, help = "Score entire sovereign stack")]
        stack: bool,
    },

    /// Calculate repository health score (0-100 scale)
    // The six categories sum to exactly 100 (15+20+15+25+20+5) and the grade
    // thresholds are 0-100, so 100 is the denominator the command actually
    // emits. "0-110" came from a bonus block (`BonusScores`, +10) that nothing
    // ever adds to `total_score`; the help promised a scale no output uses
    // (GH #685). `repo_score_scale_matches_help` pins the two together.
    #[command(name = "repo-score", visible_aliases = &["health"])]
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

    /// Calculate Rust project quality score (0-289 scale; the reported total
    /// excludes categories that do not apply to the project)
    // 289 is the sum of the eleven scorers' max_points. The help said "0-106",
    // a stale v1 figure, while the text renderer printed "/279.0" and CLAUDE.md
    // claimed 289 — three maxima for one command (GH #685).
    // `rust_project_score_scale_matches_help` fails if this number drifts from
    // `RustProjectScoreOrchestrator::max_points()`.
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

    /// Infrastructure Score (0-100 + 12 bonus) for CI/CD quality
    ///
    /// Evaluates GitHub Actions workflows across 5 dimensions:
    /// - Workflow Architecture (25 pts)
    /// - Build Reliability (25 pts)
    /// - Quality Pipeline (20 pts)
    /// - Deployment & Release (15 pts)
    /// - Supply Chain Security (15 pts)
    /// - Provable Contracts (12 pts bonus: PV-01..PV-05 = 3+3+2+2+2)
    ///
    /// Hard cutoff: <90 = auto-fail.
    #[command(name = "infra-score", visible_aliases = &["infra", "ci-score"])]
    InfraScore {
        /// Project path (defaults to current directory)
        #[arg(short = 'p', long, default_value = ".")]
        path: PathBuf,

        /// Output format
        #[arg(short = 'f', long, value_enum, default_value = "text")]
        format: RepoScoreOutputFormat,

        /// Enable verbose output (show per-check details)
        #[arg(short = 'v', long)]
        verbose: bool,

        /// Show only failures and recommendations
        #[arg(long)]
        failures_only: bool,

        /// Output file path
        #[arg(short = 'o', long)]
        output: Option<PathBuf>,
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

    // ── Infrastructure commands ────────────────────────────────────
    /// Serve the MCP tool surface over streamable HTTP (`--transport http`)
    ///
    /// `--transport http` is IMPLEMENTED and round-trips MCP over HTTP: it
    /// serves the same tools as the stdio server, is compiled in only with
    /// `--features mcp-http`, and refuses to start without a bearer token in
    /// `PMAT_MCP_HTTP_TOKEN` (unauthenticated requests get 401).
    ///
    /// The other `--transport` values — `web-socket`, `http-sse`, `both`,
    /// `all` — are NOT IMPLEMENTED and exit 2.
    ///
    /// There is no `stdio` value for `--transport`; passing one is a clap error
    /// (exit 2). For MCP over stdio run `pmat --mode mcp` or `MCP_VERSION=1
    /// pmat`.
    //
    // History, deliberately a code comment and not help text: this entry
    // declared the whole command unimplemented and "exits with an error" for
    // the entire release in which the HTTP transport shipped (EV-6, #999; its
    // end-to-end gate is `tests/e2e_http_serve_t.rs`, which spawns the binary
    // and round-trips MCP over HTTP). Help that hides a shipped feature is the
    // same defect as help that promises a missing one. The text above therefore
    // names what works, what does not, and the build flag between them —
    // `--transport stdio` really is rejected by clap, so it says that too.
    //
    // The featureless text below then got the *error* wrong in the other
    // direction: it promised, unconditionally, that `--transport http` "exits
    // with an error naming that feature". `serve_streamable_http` reads
    // `PMAT_MCP_HTTP_TOKEN` before it ever reaches the feature stub, so with no
    // token set — every first run — the diagnostic names the variable and never
    // mentions `mcp-http` (exit 4, `ExitCode::ConfigurationError`); only once a
    // token is set do you reach the feature error (exit 1). Help that names the
    // one error most users never see is the same class of defect again, so the
    // text now states both cases and which comes first.
    #[cfg_attr(
        not(feature = "mcp-http"),
        command(
            about = "[HTTP NOT COMPILED IN this build] MCP server over streamable HTTP — needs --features mcp-http",
            long_about = "Serve the MCP tool surface over streamable HTTP (`--transport http`).\n\n\
                          This binary was built WITHOUT `--features mcp-http`, so `--transport http` \
                          cannot serve here — nothing binds a socket either way. Which error you get \
                          depends on the environment, because the bearer token is checked before the \
                          missing feature is: with `PMAT_MCP_HTTP_TOKEN` unset, the usual case, the \
                          error names that variable and says nothing about the build; only with \
                          `PMAT_MCP_HTTP_TOKEN` set does the error name `mcp-http`.\n\n\
                          Rebuild with `--features mcp-http` to serve; a bearer token in \
                          `PMAT_MCP_HTTP_TOKEN` is required in that build too (unauthenticated \
                          requests get 401).\n\n\
                          The other `--transport` values — `web-socket`, `http-sse`, `both`, `all` — \
                          are NOT IMPLEMENTED in any build and exit 2.\n\n\
                          There is no `stdio` value for `--transport`; passing one is a clap error \
                          (exit 2). For MCP over stdio run `pmat --mode mcp` or `MCP_VERSION=1 pmat`."
        )
    )]
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

    /// Pre-flight verification for autonomous agents: run the CI-faithful gate
    /// set (format, complexity, satd, clippy, tests) fail-fast before committing
    #[command(visible_aliases = &["preflight", "vfy"])]
    Verify(VerifyArgs),

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

        /// Record cargo test results to .pmat-metrics/ for EvoScore (CB-142)
        #[arg(long)]
        record: bool,

        /// Parse cargo test output from stdin instead of running cargo test
        #[arg(long, requires = "record")]
        from_stdin: bool,

        /// Show what would be recorded without writing files
        #[arg(long, requires = "record")]
        dry_run: bool,
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

    /// [NOT AVAILABLE in the default build] Claude Code background agent — needs --features agent-daemon
    ///
    /// Continuous quality monitoring by a background daemon. `agent-daemon`
    /// is not in the default feature set, so on a `cargo install pmat`
    /// binary every one of the subcommands below — `start`, `stop`,
    /// `status`, `monitor`, `unmonitor`, `health`, `reload`,
    /// `quality-gate`, `mcp-server` — exits rc=1 with "Agent daemon feature
    /// not enabled".
    #[command(visible_aliases = &["ag"])]
    Agent {
        /// Agent mode subcommand
        #[command(subcommand)]
        command: AgentCommands,
    },

    // ── Dev tools commands ─────────────────────────────────────────
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

    // ── Operations commands ────────────────────────────────────────
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

    /// Explain what a check, metric, or grade means
    ///
    /// Look up any check ID (CB-1210, PV-05, TDG-A) to see what it checks,
    /// why it matters, when it fails, and how to fix it.
    #[command(visible_aliases = &["explain-check", "what-is"])]
    Explain {
        /// Check ID or search pattern (e.g., CB-1210, PV-05, TDG-A, "precondition")
        /// Omit to list all available checks.
        pattern: Option<String>,
    },

    /// Specification management and validation
    #[command(name = "spec", visible_aliases = &["specification"])]
    Spec {
        #[command(subcommand)]
        command: SpecCommands,
    },

    /// Run local CI simulation (quality gates, clippy, tests, cross-compilation)
    #[command(visible_aliases = &["ci", "local-ci"])]
    CiLocal {
        /// Project path
        #[arg(short = 'p', long, default_value = ".")]
        path: std::path::PathBuf,

        /// Run quick subset only (fmt + clippy + test-fast)
        #[arg(long)]
        quick: bool,

        /// Run specific matrix (e.g., "clippy", "test", "cross", "bench", "fmt")
        #[arg(long)]
        matrix: Option<String>,

        /// Auto-fix issues (cargo fmt, clippy --fix)
        #[arg(long)]
        fix: bool,

        /// Output format
        #[arg(short = 'f', long, value_enum, default_value = "table")]
        format: OutputFormat,

        /// Verbose output
        #[arg(short = 'v', long)]
        verbose: bool,
    },

    /// PMAT compliance checking and migration system (runs check by default)
    #[command(visible_aliases = &["compliance"])]
    Comply {
        #[command(subcommand)]
        command: Option<ComplyCommands>,
    },

    /// Autonomous continuous improvement (Toyota Way Kaizen)
    /// Scans, fixes, commits, and files GitHub issues for remaining findings
    #[command(visible_aliases = &["improve"])]
    Kaizen {
        /// Project path to analyze
        #[arg(short = 'p', long, default_value = ".")]
        path: PathBuf,

        /// Scan only, do not apply any fixes
        #[arg(long)]
        dry_run: bool,

        /// Suppress auto-commit after fixes (commit is default)
        #[arg(long)]
        no_commit: bool,

        /// Suppress GitHub issue creation for unfixed findings
        #[arg(long)]
        no_issues: bool,

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

        /// Run kaizen across all discovered batuta stack crates
        #[arg(long)]
        cross_stack: bool,
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

    /// Detect flaky and timeout-sensitive tests
    #[command(name = "test-stability", visible_aliases = &["test-flaky", "flaky"])]
    TestStability {
        /// Project path
        #[arg(short = 'p', long, default_value = ".")]
        path: std::path::PathBuf,

        /// Number of test runs
        #[arg(short = 'n', long, default_value = "3")]
        runs: usize,

        /// Filter tests by name pattern
        #[arg(long)]
        filter: Option<String>,

        /// Output format
        #[arg(short = 'f', long, value_enum, default_value = "table")]
        format: crate::cli::enums::OutputFormat,

        /// Output file
        #[arg(short = 'o', long)]
        output: Option<std::path::PathBuf>,
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
        #[arg(long, default_value = "tarantula", value_parser = parse_sbfl_formula_value)]
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

    /// Analyze and suggest semantic file splits using Louvain community detection
    ///
    /// Examines function call graphs within a file and clusters related functions.
    /// Each cluster gets a semantic name based on dominant types, themes, or prefixes.
    /// Use --execute to create split files with include!() pattern.
    /// Use --auto to scan the project and split all oversized files.
    #[command(visible_aliases = &["sp"])]
    Split {
        /// File to analyze for splitting (required unless --auto is used)
        file: Option<PathBuf>,

        /// Project path (defaults to current directory)
        #[arg(short = 'p', long, default_value = ".")]
        path: PathBuf,

        /// Execute the split (create files). Default is dry-run.
        #[arg(long)]
        execute: bool,

        /// Output format (text, json)
        #[arg(short = 'f', long, default_value = "text")]
        format: String,

        /// Write output to file
        #[arg(short = 'o', long)]
        output: Option<PathBuf>,

        /// Minimum lines for a cluster to be kept (smaller clusters become unclustered)
        #[arg(long, default_value_t = 50)]
        min_cluster_lines: usize,

        /// Louvain resolution parameter (higher = more clusters)
        #[arg(long, default_value_t = 1.0)]
        resolution: f64,

        /// Automatically find and split all oversized files in the project
        #[arg(long)]
        auto: bool,

        /// Maximum lines per file before splitting (used with --auto)
        #[arg(long, default_value_t = 500)]
        max_lines: usize,

        /// Preview split plan without making changes (used with --auto)
        #[arg(long)]
        dry_run: bool,

        /// Auto-commit each split (used with --auto)
        #[arg(long)]
        commit: bool,
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

    /// Direct SQL access to the function index database
    ///
    /// Run SQL queries against the .pmat/context.db SQLite index.
    /// Supports named example queries, custom SQL, and multiple output formats.
    #[command(name = "sql")]
    Sql {
        /// SQL query or named example (e.g., "grade-dist", "worst-files", "complex-funcs")
        query: Option<String>,

        /// Output format: table, json, csv
        #[arg(short = 'f', long, default_value = "table")]
        format: String,

        /// Query workspace index instead of local project
        #[arg(long)]
        workspace: bool,

        /// Print table schemas
        #[arg(long)]
        schema: bool,

        /// Print built-in example queries
        #[arg(long)]
        examples: bool,

        /// Project path
        #[arg(short = 'p', long, default_value = ".")]
        path: PathBuf,
    },

    /// Cross-repo dependency coordination for the sovereign AI stack
    ///
    /// Discovers batuta stack repos, checks dependency versions against crates.io,
    /// and optionally syncs them to the latest published versions.
    #[command(visible_aliases = &["stk"])]
    Stack {
        #[command(subcommand)]
        command: StackCommands,
    },
}

// ── Enum-valued options that are typed as String ────────────────────────────
//
// `--min-grade`, `--format`, `--color` and `--search-mode` are clap-validated
// and reject a bad value with exit 2. These three were plain `String`s parsed
// later with `unwrap_or_default()` / `unwrap_or(..)`, so a typo silently
// selected the default: `--rank-by bogus` produced output byte-identical to
// `--rank-by relevance`, `--type bogus` matched nothing (indistinguishable
// from an honest empty result), and `--formula bogus` printed
// `Formula: bogus` above a report the banner labelled `Tarantula`. Each parser
// below defers to the SAME `FromStr`/mapping the consumer uses, so the accepted
// set cannot drift from the implemented set.

/// clap value_parser for `query --rank-by`.
fn parse_rank_by_value(s: &str) -> Result<String, String> {
    s.parse::<crate::services::agent_context::RankBy>()?;
    Ok(s.to_string())
}

/// clap value_parser for `query --type`.
fn parse_definition_type_value(s: &str) -> Result<String, String> {
    if crate::services::agent_context::normalize_definition_type(s).is_some() {
        return Ok(s.to_string());
    }
    Err(format!(
        "unknown definition type '{s}'. Valid: {}",
        crate::services::agent_context::DEFINITION_TYPE_VALUES.join(", ")
    ))
}

/// clap value_parser for `localize --formula`.
fn parse_sbfl_formula_value(s: &str) -> Result<String, String> {
    s.parse::<crate::services::fault_localization::SbflFormula>()
        .map_err(|e| e.to_string())?;
    Ok(s.to_string())
}

#[cfg_attr(coverage_nightly, coverage(off))]
#[cfg(test)]
mod enum_valued_option_tests {
    use super::{
        parse_definition_type_value, parse_rank_by_value, parse_sbfl_formula_value,
    };

    /// `--rank-by bogus` used to exit 0 with output byte-identical to
    /// `--rank-by relevance` (`unwrap_or_default()` on the parse).
    #[test]
    fn rank_by_rejects_an_unknown_strategy() {
        assert!(parse_rank_by_value("bogus").is_err());
        assert!(parse_rank_by_value("pagerank").is_ok());
        assert!(parse_rank_by_value("relevance").is_ok());
        // Aliases the consumer accepts must not be rejected at the edge.
        assert!(parse_rank_by_value("pr").is_ok());
        assert!(parse_rank_by_value("cross-project").is_ok());
        assert!(parse_rank_by_value("priority").is_ok());
    }

    /// `--type bogus` used to exit 0 having matched zero definitions, which is
    /// indistinguishable from an honest empty result.
    #[test]
    fn definition_type_rejects_an_unknown_type() {
        assert!(parse_definition_type_value("bogus").is_err());
        for ok in ["fn", "func", "function", "struct", "enum", "trait", "type"] {
            assert!(parse_definition_type_value(ok).is_ok(), "{ok}");
        }
    }

    /// `--formula bogus` used to exit 0 and print `Formula: bogus` above a
    /// report the banner labelled `Tarantula`, scored with Tarantula.
    #[test]
    fn sbfl_formula_rejects_an_unknown_formula() {
        assert!(parse_sbfl_formula_value("bogus").is_err());
        for ok in ["tarantula", "ochiai", "dstar2", "dstar3"] {
            assert!(parse_sbfl_formula_value(ok).is_ok(), "{ok}");
        }
    }

    /// The clap tree itself must reject the value — that is the surface the
    /// defect was observed on: `pmat query 'dispatch' --rank-by bogus` exited
    /// 0 with output byte-identical to `--rank-by relevance`.
    ///
    /// `on_big_stack` because building the subcommand tree overflows the 2MB
    /// test stack.
    #[test]
    fn clap_rejects_the_bad_values_and_accepts_the_good_ones() {
        use clap::Subcommand;

        fn parses(args: &[&str]) -> bool {
            let args: Vec<String> = args.iter().map(|s| (*s).to_string()).collect();
            crate::cli::commands::on_big_stack(move || {
                let cmd = super::Commands::augment_subcommands(clap::Command::new("pmat"));
                cmd.try_get_matches_from(args).is_ok()
            })
        }

        assert!(
            !parses(&["pmat", "query", "dispatch", "--rank-by", "bogus"]),
            "--rank-by bogus must be rejected, not silently treated as relevance"
        );
        assert!(parses(&["pmat", "query", "dispatch", "--rank-by", "pagerank"]));

        assert!(
            !parses(&["pmat", "query", "cache", "--type", "bogus"]),
            "--type bogus must be rejected, not silently match zero definitions"
        );
        assert!(parses(&["pmat", "query", "cache", "--type", "struct"]));

        let localize = |formula: &str| {
            parses(&[
                "pmat",
                "localize",
                "--passed-coverage",
                "p.info",
                "--failed-coverage",
                "f.info",
                "--passed-count",
                "5",
                "--failed-count",
                "3",
                "--formula",
                formula,
            ])
        };
        assert!(
            !localize("bogus"),
            "--formula bogus must be rejected, not relabelled onto Tarantula"
        );
        assert!(localize("ochiai"));
    }

    /// The error names the alternatives rather than just failing.
    #[test]
    fn rejection_messages_list_the_valid_values() {
        let e = parse_rank_by_value("bogus").unwrap_err();
        assert!(e.contains("pagerank"), "{e}");
        let e = parse_definition_type_value("bogus").unwrap_err();
        assert!(e.contains("struct"), "{e}");
    }
}

#[cfg_attr(coverage_nightly, coverage(off))]
#[cfg(test)]
mod command_availability_tests {
    use super::Commands;
    use clap::Subcommand;

    fn about_of(name: &str) -> String {
        let name = name.to_string();
        crate::cli::commands::on_big_stack(move || {
            let cmd = Commands::augment_subcommands(clap::Command::new("pmat"));
            let about = cmd
                .get_subcommands()
                .find(|s| s.get_name() == name)
                .unwrap_or_else(|| panic!("{name} subcommand must exist"))
                .get_about()
                .map(std::string::ToString::to_string)
                .unwrap_or_default();
            about
        })
    }

    /// The text `pmat <cmd> --help` prints, which is `long_about` when the
    /// command has one and `about` otherwise — the same fallback clap applies.
    fn long_about_of(name: &str) -> String {
        let name = name.to_string();
        crate::cli::commands::on_big_stack(move || {
            let cmd = Commands::augment_subcommands(clap::Command::new("pmat"));
            let subcommand = cmd
                .get_subcommands()
                .find(|s| s.get_name() == name)
                .unwrap_or_else(|| panic!("{name} subcommand must exist"))
                .clone();
            subcommand
                .get_long_about()
                .or_else(|| subcommand.get_about())
                .map(std::string::ToString::to_string)
                .unwrap_or_default()
        })
    }

    /// The default (`cargo install pmat`) build cannot run any demo mode, yet
    /// the command list advertised "Run interactive demo of all capabilities".
    /// A command list that promises a working command the shipped binary cannot
    /// run is the defect.
    #[test]
    fn demo_is_labelled_unavailable() {
        let demo = about_of("demo");
        assert!(
            demo.contains("NOT AVAILABLE") || demo.contains("NOT IMPLEMENTED"),
            "demo must be labelled unavailable in the command list, got: {demo}"
        );
        assert!(
            demo.contains("--features demo"),
            "the label must name the feature that would enable it, got: {demo}"
        );
    }

    /// The mirror image of the `demo` defect: `serve`'s help declared the whole
    /// command "[NOT IMPLEMENTED] ... exits with an error" while
    /// `--transport http` was shipped, wired from `main` and covered by
    /// `tests/e2e_http_serve_t.rs`. Help that hides a working feature is as
    /// wrong as help that promises a missing one.
    ///
    /// Accuracy here is not "say something positive": `--transport stdio` really
    /// is rejected by clap, and the other transports really do exit 2, so the
    /// text must say which transports work and keep pointing at
    /// `pmat --mode mcp` / `MCP_VERSION=1 pmat` for stdio MCP.
    #[test]
    fn serve_help_says_which_transports_work() {
        let help = long_about_of("serve");

        assert!(
            !help.contains("[NOT IMPLEMENTED] HTTP/WebSocket server"),
            "the HTTP transport is implemented; help must not deny the whole \
             command, got: {help}"
        );
        assert!(
            help.contains("--transport http") || help.contains("`--transport http`"),
            "help must name the transport that works, got: {help}"
        );
        assert!(
            help.contains("mcp-http"),
            "help must name the feature the HTTP transport is compiled behind, \
             got: {help}"
        );
        assert!(
            help.contains("PMAT_MCP_HTTP_TOKEN"),
            "help must name the token without which the server refuses to \
             start, got: {help}"
        );

        // The transports that genuinely do not work must still say so, or this
        // fix would trade one inaccurate help text for another.
        for unimplemented in ["web-socket", "http-sse", "both", "all"] {
            assert!(
                help.contains(unimplemented),
                "help must name `{unimplemented}` among the transports that are \
                 not implemented, got: {help}"
            );
        }
        assert!(
            help.contains("NOT IMPLEMENTED"),
            "the unimplemented transports must be labelled, got: {help}"
        );

        // `--transport stdio` is a clap error, so the help must not read as if
        // stdio were a transport value here, and must route stdio MCP to the
        // entry points that serve it.
        assert!(
            help.contains("--mode mcp"),
            "help must point at `pmat --mode mcp` for stdio MCP, got: {help}"
        );
        assert!(
            help.contains("MCP_VERSION=1"),
            "help must point at `MCP_VERSION=1 pmat` for stdio MCP, got: {help}"
        );
        assert!(
            help.contains("no `stdio` value"),
            "help must say `stdio` is not a --transport value, got: {help}"
        );
    }

    /// The claim `serve_help_says_which_transports_work` makes about `stdio`
    /// has to stay true of the enum, not just of the prose.
    #[test]
    fn transport_has_no_stdio_value() {
        use clap::ValueEnum;
        let values: Vec<String> = crate::cli::commands::ServeTransport::value_variants()
            .iter()
            .filter_map(|v| {
                clap::ValueEnum::to_possible_value(v).map(|p| p.get_name().to_string())
            })
            .collect();
        assert!(
            values.iter().any(|v| v == "http"),
            "http is the transport that works: {values:?}"
        );
        assert!(
            !values.iter().any(|v| v == "stdio"),
            "if a stdio transport is ever added, the help text that says there \
             is none must be updated with it: {values:?}"
        );
    }

    /// The sentences of a help text, with clap's line wrapping normalised away
    /// so an assertion cannot pass or fail on where a paragraph happens to
    /// break. `;` ends a clause here as much as `.` does.
    fn sentences(help: &str) -> Vec<String> {
        help.replace('\n', " ")
            .split(['.', ';'])
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .collect()
    }

    /// The sentence that describes what happens when no token is set, if the
    /// help has one: it must name the variable *and* say the variable is unset,
    /// because a mention that only promises the token matters after a rebuild
    /// does not describe this build's behaviour.
    fn sentence_about_the_missing_token(help: &str) -> Option<String> {
        sentences(help).into_iter().find(|s| {
            let lower = s.to_lowercase();
            s.contains("PMAT_MCP_HTTP_TOKEN")
                && (lower.contains("unset") || lower.contains("not set") || lower.contains("without"))
        })
    }

    /// Drive `pmat serve --transport http` in-process with the token env var in
    /// a chosen state and return the error it produces.
    ///
    /// `token: Some(..)` must only be used in a build without `mcp-http`: with
    /// the feature compiled in, a token makes this bind a socket and never
    /// return.
    async fn serve_http_error(token: Option<&str>) -> String {
        use crate::mcp_pmcp::http_server::TOKEN_ENV;
        let saved = std::env::var(TOKEN_ENV).ok();
        match token {
            Some(t) => std::env::set_var(TOKEN_ENV, t),
            None => std::env::remove_var(TOKEN_ENV),
        }
        let got = crate::cli::handlers::utility_serve_handlers::handle_serve(
            "127.0.0.1".to_string(),
            0,
            false,
            crate::cli::commands::ServeTransport::Http,
        )
        .await;
        match saved {
            Some(v) => std::env::set_var(TOKEN_ENV, v),
            None => std::env::remove_var(TOKEN_ENV),
        }
        got.expect_err("`--transport http` must not serve in this test")
            .to_string()
    }

    /// The help must describe the failure the binary actually produces.
    ///
    /// `serve --help` in a build without `--features mcp-http` stated flatly
    /// that `--transport http` "exits with an error naming that feature", and
    /// mentioned `PMAT_MCP_HTTP_TOKEN` only as something the *rebuilt* binary
    /// would then want. Both halves are wrong about the first run anyone makes:
    /// `serve_streamable_http` reads the token before it reaches the feature
    /// stub, so with no token the diagnostic names the variable and never says
    /// `mcp-http`. The measurement below is what the assertion is checked
    /// against, so this cannot drift into pinning prose.
    #[tokio::test]
    #[serial_test::serial(pmat_mcp_http_token)]
    async fn serve_help_describes_the_no_token_failure_it_actually_produces() {
        let measured = serve_http_error(None).await;
        assert!(
            measured.contains("PMAT_MCP_HTTP_TOKEN"),
            "reality check: with no token the diagnostic must name the variable \
             to set, got: {measured}"
        );

        let help = long_about_of("serve");
        let described = sentence_about_the_missing_token(&help);
        assert!(
            described.is_some(),
            "`pmat serve --help` must describe the failure an unset \
             PMAT_MCP_HTTP_TOKEN actually produces ({measured}); the help \
             instead only names the token as something a later build wants, \
             got: {help}"
        );
    }

    /// The other half of the same claim, in the build where it is checkable:
    /// without the feature the `mcp-http` diagnostic exists, but you can only
    /// reach it by setting a token first, so the help may not present it as
    /// what `--transport http` does.
    #[cfg(not(feature = "mcp-http"))]
    #[tokio::test]
    #[serial_test::serial(pmat_mcp_http_token)]
    async fn serve_help_ties_the_feature_error_to_the_token_being_set() {
        let without_token = serve_http_error(None).await;
        assert!(
            !without_token.contains("mcp-http"),
            "reality check: the no-token diagnostic says nothing about the \
             build feature, got: {without_token}"
        );
        let with_token = serve_http_error(Some("0123456789abcdef0123")).await;
        assert!(
            with_token.contains("mcp-http"),
            "reality check: with a token set, the feature stub is reached and \
             names the feature, got: {with_token}"
        );

        let help = long_about_of("serve");
        let qualified = sentences(&help).into_iter().find(|s| {
            let lower = s.to_lowercase();
            lower.contains("mcp-http") && lower.contains(" set") && !lower.contains("unset")
        });
        assert!(
            qualified.is_some(),
            "the `mcp-http` error is only reachable with PMAT_MCP_HTTP_TOKEN \
             set, so the help must say so rather than assert it unconditionally, \
             got: {help}"
        );
    }

    /// Same defect as `demo`: in the default build every `org` subcommand exits
    /// with an error (`localize` is compiled out, `analyze` was removed
    /// upstream), yet the command list advertised it as a working feature.
    #[cfg(not(feature = "org-intelligence"))]
    #[test]
    fn org_is_labelled_unavailable_in_the_default_build() {
        let org = about_of("org");
        assert!(
            org.contains("NOT AVAILABLE") || org.contains("NOT IMPLEMENTED"),
            "org must be labelled unavailable in the command list, got: {org}"
        );
        assert!(
            org.contains("--features org-intelligence"),
            "the label must name the feature that would enable it, got: {org}"
        );
    }

    /// `org analyze` is dead with the feature ON as well — its upstream API was
    /// deleted — so its own entry must say so in every build.
    #[test]
    fn org_analyze_is_labelled_removed_in_every_build() {
        let analyze = crate::cli::commands::on_big_stack(|| {
            let cmd = Commands::augment_subcommands(clap::Command::new("pmat"));
            let about = cmd
                .get_subcommands()
                .find(|s| s.get_name() == "org")
                .expect("org subcommand must exist")
                .get_subcommands()
                .find(|s| s.get_name() == "analyze")
                .expect("org analyze subcommand must exist")
                .get_about()
                .map(std::string::ToString::to_string)
                .unwrap_or_default();
            about
        });
        assert!(
            analyze.contains("REMOVED"),
            "org analyze must be labelled removed, got: {analyze}"
        );
    }

    /// PMAT-INIT-001 claim 1: "CLI provides a `pmat init` command."
    ///
    /// `pmat init --help` used to answer "unrecognized subcommand". The
    /// end-to-end proof is in `tests/init_workspace_t.rs`, which spawns the
    /// binary — but that target does not run under `cargo test --lib`, which
    /// is what CI executes, so the presence of the command and of every target
    /// it advertises is also asserted here, against the clap tree itself.
    #[test]
    fn init_exists_and_offers_all_three_targets() {
        let (about, targets) = crate::cli::commands::on_big_stack(|| {
            let cmd = Commands::augment_subcommands(clap::Command::new("pmat"));
            let init = cmd
                .get_subcommands()
                .find(|s| s.get_name() == "init")
                .expect("init subcommand must exist")
                .clone();
            let about = init.get_about().map(ToString::to_string).unwrap_or_default();
            let targets: Vec<String> = init
                .get_arguments()
                .find(|a| a.get_id() == "target")
                .expect("init must take --target")
                .get_possible_values()
                .iter()
                .map(|v| v.get_name().to_string())
                .collect();
            (about, targets)
        });

        assert!(
            about.contains("Bootstrap"),
            "init must describe what it does, got: {about}"
        );
        assert_eq!(
            targets,
            vec!["agy", "claude", "ultracode"],
            "the advertised targets drifted from the generator's"
        );
    }
}

#![cfg_attr(coverage_nightly, coverage(off))]
// Analyze commands - extracted for file health (CB-040)
//
// NOTE: Rust does not allow enum definitions to span multiple files.
// This enum is organized into semantic sections with clear headers.
// Variants are grouped: Core, Debt, TDG/Quality, Advanced, Metrics,
// Coverage/Symbols, WASM/Specialized.

use super::semantic_search::ClusterMethod;
use crate::cli::handlers::coverage_improve_handler::CoverageImproveOutputFormat;
use crate::cli::{
    BigOOutputFormat, ComplexityOutputFormat, ComprehensiveOutputFormat, DagType,
    DeadCodeOutputFormat, DeepContextCacheStrategy, DeepContextDagType, DeepContextOutputFormat,
    DefectPredictionOutputFormat, DefectsOutputFormat, DuplicateOutputFormat, DuplicateType,
    EntropyOutputFormat, EntropySeverity, GraphMetricType, GraphMetricsOutputFormat,
    IncrementalCoverageOutputFormat, LintHotspotOutputFormat, MakefileOutputFormat,
    NameSimilarityOutputFormat, OutputFormat, ProofAnnotationOutputFormat, PropertyTypeFilter,
    ProvabilityOutputFormat, SatdOutputFormat, SatdSeverity, SearchScope, SymbolTableOutputFormat,
    SymbolTypeFilter, TdgOutputFormat, VerificationMethodFilter, WasmOutputFormat,
};
#[cfg(feature = "deep-wasm")]
use crate::cli::{DeepWasmFocus, DeepWasmLanguage, DeepWasmOutputFormat};
use crate::models::churn::ChurnOutputFormat;
use clap::Subcommand;
use std::path::PathBuf;

/// Analyze subcommands
#[derive(Subcommand)]
#[cfg_attr(test, derive(Debug))]
pub enum AnalyzeCommands {
    // ── Core Analysis ──────────────────────────────────────────────
    // Bottleneck, Churn, Complexity, Dag, DeadCode, Defects
    /// Detect architectural churn bottleneck files
    #[command(visible_aliases = &["btn", "hotspot"])]
    Bottleneck {
        /// Project path
        #[arg(short = 'p', long, default_value = ".")]
        path: std::path::PathBuf,

        /// Analysis period in days
        #[arg(long, default_value = "30")]
        period: u32,

        /// Minimum touches to flag a file
        #[arg(long, default_value = "5")]
        threshold: usize,

        /// Output format
        #[arg(short = 'f', long, value_enum, default_value = "table")]
        format: OutputFormat,

        /// Output file
        #[arg(short = 'o', long)]
        output: Option<std::path::PathBuf>,
    },

    /// Analyze code churn (change frequency)
    #[command(visible_aliases = &["ch"])]
    Churn {
        /// Path to analyze (file or directory)
        #[arg(short = 'p', long, default_value = ".")]
        path: PathBuf,

        /// DEPRECATED: Use --path instead
        #[arg(long, hide = true)]
        project_path: Option<PathBuf>,

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

        // Enforced by `run_within_analysis_budget` — before that it was a
        // banner only. Default raised from 60s when the bound became real:
        // this repo's 4400 files take 8.1s to walk, so 60s left under an order
        // of magnitude of headroom and would have turned a large monorepo's
        // DEFAULT invocation into a failure.
        /// Analysis timeout in seconds
        ///
        /// Cancels the analysis and exits non-zero once the budget is spent —
        /// it is a bound, not advice. `--timeout 0` is a zero-length budget,
        /// not "no limit".
        #[arg(long, default_value = "300")]
        timeout: u64,

        /// NOT IMPLEMENTED: ML-based scoring (aprender LinearRegression)
        ///
        /// The handler never consumed this flag, so `--ml` returned exactly the
        /// heuristic scores under a promise of "trained ML models instead of
        /// heuristic formulas". It now errors instead of relabelling them; the
        /// help text stays visible so the refusal is discoverable (GH-97).
        #[arg(long)]
        ml: bool,
    },

    /// Generate dependency graphs using Mermaid
    #[command(visible_aliases = &["dep", "graph"])]
    Dag {
        /// Type of dependency graph to generate
        #[arg(long, value_enum, default_value = "full-dependency")]
        dag_type: DagType,

        /// Path to analyze (file or directory)
        #[arg(short = 'p', long, default_value = ".")]
        path: PathBuf,

        /// DEPRECATED: Use --path instead
        #[arg(long, hide = true)]
        project_path: Option<PathBuf>,

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

        // #929 made this budget real, and here it bounds a `cargo check` child
        // rather than a walk. The old 60s default was then smaller than the
        // work the DEFAULT invocation does: `pmat analyze dead-code -p .` on
        // this repo measured 67.6s with a warm target dir and 245s cold, so 60s
        // made the plain command fail on the project that ships it.
        /// Analysis timeout in seconds
        ///
        /// Cancels the analysis — including the `cargo check` it runs — and
        /// exits non-zero once the budget is spent. A COLD `cargo check` on a
        /// large crate takes minutes; raise this rather than reading the
        /// failure as a hang. `--timeout 0` is a zero-length budget, not "no
        /// limit".
        #[arg(long, default_value = "900")]
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

    /// Report tracked .rs files that no compilation unit reaches
    ///
    /// rustc emits no diagnostic for a `.rs` file that no `mod`, `#[path]` or
    /// `include!` reaches, so an orphaned module compiles to nothing and its
    /// tests report `0 passed; ok`. A stack-wide audit found ~475 such files,
    /// >320,000 lines and ~8,900 tests that have never executed.
    #[command(name = "reachability", visible_aliases = &["orphans", "unreachable"])]
    Reachability {
        /// Path to analyze (defaults to current directory)
        #[arg(long, short = 'p', default_value = ".")]
        path: PathBuf,

        /// Output format
        #[arg(long, short = 'f', default_value = "summary")]
        format: String,

        /// Exit non-zero when any tracked file is unreachable
        #[arg(long)]
        fail_on_orphan: bool,
    },

    /// Find machine-specific absolute paths baked into source
    ///
    /// aprender shipped binaries containing `/home/noah/…`: correct on the
    /// machine that built them, inert everywhere else, and invisible to every
    /// gate — the code compiled, clippy was clean, and the path was just a
    /// string literal. Flags a path only when it names a specific user, nix
    /// store hash or build root; `/usr/bin/env` and `/home/$USER` are portable
    /// and are not findings.
    #[command(name = "hardcoded-paths", visible_aliases = &["abs-paths", "path-leaks"])]
    HardcodedPaths {
        /// Path to analyze (defaults to current directory)
        #[arg(long, short = 'p', default_value = ".")]
        path: PathBuf,

        /// Output format
        #[arg(long, short = 'f', default_value = "summary")]
        format: String,

        /// Exit non-zero when a path leaks into shipped (non-test, non-doc) code
        #[arg(long)]
        fail_on_shipped: bool,

        /// Exit non-zero on any finding, including tests and documentation
        #[arg(long)]
        fail_on_any: bool,
    },

    /// Find #[test] functions that cannot fail
    ///
    /// A stack-wide audit counted ~933 tests whose bodies contain nothing that
    /// can fail — 584 of forjar's 802 use `let _ = <fallible call>;`, the
    /// minimum edit that executes a line without checking it. Line coverage is
    /// the only fleet metric with a hard floor, and it measures execution, not
    /// verification. Also reports tests that `return` early when a fixture is
    /// missing: those pass having checked nothing, invisibly, unlike #[ignore].
    #[command(name = "vacuous-tests", visible_aliases = &["vacuous", "fake-tests"])]
    VacuousTests {
        /// Path to analyze (defaults to current directory)
        #[arg(long, short = 'p', default_value = ".")]
        path: PathBuf,

        /// Output format
        #[arg(long, short = 'f', default_value = "summary")]
        format: String,

        /// Exit non-zero when the vacuous rate exceeds this percentage
        #[arg(long)]
        max_rate: Option<f64>,

        /// Exit non-zero when any test cannot fail
        #[arg(long)]
        fail_on_any: bool,
    },

    // ── Debt Analysis ─────────────────────────────────────────────
    // Satd, DeepContext
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
        /// Path to analyze (defaults to current directory)
        #[arg(long, short = 'p', default_value = ".")]
        path: PathBuf,

        /// DEPRECATED: Use --path instead
        #[arg(long, hide = true)]
        project_path: Option<PathBuf>,

        /// Output file path
        #[arg(short, long)]
        output: Option<PathBuf>,

        /// Output format
        #[arg(long, value_enum, default_value = "markdown")]
        format: DeepContextOutputFormat,

        /// NOT IMPLEMENTED: enable full detailed report (default is terse)
        ///
        /// Bound to a parameter nothing read, so every variant of the command
        /// over one corpus produced the same report; the handler now refuses
        /// it rather than relabelling that report as "full" (#915). Use
        /// --top-files to size the report. The help text stays visible so the
        /// refusal is discoverable, matching `analyze complexity --ml`.
        #[arg(long)]
        full: bool,

        /// NOT IMPLEMENTED: comma-separated list of analyses to include
        ///
        /// deep-context always runs the same pipeline; this never selected a
        /// stage. Refused by the handler (#915) — use --include-pattern to
        /// select files.
        #[arg(long, value_delimiter = ',')]
        include: Vec<String>,

        /// NOT IMPLEMENTED: comma-separated list of analyses to exclude
        ///
        /// Counterpart of --include and equally unread; refused by the handler
        /// (#915) — use --exclude-pattern to drop files.
        #[arg(long, value_delimiter = ',')]
        exclude: Vec<String>,

        /// Period for churn analysis (default: 30 days)
        #[arg(long, default_value_t = 30)]
        period_days: u32,

        /// NOT IMPLEMENTED: DAG type for dependency analysis
        ///
        /// deep-context builds no DAG at all — `SimpleDeepContext` walks files
        /// for complexity and SATD — so all four values produced one identical
        /// report (same sha256 after stripping the duration). It carried a clap
        /// default, which is why it could not be refused alongside --full:
        /// there was no way to tell "user asked" from "clap filled it in". It is
        /// an `Option` now and the handler refuses it (#915). Use `analyze dag
        /// --dag-type`, which does read it.
        #[arg(long, value_enum)]
        dag_type: Option<DeepContextDagType>,

        /// NOT IMPLEMENTED: maximum directory traversal depth
        ///
        /// Traversal was never bounded by this value; refused by the handler
        /// (#915) — use --include-pattern / --exclude-pattern to limit what is
        /// walked.
        #[arg(long)]
        max_depth: Option<usize>,

        /// Include file patterns (can be specified multiple times)
        #[arg(long = "include-pattern")]
        include_patterns: Vec<String>,

        /// Exclude file patterns (can be specified multiple times)  
        #[arg(long = "exclude-pattern")]
        exclude_patterns: Vec<String>,

        /// NOT IMPLEMENTED: cache usage strategy
        ///
        /// This path consults and writes no cache: run all three values against
        /// an empty HOME and nothing is created, so `force-refresh` has nothing
        /// to refresh and `offline` nothing to fall back to. Same clap-default
        /// problem as --dag-type; an `Option` now, and refused by the handler
        /// (#915).
        #[arg(long, value_enum)]
        cache_strategy: Option<DeepContextCacheStrategy>,

        /// NOT IMPLEMENTED: parallelism level for analysis
        ///
        /// The number never reached a thread pool — `route_deep_context_analysis`
        /// collapses it to `parallel.is_some()` and the handler refuses it
        /// (#915).
        #[arg(long)]
        parallel: Option<usize>,

        /// Enable verbose logging
        #[arg(long)]
        verbose: bool,

        /// Number of top files to show (0 = all)
        #[arg(long, default_value = "10")]
        top_files: usize,
    },

    // ── TDG & Quality Gates ───────────────────────────────────────
    // Tdg, BuildTdg, LintHotspot, Makefile
    /// Analyze Technical Debt Gradient (TDG) scores
    #[command(name = "tdg")]
    Tdg {
        /// Path to analyze (defaults to current directory)
        #[arg(long, short = 'p', default_value = ".")]
        path: PathBuf,

        /// TDG threshold for filtering results
        #[arg(short, long, default_value = "1.5")]
        threshold: f64,

        /// Number of worst-scoring files to list (0 = all). Project totals are
        /// never truncated; JSON reports files_reported/files_truncated.
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

        /// NOT IMPLEMENTED: ML-based scoring (aprender LinearRegression)
        ///
        /// The route destructured this as `ml: _` and built the config without
        /// it, so `--ml` returned exactly the heuristic weighted-sum scores
        /// under a promise of "trained ML models instead of heuristic weighted
        /// sums" — relabelling a number rather than changing it. It now errors,
        /// the same refusal `analyze complexity --ml` already makes (GH-97).
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
        /// Path to analyze (file or directory)
        #[arg(short = 'p', long, default_value = ".")]
        path: PathBuf,

        /// DEPRECATED: Use --path instead
        #[arg(long, hide = true)]
        project_path: Option<PathBuf>,

        /// Analyze a specific file instead of finding the hotspot
        #[arg(long)]
        file: Option<PathBuf>,

        /// Output format
        #[arg(short = 'f', long, value_enum, default_value = "summary")]
        format: LintHotspotOutputFormat,

        /// Maximum allowed defect density, in violations per line of code
        /// (0.05 = 5 violations per 100 SLOC)
        //
        // #699: the help used to say "violations per 100 lines" while
        // `check_quality_gates` compares against `violations / sloc`, and the
        // default was 5.0 — i.e. 500 violations per 100 lines, a threshold no
        // real file can reach, so the documented gate never fired. Observed on
        // a fixture whose hotspot measured defect_density 2.0 (200 violations
        // per 100 lines): `passed: true`, exit 0. 0.05 is the same threshold
        // the help text already promised, spelled in the unit actually used.
        #[arg(
            long,
            default_value_t = crate::cli::handlers::lint_hotspot_handlers::types::DEFAULT_MAX_DENSITY
        )]
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
        #[arg(short = 'f', long, value_enum, default_value = "human")]
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

    // ── Advanced Analysis ─────────────────────────────────────────
    // Provability, Duplicates, DefectPrediction
    /// Analyze provability properties using abstract interpretation
    Provability {
        /// Path to analyze (defaults to current directory)
        #[arg(long, short = 'p', default_value = ".")]
        path: PathBuf,

        /// DEPRECATED: Use --path instead
        #[arg(long, hide = true)]
        project_path: Option<PathBuf>,

        /// Specific functions to analyze (comma-separated)
        #[arg(long, value_delimiter = ',')]
        functions: Vec<String>,

        /// NOT IMPLEMENTED: analysis depth (number of iterations)
        ///
        /// There is no iteration to bound: `LightweightProvabilityAnalyzer`
        /// scores each function once from source patterns, and its
        /// `AbstractInterpreter::analyze_iteration` has no caller at all. Depth
        /// 0, 1, 10, 50 and 1000 produced one identical report (same sha256 of
        /// the JSON). It carried a clap default, so the value could not be
        /// refused; it is an `Option` now and the route refuses it rather than
        /// accept a knob wired to nothing.
        #[arg(long)]
        analysis_depth: Option<usize>,

        /// Output format
        #[arg(short = 'f', long, value_enum, default_value = "summary")]
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
        /// Path to analyze (defaults to current directory)
        #[arg(long, short = 'p', default_value = ".")]
        path: PathBuf,

        /// DEPRECATED: Use --path instead
        #[arg(long, hide = true)]
        project_path: Option<PathBuf>,

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
        /// Path to analyze (defaults to current directory)
        #[arg(long, short = 'p', default_value = ".")]
        path: PathBuf,

        /// DEPRECATED: Use --path instead
        #[arg(long, hide = true)]
        project_path: Option<PathBuf>,

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

        // The number must stay equal to
        // `services::facades::defect_prediction_facade::HIGH_RISK_PROBABILITY`.
        // It said "> 0.7" while the band it selects starts at 0.6, so the flag
        // returned files at 0.6069833 and 0.657 — both below the documented cut.
        /// Show only high-risk files (probability >= 0.6)
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

    // ── Metrics & Comprehensive ───────────────────────────────────
    // Comprehensive, GraphMetrics, NameSimilarity, ProofAnnotations
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
        /// Path to analyze (defaults to current directory)
        #[arg(long, short = 'p', default_value = ".")]
        path: PathBuf,

        /// DEPRECATED: Use --path instead
        #[arg(long, hide = true)]
        project_path: Option<PathBuf>,

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

        // `#[arg(long, default_value = "true")]` on a `bool` gives clap
        // ArgAction::SetTrue over a value that is already true: passing the
        // flag is a literal no-op and `--include-tdg=false` is *rejected*, so
        // five flags documented as "Enable X" could neither enable nor disable
        // anything. Taking a value (with the bare flag still meaning `=true`)
        // is what makes them switches.
        /// Enable duplicate detection analysis (`--include-duplicates=false` to disable)
        #[arg(
            long,
            action = clap::ArgAction::Set,
            num_args = 0..=1,
            require_equals = true,
            default_value = "true",
            default_missing_value = "true"
        )]
        include_duplicates: bool,

        /// Enable dead code analysis (`--include-dead-code=false` to disable)
        #[arg(
            long,
            action = clap::ArgAction::Set,
            num_args = 0..=1,
            require_equals = true,
            default_value = "true",
            default_missing_value = "true"
        )]
        include_dead_code: bool,

        /// Enable defect prediction analysis (`--include-defects=false` to disable)
        #[arg(
            long,
            action = clap::ArgAction::Set,
            num_args = 0..=1,
            require_equals = true,
            default_value = "true",
            default_missing_value = "true"
        )]
        include_defects: bool,

        /// Enable complexity analysis (`--include-complexity=false` to disable)
        #[arg(
            long,
            action = clap::ArgAction::Set,
            num_args = 0..=1,
            require_equals = true,
            default_value = "true",
            default_missing_value = "true"
        )]
        include_complexity: bool,

        /// Enable SATD analysis (`--include-tdg=false` to disable)
        ///
        /// NOTE: this switch drives SATD detection, not a Technical Debt
        /// Gradient run — comprehensive analysis has no TDG stage. The help
        /// text used to promise TDG results that never appeared in the report.
        #[arg(
            long,
            action = clap::ArgAction::Set,
            num_args = 0..=1,
            require_equals = true,
            default_value = "true",
            default_missing_value = "true"
        )]
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
        /// Path to analyze (defaults to current directory)
        #[arg(long, short = 'p', default_value = ".")]
        path: PathBuf,

        /// DEPRECATED: Use --path instead
        #[arg(long, hide = true)]
        project_path: Option<PathBuf>,

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
        /// Path to analyze (defaults to current directory)
        #[arg(long, short = 'p', default_value = ".")]
        path: PathBuf,

        /// DEPRECATED: Use --path instead
        #[arg(long, hide = true)]
        project_path: Option<PathBuf>,

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
        /// Path to analyze (defaults to current directory)
        #[arg(long, short = 'p', default_value = ".")]
        path: PathBuf,

        /// DEPRECATED: Use --path instead
        #[arg(long, hide = true)]
        project_path: Option<PathBuf>,

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

    // ── Coverage & Symbols ────────────────────────────────────────
    // IncrementalCoverage, CoverageImprove, SymbolTable, BigO
    /// Analyze incremental coverage changes with caching
    IncrementalCoverage {
        /// Path to analyze (defaults to current directory)
        #[arg(long, short = 'p', default_value = ".")]
        path: PathBuf,

        /// DEPRECATED: Use --path instead
        #[arg(long, hide = true)]
        project_path: Option<PathBuf>,

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

        /// Show detailed per-file coverage (shorthand for --format detailed)
        ///
        /// It was copied into `IncrementalCoverageRequest.detailed` and read by
        /// nothing — no analyzer and no renderer — so the flag was
        /// byte-identical to no flag in every format. It now upgrades the
        /// default `summary` report to the `detailed` one; an explicit
        /// `--format` other than `summary` still wins.
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
        /// Path to analyze (defaults to current directory)
        #[arg(long, short = 'p', default_value = ".")]
        path: PathBuf,

        /// DEPRECATED: Use --path instead
        #[arg(long, hide = true)]
        project_path: Option<PathBuf>,

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

        /// Maximum files to target per iteration (0 = no limit)
        #[arg(long, default_value = "10")]
        max_targets: usize,

        /// Output file path
        #[arg(short, long)]
        output: Option<PathBuf>,

        /// Output format
        #[arg(long, short = 'f', value_enum, default_value = "text")]
        format: CoverageImproveOutputFormat,
    },

    /// Analyze symbol table with cross-references and usage patterns
    SymbolTable {
        /// Path to analyze (defaults to current directory)
        #[arg(long, short = 'p', default_value = ".")]
        path: PathBuf,

        /// DEPRECATED: Use --path instead
        #[arg(long, hide = true)]
        project_path: Option<PathBuf>,

        /// Output format
        #[arg(long, short = 'f', value_enum, default_value = "summary")]
        format: SymbolTableOutputFormat,

        /// Filter by symbol type
        #[arg(long, value_enum)]
        filter: Option<SymbolTypeFilter>,

        /// Search query for specific symbols
        // No `short = 'q'`: the global `-q/--quiet` flag already owns it, and clap's
        // debug assertion ("Short option names must be unique") aborted
        // `pmat analyze symbol-table` outright in any debug build (#654).
        #[arg(long)]
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
        /// Path to analyze (defaults to current directory)
        #[arg(long, short = 'p', default_value = ".")]
        path: PathBuf,

        /// DEPRECATED: Use --path instead
        #[arg(long, hide = true)]
        project_path: Option<PathBuf>,

        /// Output format
        #[arg(long, short = 'f', value_enum, default_value = "summary")]
        format: BigOOutputFormat,

        /// Minimum confidence threshold (0-100)
        #[arg(long, default_value = "50")]
        confidence_threshold: u8,

        /// NO-OP: space complexity is always reported alongside time
        ///
        /// Read as "switches an extra analysis on"; nothing reads the flag past
        /// the config struct, and every renderer prints space complexity either
        /// way. Kept accepted so existing invocations do not break, but the
        /// help must not promise an analysis the flag does not enable.
        #[arg(long)]
        analyze_space: bool,

        /// Include file patterns
        #[arg(long)]
        include: Vec<String>,

        /// Exclude file patterns
        #[arg(long)]
        exclude: Vec<String>,

        /// Report only the O(n²)-or-worse rows of the distribution
        ///
        /// The listed functions are ALWAYS high-complexity — `build_report`
        /// selects them with the same predicate — so this narrows the
        /// distribution (the only part of the report that covers every class).
        /// It used to `retain` the already-filtered list and could not change a
        /// byte of any format.
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

    // ── WASM & Specialized ────────────────────────────────────────
    // AssemblyScript, WebAssembly, Clippy, Entropy, Wasm, DeepWasm,
    // Mutate, Cluster, Topics, Models
    /// Analyze `AssemblyScript` code
    AssemblyScript {
        /// Path to analyze (defaults to current directory)
        #[arg(long, short = 'p', default_value = ".")]
        path: PathBuf,

        /// DEPRECATED: Use --path instead
        #[arg(long, hide = true)]
        project_path: Option<PathBuf>,

        /// Output format
        #[arg(long, short = 'f', value_enum, default_value = "summary")]
        format: ComplexityOutputFormat,

        /// NO-OP: complexity is measured for every parsed file already
        ///
        /// It used to decide whether a parsed file appeared in the report at
        /// all, which is why the default run printed "Found 3 AssemblyScript
        /// files", three "Parsed:" lines, and then `files_analyzed: 0`. Fixing
        /// that left the flag with nothing to gate — every row already carries
        /// cyclomatic and cognitive — so it is accepted (existing invocations
        /// keep working) and the help no longer promises an analysis it
        /// switches on. Same disclosure as `analyze big-o --analyze-space`.
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
        /// Path to analyze (defaults to current directory)
        #[arg(long, short = 'p', default_value = ".")]
        path: PathBuf,

        /// DEPRECATED: Use --path instead
        #[arg(long, hide = true)]
        project_path: Option<PathBuf>,

        /// Output format
        #[arg(long, short = 'f', value_enum, default_value = "summary")]
        format: ComplexityOutputFormat,

        // Both were `#[arg(long, default_value = "true")]` on a `bool`, i.e.
        // clap SetTrue over an already-true value: `--include-binary` and
        // `--include-text` were permanently on and `=false` was rejected, so
        // neither flag could select or deselect a file kind.
        /// Include binary WASM (.wasm) files (`--include-binary=false` to exclude)
        #[arg(
            long,
            action = clap::ArgAction::Set,
            num_args = 0..=1,
            require_equals = true,
            default_value = "true",
            default_missing_value = "true"
        )]
        include_binary: bool,

        /// Include text WASM (.wat) files (`--include-text=false` to exclude)
        #[arg(
            long,
            action = clap::ArgAction::Set,
            num_args = 0..=1,
            require_equals = true,
            default_value = "true",
            default_missing_value = "true"
        )]
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
        /// Path to analyze (defaults to current directory)
        #[arg(long, short = 'p', default_value = ".")]
        path: PathBuf,

        /// DEPRECATED: Use --path instead
        #[arg(long, hide = true)]
        project_path: Option<PathBuf>,

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
        /// Path to analyze (file or directory)
        #[arg(short = 'p', long, default_value = ".")]
        path: PathBuf,

        /// DEPRECATED: Use --path instead
        #[arg(long, hide = true)]
        project_path: Option<PathBuf>,

        /// Output format
        #[arg(short = 'f', long, value_enum, default_value = "summary")]
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

/// Run `work` under the wall-clock budget a `--timeout` on this enum names.
///
/// This is the ONE implementation of what `--timeout` means. It lives beside
/// the flag declarations because the flags are what promise the bound: every
/// command that declares `timeout: u64` above must route its analysis through
/// here, or its `--timeout` is a number the user reads and nothing obeys.
/// Three separate handlers previously grew their own version and only one of
/// them worked (#929): `analyze complexity` printed "⏰ Analysis timeout set to
/// N seconds" and never enforced anything (measured: `--timeout 1` ran 8.1s and
/// exited 0), and the SATD copy wrapped `tokio::time::timeout` around a future
/// polled inline.
///
/// Why `tokio::spawn` and not a bare `tokio::time::timeout(budget, work)`:
/// `timeout` polls `work` on the CALLER's task, so a future that does not yield
/// — a synchronous walk or parse inside an `async fn` — never gives the timer a
/// chance to fire. That is exactly the non-enforcement #929 diagnosed. Moving
/// the work onto its own task lets the multi-threaded runtime preempt it, and
/// the timer runs on a thread the work cannot monopolise.
///
/// Caveat, stated rather than hidden: `abort()` cancels a task only at its next
/// await point, so work that is mid-CPU-burn keeps running until it yields. It
/// dies with the process, which exits on the error returned here. Work that
/// owns a CHILD PROCESS must additionally kill it — see
/// `CargoDeadCodeAnalyzer::wait_for_cargo_check`, where the budget is carried
/// into the analyzer so `cargo check` is actually killed.
///
/// `timeout_secs == 0` is a zero-length budget and fails immediately; it is not
/// a synonym for "no limit". The error says so rather than looking like a hang.
pub(crate) async fn run_within_analysis_budget<F, T>(
    what: &str,
    timeout_secs: u64,
    work: F,
) -> anyhow::Result<T>
where
    F: std::future::Future<Output = anyhow::Result<T>> + Send + 'static,
    T: Send + 'static,
{
    let budget = std::time::Duration::from_secs(timeout_secs);
    let task = tokio::spawn(work);
    let abort = task.abort_handle();

    match tokio::time::timeout(budget, task).await {
        Ok(joined) => joined.map_err(|e| anyhow::anyhow!("{what} panicked: {e}"))?,
        Err(_) => {
            // Dropping a `JoinHandle` DETACHES the task; only `abort` stops it.
            abort.abort();
            anyhow::bail!(
                "{what} timed out after {timeout_secs} seconds — re-run with a larger --timeout \
                 (--timeout 0 is a zero-length budget, not 'no limit')"
            )
        }
    }
}

#[cfg(test)]
mod analysis_budget_tests {
    //! `--timeout` must be a bound, not a banner. See
    //! `run_within_analysis_budget`.
    use super::run_within_analysis_budget;

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn work_that_finishes_inside_the_budget_returns_its_value() {
        let got: u32 = run_within_analysis_budget("Test analysis", 30, async { Ok(7) })
            .await
            .expect("work well inside the budget must not be cancelled");
        assert_eq!(got, 7);
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn work_that_outlives_the_budget_fails_and_names_the_budget() {
        let started = std::time::Instant::now();
        let err = run_within_analysis_budget("Test analysis", 1, async {
            tokio::time::sleep(std::time::Duration::from_secs(30)).await;
            Ok(())
        })
        .await
        .expect_err("30s of work under a 1s budget must not report success");

        let elapsed = started.elapsed();
        assert!(
            elapsed < std::time::Duration::from_secs(10),
            "the budget must cut the work short, took {elapsed:?}"
        );
        let msg = err.to_string();
        assert!(
            msg.contains("timed out after 1 seconds"),
            "the error must name the budget that was exceeded, got: {msg}"
        );
        assert!(
            msg.contains("--timeout"),
            "the error must name the knob that moves the budget, got: {msg}"
        );
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn a_zero_budget_fails_rather_than_meaning_unlimited() {
        let err = run_within_analysis_budget("Test analysis", 0, async {
            tokio::time::sleep(std::time::Duration::from_secs(30)).await;
            Ok(())
        })
        .await
        .expect_err("--timeout 0 must not silently mean 'no limit'");
        assert!(
            err.to_string().contains("timed out after 0 seconds"),
            "got: {err}"
        );
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn a_panicking_analysis_is_reported_as_a_panic_not_a_timeout() {
        let err = run_within_analysis_budget("Test analysis", 30, async {
            panic!("boom");
            #[allow(unreachable_code)]
            Ok(())
        })
        .await
        .expect_err("a panicking analysis must not be reported as success");
        let msg = err.to_string();
        assert!(msg.contains("panicked"), "got: {msg}");
        assert!(
            !msg.contains("timed out"),
            "a panic must not be dressed up as a timeout, got: {msg}"
        );
    }
}

#[cfg(test)]
mod include_flags_are_switches_tests {
    //! `#[arg(long, default_value = "true")]` on a `bool` makes clap emit
    //! `ArgAction::SetTrue` for a value that is already `true`: the flag is a
    //! no-op when passed and `--flag=false` is a parse error. Seven flags
    //! documented as "Enable/Include X" behaved that way — `analyze
    //! comprehensive --include-tdg --include-duplicates` produced byte-identical
    //! JSON to no flags at all, and `analyze web-assembly --include-text` could
    //! not stop `.wasm` files being collected.
    use super::AnalyzeCommands;
    use clap::Parser;

    #[derive(Parser)]
    struct Harness {
        #[command(subcommand)]
        cmd: AnalyzeCommands,
    }

    fn parse(args: &[&str]) -> AnalyzeCommands {
        // 8MB stack, on its own thread: clap's generated parser recurses deeply
        // enough over this command tree to overflow the default 2MB test stack,
        // the same reason every other clap parsing test in this crate spawns a
        // thread (see `cli::commands::tests_cli_parsing`). Parsing inline aborts
        // the whole test binary with SIGABRT — invisible under
        // `RUST_MIN_STACK=8388608`, which is how `pmat verify` runs the suite,
        // but fatal in CI's coverage job, which sets no such variable.
        let argv: Vec<String> = std::iter::once("pmat".to_string())
            .chain(args.iter().map(|s| (*s).to_string()))
            .collect();
        std::thread::Builder::new()
            .stack_size(8 * 1024 * 1024)
            .spawn(move || {
                Harness::try_parse_from(&argv)
                    .unwrap_or_else(|e| panic!("failed to parse {argv:?}: {e}"))
                    .cmd
            })
            .expect("spawn clap parse thread")
            .join()
            .expect("clap parse thread panicked")
    }

    fn comprehensive_flags(args: &[&str]) -> (bool, bool, bool, bool, bool) {
        match parse(args) {
            AnalyzeCommands::Comprehensive {
                include_duplicates,
                include_dead_code,
                include_defects,
                include_complexity,
                include_tdg,
                ..
            } => (
                include_duplicates,
                include_dead_code,
                include_defects,
                include_complexity,
                include_tdg,
            ),
            other => panic!("expected Comprehensive, got {other:?}"),
        }
    }

    fn wasm_flags(args: &[&str]) -> (bool, bool) {
        match parse(args) {
            AnalyzeCommands::WebAssembly {
                include_binary,
                include_text,
                ..
            } => (include_binary, include_text),
            other => panic!("expected WebAssembly, got {other:?}"),
        }
    }

    #[test]
    fn comprehensive_include_flags_default_on() {
        assert_eq!(
            comprehensive_flags(&["comprehensive"]),
            (true, true, true, true, true)
        );
    }

    #[test]
    fn comprehensive_include_flags_can_be_turned_off() {
        assert!(
            !comprehensive_flags(&["comprehensive", "--include-tdg=false"]).4,
            "--include-tdg=false must disable it"
        );
        assert!(
            !comprehensive_flags(&["comprehensive", "--include-duplicates=false"]).0,
            "--include-duplicates=false must disable it"
        );
        assert_eq!(
            comprehensive_flags(&[
                "comprehensive",
                "--include-dead-code=false",
                "--include-defects=false",
                "--include-complexity=false",
            ]),
            (true, false, false, false, true)
        );
    }

    #[test]
    fn comprehensive_bare_flag_still_means_enabled() {
        assert_eq!(
            comprehensive_flags(&["comprehensive", "--include-tdg", "--include-duplicates"]),
            (true, true, true, true, true)
        );
    }

    #[test]
    fn web_assembly_kind_flags_can_select_one_kind() {
        assert_eq!(wasm_flags(&["web-assembly"]), (true, true));
        assert_eq!(
            wasm_flags(&["web-assembly", "--include-binary=false"]),
            (false, true),
            "--include-binary=false must select .wat only"
        );
        assert_eq!(
            wasm_flags(&["web-assembly", "--include-text=false"]),
            (true, false),
            "--include-text=false must select .wasm only"
        );
    }
}

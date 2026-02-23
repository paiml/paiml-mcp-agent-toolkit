#![cfg_attr(coverage_nightly, coverage(off))]
// Misc command types - extracted for file health (CB-040)

use crate::cli::{OutputFormat, TdgOutputFormat};
use clap::Subcommand;
use std::path::PathBuf;
use super::config_hooks::{ConfigCommands, ConfigFormat};

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

/// Comply subcommands for PMAT compliance checking and migration
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

        /// Additional project paths to include in cross-stack health checks
        #[arg(long, value_name = "PATH")]
        include_project: Vec<PathBuf>,
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

    /// Upgrade project to a specific quality enforcement style (e.g., Popperian)
    Upgrade {
        /// Project path (defaults to current directory)
        #[arg(short = 'p', long = "path", default_value = ".")]
        path: PathBuf,

        /// Target style (e.g., "popperian")
        #[arg(short = 't', long = "target", default_value = "popperian")]
        target: String,

        /// Dry run (show what would be upgraded)
        #[arg(long)]
        dry_run: bool,
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

    /// Layer 2 (Genchi Genbutsu): Evidence-based review checklist (COMPLY-045)
    /// Generates a reviewer checklist with reproducibility, hypothesis, and trace evidence.
    Review {
        /// Project path (defaults to current directory)
        #[arg(short = 'p', long = "path", default_value = ".")]
        path: PathBuf,

        /// Output format
        #[arg(short = 'f', long = "format", value_enum, default_value = "markdown")]
        format: ComplyOutputFormat,

        /// Write output to file
        #[arg(short = 'o', long = "output")]
        output: Option<PathBuf>,
    },

    /// Layer 3 (Governance): Generate audit artifact with sovereign trail (COMPLY-045)
    /// Requires clean git state. Produces signed compliance evidence.
    Audit {
        /// Project path (defaults to current directory)
        #[arg(short = 'p', long = "path", default_value = ".")]
        path: PathBuf,

        /// Output format
        #[arg(short = 'f', long = "format", value_enum, default_value = "json")]
        format: ComplyOutputFormat,

        /// Write output to file
        #[arg(short = 'o', long = "output")]
        output: Option<PathBuf>,
    },

    /// Generate file health baseline for ratchet enforcement
    /// Scans source files, calculates health metrics, saves to .pmat/file-health-baseline.json
    Baseline {
        /// Project path (defaults to current directory)
        #[arg(short = 'p', long = "path", default_value = ".")]
        path: PathBuf,
    },

    /// Cross-crate duplication detection (CC-001 through CC-005)
    /// Detects copy-paste duplication, API divergence, and churn correlation across workspace crates.
    #[command(visible_aliases = &["cc", "xc"])]
    CrossCrate {
        /// Workspace root path (defaults to current directory)
        #[arg(short = 'p', long = "path", default_value = ".")]
        path: PathBuf,

        /// Explicit crate paths (comma-separated)
        #[arg(long = "crates", value_delimiter = ',', num_args = 1..)]
        crates: Option<Vec<PathBuf>>,

        /// Minimum similarity threshold for clone detection (0.0-1.0)
        #[arg(long = "similarity-threshold", default_value = "0.80")]
        similarity_threshold: f64,

        /// Window in days for churn correlation (CC-004)
        #[arg(long = "churn-window-days", default_value = "7")]
        churn_window_days: u32,

        /// Comma-separated list of rules to run (e.g., "cc001,cc002")
        #[arg(long = "rules")]
        rules: Option<String>,

        /// Output format
        #[arg(short = 'f', long = "format", value_enum, default_value = "text")]
        format: ComplyOutputFormat,

        /// Write output to file
        #[arg(short = 'o', long = "output")]
        output: Option<PathBuf>,

        /// Exit with error code 1 if findings detected
        #[arg(long)]
        strict: bool,

        /// Save current finding counts as ratchet baseline
        #[arg(long = "save-baseline")]
        save_baseline: bool,
    },
}

/// Comply output formats
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

/// Output format for kaizen command
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, clap::ValueEnum)]
pub enum KaizenOutputFormat {
    /// Human-readable text format
    #[default]
    Text,
    /// JSON format for CI/CD
    Json,
    /// Markdown report format
    Markdown,
}

/// Output format for work annotate command
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, clap::ValueEnum)]
pub enum AnnotateOutputFormat {
    /// Human-readable text format
    #[default]
    Text,
    /// JSON format for CI/CD
    Json,
    /// Markdown report format
    Markdown,
}

/// Debug subcommands for time-travel debugging
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

/// Quality gates subcommands
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

/// Maintain subcommands for project maintenance tasks
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

        /// Auto-generate missing ticket files from roadmap entries
        #[arg(long)]
        generate_tickets: bool,

        /// Dry-run mode (show changes without applying)
        #[arg(long)]
        dry_run: bool,

        /// Output format
        #[arg(long, value_enum, default_value = "table")]
        format: OutputFormat,
    },

    /// Validate project health (build, tests, coverage, complexity)
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

    /// Create bug report from captured error
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

    /// Clean up development artifacts and caches
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

    /// View TDG history at specific commits
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

    /// Manage TDG baselines for quality regression detection
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

    /// Start TDG web dashboard server
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

    /// Check for quality regressions against baseline
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

    /// Check files meet minimum quality thresholds
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

/// Baseline management subcommands
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

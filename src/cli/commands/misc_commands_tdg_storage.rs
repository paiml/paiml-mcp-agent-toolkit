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

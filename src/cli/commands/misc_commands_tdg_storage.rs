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

    /// [NOT AVAILABLE in the default build] Start TDG web dashboard server — needs --features http-server
    ///
    /// `http-server` is not in the default feature set, so a
    /// `cargo install pmat` binary exits rc=1 with "Dashboard requires the
    /// 'http-server' feature" and never binds `--port`/`--host`.
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

        /// Fail with non-zero exit code if files below threshold [default: true]
        ///
        /// A BOOL WITH `default_value = "true"` AND clap's implicit `SetTrue`
        /// action is unconditionally true: the gate fired whether or not the
        /// flag was typed, `--fail-on-violation` was indistinguishable from
        /// omitting it, and `--fail-on-violation=false` was REJECTED by clap
        /// ("unexpected value 'false'") — so the opt-out the help implied did
        /// not exist at all. Taking a value makes both directions reachable
        /// (`--fail-on-violation=false` reports the violations and exits 0)
        /// while a bare `--fail-on-violation` and an absent flag keep the
        /// CI-safe default this command is for.
        #[arg(
            long,
            num_args = 0..=1,
            default_value_t = true,
            default_missing_value = "true",
            action = clap::ArgAction::Set,
        )]
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

#[cfg(test)]
mod fail_on_violation_arity_tests {
    //! `--fail-on-violation` was `#[arg(long, default_value = "true")]` on a
    //! bare `bool`. clap gives that the `SetTrue` action, so the value was
    //! unconditionally `true`: typing the flag was indistinguishable from
    //! omitting it (`tdg check-quality` and `tdg check-quality
    //! --fail-on-violation` both exited 3 with byte-identical output on a corpus
    //! with 30 files below grade), and `--fail-on-violation=false` was REJECTED
    //! by clap with "unexpected value 'false'". The opt-out the help implied did
    //! not exist.

    fn fail_on_violation(args: &[&str]) -> bool {
        // 8MB stack on its own thread: clap's generated parser recurses deeply
        // enough over this command tree to overflow the default 2MB test stack.
        let argv: Vec<String> = std::iter::once("pmat".to_string())
            .chain(args.iter().map(|s| (*s).to_string()))
            .collect();
        std::thread::Builder::new()
            .stack_size(8 * 1024 * 1024)
            .spawn(move || {
                use crate::cli::commands::TdgCommand;
                use clap::Parser;
                let cli = crate::cli::Cli::try_parse_from(&argv)
                    .unwrap_or_else(|e| panic!("failed to parse {argv:?}: {e}"));
                match cli.command {
                    crate::cli::Commands::Tdg {
                        command:
                            Some(TdgCommand::CheckQuality {
                                fail_on_violation, ..
                            }),
                        ..
                    } => fail_on_violation,
                    other => panic!("expected tdg check-quality, got {other:?}"),
                }
            })
            .expect("spawn clap parse thread")
            .join()
            .expect("clap parse thread panicked")
    }

    #[test]
    fn the_ci_safe_default_is_still_on() {
        assert!(fail_on_violation(&["tdg", "check-quality"]));
    }

    #[test]
    fn a_bare_flag_still_asks_for_the_default() {
        assert!(fail_on_violation(&[
            "tdg",
            "check-quality",
            "--fail-on-violation"
        ]));
    }

    /// The direction that did not exist: clap used to exit 2 here.
    #[test]
    fn the_flag_can_be_turned_off() {
        assert!(!fail_on_violation(&[
            "tdg",
            "check-quality",
            "--fail-on-violation=false"
        ]));
        assert!(!fail_on_violation(&[
            "tdg",
            "check-quality",
            "--fail-on-violation",
            "false"
        ]));
    }
}

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
///
/// The per-subcommand path is `Option<PathBuf>` with NO `default_value`. It used
/// to default to `"."`, which silently overrode the top-level `cuda-tdg [PATH]`
/// positional that `--help` documents: `pmat cuda-tdg /does/not/exist score`
/// graded the current directory and exited 0. `None` now means "fall back to the
/// top-level path" — see `resolve_subcommand_path`.
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
        /// Path to analyze (defaults to the top-level `cuda-tdg [PATH]`, then `.`)
        path: Option<PathBuf>,

        /// Show detailed category breakdown
        #[arg(long)]
        breakdown: bool,
    },

    /// Generate detailed defect report
    Report {
        /// Path to analyze (defaults to the top-level `cuda-tdg [PATH]`, then `.`)
        path: Option<PathBuf>,

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
        /// Path to analyze (defaults to the top-level `cuda-tdg [PATH]`, then `.`)
        path: Option<PathBuf>,

        /// Minimum score to pass (0-100)
        #[arg(long, default_value = "85")]
        min_score: f64,

        /// Fail on P0 defects
        #[arg(long)]
        fail_on_p0: bool,
    },

    /// Generate Kaizen continuous improvement report
    Kaizen {
        /// Path to analyze (defaults to the top-level `cuda-tdg [PATH]`, then `.`)
        path: Option<PathBuf>,

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

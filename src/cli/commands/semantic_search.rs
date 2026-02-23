#![cfg_attr(coverage_nightly, coverage(off))]
// Semantic search commands - extracted for file health (CB-040)

use crate::cli::OutputFormat;
#[cfg(feature = "mutation-testing")]
use clap::Args;
use clap::Subcommand;
use std::path::PathBuf;

/// Embed subcommands for semantic search
#[derive(Subcommand)]
#[cfg_attr(test, derive(Debug))]
#[command(after_help = "EXAMPLES:
# Sync embeddings for current codebase
pmat embed sync

# Check embedding database status
pmat embed status

# Clear all embeddings (requires confirmation)
pmat embed clear --confirm

# Sync with verbose output
pmat embed sync --verbose

# Check status in JSON format
pmat embed status --format json")]
pub enum EmbedCommands {
    /// Sync embeddings for codebase
    Sync {
        /// Path to analyze
        #[arg(short, long, default_value = ".")]
        path: PathBuf,

        /// Filter by language
        #[arg(long)]
        language: Option<String>,

        /// Output format
        #[arg(long, value_enum, default_value = "table")]
        format: OutputFormat,
    },

    /// Show embedding database status
    Status {
        /// Output format
        #[arg(long, value_enum, default_value = "table")]
        format: OutputFormat,
    },

    /// Clear all embeddings (requires --confirm)
    Clear {
        /// Confirm deletion
        #[arg(long)]
        confirm: bool,
    },
}

/// Semantic search subcommands
#[derive(Subcommand)]
#[cfg_attr(test, derive(Debug))]
pub enum SemanticCommands {
    /// Search code by natural language query
    Search {
        /// Natural language query
        query: String,

        /// Search mode
        #[arg(long, value_enum, default_value = "hybrid")]
        mode: SearchMode,

        /// Filter by language
        #[arg(long)]
        language: Option<String>,

        /// Max results to return
        #[arg(long, default_value_t = 10)]
        limit: usize,

        /// Output format
        #[arg(long, value_enum, default_value = "summary")]
        format: OutputFormat,
    },

    /// Find code files similar to a reference file
    Similar {
        /// Reference file path
        file_path: PathBuf,

        /// Max results to return
        #[arg(long, default_value_t = 10)]
        limit: usize,

        /// Output format
        #[arg(long, value_enum, default_value = "summary")]
        format: OutputFormat,
    },
}

/// Search mode for semantic search
#[derive(Clone, Debug, clap::ValueEnum, PartialEq)]
pub enum SearchMode {
    /// Keyword-only search (ripgrep)
    Keyword,
    /// Vector-only search (semantic similarity)
    Vector,
    /// Hybrid search (keyword + vector with RRF)
    Hybrid,
}

/// Clustering method for semantic analysis
#[derive(Clone, Debug, clap::ValueEnum, PartialEq)]
pub enum ClusterMethod {
    /// K-means clustering
    Kmeans,
    /// Hierarchical clustering
    Hierarchical,
    /// DBSCAN density-based clustering
    Dbscan,
}

/// Mutation testing arguments
#[cfg(feature = "mutation-testing")]
#[derive(Args, Debug, Clone)]
#[cfg_attr(test, derive(PartialEq))]
pub struct MutateArgs {
    /// File or directory to mutate
    #[arg(short, long, value_name = "PATH")]
    pub target: PathBuf,

    /// Programming language (rust, python, typescript, go, cpp)
    #[arg(short, long)]
    pub language: Option<String>,

    /// Timeout per mutant in seconds
    #[arg(long, default_value = "30")]
    pub timeout: u64,

    /// Parallel execution workers
    #[arg(short, long)]
    pub jobs: Option<usize>,

    /// Output format (json, markdown, text)
    #[arg(short = 'f', long, default_value = "text")]
    pub output_format: String,

    /// Output file (stdout if omitted)
    #[arg(short, long)]
    pub output: Option<PathBuf>,

    /// Mutation score threshold (fail if below)
    #[arg(long)]
    pub threshold: Option<f64>,

    /// Show only failures (survived mutants, compile errors, timeouts)
    #[arg(long, default_value = "false")]
    pub failures_only: bool,

    // Sprint 70: cargo-mutants backend options
    /// Use cargo-mutants backend for Rust mutation testing (requires cargo-mutants v24.7.0+)
    ///
    /// Provides comprehensive Rust mutation testing using the industry-standard cargo-mutants tool.
    /// Automatically detects cargo-mutants installation and validates version compatibility.
    ///
    /// Example: pmat mutate --use-cargo-mutants --timeout 600
    /// Guide: docs/user-guides/cargo-mutants-integration.md
    #[arg(long)]
    pub use_cargo_mutants: bool,

    /// Cargo features to enable for mutation testing (comma-separated)
    ///
    /// Only applies when --use-cargo-mutants is specified.
    /// Enables specific Cargo features during mutation testing.
    ///
    /// Example: --features "serde,logging"
    #[arg(long, value_delimiter = ',')]
    pub features: Option<Vec<String>>,

    /// Enable all Cargo features during mutation testing
    ///
    /// Only applies when --use-cargo-mutants is specified.
    /// Equivalent to cargo test --all-features.
    ///
    /// Example: --use-cargo-mutants --all-features
    #[arg(long)]
    pub all_features: bool,

    /// Disable default Cargo features during mutation testing
    ///
    /// Only applies when --use-cargo-mutants is specified.
    /// Equivalent to cargo test --no-default-features.
    ///
    /// Example: --use-cargo-mutants --no-default-features --features "minimal"
    #[arg(long)]
    pub no_default_features: bool,

    /// Don't shuffle mutant execution order (deterministic results)
    ///
    /// Only applies when --use-cargo-mutants is specified.
    /// Mutants will be tested in sequential order for reproducible results.
    ///
    /// Example: --use-cargo-mutants --no-shuffle
    #[arg(long)]
    pub no_shuffle: bool,
}

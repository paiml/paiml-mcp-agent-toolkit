#![cfg_attr(coverage_nightly, coverage(off))]
// Config and Hooks commands - extracted for file health (CB-040)

use crate::cli::OutputFormat;
use clap::Subcommand;

/// Configuration management commands
#[derive(Subcommand, Clone)]
#[cfg_attr(test, derive(Debug))]
pub enum ConfigCommands {
    /// Show complete configuration
    Show {
        /// Output format
        #[arg(long, value_enum, default_value = "json")]
        format: ConfigFormat,
    },

    /// Get specific configuration value
    Get {
        /// Configuration key path (e.g., `hooks.quality_gates.max_cyclomatic_complexity`)
        key: String,
    },

    /// Validate configuration file
    Validate {
        /// Fix configuration issues automatically
        #[arg(long)]
        fix: bool,
    },

    /// Show configuration source hierarchy
    Sources,
}

/// Pre-commit hook management commands  
#[derive(Subcommand, Clone)]
#[cfg_attr(test, derive(Debug))]
pub enum HooksCommands {
    /// Initialize pre-commit hooks (alias for install)
    Init {
        /// Enable interactive mode for configuration
        #[arg(long)]
        interactive: bool,

        /// Force installation (overwrite existing)
        #[arg(long)]
        force: bool,

        /// Create backup of existing hooks
        #[arg(long, default_value = "true")]
        backup: bool,

        /// Enable TDG quality enforcement hooks
        #[arg(long)]
        tdg_enforcement: bool,
    },

    /// Install or update pre-commit hooks
    Install {
        /// Enable interactive mode for configuration
        #[arg(long)]
        interactive: bool,

        /// Force installation (overwrite existing)
        #[arg(long)]
        force: bool,

        /// Create backup of existing hooks
        #[arg(long, default_value = "true")]
        backup: bool,

        /// Enable TDG quality enforcement hooks
        #[arg(long)]
        tdg_enforcement: bool,
    },

    /// Remove PMAT-managed hooks
    Uninstall {
        /// Restore backup if available
        #[arg(long)]
        restore_backup: bool,
    },

    /// Show hook installation status
    Status,

    /// Verify hooks work with current configuration
    Verify {
        /// Fix issues automatically
        #[arg(long)]
        fix: bool,
    },

    /// Regenerate hooks from current configuration
    Refresh,

    /// Run pre-commit hooks (for CI/CD integration)
    Run {
        /// Run on all files instead of just staged
        #[arg(long)]
        all_files: bool,

        /// Verbose output
        #[arg(long, short)]
        verbose: bool,

        /// Enable O(1) cache check (skip if unchanged)
        #[arg(long, default_value = "true")]
        cache: bool,
    },

    /// O(1) cache management for hooks
    Cache {
        #[command(subcommand)]
        action: HooksCacheAction,
    },
}

/// Cache actions for O(1) hooks
#[derive(Subcommand, Clone)]
#[cfg_attr(test, derive(Debug))]
pub enum HooksCacheAction {
    /// Initialize cache directory structure
    Init,

    /// Show cache status and metrics
    Status {
        /// Output format
        #[arg(long, value_enum, default_value = "table")]
        format: OutputFormat,
    },

    /// Clear cache (forces full re-run on next commit)
    Clear {
        /// Clear specific gate cache only
        #[arg(long)]
        gate: Option<String>,
    },

    /// Show detailed metrics (hit rate, timing)
    Metrics {
        /// Output format
        #[arg(long, value_enum, default_value = "table")]
        format: OutputFormat,
    },
}

/// Configuration output format
#[derive(Debug, Clone, clap::ValueEnum)]
#[cfg_attr(test, derive(PartialEq))]
pub enum ConfigFormat {
    /// JSON format
    Json,
    /// TOML format
    Toml,
    /// Environment variables format
    Env,
}

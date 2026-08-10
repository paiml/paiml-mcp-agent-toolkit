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

        /// Install hooks across all sovereign AI stack repos
        #[arg(long)]
        stack: bool,

        /// Update existing hooks when using --stack (overwrite)
        #[arg(long, requires = "stack")]
        update: bool,
    },

    /// Remove PMAT-managed hooks
    Uninstall {
        /// Restore backup if available
        #[arg(long)]
        restore_backup: bool,
    },

    /// Show hook installation status
    Status {
        /// Show status across all sovereign AI stack repos
        #[arg(long)]
        stack: bool,

        /// Output format for stack status
        #[arg(long, value_enum, default_value = "table")]
        format: OutputFormat,
    },

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
        ///
        /// This carried `default_value = "true"` with no `--no-cache`, so the
        /// cache was consulted whether or not you asked for it: a bare
        /// `hooks run --all-files` answered "All quality gates passed (cached)"
        /// without running a single gate, and nothing short of
        /// `hooks cache clear` could force a real run. It is opt-in, as the
        /// help text has always said.
        #[arg(long)]
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

#[cfg(test)]
mod hooks_run_cache_flag_tests {
    //! Regression: `hooks run` declared `--cache` with `default_value = "true"`
    //! and shipped no `--no-cache`, so the cache was consulted even when the
    //! user did not ask for it and a bare run answered "(cached)" without
    //! running any gate.
    use crate::cli::commands::{Commands, HooksCommands};
    use clap::Parser;

    fn parse(args: &[&str]) -> crate::cli::Cli {
        let owned: Vec<String> = args.iter().map(|s| (*s).to_string()).collect();
        std::thread::Builder::new()
            .stack_size(8 * 1024 * 1024)
            .spawn(move || crate::cli::Cli::try_parse_from(&owned).expect("clap accepts these"))
            .expect("spawn")
            .join()
            .expect("thread panicked")
    }

    fn cache_flag(cli: &crate::cli::Cli) -> bool {
        match &cli.command {
            Commands::Hooks(HooksCommands::Run { cache, .. }) => *cache,
            other => panic!("expected `hooks run`, got {other:?}"),
        }
    }

    #[test]
    fn test_hooks_run_cache_is_opt_in() {
        assert!(
            !cache_flag(&parse(&["pmat", "hooks", "run", "--all-files"])),
            "omitting --cache must run the gates for real, not answer from cache"
        );
    }

    #[test]
    fn test_hooks_run_cache_flag_enables_cache() {
        assert!(cache_flag(&parse(&[
            "pmat",
            "hooks",
            "run",
            "--all-files",
            "--cache"
        ])));
    }
}

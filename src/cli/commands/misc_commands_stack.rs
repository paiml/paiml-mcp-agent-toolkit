/// Stack subcommands: cross-repo dependency coordination for the sovereign AI stack
#[derive(Debug, Clone, Subcommand)]
#[cfg_attr(test, derive(PartialEq))]
pub enum StackCommands {
    /// Show dependency status across all stack repos
    #[command(visible_aliases = &["st"])]
    Status {
        /// Output format
        #[arg(short = 'f', long, value_enum, default_value = "table")]
        format: OutputFormat,
    },
    /// Sync dependency versions across repos
    #[command(visible_aliases = &["s"])]
    Sync {
        /// Actually apply changes (default is dry-run)
        #[arg(long)]
        apply: bool,
        /// Preview changes without applying
        #[arg(long)]
        dry_run: bool,
    },
    /// Generate SQI-compliant boilerplate files for repo hygiene
    #[command(visible_aliases = &["sc"])]
    Scaffold {
        /// Apply to all stack repos (default: current repo only)
        #[arg(long)]
        all: bool,
        /// Use specific template set
        #[arg(long, default_value = "sqi-a-minus")]
        template: String,
        /// Show what would be added/changed without writing
        #[arg(long)]
        diff: bool,
        /// Overwrite existing files
        #[arg(long)]
        force: bool,
    },
}

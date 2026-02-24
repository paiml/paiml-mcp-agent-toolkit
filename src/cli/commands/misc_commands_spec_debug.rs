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

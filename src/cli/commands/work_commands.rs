// Work commands - extracted for file health (CB-040)
/// Work subcommands for unified GitHub/YAML workflow
/// CRUD: Create (add), Read (list/status), Update (edit/start/complete), Delete (delete)
#[derive(Debug, Clone, Subcommand)]
pub enum WorkCommands {
    /// Add a new work ticket (CREATE)
    #[command(visible_aliases = &["new", "create", "a"])]
    Add {
        /// Ticket title (required)
        title: String,

        /// Description (optional)
        #[arg(short, long)]
        description: Option<String>,

        /// Priority level
        #[arg(short, long, value_enum, default_value = "medium")]
        priority: WorkPriority,

        /// Tags (comma-separated)
        #[arg(short, long)]
        tags: Option<String>,

        /// Project path (default: current directory)
        #[arg(long)]
        path: Option<PathBuf>,

        /// Also create GitHub issue
        #[arg(long)]
        github: bool,
    },

    /// List all work tickets (READ)
    #[command(visible_aliases = &["ls", "l"])]
    List {
        /// Filter by status
        #[arg(short, long)]
        status: Option<String>,

        /// Filter by priority
        #[arg(long)]
        priority: Option<WorkPriority>,

        /// Show only count
        #[arg(long)]
        count: bool,

        /// Project path (default: current directory)
        #[arg(short, long)]
        path: Option<PathBuf>,
    },

    /// Edit an existing ticket (UPDATE)
    #[command(visible_aliases = &["update", "e"])]
    Edit {
        /// Ticket ID to edit
        id: String,

        /// New title
        #[arg(short, long)]
        title: Option<String>,

        /// New description
        #[arg(short, long)]
        description: Option<String>,

        /// New priority
        #[arg(long)]
        priority: Option<WorkPriority>,

        /// New status
        #[arg(short, long)]
        status: Option<String>,

        /// New tags (comma-separated, replaces existing)
        #[arg(long)]
        tags: Option<String>,

        /// Project path (default: current directory)
        #[arg(short, long)]
        path: Option<PathBuf>,
    },

    /// Delete a work ticket (DELETE)
    #[command(visible_aliases = &["rm", "remove", "del"])]
    Delete {
        /// Ticket ID to delete
        id: String,

        /// Skip confirmation prompt
        #[arg(short, long)]
        force: bool,

        /// Project path (default: current directory)
        #[arg(short, long)]
        path: Option<PathBuf>,
    },

    /// Show unified quality annotations for a ticket
    #[command(visible_aliases = &["ann", "quality", "metrics"])]
    Annotate {
        /// Ticket ID to annotate
        id: String,

        /// Project path (default: current directory)
        #[arg(short, long)]
        path: Option<PathBuf>,

        /// Output format
        #[arg(short, long, value_enum, default_value = "text")]
        format: AnnotateOutputFormat,

        /// Include churn analysis (slower)
        #[arg(long)]
        with_churn: bool,

        /// Days for churn analysis
        #[arg(long, default_value = "30")]
        churn_days: u32,
    },

    /// Start work on a GitHub issue or YAML ticket
    #[command(visible_aliases = &["begin", "s"])]
    Start {
        /// Issue number (e.g., "8", "42") or YAML ticket ID (e.g., "PERF-001")
        id: String,

        /// Create specification file (docs/specifications/NNN-name.md)
        #[arg(long)]
        with_spec: bool,

        /// Create as epic with subtasks
        #[arg(long)]
        epic: bool,

        /// Project path (default: current directory)
        #[arg(short, long)]
        path: Option<PathBuf>,

        /// Force create GitHub issue for YAML ticket
        #[arg(long)]
        create_github: bool,
    },

    /// Continue work on existing issue/ticket
    #[command(visible_aliases = &["cont", "c", "resume"])]
    Continue {
        /// Issue number or ticket ID
        id: String,

        /// Project path (default: current directory)
        #[arg(short, long)]
        path: Option<PathBuf>,
    },

    /// Complete work on issue/ticket
    #[command(visible_aliases = &["done", "finish", "f"])]
    Complete {
        /// Issue number or ticket ID
        id: String,

        /// Skip quality gates (not recommended, falsification still runs)
        #[arg(long)]
        skip_quality: bool,

        /// Override specific falsification claims (requires --ticket)
        /// Use claim names like: coverage, complexity, file-size, github-sync
        #[arg(long, value_delimiter = ',')]
        override_claims: Option<Vec<String>>,

        /// Ticket ID for override accountability (MANDATORY with --override-claims)
        /// Must reference a valid debt ticket (e.g., DEBT-COV-20240115)
        #[arg(long)]
        ticket: Option<String>,

        /// Project path (default: current directory)
        #[arg(short, long)]
        path: Option<PathBuf>,
    },

    /// Show work status
    #[command(visible_aliases = &["st", "stat"])]
    Status {
        /// Issue number or ticket ID (default: all)
        id: Option<String>,

        /// Project path (default: current directory)
        #[arg(short, long)]
        path: Option<PathBuf>,

        /// Show only active items
        #[arg(long)]
        active: bool,
    },

    /// Synchronize GitHub and YAML
    #[command(visible_aliases = &["sy"])]
    Sync {
        /// Sync direction
        #[arg(long, value_enum, default_value = "full")]
        direction: SyncDirection,

        /// Project path (default: current directory)
        #[arg(short, long)]
        path: Option<PathBuf>,

        /// Dry run (show what would be synced)
        #[arg(long)]
        dry_run: bool,
    },

    /// Initialize roadmap and hooks
    #[command(visible_aliases = &["setup", "ini"])]
    Init {
        /// GitHub repository (owner/repo)
        #[arg(long)]
        github_repo: Option<String>,

        /// Disable GitHub integration
        #[arg(long)]
        no_github: bool,

        /// Project path (default: current directory)
        #[arg(short, long)]
        path: Option<PathBuf>,
    },

    /// Validate roadmap.yaml syntax and content (Part B: UX Improvements)
    #[command(visible_aliases = &["check", "lint", "v"])]
    Validate {
        /// Project path (default: current directory)
        #[arg(short, long)]
        path: Option<PathBuf>,

        /// Show verbose output with suggestions
        #[arg(long)]
        verbose: bool,

        /// Fix issues automatically where possible
        #[arg(long)]
        fix: bool,
    },

    /// Auto-fix common roadmap.yaml issues (Part B: UX Improvements)
    #[command(visible_aliases = &["fix", "m"])]
    Migrate {
        /// Project path (default: current directory)
        #[arg(short, long)]
        path: Option<PathBuf>,

        /// Dry run (show what would be changed)
        #[arg(long)]
        dry_run: bool,

        /// Create backup before migration
        #[arg(long, default_value = "true")]
        backup: bool,
    },

    /// List all valid status values with descriptions
    #[command(visible_aliases = &["values", "statuses"])]
    ListStatuses,
}

/// Sync direction for work sync command
#[derive(Debug, Clone, Copy, clap::ValueEnum, PartialEq)]
pub enum SyncDirection {
    /// Sync YAML → GitHub
    YamlToGithub,
    /// Sync GitHub → YAML
    GithubToYaml,
    /// Full bidirectional sync
    Full,
}

/// Work priority for CLI (maps to roadmap::Priority)
#[derive(Debug, Clone, Copy, clap::ValueEnum, PartialEq, Default)]
pub enum WorkPriority {
    /// Low priority
    Low,
    /// Medium priority (default)
    #[default]
    Medium,
    /// High priority
    High,
    /// Critical priority
    Critical,
}

impl WorkPriority {
    /// Convert to roadmap Priority
    pub fn to_roadmap_priority(self) -> crate::models::roadmap::Priority {
        match self {
            WorkPriority::Low => crate::models::roadmap::Priority::Low,
            WorkPriority::Medium => crate::models::roadmap::Priority::Medium,
            WorkPriority::High => crate::models::roadmap::Priority::High,
            WorkPriority::Critical => crate::models::roadmap::Priority::Critical,
        }
    }
}

/// QA Work subcommands for Toyota Way quality validation
#[derive(Debug, Clone, Subcommand)]
pub enum QaWorkCommands {
    /// Generate QA checklist for a task
    #[command(visible_aliases = &["checklist", "cl"])]
    GenerateChecklist {
        /// Task/ticket ID (GitHub issue number or YAML ticket ID)
        task_id: String,

        /// Task type for checklist customization
        #[arg(long, value_enum, default_value = "feature")]
        task_type: QaTaskType,

        /// Project path (default: current directory)
        #[arg(short = 'p', long = "path", default_value = ".")]
        path: PathBuf,

        /// Output file for checklist (YAML format)
        #[arg(short = 'o', long = "output")]
        output: Option<PathBuf>,
    },

    /// Run automated QA validation
    #[command(visible_aliases = &["check", "v"])]
    Validate {
        /// Task/ticket ID to validate
        task_id: String,

        /// Project path (default: current directory)
        #[arg(short = 'p', long = "path", default_value = ".")]
        path: PathBuf,

        /// Fail on any warning (strict mode)
        #[arg(long)]
        strict: bool,

        /// Output format
        #[arg(short = 'f', long = "format", value_enum, default_value = "text")]
        format: QaOutputFormat,
    },

    /// Generate QA report for audit trail
    #[command(visible_aliases = &["r"])]
    Report {
        /// Task/ticket ID for report
        task_id: String,

        /// Project path (default: current directory)
        #[arg(short = 'p', long = "path", default_value = ".")]
        path: PathBuf,

        /// Include evidence (coverage reports, test results)
        #[arg(long)]
        with_evidence: bool,

        /// Output file for report
        #[arg(short = 'o', long = "output")]
        output: Option<PathBuf>,

        /// Output format
        #[arg(short = 'f', long = "format", value_enum, default_value = "markdown")]
        format: QaOutputFormat,
    },

    /// Show QA status summary
    #[command(visible_aliases = &["st", "status"])]
    Summary {
        /// Task/ticket ID (optional, shows all if omitted)
        task_id: Option<String>,

        /// Project path (default: current directory)
        #[arg(short = 'p', long = "path", default_value = ".")]
        path: PathBuf,

        /// Show epic summary (aggregate all tasks in epic)
        #[arg(long)]
        epic: Option<String>,
    },

    /// Generate example scripts for a feature (V2)
    #[command(visible_aliases = &["examples", "ex"])]
    GenerateExamples {
        /// Task/ticket ID
        task_id: String,

        /// Feature/command name for examples
        #[arg(short = 'n', long = "name")]
        feature_name: String,

        /// Project path (default: current directory)
        #[arg(short = 'p', long = "path", default_value = ".")]
        path: PathBuf,

        /// Output directory for examples
        #[arg(short = 'o', long = "output")]
        output: Option<PathBuf>,
    },

    /// Validate specification with 100-point Popperian falsifiability scoring (Part D & E)
    ///
    /// Parses markdown specifications and validates claims through evidence.
    /// All claims are FALSE until PROVEN true (Popperian epistemology).
    #[command(visible_aliases = &["popper"])]
    Spec {
        /// Specification file or ticket ID (e.g., "docs/specifications/foo.md" or "GH-118")
        target: String,

        /// Project path (default: current directory)
        #[arg(short = 'p', long = "path", default_value = ".")]
        path: PathBuf,

        /// Run full validation (includes mutation testing)
        #[arg(long)]
        full: bool,

        /// Output format
        #[arg(short = 'f', long = "format", value_enum, default_value = "text")]
        format: QaOutputFormat,

        /// Output file for results
        #[arg(short = 'o', long = "output")]
        output: Option<PathBuf>,

        /// Fail if total score below threshold (default: 60 for gateway)
        #[arg(long, default_value = "60")]
        threshold: u32,

        /// Fail if gateway category (Falsifiability) below threshold
        #[arg(long, default_value = "15")]
        gateway_threshold: u32,
    },
}

/// Task type for QA checklist customization
#[derive(Debug, Clone, Copy, clap::ValueEnum, PartialEq)]
pub enum QaTaskType {
    /// New feature implementation
    Feature,
    /// Bug fix
    Bugfix,
    /// Code refactoring
    Refactor,
    /// Documentation update
    Docs,
    /// Performance optimization
    Performance,
    /// Security fix
    Security,
}

/// Output format for QA commands
#[derive(Debug, Clone, Copy, clap::ValueEnum, PartialEq)]
pub enum QaOutputFormat {
    /// Human-readable text
    Text,
    /// JSON for CI/CD
    Json,
    /// YAML config format
    Yaml,
    /// Markdown documentation
    Markdown,
}

/// Test discovery subcommands for systematic test fixing
#[derive(Debug, Clone, Subcommand)]
pub enum TestDiscoveryCommands {
    /// Discover all test failures in workspace
    #[command(visible_aliases = &["d"])]
    Run {
        /// Project path (defaults to current directory)
        #[arg(short = 'p', long = "path", default_value = ".")]
        path: PathBuf,

        /// Output file for failures JSON
        #[arg(short = 'o', long = "output", default_value = "test-failures.json")]
        output: PathBuf,

        /// Use cargo nextest (faster, parallel)
        #[arg(long, default_value = "true")]
        use_nextest: bool,

        /// Maximum test timeout in seconds
        #[arg(long, default_value = "600")]
        timeout: u64,
    },

    /// Categorize test failures by root cause
    #[command(visible_aliases = &["cat"])]
    Categorize {
        /// Input failures JSON from discovery
        #[arg(short = 'i', long = "input")]
        input: PathBuf,

        /// Output categories JSON
        #[arg(short = 'o', long = "output", default_value = "test-categories.json")]
        output: PathBuf,
    },

    /// Mark tests as #[ignore] with reasons
    #[command(visible_aliases = &["m"])]
    Mark {
        /// Input categories JSON
        #[arg(short = 'i', long = "input")]
        input: PathBuf,

        /// Actually apply changes (default: dry-run)
        #[arg(long)]
        apply: bool,
    },

    /// Verify all tests pass after marking
    #[command(visible_aliases = &["v"])]
    Verify {
        /// Project path
        #[arg(short = 'p', long = "path", default_value = ".")]
        path: PathBuf,
    },

    /// Create GitHub issues from categorized test failures (Phase 5)
    #[command(visible_aliases = &["tickets", "t"])]
    CreateTickets {
        /// Input categories JSON
        #[arg(short = 'i', long = "input")]
        input: PathBuf,

        /// Actually create GitHub issues (default: dry-run)
        #[arg(long)]
        create: bool,

        /// Output tickets summary
        #[arg(short = 'o', long = "output")]
        output: Option<PathBuf>,

        /// GitHub repository (owner/repo format)
        #[arg(long)]
        repo: Option<String>,

        /// Labels to add to created issues
        #[arg(long, value_delimiter = ',')]
        labels: Option<Vec<String>>,
    },

    /// Resolve test file paths from test names
    #[command(visible_aliases = &["resolve", "r"])]
    ResolvePaths {
        /// Input failures JSON
        #[arg(short = 'i', long = "input")]
        input: PathBuf,

        /// Output with resolved paths
        #[arg(short = 'o', long = "output")]
        output: PathBuf,

        /// Project path to search for test files
        #[arg(short = 'p', long = "path", default_value = ".")]
        path: PathBuf,
    },
}

/// Test discovery output formats
#[derive(Debug, Clone, clap::ValueEnum, PartialEq)]
pub enum TestDiscoveryFormat {
    Json,
    Markdown,
    Text,
}

#[cfg(test)]
mod work_commands_tests {
    use super::*;
    use crate::models::roadmap::Priority;

    #[test]
    fn test_to_roadmap_priority_low() {
        assert!(matches!(WorkPriority::Low.to_roadmap_priority(), Priority::Low));
    }

    #[test]
    fn test_to_roadmap_priority_medium() {
        assert!(matches!(WorkPriority::Medium.to_roadmap_priority(), Priority::Medium));
    }

    #[test]
    fn test_to_roadmap_priority_high() {
        assert!(matches!(WorkPriority::High.to_roadmap_priority(), Priority::High));
    }

    #[test]
    fn test_to_roadmap_priority_critical() {
        assert!(matches!(WorkPriority::Critical.to_roadmap_priority(), Priority::Critical));
    }
}

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

        /// DbC contract profile override (universal, rust, pmat)
        #[arg(long)]
        profile: Option<String>,

        /// Exclude specific DbC claims (comma-separated, e.g. "ensure.coverage,ensure.supply_chain")
        #[arg(long, value_delimiter = ',')]
        without: Option<Vec<String>>,

        /// Iteration number for subcontracting (inherits postconditions from prior iteration)
        #[arg(long, default_value = "1")]
        iteration: u32,
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

    /// Run invariant checkpoint (DbC §4.2)
    #[command(visible_aliases = &["ck", "cp"])]
    Checkpoint {
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

    /// Run falsification tests without completing the work item
    #[command(visible_alias = "test-claims")]
    Falsify {
        /// Issue number or ticket ID
        id: String,

        /// Override specific falsification claims (requires --ticket)
        #[arg(long, value_delimiter = ',')]
        override_claims: Option<Vec<String>>,

        /// Ticket ID for override accountability (MANDATORY with --override-claims)
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

    /// Score a work contract (DBC spec 5-dimension quality + lint)
    #[command(visible_aliases = &["sc", "quality-score"])]
    Score {
        /// Work item ID
        id: String,

        /// Minimum score threshold (0.0-1.0, default: 0.0)
        #[arg(long, default_value = "0.0")]
        min_score: f64,

        /// Project path (default: current directory)
        #[arg(short, long)]
        path: Option<PathBuf>,

        /// Output format (text, json, or sarif)
        #[arg(short, long, default_value = "text")]
        format: String,
    },

    /// Aggregate quality score across all work contracts (DBC spec §14.6)
    #[command(visible_aliases = &["cbs", "portfolio"])]
    CodebaseScore {
        /// Project path (default: current directory)
        #[arg(short, long)]
        path: Option<PathBuf>,

        /// Output format (text or json)
        #[arg(short, long, default_value = "text")]
        format: String,
    },
}

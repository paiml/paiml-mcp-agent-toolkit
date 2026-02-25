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

// Roadmap and Agent commands - extracted for file health (CB-040)

/// Roadmap management subcommands
#[derive(Subcommand)]
#[cfg_attr(test, derive(Debug))]
pub enum RoadmapCommands {
    /// Initialize a new sprint in the roadmap
    Init {
        /// Sprint version (e.g., v2.6.0)
        #[arg(long)]
        version: String,

        /// Sprint title
        #[arg(long)]
        title: String,

        /// Sprint duration in days
        #[arg(long, default_value = "14")]
        duration_days: u32,

        /// Sprint priority (P0, P1, P2)
        #[arg(long, default_value = "P0")]
        priority: String,
    },

    /// Generate PDMT todos from roadmap tasks
    Todos {
        /// Sprint ID to generate todos for (uses current if not specified)
        #[arg(long)]
        sprint: Option<String>,

        /// Output file path for todos
        #[arg(long, default_value = "todos.md")]
        output: PathBuf,

        /// Include quality gate requirements in todos
        #[arg(long)]
        include_quality_gates: bool,
    },

    /// Start working on a task
    Start {
        /// Task ID (e.g., PMAT-3001)
        task_id: String,

        /// Create a git branch for the task
        #[arg(long)]
        create_branch: bool,
    },

    /// Complete a task (with quality validation)
    Complete {
        /// Task ID (e.g., PMAT-3001)
        task_id: String,

        /// Skip quality gate checks
        #[arg(long)]
        skip_quality_check: bool,
    },

    /// Check sprint or task status
    Status {
        /// Sprint ID to check
        #[arg(long)]
        sprint: Option<String>,

        /// Task ID to check
        #[arg(long)]
        task: Option<String>,

        /// Output format
        #[arg(long, value_enum, default_value = "human")]
        format: OutputFormat,
    },

    /// Validate sprint readiness for release
    Validate {
        /// Sprint ID to validate
        #[arg(long)]
        sprint: String,

        /// Fail if validation fails (exit code 1)
        #[arg(long)]
        strict: bool,
    },

    /// Run quality checks for a task
    QualityCheck {
        /// Task ID to check
        #[arg(long)]
        task_id: String,
    },
}

/// Test suite types for performance validation
#[derive(Clone, Debug, clap::ValueEnum, PartialEq)]
pub enum TestSuite {
    /// Performance tests per SPECIFICATION.md Section 30
    Performance,
    /// Property-based testing expansion per SPECIFICATION.md Section 28
    Property,
    /// Integration test suite
    Integration,
    /// Regression detection tests
    Regression,
    /// Memory usage validation tests
    Memory,
    /// Throughput validation tests
    Throughput,
    /// All test suites
    All,
}

/// Transport protocol options for serve command
#[derive(Clone, Debug, clap::ValueEnum)]
#[cfg_attr(test, derive(PartialEq))]
pub enum ServeTransport {
    /// HTTP transport (REST API)
    Http,
    /// WebSocket transport (real-time bidirectional)
    WebSocket,
    /// HTTP Server-Sent Events transport (streaming)
    HttpSse,
    /// Both HTTP and WebSocket (hybrid mode)
    Both,
    /// All transports (HTTP, WebSocket, SSE)
    All,
}

/// Agent mode subcommands for Claude Code integration
#[derive(Subcommand)]
#[cfg_attr(test, derive(Debug))]
pub enum AgentCommands {
    /// Start the background agent daemon
    Start {
        /// Project path to monitor (defaults to current directory)
        #[arg(short = 'p', long, default_value = ".")]
        project_path: PathBuf,

        /// Configuration file path
        #[arg(short = 'c', long)]
        config: Option<PathBuf>,

        /// Working directory for the daemon
        #[arg(long)]
        working_dir: Option<PathBuf>,

        /// PID file location
        #[arg(long)]
        pid_file: Option<PathBuf>,

        /// Log file location
        #[arg(long)]
        log_file: Option<PathBuf>,

        /// Run in foreground (don't daemonize)
        #[arg(long)]
        foreground: bool,

        /// Health check interval in seconds
        #[arg(long, default_value = "30")]
        health_interval: u64,

        /// Maximum memory usage in MB before restart
        #[arg(long, default_value = "500")]
        max_memory_mb: u64,

        /// Disable auto-restart on failure
        #[arg(long)]
        no_auto_restart: bool,
    },

    /// Stop the background agent daemon
    Stop {
        /// PID file location
        #[arg(long)]
        pid_file: Option<PathBuf>,

        /// Force stop (SIGKILL) if graceful stop fails
        #[arg(long)]
        force: bool,

        /// Timeout for graceful shutdown in seconds
        #[arg(long, default_value = "10")]
        timeout: u64,
    },

    /// Show daemon status
    Status {
        /// PID file location
        #[arg(long)]
        pid_file: Option<PathBuf>,

        /// Output format
        #[arg(long, value_enum, default_value = "json")]
        format: OutputFormat,
    },

    /// Start monitoring a new project
    Monitor {
        /// Project path to start monitoring
        #[arg(short = 'p', long)]
        project_path: PathBuf,

        /// Project identifier (defaults to path basename)
        #[arg(long)]
        project_id: Option<String>,

        /// Quality thresholds configuration file
        #[arg(long)]
        thresholds: Option<PathBuf>,
    },

    /// Stop monitoring a project
    Unmonitor {
        /// Project ID to stop monitoring
        #[arg(short = 'i', long)]
        project_id: String,
    },

    /// Run health check
    Health {
        /// PID file location
        #[arg(long)]
        pid_file: Option<PathBuf>,

        /// Detailed health information
        #[arg(long)]
        detailed: bool,
    },

    /// Reload daemon configuration
    Reload {
        /// PID file location
        #[arg(long)]
        pid_file: Option<PathBuf>,

        /// Configuration file path
        #[arg(short = 'c', long)]
        config: Option<PathBuf>,
    },

    /// Run quality gate through agent
    QualityGate {
        /// Project ID or path
        #[arg(short = 'p', long)]
        project: String,

        /// Specific file to check
        #[arg(long)]
        file: Option<PathBuf>,

        /// Output format
        #[arg(long, value_enum, default_value = "summary")]
        format: QualityGateOutputFormat,
    },

    /// Start MCP server for testing
    McpServer {
        /// Configuration file path
        #[arg(short = 'c', long)]
        config: Option<PathBuf>,

        /// Debug mode (verbose logging)
        #[arg(long)]
        debug: bool,
    },
}

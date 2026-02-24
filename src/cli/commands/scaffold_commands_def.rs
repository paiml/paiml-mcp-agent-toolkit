/// Scaffold subcommands
#[derive(Subcommand)]
#[cfg_attr(test, derive(Debug))]
pub enum ScaffoldCommands {
    /// Scaffold a complete project with templates
    Project {
        /// Target toolchain
        toolchain: String,

        /// Templates to generate
        #[arg(short, long, value_delimiter = ',')]
        templates: Vec<String>,

        /// Parameters
        #[arg(short = 'p', long = "param", value_parser = crate::cli::args::parse_key_val)]
        params: Vec<(String, Value)>,

        /// Parallelism level
        #[arg(long, default_value_t = num_cpus::get())]
        parallel: usize,
    },

    /// Scaffold a deterministic MCP agent
    Agent {
        /// Agent name
        #[arg(short, long)]
        name: String,

        /// Template type (mcp-server, state-machine, hybrid, calculator, custom:<path>)
        #[arg(short, long)]
        template: String,

        /// Features to include (comma-separated)
        #[arg(short, long, value_delimiter = ',')]
        features: Vec<String>,

        /// Quality level (standard, strict, extreme)
        #[arg(short = 'l', long, default_value = "strict")]
        quality: String,

        /// Output directory
        #[arg(short, long)]
        output: Option<PathBuf>,

        /// Overwrite existing directory
        #[arg(long)]
        force: bool,

        /// Show what would be generated without creating files
        #[arg(long)]
        dry_run: bool,

        /// Interactive mode for guided creation
        #[arg(short, long)]
        interactive: bool,

        /// Deterministic core specification (for hybrid agents)
        #[arg(long)]
        deterministic_core: Option<String>,

        /// Probabilistic wrapper specification (for hybrid agents)
        #[arg(long)]
        probabilistic_wrapper: Option<String>,
    },

    /// Scaffold a WebAssembly project
    Wasm {
        /// Project name
        #[arg(short, long)]
        name: String,

        /// WASM framework (wasm-labs, pure-wasm)
        #[arg(short, long, default_value = "wasm-labs")]
        framework: String,

        /// Features to include (comma-separated)
        #[arg(short, long, value_delimiter = ',')]
        features: Vec<String>,

        /// Quality level (standard, strict, extreme)
        #[arg(short = 'l', long, default_value = "strict")]
        quality: String,

        /// Output directory
        #[arg(short, long)]
        output: Option<PathBuf>,

        /// Overwrite existing directory
        #[arg(long)]
        force: bool,

        /// Show what would be generated without creating files
        #[arg(long)]
        dry_run: bool,
    },

    /// List available agent templates
    ListTemplates,

    /// Validate an agent template
    ValidateTemplate {
        /// Path to template file
        path: PathBuf,
    },

    /// List available Claude Code sub-agents
    ListSubagents {
        /// Show all sub-agents (including future phases)
        #[arg(long)]
        all: bool,
    },

    /// Create a specific Claude Code sub-agent
    CreateSubagent {
        /// Sub-agent name (e.g., complexity-analyst, mutation-tester)
        agent_name: String,

        /// Output directory (defaults to .claude/subagents)
        #[arg(short, long)]
        output: Option<PathBuf>,
    },

    /// Create all MVP Claude Code sub-agents
    CreateAllSubagents {
        /// Output directory (defaults to .claude/subagents)
        #[arg(short, long)]
        output: Option<PathBuf>,
    },

    /// Validate a sub-agent definition file
    ValidateSubagent {
        /// Path to sub-agent definition file
        file_path: PathBuf,
    },

    /// Show MCP tool mapping for sub-agents
    ShowToolMapping {
        /// Specific sub-agent name (shows all if not specified)
        #[arg(short, long)]
        agent: Option<String>,
    },

    /// Export MCP tool mapping as JSON
    ExportToolMapping {
        /// Output file path
        #[arg(short, long)]
        output: PathBuf,
    },
}

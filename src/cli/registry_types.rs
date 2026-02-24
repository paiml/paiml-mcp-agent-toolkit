/// Single source of truth for all command metadata.
///
/// # Invariants
///
/// 1. Every CLI command has exactly one entry
/// 2. Every MCP tool maps to exactly one command
/// 3. All examples are validated at build time
/// 4. No duplicate command names or aliases
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct CommandRegistry {
    /// Semantic version of the registry format
    pub version: String,

    /// All registered commands, keyed by canonical name
    pub commands: HashMap<String, CommandMetadata>,

    /// Global flags available to all commands
    pub global_flags: Vec<FlagMetadata>,

    /// Timestamp when registry was built
    pub built_at: Option<String>,
}

/// Complete metadata for a single command.
///
/// This struct captures everything needed to:
/// - Generate accurate help text
/// - Create MCP tool schema
/// - Index for semantic search
/// - Validate documentation
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct CommandMetadata {
    /// Canonical command name (e.g., "analyze complexity")
    pub name: String,

    /// Short description for listings (max 80 chars)
    pub short_description: String,

    /// Long description for --help
    pub long_description: String,

    /// Command aliases (e.g., ["cx"] for complexity)
    pub aliases: Vec<String>,

    /// Command arguments
    pub arguments: Vec<ArgumentMetadata>,

    /// Working examples that MUST execute successfully
    pub examples: Vec<ExampleMetadata>,

    /// MCP-specific metadata (None if not exposed via MCP)
    pub mcp: Option<McpToolMetadata>,

    /// Subcommands (for nested commands)
    pub subcommands: Vec<CommandMetadata>,

    /// Semantic tags for RAG retrieval
    pub tags: Vec<String>,

    /// Related commands for cross-reference
    pub related: Vec<String>,

    /// Deprecation info if applicable
    pub deprecated: Option<DeprecationInfo>,

    /// Category for grouping (e.g., "analysis", "quality", "scaffolding")
    pub category: String,

    /// Whether this command modifies state
    pub is_mutation: bool,

    /// Estimated execution time category
    pub execution_time: ExecutionTime,
}

/// Argument metadata with validation rules.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ArgumentMetadata {
    /// Argument name (e.g., "project-path")
    pub name: String,

    /// Short flag (e.g., 'p' for -p)
    pub short: Option<char>,

    /// Long flag (e.g., "project-path" for --project-path)
    pub long: Option<String>,

    /// Description for help text
    pub description: String,

    /// Whether argument is required
    pub required: bool,

    /// Default value if not provided
    pub default: Option<String>,

    /// Type of the value
    pub value_type: ValueType,

    /// Possible values for enums
    pub possible_values: Vec<String>,

    /// Environment variable that can set this
    pub env_var: Option<String>,

    /// Whether this is a positional argument (not a flag)
    pub positional: bool,
}

/// Value types for arguments
#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq, Eq)]
pub enum ValueType {
    #[default]
    String,
    Integer,
    Float,
    Boolean,
    Path,
    Enum,
    List,
}

/// Flag metadata for global flags
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct FlagMetadata {
    pub name: String,
    pub short: Option<char>,
    pub long: Option<String>,
    pub description: String,
    pub default: Option<String>,
}

/// Example that MUST be validated at build time.
///
/// # Build-Time Validation
///
/// During `cargo build`, all examples with `requires_project: false`
/// are executed to ensure they work. This guarantees documentation
/// accuracy (Toyota Way - Poka-yoke).
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ExampleMetadata {
    /// Description of what this example demonstrates
    pub description: String,

    /// The exact command to run
    pub command: String,

    /// Expected exit code (default: 0)
    pub expected_exit_code: i32,

    /// Regex patterns that output must match (optional)
    pub output_patterns: Vec<String>,

    /// Whether this example requires a specific project structure
    pub requires_project: bool,

    /// Project type required (if requires_project is true)
    pub project_type: Option<String>,
}

/// MCP tool-specific metadata.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct McpToolMetadata {
    /// MCP tool name (may differ from CLI command)
    pub tool_name: String,

    /// JSON Schema for input validation
    pub input_schema: serde_json::Value,

    /// Whether this tool modifies state
    pub is_mutation: bool,

    /// Estimated execution time category
    pub execution_time: ExecutionTime,

    /// MCP annotations
    pub annotations: McpAnnotations,
}

/// MCP tool annotations per spec
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct McpAnnotations {
    pub title: String,
    pub read_only_hint: bool,
    pub destructive_hint: bool,
    pub idempotent_hint: bool,
    pub open_world_hint: bool,
}

/// Execution time categories
#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq, Eq)]
pub enum ExecutionTime {
    /// < 1 second
    #[default]
    Fast,
    /// 1-10 seconds
    Medium,
    /// > 10 seconds
    Slow,
}

/// Deprecation information
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct DeprecationInfo {
    /// Version when deprecated
    pub since_version: String,
    /// Version when it will be removed
    pub removal_version: Option<String>,
    /// Replacement command (if any)
    pub replacement: Option<String>,
    /// Reason for deprecation
    pub reason: String,
}

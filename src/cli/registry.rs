//! Command Registry - Single Source of Truth for CLI/MCP/Help
//!
//! This module provides unified command metadata that is used to generate:
//! - `--help` text (dynamic, always accurate)
//! - MCP tool schemas (JSON Schema)
//! - Documentation (README examples)
//! - Semantic help search (RAG-powered)
//!
//! # Architecture (Toyota Way - Jidoka)
//!
//! All command metadata flows from a single source:
//! ```text
//! CommandRegistry (source of truth)
//!        │
//!        ├─▶ HelpGenerator (--help text)
//!        ├─▶ McpSchemaGenerator (MCP tools/list)
//!        └─▶ DocsGenerator (README.md)
//! ```
//!
//! # References
//!
//! - Specification: docs/specifications/unified-cli-mcp-help-integration.md
//! - GitHub Issue: #118
//! - Toyota Way: Jidoka (built-in quality), Poka-yoke (error-proofing)

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

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

impl CommandRegistry {
    /// Create a new empty registry
    pub fn new(version: impl Into<String>) -> Self {
        Self {
            version: version.into(),
            commands: HashMap::new(),
            global_flags: Vec::new(),
            built_at: Some(chrono::Utc::now().to_rfc3339()),
        }
    }

    /// Register a command
    pub fn register(&mut self, command: CommandMetadata) -> &mut Self {
        self.commands.insert(command.name.clone(), command);
        self
    }

    /// Register a global flag
    pub fn register_global_flag(&mut self, flag: FlagMetadata) -> &mut Self {
        self.global_flags.push(flag);
        self
    }

    /// Find a command by path (e.g., "analyze complexity")
    pub fn find_command(&self, path: &str) -> Option<&CommandMetadata> {
        // First try exact match
        if let Some(cmd) = self.commands.get(path) {
            return Some(cmd);
        }

        // Try to find by alias
        for cmd in self.commands.values() {
            if cmd.aliases.iter().any(|a| a == path) {
                return Some(cmd);
            }
        }

        // Try hierarchical lookup (e.g., "analyze complexity" -> analyze -> complexity)
        let parts: Vec<&str> = path.split_whitespace().collect();
        if parts.len() > 1 {
            let parent = parts[0];
            if let Some(parent_cmd) = self.commands.get(parent) {
                let subcommand_path = parts[1..].join(" ");
                return parent_cmd.find_subcommand(&subcommand_path);
            }
        }

        None
    }

    /// Find commands by semantic tags
    pub fn find_by_tag(&self, tag: &str) -> Vec<&CommandMetadata> {
        self.commands
            .values()
            .filter(|cmd| cmd.tags.iter().any(|t| t == tag))
            .collect()
    }

    /// Find commands by category
    pub fn find_by_category(&self, category: &str) -> Vec<&CommandMetadata> {
        self.commands
            .values()
            .filter(|cmd| cmd.category == category)
            .collect()
    }

    /// Get all command names (including subcommand paths)
    pub fn all_command_paths(&self) -> Vec<String> {
        let mut paths = Vec::new();
        for (name, cmd) in &self.commands {
            paths.push(name.clone());
            for sub in &cmd.subcommands {
                paths.push(format!("{} {}", name, sub.name));
            }
        }
        paths.sort();
        paths
    }

    /// Validate registry consistency
    pub fn validate(&self) -> Result<(), Vec<RegistryError>> {
        let mut errors = Vec::new();

        // Check for duplicate aliases
        let mut seen_aliases: HashMap<&str, &str> = HashMap::new();
        for (name, cmd) in &self.commands {
            for alias in &cmd.aliases {
                if let Some(existing) = seen_aliases.get(alias.as_str()) {
                    errors.push(RegistryError::DuplicateAlias {
                        alias: alias.clone(),
                        command1: (*existing).to_string(),
                        command2: name.clone(),
                    });
                } else {
                    seen_aliases.insert(alias.as_str(), name.as_str());
                }
            }
        }

        // Check MCP tool name uniqueness
        let mut seen_mcp_tools: HashMap<&str, &str> = HashMap::new();
        for (name, cmd) in &self.commands {
            if let Some(mcp) = &cmd.mcp {
                if let Some(existing) = seen_mcp_tools.get(mcp.tool_name.as_str()) {
                    errors.push(RegistryError::DuplicateMcpTool {
                        tool_name: mcp.tool_name.clone(),
                        command1: (*existing).to_string(),
                        command2: name.clone(),
                    });
                } else {
                    seen_mcp_tools.insert(mcp.tool_name.as_str(), name.as_str());
                }
            }
        }

        // Check related command references
        for (name, cmd) in &self.commands {
            for related in &cmd.related {
                if !self.commands.contains_key(related) {
                    errors.push(RegistryError::InvalidRelatedCommand {
                        command: name.clone(),
                        related: related.clone(),
                    });
                }
            }
        }

        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }

    /// Serialize to JSON
    pub fn to_json(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string_pretty(self)
    }

    /// Deserialize from JSON
    pub fn from_json(json: &str) -> Result<Self, serde_json::Error> {
        serde_json::from_str(json)
    }
}

impl CommandMetadata {
    /// Create a new command metadata builder
    pub fn builder(name: impl Into<String>) -> CommandMetadataBuilder {
        CommandMetadataBuilder::new(name)
    }

    /// Find a subcommand by name
    pub fn find_subcommand(&self, name: &str) -> Option<&CommandMetadata> {
        self.subcommands
            .iter()
            .find(|sub| sub.name == name || sub.aliases.iter().any(|a| a == name))
    }

    /// Get full command path from root
    pub fn full_path(&self, parent: Option<&str>) -> String {
        match parent {
            Some(p) => format!("{} {}", p, self.name),
            None => self.name.clone(),
        }
    }
}

/// Builder for CommandMetadata
#[derive(Debug, Default)]
pub struct CommandMetadataBuilder {
    metadata: CommandMetadata,
}

impl CommandMetadataBuilder {
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            metadata: CommandMetadata {
                name: name.into(),
                ..Default::default()
            },
        }
    }

    pub fn short_description(mut self, desc: impl Into<String>) -> Self {
        self.metadata.short_description = desc.into();
        self
    }

    pub fn long_description(mut self, desc: impl Into<String>) -> Self {
        self.metadata.long_description = desc.into();
        self
    }

    pub fn alias(mut self, alias: impl Into<String>) -> Self {
        self.metadata.aliases.push(alias.into());
        self
    }

    pub fn aliases(mut self, aliases: impl IntoIterator<Item = impl Into<String>>) -> Self {
        self.metadata
            .aliases
            .extend(aliases.into_iter().map(Into::into));
        self
    }

    pub fn argument(mut self, arg: ArgumentMetadata) -> Self {
        self.metadata.arguments.push(arg);
        self
    }

    pub fn example(mut self, example: ExampleMetadata) -> Self {
        self.metadata.examples.push(example);
        self
    }

    pub fn mcp(mut self, mcp: McpToolMetadata) -> Self {
        self.metadata.mcp = Some(mcp);
        self
    }

    pub fn subcommand(mut self, sub: CommandMetadata) -> Self {
        self.metadata.subcommands.push(sub);
        self
    }

    pub fn tag(mut self, tag: impl Into<String>) -> Self {
        self.metadata.tags.push(tag.into());
        self
    }

    pub fn tags(mut self, tags: impl IntoIterator<Item = impl Into<String>>) -> Self {
        self.metadata.tags.extend(tags.into_iter().map(Into::into));
        self
    }

    pub fn related(mut self, related: impl Into<String>) -> Self {
        self.metadata.related.push(related.into());
        self
    }

    pub fn category(mut self, category: impl Into<String>) -> Self {
        self.metadata.category = category.into();
        self
    }

    pub fn deprecated(mut self, info: DeprecationInfo) -> Self {
        self.metadata.deprecated = Some(info);
        self
    }

    pub fn is_mutation(mut self, is_mutation: bool) -> Self {
        self.metadata.is_mutation = is_mutation;
        self
    }

    pub fn execution_time(mut self, time: ExecutionTime) -> Self {
        self.metadata.execution_time = time;
        self
    }

    pub fn build(self) -> CommandMetadata {
        self.metadata
    }
}

/// Registry validation errors
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RegistryError {
    DuplicateAlias {
        alias: String,
        command1: String,
        command2: String,
    },
    DuplicateMcpTool {
        tool_name: String,
        command1: String,
        command2: String,
    },
    InvalidRelatedCommand {
        command: String,
        related: String,
    },
    InvalidExample {
        command: String,
        example: String,
        reason: String,
    },
}

impl std::fmt::Display for RegistryError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::DuplicateAlias {
                alias,
                command1,
                command2,
            } => {
                write!(
                    f,
                    "Duplicate alias '{}' in commands '{}' and '{}'",
                    alias, command1, command2
                )
            }
            Self::DuplicateMcpTool {
                tool_name,
                command1,
                command2,
            } => {
                write!(
                    f,
                    "Duplicate MCP tool '{}' in commands '{}' and '{}'",
                    tool_name, command1, command2
                )
            }
            Self::InvalidRelatedCommand { command, related } => {
                write!(
                    f,
                    "Command '{}' references non-existent related command '{}'",
                    command, related
                )
            }
            Self::InvalidExample {
                command,
                example,
                reason,
            } => {
                write!(
                    f,
                    "Invalid example '{}' in command '{}': {}",
                    example, command, reason
                )
            }
        }
    }
}

impl std::error::Error for RegistryError {}

#[cfg_attr(coverage_nightly, coverage(off))]
#[cfg(test)]
mod tests {
    use super::*;

    // ═══════════════════════════════════════════════════════════════════════════
    // RED PHASE TESTS - These MUST fail until implementation is complete
    // ═══════════════════════════════════════════════════════════════════════════

    mod registry_core_tests {
        use super::*;

        #[test]
        fn test_registry_creation() {
            let registry = CommandRegistry::new("1.0.0");
            assert_eq!(registry.version, "1.0.0");
            assert!(registry.commands.is_empty());
            assert!(registry.global_flags.is_empty());
            assert!(registry.built_at.is_some());
        }

        #[test]
        fn test_register_command() {
            let mut registry = CommandRegistry::new("1.0.0");
            let cmd = CommandMetadata::builder("analyze")
                .short_description("Analyze code")
                .category("analysis")
                .build();

            registry.register(cmd);

            assert_eq!(registry.commands.len(), 1);
            assert!(registry.commands.contains_key("analyze"));
        }

        #[test]
        fn test_find_command_exact() {
            let mut registry = CommandRegistry::new("1.0.0");
            registry.register(
                CommandMetadata::builder("analyze")
                    .short_description("Analyze code")
                    .build(),
            );

            let found = registry.find_command("analyze");
            assert!(found.is_some());
            assert_eq!(found.unwrap().name, "analyze");
        }

        #[test]
        fn test_find_command_by_alias() {
            let mut registry = CommandRegistry::new("1.0.0");
            registry.register(
                CommandMetadata::builder("analyze")
                    .short_description("Analyze code")
                    .aliases(["a", "an"])
                    .build(),
            );

            let found = registry.find_command("a");
            assert!(found.is_some());
            assert_eq!(found.unwrap().name, "analyze");

            let found2 = registry.find_command("an");
            assert!(found2.is_some());
        }

        #[test]
        fn test_find_command_hierarchical() {
            let mut registry = CommandRegistry::new("1.0.0");
            let complexity_cmd = CommandMetadata::builder("complexity")
                .short_description("Analyze complexity")
                .build();

            registry.register(
                CommandMetadata::builder("analyze")
                    .short_description("Analyze code")
                    .subcommand(complexity_cmd)
                    .build(),
            );

            let found = registry.find_command("analyze complexity");
            assert!(found.is_some());
            assert_eq!(found.unwrap().name, "complexity");
        }

        #[test]
        fn test_find_by_tag() {
            let mut registry = CommandRegistry::new("1.0.0");
            registry.register(
                CommandMetadata::builder("analyze")
                    .tags(["quality", "metrics"])
                    .build(),
            );
            registry.register(
                CommandMetadata::builder("context")
                    .tags(["generation", "ast"])
                    .build(),
            );

            let quality_cmds = registry.find_by_tag("quality");
            assert_eq!(quality_cmds.len(), 1);
            assert_eq!(quality_cmds[0].name, "analyze");
        }

        #[test]
        fn test_find_by_category() {
            let mut registry = CommandRegistry::new("1.0.0");
            registry.register(
                CommandMetadata::builder("analyze")
                    .category("analysis")
                    .build(),
            );
            registry.register(
                CommandMetadata::builder("scaffold")
                    .category("generation")
                    .build(),
            );

            let analysis_cmds = registry.find_by_category("analysis");
            assert_eq!(analysis_cmds.len(), 1);
            assert_eq!(analysis_cmds[0].name, "analyze");
        }

        #[test]
        fn test_all_command_paths() {
            let mut registry = CommandRegistry::new("1.0.0");
            let sub = CommandMetadata::builder("complexity").build();
            registry.register(CommandMetadata::builder("analyze").subcommand(sub).build());
            registry.register(CommandMetadata::builder("context").build());

            let paths = registry.all_command_paths();
            assert!(paths.contains(&"analyze".to_string()));
            assert!(paths.contains(&"analyze complexity".to_string()));
            assert!(paths.contains(&"context".to_string()));
        }
    }

    mod registry_validation_tests {
        use super::*;

        #[test]
        fn test_validate_duplicate_alias() {
            let mut registry = CommandRegistry::new("1.0.0");
            registry.register(CommandMetadata::builder("analyze").alias("a").build());
            registry.register(
                CommandMetadata::builder("agent")
                    .alias("a") // Duplicate!
                    .build(),
            );

            let result = registry.validate();
            assert!(result.is_err());
            let errors = result.unwrap_err();
            assert!(errors
                .iter()
                .any(|e| matches!(e, RegistryError::DuplicateAlias { .. })));
        }

        #[test]
        fn test_validate_duplicate_mcp_tool() {
            let mut registry = CommandRegistry::new("1.0.0");
            registry.register(
                CommandMetadata::builder("analyze")
                    .mcp(McpToolMetadata {
                        tool_name: "pmat_analyze".to_string(),
                        ..Default::default()
                    })
                    .build(),
            );
            registry.register(
                CommandMetadata::builder("context")
                    .mcp(McpToolMetadata {
                        tool_name: "pmat_analyze".to_string(), // Duplicate!
                        ..Default::default()
                    })
                    .build(),
            );

            let result = registry.validate();
            assert!(result.is_err());
            let errors = result.unwrap_err();
            assert!(errors
                .iter()
                .any(|e| matches!(e, RegistryError::DuplicateMcpTool { .. })));
        }

        #[test]
        fn test_validate_invalid_related_command() {
            let mut registry = CommandRegistry::new("1.0.0");
            registry.register(
                CommandMetadata::builder("analyze")
                    .related("nonexistent") // Invalid reference!
                    .build(),
            );

            let result = registry.validate();
            assert!(result.is_err());
            let errors = result.unwrap_err();
            assert!(errors
                .iter()
                .any(|e| matches!(e, RegistryError::InvalidRelatedCommand { .. })));
        }

        #[test]
        fn test_validate_success() {
            let mut registry = CommandRegistry::new("1.0.0");
            registry.register(CommandMetadata::builder("analyze").alias("a").build());
            registry.register(
                CommandMetadata::builder("context")
                    .alias("ctx")
                    .related("analyze") // Valid reference
                    .build(),
            );

            let result = registry.validate();
            assert!(result.is_ok());
        }
    }

    mod serialization_tests {
        use super::*;

        #[test]
        fn test_to_json() {
            let mut registry = CommandRegistry::new("1.0.0");
            registry.register(
                CommandMetadata::builder("analyze")
                    .short_description("Analyze code")
                    .build(),
            );

            let json = registry.to_json().unwrap();
            assert!(json.contains("\"version\": \"1.0.0\""));
            assert!(json.contains("\"analyze\""));
        }

        #[test]
        fn test_from_json_roundtrip() {
            let mut registry = CommandRegistry::new("1.0.0");
            registry.register(
                CommandMetadata::builder("analyze")
                    .short_description("Analyze code")
                    .aliases(["a", "an"])
                    .tags(["quality"])
                    .build(),
            );

            let json = registry.to_json().unwrap();
            let restored = CommandRegistry::from_json(&json).unwrap();

            assert_eq!(restored.version, registry.version);
            assert_eq!(restored.commands.len(), registry.commands.len());
            let cmd = restored.commands.get("analyze").unwrap();
            assert_eq!(cmd.aliases, vec!["a", "an"]);
        }
    }

    mod builder_tests {
        use super::*;

        #[test]
        fn test_command_builder() {
            let cmd = CommandMetadata::builder("analyze")
                .short_description("Analyze code")
                .long_description("Analyze code for various metrics")
                .aliases(["a", "an"])
                .category("analysis")
                .tags(["quality", "metrics"])
                .is_mutation(false)
                .execution_time(ExecutionTime::Fast)
                .build();

            assert_eq!(cmd.name, "analyze");
            assert_eq!(cmd.short_description, "Analyze code");
            assert_eq!(cmd.aliases, vec!["a", "an"]);
            assert_eq!(cmd.category, "analysis");
            assert!(!cmd.is_mutation);
            assert_eq!(cmd.execution_time, ExecutionTime::Fast);
        }

        #[test]
        fn test_builder_with_arguments() {
            let cmd = CommandMetadata::builder("analyze")
                .argument(ArgumentMetadata {
                    name: "project-path".to_string(),
                    short: Some('p'),
                    long: Some("project-path".to_string()),
                    description: "Path to project".to_string(),
                    required: false,
                    default: Some(".".to_string()),
                    value_type: ValueType::Path,
                    ..Default::default()
                })
                .build();

            assert_eq!(cmd.arguments.len(), 1);
            assert_eq!(cmd.arguments[0].name, "project-path");
            assert_eq!(cmd.arguments[0].short, Some('p'));
        }

        #[test]
        fn test_builder_with_examples() {
            let cmd = CommandMetadata::builder("analyze")
                .example(ExampleMetadata {
                    description: "Analyze current project".to_string(),
                    command: "pmat analyze complexity".to_string(),
                    expected_exit_code: 0,
                    requires_project: false,
                    ..Default::default()
                })
                .build();

            assert_eq!(cmd.examples.len(), 1);
            assert_eq!(cmd.examples[0].command, "pmat analyze complexity");
        }

        #[test]
        fn test_builder_with_mcp() {
            let cmd = CommandMetadata::builder("analyze")
                .mcp(McpToolMetadata {
                    tool_name: "pmat_analyze_complexity".to_string(),
                    input_schema: serde_json::json!({
                        "type": "object",
                        "properties": {
                            "project_path": { "type": "string" }
                        }
                    }),
                    is_mutation: false,
                    execution_time: ExecutionTime::Medium,
                    annotations: McpAnnotations {
                        title: "Analyze Complexity".to_string(),
                        read_only_hint: true,
                        ..Default::default()
                    },
                })
                .build();

            assert!(cmd.mcp.is_some());
            let mcp = cmd.mcp.unwrap();
            assert_eq!(mcp.tool_name, "pmat_analyze_complexity");
            assert!(mcp.annotations.read_only_hint);
        }

        #[test]
        fn test_builder_with_subcommands() {
            let sub1 = CommandMetadata::builder("complexity")
                .short_description("Analyze complexity")
                .build();
            let sub2 = CommandMetadata::builder("satd")
                .short_description("Find SATD")
                .build();

            let cmd = CommandMetadata::builder("analyze")
                .subcommand(sub1)
                .subcommand(sub2)
                .build();

            assert_eq!(cmd.subcommands.len(), 2);
            assert_eq!(cmd.subcommands[0].name, "complexity");
            assert_eq!(cmd.subcommands[1].name, "satd");
        }

        #[test]
        fn test_builder_deprecated() {
            let cmd = CommandMetadata::builder("old-command")
                .deprecated(DeprecationInfo {
                    since_version: "2.0.0".to_string(),
                    removal_version: Some("3.0.0".to_string()),
                    replacement: Some("new-command".to_string()),
                    reason: "Replaced with better implementation".to_string(),
                })
                .build();

            assert!(cmd.deprecated.is_some());
            let dep = cmd.deprecated.unwrap();
            assert_eq!(dep.since_version, "2.0.0");
            assert_eq!(dep.replacement, Some("new-command".to_string()));
        }
    }

    mod metadata_tests {
        use super::*;

        #[test]
        fn test_find_subcommand() {
            let sub = CommandMetadata::builder("complexity")
                .aliases(["cx"])
                .build();
            let cmd = CommandMetadata::builder("analyze").subcommand(sub).build();

            // Find by name
            let found = cmd.find_subcommand("complexity");
            assert!(found.is_some());

            // Find by alias
            let found_alias = cmd.find_subcommand("cx");
            assert!(found_alias.is_some());
            assert_eq!(found_alias.unwrap().name, "complexity");
        }

        #[test]
        fn test_full_path() {
            let cmd = CommandMetadata::builder("complexity").build();

            assert_eq!(cmd.full_path(None), "complexity");
            assert_eq!(cmd.full_path(Some("analyze")), "analyze complexity");
        }
    }
}

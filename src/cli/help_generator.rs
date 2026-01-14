//! Help Generator - Dynamic --help text from CommandRegistry
//!
//! This module generates accurate help text from the single source of truth
//! (CommandRegistry), ensuring documentation is never out of sync with implementation.
//!
//! # Architecture (Toyota Way - Genchi Genbutsu)
//!
//! ```text
//! CommandRegistry → HelpGenerator → Formatted Help Text
//!                                      ├─ Terminal output
//!                                      ├─ Man pages
//!                                      └─ Markdown docs
//! ```
//!
//! # References
//!
//! - Specification: docs/specifications/unified-cli-mcp-help-integration.md
//! - GitHub Issue: #118

use crate::cli::registry::{
    ArgumentMetadata, CommandMetadata, CommandRegistry, ExecutionTime, ValueType,
};
use std::io::IsTerminal;

/// Generates formatted help text from CommandRegistry.
pub struct HelpGenerator {
    registry: CommandRegistry,
    color: bool,
    width: usize,
}

impl HelpGenerator {
    /// Create a new HelpGenerator
    pub fn new(registry: CommandRegistry) -> Self {
        Self {
            registry,
            color: std::io::stdout().is_terminal(),
            width: 80, // Default width, could use terminal_size crate if needed
        }
    }

    /// Create with explicit color setting
    pub fn with_color(mut self, color: bool) -> Self {
        self.color = color;
        self
    }

    /// Create with explicit width
    pub fn with_width(mut self, width: usize) -> Self {
        self.width = width;
        self
    }

    /// Generate help for a specific command path.
    ///
    /// # Arguments
    /// * `path` - Command path like "analyze complexity" or "context"
    ///
    /// # Returns
    /// Formatted help text string
    pub fn generate(&self, path: &str) -> String {
        match self.registry.find_command(path) {
            Some(metadata) => self.format_command_help(metadata),
            None => self.format_command_not_found(path),
        }
    }

    /// Generate top-level help (all commands overview)
    pub fn generate_overview(&self) -> String {
        let mut out = String::new();

        // Header
        out.push_str(&format!("pmat {}\n", self.registry.version));
        out.push_str("Professional project quantitative scaffolding and analysis toolkit\n\n");

        // Usage
        out.push_str("USAGE:\n");
        out.push_str("    pmat [OPTIONS] <COMMAND>\n\n");

        // Global flags
        if !self.registry.global_flags.is_empty() {
            out.push_str("OPTIONS:\n");
            for flag in &self.registry.global_flags {
                out.push_str(&self.format_flag(flag));
            }
            out.push('\n');
        }

        // Commands by category
        let mut categories: std::collections::HashMap<&str, Vec<&CommandMetadata>> =
            std::collections::HashMap::new();

        for cmd in self.registry.commands.values() {
            let category = if cmd.category.is_empty() {
                "Other"
            } else {
                &cmd.category
            };
            categories.entry(category).or_default().push(cmd);
        }

        out.push_str("COMMANDS:\n");
        let mut sorted_categories: Vec<_> = categories.keys().collect();
        sorted_categories.sort();

        for category in sorted_categories {
            let cmds = categories.get(category).expect("internal error");
            let mut sorted_cmds: Vec<_> = cmds.iter().collect();
            sorted_cmds.sort_by_key(|c| &c.name);

            for cmd in sorted_cmds {
                let name_with_aliases = if cmd.aliases.is_empty() {
                    cmd.name.clone()
                } else {
                    format!("{} ({})", cmd.name, cmd.aliases.join(", "))
                };
                out.push_str(&format!(
                    "    {:30} {}\n",
                    name_with_aliases,
                    truncate_str(&cmd.short_description, 45)
                ));
            }
        }

        out.push_str("\nUse 'pmat <COMMAND> --help' for more information about a command.\n");

        out
    }

    /// Generate help for a specific command
    fn format_command_help(&self, cmd: &CommandMetadata) -> String {
        let mut out = String::new();

        // Header with name and description
        out.push_str(&format!("{}\n", cmd.name));
        if !cmd.short_description.is_empty() {
            out.push_str(&format!("{}\n", cmd.short_description));
        }
        out.push('\n');

        // Long description if available
        if !cmd.long_description.is_empty() {
            out.push_str(&format!("{}\n\n", cmd.long_description));
        }

        // Deprecation warning
        if let Some(dep) = &cmd.deprecated {
            out.push_str(&format!(
                "DEPRECATED: {} (since {})\n",
                dep.reason, dep.since_version
            ));
            if let Some(replacement) = &dep.replacement {
                out.push_str(&format!("Use '{}' instead.\n", replacement));
            }
            out.push('\n');
        }

        // Usage
        out.push_str("USAGE:\n");
        out.push_str(&format!("    pmat {}", self.format_usage(cmd)));
        out.push_str("\n\n");

        // Subcommands
        if !cmd.subcommands.is_empty() {
            out.push_str("SUBCOMMANDS:\n");
            for sub in &cmd.subcommands {
                let name_with_aliases = if sub.aliases.is_empty() {
                    sub.name.clone()
                } else {
                    format!("{} ({})", sub.name, sub.aliases.join(", "))
                };
                out.push_str(&format!(
                    "    {:30} {}\n",
                    name_with_aliases,
                    truncate_str(&sub.short_description, 45)
                ));
            }
            out.push('\n');
        }

        // Arguments
        let positional: Vec<_> = cmd.arguments.iter().filter(|a| a.positional).collect();
        let flags: Vec<_> = cmd.arguments.iter().filter(|a| !a.positional).collect();

        if !positional.is_empty() {
            out.push_str("ARGUMENTS:\n");
            for arg in &positional {
                out.push_str(&self.format_argument(arg));
            }
            out.push('\n');
        }

        if !flags.is_empty() {
            out.push_str("OPTIONS:\n");
            for arg in &flags {
                out.push_str(&self.format_argument(arg));
            }
            out.push('\n');
        }

        // Examples
        if !cmd.examples.is_empty() {
            out.push_str("EXAMPLES:\n");
            for ex in &cmd.examples {
                out.push_str(&format!("    # {}\n", ex.description));
                out.push_str(&format!("    $ {}\n\n", ex.command));
            }
        }

        // Related commands
        if !cmd.related.is_empty() {
            out.push_str("SEE ALSO:\n");
            out.push_str(&format!("    {}\n", cmd.related.join(", ")));
        }

        // Execution time hint
        match cmd.execution_time {
            ExecutionTime::Slow => {
                out.push_str("\nNote: This command may take several seconds to complete.\n");
            }
            _ => {}
        }

        out
    }

    /// Format command not found message with suggestions
    fn format_command_not_found(&self, path: &str) -> String {
        let mut out = String::new();
        out.push_str(&format!("error: unrecognized command '{}'\n\n", path));

        // Find similar commands
        let suggestions = self.find_similar_commands(path, 3);
        if !suggestions.is_empty() {
            out.push_str("Did you mean:\n");
            for (cmd, _score) in suggestions {
                out.push_str(&format!("    pmat {}\n", cmd));
            }
            out.push('\n');
        }

        out.push_str("Use 'pmat --help' to see all available commands.\n");
        out
    }

    /// Format usage string for a command
    fn format_usage(&self, cmd: &CommandMetadata) -> String {
        let mut usage = cmd.name.clone();

        // Add subcommands indicator
        if !cmd.subcommands.is_empty() {
            usage.push_str(" <COMMAND>");
        }

        // Add positional arguments
        for arg in cmd.arguments.iter().filter(|a| a.positional) {
            if arg.required {
                usage.push_str(&format!(" <{}>", arg.name.to_uppercase()));
            } else {
                usage.push_str(&format!(" [{}]", arg.name.to_uppercase()));
            }
        }

        // Indicate options if any
        let has_options = cmd.arguments.iter().any(|a| !a.positional);
        if has_options {
            usage.push_str(" [OPTIONS]");
        }

        usage
    }

    /// Format a single argument for help output
    fn format_argument(&self, arg: &ArgumentMetadata) -> String {
        let mut line = String::new();

        // Build flag/name part
        let flag_part = if arg.positional {
            format!("<{}>", arg.name.to_uppercase())
        } else {
            let short = arg.short.map(|s| format!("-{}", s));
            let long = arg.long.as_ref().map(|l| format!("--{}", l));
            match (short, long) {
                (Some(s), Some(l)) => format!("{}, {}", s, l),
                (Some(s), None) => s,
                (None, Some(l)) => l,
                (None, None) => arg.name.clone(),
            }
        };

        // Add value type indicator
        let value_indicator = match arg.value_type {
            ValueType::Boolean => String::new(),
            ValueType::Enum => {
                if !arg.possible_values.is_empty() {
                    format!(" <{}>", arg.possible_values.join("|"))
                } else {
                    " <VALUE>".to_string()
                }
            }
            _ => format!(" <{}>", arg.name.to_uppercase()),
        };

        let full_flag = format!("{}{}", flag_part, value_indicator);
        line.push_str(&format!("    {:30} ", full_flag));

        // Description
        line.push_str(&arg.description);

        // Default value
        if let Some(default) = &arg.default {
            line.push_str(&format!(" [default: {}]", default));
        }

        // Required indicator
        if arg.required {
            line.push_str(" (required)");
        }

        // Environment variable
        if let Some(env) = &arg.env_var {
            line.push_str(&format!(" [env: {}]", env));
        }

        line.push('\n');
        line
    }

    /// Format a global flag
    fn format_flag(&self, flag: &crate::cli::registry::FlagMetadata) -> String {
        let mut line = String::new();

        let flag_part = match (&flag.short, &flag.long) {
            (Some(s), Some(l)) => format!("-{}, --{}", s, l),
            (Some(s), None) => format!("-{}", s),
            (None, Some(l)) => format!("--{}", l),
            (None, None) => flag.name.clone(),
        };

        line.push_str(&format!("    {:30} ", flag_part));
        line.push_str(&flag.description);

        if let Some(default) = &flag.default {
            line.push_str(&format!(" [default: {}]", default));
        }

        line.push('\n');
        line
    }

    /// Find commands similar to the query using edit distance
    fn find_similar_commands(&self, query: &str, limit: usize) -> Vec<(String, usize)> {
        let all_paths = self.registry.all_command_paths();
        let mut scored: Vec<(String, usize)> = all_paths
            .into_iter()
            .map(|path| {
                let distance = levenshtein(&path, query);
                (path, distance)
            })
            .collect();

        scored.sort_by_key(|(_, score)| *score);
        scored.truncate(limit);

        // Filter out very dissimilar results
        scored
            .into_iter()
            .filter(|(_, score)| *score <= query.len())
            .collect()
    }

    /// Print help to stdout with colors
    pub fn print_help(&self, path: Option<&str>) -> std::io::Result<()> {
        let help = match path {
            Some(p) => self.generate(p),
            None => self.generate_overview(),
        };

        if self.color {
            self.print_colored(&help)
        } else {
            print!("{}", help);
            Ok(())
        }
    }

    /// Print with ANSI colors
    fn print_colored(&self, text: &str) -> std::io::Result<()> {
        // ANSI escape codes
        const RESET: &str = "\x1b[0m";
        const BOLD: &str = "\x1b[1m";
        const YELLOW: &str = "\x1b[33m";
        const RED: &str = "\x1b[31m";
        const CYAN: &str = "\x1b[36m";
        const GREEN: &str = "\x1b[32m";

        for line in text.lines() {
            if line.starts_with("USAGE:")
                || line.starts_with("COMMANDS:")
                || line.starts_with("OPTIONS:")
                || line.starts_with("ARGUMENTS:")
                || line.starts_with("EXAMPLES:")
                || line.starts_with("SEE ALSO:")
                || line.starts_with("SUBCOMMANDS:")
            {
                println!("{BOLD}{YELLOW}{line}{RESET}");
            } else if line.starts_with("DEPRECATED:") {
                println!("{BOLD}{RED}{line}{RESET}");
            } else if line.starts_with("    #") {
                // Comment in examples
                println!("{CYAN}{line}{RESET}");
            } else if line.starts_with("    $") {
                // Command in examples
                println!("{GREEN}{line}{RESET}");
            } else if line.starts_with("error:") {
                println!("{BOLD}{RED}{line}{RESET}");
            } else {
                println!("{line}");
            }
        }

        Ok(())
    }
}

/// Simple Levenshtein distance for command suggestions
fn levenshtein(a: &str, b: &str) -> usize {
    let a_chars: Vec<char> = a.chars().collect();
    let b_chars: Vec<char> = b.chars().collect();
    let a_len = a_chars.len();
    let b_len = b_chars.len();

    if a_len == 0 {
        return b_len;
    }
    if b_len == 0 {
        return a_len;
    }

    let mut matrix = vec![vec![0usize; b_len + 1]; a_len + 1];

    for i in 0..=a_len {
        matrix[i][0] = i;
    }
    for j in 0..=b_len {
        matrix[0][j] = j;
    }

    for i in 1..=a_len {
        for j in 1..=b_len {
            let cost = if a_chars[i - 1] == b_chars[j - 1] {
                0
            } else {
                1
            };
            matrix[i][j] = std::cmp::min(
                std::cmp::min(matrix[i - 1][j] + 1, matrix[i][j - 1] + 1),
                matrix[i - 1][j - 1] + cost,
            );
        }
    }

    matrix[a_len][b_len]
}

/// Truncate string to max length with ellipsis
fn truncate_str(s: &str, max_len: usize) -> String {
    if s.len() <= max_len {
        s.to_string()
    } else {
        format!("{}...", &s[..max_len - 3])
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cli::registry::{CommandMetadata, ExampleMetadata, FlagMetadata};

    fn sample_registry() -> CommandRegistry {
        let mut registry = CommandRegistry::new("2.0.0");

        // Add global flags
        registry.register_global_flag(FlagMetadata {
            name: "verbose".to_string(),
            short: Some('v'),
            long: Some("verbose".to_string()),
            description: "Enable verbose output".to_string(),
            default: None,
        });

        // Add analyze command with subcommands
        let complexity_sub = CommandMetadata::builder("complexity")
            .short_description("Analyze code complexity")
            .long_description("Calculate cyclomatic complexity for all functions")
            .aliases(["cx"])
            .argument(crate::cli::registry::ArgumentMetadata {
                name: "project-path".to_string(),
                short: Some('p'),
                long: Some("project-path".to_string()),
                description: "Path to project".to_string(),
                required: false,
                default: Some(".".to_string()),
                value_type: ValueType::Path,
                ..Default::default()
            })
            .example(ExampleMetadata {
                description: "Analyze current directory".to_string(),
                command: "pmat analyze complexity".to_string(),
                ..Default::default()
            })
            .category("analysis")
            .build();

        registry.register(
            CommandMetadata::builder("analyze")
                .short_description("Analyze code metrics")
                .long_description("Run various code analysis tools")
                .aliases(["a", "an"])
                .subcommand(complexity_sub)
                .category("analysis")
                .tags(["quality", "metrics"])
                .related("context")
                .build(),
        );

        registry.register(
            CommandMetadata::builder("context")
                .short_description("Generate project context")
                .long_description("Generate AI-friendly project context using AST analysis")
                .aliases(["ctx"])
                .category("generation")
                .tags(["generation", "ast"])
                .related("analyze")
                .build(),
        );

        registry
    }

    #[test]
    fn test_help_generator_creation() {
        let registry = sample_registry();
        let gen = HelpGenerator::new(registry);
        assert!(gen.width > 0);
    }

    #[test]
    fn test_generate_overview() {
        let registry = sample_registry();
        let gen = HelpGenerator::new(registry);
        let overview = gen.generate_overview();

        assert!(overview.contains("pmat 2.0.0"));
        assert!(overview.contains("USAGE:"));
        assert!(overview.contains("COMMANDS:"));
        assert!(overview.contains("analyze"));
        assert!(overview.contains("context"));
    }

    #[test]
    fn test_generate_command_help() {
        let registry = sample_registry();
        let gen = HelpGenerator::new(registry);
        let help = gen.generate("analyze");

        assert!(help.contains("analyze"));
        assert!(help.contains("USAGE:"));
        assert!(help.contains("SUBCOMMANDS:"));
        assert!(help.contains("complexity"));
    }

    #[test]
    fn test_generate_subcommand_help() {
        let registry = sample_registry();
        let gen = HelpGenerator::new(registry);
        let help = gen.generate("analyze complexity");

        assert!(help.contains("complexity"));
        assert!(help.contains("cyclomatic complexity"));
        assert!(help.contains("OPTIONS:"));
        assert!(help.contains("--project-path"));
        assert!(help.contains("EXAMPLES:"));
    }

    #[test]
    fn test_generate_command_not_found() {
        let registry = sample_registry();
        let gen = HelpGenerator::new(registry);
        let help = gen.generate("nonexistent");

        assert!(help.contains("error:"));
        assert!(help.contains("unrecognized command"));
        assert!(help.contains("Did you mean:"));
    }

    #[test]
    fn test_find_by_alias() {
        let registry = sample_registry();
        let gen = HelpGenerator::new(registry);
        let help = gen.generate("ctx");

        assert!(help.contains("context"));
        assert!(help.contains("Generate project context"));
    }

    #[test]
    fn test_levenshtein_basic() {
        assert_eq!(levenshtein("analyze", "analyze"), 0);
        assert_eq!(levenshtein("analyze", "analize"), 1);
        assert_eq!(levenshtein("analyze", ""), 7);
        assert_eq!(levenshtein("", "test"), 4);
    }

    #[test]
    fn test_truncate_str() {
        assert_eq!(truncate_str("short", 10), "short");
        assert_eq!(truncate_str("this is a long string", 10), "this is...");
    }

    #[test]
    fn test_format_usage_simple() {
        let registry = sample_registry();
        let gen = HelpGenerator::new(registry);

        let cmd = CommandMetadata::builder("test")
            .argument(crate::cli::registry::ArgumentMetadata {
                name: "file".to_string(),
                positional: true,
                required: true,
                ..Default::default()
            })
            .build();

        let usage = gen.format_usage(&cmd);
        assert!(usage.contains("<FILE>"));
    }

    #[test]
    fn test_format_usage_with_options() {
        let registry = sample_registry();
        let gen = HelpGenerator::new(registry);

        let cmd = CommandMetadata::builder("test")
            .argument(crate::cli::registry::ArgumentMetadata {
                name: "verbose".to_string(),
                short: Some('v'),
                long: Some("verbose".to_string()),
                positional: false,
                ..Default::default()
            })
            .build();

        let usage = gen.format_usage(&cmd);
        assert!(usage.contains("[OPTIONS]"));
    }

    #[test]
    fn test_overview_contains_global_flags() {
        let registry = sample_registry();
        let gen = HelpGenerator::new(registry);
        let overview = gen.generate_overview();

        assert!(overview.contains("OPTIONS:"));
        assert!(overview.contains("-v, --verbose"));
    }
}

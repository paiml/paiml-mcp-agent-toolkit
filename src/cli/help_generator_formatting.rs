// help_generator_formatting.rs — included by help_generator.rs
// Contains HelpGenerator constructor, generation, and formatting methods.

impl HelpGenerator {
    /// Create a new HelpGenerator
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn new(registry: CommandRegistry) -> Self {
        Self {
            registry,
            color: std::io::stdout().is_terminal(),
            width: 80, // Default width, could use terminal_size crate if needed
        }
    }

    /// Create with explicit color setting
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn with_color(mut self, color: bool) -> Self {
        self.color = color;
        self
    }

    /// Create with explicit width
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn with_width(mut self, width: usize) -> Self {
        debug_assert!(width > 0, "width must be positive");
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
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn generate(&self, path: &str) -> String {
        debug_assert!(!path.is_empty(), "path must not be empty");
        match self.registry.find_command(path) {
            Some(metadata) => self.format_command_help(metadata),
            None => self.format_command_not_found(path),
        }
    }

    /// Generate top-level help (all commands overview)
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
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
        debug_assert!(true, "contract: format_command_help");
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
        debug_assert!(!path.is_empty(), "path must not be empty");
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
        debug_assert!(true, "contract: format_usage");
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
        debug_assert!(true, "contract: format_argument");
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
        debug_assert!(true, "contract: format_flag");
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
        debug_assert!(!query.is_empty(), "query must not be empty");
        debug_assert!(limit > 0, "limit must be positive");
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
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
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
        debug_assert!(!text.is_empty(), "text must not be empty");
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

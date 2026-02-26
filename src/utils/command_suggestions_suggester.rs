/// Command suggestion engine
pub struct CommandSuggester {
    main_commands: Vec<String>,
    analyze_subcommands: Vec<String>,
    common_mistakes: HashMap<String, String>,
}

fn find_closest(candidate: &str, commands: &[String], max_distance: usize) -> Option<String> {
    let mut best_match = None;
    let mut best_distance = usize::MAX;

    for cmd in commands {
        let distance = levenshtein_distance(candidate, cmd);
        if distance <= max_distance && distance < best_distance {
            best_distance = distance;
            best_match = Some(cmd.clone());
        }
    }

    best_match
}

impl CommandSuggester {
    #[must_use]
    pub fn new() -> Self {
        let main_commands = vec![
            "analyze".to_string(),
            "generate".to_string(),
            "scaffold".to_string(),
            "context".to_string(),
            "quality-gate".to_string(),
            "demo".to_string(),
            "agent".to_string(),
            "refactor".to_string(),
            "enforce".to_string(),
        ];

        let analyze_subcommands = vec![
            "complexity".to_string(),
            "satd".to_string(),
            "dead-code".to_string(),
            "tdg".to_string(),
            "churn".to_string(),
            "duplicates".to_string(),
        ];

        let mut common_mistakes = HashMap::new();
        common_mistakes.insert("agent analyze".to_string(), "analyze".to_string());
        common_mistakes.insert("analize".to_string(), "analyze".to_string());
        common_mistakes.insert("analyse".to_string(), "analyze".to_string());
        common_mistakes.insert("analize complexity".to_string(), "analyze complexity".to_string());
        common_mistakes.insert("analize satd".to_string(), "analyze satd".to_string());
        common_mistakes.insert("analize dead-code".to_string(), "analyze dead-code".to_string());
        common_mistakes.insert("complexity".to_string(), "analyze complexity".to_string());
        common_mistakes.insert("satd".to_string(), "analyze satd".to_string());
        common_mistakes.insert("dead-code".to_string(), "analyze dead-code".to_string());
        common_mistakes.insert("tdg".to_string(), "analyze tdg".to_string());
        common_mistakes.insert("std".to_string(), "satd".to_string());
        common_mistakes.insert("stad".to_string(), "satd".to_string());
        common_mistakes.insert("sadt".to_string(), "satd".to_string());

        Self { main_commands, analyze_subcommands, common_mistakes }
    }

    /// Get command suggestions for a failed command
    #[must_use]
    pub fn suggest_command(&self, failed_args: &[String]) -> Option<String> {
        if failed_args.is_empty() {
            return None;
        }

        if failed_args.len() == 1 && self.main_commands.contains(&failed_args[0]) {
            return None;
        }

        let input = failed_args.join(" ");
        if let Some(suggestion) = self.common_mistakes.get(&input) {
            return Some(format!("Did you mean 'pmat {suggestion}'?"));
        }

        if failed_args.len() == 2 {
            let combined = format!("{} {}", failed_args[0], failed_args[1]);
            if let Some(suggestion) = self.common_mistakes.get(&combined) {
                return Some(format!("Did you mean 'pmat {suggestion}'?"));
            }
        }

        if failed_args.len() == 1 {
            return self.suggest_single_arg(&failed_args[0]);
        }

        if failed_args.len() >= 2 && failed_args[0] == "analyze" {
            if let Some(sub) = find_closest(&failed_args[1], &self.analyze_subcommands, 2) {
                return Some(format!("Did you mean 'pmat analyze {sub}'?"));
            }
        }

        None
    }

    fn suggest_single_arg(&self, arg: &str) -> Option<String> {
        if self.analyze_subcommands.iter().any(|cmd| cmd == arg) {
            return Some(format!("Did you mean 'pmat analyze {arg}'?"));
        }

        if let Some(cmd) = find_closest(arg, &self.main_commands, 3) {
            return Some(format!("Did you mean 'pmat {cmd}'?"));
        }

        None
    }

    /// Get help text with working examples
    #[must_use]
    pub fn get_help_examples() -> String {
        let examples = vec![
            "# Analyze code complexity",
            "pmat analyze complexity --project-path .",
            "",
            "# Find technical debt",
            "pmat analyze satd --path .",
            "",
            "# Find dead code",
            "pmat analyze dead-code --path .",
            "",
            "# Generate project context",
            "pmat context",
            "",
            "# Run quality gates",
            "pmat quality-gate",
            "",
            "# Start agent daemon",
            "pmat agent start",
        ];

        format!("\nEXAMPLES:\n{}", examples.join("\n"))
    }
}

impl Default for CommandSuggester {
    fn default() -> Self {
        Self::new()
    }
}

// DriftDetector analysis methods — included from drift_detector.rs

impl DriftDetector {
    /// Create a new drift detector
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn new(registry: CommandRegistry) -> Self {
        Self {
            registry,
            // Match: pmat <command> [args]
            command_regex: Regex::new(r"pmat\s+([\w\-]+(?:\s+[\w\-]+)*)").expect("internal error"),
            // Match: ```bash\npmat ... or $ pmat ...
            code_block_regex: Regex::new(r"(?:```(?:bash|shell|sh)?\n|\$\s*)(pmat[^\n`]+)")
                .expect("internal error"),
        }
    }

    /// Detect drift in a markdown file
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "path_exists")]
    pub fn detect_in_file(&self, path: &Path) -> Result<Vec<DriftError>, std::io::Error> {
        let content = std::fs::read_to_string(path)?;
        let file_name = path.to_string_lossy().to_string();
        Ok(self.detect_in_content(&content, &file_name))
    }

    /// Detect drift in markdown content
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn detect_in_content(&self, content: &str, file_name: &str) -> Vec<DriftError> {
        let mut errors = Vec::new();

        // Track line numbers
        let lines: Vec<&str> = content.lines().collect();

        // 1. Find all pmat command references
        for (line_idx, line) in lines.iter().enumerate() {
            let line_num = line_idx + 1;

            for cap in self.command_regex.captures_iter(line) {
                let cmd_path = cap.get(1).expect("internal error").as_str();

                // Skip if it's inside a "deprecated" context
                let is_deprecated_context = line.to_lowercase().contains("deprecated");

                if !self.command_exists(cmd_path) {
                    errors.push(DriftError::NonExistentCommand {
                        mentioned: cmd_path.to_string(),
                        file: file_name.to_string(),
                        line: line_num,
                        suggestion: self.find_similar_command(cmd_path),
                    });
                } else if let Some(cmd) = self.registry.find_command(cmd_path) {
                    // Check if deprecated command documented without warning
                    if cmd.deprecated.is_some() && !is_deprecated_context {
                        errors.push(DriftError::DeprecatedWithoutWarning {
                            command: cmd_path.to_string(),
                            file: file_name.to_string(),
                            line: line_num,
                        });
                    }
                }
            }
        }

        // 2. Validate code block examples
        for cap in self.code_block_regex.captures_iter(content) {
            let example = cap.get(1).expect("internal error").as_str().trim();
            if let Some(error) = self.validate_example(example, file_name) {
                errors.push(error);
            }
        }

        errors
    }

    /// Generate full drift report for multiple files
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "path_exists")]
    pub fn generate_report(&self, paths: &[&Path]) -> DriftReport {
        let mut all_errors = Vec::new();
        let mut documented_commands = HashSet::new();

        for path in paths {
            if let Ok(content) = std::fs::read_to_string(path) {
                let file_name = path.to_string_lossy().to_string();
                let errors = self.detect_in_content(&content, &file_name);
                all_errors.extend(errors);

                // Track which commands are documented
                for cap in self.command_regex.captures_iter(&content) {
                    let cmd_path = cap.get(1).expect("internal error").as_str();
                    if self.command_exists(cmd_path) {
                        documented_commands.insert(cmd_path.to_string());
                    }
                }
            }
        }

        // Find undocumented commands
        let all_commands: HashSet<_> = self.registry.all_command_paths().into_iter().collect();
        let undocumented: HashSet<_> = all_commands
            .difference(&documented_commands)
            .filter(|cmd| self.is_user_facing(cmd))
            .cloned()
            .collect();

        let total = all_commands.len();
        let coverage = if total > 0 {
            (documented_commands.len() as f64 / total as f64) * 100.0
        } else {
            100.0
        };

        DriftReport {
            errors: all_errors,
            documented_commands,
            undocumented_commands: undocumented,
            total_commands: total,
            coverage,
        }
    }

    /// Check if command exists in registry
    fn command_exists(&self, path: &str) -> bool {
        self.registry.find_command(path).is_some()
    }

    /// Find similar command for suggestions
    fn find_similar_command(&self, query: &str) -> Option<String> {
        let all_commands = self.registry.all_command_paths();

        all_commands
            .into_iter()
            .min_by_key(|cmd| levenshtein(&cmd.to_lowercase(), &query.to_lowercase()))
            .filter(|cmd| levenshtein(&cmd.to_lowercase(), &query.to_lowercase()) <= 3)
    }

    /// Check if command is user-facing (should be documented)
    fn is_user_facing(&self, command: &str) -> bool {
        // Filter out internal commands
        if let Some(cmd) = self.registry.find_command(command) {
            !cmd.category.to_lowercase().contains("internal")
        } else {
            false
        }
    }

    /// Validate a command example
    fn validate_example(&self, example: &str, file_name: &str) -> Option<DriftError> {
        // Extract command from example
        let parts: Vec<&str> = example.split_whitespace().collect();
        if parts.len() < 2 || parts[0] != "pmat" {
            return None;
        }

        // Build command path (could be multi-word like "analyze complexity")
        let mut cmd_path = parts[1].to_string();
        if parts.len() > 2 && !parts[2].starts_with('-') {
            // Check if second word is a subcommand
            let extended_path = format!("{} {}", cmd_path, parts[2]);
            if self.command_exists(&extended_path) {
                cmd_path = extended_path;
            }
        }

        if !self.command_exists(&cmd_path) {
            return Some(DriftError::InvalidExample {
                example: example.to_string(),
                file: file_name.to_string(),
                line: 0, // Would need to track this
                reason: format!("command '{}' doesn't exist", cmd_path),
            });
        }

        None
    }
}

/// Levenshtein distance for suggestions
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

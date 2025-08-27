//! Command Suggestion System
//!
//! Provides "did you mean?" functionality for CLI commands using Levenshtein distance
//! and semantic command mapping to improve discoverability.

use std::collections::HashMap;

/// Calculate Levenshtein distance between two strings
pub fn levenshtein_distance(a: &str, b: &str) -> usize {
    let a_len = a.len();
    let b_len = b.len();

    if a_len == 0 {
        return b_len;
    }
    if b_len == 0 {
        return a_len;
    }

    let mut matrix = vec![vec![0; b_len + 1]; a_len + 1];

    // Initialize first row and column
    for i in 0..=a_len {
        matrix[i][0] = i;
    }
    for j in 0..=b_len {
        matrix[0][j] = j;
    }

    // Fill the matrix
    for i in 1..=a_len {
        for j in 1..=b_len {
            let cost = if a.chars().nth(i - 1) == b.chars().nth(j - 1) {
                0
            } else {
                1
            };

            matrix[i][j] = std::cmp::min(
                std::cmp::min(
                    matrix[i - 1][j] + 1, // deletion
                    matrix[i][j - 1] + 1, // insertion
                ),
                matrix[i - 1][j - 1] + cost, // substitution
            );
        }
    }

    matrix[a_len][b_len]
}

/// Command suggestion engine
pub struct CommandSuggester {
    main_commands: Vec<String>,
    analyze_subcommands: Vec<String>,
    common_mistakes: HashMap<String, String>,
}

impl CommandSuggester {
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

        // Common AI/user mistakes
        common_mistakes.insert("agent analyze".to_string(), "analyze".to_string());
        common_mistakes.insert("analize".to_string(), "analyze".to_string());
        common_mistakes.insert("analyse".to_string(), "analyze".to_string());
        common_mistakes.insert("complexity".to_string(), "analyze complexity".to_string());
        common_mistakes.insert("satd".to_string(), "analyze satd".to_string());
        common_mistakes.insert("dead-code".to_string(), "analyze dead-code".to_string());
        common_mistakes.insert("tdg".to_string(), "analyze tdg".to_string());

        // SATD typos
        common_mistakes.insert("std".to_string(), "satd".to_string());
        common_mistakes.insert("stad".to_string(), "satd".to_string());
        common_mistakes.insert("sadt".to_string(), "satd".to_string());

        Self {
            main_commands,
            analyze_subcommands,
            common_mistakes,
        }
    }

    /// Get command suggestions for a failed command
    pub fn suggest_command(&self, failed_args: &[String]) -> Option<String> {
        if failed_args.is_empty() {
            return None;
        }

        let input = failed_args.join(" ");

        // Check exact matches in common mistakes first
        if let Some(suggestion) = self.common_mistakes.get(&input) {
            return Some(format!("Did you mean 'pmat {}'?", suggestion));
        }

        // Handle two-argument patterns like "agent analyze"
        if failed_args.len() == 2 {
            let combined = format!("{} {}", failed_args[0], failed_args[1]);
            if let Some(suggestion) = self.common_mistakes.get(&combined) {
                return Some(format!("Did you mean 'pmat {}'?", suggestion));
            }
        }

        // Handle single argument patterns
        if failed_args.len() == 1 {
            let arg = &failed_args[0];

            // Check if it's a subcommand that needs "analyze" prefix
            if self.analyze_subcommands.iter().any(|cmd| cmd == arg) {
                return Some(format!("Did you mean 'pmat analyze {}'?", arg));
            }

            // Find closest main command using Levenshtein distance
            let mut best_match = None;
            let mut best_distance = usize::MAX;

            for cmd in &self.main_commands {
                let distance = levenshtein_distance(arg, cmd);
                // Only suggest if distance is reasonable (≤ 3 for most typos)
                if distance <= 3 && distance < best_distance {
                    best_distance = distance;
                    best_match = Some(cmd);
                }
            }

            if let Some(suggestion) = best_match {
                return Some(format!("Did you mean 'pmat {}'?", suggestion));
            }
        }

        // Handle analyze subcommand suggestions
        if failed_args.len() >= 2 && failed_args[0] == "analyze" {
            let subcmd = &failed_args[1];
            let mut best_match = None;
            let mut best_distance = usize::MAX;

            for cmd in &self.analyze_subcommands {
                let distance = levenshtein_distance(subcmd, cmd);
                if distance <= 2 && distance < best_distance {
                    best_distance = distance;
                    best_match = Some(cmd);
                }
            }

            if let Some(suggestion) = best_match {
                return Some(format!("Did you mean 'pmat analyze {}'?", suggestion));
            }
        }

        None
    }

    /// Get help text with working examples
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
            "pmat quality-gate --strict",
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_levenshtein_distance() {
        assert_eq!(levenshtein_distance("", ""), 0);
        assert_eq!(levenshtein_distance("", "abc"), 3);
        assert_eq!(levenshtein_distance("abc", ""), 3);
        assert_eq!(levenshtein_distance("abc", "abc"), 0);
        assert_eq!(levenshtein_distance("abc", "ab"), 1);
        assert_eq!(levenshtein_distance("analyze", "analize"), 1);
        assert_eq!(levenshtein_distance("satd", "std"), 2);
    }

    #[test]
    fn test_common_mistake_suggestions() {
        let suggester = CommandSuggester::new();

        // Test agent analyze mistake
        let result = suggester.suggest_command(&["agent".to_string(), "analyze".to_string()]);
        assert!(result.is_some());
        assert!(result.unwrap().contains("pmat analyze"));

        // Test analize typo
        let result = suggester.suggest_command(&["analize".to_string()]);
        assert!(result.is_some());
        assert!(result.unwrap().contains("analyze"));

        // Test missing analyze prefix
        let result = suggester.suggest_command(&["complexity".to_string()]);
        assert!(result.is_some());
        assert!(result.unwrap().contains("analyze complexity"));
    }

    #[test]
    fn test_analyze_subcommand_suggestions() {
        let suggester = CommandSuggester::new();

        // Test SATD typos in analyze context
        let result = suggester.suggest_command(&["analyze".to_string(), "std".to_string()]);
        assert!(result.is_some());
        assert!(result.unwrap().contains("satd"));
    }

    #[test]
    fn test_no_suggestion_for_valid_commands() {
        let suggester = CommandSuggester::new();

        // Valid commands should not get suggestions
        let result = suggester.suggest_command(&["analyze".to_string()]);
        assert!(result.is_none());
    }
}

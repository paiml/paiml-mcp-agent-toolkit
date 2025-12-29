//! Drift Detector - Detect documentation/code drift
//!
//! Detects when documentation (README, help text) diverges from actual code,
//! preventing the issue reported in GitHub #118 where `pmat mcp` was documented
//! but didn't exist.
//!
//! # Architecture (Toyota Way - Poka-yoke)
//!
//! Error-proofing through automated validation:
//! - Pre-commit hook validates all documentation references
//! - CI/CD blocks PRs with drift
//! - Build-time validation of examples
//!
//! # References
//!
//! - Specification: docs/specifications/unified-cli-mcp-help-integration.md
//! - GitHub Issue: #118
//! - Toyota Way: Poka-yoke (error-proofing), Jidoka (built-in quality)

use crate::cli::registry::CommandRegistry;
use regex::Regex;
use std::collections::HashSet;
use std::path::Path;

/// Drift detection errors
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DriftError {
    /// Command referenced in docs doesn't exist
    NonExistentCommand {
        mentioned: String,
        file: String,
        line: usize,
        suggestion: Option<String>,
    },
    /// Example in docs doesn't work
    InvalidExample {
        example: String,
        file: String,
        line: usize,
        reason: String,
    },
    /// Command exists but not documented
    UndocumentedCommand { command: String },
    /// Deprecated command still documented without warning
    DeprecatedWithoutWarning {
        command: String,
        file: String,
        line: usize,
    },
    /// Broken link in documentation
    BrokenLink {
        url: String,
        file: String,
        line: usize,
    },
}

impl std::fmt::Display for DriftError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::NonExistentCommand {
                mentioned,
                file,
                line,
                suggestion,
            } => {
                write!(
                    f,
                    "{}:{}: command '{}' doesn't exist",
                    file, line, mentioned
                )?;
                if let Some(s) = suggestion {
                    write!(f, " (did you mean '{}'?)", s)?;
                }
                Ok(())
            }
            Self::InvalidExample {
                example,
                file,
                line,
                reason,
            } => {
                write!(
                    f,
                    "{}:{}: invalid example '{}': {}",
                    file, line, example, reason
                )
            }
            Self::UndocumentedCommand { command } => {
                write!(f, "command '{}' is not documented", command)
            }
            Self::DeprecatedWithoutWarning {
                command,
                file,
                line,
            } => {
                write!(
                    f,
                    "{}:{}: deprecated command '{}' documented without deprecation notice",
                    file, line, command
                )
            }
            Self::BrokenLink { url, file, line } => {
                write!(f, "{}:{}: broken link '{}'", file, line, url)
            }
        }
    }
}

impl std::error::Error for DriftError {}

/// Drift detection result
#[derive(Debug)]
pub struct DriftReport {
    /// Detected errors
    pub errors: Vec<DriftError>,
    /// Commands found in documentation
    pub documented_commands: HashSet<String>,
    /// Commands in registry but not documented
    pub undocumented_commands: HashSet<String>,
    /// Total commands in registry
    pub total_commands: usize,
    /// Documentation coverage percentage
    pub coverage: f64,
}

impl DriftReport {
    /// Check if the report has any errors
    pub fn has_errors(&self) -> bool {
        !self.errors.is_empty()
    }

    /// Get error count
    pub fn error_count(&self) -> usize {
        self.errors.len()
    }

    /// Format as human-readable report
    pub fn to_string_report(&self) -> String {
        let mut report = String::new();

        report.push_str("Drift Detection Report\n");
        report.push_str("======================\n\n");

        report.push_str(&format!(
            "Commands: {} total, {} documented ({:.1}% coverage)\n",
            self.total_commands,
            self.documented_commands.len(),
            self.coverage
        ));

        if self.has_errors() {
            report.push_str(&format!("\n❌ {} errors detected:\n\n", self.errors.len()));
            for error in &self.errors {
                report.push_str(&format!("  • {}\n", error));
            }
        } else {
            report.push_str("\n✅ No drift detected\n");
        }

        if !self.undocumented_commands.is_empty() {
            report.push_str("\n⚠️ Undocumented commands:\n");
            for cmd in &self.undocumented_commands {
                report.push_str(&format!("  • {}\n", cmd));
            }
        }

        report
    }
}

/// Detects documentation drift
pub struct DriftDetector {
    registry: CommandRegistry,
    /// Regex to find pmat command references
    command_regex: Regex,
    /// Regex to find code blocks
    code_block_regex: Regex,
}

impl DriftDetector {
    /// Create a new drift detector
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
    pub fn detect_in_file(&self, path: &Path) -> Result<Vec<DriftError>, std::io::Error> {
        let content = std::fs::read_to_string(path)?;
        let file_name = path.to_string_lossy().to_string();
        Ok(self.detect_in_content(&content, &file_name))
    }

    /// Detect drift in markdown content
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cli::registry::{CommandMetadata, DeprecationInfo};

    fn sample_registry() -> CommandRegistry {
        let mut registry = CommandRegistry::new("2.0.0");

        let complexity_sub = CommandMetadata::builder("complexity")
            .short_description("Analyze complexity")
            .build();

        registry.register(
            CommandMetadata::builder("analyze")
                .short_description("Analyze code")
                .subcommand(complexity_sub)
                .category("analysis")
                .build(),
        );

        registry.register(
            CommandMetadata::builder("context")
                .short_description("Generate context")
                .aliases(["ctx"])
                .category("generation")
                .build(),
        );

        registry.register(
            CommandMetadata::builder("old-command")
                .short_description("Old deprecated command")
                .deprecated(DeprecationInfo {
                    since_version: "2.0.0".to_string(),
                    removal_version: Some("3.0.0".to_string()),
                    replacement: Some("new-command".to_string()),
                    reason: "Replaced".to_string(),
                })
                .category("internal")
                .build(),
        );

        registry
    }

    #[test]
    fn test_detector_creation() {
        let registry = sample_registry();
        let _detector = DriftDetector::new(registry);
    }

    #[test]
    fn test_detect_nonexistent_command() {
        let registry = sample_registry();
        let detector = DriftDetector::new(registry);

        let content = "Run `pmat mcp` to start the server";
        let errors = detector.detect_in_content(content, "README.md");

        assert_eq!(errors.len(), 1);
        assert!(
            matches!(&errors[0], DriftError::NonExistentCommand { mentioned, .. } if mentioned == "mcp")
        );
    }

    #[test]
    fn test_detect_valid_command() {
        let registry = sample_registry();
        let detector = DriftDetector::new(registry);

        let content = "Run `pmat analyze complexity` for metrics";
        let errors = detector.detect_in_content(content, "README.md");

        assert!(errors.is_empty());
    }

    #[test]
    fn test_detect_command_with_alias() {
        let registry = sample_registry();
        let detector = DriftDetector::new(registry);

        let content = "Use `pmat ctx` for quick context";
        let errors = detector.detect_in_content(content, "README.md");

        // ctx is an alias for context, should be valid
        assert!(errors.is_empty());
    }

    #[test]
    fn test_detect_invalid_example() {
        let registry = sample_registry();
        let detector = DriftDetector::new(registry);

        let content = r#"
```bash
pmat nonexistent --flag
```
"#;
        let errors = detector.detect_in_content(content, "README.md");

        assert!(!errors.is_empty());
    }

    #[test]
    fn test_suggest_similar_command() {
        let registry = sample_registry();
        let detector = DriftDetector::new(registry);

        let content = "Run `pmat analize` to check code"; // typo
        let errors = detector.detect_in_content(content, "README.md");

        assert_eq!(errors.len(), 1);
        if let DriftError::NonExistentCommand { suggestion, .. } = &errors[0] {
            assert_eq!(suggestion.as_deref(), Some("analyze"));
        }
    }

    #[test]
    fn test_generate_report() {
        let registry = sample_registry();
        let detector = DriftDetector::new(registry);

        // Create temp file
        let temp_dir = std::env::temp_dir();
        let temp_file = temp_dir.join("test_readme.md");
        std::fs::write(&temp_file, "Use `pmat analyze` and `pmat context`").expect("internal error");

        let report = detector.generate_report(&[temp_file.as_path()]);

        assert!(report.documented_commands.contains("analyze"));
        assert!(report.documented_commands.contains("context"));

        // Cleanup
        std::fs::remove_file(temp_file).ok();
    }

    #[test]
    fn test_drift_report_format() {
        let report = DriftReport {
            errors: vec![DriftError::NonExistentCommand {
                mentioned: "mcp".to_string(),
                file: "README.md".to_string(),
                line: 10,
                suggestion: None,
            }],
            documented_commands: HashSet::from(["analyze".to_string()]),
            undocumented_commands: HashSet::from(["internal".to_string()]),
            total_commands: 3,
            coverage: 33.3,
        };

        let formatted = report.to_string_report();
        assert!(formatted.contains("1 errors detected"));
        assert!(formatted.contains("33.3% coverage"));
    }

    #[test]
    fn test_levenshtein_for_suggestions() {
        assert_eq!(levenshtein("analyze", "analyze"), 0);
        assert_eq!(levenshtein("analyze", "analize"), 1);
        assert_eq!(levenshtein("analyze", "analyz"), 1);
    }
}

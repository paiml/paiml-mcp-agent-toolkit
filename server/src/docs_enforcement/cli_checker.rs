//! CLI Documentation Checker
//!
//! TICKET: PMAT-7001 Phase 2 (GREEN)
//!
//! This module validates that CLI commands have complete, accurate help text.
//! It checks that all flags are documented and descriptions are non-generic.

use crate::docs_enforcement::generic_detector::is_generic_description;
use anyhow::{Context, Result};
use std::collections::HashSet;
use std::process::Command;

/// CLI documentation validation result
#[derive(Debug, Clone)]
pub struct CliDocumentationReport {
    pub command: String,
    pub has_help: bool,
    pub has_usage_section: bool,
    pub has_options_section: bool,
    pub has_examples_section: bool,
    pub documented_flags: Vec<String>,
    pub generic_descriptions: Vec<String>,
    pub missing_descriptions: Vec<String>,
    pub issues: Vec<String>,
}

impl CliDocumentationReport {
    pub fn is_valid(&self) -> bool {
        self.has_help
            && self.has_usage_section
            && self.has_options_section
            && self.generic_descriptions.is_empty()
            && self.missing_descriptions.is_empty()
    }
}

/// Validate CLI documentation for a command
///
/// Checks that the command has:
/// - Working `--help` flag
/// - Usage section
/// - Options/FLAGS section
/// - Non-generic descriptions
/// - Examples (recommended)
pub fn validate_cli_documentation(
    binary_path: &str,
    command: &[&str],
) -> Result<CliDocumentationReport> {
    let mut report = CliDocumentationReport {
        command: command.join(" "),
        has_help: false,
        has_usage_section: false,
        has_options_section: false,
        has_examples_section: false,
        documented_flags: Vec::new(),
        generic_descriptions: Vec::new(),
        missing_descriptions: Vec::new(),
        issues: Vec::new(),
    };

    // Try to run --help
    let mut cmd = Command::new(binary_path);
    for arg in command {
        cmd.arg(arg);
    }
    cmd.arg("--help");

    let output = cmd.output()
        .context("Failed to execute command")?;

    if !output.status.success() {
        report.issues.push(format!(
            "Command failed with exit code: {}",
            output.status.code().unwrap_or(-1)
        ));
        return Ok(report);
    }

    report.has_help = true;

    // Parse help text
    let help_text = String::from_utf8_lossy(&output.stdout);

    // Check for required sections
    report.has_usage_section = help_text.contains("Usage:");
    report.has_options_section = help_text.contains("Options:") || help_text.contains("FLAGS:");
    report.has_examples_section = help_text.contains("EXAMPLE")
        || help_text.contains("Example")
        || help_text.contains("example");

    if !report.has_usage_section {
        report.issues.push("Missing 'Usage:' section".to_string());
    }

    if !report.has_options_section {
        report.issues.push("Missing 'Options:' or 'FLAGS:' section".to_string());
    }

    // Extract flags
    report.documented_flags = extract_flags_from_help(&help_text);

    // Check for generic descriptions
    for line in help_text.lines() {
        // Skip empty lines and section headers
        if line.trim().is_empty() || line.ends_with(':') {
            continue;
        }

        // Check if line looks like a flag description
        // Format: "  -f, --flag <VALUE>    Description here"
        if line.trim_start().starts_with('-') {
            // Extract description part (after flag definition)
            if let Some(desc_part) = line.split_whitespace().skip_while(|w| w.starts_with('-') || w.starts_with('<') || w.starts_with('[')).collect::<Vec<_>>().get(0..) {
                let description = desc_part.join(" ");
                if !description.is_empty() && is_generic_description(&description) {
                    report.generic_descriptions.push(format!(
                        "Flag '{}': {}",
                        extract_flag_name(line),
                        description
                    ));
                }
            }
        }
    }

    Ok(report)
}

/// Extract flag names from help text
///
/// Parses help output to find all documented flags.
/// Returns flags like ["--help", "--verbose", "-v", etc.]
fn extract_flags_from_help(help_text: &str) -> Vec<String> {
    let mut flags = Vec::new();
    let mut in_options_section = false;

    for line in help_text.lines() {
        // Detect start of options section
        if line.contains("Options:") || line.contains("FLAGS:") {
            in_options_section = true;
            continue;
        }

        // Detect end of options section (next section)
        if in_options_section && !line.trim().is_empty() && !line.starts_with(' ') {
            break;
        }

        if in_options_section {
            // Extract flags from line
            // Format: "  -f, --flag <VALUE>    Description"
            let trimmed = line.trim_start();
            if trimmed.starts_with('-') {
                // Parse flag names (could be multiple like "-v, --verbose")
                let flag_part = trimmed.split_whitespace()
                    .take_while(|w| w.starts_with('-') || w == &",")
                    .collect::<Vec<_>>();

                for token in flag_part {
                    if token.starts_with('-') {
                        // Remove trailing comma if present
                        let flag = token.trim_end_matches(',');
                        flags.push(flag.to_string());
                    }
                }
            }
        }
    }

    flags
}

/// Extract primary flag name from a help line
fn extract_flag_name(line: &str) -> String {
    let trimmed = line.trim_start();
    if let Some(long_flag) = trimmed.split_whitespace()
        .find(|w| w.starts_with("--"))
    {
        long_flag.trim_end_matches(',').to_string()
    } else if let Some(short_flag) = trimmed.split_whitespace()
        .find(|w| w.starts_with('-') && !w.starts_with("--"))
    {
        short_flag.trim_end_matches(',').to_string()
    } else {
        "unknown".to_string()
    }
}

/// Check if all expected flags are documented
///
/// Compares expected flags (from code) with documented flags (from --help).
/// Returns flags that are missing from documentation.
pub fn find_undocumented_flags(
    expected_flags: &[&str],
    documented_flags: &[String],
) -> Vec<String> {
    let documented_set: HashSet<String> = documented_flags.iter()
        .map(|f| f.to_string())
        .collect();

    expected_flags.iter()
        .filter(|flag| !documented_set.contains(&flag.to_string()))
        .map(|f| f.to_string())
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_flags_from_help() {
        let help_text = r#"
Usage: pmat scaffold agent [OPTIONS]

Options:
  -n, --name <NAME>      Agent name
  -t, --template <TEMPLATE>  Template type
  -v, --verbose          Enable verbose output
  -h, --help            Print help
"#;

        let flags = extract_flags_from_help(help_text);
        assert!(flags.contains(&"-n".to_string()));
        assert!(flags.contains(&"--name".to_string()));
        assert!(flags.contains(&"-t".to_string()));
        assert!(flags.contains(&"--template".to_string()));
        assert!(flags.contains(&"-v".to_string()));
        assert!(flags.contains(&"--verbose".to_string()));
    }

    #[test]
    fn test_find_undocumented_flags() {
        let expected = vec!["--name", "--template", "--output", "--force"];
        let documented = vec![
            "--name".to_string(),
            "--template".to_string(),
            "--output".to_string(),
        ];

        let missing = find_undocumented_flags(&expected, &documented);
        assert_eq!(missing, vec!["--force"]);
    }

    #[test]
    fn test_extract_flag_name() {
        assert_eq!(extract_flag_name("  -n, --name <NAME>"), "--name");
        assert_eq!(extract_flag_name("  -v, --verbose"), "--verbose");
        assert_eq!(extract_flag_name("  -h"), "-h");
    }
}

// cli_checker_validation.rs — Core validation logic for CLI documentation checking
// Included by cli_checker.rs — shares parent module scope (no `use` imports here)

/// Validate CLI documentation for a command
///
/// Checks that the command has:
/// - Working `--help` flag
/// - Usage section
/// - Options/FLAGS section
/// - Non-generic descriptions
/// - Examples (recommended)
#[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
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

    // Execute command and get help text
    let help_text = match execute_help_command(binary_path, command) {
        Ok(text) => {
            report.has_help = true;
            text
        }
        Err(exit_code) => {
            report
                .issues
                .push(format!("Command failed with exit code: {}", exit_code));
            return Ok(report);
        }
    };

    // Validate required sections
    validate_sections(&help_text, &mut report);

    // Extract flags
    report.documented_flags = extract_flags_from_help(&help_text);

    // Check for generic descriptions in flag help text
    report.generic_descriptions = find_generic_flag_descriptions(&help_text);

    Ok(report)
}

/// Execute command with --help flag and return output
fn execute_help_command(binary_path: &str, command: &[&str]) -> Result<String, i32> {
    let mut cmd = Command::new(binary_path);
    for arg in command {
        cmd.arg(arg);
    }
    cmd.arg("--help");

    let output = cmd.output().map_err(|_| -1)?;

    if !output.status.success() {
        return Err(output.status.code().unwrap_or(-1));
    }

    Ok(String::from_utf8_lossy(&output.stdout).to_string())
}

/// Validate presence of required sections and add issues if missing
fn validate_sections(help_text: &str, report: &mut CliDocumentationReport) {
    report.has_usage_section = help_text.contains("Usage:");
    report.has_options_section = help_text.contains("Options:") || help_text.contains("FLAGS:");
    report.has_examples_section = help_text.contains("EXAMPLE")
        || help_text.contains("Example")
        || help_text.contains("example");

    if !report.has_usage_section {
        report.issues.push("Missing 'Usage:' section".to_string());
    }

    if !report.has_options_section {
        report
            .issues
            .push("Missing 'Options:' or 'FLAGS:' section".to_string());
    }
}

/// Find generic descriptions in flag help text
fn find_generic_flag_descriptions(help_text: &str) -> Vec<String> {
    let mut generic_descriptions = Vec::new();

    for line in help_text.lines() {
        // Skip empty lines and section headers
        if line.trim().is_empty() || line.ends_with(':') {
            continue;
        }

        // Check if line looks like a flag description
        if line.trim_start().starts_with('-') {
            if let Some(description) = extract_description_from_flag_line(line) {
                if is_generic_description(&description) {
                    generic_descriptions.push(format!(
                        "Flag '{}': {}",
                        extract_flag_name(line),
                        description
                    ));
                }
            }
        }
    }

    generic_descriptions
}

/// Extract description part from a flag help line
fn extract_description_from_flag_line(line: &str) -> Option<String> {
    // Format: "  -f, --flag <VALUE>    Description here"
    let desc_part: Vec<&str> = line
        .split_whitespace()
        .skip_while(|w| w.starts_with('-') || w.starts_with('<') || w.starts_with('['))
        .collect();

    if desc_part.is_empty() {
        return None;
    }

    let description = desc_part.join(" ");
    if description.is_empty() {
        None
    } else {
        Some(description)
    }
}

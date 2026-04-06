// cli_checker_parsing.rs — Flag extraction and parsing helpers
// Included by cli_checker.rs — shares parent module scope (no `use` imports here)

/// Extract flag names from help text
///
/// Parses help output to find all documented flags.
/// Returns flags like ["--help", "--verbose", "-v", etc.]
fn extract_flags_from_help(help_text: &str) -> Vec<String> {
    let mut flags = Vec::new();
    let options_lines = extract_options_section_lines(help_text);

    for line in options_lines {
        if let Some(line_flags) = parse_flags_from_line(line) {
            flags.extend(line_flags);
        }
    }

    flags
}

/// Extract lines from the Options/FLAGS section
fn extract_options_section_lines(help_text: &str) -> Vec<&str> {
    let mut section_lines = Vec::new();
    let mut in_options_section = false;

    for line in help_text.lines() {
        // Detect start of options section
        if is_options_section_start(line) {
            in_options_section = true;
            continue;
        }

        // Detect end of options section (next section)
        if is_section_boundary(line, in_options_section) {
            break;
        }

        if in_options_section {
            section_lines.push(line);
        }
    }

    section_lines
}

/// Check if line marks the start of options section
fn is_options_section_start(line: &str) -> bool {
    line.contains("Options:") || line.contains("FLAGS:")
}

/// Check if line is a section boundary (end of current section)
fn is_section_boundary(line: &str, in_section: bool) -> bool {
    in_section && !line.trim().is_empty() && !line.starts_with(' ')
}

/// Parse flag names from a single help line
/// Format: "  -f, --flag <VALUE>    Description"
/// Returns: Some(vec!["-f", "--flag"]) or None
fn parse_flags_from_line(line: &str) -> Option<Vec<String>> {
    let trimmed = line.trim_start();
    if !trimmed.starts_with('-') {
        return None;
    }

    let flag_tokens: Vec<&str> = trimmed
        .split_whitespace()
        .take_while(|w| w.starts_with('-') || w == &",")
        .collect();

    let flags: Vec<String> = flag_tokens
        .iter()
        .filter(|token| token.starts_with('-'))
        .map(|token| token.trim_end_matches(',').to_string())
        .collect();

    if flags.is_empty() {
        None
    } else {
        Some(flags)
    }
}

/// Extract primary flag name from a help line
fn extract_flag_name(line: &str) -> String {
    let trimmed = line.trim_start();
    if let Some(long_flag) = trimmed.split_whitespace().find(|w| w.starts_with("--")) {
        long_flag.trim_end_matches(',').to_string()
    } else if let Some(short_flag) = trimmed
        .split_whitespace()
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
#[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
pub fn find_undocumented_flags(
    expected_flags: &[&str],
    documented_flags: &[String],
) -> Vec<String> {
    let documented_set: HashSet<String> = documented_flags.iter().map(|f| f.to_string()).collect();

    expected_flags
        .iter()
        .filter(|flag| !documented_set.contains(**flag))
        .map(|f| f.to_string())
        .collect()
}

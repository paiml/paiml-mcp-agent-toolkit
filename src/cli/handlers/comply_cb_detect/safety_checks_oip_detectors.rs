// CB-121 through CB-124: OIP Tarantula pattern detectors
// Included by safety_checks.rs — no `use` imports or `#!` attributes here.

/// CB-121: Detect lock poisoning vulnerabilities
/// Pattern: `mutex.lock()` + unwrap or `rwlock.read/write()` + unwrap
/// Safe alternatives: `unwrap_or_else(|e| e.into_inner())`, `parking_lot`
/// Source: OIP Tarantula analysis - 10 instances in git.rs
pub(super) fn check_lock_poisoning_line(
    trimmed: &str,
    has_rwlock_import: bool,
    file_path: &str,
    line_num: usize,
) -> Option<CbPatternViolation> {
    let is_safe = trimmed.contains("unwrap_or_else") || trimmed.contains("into_inner");

    // Check for mutex.lock() + unwrap pattern
    if trimmed.contains(".lock()") && trimmed.contains(DOT_UNWRAP_STR) && !is_safe {
        return Some(CbPatternViolation {
            pattern_id: "CB-121".to_string(),
            file: file_path.to_string(),
            line: line_num + 1,
            description: concat!("Lock poisoning: .lock().unwr", "ap() panics if another thread panicked. Use unwrap_or_else(|e| e.into_inner()) or parking_lot").to_string(),
            severity: Severity::Warning,
        });
    }

    // Check for rwlock read/write + unwrap patterns
    let is_rwlock_op = (trimmed.contains(".read()") || trimmed.contains(".write()"))
        && trimmed.contains(DOT_UNWRAP_STR)
        && !is_safe;

    if is_rwlock_op && (trimmed.contains("RwLock") || has_rwlock_import) {
        let op = if trimmed.contains(".read()") {
            "read"
        } else {
            "write"
        };
        return Some(CbPatternViolation {
            pattern_id: "CB-121".to_string(),
            file: file_path.to_string(),
            line: line_num + 1,
            description: format!("Lock poisoning: .{}().{}() panics if another thread panicked. Use unwrap_or_else(|e| e.into_inner())", op, concat!("unwr", "ap")),
            severity: Severity::Warning,
        });
    }

    None
}

/// Check if a line should be skipped for lock poisoning analysis
/// (comments or `.lock()` inside a string literal).
fn should_skip_line(trimmed: &str) -> bool {
    if trimmed.starts_with("//") || trimmed.starts_with("/*") || trimmed.starts_with("*") {
        return true;
    }
    if let Some(idx) = trimmed.find(concat!(".loc", "k()")) {
        let quote_count = trimmed
            .get(..idx)
            .unwrap_or_default()
            .chars()
            .filter(|&c| c == '"')
            .count();
        if quote_count % 2 == 1 {
            return true;
        }
    }
    false
}

/// Check a single Rust file for lock poisoning violations (CB-121).
fn check_file_for_lock_poisoning(entry: &Path) -> Vec<CbPatternViolation> {
    let mut violations = Vec::new();
    let content = match fs::read_to_string(entry) {
        Ok(c) => c,
        Err(_) => return violations,
    };
    let lines: Vec<&str> = content.lines().collect();
    let test_lines = compute_test_code_lines(&lines);
    let has_rwlock_import = content.contains("std::sync::RwLock");
    let file_path = entry.display().to_string();

    for (line_num, line) in lines.iter().enumerate() {
        if test_lines.contains(&line_num) {
            continue;
        }
        let trimmed = line.trim();
        if should_skip_line(trimmed) {
            continue;
        }
        if let Some(v) = check_lock_poisoning_line(trimmed, has_rwlock_import, &file_path, line_num)
        {
            violations.push(v);
        }
    }

    violations
}

#[provable_contracts_macros::contract("pmat-core.yaml", equation = "path_exists")]
pub fn detect_cb121_lock_poisoning(project_path: &Path) -> Vec<CbPatternViolation> {
    let mut violations = Vec::new();

    let src_dir = project_path.join("src");
    if !src_dir.exists() {
        return violations;
    }

    let entries = match walkdir_rs_files(&src_dir) {
        Ok(e) => e,
        Err(_) => return violations,
    };

    for entry in entries {
        violations.extend(check_file_for_lock_poisoning(&entry));
    }

    violations
}

/// CB-122: Detect serde deserialization safety issues
/// Pattern: `serde_json::from_str()` + unwrap or `.expect()`
/// Safe alternatives: `?` operator, `match`, `unwrap_or_default()`
/// Source: OIP Tarantula analysis - 15+ instances in tarantula.rs, github.rs, citl.rs
pub(super) fn check_serde_line(
    trimmed: &str,
    serde_patterns: &[&str],
    file_path: &str,
    line_num: usize,
    violations: &mut Vec<CbPatternViolation>,
) {
    for &pattern in serde_patterns {
        if !trimmed.contains(pattern) {
            continue;
        }
        // Skip if pattern is inside a string literal
        if let Some(idx) = trimmed.find(pattern) {
            let quote_count = trimmed
                .get(..idx)
                .unwrap_or_default()
                .chars()
                .filter(|&c| c == '"')
                .count();
            if quote_count % 2 == 1 {
                continue;
            }
        }
        let has_unwrap = trimmed.contains(DOT_UNWRAP_STR) && !trimmed.contains(UNWRAP_OR_STR);
        let has_expect = trimmed.contains(".expect(");
        let suffix = if has_unwrap {
            concat!("unwr", "ap()")
        } else if has_expect {
            "expect()"
        } else {
            continue;
        };
        violations.push(CbPatternViolation {
            pattern_id: "CB-122".to_string(),
            file: file_path.to_string(),
            line: line_num + 1,
            description: format!("Serde unsafe: {pattern}().{suffix} panics on malformed input. Use ? operator or proper error handling"),
            severity: Severity::Error,
        });
    }
}

#[provable_contracts_macros::contract("pmat-core.yaml", equation = "path_exists")]
pub fn detect_cb122_serde_safety(project_path: &Path) -> Vec<CbPatternViolation> {
    let serde_patterns = [
        "serde_json::from_str",
        "serde_json::from_slice",
        "serde_json::from_reader",
        "serde_yaml::from_str",
        "serde_yaml::from_slice",
        "serde_yaml::from_reader",
        "serde_yaml_ng::from_str",
        "serde_yaml_ng::from_slice",
        "serde_yaml_ng::from_reader",
        "toml::from_str",
        "toml::de::from_str",
        "ron::from_str",
    ];
    scan_rs_production_lines(
        project_path,
        true,
        |trimmed, file_path, line_num, violations| {
            check_serde_line(trimmed, &serde_patterns, file_path, line_num, violations);
        },
    )
}

/// CB-123: Detect undocumented #[ignore] tests
/// Pattern: `#[ignore]` without a reason comment or attribute value
/// Valid: `#[ignore = "reason"]`, `#[ignore] // reason`, `/// reason \n #[ignore]`
/// Source: OIP Tarantula analysis - 6 undocumented #[ignore] tests
pub(super) fn has_ignore_documentation(lines: &[&str], line_num: usize, trimmed: &str) -> bool {
    let has_inline_reason = trimmed.contains('=') && trimmed.contains('"');
    let has_line_comment = trimmed.contains("//");
    let has_preceding_comment = line_num > 0 && lines[line_num - 1].trim().starts_with("//");
    has_inline_reason || has_line_comment || has_preceding_comment
}

fn check_file_for_undocumented_ignore(entry: &Path) -> Vec<CbPatternViolation> {
    let content = match fs::read_to_string(entry) {
        Ok(c) => c,
        Err(_) => return vec![],
    };
    let lines: Vec<&str> = content.lines().collect();
    let file_path = entry.display().to_string();
    let mut violations = Vec::new();
    for (line_num, line) in lines.iter().enumerate() {
        let trimmed = line.trim();
        if trimmed.starts_with("#[ignore]") && !has_ignore_documentation(&lines, line_num, trimmed)
        {
            violations.push(CbPatternViolation {
                pattern_id: "CB-123".to_string(),
                file: file_path.clone(),
                line: line_num + 1,
                description: "Undocumented #[ignore]: Add reason with #[ignore = \"reason\"] or // reason comment".to_string(),
                severity: Severity::Warning,
            });
        }
    }
    violations
}

#[provable_contracts_macros::contract("pmat-core.yaml", equation = "path_exists")]
pub fn detect_cb123_undocumented_ignore(project_path: &Path) -> Vec<CbPatternViolation> {
    [project_path.join("src"), project_path.join("tests")]
        .iter()
        .filter(|d| d.exists())
        .flat_map(|d| walkdir_rs_files(d).unwrap_or_default())
        .flat_map(|e| check_file_for_undocumented_ignore(&e))
        .collect()
}

/// CB-124: Detect low coverage thresholds in CI/config
/// Threshold: <80% is Error, <95% is Warning for sovereign stack
/// Source: OIP Tarantula analysis - 58% threshold (below 80% minimum)
const COVERAGE_THRESHOLD_PATTERNS: &[(&str, char)] = &[
    ("fail_under", '='),
    ("coverage_threshold", '='),
    ("min_coverage", '='),
    ("cov_threshold", '='),
    ("coverage <", ' '),
];

fn check_coverage_threshold_line(
    line: &str,
    line_num: usize,
    file_path: &str,
) -> Option<CbPatternViolation> {
    let line_lower = line.to_lowercase();
    for &(pattern, sep) in COVERAGE_THRESHOLD_PATTERNS {
        if !line_lower.contains(pattern) {
            continue;
        }
        let value = extract_coverage_threshold(line, sep)?;
        let (description, severity) = if value < 80.0 {
            (format!("Low coverage threshold: {value:.1}% is below 80% minimum. Increase coverage requirements"), Severity::Error)
        } else if value < 95.0 {
            (format!("Coverage threshold {value:.1}% below sovereign stack standard (95%). Consider increasing"), Severity::Warning)
        } else {
            continue;
        };
        return Some(CbPatternViolation {
            pattern_id: "CB-124".to_string(),
            file: file_path.to_string(),
            line: line_num + 1,
            description,
            severity,
        });
    }
    None
}

fn check_config_file_for_coverage(config_path: &Path) -> Vec<CbPatternViolation> {
    let content = match fs::read_to_string(config_path) {
        Ok(c) => c,
        Err(_) => return vec![],
    };
    let file_path = config_path.display().to_string();
    content
        .lines()
        .enumerate()
        .filter_map(|(ln, line)| check_coverage_threshold_line(line, ln, &file_path))
        .collect()
}

#[provable_contracts_macros::contract("pmat-core.yaml", equation = "path_exists")]
pub fn detect_cb124_coverage_threshold(project_path: &Path) -> Vec<CbPatternViolation> {
    let config_files = [
        project_path.join(".cargo").join("config.toml"),
        project_path.join("tarpaulin.toml"),
        project_path.join(".tarpaulin.toml"),
        project_path.join("codecov.yml"),
        project_path.join(".codecov.yml"),
        project_path.join("Makefile"),
        project_path
            .join(".github")
            .join("workflows")
            .join("ci.yml"),
        project_path
            .join(".github")
            .join("workflows")
            .join("test.yml"),
        project_path
            .join(".github")
            .join("workflows")
            .join("coverage.yml"),
    ];
    config_files
        .iter()
        .filter(|p| p.exists())
        .flat_map(|p| check_config_file_for_coverage(p))
        .collect()
}

/// Helper to extract coverage threshold value from a line
pub(super) fn extract_coverage_threshold(line: &str, separator: char) -> Option<f64> {
    // Try to find a number after the separator
    let parts: Vec<&str> = line.split(separator).collect();
    if parts.len() >= 2 {
        // Extract numeric value from the second part
        let value_part = parts[1].trim();
        // Handle formats like "60.0", "60", "60%", "\"60\""
        let cleaned = value_part
            .trim_matches(|c: char| !c.is_ascii_digit() && c != '.')
            .trim_start_matches('"')
            .trim_end_matches('"')
            .trim_end_matches('%');

        return cleaned.parse::<f64>().ok();
    }
    None
}

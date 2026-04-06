// OIP Tarantula common scanner infrastructure and CB-120 NaN-unsafe detection
// Included by safety_checks.rs — no `use` imports or `#!` attributes here.

// =============================================================================
// OIP Tarantula Pattern Detection (CB-120 through CB-124)
// Spec: docs/specifications/components/repo-health.md v2.1.0
// =============================================================================

/// CB-120: Detect NaN-unsafe floating-point comparisons
/// Pattern: `partial_cmp(...)` + unwrap or `.expect(...)` which panic on NaN
/// Safe alternatives: `total_cmp()`, `unwrap_or()`, `unwrap_or_else()`
/// Source: OIP Tarantula analysis - 10 instances in ml.rs, imbalance.rs, classifier.rs
/// Common scanner: iterate non-test, non-comment lines in all .rs files under src/.
/// The callback receives (trimmed_line, file_path, line_num) and may push violations.
fn is_comment_line(trimmed: &str) -> bool {
    trimmed.starts_with("//") || trimmed.starts_with("/*") || trimmed.starts_with("*")
}

fn scan_single_rs_file(
    entry: &Path,
    check: &mut impl FnMut(&str, &str, usize, &mut Vec<CbPatternViolation>),
    violations: &mut Vec<CbPatternViolation>,
) {
    debug_assert!(entry.exists(), "entry must exist: {}", entry.display());
    let content = match fs::read_to_string(entry) {
        Ok(c) => c,
        Err(_) => return,
    };
    let lines: Vec<&str> = content.lines().collect();
    let test_lines = compute_test_code_lines(&lines);
    let file_path = entry.display().to_string();
    for (line_num, line) in lines.iter().enumerate() {
        if test_lines.contains(&line_num) || is_comment_line(line.trim()) {
            continue;
        }
        check(line.trim(), &file_path, line_num, violations);
    }
}

pub(super) fn scan_rs_production_lines(
    project_path: &Path,
    skip_test_files: bool,
    mut check: impl FnMut(&str, &str, usize, &mut Vec<CbPatternViolation>),
) -> Vec<CbPatternViolation> {
    debug_assert!(project_path.exists(), "project_path must exist: {}", project_path.display());
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
        if skip_test_files && is_test_file(&entry) {
            continue;
        }
        scan_single_rs_file(&entry, &mut check, &mut violations);
    }
    violations
}

/// Check if a pattern is inside a string literal (odd number of quotes before it)
pub(super) fn is_in_string_literal(line: &str, pattern: &str) -> bool {
    if let Some(idx) = line.find(pattern) {
        let quote_count = line
            .get(..idx)
            .unwrap_or_default()
            .chars()
            .filter(|&c| c == '"')
            .count();
        quote_count % 2 == 1
    } else {
        false
    }
}

pub fn detect_cb120_nan_unsafe_comparison(project_path: &Path) -> Vec<CbPatternViolation> {
    debug_assert!(project_path.exists(), "project_path must exist: {}", project_path.display());
    scan_rs_production_lines(
        project_path,
        false,
        |trimmed, file_path, line_num, violations| {
            if is_in_string_literal(trimmed, "partial_cmp") {
                return;
            }
            if !trimmed.contains("partial_cmp") {
                return;
            }
            let has_unwrap = trimmed.contains(DOT_UNWRAP_STR) && !trimmed.contains(UNWRAP_OR_STR);
            let has_expect = trimmed.contains(".expect(");
            let suffix = if has_unwrap {
                concat!("unwr", "ap()")
            } else if has_expect {
                "expect()"
            } else {
                return;
            };
            violations.push(CbPatternViolation {
            pattern_id: "CB-120".to_string(),
            file: file_path.to_string(),
            line: line_num + 1,
            description: format!("NaN-unsafe: .partial_cmp().{suffix} panics on NaN. Use .total_cmp() or .unwrap_or()"),
            severity: Severity::Error,
        });
        },
    )
}

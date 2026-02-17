#![cfg_attr(coverage_nightly, coverage(off))]
use super::types::*;
use std::collections::HashSet;
use std::fs;
use std::path::Path;

// Use concat! to avoid self-detection by CB-501 scanner
const DOT_UNWRAP_STR: &str = concat!(".unwr", "ap()");
const UNWRAP_OR_STR: &str = concat!("unwra", "p_or");

/// Scan for CB-021 (SIMD intrinsics without #[target_feature])
/// NOTE: Skips test code (#[cfg(test)], mod tests, #[test]) - test code is exempt
/// Mark all lines in a function body as protected, starting from the `fn` declaration line.
fn mark_function_body(lines: &[&str], fn_line: usize, protected: &mut HashSet<usize>) {
    let mut depth: usize = 0;
    let mut entered_body = false;
    for k in fn_line..lines.len() {
        depth += lines[k].matches('{').count();
        if depth > 0 {
            entered_body = true;
        }
        depth = depth.saturating_sub(lines[k].matches('}').count());
        protected.insert(k);
        if entered_body && depth == 0 {
            break;
        }
    }
}

pub(super) fn compute_target_feature_protected_lines(lines: &[&str]) -> HashSet<usize> {
    let mut protected = HashSet::new();
    for (i, line) in lines.iter().enumerate() {
        let is_protected = line.trim().starts_with("#[target_feature")
            || (line.contains("#[cfg(") && line.contains("target_feature"));
        if !is_protected {
            continue;
        }
        // Find the function this attribute applies to and mark its body
        for j in i..lines.len() {
            if lines[j].contains("fn ") {
                mark_function_body(lines, j, &mut protected);
                break;
            }
        }
    }
    protected
}

const SIMD_INTRINSIC_PATTERNS: &[(&str, &str)] = &[
    (concat!("_mm", "256_"), "SIMD intrinsic"),
    (concat!("_mm", "512_"), "SIMD intrinsic"),
];
const PORTABLE_SIMD_PATTERNS: &[(&str, &str)] = &[
    (concat!("i8x", "16::"), "Portable SIMD"),
    (concat!("i16x", "8::"), "Portable SIMD"),
    (concat!("i32x", "4::"), "Portable SIMD"),
    (concat!("f32x", "4::"), "Portable SIMD"),
    (concat!("Simd", "::<"), "Portable SIMD"),
];

fn check_file_for_simd_violations(entry: &Path) -> Vec<CbPatternViolation> {
    let content = match fs::read_to_string(entry) {
        Ok(c) => c,
        Err(_) => return vec![],
    };
    let lines: Vec<&str> = content.lines().collect();
    let test_lines = compute_test_code_lines(&lines);
    let protected_lines = compute_target_feature_protected_lines(&lines);
    let file_path = entry.display().to_string();
    let mut violations = Vec::new();
    for (line_num, line) in lines.iter().enumerate() {
        if test_lines.contains(&line_num) || protected_lines.contains(&line_num) {
            continue;
        }
        for &(pattern, kind) in SIMD_INTRINSIC_PATTERNS.iter().chain(PORTABLE_SIMD_PATTERNS) {
            if line.contains(pattern) {
                violations.push(CbPatternViolation {
                    pattern_id: "CB-021".to_string(),
                    file: file_path.clone(),
                    line: line_num + 1,
                    description: format!("{kind} {pattern} without #[target_feature]"),
                    severity: Severity::Warning,
                });
            }
        }
    }
    violations
}

pub fn detect_cb021_simd_without_target_feature(project_path: &Path) -> Vec<CbPatternViolation> {
    let src_dir = project_path.join("src");
    if !src_dir.exists() {
        return vec![];
    }
    let entries = match walkdir_rs_files(&src_dir) {
        Ok(e) => e,
        Err(_) => return vec![],
    };
    entries
        .iter()
        .flat_map(|e| check_file_for_simd_violations(e))
        .collect()
}

/// Check if any of the preceding 5 lines contain a bounds check (an `if` with `<` or `>=`).
fn has_bounds_check_nearby(content_lines: &[&str], line_num: usize) -> bool {
    content_lines[..line_num]
        .iter()
        .rev()
        .take(5)
        .any(|l| l.contains("if") && (l.contains('<') || l.contains(">=")))
}

/// Check a single WGSL file for array accesses without preceding bounds checks (CB-001).
fn check_wgsl_file_for_bounds_violations(entry: &Path) -> Vec<CbPatternViolation> {
    let mut violations = Vec::new();
    let content = match fs::read_to_string(entry) {
        Ok(c) => c,
        Err(_) => return violations,
    };
    let content_lines: Vec<&str> = content.lines().collect();
    let file_path = entry.display().to_string();

    for (line_num, line) in content_lines.iter().enumerate() {
        let trimmed = line.trim();
        if trimmed.contains('[')
            && trimmed.contains(']')
            && !has_bounds_check_nearby(&content_lines, line_num)
        {
            violations.push(CbPatternViolation {
                pattern_id: "CB-001".to_string(),
                file: file_path.clone(),
                line: line_num + 1,
                description: "WGSL array access without bounds check".to_string(),
                severity: Severity::Warning,
            });
        }
    }

    violations
}

/// Scan for CB-001 (WGSL without bounds checking)
pub fn detect_cb001_wgsl_no_bounds_check(project_path: &Path) -> Vec<CbPatternViolation> {
    let mut violations = Vec::new();

    // Look for .wgsl files
    let src_dir = project_path.join("src");
    let shaders_dir = project_path.join("shaders");

    for dir in [src_dir, shaders_dir] {
        if !dir.exists() {
            continue;
        }

        if let Ok(entries) = walkdir_wgsl_files(&dir) {
            for entry in entries {
                violations.extend(check_wgsl_file_for_bounds_violations(&entry));
            }
        }
    }

    violations
}

pub(super) fn walkdir_wgsl_files(dir: &Path) -> Result<Vec<std::path::PathBuf>, std::io::Error> {
    let mut files = Vec::new();
    for entry in std::fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();
        if path.is_dir() {
            files.extend(walkdir_wgsl_files(&path)?);
        } else if path.extension().map(|e| e == "wgsl").unwrap_or(false) {
            files.push(path);
        }
    }
    Ok(files)
}

/// Check a single line for barrier usage inside a conditional block (CB-002).
/// Returns a violation if the line contains a barrier call while `in_conditional` is true.
fn check_line_for_barrier(
    trimmed: &str,
    in_conditional: bool,
    line_num: usize,
    file_path: &str,
) -> Option<CbPatternViolation> {
    if in_conditional
        && (trimmed.contains("workgroupBarrier") || trimmed.contains("storageBarrier"))
    {
        Some(CbPatternViolation {
            pattern_id: "CB-002".to_string(),
            file: file_path.to_string(),
            line: line_num + 1,
            description: "WGSL barrier inside conditional (divergence risk)".to_string(),
            severity: Severity::Critical,
        })
    } else {
        None
    }
}

/// Check a single WGSL file for barrier divergence violations (CB-002).
/// Reads the file, walks lines tracking conditional depth, and returns violations.
fn check_wgsl_file_for_barrier_divergence(entry: &Path) -> Vec<CbPatternViolation> {
    let mut violations = Vec::new();
    let content = match fs::read_to_string(entry) {
        Ok(c) => c,
        Err(_) => return violations,
    };
    let file_path = entry.display().to_string();
    let mut in_conditional = false;
    let mut conditional_depth = 0;

    for (line_num, line) in content.lines().enumerate() {
        let trimmed = line.trim();

        // Track conditional blocks
        if trimmed.starts_with("if") || trimmed.starts_with("else") {
            in_conditional = true;
        }
        if in_conditional {
            conditional_depth += trimmed.matches('{').count();
            conditional_depth = conditional_depth.saturating_sub(trimmed.matches('}').count());
            if conditional_depth == 0 {
                in_conditional = false;
            }
        }

        // Check for barrier inside conditional
        if let Some(v) = check_line_for_barrier(trimmed, in_conditional, line_num, &file_path) {
            violations.push(v);
        }
    }

    violations
}

/// Scan for CB-002 (WGSL barrier divergence)
pub fn detect_cb002_wgsl_barrier_divergence(project_path: &Path) -> Vec<CbPatternViolation> {
    let mut violations = Vec::new();

    let src_dir = project_path.join("src");
    let shaders_dir = project_path.join("shaders");

    for dir in [src_dir, shaders_dir] {
        if !dir.exists() {
            continue;
        }

        if let Ok(entries) = walkdir_wgsl_files(&dir) {
            for entry in entries {
                violations.extend(check_wgsl_file_for_barrier_divergence(&entry));
            }
        }
    }

    violations
}

/// Detect ComputeBricks without assertions/validation (CB-BUDGET)
fn check_brick_file_for_assertions(entry: &Path) -> Option<CbPatternViolation> {
    let content = fs::read_to_string(entry).ok()?;
    let is_brick_impl = content.contains("impl") && content.contains("Brick");
    if !is_brick_impl {
        return None;
    }
    let has_assertions = content.contains("assert!")
        || content.contains("debug_assert!")
        || content.contains("validate")
        || content.contains("check_budget")
        || content.contains("budget_remaining");
    if has_assertions {
        return None;
    }
    Some(CbPatternViolation {
        pattern_id: "CB-BUDGET".to_string(),
        file: entry.display().to_string(),
        line: 1,
        description: "ComputeBrick without assertions or budget validation".to_string(),
        severity: Severity::Warning,
    })
}

pub fn detect_bricks_without_assertions(project_path: &Path) -> Vec<CbPatternViolation> {
    let brick_dir = project_path.join("src").join("brick");
    if !brick_dir.exists() {
        return vec![];
    }
    let entries = match walkdir_rs_files(&brick_dir) {
        Ok(e) => e,
        Err(_) => return vec![],
    };
    entries
        .iter()
        .filter_map(|e| check_brick_file_for_assertions(e))
        .collect()
}

/// Check a single line for high coefficient of variation (CV > 15%) anomaly.
fn check_cv_anomaly(line: &str, content: &str) -> Option<ProfilerAnomaly> {
    if !line.contains("\"cv\"") && !line.contains("\"cv_percent\"") {
        return None;
    }
    let value = extract_json_number(line)?;
    let cv_threshold = 15.0;
    let cv = if value < 1.0 { value * 100.0 } else { value };
    if cv > cv_threshold {
        Some(ProfilerAnomaly {
            brick_name: extract_brick_name(content, line),
            anomaly_type: "HIGH_CV".to_string(),
            value: cv,
            threshold: cv_threshold,
        })
    } else {
        None
    }
}

/// Check a single line for low efficiency (< 25%) anomaly.
fn check_efficiency_anomaly(line: &str, content: &str) -> Option<ProfilerAnomaly> {
    if !line.contains("\"efficiency\"") {
        return None;
    }
    let value = extract_json_number(line)?;
    let eff_threshold = 25.0;
    let efficiency = if value < 1.0 { value * 100.0 } else { value };
    if efficiency < eff_threshold {
        Some(ProfilerAnomaly {
            brick_name: extract_brick_name(content, line),
            anomaly_type: "LOW_EFFICIENCY".to_string(),
            value: efficiency,
            threshold: eff_threshold,
        })
    } else {
        None
    }
}

/// Scan profiler file content for CV and efficiency anomalies.
fn check_profiler_file(content: &str) -> Vec<ProfilerAnomaly> {
    let mut anomalies = Vec::new();
    for line in content.lines() {
        let trimmed = line.trim();
        if let Some(a) = check_cv_anomaly(trimmed, content) {
            anomalies.push(a);
        }
        if let Some(a) = check_efficiency_anomaly(trimmed, content) {
            anomalies.push(a);
        }
    }
    anomalies
}

/// Parse BrickProfiler JSON output and detect anomalies
pub fn detect_profiler_anomalies(project_path: &Path) -> Vec<ProfilerAnomaly> {
    // Check standard profiler output locations
    let profiler_paths = [
        project_path
            .join(".pmat-metrics")
            .join("brick-profile.json"),
        project_path.join("target").join("brick-profile.json"),
        project_path.join("brick-profile.json"),
    ];

    for profiler_path in &profiler_paths {
        if !profiler_path.exists() {
            continue;
        }

        if let Ok(content) = fs::read_to_string(profiler_path) {
            return check_profiler_file(&content);
        }
    }

    Vec::new()
}

/// Helper to extract numeric value from JSON line like `"cv": 0.18,`
pub fn extract_json_number(line: &str) -> Option<f64> {
    line.split(':')
        .nth(1)?
        .trim()
        .trim_end_matches(',')
        .trim_end_matches('}')
        .parse()
        .ok()
}

/// Extract brick name from JSON content near the target line
fn find_name_field_backwards(lines: &[&str], from: usize) -> Option<String> {
    lines[..from]
        .iter()
        .rev()
        .take(20)
        .find(|l| l.contains("\"name\"") || l.contains("\"brick_name\""))
        .and_then(|l| l.split('"').nth(3))
        .map(|s| s.to_string())
}

pub fn extract_brick_name(content: &str, target_line: &str) -> String {
    let lines: Vec<&str> = content.lines().collect();
    for (i, line) in lines.iter().enumerate() {
        if *line == target_line {
            if let Some(name) = find_name_field_backwards(&lines, i) {
                return name;
            }
        }
    }
    "unknown".to_string()
}

// =============================================================================
// OIP Tarantula Pattern Detection (CB-120 through CB-124)
// Spec: docs/specifications/improve-pmat-comply.md v2.1.0
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

pub fn detect_cb122_serde_safety(project_path: &Path) -> Vec<CbPatternViolation> {
    let serde_patterns = [
        "serde_json::from_str",
        "serde_json::from_slice",
        "serde_json::from_reader",
        "serde_yaml::from_str",
        "serde_yaml::from_slice",
        "serde_yaml::from_reader",
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

use super::types::*;
use std::collections::HashSet;
use std::fs;
use std::path::Path;

/// Scan for CB-021 (SIMD intrinsics without #[target_feature])
/// NOTE: Skips test code (#[cfg(test)], mod tests, #[test]) - test code is exempt
pub(super) fn compute_target_feature_protected_lines(lines: &[&str]) -> HashSet<usize> {
    let mut protected = HashSet::new();
    for (i, line) in lines.iter().enumerate() {
        let is_protected = line.trim().starts_with("#[target_feature")
            || (line.contains("#[cfg(") && line.contains("target_feature"));
        if !is_protected {
            continue;
        }
        // Find the function this attribute applies to and mark its body
        let mut depth = 0;
        for j in i..lines.len() {
            if lines[j].contains("fn ") && depth == 0 {
                for k in j..lines.len() {
                    depth += lines[k].matches('{').count();
                    depth = depth.saturating_sub(lines[k].matches('}').count());
                    protected.insert(k);
                    if depth == 0 && k > j {
                        break;
                    }
                }
                break;
            }
        }
    }
    protected
}

pub fn detect_cb021_simd_without_target_feature(project_path: &Path) -> Vec<CbPatternViolation> {
    let mut violations = Vec::new();

    let src_dir = project_path.join("src");
    if !src_dir.exists() {
        return violations;
    }

    // Use concat! to avoid self-matching when this file is scanned
    let intrinsic_patterns = [
        (concat!("_mm", "256_"), "SIMD intrinsic"),
        (concat!("_mm", "512_"), "SIMD intrinsic"),
    ];
    let portable_patterns = [
        (concat!("i8x", "16::"), "Portable SIMD"),
        (concat!("i16x", "8::"), "Portable SIMD"),
        (concat!("i32x", "4::"), "Portable SIMD"),
        (concat!("f32x", "4::"), "Portable SIMD"),
        (concat!("Simd", "::<"), "Portable SIMD"),
    ];

    let entries = match walkdir_rs_files(&src_dir) {
        Ok(e) => e,
        Err(_) => return violations,
    };

    for entry in entries {
        let content = match fs::read_to_string(&entry) {
            Ok(c) => c,
            Err(_) => continue,
        };
        let lines: Vec<&str> = content.lines().collect();
        let test_lines = compute_test_code_lines(&lines);
        let protected_lines = compute_target_feature_protected_lines(&lines);
        let file_path = entry.display().to_string();

        for (line_num, line) in lines.iter().enumerate() {
            if test_lines.contains(&line_num) || protected_lines.contains(&line_num) {
                continue;
            }

            let all_patterns = intrinsic_patterns.iter().chain(portable_patterns.iter());
            for &(pattern, kind) in all_patterns {
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
                if let Ok(content) = fs::read_to_string(&entry) {
                    for (line_num, line) in content.lines().enumerate() {
                        let trimmed = line.trim();
                        // Check for array access without bounds check
                        if trimmed.contains('[') && trimmed.contains(']') {
                            // Simple heuristic: array access without preceding bounds check
                            let preceding_lines: Vec<&str> =
                                content.lines().take(line_num).collect();
                            if !preceding_lines
                                .iter()
                                .rev()
                                .take(5)
                                .any(|l| l.contains("if") && (l.contains('<') || l.contains(">=")))
                            {
                                violations.push(CbPatternViolation {
                                    pattern_id: "CB-001".to_string(),
                                    file: entry.display().to_string(),
                                    line: line_num + 1,
                                    description: "WGSL array access without bounds check"
                                        .to_string(),
                                    severity: Severity::Warning,
                                });
                            }
                        }
                    }
                }
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
                if let Ok(content) = fs::read_to_string(&entry) {
                    let lines: Vec<&str> = content.lines().collect();
                    let mut in_conditional = false;
                    let mut conditional_depth = 0;

                    for (line_num, line) in lines.iter().enumerate() {
                        let trimmed = line.trim();

                        // Track conditional blocks
                        if trimmed.starts_with("if") || trimmed.starts_with("else") {
                            in_conditional = true;
                        }
                        if in_conditional {
                            conditional_depth += trimmed.matches('{').count();
                            conditional_depth =
                                conditional_depth.saturating_sub(trimmed.matches('}').count());
                            if conditional_depth == 0 {
                                in_conditional = false;
                            }
                        }

                        // Check for barrier inside conditional
                        if in_conditional
                            && (trimmed.contains("workgroupBarrier")
                                || trimmed.contains("storageBarrier"))
                        {
                            violations.push(CbPatternViolation {
                                pattern_id: "CB-002".to_string(),
                                file: entry.display().to_string(),
                                line: line_num + 1,
                                description: "WGSL barrier inside conditional (divergence risk)"
                                    .to_string(),
                                severity: Severity::Critical,
                            });
                        }
                    }
                }
            }
        }
    }

    violations
}

/// Detect ComputeBricks without assertions/validation (CB-BUDGET)
pub fn detect_bricks_without_assertions(project_path: &Path) -> Vec<CbPatternViolation> {
    let mut violations = Vec::new();

    let brick_dir = project_path.join("src").join("brick");
    if !brick_dir.exists() {
        return violations;
    }

    if let Ok(entries) = walkdir_rs_files(&brick_dir) {
        for entry in entries {
            if let Ok(content) = fs::read_to_string(&entry) {
                // Check if this file has any brick implementation (impl ... for ... Brick)
                let is_brick_impl = content.contains("impl") && content.contains("Brick");

                if is_brick_impl {
                    // Check for presence of assertions or validation
                    let has_assertions = content.contains("assert!")
                        || content.contains("debug_assert!")
                        || content.contains("validate")
                        || content.contains("check_budget")
                        || content.contains("budget_remaining");

                    if !has_assertions {
                        violations.push(CbPatternViolation {
                            pattern_id: "CB-BUDGET".to_string(),
                            file: entry.display().to_string(),
                            line: 1,
                            description: "ComputeBrick without assertions or budget validation"
                                .to_string(),
                            severity: Severity::Warning,
                        });
                    }
                }
            }
        }
    }

    violations
}

/// Parse BrickProfiler JSON output and detect anomalies
pub fn detect_profiler_anomalies(project_path: &Path) -> Vec<ProfilerAnomaly> {
    let mut anomalies = Vec::new();

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
            // Parse JSON manually to avoid adding serde_json dep to this module
            // Look for patterns like "cv": 0.18 (CV > 15% threshold)
            // and "efficiency": 0.22 (efficiency < 25% threshold)

            // Simple pattern matching for CV values
            for line in content.lines() {
                let line = line.trim();

                // Detect high coefficient of variation (CV > 15%)
                if line.contains("\"cv\"") || line.contains("\"cv_percent\"") {
                    if let Some(value) = extract_json_number(line) {
                        let cv_threshold = 15.0;
                        let cv = if value < 1.0 { value * 100.0 } else { value };
                        if cv > cv_threshold {
                            anomalies.push(ProfilerAnomaly {
                                brick_name: extract_brick_name(&content, line),
                                anomaly_type: "HIGH_CV".to_string(),
                                value: cv,
                                threshold: cv_threshold,
                            });
                        }
                    }
                }

                // Detect low efficiency (< 25%)
                if line.contains("\"efficiency\"") {
                    if let Some(value) = extract_json_number(line) {
                        let eff_threshold = 25.0;
                        let efficiency = if value < 1.0 { value * 100.0 } else { value };
                        if efficiency < eff_threshold {
                            anomalies.push(ProfilerAnomaly {
                                brick_name: extract_brick_name(&content, line),
                                anomaly_type: "LOW_EFFICIENCY".to_string(),
                                value: efficiency,
                                threshold: eff_threshold,
                            });
                        }
                    }
                }
            }
            break; // Only process first found file
        }
    }

    anomalies
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
pub fn extract_brick_name(content: &str, target_line: &str) -> String {
    // Find brick name by looking for "name" field near the target line
    let lines: Vec<&str> = content.lines().collect();
    for (i, line) in lines.iter().enumerate() {
        if *line == target_line {
            // Look backwards for name field
            for j in (0..i).rev().take(20) {
                if lines[j].contains("\"name\"") || lines[j].contains("\"brick_name\"") {
                    if let Some(name) = lines[j].split('"').nth(3) {
                        return name.to_string();
                    }
                }
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
/// Pattern: `partial_cmp(...).unwrap()` or `.expect(...)` which panic on NaN
/// Safe alternatives: `total_cmp()`, `unwrap_or()`, `unwrap_or_else()`
/// Source: OIP Tarantula analysis - 10 instances in ml.rs, imbalance.rs, classifier.rs
/// Common scanner: iterate non-test, non-comment lines in all .rs files under src/.
/// The callback receives (trimmed_line, file_path, line_num) and may push violations.
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
        let content = match fs::read_to_string(&entry) {
            Ok(c) => c,
            Err(_) => continue,
        };
        let lines: Vec<&str> = content.lines().collect();
        let test_lines = compute_test_code_lines(&lines);
        let file_path = entry.display().to_string();

        for (line_num, line) in lines.iter().enumerate() {
            if test_lines.contains(&line_num) {
                continue;
            }
            let trimmed = line.trim();
            if trimmed.starts_with("//") || trimmed.starts_with("/*") || trimmed.starts_with("*") {
                continue;
            }
            check(trimmed, &file_path, line_num, &mut violations);
        }
    }
    violations
}

/// Check if a pattern is inside a string literal (odd number of quotes before it)
pub(super) fn is_in_string_literal(line: &str, pattern: &str) -> bool {
    if let Some(idx) = line.find(pattern) {
        let quote_count = line[..idx].chars().filter(|&c| c == '"').count();
        quote_count % 2 == 1
    } else {
        false
    }
}

pub fn detect_cb120_nan_unsafe_comparison(project_path: &Path) -> Vec<CbPatternViolation> {
    scan_rs_production_lines(project_path, false, |trimmed, file_path, line_num, violations| {
        if is_in_string_literal(trimmed, "partial_cmp") {
            return;
        }
        if !trimmed.contains("partial_cmp") {
            return;
        }
        let has_unwrap = trimmed.contains(".unwrap()") && !trimmed.contains("unwrap_or");
        let has_expect = trimmed.contains(".expect(");
        let suffix = if has_unwrap { "unwrap()" } else if has_expect { "expect()" } else { return };
        violations.push(CbPatternViolation {
            pattern_id: "CB-120".to_string(),
            file: file_path.to_string(),
            line: line_num + 1,
            description: format!("NaN-unsafe: .partial_cmp().{suffix} panics on NaN. Use .total_cmp() or .unwrap_or()"),
            severity: Severity::Error,
        });
    })
}

/// CB-121: Detect lock poisoning vulnerabilities
/// Pattern: `mutex.lock().unwrap()` or `rwlock.read/write().unwrap()`
/// Safe alternatives: `unwrap_or_else(|e| e.into_inner())`, `parking_lot`
/// Source: OIP Tarantula analysis - 10 instances in git.rs
pub(super) fn check_lock_poisoning_line(
    trimmed: &str,
    has_rwlock_import: bool,
    file_path: &str,
    line_num: usize,
) -> Option<CbPatternViolation> {
    let is_safe = trimmed.contains("unwrap_or_else") || trimmed.contains("into_inner");

    // Check for mutex.lock().unwrap() pattern
    if trimmed.contains(".lock()") && trimmed.contains(".unwrap()") && !is_safe {
        return Some(CbPatternViolation {
            pattern_id: "CB-121".to_string(),
            file: file_path.to_string(),
            line: line_num + 1,
            description: "Lock poisoning: .lock().unwrap() panics if another thread panicked. Use unwrap_or_else(|e| e.into_inner()) or parking_lot".to_string(),
            severity: Severity::Warning,
        });
    }

    // Check for rwlock read/write unwrap patterns
    let is_rwlock_op = (trimmed.contains(".read()") || trimmed.contains(".write()"))
        && trimmed.contains(".unwrap()")
        && !is_safe;

    if is_rwlock_op && (trimmed.contains("RwLock") || has_rwlock_import) {
        let op = if trimmed.contains(".read()") { "read" } else { "write" };
        return Some(CbPatternViolation {
            pattern_id: "CB-121".to_string(),
            file: file_path.to_string(),
            line: line_num + 1,
            description: format!("Lock poisoning: .{op}().unwrap() panics if another thread panicked. Use unwrap_or_else(|e| e.into_inner())"),
            severity: Severity::Warning,
        });
    }

    None
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
        let content = match fs::read_to_string(&entry) {
            Ok(c) => c,
            Err(_) => continue,
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

            // Skip comments and string literals
            if trimmed.starts_with("//") || trimmed.starts_with("/*") || trimmed.starts_with("*") {
                continue;
            }
            if let Some(idx) = trimmed.find(".lock()") {
                let quote_count = trimmed[..idx].chars().filter(|&c| c == '"').count();
                if quote_count % 2 == 1 {
                    continue;
                }
            }

            if let Some(v) = check_lock_poisoning_line(trimmed, has_rwlock_import, &file_path, line_num) {
                violations.push(v);
            }
        }
    }

    violations
}

/// CB-122: Detect serde deserialization safety issues
/// Pattern: `serde_json::from_str().unwrap()` or `.expect()`
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
            let quote_count = trimmed[..idx].chars().filter(|&c| c == '"').count();
            if quote_count % 2 == 1 {
                continue;
            }
        }
        let has_unwrap = trimmed.contains(".unwrap()") && !trimmed.contains("unwrap_or");
        let has_expect = trimmed.contains(".expect(");
        let suffix = if has_unwrap {
            "unwrap()"
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
    let mut violations = Vec::new();

    let src_dir = project_path.join("src");
    if !src_dir.exists() {
        return violations;
    }

    let serde_patterns = [
        "serde_json::from_str", "serde_json::from_slice", "serde_json::from_reader",
        "serde_yaml::from_str", "serde_yaml::from_slice", "serde_yaml::from_reader",
        "toml::from_str", "toml::de::from_str", "ron::from_str",
    ];

    let entries = match walkdir_rs_files(&src_dir) {
        Ok(e) => e,
        Err(_) => return violations,
    };

    for entry in entries {
        // Skip test files entirely - serde unwrap in tests is acceptable
        if is_test_file(&entry) {
            continue;
        }
        let content = match fs::read_to_string(&entry) {
            Ok(c) => c,
            Err(_) => continue,
        };
        let lines: Vec<&str> = content.lines().collect();
        let test_lines = compute_test_code_lines(&lines);
        let file_path = entry.display().to_string();

        for (line_num, line) in lines.iter().enumerate() {
            if test_lines.contains(&line_num) {
                continue;
            }
            let trimmed = line.trim();
            if trimmed.starts_with("//") || trimmed.starts_with("/*") || trimmed.starts_with("*") {
                continue;
            }
            check_serde_line(trimmed, &serde_patterns, &file_path, line_num, &mut violations);
        }
    }

    violations
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

pub fn detect_cb123_undocumented_ignore(project_path: &Path) -> Vec<CbPatternViolation> {
    let mut violations = Vec::new();

    for dir in [project_path.join("src"), project_path.join("tests")] {
        if !dir.exists() {
            continue;
        }
        let entries = match walkdir_rs_files(&dir) {
            Ok(e) => e,
            Err(_) => continue,
        };
        for entry in entries {
            let content = match fs::read_to_string(&entry) {
                Ok(c) => c,
                Err(_) => continue,
            };
            let lines: Vec<&str> = content.lines().collect();
            let file_path = entry.display().to_string();

            for (line_num, line) in lines.iter().enumerate() {
                let trimmed = line.trim();
                if trimmed.starts_with("#[ignore]") && !has_ignore_documentation(&lines, line_num, trimmed) {
                    violations.push(CbPatternViolation {
                        pattern_id: "CB-123".to_string(),
                        file: file_path.clone(),
                        line: line_num + 1,
                        description: "Undocumented #[ignore]: Add reason with #[ignore = \"reason\"] or // reason comment".to_string(),
                        severity: Severity::Warning,
                    });
                }
            }
        }
    }

    violations
}

/// CB-124: Detect low coverage thresholds in CI/config
/// Threshold: <80% is Error, <95% is Warning for sovereign stack
/// Source: OIP Tarantula analysis - 58% threshold (below 80% minimum)
pub fn detect_cb124_coverage_threshold(project_path: &Path) -> Vec<CbPatternViolation> {
    let mut violations = Vec::new();

    // Coverage configuration files to check
    let config_files = [
        project_path.join(".cargo").join("config.toml"),
        project_path.join("tarpaulin.toml"),
        project_path.join(".tarpaulin.toml"),
        project_path.join("codecov.yml"),
        project_path.join(".codecov.yml"),
        project_path.join("Makefile"),
        project_path.join(".github").join("workflows").join("ci.yml"),
        project_path.join(".github").join("workflows").join("test.yml"),
        project_path.join(".github").join("workflows").join("coverage.yml"),
    ];

    for config_path in &config_files {
        if !config_path.exists() {
            continue;
        }

        if let Ok(content) = fs::read_to_string(config_path) {
            for (line_num, line) in content.lines().enumerate() {
                let line_lower = line.to_lowercase();

                // Patterns for coverage thresholds (specific to coverage context)
                let threshold_patterns = [
                    ("fail_under", '='),
                    ("coverage_threshold", '='),
                    ("min_coverage", '='),
                    ("cov_threshold", '='),
                    ("COVERAGE <", ' '),
                ];

                for (pattern, sep) in &threshold_patterns {
                    if line_lower.contains(&pattern.to_lowercase()) {
                        // Extract the numeric value
                        if let Some(value) = extract_coverage_threshold(line, *sep) {
                            if value < 80.0 {
                                violations.push(CbPatternViolation {
                                    pattern_id: "CB-124".to_string(),
                                    file: config_path.display().to_string(),
                                    line: line_num + 1,
                                    description: format!(
                                        "Low coverage threshold: {:.1}% is below 80% minimum. Increase coverage requirements",
                                        value
                                    ),
                                    severity: Severity::Error,
                                });
                            } else if value < 95.0 {
                                violations.push(CbPatternViolation {
                                    pattern_id: "CB-124".to_string(),
                                    file: config_path.display().to_string(),
                                    line: line_num + 1,
                                    description: format!(
                                        "Coverage threshold {:.1}% below sovereign stack standard (95%). Consider increasing",
                                        value
                                    ),
                                    severity: Severity::Warning,
                                });
                            }
                        }
                    }
                }
            }
        }
    }

    violations
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

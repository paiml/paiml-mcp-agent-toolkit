//! ComputeBrick Pattern Detection for PMAT Compliance
//!
//! Extracted from comply_handlers.rs for file health compliance (CB-040).
//! Contains CB pattern detection functions and check_compute_brick.

use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;

/// A compliance check result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComplianceCheck {
    pub name: String,
    pub status: CheckStatus,
    pub message: String,
    pub severity: Severity,
}

/// Status of a compliance check
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub enum CheckStatus {
    Pass,
    Warn,
    Fail,
    Skip,
}

/// Severity level for compliance issues
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub enum Severity {
    Info,
    Warning,
    Error,
    Critical,
}

/// ComputeBrick pattern detection result
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[allow(dead_code)]
pub struct CbPatternViolation {
    pub pattern_id: String,
    pub file: String,
    pub line: usize,
    pub description: String,
    pub severity: Severity,
}

/// BrickProfiler anomaly from JSON output
#[derive(Debug, Clone)]
pub struct ProfilerAnomaly {
    pub brick_name: String,
    pub anomaly_type: String,
    pub value: f64,
    pub threshold: f64,
}

/// Compute line ranges that are inside test code (#[cfg(test)] mod tests { ... })
/// Returns a HashSet of line indices that should be skipped for production code analysis.
pub fn compute_test_code_lines(lines: &[&str]) -> std::collections::HashSet<usize> {
    let mut test_lines = std::collections::HashSet::new();
    let mut i = 0;

    while i < lines.len() {
        let line = lines[i].trim();

        // Detect #[cfg(test)] followed by mod (within next 3 lines)
        if line.starts_with("#[cfg(test)]") {
            // Find the mod line
            for j in i..std::cmp::min(i + 4, lines.len()) {
                if lines[j].contains("mod ") {
                    // Found test module - track all lines until closing brace
                    let mut depth = 0;
                    for k in j..lines.len() {
                        depth += lines[k].matches('{').count();
                        depth = depth.saturating_sub(lines[k].matches('}').count());
                        test_lines.insert(k);
                        if depth == 0 && k > j && lines[k].contains('}') {
                            break;
                        }
                    }
                    // Also mark the #[cfg(test)] line
                    test_lines.insert(i);
                    break;
                }
            }
        }

        // Also detect standalone `mod tests {` without #[cfg(test)] (common pattern)
        if (line.starts_with("mod tests") || line.starts_with("pub mod tests"))
            && line.contains('{')
        {
            let mut depth = 0;
            for k in i..lines.len() {
                depth += lines[k].matches('{').count();
                depth = depth.saturating_sub(lines[k].matches('}').count());
                test_lines.insert(k);
                if depth == 0 && k > i && lines[k].contains('}') {
                    break;
                }
            }
        }

        // Detect #[test] function (individual test functions)
        if line.starts_with("#[test]") {
            test_lines.insert(i);
            // Mark the function that follows
            for j in i + 1..std::cmp::min(i + 4, lines.len()) {
                if lines[j].contains("fn ") {
                    let mut depth = 0;
                    for k in j..lines.len() {
                        depth += lines[k].matches('{').count();
                        depth = depth.saturating_sub(lines[k].matches('}').count());
                        test_lines.insert(k);
                        if depth == 0 && k > j {
                            break;
                        }
                    }
                    break;
                }
            }
        }

        i += 1;
    }

    test_lines
}

/// Scan Rust files for CB-020 (unsafe without SAFETY comment)
/// NOTE: Skips test code (#[cfg(test)], mod tests, #[test]) - test code can use .unwrap() freely
pub fn detect_cb020_unsafe_without_safety(project_path: &Path) -> Vec<CbPatternViolation> {
    let mut violations = Vec::new();

    // Walk src/ directory for .rs files
    let src_dir = project_path.join("src");
    if !src_dir.exists() {
        return violations;
    }

    if let Ok(entries) = walkdir_rs_files(&src_dir) {
        for entry in entries {
            if let Ok(content) = fs::read_to_string(&entry) {
                let lines: Vec<&str> = content.lines().collect();
                let test_lines = compute_test_code_lines(&lines);

                for (line_num, line) in lines.iter().enumerate() {
                    // Skip test code - unsafe in tests is fine
                    if test_lines.contains(&line_num) {
                        continue;
                    }

                    let trimmed = line.trim();
                    // Check for unsafe block without preceding SAFETY comment
                    if trimmed.starts_with("unsafe {") || trimmed.starts_with("unsafe{") {
                        // Look at previous non-empty lines for SAFETY comment
                        // Check up to 10 lines back to handle multi-line safety comments
                        let has_safety = lines.iter().take(line_num).rev().take(10).any(|l| {
                            l.contains("// SAFETY:")
                                || l.contains("// SAFETY :")
                                || l.contains("/ SAFETY:")
                        });

                        if !has_safety {
                            violations.push(CbPatternViolation {
                                pattern_id: "CB-020".to_string(),
                                file: entry.display().to_string(),
                                line: line_num + 1,
                                description: "unsafe block without SAFETY comment".to_string(),
                                severity: Severity::Warning,
                            });
                        }
                    }
                }
            }
        }
    }

    violations
}

/// Helper to walk directory for .rs files
pub fn walkdir_rs_files(dir: &Path) -> Result<Vec<std::path::PathBuf>, std::io::Error> {
    let mut files = Vec::new();
    for entry in std::fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();
        if path.is_dir() {
            files.extend(walkdir_rs_files(&path)?);
        } else if path.extension().map(|e| e == "rs").unwrap_or(false) {
            files.push(path);
        }
    }
    Ok(files)
}

/// Scan for CB-021 (SIMD intrinsics without #[target_feature])
/// NOTE: Skips test code (#[cfg(test)], mod tests, #[test]) - test code is exempt
pub fn detect_cb021_simd_without_target_feature(project_path: &Path) -> Vec<CbPatternViolation> {
    let mut violations = Vec::new();

    let src_dir = project_path.join("src");
    if !src_dir.exists() {
        return violations;
    }

    // Common SIMD intrinsic patterns - must be actual intrinsic calls
    // Use concat! to avoid self-matching when this file is scanned
    let simd_patterns_needing_target_feature = [
        concat!("_mm", "256_"),
        concat!("_mm", "512_"), // x86 AVX/AVX-512 (not SSE which is baseline)
    ];
    // Portable SIMD - require :: suffix to distinguish from identifiers
    // Use concat! to avoid self-matching when this file is scanned
    let portable_simd_patterns = [
        concat!("i8x", "16::"),
        concat!("i16x", "8::"),
        concat!("i32x", "4::"),
        concat!("f32x", "4::"),
        concat!("Simd", "::<"),
    ];

    if let Ok(entries) = walkdir_rs_files(&src_dir) {
        for entry in entries {
            if let Ok(content) = fs::read_to_string(&entry) {
                let lines: Vec<&str> = content.lines().collect();
                let test_lines = compute_test_code_lines(&lines);

                // Find functions with #[target_feature] attribute
                let mut protected_lines: std::collections::HashSet<usize> =
                    std::collections::HashSet::new();

                for (i, line) in lines.iter().enumerate() {
                    // Both #[target_feature] and #[cfg(target_feature = "...")] protect SIMD code
                    let is_protected = line.trim().starts_with("#[target_feature")
                        || (line.contains("#[cfg(") && line.contains("target_feature"));

                    if is_protected {
                        // Find the function this attribute applies to
                        let mut depth = 0;
                        for j in i..lines.len() {
                            if lines[j].contains("fn ") && depth == 0 {
                                // Mark all lines in this function as protected
                                for k in j..lines.len() {
                                    depth += lines[k].matches('{').count();
                                    depth = depth.saturating_sub(lines[k].matches('}').count());
                                    protected_lines.insert(k);
                                    if depth == 0 && k > j {
                                        break;
                                    }
                                }
                                break;
                            }
                        }
                    }
                }

                for (line_num, line) in lines.iter().enumerate() {
                    // Skip test code
                    if test_lines.contains(&line_num) {
                        continue;
                    }
                    // Skip protected functions
                    if protected_lines.contains(&line_num) {
                        continue;
                    }

                    // Check for SIMD intrinsics that need target_feature
                    for pattern in &simd_patterns_needing_target_feature {
                        if line.contains(pattern) {
                            violations.push(CbPatternViolation {
                                pattern_id: "CB-021".to_string(),
                                file: entry.display().to_string(),
                                line: line_num + 1,
                                description: format!(
                                    "SIMD intrinsic {} without #[target_feature]",
                                    pattern
                                ),
                                severity: Severity::Warning,
                            });
                        }
                    }

                    // Check portable SIMD patterns
                    for pattern in &portable_simd_patterns {
                        if line.contains(pattern) {
                            violations.push(CbPatternViolation {
                                pattern_id: "CB-021".to_string(),
                                file: entry.display().to_string(),
                                line: line_num + 1,
                                description: format!(
                                    "Portable SIMD {} without #[target_feature]",
                                    pattern
                                ),
                                severity: Severity::Warning,
                            });
                        }
                    }
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

fn walkdir_wgsl_files(dir: &Path) -> Result<Vec<std::path::PathBuf>, std::io::Error> {
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
pub fn detect_cb120_nan_unsafe_comparison(project_path: &Path) -> Vec<CbPatternViolation> {
    let mut violations = Vec::new();

    let src_dir = project_path.join("src");
    if !src_dir.exists() {
        return violations;
    }

    if let Ok(entries) = walkdir_rs_files(&src_dir) {
        for entry in entries {
            if let Ok(content) = fs::read_to_string(&entry) {
                let lines: Vec<&str> = content.lines().collect();
                let test_lines = compute_test_code_lines(&lines);

                for (line_num, line) in lines.iter().enumerate() {
                    // Skip test code - NaN panics in tests are acceptable
                    if test_lines.contains(&line_num) {
                        continue;
                    }

                    let trimmed = line.trim();

                    // Skip comment lines to avoid false positives from documentation
                    if trimmed.starts_with("//") || trimmed.starts_with("/*") || trimmed.starts_with("*") {
                        continue;
                    }

                    // Skip string literals containing the pattern (error messages, docs)
                    // Heuristic: if odd number of quotes before "partial_cmp", it's inside a string
                    if let Some(idx) = trimmed.find("partial_cmp") {
                        let before = &trimmed[..idx];
                        let quote_count = before.chars().filter(|&c| c == '"').count();
                        if quote_count % 2 == 1 {
                            continue;
                        }
                    }

                    // Check for partial_cmp().unwrap() pattern
                    if trimmed.contains("partial_cmp") && trimmed.contains(".unwrap()") {
                        // Make sure it's not a safe variant
                        if !trimmed.contains("unwrap_or") && !trimmed.contains("unwrap_or_else") {
                            violations.push(CbPatternViolation {
                                pattern_id: "CB-120".to_string(),
                                file: entry.display().to_string(),
                                line: line_num + 1,
                                description: "NaN-unsafe: .partial_cmp().unwrap() panics on NaN. Use .total_cmp() or .unwrap_or()".to_string(),
                                severity: Severity::Error,
                            });
                        }
                    }

                    // Check for partial_cmp().expect() pattern
                    if trimmed.contains("partial_cmp") && trimmed.contains(".expect(") {
                        violations.push(CbPatternViolation {
                            pattern_id: "CB-120".to_string(),
                            file: entry.display().to_string(),
                            line: line_num + 1,
                            description: "NaN-unsafe: .partial_cmp().expect() panics on NaN. Use .total_cmp() or .unwrap_or()".to_string(),
                            severity: Severity::Error,
                        });
                    }
                }
            }
        }
    }

    violations
}

/// CB-121: Detect lock poisoning vulnerabilities
/// Pattern: `mutex.lock().unwrap()` or `rwlock.read/write().unwrap()`
/// Safe alternatives: `unwrap_or_else(|e| e.into_inner())`, `parking_lot`
/// Source: OIP Tarantula analysis - 10 instances in git.rs
pub fn detect_cb121_lock_poisoning(project_path: &Path) -> Vec<CbPatternViolation> {
    let mut violations = Vec::new();

    let src_dir = project_path.join("src");
    if !src_dir.exists() {
        return violations;
    }

    if let Ok(entries) = walkdir_rs_files(&src_dir) {
        for entry in entries {
            if let Ok(content) = fs::read_to_string(&entry) {
                let lines: Vec<&str> = content.lines().collect();
                let test_lines = compute_test_code_lines(&lines);

                for (line_num, line) in lines.iter().enumerate() {
                    // Skip test code
                    if test_lines.contains(&line_num) {
                        continue;
                    }

                    let trimmed = line.trim();

                    // Skip comment lines to avoid false positives from documentation
                    if trimmed.starts_with("//") || trimmed.starts_with("/*") || trimmed.starts_with("*") {
                        continue;
                    }

                    // Skip string literals containing the pattern (error messages, docs)
                    if let Some(idx) = trimmed.find(".lock()") {
                        let before = &trimmed[..idx];
                        let quote_count = before.chars().filter(|&c| c == '"').count();
                        if quote_count % 2 == 1 {
                            continue;
                        }
                    }

                    // Check for mutex.lock().unwrap() pattern
                    if trimmed.contains(".lock()") && trimmed.contains(".unwrap()") {
                        // Skip safe patterns
                        if trimmed.contains("unwrap_or_else") || trimmed.contains("into_inner") {
                            continue;
                        }
                        violations.push(CbPatternViolation {
                            pattern_id: "CB-121".to_string(),
                            file: entry.display().to_string(),
                            line: line_num + 1,
                            description: "Lock poisoning: .lock().unwrap() panics if another thread panicked. Use unwrap_or_else(|e| e.into_inner()) or parking_lot".to_string(),
                            severity: Severity::Warning,
                        });
                    }

                    // Check for rwlock.read().unwrap() pattern
                    if trimmed.contains(".read()") && trimmed.contains(".unwrap()") {
                        // Avoid false positives on file reads - check for lock context
                        if (trimmed.contains("RwLock") || content.contains("std::sync::RwLock") || content.contains("use std::sync::RwLock"))
                            && !trimmed.contains("unwrap_or_else") && !trimmed.contains("into_inner") {
                                violations.push(CbPatternViolation {
                                    pattern_id: "CB-121".to_string(),
                                    file: entry.display().to_string(),
                                    line: line_num + 1,
                                    description: "Lock poisoning: .read().unwrap() panics if another thread panicked. Use unwrap_or_else(|e| e.into_inner())".to_string(),
                                    severity: Severity::Warning,
                                });
                            }
                    }

                    // Check for rwlock.write().unwrap() pattern
                    if trimmed.contains(".write()") && trimmed.contains(".unwrap()") {
                        // Avoid false positives on file writes - check for lock context
                        if (trimmed.contains("RwLock") || content.contains("std::sync::RwLock") || content.contains("use std::sync::RwLock"))
                            && !trimmed.contains("unwrap_or_else") && !trimmed.contains("into_inner") {
                                violations.push(CbPatternViolation {
                                    pattern_id: "CB-121".to_string(),
                                    file: entry.display().to_string(),
                                    line: line_num + 1,
                                    description: "Lock poisoning: .write().unwrap() panics if another thread panicked. Use unwrap_or_else(|e| e.into_inner())".to_string(),
                                    severity: Severity::Warning,
                                });
                            }
                    }
                }
            }
        }
    }

    violations
}

/// CB-122: Detect serde deserialization safety issues
/// Pattern: `serde_json::from_str().unwrap()` or `.expect()`
/// Safe alternatives: `?` operator, `match`, `unwrap_or_default()`
/// Source: OIP Tarantula analysis - 15+ instances in tarantula.rs, github.rs, citl.rs
pub fn detect_cb122_serde_safety(project_path: &Path) -> Vec<CbPatternViolation> {
    let mut violations = Vec::new();

    let src_dir = project_path.join("src");
    if !src_dir.exists() {
        return violations;
    }

    // Serde-related parsing functions to check
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

    if let Ok(entries) = walkdir_rs_files(&src_dir) {
        for entry in entries {
            if let Ok(content) = fs::read_to_string(&entry) {
                let lines: Vec<&str> = content.lines().collect();
                let test_lines = compute_test_code_lines(&lines);

                for (line_num, line) in lines.iter().enumerate() {
                    // Skip test code - panicking on bad input in tests is fine
                    if test_lines.contains(&line_num) {
                        continue;
                    }

                    let trimmed = line.trim();

                    // Skip comment lines to avoid false positives from documentation
                    if trimmed.starts_with("//") || trimmed.starts_with("/*") || trimmed.starts_with("*") {
                        continue;
                    }

                    for pattern in &serde_patterns {
                        // Skip if pattern is inside a string literal (error messages)
                        if let Some(idx) = trimmed.find(pattern) {
                            let before = &trimmed[..idx];
                            let quote_count = before.chars().filter(|&c| c == '"').count();
                            if quote_count % 2 == 1 {
                                continue;
                            }
                        }

                        if trimmed.contains(pattern) {
                            // Check for unsafe unwrap patterns
                            if trimmed.contains(".unwrap()") && !trimmed.contains("unwrap_or") {
                                violations.push(CbPatternViolation {
                                    pattern_id: "CB-122".to_string(),
                                    file: entry.display().to_string(),
                                    line: line_num + 1,
                                    description: format!(
                                        "Serde unsafe: {}().unwrap() panics on malformed input. Use ? operator or proper error handling",
                                        pattern
                                    ),
                                    severity: Severity::Error,
                                });
                            }

                            // Check for expect patterns
                            if trimmed.contains(".expect(") {
                                violations.push(CbPatternViolation {
                                    pattern_id: "CB-122".to_string(),
                                    file: entry.display().to_string(),
                                    line: line_num + 1,
                                    description: format!(
                                        "Serde unsafe: {}().expect() panics on malformed input. Use ? operator or proper error handling",
                                        pattern
                                    ),
                                    severity: Severity::Error,
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

/// CB-123: Detect undocumented #[ignore] tests
/// Pattern: `#[ignore]` without a reason comment or attribute value
/// Valid: `#[ignore = "reason"]`, `#[ignore] // reason`, `/// reason \n #[ignore]`
/// Source: OIP Tarantula analysis - 6 undocumented #[ignore] tests
pub fn detect_cb123_undocumented_ignore(project_path: &Path) -> Vec<CbPatternViolation> {
    let mut violations = Vec::new();

    let src_dir = project_path.join("src");
    let tests_dir = project_path.join("tests");

    for dir in [src_dir, tests_dir] {
        if !dir.exists() {
            continue;
        }

        if let Ok(entries) = walkdir_rs_files(&dir) {
            for entry in entries {
                if let Ok(content) = fs::read_to_string(&entry) {
                    let lines: Vec<&str> = content.lines().collect();

                    for (line_num, line) in lines.iter().enumerate() {
                        let trimmed = line.trim();

                        // Check for #[ignore] attribute
                        if trimmed.starts_with("#[ignore]") {
                            // Check if it has inline reason: #[ignore = "reason"]
                            let has_inline_reason = trimmed.contains('=') && trimmed.contains('"');

                            // Check if same line has comment: #[ignore] // reason
                            let has_line_comment = trimmed.contains("//");

                            // Check preceding line for doc comment: /// reason
                            let has_doc_comment = line_num > 0
                                && lines[line_num - 1].trim().starts_with("///");

                            // Check preceding line for regular comment
                            let has_preceding_comment = line_num > 0
                                && lines[line_num - 1].trim().starts_with("//");

                            if !has_inline_reason && !has_line_comment && !has_doc_comment && !has_preceding_comment {
                                violations.push(CbPatternViolation {
                                    pattern_id: "CB-123".to_string(),
                                    file: entry.display().to_string(),
                                    line: line_num + 1,
                                    description: "Undocumented #[ignore]: Add reason with #[ignore = \"reason\"] or // reason comment".to_string(),
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

                // Patterns for coverage thresholds
                let threshold_patterns = [
                    ("fail_under", '='),
                    ("coverage_threshold", '='),
                    ("min_coverage", '='),
                    ("threshold", ':'),
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
fn extract_coverage_threshold(line: &str, separator: char) -> Option<f64> {
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

// =============================================================================
// CB-125, CB-126, CB-127: Coverage Quality & Test Performance (v2.2)
// Per improve-pmat-comply.md v2.2.0 specification
// =============================================================================

/// CB-125: Detect coverage exclusion gaming
/// Per [GAME-001] Popper: Unfalsifiable claims are unscientific
/// Per [GAME-002] Google TAP: >20% exclusion indicates gaming
/// Thresholds:
/// - >10 exclusion patterns = Warning (complexity suggests gaming)
/// - >20% LOC excluded = Error (significant coverage blind spot)
/// - >50% LOC excluded = Critical (coverage metric meaningless)
pub fn detect_cb125_coverage_exclusion_gaming(project_path: &Path) -> Vec<CbPatternViolation> {
    let mut violations = Vec::new();

    // Check Makefile for --ignore-filename-regex patterns
    let makefile_path = project_path.join("Makefile");
    if makefile_path.exists() {
        if let Ok(content) = fs::read_to_string(&makefile_path) {
            let mut exclusion_count = 0;
            let mut exclusion_line = 0;

            for (line_num, line) in content.lines().enumerate() {
                // Count exclusion patterns
                if line.contains("--ignore-filename-regex")
                    || line.contains("COVERAGE_EXCLUDE")
                    || line.contains("--exclude")
                {
                    exclusion_line = line_num + 1;

                    // Count pipe-separated patterns in regex
                    if let Some(start) = line.find("'") {
                        if let Some(end) = line.rfind("'") {
                            if start < end {
                                let pattern = &line[start + 1..end];
                                exclusion_count += pattern.matches('|').count() + 1;
                            }
                        }
                    }
                }
            }

            // Severity based on pattern count (per [GAME-002] Google TAP)
            if exclusion_count > 50 {
                violations.push(CbPatternViolation {
                    pattern_id: "CB-125-C".to_string(),
                    file: makefile_path.display().to_string(),
                    line: exclusion_line,
                    description: format!(
                        "CRITICAL: {} coverage exclusion patterns detected. Coverage metric is meaningless. \
                        Per [GAME-001] Popper: unfalsifiable coverage claims are unscientific. \
                        Reduce to ≤10 patterns (binary entry points only)",
                        exclusion_count
                    ),
                    severity: Severity::Critical,
                });
            } else if exclusion_count > 20 {
                violations.push(CbPatternViolation {
                    pattern_id: "CB-125-B".to_string(),
                    file: makefile_path.display().to_string(),
                    line: exclusion_line,
                    description: format!(
                        "{} coverage exclusion patterns exceed 20% budget per [GAME-002] Google TAP. \
                        Significant coverage blind spot. Reduce exclusions or document technical debt",
                        exclusion_count
                    ),
                    severity: Severity::Error,
                });
            } else if exclusion_count > 10 {
                violations.push(CbPatternViolation {
                    pattern_id: "CB-125-A".to_string(),
                    file: makefile_path.display().to_string(),
                    line: exclusion_line,
                    description: format!(
                        "{} coverage exclusion patterns suggests complexity. \
                        Consider reducing to ≤10 patterns (binary entry points only)",
                        exclusion_count
                    ),
                    severity: Severity::Warning,
                });
            }
        }
    }

    violations
}

/// Check sleep duration and return violation if threshold exceeded
fn check_sleep_violation(duration: f64, file: &str, line: usize) -> Option<CbPatternViolation> {
    let (pattern_id, desc, severity) = if duration > 300.0 {
        ("CB-126-C", "Test sleep exceeds 300s critical threshold", Severity::Critical)
    } else if duration > 60.0 {
        ("CB-126-B", "Test sleep exceeds 60s Tier 2 threshold", Severity::Error)
    } else if duration > 5.0 {
        ("CB-126-A", "Test sleep exceeds 5s Tier 1 threshold", Severity::Warning)
    } else {
        return None;
    };
    Some(CbPatternViolation {
        pattern_id: pattern_id.to_string(),
        file: file.to_string(),
        line,
        description: desc.to_string(),
        severity,
    })
}

/// CB-126: Detect slow tests that violate tiered TDD feedback requirements
pub fn detect_cb126_slow_tests(project_path: &Path) -> Vec<CbPatternViolation> {
    let mut violations = Vec::new();
    violations.extend(check_makefile_test_targets(project_path));
    violations.extend(check_sleep_durations(project_path));
    violations
}

fn check_makefile_test_targets(project_path: &Path) -> Vec<CbPatternViolation> {
    let mut violations = Vec::new();
    let makefile_path = project_path.join("Makefile");
    let content = match fs::read_to_string(&makefile_path) {
        Ok(c) => c,
        Err(_) => return violations,
    };

    let mut in_test_target = false;
    let mut test_target_line = 0;
    let mut has_proptest_cases = false;
    let file_path = makefile_path.display().to_string();

    for (line_num, line) in content.lines().enumerate() {
        if line.starts_with("test") && line.contains(':') {
            in_test_target = true;
            test_target_line = line_num + 1;
            has_proptest_cases = false;
        }

        if in_test_target {
            if line.contains("PROPTEST_CASES") || line.contains("QUICKCHECK_TESTS") {
                has_proptest_cases = true;
            }
            if is_end_of_makefile_target_generic(line, "test") {
                if !has_proptest_cases && test_target_line > 0 {
                    violations.push(CbPatternViolation {
                        pattern_id: "CB-126-D".to_string(),
                        file: file_path.clone(),
                        line: test_target_line,
                        description: "Test target missing PROPTEST_CASES/QUICKCHECK_TESTS".to_string(),
                        severity: Severity::Warning,
                    });
                }
                in_test_target = false;
            }
        }
    }
    violations
}

fn is_end_of_makefile_target_generic(line: &str, target_prefix: &str) -> bool {
    line.is_empty()
        || (line.chars().next().map(|c| !c.is_whitespace()).unwrap_or(false)
            && !line.starts_with('\t')
            && !line.starts_with(target_prefix))
}

fn check_sleep_durations(project_path: &Path) -> Vec<CbPatternViolation> {
    let mut violations = Vec::new();
    let src_dir = project_path.join("src");

    let entries = match walkdir_rs_files(&src_dir) {
        Ok(e) => e,
        Err(_) => return violations,
    };

    for entry in entries {
        let content = match fs::read_to_string(&entry) {
            Ok(c) => c,
            Err(_) => continue,
        };
        let file_path = entry.display().to_string();

        for (i, line) in content.lines().enumerate() {
            if line.contains("thread::sleep") && line.contains("Duration::from_secs") {
                if let Some(duration) = extract_sleep_duration(line) {
                    if let Some(v) = check_sleep_violation(duration, &file_path, i + 1) {
                        violations.push(v);
                    }
                }
            }
        }
    }
    violations
}

/// Helper to extract sleep duration from a line like `thread::sleep(Duration::from_secs(10))`
fn extract_sleep_duration(line: &str) -> Option<f64> {
    if let Some(start) = line.find("from_secs(") {
        let after = &line[start + 10..];
        if let Some(end) = after.find(')') {
            let num_str = &after[..end];
            return num_str.trim().parse::<f64>().ok();
        }
    }
    if let Some(start) = line.find("from_millis(") {
        let after = &line[start + 12..];
        if let Some(end) = after.find(')') {
            let num_str = &after[..end];
            if let Ok(millis) = num_str.trim().parse::<f64>() {
                return Some(millis / 1000.0);
            }
        }
    }
    None
}

/// State for tracking coverage target parsing
#[derive(Default)]
struct CoverageTargetState {
    active: bool,
    line: usize,
    has_nextest: bool,
    has_llvm_cov: bool,
    has_proptest_cases: bool,
    has_lib_flag: bool,
    /// Whether this target actually runs cargo tests (vs. report/clean/alias/deno)
    runs_cargo_tests: bool,
}

impl CoverageTargetState {
    fn reset(&mut self, line: usize) {
        self.active = true;
        self.line = line;
        self.has_nextest = false;
        self.has_llvm_cov = false;
        self.has_proptest_cases = false;
        self.has_lib_flag = false;
        self.runs_cargo_tests = false;
    }

    fn update_from_line(&mut self, line: &str) {
        let trimmed = line.trim();
        // Skip comments and echo statements
        if trimmed.starts_with('#') || trimmed.starts_with("@#") {
            return;
        }
        let is_echo = trimmed.starts_with("@echo") || trimmed.starts_with("echo");
        if !is_echo && line.contains("nextest") {
            self.has_nextest = true;
            self.runs_cargo_tests = true;
        }
        if line.contains("llvm-cov") || line.contains("cargo-llvm-cov") {
            self.has_llvm_cov = true;
        }
        // Detect actual test execution: `cargo test` or `cargo llvm-cov test`
        // Exclude report-only commands like `cargo llvm-cov report`
        if !is_echo && (line.contains("cargo test") || line.contains("cargo llvm-cov test")) {
            self.runs_cargo_tests = true;
        }
        if line.contains("PROPTEST_CASES") || line.contains("QUICKCHECK_TESTS") {
            self.has_proptest_cases = true;
        }
        if line.contains("--lib") {
            self.has_lib_flag = true;
        }
    }

    fn collect_violations(&self, file_path: &str) -> Vec<CbPatternViolation> {
        let mut violations = Vec::new();

        // Only flag targets that actually run cargo tests.
        // Skip: alias/delegate targets, report-only, clean, open, invalidate, deno targets.
        if !self.runs_cargo_tests {
            return violations;
        }

        if self.has_nextest && self.has_llvm_cov {
            violations.push(CbPatternViolation {
                pattern_id: "CB-127-A".to_string(),
                file: file_path.to_string(),
                line: self.line,
                description: "CRITICAL: nextest + llvm-cov causes profraw explosion. \
                    Use 'cargo llvm-cov test' instead".to_string(),
                severity: Severity::Error,
            });
        }
        if !self.has_proptest_cases {
            violations.push(CbPatternViolation {
                pattern_id: "CB-127-B".to_string(),
                file: file_path.to_string(),
                line: self.line,
                description: "Coverage target missing PROPTEST_CASES/QUICKCHECK_TESTS".to_string(),
                severity: Severity::Warning,
            });
        }
        if !self.has_lib_flag && self.has_llvm_cov {
            violations.push(CbPatternViolation {
                pattern_id: "CB-127-C".to_string(),
                file: file_path.to_string(),
                line: self.line,
                description: "Coverage target missing --lib flag".to_string(),
                severity: Severity::Warning,
            });
        }
        violations
    }
}

fn is_end_of_makefile_target(line: &str) -> bool {
    line.is_empty()
        || (line.chars().next().map(|c| !c.is_whitespace()).unwrap_or(false)
            && !line.starts_with('\t')
            && !line.starts_with("coverage"))
}

/// CB-127: Detect slow coverage configurations
/// Per [PERF-001] certeza: coverage budget <2min for Tier 2
pub fn detect_cb127_slow_coverage(project_path: &Path) -> Vec<CbPatternViolation> {
    let mut violations = Vec::new();
    let makefile_path = project_path.join("Makefile");

    let content = match fs::read_to_string(&makefile_path) {
        Ok(c) => c,
        Err(_) => return violations,
    };

    let mut state = CoverageTargetState::default();
    let file_path = makefile_path.display().to_string();

    for (line_num, line) in content.lines().enumerate() {
        // Detect coverage target start
        if (line.starts_with("coverage") || line.starts_with("coverage-")) && line.contains(':') {
            state.reset(line_num + 1);
            continue;
        }

        if state.active {
            if is_end_of_makefile_target(line) {
                violations.extend(state.collect_violations(&file_path));
                state.active = false;
            } else {
                state.update_from_line(line);
            }
        }
    }

    violations
}

// =============================================================================
// CB-400: Shell & Makefile Quality (bashrs integration)
// Uses bashrs for deterministic, idempotent, and safe shell scripting.
//
// Sub-checks:
// - CB-400: Git hooks quality (pre-commit, pre-push, etc.)
// - CB-401: Makefile quality
// - CB-402: Shell script quality (*.sh)
// =============================================================================

/// Result of bashrs lint check
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct BashrsLintResult {
    pub file: String,
    pub issues: Vec<BashrsIssue>,
    pub passed: bool,
}

/// Individual bashrs issue
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct BashrsIssue {
    pub code: String,
    pub message: String,
    pub line: usize,
    pub severity: String,
}

/// CB-400: Check git hooks with bashrs
pub fn detect_cb400_git_hooks_quality(project_path: &Path) -> Vec<CbPatternViolation> {
    let mut violations = Vec::new();
    let hooks_dir = project_path.join(".git/hooks");

    if !hooks_dir.exists() {
        return violations;
    }

    // Common hook names to check
    let hook_names = ["pre-commit", "pre-push", "commit-msg", "post-commit"];

    for hook_name in hook_names {
        let hook_path = hooks_dir.join(hook_name);
        if hook_path.exists() && !hook_path.to_string_lossy().ends_with(".sample") {
            // Run bashrs lint on the hook
            match run_bashrs_lint(&hook_path) {
                Ok(issues) if !issues.is_empty() => {
                    for issue in issues {
                        violations.push(CbPatternViolation {
                            pattern_id: format!("CB-400-{}", issue.code),
                            file: format!(".git/hooks/{}", hook_name),
                            line: issue.line,
                            description: format!("{}: {}", issue.code, issue.message),
                            severity: match issue.severity.as_str() {
                                "error" => Severity::Error,
                                "warning" => Severity::Warning,
                                _ => Severity::Info,
                            },
                        });
                    }
                }
                Ok(_) => {} // No issues
                Err(e) => {
                    // bashrs not available or error running it
                    if !e.contains("not found") {
                        violations.push(CbPatternViolation {
                            pattern_id: "CB-400".to_string(),
                            file: format!(".git/hooks/{}", hook_name),
                            line: 0,
                            description: format!("bashrs lint error: {}", e),
                            severity: Severity::Warning,
                        });
                    }
                }
            }
        }
    }

    violations
}

/// CB-401: Check Makefile with bashrs
pub fn detect_cb401_makefile_quality(project_path: &Path) -> Vec<CbPatternViolation> {
    let mut violations = Vec::new();
    let makefile_path = project_path.join("Makefile");

    if !makefile_path.exists() {
        return violations;
    }

    // Run bashrs make lint on Makefile
    match run_bashrs_make_lint(&makefile_path) {
        Ok(issues) if !issues.is_empty() => {
            for issue in issues {
                violations.push(CbPatternViolation {
                    pattern_id: format!("CB-401-{}", issue.code),
                    file: "Makefile".to_string(),
                    line: issue.line,
                    description: format!("{}: {}", issue.code, issue.message),
                    severity: match issue.severity.as_str() {
                        "error" => Severity::Error,
                        "warning" => Severity::Warning,
                        _ => Severity::Info,
                    },
                });
            }
        }
        Ok(_) => {} // No issues
        Err(e) => {
            if !e.contains("not found") {
                violations.push(CbPatternViolation {
                    pattern_id: "CB-401".to_string(),
                    file: "Makefile".to_string(),
                    line: 0,
                    description: format!("bashrs make lint error: {}", e),
                    severity: Severity::Warning,
                });
            }
        }
    }

    violations
}

/// CB-402: Check shell scripts with bashrs
pub fn detect_cb402_shell_script_quality(project_path: &Path) -> Vec<CbPatternViolation> {
    let mut violations = Vec::new();

    // Find all .sh files (limit to reasonable depth)
    let sh_files: Vec<_> = walkdir::WalkDir::new(project_path)
        .max_depth(4)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| {
            let path = e.path();
            path.extension().is_some_and(|ext| ext == "sh")
                && !path.to_string_lossy().contains("target/")
                && !path.to_string_lossy().contains("node_modules/")
        })
        .take(20) // Limit to avoid slow scans
        .collect();

    for entry in sh_files {
        match run_bashrs_lint(entry.path()) {
            Ok(issues) if !issues.is_empty() => {
                for issue in issues {
                    violations.push(CbPatternViolation {
                        pattern_id: format!("CB-402-{}", issue.code),
                        file: entry.path().strip_prefix(project_path)
                            .map(|p| p.display().to_string())
                            .unwrap_or_else(|_| entry.path().display().to_string()),
                        line: issue.line,
                        description: format!("{}: {}", issue.code, issue.message),
                        severity: match issue.severity.as_str() {
                            "error" => Severity::Error,
                            "warning" => Severity::Warning,
                            _ => Severity::Info,
                        },
                    });
                }
            }
            Ok(_) => {} // No issues
            Err(_) => {} // Skip silently for shell scripts
        }
    }

    violations
}

/// Run bashrs lint on a file and parse results
fn run_bashrs_lint(path: &Path) -> Result<Vec<BashrsIssue>, String> {
    use std::process::Command;

    let output = Command::new("bashrs")
        .args(["lint", "--format", "json", "--level", "warning"])
        .arg(path)
        .output()
        .map_err(|e| format!("bashrs not found: {}", e))?;

    if output.status.success() {
        // No issues
        return Ok(Vec::new());
    }

    // Parse JSON output
    let stdout = String::from_utf8_lossy(&output.stdout);
    parse_bashrs_json_output(&stdout)
}

/// Run bashrs make lint on Makefile
fn run_bashrs_make_lint(path: &Path) -> Result<Vec<BashrsIssue>, String> {
    use std::process::Command;

    let output = Command::new("bashrs")
        .args(["make", "lint", "--format", "json"])
        .arg(path)
        .output()
        .map_err(|e| format!("bashrs not found: {}", e))?;

    if output.status.success() {
        return Ok(Vec::new());
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    parse_bashrs_json_output(&stdout)
}

/// Parse bashrs JSON output into issues
fn parse_bashrs_json_output(json_str: &str) -> Result<Vec<BashrsIssue>, String> {
    // bashrs outputs JSON array of diagnostics
    #[derive(serde::Deserialize)]
    struct BashrsOutput {
        #[serde(default)]
        diagnostics: Vec<BashrsDiagnostic>,
    }

    #[derive(serde::Deserialize)]
    struct BashrsDiagnostic {
        code: String,
        message: String,
        #[serde(default)]
        line: usize,
        #[serde(default)]
        severity: String,
    }

    // Try to parse as array first, then as object
    if let Ok(diagnostics) = serde_json::from_str::<Vec<BashrsDiagnostic>>(json_str) {
        return Ok(diagnostics.into_iter().map(|d| BashrsIssue {
            code: d.code,
            message: d.message,
            line: d.line,
            severity: d.severity,
        }).collect());
    }

    if let Ok(output) = serde_json::from_str::<BashrsOutput>(json_str) {
        return Ok(output.diagnostics.into_iter().map(|d| BashrsIssue {
            code: d.code,
            message: d.message,
            line: d.line,
            severity: d.severity,
        }).collect());
    }

    // If JSON parsing fails, return empty (graceful degradation)
    Ok(Vec::new())
}

// =============================================================================
// CB-081: Dependency Count Detection (Enhanced v2.9)
// Per rust-project-score spec: Too many dependencies degrades build times,
// increases supply chain risk, and bloats binaries.
//
// Enhancements:
// - CB-081-A: Base dependency count scoring
// - CB-081-B: Duplicate crate detection
// - CB-081-C: Feature flag hygiene analysis
// - CB-081-D: Sovereign stack bonus
// - CB-081-E: Trend tracking
// =============================================================================

/// Sovereign stack crates (batuta ecosystem)
const SOVEREIGN_CRATES: &[&str] = &[
    "aprender", "trueno", "trueno-graph", "trueno-db", "trueno-rag",
    "trueno-viz", "trueno-zram-core", "pmcp", "presentar-core",
    "renacer", "certeza", "bashrs", "probar", "ruchy",
];

/// Dependency count analysis result (enhanced)
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct DependencyCountReport {
    pub direct_count: usize,
    pub transitive_count: usize,
    pub score: u8,  // 0-5 points based on rust-project-score thresholds
    /// Crates with multiple versions in Cargo.lock
    pub duplicate_crates: Vec<DuplicateCrate>,
    /// Dependencies using default-features = false
    pub feature_gated_count: usize,
    pub feature_gated_pct: f64,
    /// Sovereign stack crates used
    pub sovereign_crates: Vec<String>,
    pub sovereign_bonus: u8,  // 0-3 bonus points
    /// Delta from previous check (if available)
    pub trend: Option<DependencyTrend>,
    pub violations: Vec<CbPatternViolation>,
}

/// Duplicate crate info
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct DuplicateCrate {
    pub name: String,
    pub versions: Vec<String>,
}

/// Trend tracking data
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct DependencyTrend {
    pub direct_delta: i32,
    pub transitive_delta: i32,
    pub previous_timestamp: String,
}

/// CB-081: Detect excessive dependency counts (enhanced)
/// Thresholds from rust-project-score-v1.1-update.md:
/// - 5 points: ≤20 direct, ≤100 transitive
/// - 4 points: ≤30 direct, ≤150 transitive
/// - 3 points: ≤40 direct, ≤200 transitive
/// - 2 points: ≤50 direct, ≤250 transitive
/// - 0 points: >50 direct or >250 transitive
pub fn detect_cb081_dependency_count(project_path: &Path) -> DependencyCountReport {
    let cargo_toml_path = project_path.join("Cargo.toml");
    let cargo_lock_path = project_path.join("Cargo.lock");

    let mut violations = Vec::new();

    // CB-081-A: Count direct dependencies from Cargo.toml
    let (direct_count, feature_gated_count, sovereign_crates) =
        analyze_cargo_toml(&cargo_toml_path);

    // CB-081-A: Count transitive dependencies from Cargo.lock
    let transitive_count = count_transitive_dependencies(&cargo_lock_path);

    // CB-081-B: Detect duplicate crates
    let duplicate_crates = detect_duplicate_crates(&cargo_lock_path);

    // CB-081-C: Calculate feature gating percentage
    let feature_gated_pct = if direct_count > 0 {
        (feature_gated_count as f64 / direct_count as f64) * 100.0
    } else {
        0.0
    };

    // CB-081-D: Calculate sovereign bonus (max +3)
    let sovereign_bonus = std::cmp::min(sovereign_crates.len() as u8, 3);

    // CB-081-E: Load trend data
    let trend = load_dependency_trend(project_path);

    // Calculate base score
    let mut score = calculate_dependency_score(direct_count, transitive_count);

    // Apply bonuses (capped at 5 total)
    if feature_gated_pct >= 50.0 && score < 5 {
        score = std::cmp::min(score + 1, 5);
    }

    // Generate violations based on severity
    // CB-081-A: Count thresholds
    if direct_count > 50 || transitive_count > 250 {
        violations.push(CbPatternViolation {
            pattern_id: "CB-081-A".to_string(),
            file: cargo_toml_path.display().to_string(),
            line: 0,
            description: format!(
                "Critical: {} direct deps (max 50), {} transitive deps (max 250)",
                direct_count, transitive_count
            ),
            severity: Severity::Error,
        });
    } else if direct_count > 40 || transitive_count > 200 {
        violations.push(CbPatternViolation {
            pattern_id: "CB-081-A".to_string(),
            file: cargo_toml_path.display().to_string(),
            line: 0,
            description: format!(
                "High: {} direct (threshold 40), {} transitive (threshold 200)",
                direct_count, transitive_count
            ),
            severity: Severity::Warning,
        });
    }

    // CB-081-B: Duplicate crates
    if !duplicate_crates.is_empty() {
        let dup_names: Vec<_> = duplicate_crates.iter().map(|d| d.name.as_str()).collect();
        violations.push(CbPatternViolation {
            pattern_id: "CB-081-B".to_string(),
            file: cargo_lock_path.display().to_string(),
            line: 0,
            description: format!(
                "{} duplicate crates: {}. Run 'cargo tree --duplicates'",
                duplicate_crates.len(),
                dup_names.join(", ")
            ),
            severity: Severity::Warning,
        });
    }

    // CB-081-C: Low feature gating (only warn if deps exceed excellent tier threshold)
    if direct_count > 20 && feature_gated_pct < 30.0 {
        violations.push(CbPatternViolation {
            pattern_id: "CB-081-C".to_string(),
            file: cargo_toml_path.display().to_string(),
            line: 0,
            description: format!(
                "Only {:.0}% deps use default-features=false. Consider disabling unused features",
                feature_gated_pct
            ),
            severity: Severity::Info,
        });
    }

    // CB-081-E: Trend regression
    if let Some(ref t) = trend {
        let pct_increase = if t.transitive_delta > 0 {
            (t.transitive_delta as f64 / (transitive_count as i32 - t.transitive_delta) as f64) * 100.0
        } else {
            0.0
        };
        if pct_increase > 10.0 {
            violations.push(CbPatternViolation {
                pattern_id: "CB-081-E".to_string(),
                file: cargo_toml_path.display().to_string(),
                line: 0,
                description: format!(
                    "Dependency creep: +{} transitive deps ({:.0}% increase) since {}",
                    t.transitive_delta, pct_increase, t.previous_timestamp
                ),
                severity: Severity::Warning,
            });
        }
    }

    // Save current metrics for future trend tracking
    let _ = save_dependency_metrics(project_path, direct_count, transitive_count);

    DependencyCountReport {
        direct_count,
        transitive_count,
        score,
        duplicate_crates,
        feature_gated_count,
        feature_gated_pct,
        sovereign_crates,
        sovereign_bonus,
        trend,
        violations,
    }
}

/// Analyze Cargo.toml for dependencies, feature gating, and sovereign crates
fn analyze_cargo_toml(cargo_toml_path: &Path) -> (usize, usize, Vec<String>) {
    let content = match fs::read_to_string(cargo_toml_path) {
        Ok(c) => c,
        Err(_) => return (0, 0, Vec::new()),
    };

    let mut direct_count = 0;
    let mut feature_gated_count = 0;
    let mut sovereign_found = Vec::new();
    let mut in_dependencies = false;
    let mut in_dev_dependencies = false;
    let mut in_build_dependencies = false;

    for line in content.lines() {
        let trimmed = line.trim();

        // Track section headers
        if trimmed.starts_with('[') {
            in_dependencies = trimmed == "[dependencies]"
                || trimmed.starts_with("[dependencies.")
                || trimmed.starts_with("[target.");
            in_dev_dependencies = trimmed == "[dev-dependencies]"
                || trimmed.starts_with("[dev-dependencies.");
            in_build_dependencies = trimmed == "[build-dependencies]"
                || trimmed.starts_with("[build-dependencies.");
            continue;
        }

        // Count dependencies (excluding dev, build, and optional deps for scoring)
        if in_dependencies && !in_dev_dependencies && !in_build_dependencies
            && trimmed.contains('=') && !trimmed.starts_with('#')
        {
            // Skip optional dependencies - they don't count toward direct count
            let is_optional = trimmed.contains("optional") && trimmed.contains("true");
            if !is_optional {
                direct_count += 1;
            }

            // Check for default-features = false
            if trimmed.contains("default-features") && trimmed.contains("false") {
                feature_gated_count += 1;
            }

            // Check for sovereign crates
            for crate_name in SOVEREIGN_CRATES {
                if trimmed.starts_with(crate_name)
                    && (trimmed.chars().nth(crate_name.len()) == Some(' ')
                        || trimmed.chars().nth(crate_name.len()) == Some('='))
                {
                    sovereign_found.push(crate_name.to_string());
                }
            }
        }
    }

    (direct_count, feature_gated_count, sovereign_found)
}

/// Count transitive dependencies from Cargo.lock
fn count_transitive_dependencies(cargo_lock_path: &Path) -> usize {
    let content = match fs::read_to_string(cargo_lock_path) {
        Ok(c) => c,
        Err(_) => return 0,
    };

    // Count [[package]] entries in Cargo.lock
    content.matches("[[package]]").count()
}

/// CB-081-B: Detect duplicate crates in Cargo.lock
fn detect_duplicate_crates(cargo_lock_path: &Path) -> Vec<DuplicateCrate> {
    let content = match fs::read_to_string(cargo_lock_path) {
        Ok(c) => c,
        Err(_) => return Vec::new(),
    };

    let mut crate_versions: std::collections::HashMap<String, Vec<String>> =
        std::collections::HashMap::new();

    let mut current_name: Option<String> = None;
    let mut current_version: Option<String> = None;

    for line in content.lines() {
        let trimmed = line.trim();

        if trimmed == "[[package]]" {
            // Save previous package if complete
            if let (Some(name), Some(version)) = (current_name.take(), current_version.take()) {
                crate_versions
                    .entry(name)
                    .or_default()
                    .push(version);
            }
        } else if let Some(name) = trimmed.strip_prefix("name = \"") {
            current_name = name.strip_suffix('"').map(|s| s.to_string());
        } else if let Some(version) = trimmed.strip_prefix("version = \"") {
            current_version = version.strip_suffix('"').map(|s| s.to_string());
        }
    }

    // Don't forget the last package
    if let (Some(name), Some(version)) = (current_name, current_version) {
        crate_versions.entry(name).or_default().push(version);
    }

    // Filter to only duplicates (>1 version)
    crate_versions
        .into_iter()
        .filter(|(_, versions)| versions.len() > 1)
        .map(|(name, mut versions)| {
            versions.sort();
            versions.dedup();
            DuplicateCrate { name, versions }
        })
        .filter(|d| d.versions.len() > 1)
        .collect()
}

/// Calculate dependency health score (0-5 points)
fn calculate_dependency_score(direct: usize, transitive: usize) -> u8 {
    if direct <= 20 && transitive <= 100 {
        5
    } else if direct <= 30 && transitive <= 150 {
        4
    } else if direct <= 40 && transitive <= 200 {
        3
    } else if direct <= 50 && transitive <= 250 {
        2
    } else {
        0
    }
}

/// CB-081-E: Load previous dependency metrics for trend tracking
fn load_dependency_trend(project_path: &Path) -> Option<DependencyTrend> {
    let metrics_path = project_path
        .join(".pmat")
        .join("metrics")
        .join("dependencies.json");

    let content = fs::read_to_string(&metrics_path).ok()?;

    #[derive(serde::Deserialize)]
    #[allow(dead_code)] // Fields used for JSON deserialization structure matching
    struct PreviousMetrics {
        direct_count: usize,
        transitive_count: usize,
        timestamp: String,
    }

    let prev: PreviousMetrics = serde_json::from_str(&content).ok()?;

    // Return trend with previous timestamp - deltas calculated elsewhere
    Some(DependencyTrend {
        direct_delta: 0,
        transitive_delta: 0,
        previous_timestamp: prev.timestamp,
    })
}

/// CB-081-E: Save current dependency metrics for future trend tracking
fn save_dependency_metrics(project_path: &Path, direct: usize, transitive: usize) -> std::io::Result<()> {
    let metrics_dir = project_path.join(".pmat").join("metrics");
    fs::create_dir_all(&metrics_dir)?;

    let metrics_path = metrics_dir.join("dependencies.json");

    // Load previous metrics to calculate deltas
    let previous = if metrics_path.exists() {
        fs::read_to_string(&metrics_path)
            .ok()
            .and_then(|c| serde_json::from_str::<serde_json::Value>(&c).ok())
    } else {
        None
    };

    let timestamp = chrono::Utc::now().to_rfc3339();

    let metrics = serde_json::json!({
        "direct_count": direct,
        "transitive_count": transitive,
        "timestamp": timestamp,
        "previous": previous,
    });

    fs::write(&metrics_path, serde_json::to_string_pretty(&metrics)?)
}

/// Recalculate trend deltas with current counts
#[allow(dead_code)] // Reserved for future trend comparison feature
fn calculate_trend_deltas(
    project_path: &Path,
    current_direct: usize,
    current_transitive: usize,
) -> Option<DependencyTrend> {
    let metrics_path = project_path
        .join(".pmat")
        .join("metrics")
        .join("dependencies.json");

    let content = fs::read_to_string(&metrics_path).ok()?;
    let prev: serde_json::Value = serde_json::from_str(&content).ok()?;

    let prev_direct = prev.get("previous")?.get("direct_count")?.as_u64()? as usize;
    let prev_transitive = prev.get("previous")?.get("transitive_count")?.as_u64()? as usize;
    let prev_timestamp = prev.get("previous")?.get("timestamp")?.as_str()?;

    Some(DependencyTrend {
        direct_delta: current_direct as i32 - prev_direct as i32,
        transitive_delta: current_transitive as i32 - prev_transitive as i32,
        previous_timestamp: prev_timestamp.to_string(),
    })
}

// =============================================================================
// Tests for OIP Tarantula Pattern Detection
// =============================================================================

#[cfg_attr(coverage_nightly, coverage(off))]
#[cfg(test)]
mod oip_tarantula_tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    // CB-120 Tests: NaN-unsafe comparison detection

    #[test]
    fn test_cb120_detects_partial_cmp_unwrap() {
        let temp = TempDir::new().unwrap();
        let src = temp.path().join("src");
        fs::create_dir_all(&src).unwrap();
        fs::write(
            src.join("ml.rs"),
            r#"
fn sort_floats(vec: &mut Vec<f64>) {
    vec.sort_by(|a, b| a.partial_cmp(b).unwrap());
}
"#,
        )
        .unwrap();

        let violations = detect_cb120_nan_unsafe_comparison(temp.path());
        assert_eq!(violations.len(), 1);
        assert_eq!(violations[0].pattern_id, "CB-120");
        assert!(violations[0].description.contains("partial_cmp"));
    }

    #[test]
    fn test_cb120_skips_unwrap_or() {
        let temp = TempDir::new().unwrap();
        let src = temp.path().join("src");
        fs::create_dir_all(&src).unwrap();
        fs::write(
            src.join("safe.rs"),
            r#"
fn sort_floats(vec: &mut Vec<f64>) {
    vec.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
}
"#,
        )
        .unwrap();

        let violations = detect_cb120_nan_unsafe_comparison(temp.path());
        assert!(violations.is_empty());
    }

    #[test]
    fn test_cb120_skips_test_code() {
        let temp = TempDir::new().unwrap();
        let src = temp.path().join("src");
        fs::create_dir_all(&src).unwrap();
        fs::write(
            src.join("lib.rs"),
            r#"
#[cfg_attr(coverage_nightly, coverage(off))]
#[cfg(test)]
mod tests {
    #[test]
    fn test_sort() {
        let mut v = vec![1.0, 2.0];
        v.sort_by(|a, b| a.partial_cmp(b).unwrap());
    }
}
"#,
        )
        .unwrap();

        let violations = detect_cb120_nan_unsafe_comparison(temp.path());
        assert!(violations.is_empty());
    }

    // CB-121 Tests: Lock poisoning detection

    #[test]
    fn test_cb121_detects_mutex_lock_unwrap() {
        let temp = TempDir::new().unwrap();
        let src = temp.path().join("src");
        fs::create_dir_all(&src).unwrap();
        fs::write(
            src.join("sync.rs"),
            r#"
use std::sync::Mutex;
fn get_data(m: &Mutex<i32>) -> i32 {
    *m.lock().unwrap()
}
"#,
        )
        .unwrap();

        let violations = detect_cb121_lock_poisoning(temp.path());
        assert_eq!(violations.len(), 1);
        assert_eq!(violations[0].pattern_id, "CB-121");
    }

    #[test]
    fn test_cb121_skips_into_inner() {
        let temp = TempDir::new().unwrap();
        let src = temp.path().join("src");
        fs::create_dir_all(&src).unwrap();
        fs::write(
            src.join("safe.rs"),
            r#"
use std::sync::Mutex;
fn get_data(m: &Mutex<i32>) -> i32 {
    *m.lock().unwrap_or_else(|e| e.into_inner())
}
"#,
        )
        .unwrap();

        let violations = detect_cb121_lock_poisoning(temp.path());
        assert!(violations.is_empty());
    }

    // CB-122 Tests: Serde deserialization safety

    #[test]
    fn test_cb122_detects_serde_json_unwrap() {
        let temp = TempDir::new().unwrap();
        let src = temp.path().join("src");
        fs::create_dir_all(&src).unwrap();
        fs::write(
            src.join("parser.rs"),
            r#"
fn parse_config(s: &str) -> Config {
    serde_json::from_str(s).unwrap()
}
"#,
        )
        .unwrap();

        let violations = detect_cb122_serde_safety(temp.path());
        assert_eq!(violations.len(), 1);
        assert_eq!(violations[0].pattern_id, "CB-122");
    }

    #[test]
    fn test_cb122_detects_toml_expect() {
        let temp = TempDir::new().unwrap();
        let src = temp.path().join("src");
        fs::create_dir_all(&src).unwrap();
        fs::write(
            src.join("config.rs"),
            r#"
fn load(s: &str) -> Settings {
    toml::from_str(s).expect("invalid toml")
}
"#,
        )
        .unwrap();

        let violations = detect_cb122_serde_safety(temp.path());
        assert_eq!(violations.len(), 1);
    }

    #[test]
    fn test_cb122_skips_test_code() {
        let temp = TempDir::new().unwrap();
        let src = temp.path().join("src");
        fs::create_dir_all(&src).unwrap();
        fs::write(
            src.join("lib.rs"),
            r#"
#[cfg_attr(coverage_nightly, coverage(off))]
#[cfg(test)]
mod tests {
    #[test]
    fn test_parse() {
        let v: Value = serde_json::from_str("{}").unwrap();
    }
}
"#,
        )
        .unwrap();

        let violations = detect_cb122_serde_safety(temp.path());
        assert!(violations.is_empty());
    }

    // CB-123 Tests: Undocumented #[ignore = "compliance detector test"]

    #[test]
    fn test_cb123_detects_bare_ignore() {
        let temp = TempDir::new().unwrap();
        let src = temp.path().join("src");
        fs::create_dir_all(&src).unwrap();
        fs::write(
            src.join("tests.rs"),
            r#"
#[ignore = "compliance detector test"]
#[test]
fn slow_test() {}
"#,
        )
        .unwrap();

        let violations = detect_cb123_undocumented_ignore(temp.path());
        assert_eq!(violations.len(), 1);
        assert_eq!(violations[0].pattern_id, "CB-123");
    }

    #[test]
    fn test_cb123_skips_ignore_with_reason() {
        let temp = TempDir::new().unwrap();
        let src = temp.path().join("src");
        fs::create_dir_all(&src).unwrap();
        fs::write(
            src.join("tests.rs"),
            r#"
#[ignore = "requires GPU"]
#[test]
fn gpu_test() {}
"#,
        )
        .unwrap();

        let violations = detect_cb123_undocumented_ignore(temp.path());
        assert!(violations.is_empty());
    }

    #[test]
    fn test_cb123_skips_ignore_with_comment() {
        let temp = TempDir::new().unwrap();
        let src = temp.path().join("src");
        fs::create_dir_all(&src).unwrap();
        fs::write(
            src.join("tests.rs"),
            r#"
#[ignore] // flaky on CI
#[test]
fn flaky_test() {}
"#,
        )
        .unwrap();

        let violations = detect_cb123_undocumented_ignore(temp.path());
        assert!(violations.is_empty());
    }

    // CB-124 Tests: Coverage threshold enforcement

    #[test]
    fn test_cb124_detects_low_threshold() {
        let temp = TempDir::new().unwrap();
        fs::write(
            temp.path().join("tarpaulin.toml"),
            r#"
[report]
fail_under = 58.0
"#,
        )
        .unwrap();

        let violations = detect_cb124_coverage_threshold(temp.path());
        assert_eq!(violations.len(), 1);
        assert_eq!(violations[0].pattern_id, "CB-124");
        assert_eq!(violations[0].severity, Severity::Error);
    }

    #[test]
    fn test_cb124_warns_below_95() {
        let temp = TempDir::new().unwrap();
        fs::write(
            temp.path().join("tarpaulin.toml"),
            r#"
[report]
fail_under = 85.0
"#,
        )
        .unwrap();

        let violations = detect_cb124_coverage_threshold(temp.path());
        assert_eq!(violations.len(), 1);
        assert_eq!(violations[0].severity, Severity::Warning);
    }

    #[test]
    fn test_cb124_passes_high_threshold() {
        let temp = TempDir::new().unwrap();
        fs::write(
            temp.path().join("tarpaulin.toml"),
            r#"
[report]
fail_under = 95.0
"#,
        )
        .unwrap();

        let violations = detect_cb124_coverage_threshold(temp.path());
        assert!(violations.is_empty());
    }
}

// =============================================================================
// Tests for CB-081 Dependency Count Detection
// =============================================================================

#[cfg_attr(coverage_nightly, coverage(off))]
#[cfg(test)]
mod cb081_dependency_tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn test_cb081_detects_excessive_direct_deps() {
        let temp = TempDir::new().unwrap();

        // Create Cargo.toml with many dependencies (>50)
        let mut deps = String::from("[package]\nname = \"test\"\nversion = \"0.1.0\"\n\n[dependencies]\n");
        for i in 0..60 {
            deps.push_str(&format!("dep{} = \"1.0\"\n", i));
        }
        fs::write(temp.path().join("Cargo.toml"), &deps).unwrap();
        fs::write(temp.path().join("Cargo.lock"), "[[package]]\nname = \"test\"").unwrap();

        let report = detect_cb081_dependency_count(temp.path());
        assert_eq!(report.direct_count, 60);
        assert_eq!(report.score, 0);  // >50 direct = score 0
        assert!(!report.violations.is_empty());
        assert_eq!(report.violations[0].pattern_id, "CB-081-A");
    }

    #[test]
    fn test_cb081_moderate_deps() {
        let temp = TempDir::new().unwrap();

        // Create Cargo.toml with moderate dependencies (30-40)
        let mut deps = String::from("[package]\nname = \"test\"\nversion = \"0.1.0\"\n\n[dependencies]\n");
        for i in 0..35 {
            deps.push_str(&format!("dep{} = \"1.0\"\n", i));
        }
        fs::write(temp.path().join("Cargo.toml"), &deps).unwrap();

        // Create Cargo.lock with 180 packages (between 150-200)
        let mut lock = String::new();
        for _ in 0..180 {
            lock.push_str("[[package]]\nname = \"pkg\"\n");
        }
        fs::write(temp.path().join("Cargo.lock"), &lock).unwrap();

        let report = detect_cb081_dependency_count(temp.path());
        assert_eq!(report.direct_count, 35);
        assert_eq!(report.transitive_count, 180);
        assert_eq!(report.score, 3);  // 30-40 direct, 150-200 transitive = 3
    }

    #[test]
    fn test_cb081_low_deps_excellent() {
        let temp = TempDir::new().unwrap();

        // Create Cargo.toml with few dependencies (≤20)
        let mut deps = String::from("[package]\nname = \"test\"\nversion = \"0.1.0\"\n\n[dependencies]\n");
        for i in 0..15 {
            deps.push_str(&format!("dep{} = \"1.0\"\n", i));
        }
        fs::write(temp.path().join("Cargo.toml"), &deps).unwrap();

        // Create Cargo.lock with few packages (≤100)
        let mut lock = String::new();
        for _ in 0..80 {
            lock.push_str("[[package]]\nname = \"pkg\"\n");
        }
        fs::write(temp.path().join("Cargo.lock"), &lock).unwrap();

        let report = detect_cb081_dependency_count(temp.path());
        assert_eq!(report.direct_count, 15);
        assert_eq!(report.transitive_count, 80);
        assert_eq!(report.score, 5);  // ≤20 direct, ≤100 transitive = 5
        assert!(report.violations.is_empty());
    }

    #[test]
    fn test_cb081_excludes_dev_dependencies() {
        let temp = TempDir::new().unwrap();

        // Create Cargo.toml with few regular deps but many dev-deps
        let deps = r#"
[package]
name = "test"
version = "0.1.0"

[dependencies]
serde = "1.0"
anyhow = "1.0"

[dev-dependencies]
criterion = "0.5"
tempfile = "3.0"
proptest = "1.0"
quickcheck = "1.0"
tokio-test = "0.4"
"#;
        fs::write(temp.path().join("Cargo.toml"), deps).unwrap();
        fs::write(temp.path().join("Cargo.lock"), "[[package]]\nname = \"test\"").unwrap();

        let report = detect_cb081_dependency_count(temp.path());
        // Only counts [dependencies], not [dev-dependencies]
        assert_eq!(report.direct_count, 2);
    }

    #[test]
    fn test_cb081_no_cargo_toml() {
        let temp = TempDir::new().unwrap();
        // No Cargo.toml

        let report = detect_cb081_dependency_count(temp.path());
        assert_eq!(report.direct_count, 0);
        assert_eq!(report.transitive_count, 0);
    }

    // =========================================================================
    // CB-400/401/402 bashrs integration tests
    // =========================================================================

    #[test]
    fn test_cb400_no_git_hooks_dir() {
        let temp = TempDir::new().unwrap();
        // No .git/hooks directory
        let violations = detect_cb400_git_hooks_quality(temp.path());
        assert!(violations.is_empty(), "No hooks dir should return empty");
    }

    #[test]
    fn test_cb400_empty_git_hooks_dir() {
        let temp = TempDir::new().unwrap();
        fs::create_dir_all(temp.path().join(".git/hooks")).unwrap();
        // Empty hooks dir - no hook files
        let violations = detect_cb400_git_hooks_quality(temp.path());
        assert!(violations.is_empty(), "Empty hooks dir should return empty");
    }

    #[test]
    fn test_cb400_sample_hooks_ignored() {
        let temp = TempDir::new().unwrap();
        let hooks_dir = temp.path().join(".git/hooks");
        fs::create_dir_all(&hooks_dir).unwrap();
        // Sample hooks should be ignored
        fs::write(hooks_dir.join("pre-commit.sample"), "#!/bin/bash\necho test").unwrap();
        let violations = detect_cb400_git_hooks_quality(temp.path());
        assert!(violations.is_empty(), "Sample hooks should be ignored");
    }

    #[test]
    fn test_cb401_no_makefile() {
        let temp = TempDir::new().unwrap();
        // No Makefile
        let violations = detect_cb401_makefile_quality(temp.path());
        assert!(violations.is_empty(), "No Makefile should return empty");
    }

    #[test]
    fn test_cb402_no_shell_scripts() {
        let temp = TempDir::new().unwrap();
        // No shell scripts
        let violations = detect_cb402_shell_script_quality(temp.path());
        assert!(violations.is_empty(), "No shell scripts should return empty");
    }

    #[test]
    fn test_cb402_target_dir_excluded() {
        let temp = TempDir::new().unwrap();
        // Shell script in target/ should be ignored
        let target_dir = temp.path().join("target");
        fs::create_dir_all(&target_dir).unwrap();
        fs::write(target_dir.join("test.sh"), "#!/bin/bash\necho test").unwrap();
        let violations = detect_cb402_shell_script_quality(temp.path());
        assert!(violations.is_empty(), "Scripts in target/ should be ignored");
    }

    #[test]
    fn test_parse_bashrs_json_array() {
        let json = r#"[{"code":"SC2086","message":"Double quote","line":5,"severity":"warning"}]"#;
        let result = parse_bashrs_json_output(json).unwrap();
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].code, "SC2086");
        assert_eq!(result[0].line, 5);
    }

    #[test]
    fn test_parse_bashrs_json_object() {
        let json = r#"{"diagnostics":[{"code":"SC2046","message":"Quote this","line":3,"severity":"error"}]}"#;
        let result = parse_bashrs_json_output(json).unwrap();
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].code, "SC2046");
        assert_eq!(result[0].severity, "error");
    }

    #[test]
    fn test_parse_bashrs_json_invalid() {
        let json = "not valid json";
        let result = parse_bashrs_json_output(json).unwrap();
        assert!(result.is_empty(), "Invalid JSON should return empty");
    }

    #[test]
    fn test_parse_bashrs_json_empty_array() {
        let json = "[]";
        let result = parse_bashrs_json_output(json).unwrap();
        assert!(result.is_empty());
    }

    #[test]
    fn test_parse_bashrs_json_multiple_issues() {
        let json = r#"[
            {"code":"SC2086","message":"Double quote","line":5,"severity":"warning"},
            {"code":"SC2046","message":"Quote this","line":10,"severity":"error"},
            {"code":"SC2116","message":"Useless echo","line":15,"severity":"info"}
        ]"#;
        let result = parse_bashrs_json_output(json).unwrap();
        assert_eq!(result.len(), 3);
        assert_eq!(result[0].code, "SC2086");
        assert_eq!(result[1].code, "SC2046");
        assert_eq!(result[2].code, "SC2116");
    }
}

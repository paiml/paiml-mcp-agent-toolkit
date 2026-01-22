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
#[derive(Debug, Clone)]
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

//! CB-300: Muda (Seven Wastes) Score for Code Quality
//!
//! Maps Toyota Production System's Seven Wastes to code quality metrics:
//!
//! | Toyota Waste     | Code Equivalent           | Detection                    |
//! |------------------|---------------------------|------------------------------|
//! | Overproduction   | Dead Code                 | `dead_code` analysis (CB-128)|
//! | Waiting          | Slow Tests / Builds       | Test time (CB-126/CB-127)    |
//! | Inventory        | Stale SATD markers        | SATD age > 90 days           |
//! | Transport        | Excessive cloning         | `.clone()` in hot paths      |
//! | Over-processing  | High complexity           | Cyclomatic > 15              |
//! | Motion           | Dependency sprawl         | Dep count / graph depth      |
//! | Defects          | Bugs / Test failures      | Panic count / stub count     |
//!
//! Score: 0-100 (lower is better, 0 = zero waste)

use serde::{Deserialize, Serialize};
use std::path::Path;

/// Muda Waste Report aggregating all seven wastes
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MudaReport {
    /// Overproduction: dead code percentage (0-100)
    pub overproduction: f64,
    /// Waiting: slow test/build score (0-100)
    pub waiting: f64,
    /// Inventory: stale SATD/branch score (0-100)
    pub inventory: f64,
    /// Transport: excessive copying score (0-100)
    pub transport: f64,
    /// Over-processing: complexity waste (0-100)
    pub over_processing: f64,
    /// Motion: dependency sprawl (0-100)
    pub motion: f64,
    /// Defects: bug/panic indicators (0-100)
    pub defects: f64,
    /// Total aggregate score (0-100, lower is better)
    pub total_score: f64,
    /// Grade based on total score
    pub grade: MudaGrade,
}

/// Muda grade classification
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum MudaGrade {
    /// 0-20: Lean (minimal waste)
    Lean,
    /// 21-40: Efficient
    Efficient,
    /// 41-60: Moderate waste
    Moderate,
    /// 61-80: High waste
    High,
    /// 81-100: Critical waste
    Critical,
}

impl std::fmt::Display for MudaGrade {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            MudaGrade::Lean => write!(f, "Lean"),
            MudaGrade::Efficient => write!(f, "Efficient"),
            MudaGrade::Moderate => write!(f, "Moderate"),
            MudaGrade::High => write!(f, "High"),
            MudaGrade::Critical => write!(f, "Critical"),
        }
    }
}

impl MudaGrade {
    fn from_score(score: f64) -> Self {
        match score as u32 {
            0..=20 => MudaGrade::Lean,
            21..=40 => MudaGrade::Efficient,
            41..=60 => MudaGrade::Moderate,
            61..=80 => MudaGrade::High,
            _ => MudaGrade::Critical,
        }
    }
}

/// Calculate the Muda Waste Score for a project.
///
/// Weights: Defects (25%), Inventory (20%), Over-processing (15%),
///          Overproduction (15%), Waiting (15%), Motion (5%), Transport (5%)
///
/// Inventory (SATD) elevated to 20% — stale TODO/FIXME/HACK accumulation
/// is a primary signal of unmaintained code and must not be masked.
pub fn calculate_muda_score(project_path: &Path) -> MudaReport {
    let overproduction = measure_overproduction(project_path);
    let waiting = measure_waiting(project_path);
    let inventory = measure_inventory(project_path);
    let transport = measure_transport(project_path);
    let over_processing = measure_over_processing(project_path);
    let motion = measure_motion(project_path);
    let defects = measure_defects(project_path);

    // Weighted average (weights sum to 1.0)
    // Inventory elevated: stale SATD is a primary waste signal
    let total_score = (defects * 0.25)
        + (inventory * 0.20)
        + (over_processing * 0.15)
        + (overproduction * 0.15)
        + (waiting * 0.15)
        + (motion * 0.05)
        + (transport * 0.05);

    let total_score = total_score.clamp(0.0, 100.0);
    let grade = MudaGrade::from_score(total_score);

    MudaReport {
        overproduction,
        waiting,
        inventory,
        transport,
        over_processing,
        motion,
        defects,
        total_score,
        grade,
    }
}

/// Overproduction waste: dead code percentage
/// Uses cached dead-code analysis if available
fn measure_overproduction(project_path: &Path) -> f64 {
    let cache_path = project_path.join(".pmat/dead-code-cache.json");
    if let Ok(content) = std::fs::read_to_string(&cache_path) {
        if let Ok(json) = serde_json::from_str::<serde_json::Value>(&content) {
            // Look for dead code percentage in cache
            if let Some(pct) = json.get("dead_code_percentage").and_then(|v| v.as_f64()) {
                // Scale: 0% dead = 0 waste, 10%+ dead = 100 waste
                return (pct * 10.0).clamp(0.0, 100.0);
            }
            // Alternative: count dead items
            if let Some(items) = json.get("dead_items").and_then(|v| v.as_array()) {
                let count = items.len();
                return ((count as f64) * 2.0).clamp(0.0, 100.0);
            }
        }
    }
    0.0 // No data = assume no waste (conservative)
}

/// Waiting waste: slow tests and builds
/// Checks cached test timing and build metrics
fn measure_waiting(project_path: &Path) -> f64 {
    let mut score = 0.0;

    // Check test timing from hooks cache
    let metrics_path = project_path.join(".pmat/hooks-cache/metrics.json");
    if let Ok(content) = std::fs::read_to_string(&metrics_path) {
        if let Ok(json) = serde_json::from_str::<serde_json::Value>(&content) {
            // Check test-fast duration
            if let Some(test_ms) = json
                .get("test-fast")
                .and_then(|v| v.get("duration_ms"))
                .and_then(|v| v.as_f64())
            {
                let test_secs = test_ms / 1000.0;
                // Scale: <30s = 0 waste, >300s = 100 waste
                score += ((test_secs - 30.0) / 270.0 * 100.0).clamp(0.0, 100.0) * 0.5;
            }
            // Check lint duration
            if let Some(lint_ms) = json
                .get("lint")
                .and_then(|v| v.get("duration_ms"))
                .and_then(|v| v.as_f64())
            {
                let lint_secs = lint_ms / 1000.0;
                // Scale: <10s = 0 waste, >60s = 100 waste
                score += ((lint_secs - 10.0) / 50.0 * 100.0).clamp(0.0, 100.0) * 0.5;
            }
        }
    }

    score.clamp(0.0, 100.0)
}

/// Inventory waste: stale SATD markers (TODO/FIXME/HACK)
/// Counts SATD markers as inventory that should be processed
fn measure_inventory(project_path: &Path) -> f64 {
    // Check for SATD count in cached analysis
    let satd_cache = project_path.join(".pmat/satd-cache.json");
    if let Ok(content) = std::fs::read_to_string(&satd_cache) {
        if let Ok(json) = serde_json::from_str::<serde_json::Value>(&content) {
            if let Some(count) = json.get("total_count").and_then(|v| v.as_u64()) {
                // Scale: 0 SATD = 0 waste, 50+ SATD = 100 waste
                return ((count as f64) * 2.0).clamp(0.0, 100.0);
            }
        }
    }

    // Quick heuristic: count TODO/FIXME in source
    let count = count_satd_markers(project_path);
    ((count as f64) * 2.0).clamp(0.0, 100.0)
}

/// Count SATD markers in Rust source files (quick heuristic)
fn count_satd_markers(project_path: &Path) -> usize {
    let src_dir = project_path.join("src");
    if !src_dir.exists() {
        return 0;
    }

    collect_rs_source_files(&src_dir)
        .iter()
        .filter_map(|p| std::fs::read_to_string(p).ok())
        .map(|content| count_satd_in_content(&content))
        .sum()
}

/// Collect .rs files under a directory, excluding test files.
fn collect_rs_source_files(src_dir: &Path) -> Vec<std::path::PathBuf> {
    walkdir::WalkDir::new(src_dir)
        .max_depth(5)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| {
            e.path().extension().is_some_and(|ext| ext == "rs")
                && !e.path().to_string_lossy().contains("test")
        })
        .map(|e| e.path().to_path_buf())
        .collect()
}

/// Count SATD markers (TODO/FIXME/HACK) in file content.
/// Only counts markers in actual production code comments, not string literals,
/// doc comments, security annotations, or test modules.
fn count_satd_in_content(content: &str) -> usize {
    let mut count = 0;
    let mut in_raw_string = false;
    let mut in_test_module = false;

    for line in content.lines() {
        let trimmed = line.trim();

        // Stop counting after #[cfg(test)] — everything below is test code
        if trimmed == "#[cfg(test)]" || trimmed.starts_with("#[cfg(all(test,") {
            in_test_module = true;
        }
        if in_test_module {
            continue;
        }

        // Track raw string boundaries (r#"..."#) to skip embedded comments
        if !in_raw_string && trimmed.contains("r#\"") {
            in_raw_string = true;
            // Check if it closes on the same line
            if trimmed.contains("\"#") && trimmed.rfind("\"#") > trimmed.find("r#\"") {
                in_raw_string = false;
            }
            continue;
        }
        if in_raw_string {
            if trimmed.contains("\"#") {
                in_raw_string = false;
            }
            continue;
        }

        if is_satd_marker(trimmed) {
            count += 1;
        }
    }
    count
}

/// Check if a line is a genuine SATD comment (not a string literal,
/// doc comment, or security annotation).
fn is_satd_marker(trimmed: &str) -> bool {
    // Must be a line comment (not doc comment)
    if !trimmed.starts_with("//") || trimmed.starts_with("///") || trimmed.starts_with("//!") {
        return false;
    }

    // Extract the comment text after //
    let comment = trimmed.get(2..).unwrap_or("");

    // Exclude security annotations — these are hardening notes, not debt
    if comment.trim_start().starts_with("SECURITY:")
        || comment.trim_start().starts_with("SAFETY:")
    {
        return false;
    }

    // The marker must appear in the comment text itself, not in a string
    // literal that happens to be on this line. If the line has quotes before
    // the marker, it's likely a string literal reference.
    let has_marker = comment.contains("TODO") || comment.contains("FIXME") || comment.contains("HACK");
    if !has_marker {
        return false;
    }

    // Exclude lines where the marker appears only inside quotes
    // (test fixtures, string constants referencing SATD patterns)
    let unquoted = strip_quoted_strings(trimmed);
    unquoted.contains("TODO") || unquoted.contains("FIXME") || unquoted.contains("HACK")
}

/// Strip content inside double quotes to avoid matching string literals.
fn strip_quoted_strings(s: &str) -> String {
    let mut result = String::with_capacity(s.len());
    let mut in_quote = false;
    let mut prev_escape = false;
    for ch in s.chars() {
        if ch == '"' && !prev_escape {
            in_quote = !in_quote;
        } else if !in_quote {
            result.push(ch);
        }
        prev_escape = ch == '\\' && !prev_escape;
    }
    result
}

/// Transport waste: excessive data copying (.clone() density)
fn measure_transport(project_path: &Path) -> f64 {
    let src_dir = project_path.join("src");
    if !src_dir.exists() {
        return 0.0;
    }

    let mut total_lines = 0usize;
    let mut clone_calls = 0usize;

    if let Ok(entries) = walkdir::WalkDir::new(&src_dir)
        .max_depth(5)
        .into_iter()
        .collect::<Result<Vec<_>, _>>()
    {
        for entry in entries.iter().filter(|e| {
            e.path().extension().is_some_and(|ext| ext == "rs")
                && !e.path().to_string_lossy().contains("test")
        }) {
            if let Ok(content) = std::fs::read_to_string(entry.path()) {
                total_lines += content.lines().count();
                clone_calls += content.matches(".clone()").count();
            }
        }
    }

    if total_lines == 0 {
        return 0.0;
    }

    // Clone density: clones per 100 lines
    let density = (clone_calls as f64 / total_lines as f64) * 100.0;
    // Scale: <0.5 clones/100 lines = 0 waste, >5 = 100 waste
    ((density - 0.5) / 4.5 * 100.0).clamp(0.0, 100.0)
}

/// Over-processing waste: cyclomatic complexity
fn measure_over_processing(project_path: &Path) -> f64 {
    // Check cached complexity metrics
    let metrics_path = project_path.join(".pmat/hooks-cache/metrics.json");
    if let Ok(content) = std::fs::read_to_string(&metrics_path) {
        if let Ok(json) = serde_json::from_str::<serde_json::Value>(&content) {
            if let Some(max_cc) = json
                .get("complexity")
                .and_then(|v| v.get("max_cyclomatic"))
                .and_then(|v| v.as_f64())
            {
                // Scale: <10 = 0 waste, >50 = 100 waste
                return ((max_cc - 10.0) / 40.0 * 100.0).clamp(0.0, 100.0);
            }
        }
    }
    20.0 // Default: assume moderate complexity
}

/// Motion waste: dependency sprawl
fn measure_motion(project_path: &Path) -> f64 {
    let cargo_lock = project_path.join("Cargo.lock");
    if !cargo_lock.exists() {
        return 0.0;
    }

    // Count dependency packages in Cargo.lock
    if let Ok(content) = std::fs::read_to_string(&cargo_lock) {
        let dep_count = content.matches("[[package]]").count();
        // Scale: <50 deps = 0 waste, >500 deps = 100 waste
        return ((dep_count as f64 - 50.0) / 450.0 * 100.0).clamp(0.0, 100.0);
    }
    0.0
}

/// Defects waste: stub/panic indicators
fn measure_defects(project_path: &Path) -> f64 {
    let src_dir = project_path.join("src");
    if !src_dir.exists() {
        return 0.0;
    }

    let mut stub_count = 0usize;
    let mut unwrap_count = 0usize;

    if let Ok(entries) = walkdir::WalkDir::new(&src_dir)
        .max_depth(5)
        .into_iter()
        .collect::<Result<Vec<_>, _>>()
    {
        for entry in entries.iter().filter(|e| {
            e.path().extension().is_some_and(|ext| ext == "rs")
                && !e.path().to_string_lossy().contains("test")
        }) {
            if let Ok(content) = std::fs::read_to_string(entry.path()) {
                stub_count += content.matches("todo!()").count();
                stub_count += content.matches("unimplemented!()").count();
                unwrap_count += content.matches(".unwrap()").count();
            }
        }
    }

    // Stubs are critical (10 points each), unwraps are moderate (1 point each)
    let score = (stub_count as f64 * 10.0) + (unwrap_count as f64 * 0.5);
    score.clamp(0.0, 100.0)
}

#[cfg_attr(coverage_nightly, coverage(off))]
#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn test_muda_grade_classification() {
        assert_eq!(MudaGrade::from_score(0.0), MudaGrade::Lean);
        assert_eq!(MudaGrade::from_score(20.0), MudaGrade::Lean);
        assert_eq!(MudaGrade::from_score(21.0), MudaGrade::Efficient);
        assert_eq!(MudaGrade::from_score(40.0), MudaGrade::Efficient);
        assert_eq!(MudaGrade::from_score(50.0), MudaGrade::Moderate);
        assert_eq!(MudaGrade::from_score(70.0), MudaGrade::High);
        assert_eq!(MudaGrade::from_score(90.0), MudaGrade::Critical);
    }

    #[test]
    fn test_muda_score_on_self() {
        let project_path = PathBuf::from(".");
        let report = calculate_muda_score(&project_path);
        // Total should be in valid range
        assert!(report.total_score >= 0.0);
        assert!(report.total_score <= 100.0);
        // All individual scores should be in range
        assert!(report.overproduction >= 0.0 && report.overproduction <= 100.0);
        assert!(report.waiting >= 0.0 && report.waiting <= 100.0);
        assert!(report.inventory >= 0.0 && report.inventory <= 100.0);
        assert!(report.transport >= 0.0 && report.transport <= 100.0);
        assert!(report.over_processing >= 0.0 && report.over_processing <= 100.0);
        assert!(report.motion >= 0.0 && report.motion <= 100.0);
        assert!(report.defects >= 0.0 && report.defects <= 100.0);
    }

    #[test]
    fn test_muda_grade_display() {
        assert_eq!(format!("{}", MudaGrade::Lean), "Lean");
        assert_eq!(format!("{}", MudaGrade::Critical), "Critical");
    }

    #[test]
    fn test_transport_empty_project() {
        let path = PathBuf::from("/nonexistent/path");
        let score = measure_transport(&path);
        assert_eq!(score, 0.0);
    }

    #[test]
    fn test_motion_no_cargo_lock() {
        let path = PathBuf::from("/nonexistent/path");
        let score = measure_motion(&path);
        assert_eq!(score, 0.0);
    }

    #[test]
    fn test_defects_empty_project() {
        let path = PathBuf::from("/nonexistent/path");
        let score = measure_defects(&path);
        assert_eq!(score, 0.0);
    }

    #[test]
    fn test_is_satd_marker_real_comments() {
        // Real SATD markers
        assert!(is_satd_marker("// TODO: implement this"));
        assert!(is_satd_marker("// FIXME: broken logic"));
        assert!(is_satd_marker("// HACK: temporary workaround"));
        assert!(is_satd_marker("//TODO: no space"));
        assert!(is_satd_marker("// FIXME(noah): needs refactor"));
    }

    #[test]
    fn test_is_satd_marker_excludes_non_comments() {
        // Not comments — should NOT be flagged
        assert!(!is_satd_marker(r#"patterns: vec!["TODO".to_string()]"#));
        assert!(!is_satd_marker(r#"let s = "FIXME: broken";"#));
        assert!(!is_satd_marker("fn check_todo() {"));
        assert!(!is_satd_marker(r#"Regex::new(r"\bHACK\b")"#));
    }

    #[test]
    fn test_is_satd_marker_excludes_doc_comments() {
        assert!(!is_satd_marker("/// TODO: document this"));
        assert!(!is_satd_marker("//! FIXME: module docs"));
    }

    #[test]
    fn test_is_satd_marker_excludes_security_annotations() {
        assert!(!is_satd_marker("// SECURITY: Require 'passed' field to exist"));
        assert!(!is_satd_marker("// SAFETY: this pointer is valid because..."));
    }

    #[test]
    fn test_is_satd_marker_excludes_string_literals_in_comments() {
        // Comments that reference SATD patterns in quotes (meta-discussion)
        assert!(!is_satd_marker(r#"// tracking "TODO" and "FIXME" comments"#));
        assert!(!is_satd_marker(r#"// scans for "HACK" markers"#));
    }

    #[test]
    fn test_count_satd_in_content() {
        let content = r#"
// TODO: real debt marker
/// TODO: doc comment (excluded)
//! FIXME: module doc (excluded)
let x = "TODO: string literal (excluded)";
// SECURITY: FIXME cache validation (excluded)
// HACK: actual hack
fn contains_todo() {} // no marker, just identifier
"#;
        assert_eq!(count_satd_in_content(content), 2); // Only the real TODO and HACK
    }

    #[test]
    fn test_count_satd_skips_test_modules() {
        let content = "// TODO: real debt in production\nfn prod() {}\n\n#[cfg(test)]\nmod tests {\n    // TODO: test marker (excluded)\n    // FIXME: test fix (excluded)\n}\n";
        assert_eq!(count_satd_in_content(content), 1); // Only the production TODO
    }

    #[test]
    fn test_count_satd_skips_raw_string_content() {
        let content = "fn check() {\n    let code = r#\"\n        // TODO: embedded comment\n        // FIXME: also embedded\n    \"#;\n}\n// HACK: real marker\n";
        assert_eq!(count_satd_in_content(content), 1); // Only the real HACK
    }

    #[test]
    fn test_strip_quoted_strings() {
        assert_eq!(strip_quoted_strings(r#"hello "world" foo"#), "hello  foo");
        assert_eq!(strip_quoted_strings(r#""TODO" marker"#), " marker");
        assert_eq!(strip_quoted_strings("no quotes"), "no quotes");
        // Multiple quoted segments
        assert_eq!(
            strip_quoted_strings(r#"vec!["TODO", "FIXME"]"#),
            "vec![, ]"
        );
    }
}

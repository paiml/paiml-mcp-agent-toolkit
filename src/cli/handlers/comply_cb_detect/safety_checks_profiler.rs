// CB-BUDGET: ComputeBrick assertions and BrickProfiler anomaly detection
// Included by safety_checks.rs — no `use` imports or `#!` attributes here.

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

#[provable_contracts_macros::contract("pmat-core.yaml", equation = "path_exists")]
/// Detect bricks without assertions.
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
#[provable_contracts_macros::contract("pmat-core.yaml", equation = "path_exists")]
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
#[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
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

#[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
/// Extract brick name.
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

#[cfg(test)]
mod safety_checks_profiler_tests {
    //! Covers safety_checks_profiler.rs pure helpers (79 uncov on broad, 0% cov).
    use super::*;

    // ── extract_json_number ──

    #[test]
    fn test_extract_json_number_parses_plain_value() {
        assert_eq!(extract_json_number("\"cv\": 0.18,"), Some(0.18));
    }

    #[test]
    fn test_extract_json_number_strips_trailing_comma_and_brace() {
        assert_eq!(extract_json_number("\"cv\": 20.5}"), Some(20.5));
    }

    #[test]
    fn test_extract_json_number_no_colon_returns_none() {
        assert_eq!(extract_json_number("just a string"), None);
    }

    #[test]
    fn test_extract_json_number_non_numeric_returns_none() {
        assert_eq!(extract_json_number("\"x\": not_a_number,"), None);
    }

    // ── extract_brick_name ──

    #[test]
    fn test_extract_brick_name_finds_name_field_above_target() {
        let content = "  \"name\": \"my_brick\",\n  \"cv\": 0.25,\n";
        let name = extract_brick_name(content, "  \"cv\": 0.25,");
        assert_eq!(name, "my_brick");
    }

    #[test]
    fn test_extract_brick_name_finds_brick_name_field() {
        let content = "  \"brick_name\": \"other_brick\",\n  \"cv\": 0.25,\n";
        let name = extract_brick_name(content, "  \"cv\": 0.25,");
        assert_eq!(name, "other_brick");
    }

    #[test]
    fn test_extract_brick_name_target_missing_returns_unknown() {
        let content = "  \"name\": \"x\",\n  \"other\": 1,\n";
        assert_eq!(extract_brick_name(content, "does not appear"), "unknown");
    }

    #[test]
    fn test_extract_brick_name_target_found_but_no_name_above_returns_unknown() {
        let content = "  \"other\": 1,\n  \"cv\": 0.25,\n";
        assert_eq!(extract_brick_name(content, "  \"cv\": 0.25,"), "unknown");
    }

    // ── check_cv_anomaly ──

    #[test]
    fn test_check_cv_anomaly_non_cv_line_returns_none() {
        assert!(check_cv_anomaly("\"other\": 0.5", "").is_none());
    }

    #[test]
    fn test_check_cv_anomaly_below_threshold_returns_none() {
        // cv = 0.1 * 100 = 10% < 15%
        let line = "\"cv\": 0.1,";
        assert!(check_cv_anomaly(line, line).is_none());
    }

    #[test]
    fn test_check_cv_anomaly_above_threshold_ratio_form() {
        // cv = 0.25 * 100 = 25% > 15%
        let line = "\"cv\": 0.25,";
        let a = check_cv_anomaly(line, line).unwrap();
        assert_eq!(a.anomaly_type, "HIGH_CV");
        assert!((a.value - 25.0).abs() < 1e-6);
        assert_eq!(a.threshold, 15.0);
    }

    #[test]
    fn test_check_cv_anomaly_above_threshold_percent_form() {
        // Already a percent (value >= 1.0) → used as-is.
        let line = "\"cv_percent\": 20.0";
        let a = check_cv_anomaly(line, line).unwrap();
        assert!((a.value - 20.0).abs() < 1e-6);
    }

    // ── check_efficiency_anomaly ──

    #[test]
    fn test_check_efficiency_anomaly_non_eff_line_returns_none() {
        assert!(check_efficiency_anomaly("\"cv\": 0.1", "").is_none());
    }

    #[test]
    fn test_check_efficiency_anomaly_above_threshold_returns_none() {
        // 0.5 * 100 = 50% > 25% (threshold is LOW efficiency, so >25 is OK).
        let line = "\"efficiency\": 0.5";
        assert!(check_efficiency_anomaly(line, line).is_none());
    }

    #[test]
    fn test_check_efficiency_anomaly_below_threshold_emits() {
        // 0.1 * 100 = 10% < 25%
        let line = "\"efficiency\": 0.1,";
        let a = check_efficiency_anomaly(line, line).unwrap();
        assert_eq!(a.anomaly_type, "LOW_EFFICIENCY");
        assert!((a.value - 10.0).abs() < 1e-6);
        assert_eq!(a.threshold, 25.0);
    }

    // ── check_profiler_file: combines CV + efficiency over all lines ──

    #[test]
    fn test_check_profiler_file_empty_returns_empty() {
        let v = check_profiler_file("");
        assert!(v.is_empty());
    }

    #[test]
    fn test_check_profiler_file_detects_both_anomaly_types() {
        let content = "  \"name\": \"b1\",\n  \"cv\": 0.25,\n  \"efficiency\": 0.1,\n";
        let v = check_profiler_file(content);
        assert_eq!(v.len(), 2);
        assert!(v.iter().any(|a| a.anomaly_type == "HIGH_CV"));
        assert!(v.iter().any(|a| a.anomaly_type == "LOW_EFFICIENCY"));
    }

    // ── detect_profiler_anomalies: no file → empty ──

    #[test]
    fn test_detect_profiler_anomalies_missing_file_returns_empty() {
        let tmp = tempfile::tempdir().unwrap();
        let v = detect_profiler_anomalies(tmp.path());
        assert!(v.is_empty());
    }

    #[test]
    fn test_detect_profiler_anomalies_reads_brick_profile_json() {
        let tmp = tempfile::tempdir().unwrap();
        std::fs::write(
            tmp.path().join("brick-profile.json"),
            "{\n  \"name\": \"bad\",\n  \"cv\": 0.8\n}",
        )
        .unwrap();
        let v = detect_profiler_anomalies(tmp.path());
        assert!(!v.is_empty());
    }

    // ── detect_bricks_without_assertions: missing src/brick/ → empty ──

    #[test]
    fn test_detect_bricks_without_assertions_missing_dir_returns_empty() {
        let tmp = tempfile::tempdir().unwrap();
        let v = detect_bricks_without_assertions(tmp.path());
        assert!(v.is_empty());
    }

    #[test]
    fn test_detect_bricks_without_assertions_brick_without_assertions_flagged() {
        let tmp = tempfile::tempdir().unwrap();
        let brick_dir = tmp.path().join("src").join("brick");
        std::fs::create_dir_all(&brick_dir).unwrap();
        // impl + Brick + no assert/debug_assert/validate/etc → flagged.
        std::fs::write(
            brick_dir.join("a.rs"),
            "impl Brick for Foo { fn run(&self) {} }\n",
        )
        .unwrap();
        let v = detect_bricks_without_assertions(tmp.path());
        assert_eq!(v.len(), 1);
        assert_eq!(v[0].pattern_id, "CB-BUDGET");
    }

    #[test]
    fn test_detect_bricks_without_assertions_brick_with_assertion_not_flagged() {
        let tmp = tempfile::tempdir().unwrap();
        let brick_dir = tmp.path().join("src").join("brick");
        std::fs::create_dir_all(&brick_dir).unwrap();
        std::fs::write(
            brick_dir.join("a.rs"),
            "impl Brick for Foo { fn run(&self) { assert!(true); } }\n",
        )
        .unwrap();
        let v = detect_bricks_without_assertions(tmp.path());
        assert!(v.is_empty());
    }
}

// CB-BUDGET: ComputeBrick assertions and BrickProfiler anomaly detection
// Included by safety_checks.rs — no `use` imports or `#!` attributes here.

/// Detect ComputeBricks without assertions/validation (CB-BUDGET)
fn check_brick_file_for_assertions(entry: &Path) -> Option<CbPatternViolation> {
    debug_assert!(entry.exists(), "entry must exist: {}", entry.display());
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
    debug_assert!(project_path.exists(), "project_path must exist: {}", project_path.display());
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
    debug_assert!(project_path.exists(), "project_path must exist: {}", project_path.display());
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
    debug_assert!(!content.is_empty(), "content must not be empty");
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

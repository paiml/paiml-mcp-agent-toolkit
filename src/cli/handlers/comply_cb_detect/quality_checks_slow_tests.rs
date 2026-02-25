// CB-126, CB-127: Slow test and coverage detection
// Included from quality_checks.rs - shares parent module scope (no `use` imports)

/// Check sleep duration and return violation if threshold exceeded
pub(super) fn check_sleep_violation(
    duration: f64,
    file: &str,
    line: usize,
) -> Option<CbPatternViolation> {
    let (pattern_id, desc, severity) = if duration > 300.0 {
        (
            "CB-126-C",
            "Test sleep exceeds 300s critical threshold",
            Severity::Critical,
        )
    } else if duration > 60.0 {
        (
            "CB-126-B",
            "Test sleep exceeds 60s Tier 2 threshold",
            Severity::Error,
        )
    } else if duration > 5.0 {
        (
            "CB-126-A",
            "Test sleep exceeds 5s Tier 1 threshold",
            Severity::Warning,
        )
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

pub(super) fn check_makefile_test_targets(project_path: &Path) -> Vec<CbPatternViolation> {
    let makefile_path = project_path.join("Makefile");
    let content = match fs::read_to_string(&makefile_path) {
        Ok(c) => c,
        Err(_) => return Vec::new(),
    };
    find_test_targets_missing_proptest(&content, &makefile_path.display().to_string())
}

fn find_test_targets_missing_proptest(content: &str, file_path: &str) -> Vec<CbPatternViolation> {
    let mut violations = Vec::new();
    let mut in_test_target = false;
    let mut target_line = 0;
    let mut has_proptest = false;
    let mut has_cargo_test = false;

    for (line_num, line) in content.lines().enumerate() {
        if line.starts_with("test") && line.contains(':') {
            in_test_target = true;
            target_line = line_num + 1;
            has_proptest = false;
            has_cargo_test = false;
        }
        if !in_test_target {
            continue;
        }
        has_proptest |= line.contains("PROPTEST_CASES") || line.contains("QUICKCHECK_TESTS");
        has_cargo_test |=
            line.contains("cargo test") || line.contains("cargo +nightly llvm-cov test");

        if is_end_of_makefile_target_generic(line, "test") {
            if has_cargo_test && !has_proptest {
                violations.push(CbPatternViolation {
                    pattern_id: "CB-126-D".to_string(),
                    file: file_path.to_string(),
                    line: target_line,
                    description: "Test target missing PROPTEST_CASES/QUICKCHECK_TESTS".to_string(),
                    severity: Severity::Warning,
                });
            }
            in_test_target = false;
        }
    }
    violations
}

pub(super) fn is_end_of_makefile_target_generic(line: &str, target_prefix: &str) -> bool {
    line.is_empty()
        || (line
            .chars()
            .next()
            .map(|c| !c.is_whitespace())
            .unwrap_or(false)
            && !line.starts_with('\t')
            && !line.starts_with(target_prefix))
}

pub(super) fn check_sleep_durations(project_path: &Path) -> Vec<CbPatternViolation> {
    let src_dir = project_path.join("src");
    let entries = match walkdir_rs_files(&src_dir) {
        Ok(e) => e,
        Err(_) => return Vec::new(),
    };
    entries
        .iter()
        .flat_map(|entry| scan_file_for_sleep_violations(entry))
        .collect()
}

fn scan_file_for_sleep_violations(path: &Path) -> Vec<CbPatternViolation> {
    let content = match fs::read_to_string(path) {
        Ok(c) => c,
        Err(_) => return Vec::new(),
    };
    let file_path = path.display().to_string();
    content
        .lines()
        .enumerate()
        .filter(|(_, line)| line.contains("thread::sleep") && line.contains("Duration::from_secs"))
        .filter_map(|(i, line)| {
            extract_sleep_duration(line)
                .and_then(|dur| check_sleep_violation(dur, &file_path, i + 1))
        })
        .collect()
}

/// Helper to extract sleep duration from a line containing sleep calls
pub(super) fn extract_sleep_duration(line: &str) -> Option<f64> {
    if let Some(start) = line.find("from_secs(") {
        let after = line.get(start + 10..).unwrap_or_default();
        if let Some(end) = after.find(')') {
            let num_str = after.get(..end).unwrap_or_default();
            return num_str.trim().parse::<f64>().ok();
        }
    }
    if let Some(start) = line.find("from_millis(") {
        let after = line.get(start + 12..).unwrap_or_default();
        if let Some(end) = after.find(')') {
            let num_str = after.get(..end).unwrap_or_default();
            if let Ok(millis) = num_str.trim().parse::<f64>() {
                return Some(millis / 1000.0);
            }
        }
    }
    None
}

pub(super) fn is_end_of_makefile_target(line: &str) -> bool {
    line.is_empty()
        || (line
            .chars()
            .next()
            .map(|c| !c.is_whitespace())
            .unwrap_or(false)
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

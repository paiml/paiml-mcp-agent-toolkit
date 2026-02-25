// CB-125: Coverage exclusion gaming detection
// Included from quality_checks.rs - shares parent module scope (no `use` imports)

/// CB-125: Detect coverage exclusion gaming
/// Per [GAME-001] Popper: Unfalsifiable claims are unscientific
/// Per [GAME-002] Google TAP: >20% exclusion indicates gaming
/// Thresholds:
/// - >10 exclusion patterns = Warning (complexity suggests gaming)
/// - >20% LOC excluded = Error (significant coverage blind spot)
/// - >50% LOC excluded = Critical (coverage metric meaningless)
pub fn detect_cb125_coverage_exclusion_gaming(project_path: &Path) -> Vec<CbPatternViolation> {
    let makefile_path = project_path.join("Makefile");
    let content = match fs::read_to_string(&makefile_path) {
        Ok(c) => c,
        Err(_) => return Vec::new(),
    };

    let (exclusion_count, exclusion_line) = count_exclusion_patterns(&content);
    classify_exclusion_severity(exclusion_count, exclusion_line, &makefile_path)
}

/// Count pipe-separated patterns in --ignore-filename-regex lines.
fn count_exclusion_patterns(content: &str) -> (usize, usize) {
    let mut count = 0;
    let mut last_line = 0;
    for (line_num, line) in content.lines().enumerate() {
        if !line.contains("--ignore-filename-regex") {
            continue;
        }
        last_line = line_num + 1;
        let start = line.find('\'').unwrap_or(0);
        let end = line.rfind('\'').unwrap_or(0);
        if start < end {
            count += line
                .get(start + 1..end)
                .unwrap_or_default()
                .matches('|')
                .count()
                + 1;
        }
    }
    (count, last_line)
}

/// Map exclusion pattern count to CB-125 severity tier.
fn classify_exclusion_severity(
    count: usize,
    line: usize,
    makefile_path: &Path,
) -> Vec<CbPatternViolation> {
    let file = makefile_path.display().to_string();
    let (pattern_id, desc, severity) = if count > 50 {
        ("CB-125-C", format!(
            "CRITICAL: {count} coverage exclusion patterns detected. Coverage metric is meaningless. \
            Per [GAME-001] Popper: unfalsifiable coverage claims are unscientific. \
            Reduce to ≤10 patterns (binary entry points only)"
        ), Severity::Critical)
    } else if count > 20 {
        (
            "CB-125-B",
            format!(
                "{count} coverage exclusion patterns exceed 20% budget per [GAME-002] Google TAP. \
            Significant coverage blind spot. Reduce exclusions or document technical debt"
            ),
            Severity::Error,
        )
    } else if count > 10 {
        (
            "CB-125-A",
            format!(
                "{count} coverage exclusion patterns suggests complexity. \
            Consider reducing to ≤10 patterns (binary entry points only)"
            ),
            Severity::Warning,
        )
    } else {
        return Vec::new();
    };
    vec![CbPatternViolation {
        pattern_id: pattern_id.to_string(),
        file,
        line,
        description: desc,
        severity,
    }]
}

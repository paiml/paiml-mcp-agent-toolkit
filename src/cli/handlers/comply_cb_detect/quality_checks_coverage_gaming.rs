// CB-125: Coverage exclusion gaming detection
// Included from quality_checks.rs - shares parent module scope (no `use` imports)

/// CB-125: Detect coverage exclusion gaming
/// Per [GAME-001] Popper: Unfalsifiable claims are unscientific
/// Per [GAME-002] Google TAP: >20% exclusion indicates gaming
/// Thresholds:
/// - >10 exclusion patterns = Warning (complexity suggests gaming)
/// - >20% LOC excluded = Error (significant coverage blind spot)
/// - >50% LOC excluded = Critical (coverage metric meaningless)
#[provable_contracts_macros::contract("pmat-core.yaml", equation = "path_exists")]
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

#[cfg(test)]
mod coverage_gaming_tests {
    //! Covers CB-125 coverage-gaming detection (59 uncov on broad, 0% cov).
    use super::*;

    // ── detect_cb125_coverage_exclusion_gaming: no Makefile → empty ──

    #[test]
    fn test_detect_missing_makefile_returns_empty() {
        let tmp = tempfile::tempdir().unwrap();
        let v = detect_cb125_coverage_exclusion_gaming(tmp.path());
        assert!(v.is_empty());
    }

    // ── count_exclusion_patterns: counting pipe-separated patterns ──

    #[test]
    fn test_count_exclusion_patterns_no_ignore_lines_returns_zero() {
        let content = "all:\n\techo ok\n";
        let (count, line) = count_exclusion_patterns(content);
        assert_eq!(count, 0);
        assert_eq!(line, 0);
    }

    #[test]
    fn test_count_exclusion_patterns_single_pattern_single_line() {
        // One pattern = 0 pipes + 1 = 1.
        let content = "t:\n\tcargo llvm-cov --ignore-filename-regex 'tests/' -- --lib\n";
        let (count, line) = count_exclusion_patterns(content);
        assert_eq!(count, 1);
        assert_eq!(line, 2); // 1-based line of last matching --ignore
    }

    #[test]
    fn test_count_exclusion_patterns_multiple_patterns_via_pipes() {
        // Three patterns: 2 pipes + 1 = 3.
        let content = "t:\n\tcargo llvm-cov --ignore-filename-regex 'tests/|benches/|examples/' -- --lib\n";
        let (count, line) = count_exclusion_patterns(content);
        assert_eq!(count, 3);
        assert_eq!(line, 2);
    }

    #[test]
    fn test_count_exclusion_patterns_multiple_ignore_lines_accumulate() {
        // Two separate --ignore-filename-regex lines, each contributing pattern count.
        let content = "t1:\n\tllvm-cov --ignore-filename-regex 'a|b' --lib\nt2:\n\tllvm-cov --ignore-filename-regex 'c|d|e' --lib\n";
        let (count, line) = count_exclusion_patterns(content);
        assert_eq!(count, 2 + 3); // 'a|b' = 2, 'c|d|e' = 3
        assert_eq!(line, 4); // last matching line
    }

    #[test]
    fn test_count_exclusion_patterns_no_single_quotes_yields_zero() {
        // Without matching single quotes, start < end is false → no increment.
        let content = "t:\n\tllvm-cov --ignore-filename-regex \"tests/\" --lib\n";
        let (count, _line) = count_exclusion_patterns(content);
        assert_eq!(count, 0, "double-quoted pattern should not count");
    }

    // ── classify_exclusion_severity: all 4 tier arms ──

    #[test]
    fn test_classify_below_10_no_violations() {
        // count <= 10 → no violations.
        let v = classify_exclusion_severity(0, 1, std::path::Path::new("Makefile"));
        assert!(v.is_empty());

        let v = classify_exclusion_severity(10, 1, std::path::Path::new("Makefile"));
        assert!(v.is_empty());
    }

    #[test]
    fn test_classify_11_to_20_is_warning_cb125a() {
        let v = classify_exclusion_severity(15, 5, std::path::Path::new("Makefile"));
        assert_eq!(v.len(), 1);
        assert_eq!(v[0].pattern_id, "CB-125-A");
        assert_eq!(v[0].line, 5);
        assert!(matches!(v[0].severity, Severity::Warning));
    }

    #[test]
    fn test_classify_21_to_50_is_error_cb125b() {
        let v = classify_exclusion_severity(35, 7, std::path::Path::new("Makefile"));
        assert_eq!(v.len(), 1);
        assert_eq!(v[0].pattern_id, "CB-125-B");
        assert!(matches!(v[0].severity, Severity::Error));
    }

    #[test]
    fn test_classify_above_50_is_critical_cb125c() {
        let v = classify_exclusion_severity(75, 9, std::path::Path::new("Makefile"));
        assert_eq!(v.len(), 1);
        assert_eq!(v[0].pattern_id, "CB-125-C");
        assert!(matches!(v[0].severity, Severity::Critical));
        assert!(v[0].description.contains("CRITICAL"));
    }

    // ── Integration via detect_cb125 ──

    #[test]
    fn test_detect_cb125_with_many_exclusions_emits_violation() {
        let tmp = tempfile::tempdir().unwrap();
        // Craft a Makefile with 12 pipe-separated patterns → tier A (Warning).
        let patterns = (0..12)
            .map(|i| format!("pat{i}"))
            .collect::<Vec<_>>()
            .join("|");
        let content = format!(
            "t:\n\tcargo llvm-cov --ignore-filename-regex '{patterns}' -- --lib\n"
        );
        std::fs::write(tmp.path().join("Makefile"), content).unwrap();

        let v = detect_cb125_coverage_exclusion_gaming(tmp.path());
        assert_eq!(v.len(), 1);
        assert_eq!(v[0].pattern_id, "CB-125-A");
    }
}

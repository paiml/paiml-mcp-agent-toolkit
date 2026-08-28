// Coverage checking and documentation section validation
// Included by quality_checks_part2.rs

async fn check_coverage(project_path: &Path, min_coverage: f64) -> Result<Vec<QualityViolation>> {
    let mut violations = Vec::new();

    // Read actual coverage from cache files (#228: was hardcoded 75.0)
    let current_coverage = read_coverage_from_cache(project_path);

    if let Some(coverage) = current_coverage {
        if coverage < min_coverage {
            violations.push(QualityViolation {
                check_type: "coverage".to_string(),
                severity: "error".to_string(),
                message: format!(
                    "Code coverage {coverage:.1}% is below minimum {min_coverage:.1}%"
                ),
                file: "project".to_string(),
                line: None,
                details: None,
            });
        }
    }
    // If no coverage data available, skip check (no violation)

    Ok(violations)
}

/// Read coverage percentage from cache files (#228).
///
/// Priority: `.pmat/coverage-cache.json` > `.pmat-metrics/coverage.json`
/// Computes line coverage from per-file hit data in coverage-cache.json.
fn read_coverage_from_cache(project_path: &Path) -> Option<f64> {
    read_coverage_from_detail_cache(project_path)
        .or_else(|| read_coverage_from_metrics(project_path))
}

/// Read line coverage from `.pmat/coverage-cache.json` per-file hit data.
fn read_coverage_from_detail_cache(project_path: &Path) -> Option<f64> {
    let content = std::fs::read_to_string(project_path.join(".pmat/coverage-cache.json")).ok()?;
    let cache: serde_json::Value = serde_json::from_str(&content).ok()?;
    let files = cache.get("files")?.as_object()?;
    let (total, covered) = compute_line_coverage(files);
    (total > 0).then(|| covered as f64 / total as f64 * 100.0)
}

/// Compute total and covered lines from per-file hit maps.
fn compute_line_coverage(files: &serde_json::Map<String, serde_json::Value>) -> (u64, u64) {
    let mut total = 0u64;
    let mut covered = 0u64;
    for line_hits in files.values() {
        let Some(hits) = line_hits.as_object() else {
            continue;
        };
        for count in hits.values() {
            total += 1;
            if count.as_u64().unwrap_or(0) > 0 {
                covered += 1;
            }
        }
    }
    (total, covered)
}

/// Read aggregate coverage from `.pmat-metrics/coverage.json`.
fn read_coverage_from_metrics(project_path: &Path) -> Option<f64> {
    let content = std::fs::read_to_string(project_path.join(".pmat-metrics/coverage.json")).ok()?;
    let cache: serde_json::Value = serde_json::from_str(&content).ok()?;
    cache.get("coverage")?.as_f64()
}

/// The README the sections check reads, or `None` when there is none to read.
///
/// ONE answer to "is there anything here for the sections check to measure".
/// `check_sections` returns an empty violation list for a project with no
/// README, which is the same value it returns for a README carrying all four
/// required sections — so a caller reporting what the gate covered asks this
/// rather than inferring "clean" from a zero, exactly as `read_coverage_from_cache`
/// is asked instead of reading `check_coverage`'s empty list as a pass.
fn sections_source(project_path: &Path) -> Option<PathBuf> {
    let readme = project_path.join("README.md");
    readme.is_file().then_some(readme)
}

async fn check_sections(project_path: &Path) -> Result<Vec<QualityViolation>> {
    let mut violations = Vec::new();

    // Check for required documentation sections
    let Some(readme_path) = sections_source(project_path) else {
        return Ok(violations);
    };
    if let Ok(readme) = tokio::fs::read_to_string(readme_path).await {
        let required_sections = ["Installation", "Usage", "Contributing", "License"];
        for section in required_sections {
            if !readme.contains(&format!("# {section}"))
                && !readme.contains(&format!("## {section}"))
            {
                violations.push(QualityViolation {
                    check_type: "sections".to_string(),
                    severity: "warning".to_string(),
                    message: format!("Missing required section: {section}"),
                    file: "README.md".to_string(),
                    line: None,
                    details: None,
                });
            }
        }
    }

    Ok(violations)
}

#[cfg(test)]
mod coverage_sections_tests {
    //! Covers quality_checks_part2_coverage_sections.rs pure-compute helpers
    //! (73 uncov on broad, 0% cov).
    use super::*;

    // ── compute_line_coverage: sum across per-file hit maps ──

    #[test]
    fn test_compute_line_coverage_empty_files_map_is_zero_zero() {
        let files = serde_json::Map::new();
        let (total, covered) = compute_line_coverage(&files);
        assert_eq!(total, 0);
        assert_eq!(covered, 0);
    }

    #[test]
    fn test_compute_line_coverage_counts_only_object_values() {
        let json = serde_json::json!({
            "src/a.rs": {"1": 5, "2": 0, "3": 12},
            "src/b.rs": "not-an-object",  // skipped by as_object() branch
            "src/c.rs": {"1": 0}
        });
        let files = json.as_object().unwrap().clone();
        let (total, covered) = compute_line_coverage(&files);
        assert_eq!(total, 4, "3 from a.rs + 1 from c.rs = 4");
        assert_eq!(covered, 2, "a.rs:1=5 and a.rs:3=12 → 2 covered");
    }

    #[test]
    fn test_compute_line_coverage_non_numeric_hits_count_as_uncovered() {
        // count.as_u64() returns None for strings → unwrap_or(0) → not > 0 → uncovered.
        let json = serde_json::json!({
            "f.rs": {"1": "notanumber", "2": 3}
        });
        let files = json.as_object().unwrap().clone();
        let (total, covered) = compute_line_coverage(&files);
        assert_eq!(total, 2);
        assert_eq!(covered, 1);
    }

    // ── read_coverage_from_detail_cache ──

    #[test]
    fn test_read_coverage_from_detail_cache_missing_file_is_none() {
        let tmp = tempfile::tempdir().unwrap();
        assert!(read_coverage_from_detail_cache(tmp.path()).is_none());
    }

    #[test]
    fn test_read_coverage_from_detail_cache_malformed_json_is_none() {
        let tmp = tempfile::tempdir().unwrap();
        std::fs::create_dir_all(tmp.path().join(".pmat")).unwrap();
        std::fs::write(tmp.path().join(".pmat/coverage-cache.json"), "not json").unwrap();
        assert!(read_coverage_from_detail_cache(tmp.path()).is_none());
    }

    #[test]
    fn test_read_coverage_from_detail_cache_no_files_key_is_none() {
        let tmp = tempfile::tempdir().unwrap();
        std::fs::create_dir_all(tmp.path().join(".pmat")).unwrap();
        std::fs::write(tmp.path().join(".pmat/coverage-cache.json"), "{}").unwrap();
        assert!(read_coverage_from_detail_cache(tmp.path()).is_none());
    }

    #[test]
    fn test_read_coverage_from_detail_cache_total_zero_is_none() {
        let tmp = tempfile::tempdir().unwrap();
        std::fs::create_dir_all(tmp.path().join(".pmat")).unwrap();
        // Empty files object → total=0 → None.
        std::fs::write(
            tmp.path().join(".pmat/coverage-cache.json"),
            "{\"files\":{}}",
        )
        .unwrap();
        assert!(read_coverage_from_detail_cache(tmp.path()).is_none());
    }

    #[test]
    fn test_read_coverage_from_detail_cache_valid_returns_percentage() {
        let tmp = tempfile::tempdir().unwrap();
        std::fs::create_dir_all(tmp.path().join(".pmat")).unwrap();
        // 4 lines, 3 covered → 75%.
        std::fs::write(
            tmp.path().join(".pmat/coverage-cache.json"),
            "{\"files\":{\"a.rs\":{\"1\":1,\"2\":2,\"3\":0,\"4\":5}}}",
        )
        .unwrap();
        let pct = read_coverage_from_detail_cache(tmp.path()).unwrap();
        assert!((pct - 75.0).abs() < 1e-6);
    }

    // ── read_coverage_from_metrics ──

    #[test]
    fn test_read_coverage_from_metrics_missing_file_is_none() {
        let tmp = tempfile::tempdir().unwrap();
        assert!(read_coverage_from_metrics(tmp.path()).is_none());
    }

    #[test]
    fn test_read_coverage_from_metrics_valid_returns_value() {
        let tmp = tempfile::tempdir().unwrap();
        std::fs::create_dir_all(tmp.path().join(".pmat-metrics")).unwrap();
        std::fs::write(
            tmp.path().join(".pmat-metrics/coverage.json"),
            "{\"coverage\": 92.5}",
        )
        .unwrap();
        let pct = read_coverage_from_metrics(tmp.path()).unwrap();
        assert!((pct - 92.5).abs() < 1e-6);
    }

    #[test]
    fn test_read_coverage_from_metrics_missing_coverage_key_is_none() {
        let tmp = tempfile::tempdir().unwrap();
        std::fs::create_dir_all(tmp.path().join(".pmat-metrics")).unwrap();
        std::fs::write(
            tmp.path().join(".pmat-metrics/coverage.json"),
            "{\"other\": 1}",
        )
        .unwrap();
        assert!(read_coverage_from_metrics(tmp.path()).is_none());
    }

    // ── read_coverage_from_cache: falls back from detail to metrics ──

    #[test]
    fn test_read_coverage_from_cache_detail_preferred_over_metrics() {
        let tmp = tempfile::tempdir().unwrap();
        std::fs::create_dir_all(tmp.path().join(".pmat")).unwrap();
        std::fs::create_dir_all(tmp.path().join(".pmat-metrics")).unwrap();
        // Detail cache present + metrics also present → detail wins.
        std::fs::write(
            tmp.path().join(".pmat/coverage-cache.json"),
            "{\"files\":{\"a.rs\":{\"1\":1,\"2\":0}}}",
        )
        .unwrap(); // 50%
        std::fs::write(
            tmp.path().join(".pmat-metrics/coverage.json"),
            "{\"coverage\": 92.5}",
        )
        .unwrap();
        let pct = read_coverage_from_cache(tmp.path()).unwrap();
        assert!((pct - 50.0).abs() < 1e-6);
    }

    #[test]
    fn test_read_coverage_from_cache_falls_back_to_metrics() {
        let tmp = tempfile::tempdir().unwrap();
        std::fs::create_dir_all(tmp.path().join(".pmat-metrics")).unwrap();
        // No detail cache → fallback to metrics.
        std::fs::write(
            tmp.path().join(".pmat-metrics/coverage.json"),
            "{\"coverage\": 80.0}",
        )
        .unwrap();
        let pct = read_coverage_from_cache(tmp.path()).unwrap();
        assert!((pct - 80.0).abs() < 1e-6);
    }

    #[test]
    fn test_read_coverage_from_cache_no_files_is_none() {
        let tmp = tempfile::tempdir().unwrap();
        assert!(read_coverage_from_cache(tmp.path()).is_none());
    }

    // ── check_coverage async ──

    #[tokio::test]
    async fn test_check_coverage_no_data_no_violations() {
        let tmp = tempfile::tempdir().unwrap();
        let v = check_coverage(tmp.path(), 95.0).await.unwrap();
        assert!(v.is_empty(), "no data → skip, no violations");
    }

    #[tokio::test]
    async fn test_check_coverage_below_threshold_flagged() {
        let tmp = tempfile::tempdir().unwrap();
        std::fs::create_dir_all(tmp.path().join(".pmat-metrics")).unwrap();
        std::fs::write(
            tmp.path().join(".pmat-metrics/coverage.json"),
            "{\"coverage\": 60.0}",
        )
        .unwrap();
        let v = check_coverage(tmp.path(), 95.0).await.unwrap();
        assert_eq!(v.len(), 1);
        assert_eq!(v[0].check_type, "coverage");
        assert_eq!(v[0].severity, "error");
    }

    #[tokio::test]
    async fn test_check_coverage_above_threshold_no_violation() {
        let tmp = tempfile::tempdir().unwrap();
        std::fs::create_dir_all(tmp.path().join(".pmat-metrics")).unwrap();
        std::fs::write(
            tmp.path().join(".pmat-metrics/coverage.json"),
            "{\"coverage\": 97.0}",
        )
        .unwrap();
        let v = check_coverage(tmp.path(), 95.0).await.unwrap();
        assert!(v.is_empty());
    }

    // ── check_sections async ──

    #[tokio::test]
    async fn test_check_sections_no_readme_no_violations() {
        let tmp = tempfile::tempdir().unwrap();
        let v = check_sections(tmp.path()).await.unwrap();
        assert!(v.is_empty());
    }

    #[tokio::test]
    async fn test_check_sections_complete_readme_no_violations() {
        let tmp = tempfile::tempdir().unwrap();
        std::fs::write(
            tmp.path().join("README.md"),
            "# Project\n## Installation\n## Usage\n## Contributing\n## License\n",
        )
        .unwrap();
        let v = check_sections(tmp.path()).await.unwrap();
        assert!(v.is_empty());
    }

    #[tokio::test]
    async fn test_check_sections_missing_sections_all_flagged() {
        let tmp = tempfile::tempdir().unwrap();
        std::fs::write(tmp.path().join("README.md"), "# Project only\n").unwrap();
        let v = check_sections(tmp.path()).await.unwrap();
        assert_eq!(v.len(), 4, "all 4 required sections missing");
        for x in &v {
            assert_eq!(x.check_type, "sections");
            assert_eq!(x.severity, "warning");
        }
    }
}

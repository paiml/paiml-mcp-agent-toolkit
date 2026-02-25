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
    let content =
        std::fs::read_to_string(project_path.join(".pmat-metrics/coverage.json")).ok()?;
    let cache: serde_json::Value = serde_json::from_str(&content).ok()?;
    cache.get("coverage")?.as_f64()
}

async fn check_sections(project_path: &Path) -> Result<Vec<QualityViolation>> {
    let mut violations = Vec::new();

    // Check for required documentation sections
    if let Ok(readme) = tokio::fs::read_to_string(project_path.join("README.md")).await {
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

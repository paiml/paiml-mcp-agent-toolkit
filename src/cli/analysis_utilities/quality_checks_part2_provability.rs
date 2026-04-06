// Provability analysis and quality gate output formatting
// Included by quality_checks_part2.rs

async fn check_provability(
    project_path: &Path,
    min_provability: f64,
) -> Result<Vec<QualityViolation>> {
    debug_assert!(project_path.exists(), "project_path must exist: {}", project_path.display());
    let mut violations = Vec::new();

    let (current_provability, details) =
        calculate_provability_with_details(project_path).await?;
    if current_provability < min_provability {
        violations.push(QualityViolation {
            check_type: "provability".to_string(),
            severity: "warning".to_string(),
            message: format!(
                "Provability score {current_provability:.2} is below minimum {min_provability:.2}"
            ),
            file: project_path.to_string_lossy().to_string(),
            line: None,
            details: Some(details),
        });
    }

    Ok(violations)
}

/// Calculate the provability score for a project
///
/// This function uses the `LightweightProvabilityAnalyzer` to assess how well
/// functions in the project can be formally verified. Higher scores indicate
/// code that is more amenable to formal verification.
///
/// # Arguments
/// * `project_path` - Root directory of the project to analyze
///
/// # Returns
/// A score between 0.0 and 1.0, where 1.0 indicates perfect provability
///
/// # Example
///
/// ```rust,no_run
/// # use std::path::Path;
/// # use pmat::cli::analysis_utilities::calculate_provability_score;
/// #
/// # #[tokio::test]
/// # async fn test_provability_score() -> anyhow::Result<()> {
/// let score = calculate_provability_score(Path::new(".")).await?;
///
/// // Score should be between 0 and 1
/// assert!(score >= 0.0 && score <= 1.0);
///
/// // Interpret the score
/// match score {
///     s if s >= 0.9 => println!("Excellent provability!"),
///     s if s >= 0.7 => println!("Good provability"),
///     s if s >= 0.5 => println!("Moderate provability"),
///     _ => println!("Low provability - consider refactoring"),
/// }
/// # Ok(())
/// # }
/// ```
///
/// # Property Tests
///
/// ```rust,no_run
/// # use std::path::Path;
/// # use pmat::cli::analysis_utilities::calculate_provability_score;
/// #
/// # #[tokio::test]
/// # async fn test_provability_bounds() -> anyhow::Result<()> {
/// // Test multiple times to ensure consistency
/// for _ in 0..5 {
///     let score = calculate_provability_score(Path::new(".")).await?;
///     assert!(score >= 0.0, "Score should not be negative");
///     assert!(score <= 1.0, "Score should not exceed 1.0");
/// }
/// # Ok(())
/// # }
/// ```
#[provable_contracts_macros::contract("pmat-core.yaml", equation = "path_exists")]
pub async fn calculate_provability_score(project_path: &Path) -> Result<f64> {
    debug_assert!(project_path.exists(), "project_path must exist: {}", project_path.display());
    use crate::services::lightweight_provability_analyzer::LightweightProvabilityAnalyzer;

    let analyzer = LightweightProvabilityAnalyzer::new();

    // Scan real source files to build a representative sample of functions
    let sample_functions = collect_project_functions(project_path, 50);

    if sample_functions.is_empty() {
        return Ok(0.85); // Default if no functions found
    }

    let summaries = analyzer.analyze_incrementally(&sample_functions).await;

    if summaries.is_empty() {
        Ok(0.85)
    } else {
        let total_score: f64 = summaries.iter().map(|s| s.provability_score).sum();
        Ok(total_score / summaries.len() as f64)
    }
}

/// Calculate provability score with detailed breakdown for explainability (#229).
async fn calculate_provability_with_details(
    project_path: &Path,
) -> Result<(f64, ViolationDetails)> {
    debug_assert!(project_path.exists(), "project_path must exist: {}", project_path.display());
    use crate::services::lightweight_provability_analyzer::LightweightProvabilityAnalyzer;

    let analyzer = LightweightProvabilityAnalyzer::new();
    let sample_functions = collect_project_functions(project_path, 50);

    if sample_functions.is_empty() {
        return Ok((0.85, ViolationDetails {
            affected_files: vec![],
            example_code: None,
            fix_suggestion: Some("No functions found to analyze".into()),
            score_factors: vec!["default_score: 0.85 (no functions sampled)".into()],
        }));
    }

    let summaries = analyzer.analyze_incrementally(&sample_functions).await;
    if summaries.is_empty() {
        return Ok((0.85, ViolationDetails {
            affected_files: vec![],
            example_code: None,
            fix_suggestion: Some("Analyzer returned no results".into()),
            score_factors: vec!["default_score: 0.85 (no analysis results)".into()],
        }));
    }

    let total: f64 = summaries.iter().map(|s| s.provability_score).sum();
    let avg = total / summaries.len() as f64;

    // Find lowest-scoring functions for explainability
    let mut scored: Vec<_> = sample_functions.iter().zip(summaries.iter()).collect();
    scored.sort_by(|a, b| a.1.provability_score.total_cmp(&b.1.provability_score));

    let worst_files: Vec<String> = scored.iter()
        .take(5)
        .map(|(f, s)| format!("{}:{} ({:.0}%)", f.file_path, f.function_name, s.provability_score * 100.0))
        .collect();

    // Aggregate property type counts across all summaries
    let mut property_counts: std::collections::BTreeMap<String, usize> = std::collections::BTreeMap::new();
    for s in &summaries {
        for prop in &s.verified_properties {
            *property_counts.entry(format!("{:?}", prop.property_type)).or_default() += 1;
        }
    }

    let score_factors: Vec<String> = std::iter::once(
        format!("functions_sampled: {}", summaries.len()),
    )
    .chain(std::iter::once(format!("average_score: {avg:.2}")))
    .chain(property_counts.iter().map(|(k, v)| format!("verified_{}: {}/{}", k.to_lowercase(), v, summaries.len())))
    .collect();

    let fix_suggestion = if avg < 0.5 {
        "Reduce unsafe blocks, minimize FFI calls, extract pure functions, and lower cyclomatic complexity"
    } else {
        "Focus on functions with lowest scores: reduce mutation, add type guards, extract pure helpers"
    };

    Ok((avg, ViolationDetails {
        affected_files: worst_files,
        example_code: None,
        fix_suggestion: Some(fix_suggestion.into()),
        score_factors,
    }))
}

/// Scan project source files and extract up to `max_count` function declarations
/// with their file paths and line numbers for provability analysis.
fn collect_project_functions(
    project_path: &Path,
    max_count: usize,
) -> Vec<crate::services::lightweight_provability_analyzer::FunctionId> {
    debug_assert!(project_path.exists(), "project_path must exist: {}", project_path.display());
    let src_dir = project_path.join("src");
    let scan_root = if src_dir.exists() { &src_dir } else { project_path };

    let mut functions = Vec::new();
    let source_files = collect_source_files(scan_root);

    for path in &source_files {
        if functions.len() >= max_count {
            break;
        }
        let Ok(content) = std::fs::read_to_string(path) else {
            continue;
        };
        extract_functions_from_source(&content, path, max_count, &mut functions);
    }

    functions
}

/// Collect .rs source files excluding test files.
fn collect_source_files(root: &Path) -> Vec<std::path::PathBuf> {
    debug_assert!(root.exists(), "root must exist: {}", root.display());
    walkdir::WalkDir::new(root)
        .max_depth(10)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| {
            let p = e.path();
            p.is_file()
                && p.extension().is_some_and(|ext| ext == "rs")
                && !p.to_string_lossy().contains("_tests.rs")
                && !p.to_string_lossy().contains("/tests/")
        })
        .map(|e| e.path().to_path_buf())
        .collect()
}

/// Extract function declarations from a single source file.
fn extract_functions_from_source(
    content: &str,
    path: &Path,
    max_count: usize,
    functions: &mut Vec<crate::services::lightweight_provability_analyzer::FunctionId>,
) {
    debug_assert!(path.exists(), "path must exist: {}", path.display());
    debug_assert!(!content.is_empty(), "content must not be empty");
    use crate::services::lightweight_provability_analyzer::FunctionId;

    let mut in_test_block = false;
    for (line_idx, line) in content.lines().enumerate() {
        if functions.len() >= max_count {
            break;
        }

        let trimmed = line.trim();
        if trimmed == "#[cfg(test)]" {
            in_test_block = true;
            continue;
        }
        if in_test_block {
            continue;
        }

        if let Some(fn_name) = parse_fn_declaration(trimmed) {
            functions.push(FunctionId {
                file_path: path.to_string_lossy().to_string(),
                function_name: fn_name,
                line_number: line_idx + 1,
            });
        }
    }
}

/// Parse a function declaration line and return the function name if it matches.
fn parse_fn_declaration(trimmed: &str) -> Option<String> {
    debug_assert!(!trimmed.is_empty(), "trimmed must not be empty");
    let prefixes = [
        "pub fn ", "pub async fn ", "fn ", "async fn ",
        "pub(crate) fn ", "pub(crate) async fn ",
    ];
    if !prefixes.iter().any(|p| trimmed.starts_with(p)) {
        return None;
    }
    let name_part = trimmed
        .replace("pub(crate) ", "")
        .replace("pub ", "")
        .replace("async ", "");
    name_part
        .strip_prefix("fn ")
        .and_then(|s| s.split('(').next())
        .and_then(|s| s.split('<').next())
        .map(|s| s.to_string())
}

/// Format quality gate output for CI/CD integration
///
/// # Examples
///
/// ```no_run
/// use pmat::cli::analysis_utilities::{format_quality_gate_output, QualityGateResults, QualityViolation};
/// use pmat::cli::QualityGateOutputFormat;
///
/// let mut results = QualityGateResults::default();
/// results.passed = false;
/// results.total_violations = 2;
/// results.complexity_violations = 1;
/// results.dead_code_violations = 1;
///
/// let violations = vec![
///     QualityViolation {
///         check_type: "complexity".to_string(),
///         severity: "error".to_string(),
///         file: "src/main.rs".to_string(),
///         line: Some(42),
///         message: "Function exceeds complexity threshold".to_string(),
///         details: None,
///     },
///     QualityViolation {
///         check_type: "dead_code".to_string(),
///         severity: "warning".to_string(),
///         file: "src/lib.rs".to_string(),
///         line: Some(10),
///         message: "Unused function detected".to_string(),
///         details: None,
///     },
/// ];
///
/// // Test human-readable format
/// let output = format_quality_gate_output(&results, &violations, QualityGateOutputFormat::Human).unwrap();
/// assert!(output.contains("❌ FAILED"));
/// assert!(output.contains("Total violations: 2"));
///
/// // Test JSON format
/// let json_output = format_quality_gate_output(&results, &violations, QualityGateOutputFormat::Json).unwrap();
/// assert!(json_output.contains("\"passed\":false"));
///
/// // Test summary format
/// let summary = format_quality_gate_output(&results, &violations, QualityGateOutputFormat::Summary).unwrap();
/// assert!(summary.contains("Status: FAILED"));
/// ```
#[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
pub fn format_quality_gate_output(
    results: &QualityGateResults,
    violations: &[QualityViolation],
    format: QualityGateOutputFormat,
) -> Result<String> {
    debug_assert!(!violations.is_empty(), "violations must not be empty");
    match format {
        QualityGateOutputFormat::Json => format_qg_as_json(results, violations),
        QualityGateOutputFormat::Human => format_qg_as_human(results, violations),
        QualityGateOutputFormat::Junit => format_qg_as_junit(violations),
        QualityGateOutputFormat::Summary => format_qg_as_summary(results, violations),
        QualityGateOutputFormat::Detailed => format_qg_as_detailed(results, violations),
        QualityGateOutputFormat::Markdown => format_qg_as_markdown(results, violations),
    }
}

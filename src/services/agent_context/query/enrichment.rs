#![cfg_attr(coverage_nightly, coverage(off))]

use super::types::QueryResult;
use crate::models::churn::FileChurnMetrics;
use std::collections::HashMap;
use std::path::Path;

/// Enrich query results with churn metrics from pre-computed file churn data.
///
/// Maps file-level churn to function-level results. Since churn is computed
/// per-file (not per-function), all functions in the same file share the
/// same churn metrics.
///
/// # Arguments
/// * `results` - Query results to enrich
/// * `file_churn` - Map of relative file path -> churn metrics
///
/// # Example
/// ```rust,no_run
/// use pmat::services::agent_context::{enrich_with_churn, QueryResult};
/// use std::collections::HashMap;
///
/// let mut results = vec![/* ... */];
/// let churn_map: HashMap<String, (u32, f32)> = HashMap::new();
/// enrich_with_churn(&mut results, &churn_map);
/// ```
pub fn enrich_with_churn(results: &mut [QueryResult], file_churn: &HashMap<String, (u32, f32)>) {
    for result in results.iter_mut() {
        if let Some((commit_count, churn_score)) = file_churn.get(&result.file_path) {
            result.commit_count = *commit_count;
            result.churn_score = *churn_score;
        }
    }
}

/// Build a churn lookup map from FileChurnMetrics.
///
/// Converts a slice of file churn metrics into a HashMap keyed by relative path
/// for O(1) lookup during result enrichment.
pub fn build_churn_map(metrics: &[FileChurnMetrics]) -> HashMap<String, (u32, f32)> {
    metrics
        .iter()
        .map(|m| (m.relative_path.clone(), (m.commit_count as u32, m.churn_score)))
        .collect()
}

/// Compute churn for files in query results.
///
/// Uses git log to compute churn metrics for files referenced in query results.
/// This is a convenience function for on-demand churn enrichment.
///
/// # Arguments
/// * `results` - Query results to enrich
/// * `project_root` - Project root path for git operations
/// * `period_days` - Number of days to look back in git history
#[cfg_attr(coverage_nightly, coverage(off))] // Integration: requires git + IncrementalChurnAnalyzer
pub async fn enrich_results_with_churn(
    results: &mut [QueryResult],
    project_root: &Path,
    period_days: u32,
) -> Result<(), String> {
    use crate::services::incremental_churn::IncrementalChurnAnalyzer;

    if results.is_empty() {
        return Ok(());
    }

    // Skip if most results already have cached churn data from index build.
    // Struct/type definitions legitimately have zero churn, so use majority check.
    let cached = results.iter().filter(|r| r.commit_count > 0 || r.churn_score > 0.0).count();
    if cached * 2 > results.len() {
        return Ok(());
    }

    // Collect unique files from results
    let files: Vec<std::path::PathBuf> = results
        .iter()
        .map(|r| project_root.join(&r.file_path))
        .collect::<std::collections::HashSet<_>>()
        .into_iter()
        .collect();

    // Analyze churn for these files
    let analyzer = IncrementalChurnAnalyzer::new(project_root.to_path_buf());
    let analysis = analyzer
        .analyze_incremental(files, period_days)
        .await
        .map_err(|e| format!("Churn analysis failed: {e}"))?;

    // Build lookup map
    let churn_map = build_churn_map(&analysis.files);

    // Enrich results
    enrich_with_churn(results, &churn_map);

    Ok(())
}

/// Detect language from file extension for duplicate detection.
fn detect_language_for_duplication(path: &str) -> Option<crate::services::duplicate_detector::Language> {
    use crate::services::duplicate_detector::Language;
    let ext_langs: &[(&[&str], Language)] = &[
        (&[".rs"], Language::Rust),
        (&[".ts", ".tsx"], Language::TypeScript),
        (&[".js", ".jsx"], Language::JavaScript),
        (&[".py"], Language::Python),
        (&[".c"], Language::C),
        (&[".cpp", ".cc", ".cxx"], Language::Cpp),
        (&[".kt"], Language::Kotlin),
    ];
    ext_langs
        .iter()
        .find(|(exts, _)| exts.iter().any(|ext| path.ends_with(ext)))
        .map(|(_, lang)| *lang)
}

/// Collect unique file contents from query results for analysis.
fn collect_file_contents(
    results: &[QueryResult],
    project_root: &Path,
) -> HashMap<String, String> {
    let mut contents: HashMap<String, String> = HashMap::new();
    for result in results {
        if contents.contains_key(&result.file_path) {
            continue;
        }
        let full_path = project_root.join(&result.file_path);
        if let Ok(content) = std::fs::read_to_string(&full_path) {
            contents.insert(result.file_path.clone(), content);
        }
    }
    contents
}

/// Enrich query results with duplicate detection data.
///
/// Detects code clones using MinHash + LSH for O(1) similarity matching.
/// Results are enriched with clone_count and duplication_score.
#[cfg_attr(coverage_nightly, coverage(off))] // Integration: requires filesystem + DuplicateDetectionEngine
pub async fn enrich_results_with_duplicates(
    results: &mut [QueryResult],
    project_root: &Path,
) -> Result<(), String> {
    use crate::services::duplicate_detector::{DuplicateDetectionConfig, DuplicateDetectionEngine};

    if results.is_empty() {
        return Ok(());
    }

    let file_contents = collect_file_contents(results, project_root);

    // Build file list with detected languages
    let files_to_analyze: Vec<_> = file_contents
        .iter()
        .filter_map(|(path, content)| {
            detect_language_for_duplication(path)
                .map(|lang| (std::path::PathBuf::from(path), content.clone(), lang))
        })
        .collect();

    if files_to_analyze.is_empty() {
        return Ok(());
    }

    let config = DuplicateDetectionConfig {
        min_tokens: 20,
        similarity_threshold: 0.65,
        ..Default::default()
    };

    let engine = DuplicateDetectionEngine::new(config);
    let report = engine
        .detect_duplicates(&files_to_analyze)
        .map_err(|e| format!("Duplicate detection failed: {e}"))?;

    // Build file -> (clone_count, max_similarity) map
    let mut file_duplication: HashMap<String, (u32, f32)> = HashMap::new();
    for group in &report.groups {
        for fragment in &group.fragments {
            let path_str = fragment.file.to_string_lossy().to_string();
            let entry = file_duplication.entry(path_str).or_insert((0, 0.0));
            entry.0 += 1;
            entry.1 = entry.1.max(group.average_similarity as f32);
        }
    }

    for result in results.iter_mut() {
        if let Some((clone_count, dup_score)) = file_duplication.get(&result.file_path) {
            result.clone_count = *clone_count;
            result.duplication_score = *dup_score;
        }
    }

    Ok(())
}

/// Enrich query results with entropy/pattern diversity metrics.
///
/// Analyzes code for repetitive patterns using AST-based pattern extraction.
/// Low pattern diversity indicates code that could benefit from refactoring.
///
/// # Arguments
/// * `results` - Query results to enrich
/// * `project_root` - Project root path for analysis
#[cfg_attr(coverage_nightly, coverage(off))] // Integration: requires EntropyAnalyzer + filesystem
pub async fn enrich_results_with_entropy(
    results: &mut [QueryResult],
    project_root: &Path,
) -> Result<(), String> {
    use crate::entropy::{EntropyAnalyzer, EntropyConfig};

    if results.is_empty() {
        return Ok(());
    }

    // Skip if most results already have cached pattern diversity from index build
    let cached = results.iter().filter(|r| r.pattern_diversity > 0.0).count();
    if cached * 2 > results.len() {
        return Ok(());
    }

    // Run entropy analysis on the project
    let config = EntropyConfig::default();
    let analyzer = EntropyAnalyzer::with_config(config);
    let report = analyzer
        .analyze(project_root)
        .await
        .map_err(|e| format!("Entropy analysis failed: {e}"))?;

    // Get overall pattern diversity
    let overall_diversity = report.entropy_metrics.pattern_diversity as f32;

    // Build file -> pattern count map from violations
    let mut file_pattern_count: HashMap<String, usize> = HashMap::new();
    for violation in &report.actionable_violations {
        for file in &violation.affected_files {
            let path_str = file
                .strip_prefix(project_root)
                .unwrap_or(file)
                .to_string_lossy()
                .to_string();
            *file_pattern_count.entry(path_str).or_insert(0) += 1;
        }
    }

    // Calculate per-file diversity (inverse of pattern repetition)
    let max_patterns = file_pattern_count.values().max().copied().unwrap_or(1) as f32;

    // Enrich results
    for result in results.iter_mut() {
        if let Some(&pattern_count) = file_pattern_count.get(&result.file_path) {
            // Lower diversity = more repetitive patterns
            result.pattern_diversity = 1.0 - (pattern_count as f32 / max_patterns).min(1.0);
        } else {
            // No violations = high diversity (good)
            result.pattern_diversity = overall_diversity;
        }
    }

    Ok(())
}

/// Run batuta bug-hunter and parse findings into a file->annotations map.
fn run_batuta_and_parse(project_root: &Path) -> Result<HashMap<String, Vec<String>>, String> {
    use std::process::Command;

    let output = Command::new("batuta")
        .args(["bug-hunter", "falsify", "--format", "json", "--target", "."])
        .current_dir(project_root)
        .output()
        .map_err(|e| format!("Failed to run batuta: {e}"))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        if !stderr.contains("Usage:") {
            return Err(format!("batuta failed: {stderr}"));
        }
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    let json_start = match stdout.find('{') {
        Some(s) => s,
        None => return Ok(HashMap::new()),
    };

    let parsed: serde_json::Value = serde_json::from_str(&stdout[json_start..])
        .map_err(|e| format!("Failed to parse batuta output: {e}"))?;

    let findings = match parsed.get("findings").and_then(|f| f.as_array()) {
        Some(f) => f,
        None => return Ok(HashMap::new()),
    };

    let mut fault_map: HashMap<String, Vec<String>> = HashMap::new();
    for finding in findings {
        let file = finding.get("file").and_then(|f| f.as_str()).unwrap_or("");
        let line = finding.get("line").and_then(|l| l.as_u64()).unwrap_or(0);
        let title = finding.get("title").and_then(|t| t.as_str()).unwrap_or("Unknown fault pattern");
        let id = finding.get("id").and_then(|i| i.as_str()).unwrap_or("BH");
        let normalized = file.strip_prefix("./").unwrap_or(file);
        fault_map
            .entry(normalized.to_string())
            .or_default()
            .push(format!("{id}: {title} at line {line}"));
    }

    Ok(fault_map)
}

/// Filter fault annotations to those within a function's line range.
fn faults_in_range(faults: &[String], start_line: usize, end_line: usize) -> Vec<String> {
    faults
        .iter()
        .filter(|f| {
            f.split("at line ")
                .last()
                .and_then(|s| s.parse::<usize>().ok())
                .is_some_and(|line| line >= start_line && line <= end_line)
        })
        .cloned()
        .collect()
}

/// Enrich query results with batuta fault pattern annotations.
///
/// Runs batuta bug-hunter falsify to detect mutation targets and boundary conditions.
/// Results are enriched with fault_annotations containing any detected issues.
#[cfg_attr(coverage_nightly, coverage(off))] // Integration: requires pmat subprocess
pub async fn enrich_results_with_faults(
    results: &mut [QueryResult],
    project_root: &Path,
) -> Result<(), String> {
    if results.is_empty() {
        return Ok(());
    }

    // Skip if most results already have cached fault annotations from index build
    let cached = results.iter().filter(|r| !r.fault_annotations.is_empty()).count();
    if cached * 2 > results.len() {
        return Ok(());
    }

    let fault_map = run_batuta_and_parse(project_root)?;

    for result in results.iter_mut() {
        if let Some(faults) = fault_map.get(&result.file_path) {
            let func_end = result.start_line + result.loc as usize;
            let relevant = faults_in_range(faults, result.start_line, func_end);
            if !relevant.is_empty() {
                result.fault_annotations = relevant;
            }
        }
    }

    Ok(())
}

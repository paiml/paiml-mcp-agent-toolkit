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
#[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
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
#[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
pub fn build_churn_map(metrics: &[FileChurnMetrics]) -> HashMap<String, (u32, f32)> {
    metrics
        .iter()
        .map(|m| {
            (
                m.relative_path.clone(),
                (m.commit_count as u32, m.churn_score),
            )
        })
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
#[provable_contracts_macros::contract("pmat-core.yaml", equation = "path_exists")]
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
    let cached = results
        .iter()
        .filter(|r| r.commit_count > 0 || r.churn_score > 0.0)
        .count();
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
fn detect_language_for_duplication(
    path: &str,
) -> Option<crate::services::duplicate_detector::Language> {
    use crate::services::duplicate_detector::Language;
    let ext_langs: &[(&[&str], Language)] = &[
        (&[".rs"], Language::Rust),
        (&[".ts", ".tsx"], Language::TypeScript),
        (&[".js", ".jsx"], Language::JavaScript),
        (&[".py"], Language::Python),
        (&[".c"], Language::C),
        (&[".cpp", ".cc", ".cxx", ".cu", ".cuh"], Language::Cpp),
        (&[".kt"], Language::Kotlin),
    ];
    ext_langs
        .iter()
        .find(|(exts, _)| exts.iter().any(|ext| path.ends_with(ext)))
        .map(|(_, lang)| *lang)
}

/// Collect unique file contents from query results for analysis.
fn collect_file_contents(results: &[QueryResult], project_root: &Path) -> HashMap<String, String> {
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
#[provable_contracts_macros::contract("pmat-core.yaml", equation = "path_exists")]
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
#[provable_contracts_macros::contract("pmat-core.yaml", equation = "path_exists")]
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

    // Run entropy analysis on the project, loading .pmatignore
    let config = EntropyConfig::default().with_project_ignores(project_root);
    let analyzer = EntropyAnalyzer::with_config(config);
    let report = analyzer
        .analyze(project_root)
        .await
        .map_err(|e| format!("Entropy analysis failed: {e}"))?;

    // Get overall pattern diversity. `None` means the analysis found no pattern
    // distribution to measure; leave those results untouched rather than
    // stamping them with a number that was never computed.
    let overall_diversity = report.entropy_metrics.pattern_diversity.map(|d| d as f32);

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
        } else if let Some(diversity) = overall_diversity {
            // No violations = high diversity (good)
            result.pattern_diversity = diversity;
        }
    }

    Ok(())
}

/// Load fault findings from the newest `.pmat/bug-hunter-cache/*.json`.
///
/// Pure cache reader: the cache is populated by `aprender-orchestrate`'s
/// `bug_hunter::hunt` (sovereign stack, formerly `batuta`). Returns an empty
/// map when no cache is present, in which case enrichment is a no-op.
fn load_faults_from_cache(project_root: &Path) -> Result<HashMap<String, Vec<String>>, String> {
    let cache_dir = project_root.join(".pmat/bug-hunter-cache");
    let entries = match std::fs::read_dir(&cache_dir) {
        Ok(e) => e,
        Err(_) => return Ok(HashMap::new()),
    };

    let newest = entries
        .flatten()
        .filter(|e| e.path().extension().is_some_and(|ext| ext == "json"))
        .max_by_key(|e| {
            e.metadata()
                .and_then(|m| m.modified())
                .unwrap_or(std::time::SystemTime::UNIX_EPOCH)
        });

    let entry = match newest {
        Some(e) => e,
        None => return Ok(HashMap::new()),
    };

    let data = match std::fs::read_to_string(entry.path()) {
        Ok(d) => d,
        Err(_) => return Ok(HashMap::new()),
    };

    let parsed: serde_json::Value = match serde_json::from_str(&data) {
        Ok(v) => v,
        Err(_) => return Ok(HashMap::new()),
    };

    let findings = match parsed.get("findings").and_then(|f| f.as_array()) {
        Some(f) => f,
        None => return Ok(HashMap::new()),
    };

    let mut fault_map: HashMap<String, Vec<String>> = HashMap::new();
    for finding in findings {
        let file = finding.get("file").and_then(|f| f.as_str()).unwrap_or("");
        let line = finding.get("line").and_then(|l| l.as_u64()).unwrap_or(0);
        let title = finding
            .get("title")
            .and_then(|t| t.as_str())
            .unwrap_or("Unknown fault pattern");
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

/// Enrich query results with fault pattern annotations from the bug-hunter cache.
///
/// Reads findings from `.pmat/bug-hunter-cache/*.json` (newest file) without
/// spawning any subprocess. If the cache is absent, results are left as-is.
#[cfg_attr(coverage_nightly, coverage(off))] // Integration: requires filesystem cache
#[provable_contracts_macros::contract("pmat-core.yaml", equation = "path_exists")]
pub async fn enrich_results_with_faults(
    results: &mut [QueryResult],
    project_root: &Path,
) -> Result<(), String> {
    if results.is_empty() {
        return Ok(());
    }

    // Skip if most results already have cached fault annotations from index build
    let cached = results
        .iter()
        .filter(|r| !r.fault_annotations.is_empty())
        .count();
    if cached * 2 > results.len() {
        return Ok(());
    }

    let fault_map = load_faults_from_cache(project_root)?;
    if fault_map.is_empty() {
        return Ok(());
    }

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

#[cfg(test)]
mod enrichment_pure_tests {
    //! Covers pure helpers in enrichment.rs: enrich_with_churn,
    //! build_churn_map, detect_language_for_duplication, faults_in_range.
    //! Skips async fns (coverage(off)) and filesystem-bound fns.
    use super::*;
    use crate::services::duplicate_detector::Language;

    fn make_result(path: &str) -> QueryResult {
        let mut r = QueryResult {
            function_name: "f".to_string(),
            file_path: path.to_string(),
            signature: "fn f()".to_string(),
            definition_type: "function".to_string(),
            doc_comment: None,
            start_line: 1,
            end_line: 10,
            language: "Rust".to_string(),
            tdg_score: 80.0,
            tdg_grade: "A".to_string(),
            complexity: 5,
            big_o: "O(1)".to_string(),
            satd_count: 0,
            loc: 10,
            relevance_score: 0.95,
            source: None,
            calls: Vec::new(),
            called_by: Vec::new(),
            pagerank: 0.0,
            in_degree: 0,
            out_degree: 0,
            commit_count: 0,
            churn_score: 0.0,
            clone_count: 0,
            duplication_score: 0.0,
            pattern_diversity: 0.0,
            fault_annotations: Vec::new(),
            line_coverage_pct: 0.0,
            lines_covered: 0,
            lines_total: 0,
            missed_lines: 0,
            impact_score: 0.0,
            coverage_status: String::new(),
            coverage_diff: 0.0,
            coverage_exclusion: Default::default(),
            coverage_excluded: false,
            cross_project_callers: 0,
            io_classification: String::new(),
            io_patterns: Vec::new(),
            suggested_module: String::new(),
            contract_level: None,
            contract_equation: None,
        };
        r.start_line = 0;
        r
    }

    // ── enrich_with_churn ──

    #[test]
    fn test_enrich_with_churn_applies_metrics_for_matching_paths() {
        let mut results = vec![make_result("src/a.rs"), make_result("src/b.rs")];
        let mut churn = HashMap::new();
        churn.insert("src/a.rs".to_string(), (5u32, 0.5_f32));
        enrich_with_churn(&mut results, &churn);
        assert_eq!(results[0].commit_count, 5);
        assert!((results[0].churn_score - 0.5).abs() < 1e-6);
        // Unmatched stays at default.
        assert_eq!(results[1].commit_count, 0);
    }

    #[test]
    fn test_enrich_with_churn_empty_map_no_op() {
        let mut results = vec![make_result("src/a.rs")];
        enrich_with_churn(&mut results, &HashMap::new());
        assert_eq!(results[0].commit_count, 0);
    }

    // ── build_churn_map ──

    #[test]
    fn test_build_churn_map_builds_lookup_from_metrics() {
        use crate::models::churn::FileChurnMetrics;
        use chrono::Utc;
        use std::path::PathBuf;
        let metrics = vec![
            FileChurnMetrics {
                path: PathBuf::from("src/a.rs"),
                relative_path: "src/a.rs".to_string(),
                commit_count: 3,
                unique_authors: vec![],
                additions: 0,
                deletions: 0,
                churn_score: 0.3,
                last_modified: Utc::now(),
                first_seen: Utc::now(),
            },
            FileChurnMetrics {
                path: PathBuf::from("src/b.rs"),
                relative_path: "src/b.rs".to_string(),
                commit_count: 7,
                unique_authors: vec![],
                additions: 0,
                deletions: 0,
                churn_score: 0.7,
                last_modified: Utc::now(),
                first_seen: Utc::now(),
            },
        ];
        let map = build_churn_map(&metrics);
        assert_eq!(map.get("src/a.rs"), Some(&(3u32, 0.3_f32)));
        assert_eq!(map.get("src/b.rs"), Some(&(7u32, 0.7_f32)));
    }

    #[test]
    fn test_build_churn_map_empty_input_returns_empty() {
        let map = build_churn_map(&[]);
        assert!(map.is_empty());
    }

    // ── detect_language_for_duplication ──

    #[test]
    fn test_detect_language_rust() {
        assert!(matches!(
            detect_language_for_duplication("src/foo.rs"),
            Some(Language::Rust)
        ));
    }

    #[test]
    fn test_detect_language_typescript_ts_and_tsx() {
        assert!(matches!(
            detect_language_for_duplication("foo.ts"),
            Some(Language::TypeScript)
        ));
        assert!(matches!(
            detect_language_for_duplication("foo.tsx"),
            Some(Language::TypeScript)
        ));
    }

    #[test]
    fn test_detect_language_javascript_js_and_jsx() {
        assert!(matches!(
            detect_language_for_duplication("foo.js"),
            Some(Language::JavaScript)
        ));
        assert!(matches!(
            detect_language_for_duplication("foo.jsx"),
            Some(Language::JavaScript)
        ));
    }

    #[test]
    fn test_detect_language_python() {
        assert!(matches!(
            detect_language_for_duplication("foo.py"),
            Some(Language::Python)
        ));
    }

    #[test]
    fn test_detect_language_c_and_cpp_variants() {
        assert!(matches!(
            detect_language_for_duplication("a.c"),
            Some(Language::C)
        ));
        for ext in &["cpp", "cc", "cxx", "cu", "cuh"] {
            assert!(matches!(
                detect_language_for_duplication(&format!("a.{ext}")),
                Some(Language::Cpp)
            ));
        }
    }

    #[test]
    fn test_detect_language_kotlin() {
        assert!(matches!(
            detect_language_for_duplication("foo.kt"),
            Some(Language::Kotlin)
        ));
    }

    #[test]
    fn test_detect_language_unknown_returns_none() {
        assert!(detect_language_for_duplication("foo.xyz").is_none());
        assert!(detect_language_for_duplication("foo").is_none());
    }

    // ── faults_in_range ──

    #[test]
    fn test_faults_in_range_includes_in_range_only() {
        let faults = vec![
            "BH-1: x at line 5".to_string(),
            "BH-2: y at line 50".to_string(),
            "BH-3: z at line 105".to_string(),
        ];
        let in_range = faults_in_range(&faults, 1, 100);
        assert_eq!(in_range.len(), 2);
        assert!(in_range.iter().any(|f| f.contains("line 5")));
        assert!(in_range.iter().any(|f| f.contains("line 50")));
    }

    #[test]
    fn test_faults_in_range_empty_input_returns_empty() {
        let in_range = faults_in_range(&[], 1, 100);
        assert!(in_range.is_empty());
    }

    #[test]
    fn test_faults_in_range_skips_unparseable_line_marker() {
        let faults = vec!["BH-1: badly formatted".to_string()];
        let in_range = faults_in_range(&faults, 1, 100);
        // No "at line N" suffix → skipped.
        assert!(in_range.is_empty());
    }

    #[test]
    fn test_faults_in_range_boundary_inclusive() {
        let faults = vec![
            "BH-1: low at line 1".to_string(),
            "BH-2: high at line 100".to_string(),
        ];
        let in_range = faults_in_range(&faults, 1, 100);
        // Both endpoints included.
        assert_eq!(in_range.len(), 2);
    }
}

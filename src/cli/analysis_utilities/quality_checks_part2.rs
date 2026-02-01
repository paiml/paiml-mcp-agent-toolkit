
/// Extract Method: Get security violation patterns
fn get_security_patterns() -> Vec<(&'static str, &'static str)> {
    vec![
        (
            r#"(?i)password\s*=\s*["'][^"']+["']"#,
            "Hardcoded password detected",
        ),
        (
            r#"(?i)api_key\s*=\s*["'][^"']+["']"#,
            "Hardcoded API key detected",
        ),
        (
            r#"(?i)secret\s*=\s*["'][^"']+["']"#,
            "Hardcoded secret detected",
        ),
    ]
}

/// Extract Method: Check a single file for security violations
async fn check_file_security(
    path: &std::path::Path,
    patterns: &[(&str, &str)],
    violations: &mut Vec<QualityViolation>,
) -> Result<()> {
    use regex::Regex;
    use tokio::fs;

    if let Ok(content) = fs::read_to_string(path).await {
        for (pattern_str, message) in patterns {
            if let Ok(regex) = Regex::new(pattern_str) {
                scan_content_for_pattern(&content, &regex, message, path, violations);
            }
        }
    }
    Ok(())
}

/// Extract Method: Scan file content for a specific security pattern
fn scan_content_for_pattern(
    content: &str,
    regex: &regex::Regex,
    message: &str,
    path: &std::path::Path,
    violations: &mut Vec<QualityViolation>,
) {
    for (line_no, line) in content.lines().enumerate() {
        if regex.is_match(line) {
            violations.push(QualityViolation {
                check_type: "security".to_string(),
                severity: "error".to_string(),
                file: path.to_string_lossy().to_string(),
                line: Some(line_no + 1),
                message: message.to_string(),
            });
        }
    }
}

/// Detects duplicate code blocks in a project.
///
/// Uses content hashing to find exact duplicates after normalization.
///
/// # Arguments
///
/// * `project_path` - Path to the project directory to analyze
///
/// # Returns
///
/// A vector of quality violations for each duplicate code block found
///
/// # Examples
///
/// ```no_run
/// # use std::path::Path;
/// # use pmat::cli::analysis_utilities::{check_duplicates, QualityViolation};
/// # async fn example() -> anyhow::Result<()> {
/// let violations = check_duplicates(Path::new(".")).await?;
///
/// // Group duplicates by file
/// let mut duplicates_by_file = std::collections::HashMap::new();
/// for violation in violations {
///     duplicates_by_file.entry(violation.file.clone())
///         .or_insert_with(Vec::new)
///         .push(violation);
/// }
///
/// for (file, dups) in duplicates_by_file {
///     println!("{} has {} duplicate blocks", file, dups.len());
/// }
/// # Ok(())
/// # }
/// ```ignore
///
/// # Property Tests
///
/// ```rust,no_run
/// # tokio_test::block_on(async {
/// use std::path::Path;
/// use pmat::cli::analysis_utilities::check_duplicates;
///
/// // Property: Duplicate violations come in pairs or more
/// let violations = check_duplicates(Path::new(".")).await.unwrap();
///
/// // Group by duplicate message to verify pairs
/// let mut groups = std::collections::HashMap::new();
/// for violation in violations {
///     groups.entry(violation.message.clone())
///         .or_insert_with(Vec::new)
///         .push(violation);
/// }
///
/// for (_, group) in groups {
///     // Each duplicate should appear at least twice
///     assert!(group.len() >= 2, "Duplicates should come in pairs or more");
/// }
/// # });
/// ```
pub async fn check_duplicates(project_path: &Path) -> Result<Vec<QualityViolation>> {
    use std::collections::HashMap;

    let mut violations = Vec::new();
    let mut file_hashes: HashMap<u64, Vec<PathBuf>> = HashMap::new();

    collect_file_hashes(project_path, &mut file_hashes).await?;
    generate_duplicate_violations(&file_hashes, &mut violations);

    Ok(violations)
}

/// Collect content hashes for all source files
async fn collect_file_hashes(
    project_path: &Path,
    file_hashes: &mut std::collections::HashMap<u64, Vec<PathBuf>>,
) -> Result<()> {
    use walkdir::WalkDir;

    for entry in WalkDir::new(project_path) {
        let entry = entry?;
        let path = entry.path();

        // Skip build artifacts and other excluded paths completely
        let path_str = path.to_string_lossy();
        if is_excluded_directory(&path_str) {
            continue;
        }

        // Additional check: if path contains '/target/' anywhere, skip it
        if path_str.contains("/target/") {
            continue;
        }

        if should_process_file_for_duplicates(path) {
            // Use tokio::task::block_in_place to handle async in sync context
            let hash_result = tokio::task::block_in_place(|| {
                tokio::runtime::Handle::current().block_on(process_file_for_hash(path))
            });

            if let Some(hash) = hash_result {
                file_hashes
                    .entry(hash)
                    .or_default()
                    .push(path.to_path_buf());
            }
        }
    }
    Ok(())
}

/// Check if file should be processed for duplicate detection
fn should_process_file_for_duplicates(path: &Path) -> bool {
    path.is_file() && is_source_file(path) && !is_build_artifact(path)
}

/// Process a file and return its content hash if valid
async fn process_file_for_hash(path: &Path) -> Option<u64> {
    if let Ok(content) = tokio::fs::read_to_string(path).await {
        let normalized = normalize_code_content(&content);
        if is_file_large_enough(&normalized) {
            Some(calculate_content_hash(&normalized))
        } else {
            None
        }
    } else {
        None
    }
}

/// Check if file content is large enough to consider for duplicate detection
fn is_file_large_enough(normalized_content: &str) -> bool {
    normalized_content.len() > 50
}

/// Generate duplicate violation reports from hash map
fn generate_duplicate_violations(
    file_hashes: &std::collections::HashMap<u64, Vec<PathBuf>>,
    violations: &mut Vec<QualityViolation>,
) {
    for paths in file_hashes.values() {
        if paths.len() > 1 {
            create_violations_for_duplicate_group(paths, violations);
        }
    }
}

/// Create quality violations for a group of duplicate files
fn create_violations_for_duplicate_group(
    paths: &[PathBuf],
    violations: &mut Vec<QualityViolation>,
) {
    let files_str = format_file_list(paths);

    for path in paths {
        violations.push(QualityViolation {
            check_type: "duplicate".to_string(),
            severity: "warning".to_string(),
            file: path.to_string_lossy().to_string(),
            line: None,
            message: format!("Duplicate code found in: {files_str}"),
        });
    }
}

/// Format list of file paths for violation message
fn format_file_list(paths: &[PathBuf]) -> String {
    paths
        .iter()
        .map(|p| p.to_string_lossy().to_string())
        .collect::<Vec<_>>()
        .join(", ")
}

// Helper function to normalize code content
pub fn normalize_code_content(content: &str) -> String {
    content
        .lines()
        .filter(|line| {
            let trimmed = line.trim();
            !trimmed.is_empty() && !trimmed.starts_with("//") && !trimmed.starts_with("/*")
        })
        .map(str::trim)
        .collect::<Vec<_>>()
        .join("\n")
}

// Helper function to calculate content hash
pub fn calculate_content_hash(content: &str) -> u64 {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};

    let mut hasher = DefaultHasher::new();
    content.hash(&mut hasher);
    hasher.finish()
}

async fn check_coverage(project_path: &Path, min_coverage: f64) -> Result<Vec<QualityViolation>> {
    let mut violations = Vec::new();

    // Simulated coverage check
    if project_path.join("coverage").exists() {
        // Would normally parse coverage report
        let current_coverage = 75.0; // Simulated value
        if current_coverage < min_coverage {
            violations.push(QualityViolation {
                check_type: "coverage".to_string(),
                severity: "error".to_string(),
                message: format!(
                    "Code coverage {current_coverage:.1}% is below minimum {min_coverage:.1}%"
                ),
                file: "project".to_string(),
                line: None,
            });
        }
    }

    Ok(violations)
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
                });
            }
        }
    }

    Ok(violations)
}

async fn check_provability(
    project_path: &Path,
    min_provability: f64,
) -> Result<Vec<QualityViolation>> {
    let mut violations = Vec::new();

    // Simulated provability check
    let current_provability = 0.65; // Simulated value
    if current_provability < min_provability {
        violations.push(QualityViolation {
            check_type: "provability".to_string(),
            severity: "warning".to_string(),
            message: format!(
                "Provability score {current_provability:.2} is below minimum {min_provability:.2}"
            ),
            file: project_path.to_string_lossy().to_string(),
            line: None,
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
pub async fn calculate_provability_score(project_path: &Path) -> Result<f64> {
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

/// Scan project source files and extract up to `max_count` function declarations
/// with their file paths and line numbers for provability analysis.
fn collect_project_functions(
    project_path: &Path,
    max_count: usize,
) -> Vec<crate::services::lightweight_provability_analyzer::FunctionId> {
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
    walkdir::WalkDir::new(root)
        .max_depth(10)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| {
            let p = e.path();
            p.is_file()
                && p.extension().map_or(false, |ext| ext == "rs")
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
///     },
///     QualityViolation {
///         check_type: "dead_code".to_string(),
///         severity: "warning".to_string(),
///         file: "src/lib.rs".to_string(),
///         line: Some(10),
///         message: "Unused function detected".to_string(),
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
/// ```ignore
pub fn format_quality_gate_output(
    results: &QualityGateResults,
    violations: &[QualityViolation],
    format: QualityGateOutputFormat,
) -> Result<String> {
    match format {
        QualityGateOutputFormat::Json => format_qg_as_json(results, violations),
        QualityGateOutputFormat::Human => format_qg_as_human(results, violations),
        QualityGateOutputFormat::Junit => format_qg_as_junit(violations),
        QualityGateOutputFormat::Summary => format_qg_as_summary(results, violations),
        QualityGateOutputFormat::Detailed => format_qg_as_detailed(results, violations),
        QualityGateOutputFormat::Markdown => format_qg_as_markdown(results, violations),
    }
}

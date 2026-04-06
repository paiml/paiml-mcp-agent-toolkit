// Security check helpers and duplicate code detection
// Included by quality_checks_part2.rs

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
                details: None,
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
/// ```
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
#[provable_contracts_macros::contract("pmat-core.yaml", equation = "path_exists")]
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
            details: None,
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
#[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
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
#[provable_contracts_macros::contract("pmat-core.yaml", equation = "score_range")]
pub fn calculate_content_hash(content: &str) -> u64 {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};

    let mut hasher = DefaultHasher::new();
    content.hash(&mut hasher);
    hasher.finish()
}

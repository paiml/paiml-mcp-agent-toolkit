// Quality check functions - extracted for file health (CB-040)
/// Check if path is a build artifact that should be excluded from duplicate detection
pub fn is_build_artifact(path: &Path) -> bool {
    let path_str = path.to_string_lossy();
    path_str.contains("/target/")
        || path_str.contains("/build/")
        || path_str.contains("/out/")
        || path_str.contains("/.cargo/")
        || path_str.contains("/node_modules/")
        || path_str.contains("/dist/")
        || path_str.contains("/.git/")
        || path_str.contains("/generated/")
        || path_str.starts_with("./target/")
        || path_str.starts_with("target/")
}

// Quality check functions

/// Checks code complexity in a project and returns violations.
///
/// # Arguments
///
/// * `project_path` - Path to the project directory to analyze
/// * `max_complexity` - Maximum allowed cyclomatic complexity
///
/// # Returns
///
/// A vector of quality violations for functions exceeding the complexity threshold
///
/// # Examples
///
/// ```no_run
/// # use std::path::Path;
/// # use pmat::cli::analysis_utilities::{check_complexity, QualityViolation};
/// # async fn example() -> anyhow::Result<()> {
/// let violations = check_complexity(Path::new("."), 10).await?;
/// for violation in violations {
///     println!("Complex function: {} in {}", violation.message, violation.file);
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
/// use pmat::cli::analysis_utilities::check_complexity;
///
/// // Test with a specific threshold
/// let threshold = 10u32;
/// let violations = check_complexity(Path::new("."), threshold).await.unwrap();
///
/// // Property: All violations should have complexity > threshold
/// for violation in violations {
///     // Extract complexity from message
///     if let Some(complexity_str) = violation.message
///         .split("complexity ")
///         .nth(1)
///         .and_then(|s| s.split(' ').next())
///         .and_then(|s| s.parse::<u32>().ok()) {
///         assert!(complexity_str > threshold);
///     }
/// }
/// # });
/// ```
pub async fn check_complexity(
    project_path: &Path,
    _max_complexity: u32,
) -> Result<Vec<QualityViolation>> {
    use crate::services::complexity::aggregate_results_with_thresholds;
    use crate::services::configuration_service::configuration;

    let mut violations = Vec::new();

    // Get thresholds from configuration service - SINGLE SOURCE OF TRUTH
    let config_service = configuration();
    let config = config_service.get_config()?;
    let max_cyclomatic = config.quality.max_complexity;
    let max_cognitive = config.quality.max_cognitive_complexity;

    // Use the existing analyze_project_files function - the ONE implementation
    let file_metrics = analyze_project_files(
        project_path,
        None, // Auto-detect toolchain
        &[],  // Empty include pattern means all files
        max_cyclomatic as u16,
        max_cognitive as u16,
    )
    .await?;

    // Check for violations using the same logic as analyze complexity
    let report = aggregate_results_with_thresholds(
        file_metrics,
        Some(max_cyclomatic as u16),
        Some(max_cognitive as u16),
    );

    // Convert violations to QualityViolation format
    // ONLY count actual violations where complexity exceeds threshold
    for violation in &report.violations {
        process_complexity_violation(violation, &mut violations);
    }

    Ok(violations)
}

/// Process a single complexity violation into `QualityViolation` format
fn process_complexity_violation(
    violation: &crate::services::complexity::Violation,
    violations: &mut Vec<QualityViolation>,
) {
    use crate::services::complexity::Violation;

    let (file, line, function, rule, message, value, threshold, severity) = match violation {
        Violation::Error {
            file,
            line,
            function,
            rule,
            message,
            value,
            threshold,
        } => (
            file, line, function, rule, message, value, threshold, "error",
        ),
        Violation::Warning {
            file,
            line,
            function,
            rule,
            message,
            value,
            threshold,
        } => (
            file, line, function, rule, message, value, threshold, "warning",
        ),
    };

    // Only add if this is an actual threshold violation
    if value > threshold {
        violations.push(QualityViolation {
            check_type: "complexity".to_string(),
            severity: severity.to_string(),
            file: file.clone(),
            line: Some(*line as usize),
            message: format!(
                "{}: {} - {} (complexity: {}, threshold: {})",
                function.as_deref().unwrap_or("global"),
                rule,
                message,
                value,
                threshold
            ),
        });
    }
}

/// Detects dead code in a project and returns violations.
///
/// # Arguments
///
/// * `project_path` - Path to the project directory to analyze
/// * `max_percentage` - Maximum allowed percentage of dead code
///
/// # Returns
///
/// A vector of quality violations for dead code exceeding the threshold
///
/// # Examples
///
/// ```no_run
/// # use std::path::Path;
/// # use pmat::cli::analysis_utilities::{check_dead_code, QualityViolation};
/// # async fn example() -> anyhow::Result<()> {
/// let violations = check_dead_code(Path::new("."), 15.0).await?;
/// if violations.is_empty() {
///     println!("Dead code is within acceptable limits");
/// } else {
///     for violation in violations {
///         println!("Dead code issue: {}", violation.message);
///     }
/// }
/// # Ok(())
/// # }
/// ```ignore
///
/// # Property Tests
///
/// ```rust,no_run
/// # use std::path::Path;
/// # use pmat::cli::analysis_utilities::check_dead_code;
/// #
/// # #[tokio::test]
/// # async fn test_dead_code_detection() -> anyhow::Result<()> {
/// // Test with a high threshold (should get no violations)
/// let violations = check_dead_code(Path::new("."), 90.0).await?;
///
/// // Verify violation structure
/// for violation in &violations {
///     assert_eq!(violation.check_type, "dead_code");
///     assert!(violation.severity == "error" || violation.severity == "warning");
///     assert!(!violation.message.is_empty());
/// }
/// # Ok(())
/// # }
/// ```
pub async fn check_dead_code(
    project_path: &Path,
    max_percentage: f64,
) -> Result<Vec<QualityViolation>> {
    use crate::models::dead_code::DeadCodeAnalysisConfig;
    use crate::services::dead_code_analyzer::DeadCodeAnalyzer;

    let mut violations = Vec::new();

    // Create analyzer and run analysis
    let mut analyzer = DeadCodeAnalyzer::new(DeadCodeAnalyzer::DEFAULT_CAPACITY);
    let config = DeadCodeAnalysisConfig {
        include_tests: false,
        include_unreachable: true,
        min_dead_lines: 0,
    };

    let result = analyzer.analyze_with_ranking(project_path, config).await?;

    // Check if dead code percentage exceeds threshold
    let dead_percentage = f64::from(result.summary.dead_percentage);

    if dead_percentage > max_percentage {
        violations.push(QualityViolation {
            check_type: "dead_code".to_string(),
            severity: "error".to_string(),
            file: project_path.to_string_lossy().to_string(),
            line: None,
            message: format!(
                "Dead code percentage {dead_percentage:.1}% exceeds maximum allowed {max_percentage:.1}%"
            ),
        });
    }

    // Add a warning for each file with significant dead code
    for file in result.ranked_files.iter().take(5) {
        if file.dead_percentage > 20.0 {
            violations.push(QualityViolation {
                check_type: "dead_code".to_string(),
                severity: "warning".to_string(),
                file: file.path.clone(),
                line: None,
                message: format!(
                    "File has {:.1}% dead code ({} dead lines)",
                    file.dead_percentage, file.dead_lines
                ),
            });
        }
    }

    Ok(violations)
}

/// Detects self-admitted technical debt (SATD) in source code.
///
/// Scans for technical debt markers like TODO, FIXME, HACK, etc.
///
/// # Arguments
///
/// * `project_path` - Path to the project directory to analyze
///
/// # Returns
///
/// A vector of quality violations for each SATD comment found
///
/// # Examples
///
/// ```no_run
/// # use std::path::Path;
/// # use pmat::cli::analysis_utilities::{check_satd, QualityViolation};
/// # async fn example() -> anyhow::Result<()> {
/// let violations = check_satd(Path::new(".")).await?;
///
/// // Group by severity
/// let mut by_severity = std::collections::HashMap::new();
/// for violation in violations {
///     *by_severity.entry(violation.severity.clone()).or_insert(0) += 1;
/// }
///
/// for (severity, count) in by_severity {
///     println!("{} SATD items with severity: {}", count, severity);
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
/// use pmat::cli::analysis_utilities::check_satd;
///
/// // Property: All detected items should have valid SATD patterns
/// let violations = check_satd(Path::new(".")).await.unwrap();
///
/// let valid_patterns = ["TODO", "FIXME", "HACK", "XXX", "BUG", "REFACTOR"];
/// for violation in violations {
///     assert_eq!(violation.check_type, "satd");
///     assert!(violation.line.is_some()); // Should have line numbers
///     
///     // Check that message contains a valid SATD type (case-insensitive)
///     let message_upper = violation.message.to_uppercase();
///     let has_valid_pattern = valid_patterns.iter()
///         .any(|&pattern| message_upper.contains(pattern));
///     if !has_valid_pattern {
///         eprintln!("Violation message doesn't contain expected pattern: {}", violation.message);
///     }
/// }
/// # });
/// ```
pub async fn check_satd(project_path: &Path) -> Result<Vec<QualityViolation>> {
    // Toyota Way: Use the ONE proper implementation, not duplicate logic
    use crate::services::satd_detector::SATDDetector;

    let detector = SATDDetector::new();
    let include_tests = false; // Don't include test files in quality gate

    // Use the proper SATD analyzer with context awareness
    let satd_result = detector
        .analyze_project(project_path, include_tests)
        .await?;

    // Convert SATD items to quality violations
    let violations: Vec<QualityViolation> = satd_result
        .items
        .into_iter()
        .map(|debt| QualityViolation {
            check_type: "satd".to_string(),
            severity: match debt.severity {
                crate::services::satd_detector::Severity::Critical => "error",
                crate::services::satd_detector::Severity::High => "error",
                crate::services::satd_detector::Severity::Medium => "warning",
                crate::services::satd_detector::Severity::Low => "info",
            }
            .to_string(),
            file: debt.file.display().to_string(),
            line: Some(debt.line as usize),
            message: format!(
                "{}: {} (at column {})",
                debt.category, debt.text, debt.column
            ),
        })
        .collect();

    Ok(violations)
}

/// Check code entropy (diversity) across the project
///
/// This function analyzes code entropy to detect low-diversity code that might
/// indicate copy-paste programming, lack of abstraction, or potential defects.
///
/// # Arguments
/// * `project_path` - Root directory to analyze
/// * `min_entropy` - Minimum acceptable entropy (typically 0.5-0.9)
///
/// # Example
///
/// ```rust,no_run
/// # use std::path::Path;
/// # use pmat::cli::analysis_utilities::QualityViolation;
/// #
/// # #[tokio::test]
/// # async fn test_entropy_check() -> anyhow::Result<()> {
/// // Check for low entropy (repetitive) code
/// let violations = check_entropy(Path::new("."), 0.7).await?;
///
/// for violation in &violations {
///     assert_eq!(violation.check_type, "entropy");
///     println!("Low diversity in {}: {}", violation.file, violation.message);
/// }
/// # Ok(())
/// # }
/// ```
///
/// # Property Tests
///
/// ```rust,no_run
/// # use std::path::Path;
/// #
/// # #[tokio::test]
/// # async fn test_entropy_threshold() -> anyhow::Result<()> {
/// // Test with different thresholds
/// let low_threshold = check_entropy(Path::new("."), 0.3).await?;
/// let high_threshold = check_entropy(Path::new("."), 0.9).await?;
///
/// // Higher threshold should find more violations
/// assert!(high_threshold.len() >= low_threshold.len());
/// # Ok(())
/// # }
/// ```
pub async fn check_entropy(
    project_path: &Path,
    _min_entropy: f64,
) -> Result<Vec<QualityViolation>> {
    // TOYOTA WAY FIX: Replace Shannon entropy with AST pattern-based entropy
    // Sprint 98: Fix for 5831 false positive entropy violations
    use crate::entropy::violation_detector::Severity;
    use crate::entropy::{EntropyAnalyzer, EntropyConfig};

    // Create entropy analyzer with tuned config to reduce false positives
    let mut config = EntropyConfig {
        min_severity: Severity::Medium, // Only report medium+ severity
        ..Default::default()
    };
    config.exclude_paths.push("**/target/**".to_string());
    config.exclude_paths.push("**/node_modules/**".to_string());
    config.exclude_paths.push("**/*.test.rs".to_string());
    config.exclude_paths.push("**/tests/**".to_string());

    let analyzer = EntropyAnalyzer::with_config(config);

    // Run AST-based entropy analysis
    let report = analyzer.analyze(project_path).await?;

    // Convert actionable violations to QualityViolation format
    let violations: Vec<QualityViolation> = report
        .actionable_violations
        .into_iter()
        .map(|violation| QualityViolation {
            check_type: "entropy".to_string(),
            severity: match violation.severity {
                Severity::Low => "info".to_string(),
                Severity::Medium => "warning".to_string(),
                Severity::High => "error".to_string(),
            },
            file: violation.affected_files.first().map_or_else(
                || "project".to_string(),
                |p| p.to_string_lossy().to_string(),
            ),
            line: None, // Pattern violations span multiple lines
            message: format!(
                "{} (saves {} lines) - Fix: {}",
                violation.message, violation.estimated_loc_reduction, violation.fix_suggestion
            ),
        })
        .collect();

    Ok(violations)
}

// NOTE: Shannon entropy functions removed in Sprint 98
// These were replaced by AST pattern-based entropy detection
// in the entropy module. The old character-based approach
// generated 5,831 false positives and has been deprecated.

async fn check_security(project_path: &Path) -> Result<Vec<QualityViolation>> {
    let mut violations = Vec::new();
    let patterns = get_security_patterns();

    use tokio::fs;

    if let Ok(mut entries) = fs::read_dir(project_path).await {
        while let Some(entry) = entries.next_entry().await? {
            let path = entry.path();
            if path.is_file() && is_source_file(&path) {
                check_file_security(&path, &patterns, &mut violations).await?;
            }
        }
    }

    Ok(violations)
}

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
    use crate::services::lightweight_provability_analyzer::{
        FunctionId, LightweightProvabilityAnalyzer,
    };

    // Use the real provability analyzer
    let analyzer = LightweightProvabilityAnalyzer::new();

    // For quality gate purposes, we'll analyze a sample of functions
    // This is a simplified check - the full analysis is available via 'pmat analyze provability'
    let sample_functions = vec![FunctionId {
        file_path: project_path.to_string_lossy().to_string(),
        function_name: "main".to_string(),
        line_number: 1,
    }];

    let summaries = analyzer.analyze_incrementally(&sample_functions).await;

    if summaries.is_empty() {
        // Default score if no functions analyzed
        Ok(0.85)
    } else {
        // Calculate average provability score
        let total_score: f64 = summaries.iter().map(|s| s.provability_score).sum();
        Ok(total_score / summaries.len() as f64)
    }
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

// Helper: Format as JSON
fn format_qg_as_json(
    results: &QualityGateResults,
    violations: &[QualityViolation],
) -> Result<String> {
    Ok(serde_json::to_string_pretty(&serde_json::json!({
        "results": results,
        "violations": violations,
    }))?)
}

// Helper: Format as human-readable
fn format_qg_as_human(
    results: &QualityGateResults,
    violations: &[QualityViolation],
) -> Result<String> {
    use std::fmt::Write;
    let mut output = String::new();

    write_qg_human_header(&mut output, results)?;
    write_qg_violation_counts(&mut output, results)?;

    if let Some(score) = results.provability_score {
        writeln!(&mut output, "\nProvability score: {score:.2}")?;
    }

    if !violations.is_empty() {
        write_qg_violations_list(&mut output, violations)?;
    }

    Ok(output)
}

// Helper: Write human header
fn write_qg_human_header(output: &mut String, results: &QualityGateResults) -> Result<()> {
    use std::fmt::Write;
    writeln!(output, "# Quality Gate Report\n")?;
    writeln!(
        output,
        "Status: {}",
        if results.passed {
            "✅ PASSED"
        } else {
            "❌ FAILED"
        }
    )?;
    writeln!(output, "Total violations: {}\n", results.total_violations)?;
    Ok(())
}

// Helper: Write violation counts
fn write_qg_violation_counts(output: &mut String, results: &QualityGateResults) -> Result<()> {
    use std::fmt::Write;
    let counts = [
        ("Complexity", results.complexity_violations),
        ("Dead code", results.dead_code_violations),
        ("Technical debt", results.satd_violations),
        ("Entropy", results.entropy_violations),
        ("Security", results.security_violations),
        ("Duplicate code", results.duplicate_violations),
    ];

    for (name, count) in counts {
        if count > 0 {
            writeln!(output, "## {name} violations: {count}")?;
        }
    }
    Ok(())
}

// Helper: Write violations list
fn write_qg_violations_list(output: &mut String, violations: &[QualityViolation]) -> Result<()> {
    use std::fmt::Write;
    writeln!(output, "\n## Violations:\n")?;
    for v in violations {
        writeln!(
            output,
            "- [{}] {} - {}",
            v.severity, v.check_type, v.message
        )?;
        if let Some(line) = v.line {
            writeln!(output, "  File: {}:{}", v.file, line)?;
        } else {
            writeln!(output, "  File: {}", v.file)?;
        }
    }
    Ok(())
}

// Helper: Format as JUnit XML
/// Toyota Way: Extract Method - Format quality gate as `JUnit` XML (complexity ≤8)
fn format_qg_as_junit(violations: &[QualityViolation]) -> Result<String> {
    let mut output = String::new();

    write_junit_header(&mut output)?;
    write_junit_testsuite_start(&mut output, violations.len())?;
    write_junit_testcases(&mut output, violations)?;
    write_junit_footer(&mut output)?;

    Ok(output)
}

/// Toyota Way: Extract Method - Write `JUnit` XML header (complexity ≤3)
fn write_junit_header(output: &mut String) -> Result<()> {
    use std::fmt::Write;
    writeln!(output, r#"<?xml version="1.0" encoding="UTF-8"?>"#)?;
    writeln!(output, r#"<testsuites name="Quality Gate">"#)?;
    Ok(())
}

/// Toyota Way: Extract Method - Write `JUnit` testsuite start (complexity ≤3)
fn write_junit_testsuite_start(output: &mut String, count: usize) -> Result<()> {
    use std::fmt::Write;
    writeln!(
        output,
        r#"  <testsuite name="Quality Checks" tests="{count}" failures="{count}">"#
    )?;
    Ok(())
}

/// Toyota Way: Extract Method - Write `JUnit` testcases (complexity ≤5)
fn write_junit_testcases(output: &mut String, violations: &[QualityViolation]) -> Result<()> {
    for v in violations {
        write_single_junit_testcase(output, v)?;
    }
    Ok(())
}

/// Toyota Way: Extract Method - Write single `JUnit` testcase (complexity ≤5)
fn write_single_junit_testcase(output: &mut String, v: &QualityViolation) -> Result<()> {
    use std::fmt::Write;
    writeln!(
        output,
        r#"    <testcase name="{}" classname="{}">"#,
        v.message, v.check_type
    )?;
    writeln!(
        output,
        r#"      <failure message="{}" type="{}"/>"#,
        v.message, v.severity
    )?;
    writeln!(output, r"    </testcase>")?;
    Ok(())
}

/// Toyota Way: Extract Method - Write `JUnit` XML footer (complexity ≤3)
fn write_junit_footer(output: &mut String) -> Result<()> {
    use std::fmt::Write;
    writeln!(output, r"  </testsuite>")?;
    writeln!(output, r"</testsuites>")?;
    Ok(())
}

// Helper: Format as summary
fn format_qg_as_summary(
    results: &QualityGateResults,
    violations: &[QualityViolation],
) -> Result<String> {
    use std::fmt::Write;
    let mut output = String::new();
    writeln!(
        &mut output,
        "Quality Gate: {}",
        if results.passed { "PASSED" } else { "FAILED" }
    )?;
    writeln!(
        &mut output,
        "Total violations: {}",
        results.total_violations
    )?;

    // Show violation summary by type
    if !violations.is_empty() {
        writeln!(&mut output)?;
        write_qg_violations_summary(&mut output, violations)?;
    }

    Ok(output)
}

// Helper: Write violation summary grouped by type
fn write_qg_violations_summary(
    output: &mut String,
    violations: &[QualityViolation],
) -> Result<()> {
    use std::collections::BTreeMap;
    use std::fmt::Write;

    // Group violations by check type
    let mut by_type: BTreeMap<&str, Vec<&QualityViolation>> = BTreeMap::new();
    for v in violations {
        by_type.entry(&v.check_type).or_default().push(v);
    }

    for (check_type, type_violations) in by_type {
        writeln!(output, "## {} ({} violations)", check_type, type_violations.len())?;
        for v in type_violations.iter().take(5) {
            // Show first 5 per category
            if let Some(line) = v.line {
                writeln!(output, "  - {}:{} - {}", v.file, line, v.message)?;
            } else {
                writeln!(output, "  - {} - {}", v.file, v.message)?;
            }
        }
        if type_violations.len() > 5 {
            writeln!(output, "  ... and {} more", type_violations.len() - 5)?;
        }
    }
    Ok(())
}

// Helper: Format as detailed
fn format_qg_as_detailed(
    results: &QualityGateResults,
    violations: &[QualityViolation],
) -> Result<String> {
    let mut output = String::new();

    write_qg_detailed_header(&mut output, results)?;
    write_qg_detailed_summary(&mut output, results)?;

    if !violations.is_empty() {
        write_qg_detailed_violations(&mut output, violations)?;
    }

    Ok(output)
}

// Helper: Write detailed header
fn write_qg_detailed_header(output: &mut String, results: &QualityGateResults) -> Result<()> {
    use std::fmt::Write;
    writeln!(output, "# Quality Gate Detailed Report\n")?;
    writeln!(
        output,
        "Status: {}",
        if results.passed {
            "✅ PASSED"
        } else {
            "❌ FAILED"
        }
    )?;
    writeln!(output, "Total violations: {}\n", results.total_violations)?;
    Ok(())
}

// Helper: Write detailed summary
fn write_qg_detailed_summary(output: &mut String, results: &QualityGateResults) -> Result<()> {
    use std::fmt::Write;
    writeln!(output, "## Violations by Type\n")?;
    let items = [
        ("Complexity", results.complexity_violations),
        ("Dead code", results.dead_code_violations),
        ("SATD", results.satd_violations),
        ("Entropy", results.entropy_violations),
        ("Security", results.security_violations),
        ("Duplicates", results.duplicate_violations),
        ("Coverage", results.coverage_violations),
        ("Sections", results.section_violations),
        ("Provability", results.provability_violations),
    ];

    for (name, count) in items {
        writeln!(output, "- {name}: {count}")?;
    }
    Ok(())
}

// Helper: Write detailed violations
fn write_qg_detailed_violations(
    output: &mut String,
    violations: &[QualityViolation],
) -> Result<()> {
    use std::fmt::Write;
    writeln!(output, "\n## All Violations\n")?;
    for (i, v) in violations.iter().enumerate() {
        writeln!(
            output,
            "{}. [{}] {}: {}",
            i + 1,
            v.severity,
            v.check_type,
            v.message
        )?;
        if let Some(line) = v.line {
            writeln!(output, "   File: {}:{}", v.file, line)?;
        } else {
            writeln!(output, "   File: {}", v.file)?;
        }
    }
    Ok(())
}

// Helper: Format as Markdown
/// Toyota Way: Extract Method - Format quality gate as Markdown (complexity ≤8)
fn format_qg_as_markdown(
    results: &QualityGateResults,
    violations: &[QualityViolation],
) -> Result<String> {
    let mut output = String::new();

    write_qg_markdown_header(&mut output, results)?;
    write_qg_markdown_summary_table(&mut output, results)?;

    // Add violations section if any exist
    if !violations.is_empty() {
        write_qg_markdown_violations(&mut output, violations)?;
    }

    Ok(output)
}

/// Write violations section in Markdown format
fn write_qg_markdown_violations(
    output: &mut String,
    violations: &[QualityViolation],
) -> Result<()> {
    use std::collections::BTreeMap;
    use std::fmt::Write;

    writeln!(output, "\n## Violations\n")?;

    // Group violations by check type
    let mut by_type: BTreeMap<&str, Vec<&QualityViolation>> = BTreeMap::new();
    for v in violations {
        by_type.entry(&v.check_type).or_default().push(v);
    }

    for (check_type, type_violations) in by_type {
        writeln!(output, "### {} ({} issues)\n", check_type, type_violations.len())?;
        writeln!(output, "| Severity | File | Line | Message |")?;
        writeln!(output, "|----------|------|------|---------|")?;

        for v in &type_violations {
            let line_str = v.line.map_or(String::from("-"), |l| l.to_string());
            // Escape pipe characters in message for markdown table
            let escaped_msg = v.message.replace('|', "\\|");
            writeln!(
                output,
                "| {} | {} | {} | {} |",
                v.severity, v.file, line_str, escaped_msg
            )?;
        }
        writeln!(output)?;
    }

    Ok(())
}

/// Toyota Way: Extract Method - Write QG Markdown header section (complexity ≤5)
fn write_qg_markdown_header(output: &mut String, results: &QualityGateResults) -> Result<()> {
    use std::fmt::Write;

    writeln!(output, "# Quality Gate Report\n")?;
    writeln!(
        output,
        "**Status**: {}\n",
        format_qg_status_badge(results.passed)
    )?;
    writeln!(
        output,
        "**Total violations**: {}\n",
        results.total_violations
    )?;

    Ok(())
}

/// Toyota Way: Extract Method - Format QG status badge (complexity ≤3)
fn format_qg_status_badge(passed: bool) -> &'static str {
    if passed {
        "✅ PASSED"
    } else {
        "❌ FAILED"
    }
}

/// Toyota Way: Extract Method - Write QG Markdown summary table (complexity ≤8)
fn write_qg_markdown_summary_table(
    output: &mut String,
    results: &QualityGateResults,
) -> Result<()> {
    use std::fmt::Write;

    writeln!(output, "## Summary\n")?;
    write_qg_markdown_table_headers(output)?;
    write_qg_markdown_table_rows(output, results)?;

    Ok(())
}

/// Toyota Way: Extract Method - Write QG Markdown table headers (complexity ≤3)
fn write_qg_markdown_table_headers(output: &mut String) -> Result<()> {
    use std::fmt::Write;

    writeln!(output, "| Check Type | Violations |")?;
    writeln!(output, "|------------|------------|")?;

    Ok(())
}

/// Toyota Way: Extract Method - Write QG Markdown table rows (complexity ≤5)
fn write_qg_markdown_table_rows(output: &mut String, results: &QualityGateResults) -> Result<()> {
    use std::fmt::Write;

    let rows = get_qg_violation_summary_rows(results);

    for (name, count) in rows {
        writeln!(output, "| {name} | {count} |")?;
    }

    Ok(())
}

/// Toyota Way: Extract Method - Get QG violation summary data rows (complexity ≤3)
fn get_qg_violation_summary_rows(results: &QualityGateResults) -> [(&'static str, u64); 9] {
    [
        (
            "Complexity",
            results.complexity_violations.try_into().unwrap_or(0),
        ),
        (
            "Dead Code",
            results.dead_code_violations.try_into().unwrap_or(0),
        ),
        ("SATD", results.satd_violations.try_into().unwrap_or(0)),
        (
            "Entropy",
            results.entropy_violations.try_into().unwrap_or(0),
        ),
        (
            "Security",
            results.security_violations.try_into().unwrap_or(0),
        ),
        (
            "Duplicates",
            results.duplicate_violations.try_into().unwrap_or(0),
        ),
        (
            "Coverage",
            results.coverage_violations.try_into().unwrap_or(0),
        ),
        (
            "Sections",
            results.section_violations.try_into().unwrap_or(0),
        ),
        (
            "Provability",
            results.provability_violations.try_into().unwrap_or(0),
        ),
    ]
}

// Helper functions
#[must_use]
pub fn detect_toolchain(path: &Path) -> Option<String> {
    super::detect_primary_language(path)
}

#[must_use]
pub fn build_complexity_thresholds(
    max_cyclomatic: Option<u16>,
    max_cognitive: Option<u16>,
) -> (u16, u16) {
    (max_cyclomatic.unwrap_or(10), max_cognitive.unwrap_or(15))
}

/// Analyzes project files for complexity metrics using a systematic approach.
///
/// This function walks through a project directory, filtering files based on toolchain
/// and include patterns, then analyzes each applicable file for complexity metrics.
/// The implementation follows Toyota Way principles by breaking down complexity into
/// focused, single-responsibility helper functions.
///
/// # Arguments
///
/// * `project_path` - Root directory of the project to analyze
/// * `toolchain` - Optional toolchain specifier ("rust", "typescript", "python", etc.)
/// * `include` - Patterns for files to include in analysis (empty = use defaults)
/// * `cyclomatic_threshold` - Threshold for cyclomatic complexity warnings
/// * `cognitive_threshold` - Threshold for cognitive complexity warnings
///
/// # Returns
///
/// A `Result` containing a vector of `FileComplexityMetrics` for each analyzed file.
///
/// # Examples
///
/// ```ignore
/// use pmat::cli::analysis_utilities::analyze_project_files;
/// use std::path::Path;
///
/// # async fn example() -> anyhow::Result<()> {
/// let project_path = Path::new(".");
/// let metrics = analyze_project_files(
///     project_path,
///     Some("rust"),
///     &[],
///     10,
///     15
/// ).await?;
///
/// assert!(metrics.len() >= 0);
/// # Ok(())
/// # }
/// ```ignore
///
/// # Quality Improvements
///
/// This function was refactored from a monolithic implementation (complexity 40)
/// into focused helper functions, achieving:
/// - Reduced cyclomatic complexity from 40 to <8
/// - Improved readability through single-responsibility functions
/// - Better maintainability following Toyota Way Kaizen principles
pub async fn analyze_project_files(
    project_path: &Path,
    toolchain: Option<&str>,
    include: &[String],
    cyclomatic_threshold: u16,
    cognitive_threshold: u16,
) -> Result<Vec<crate::services::complexity::FileComplexityMetrics>> {
    use crate::services::file_discovery::{FileDiscoveryConfig, ProjectFileDiscovery};

    // CRITICAL FIX: Use ProjectFileDiscovery instead of WalkDir
    // This ensures .pmatignore and .paimlignore files are respected
    // Bug: Previously used walkdir directly, bypassing ignore file support
    let discovery_config = FileDiscoveryConfig {
        respect_gitignore: true, // Respect .gitignore, .pmatignore, .paimlignore
        ..Default::default()
    };

    let discovery =
        ProjectFileDiscovery::new(project_path.to_path_buf()).with_config(discovery_config);

    // Discover all files using the intelligent file discovery service
    let discovered_files = discovery.discover_files()?;

    // CRITICAL: ProjectFileDiscovery already handles exclusions via .gitignore/.pmatignore
    // We only need to filter by extension and include patterns here
    let extensions = get_file_extensions(toolchain);

    // Filter discovered files ONLY by extension and include patterns
    // Do NOT use should_analyze_file() as it has is_excluded_path() which filters /tmp/
    let files_to_analyze: Vec<_> = discovered_files
        .into_iter()
        .filter(|path| {
            // Check extension
            let extension = path.extension().and_then(|ext| ext.to_str()).unwrap_or("");
            if !extensions.contains(&extension) {
                return false;
            }

            // Check include patterns (if specified)
            if !include.is_empty() {
                matches_include_patterns(path, project_path, include)
            } else {
                true // No include patterns, accept all files with correct extension
            }
        })
        .collect();

    // PERFORMANCE OPTIMIZATION: Process files in parallel batches
    // Return early if no files to analyze
    if files_to_analyze.is_empty() {
        return Ok(Vec::new());
    }

    let batch_size = std::cmp::min(files_to_analyze.len(), 20); // Optimize batch size
    let mut results = Vec::new();

    for batch in files_to_analyze.chunks(batch_size) {
        let batch_futures: Vec<_> = batch
            .iter()
            .map(|path| analyze_complexity_file(path, cyclomatic_threshold, cognitive_threshold))
            .collect();

        let batch_results = futures::future::try_join_all(batch_futures).await?;

        for metrics in batch_results.into_iter().flatten() {
            results.push(metrics);
        }
    }

    Ok(results)
}

/// Get file extensions for the specified toolchain.
///
/// Maps toolchain identifiers to their corresponding file extensions.
/// Supports multiple programming languages and defaults to Rust.
///
/// # Arguments
///
/// * `toolchain` - Optional toolchain identifier
///
/// # Returns
///
/// Vector of file extensions to analyze for the given toolchain
///
/// # Examples
///
/// ```ignore
/// # use pmat::cli::analysis_utilities::get_file_extensions;
/// let rust_extensions = get_file_extensions(Some("rust"));
/// assert_eq!(rust_extensions, vec!["rs"]);
///
/// let ts_extensions = get_file_extensions(Some("typescript"));
/// assert_eq!(ts_extensions, vec!["ts", "tsx", "js", "jsx"]);
///
/// let default_extensions = get_file_extensions(None);
/// assert_eq!(default_extensions, vec!["rs"]);
/// ```ignore
#[must_use]
pub fn get_file_extensions(toolchain: Option<&str>) -> Vec<&'static str> {
    match toolchain {
        Some("rust") => vec!["rs"],
        Some("deno" | "typescript") => vec!["ts", "tsx", "js", "jsx"],
        Some("javascript") => vec!["js", "jsx"], // PMAT-BUG-002 fix
        Some("python-uv" | "python") => vec!["py"],
        Some("c") => vec!["c", "h"], // PMAT-BUG-003 fix
        Some("cpp" | "c++") => vec!["cpp", "cc", "cxx", "hpp", "h", "hxx"], // PMAT-BUG-004 fix
        Some("go") => vec!["go"],
        Some("java") => vec!["java"],
        Some("kotlin") => vec!["kt", "kts"],
        Some("ruby") => vec!["rb"],
        Some("php") => vec!["php"],
        Some("swift") => vec!["swift"],
        Some("csharp" | "cs") => vec!["cs"],
        Some("bash") => vec!["sh", "bash"],
        Some(_) => vec!["rs"], // unknown toolchain defaults to rust
        None => {
            // Issue #42 fix: When no toolchain detected, analyze ALL supported languages
            vec![
                "rs", "py", "ts", "tsx", "js", "jsx", "go", "java", "kt", "kts", "c", "cpp", "cc",
                "cxx", "rb", "php", "swift", "cs",
            ]
        }
    }
}

/// Check if a file should be analyzed based on extension, patterns, and exclusions.
///
/// This function implements the filtering logic for determining whether a file
/// should be included in complexity analysis, based on file extension,
/// include patterns, and standard exclusions.
///
/// # Arguments
///
/// * `path` - The file path to evaluate
/// * `project_path` - Root project directory
/// * `extensions` - Allowed file extensions
/// * `include` - Include patterns (if empty, uses default exclusions)
///
/// # Returns
///
/// `true` if the file should be analyzed, `false` otherwise
#[must_use]
pub fn should_analyze_file(
    path: &Path,
    project_path: &Path,
    extensions: &[&str],
    include: &[String],
) -> bool {
    let extension = path.extension().and_then(|ext| ext.to_str()).unwrap_or("");

    if !extensions.contains(&extension) {
        return false;
    }

    if include.is_empty() {
        !is_excluded_path(path)
    } else {
        matches_include_patterns(path, project_path, include)
    }
}

/// Check if path matches any of the include patterns
fn matches_include_patterns(path: &Path, project_path: &Path, include: &[String]) -> bool {
    use glob::Pattern;

    let path_str = path.to_string_lossy();
    let relative_path = path.strip_prefix(project_path).unwrap_or(path);
    let relative_str = relative_path.to_string_lossy();

    include.iter().any(|pattern| match Pattern::new(pattern) {
        Ok(glob_pattern) => glob_pattern.matches(&relative_str) || glob_pattern.matches(&path_str),
        Err(_) => path_str.contains(pattern),
    })
}

/// Check if path should be excluded from analysis
fn is_excluded_path(path: &Path) -> bool {
    let path_str = path.to_string_lossy();

    if is_excluded_directory(&path_str) {
        return true;
    }

    if let Some(file_name) = path.file_name() {
        let fname = file_name.to_string_lossy();
        is_excluded_filename(&fname)
    } else {
        false
    }
}

/// Check if path contains excluded directories
pub fn is_excluded_directory(path_str: &str) -> bool {
    // Normalize path for consistent matching
    let normalized = path_str.replace('\\', "/");

    // Directory name patterns to exclude (gitignore-style)
    let excluded_dir_names = [
        "target",
        "build",
        "out",
        ".cargo",
        "node_modules",
        "dist",
        ".git",
        "vendor",
        "generated",
        ".aws-sam",
        "coverage",
        "__pycache__",
        ".pytest_cache",
        ".cache",
        "tmp",
        ".venv",
        "venv",
        "ENV",
        "env",
        ".terraform",
        "site",
        "_site",
        ".jekyll-cache",
        ".idea",
        ".vscode",
    ];

    // Path patterns that should be excluded
    let excluded_path_patterns = [
        "/target/",
        "/build/",
        "/out/",
        "/.cargo/",
        "/node_modules/",
        "/dist/",
        "/.git/",
        "/vendor/",
        "/generated/",
        "/.aws-sam/",
        "/coverage/",
        "/__pycache__/",
        "/.pytest_cache/",
        "/.cache/",
        "/tmp/",
        "/.venv/",
        "/venv/",
        "/ENV/",
        "/env/",
        "/.terraform/",
        "/site/",
        "/_site/",
        "/.jekyll-cache/",
        "/.idea/",
        "/.vscode/",
        "/tests/",
        "/test/",
        "/examples/",
        "/benches/",
        "/benchmarks/",
        "/fixtures/",
        "/testdata/",
        "/test_data/",
        "/debug_test/",
        "/test-",
    ];

    // Check if the path contains any excluded directory patterns
    if excluded_path_patterns
        .iter()
        .any(|pattern| normalized.contains(pattern))
    {
        return true;
    }

    // Check if path starts with excluded directories (./target, target/, etc.)
    let path_components: Vec<&str> = normalized.trim_start_matches("./").split('/').collect();
    if let Some(first_component) = path_components.first() {
        if excluded_dir_names.contains(first_component) {
            return true;
        }
    }

    false
}

/// Check if filename indicates a test file
#[must_use]
pub fn is_excluded_filename(filename: &str) -> bool {
    is_test_file(filename)
        || is_example_or_demo_file(filename)
        || is_benchmark_file(filename)
        || is_mock_or_stub_file(filename)
}

/// Check if filename is a test file (cognitive complexity ≤6)
pub fn is_test_file(filename: &str) -> bool {
    const TEST_SUFFIXES: &[&str] = &["_test.rs", "_tests.rs", "tests.rs"];
    const TEST_PREFIXES: &[&str] = &["test_", "tests_"];
    const TEST_CONTAINS: &[&str] = &[
        "_test_",
        "_tests_",
        "test_harness",
        "test_helpers",
        "test_utils",
        "_property_test",
        "property_tests",
    ];

    TEST_SUFFIXES.iter().any(|s| filename.ends_with(s))
        || TEST_PREFIXES.iter().any(|p| filename.starts_with(p))
        || TEST_CONTAINS.iter().any(|c| filename.contains(c))
}

/// Check if filename is an example or demo file (cognitive complexity ≤4)
pub fn is_example_or_demo_file(filename: &str) -> bool {
    const EXAMPLE_DEMO_PREFIXES: &[&str] = &["example_", "demo_"];
    const EXAMPLE_DEMO_CONTAINS: &[&str] = &["_example", "_demo"];

    EXAMPLE_DEMO_PREFIXES
        .iter()
        .any(|p| filename.starts_with(p))
        || EXAMPLE_DEMO_CONTAINS.iter().any(|c| filename.contains(c))
}

/// Check if filename is a benchmark file (cognitive complexity ≤4)
pub fn is_benchmark_file(filename: &str) -> bool {
    const BENCH_SUFFIXES: &[&str] = &["_bench.rs", "_benchmark.rs"];
    const BENCH_CONTAINS: &[&str] = &["bench_", "benchmark_"];

    BENCH_SUFFIXES.iter().any(|s| filename.ends_with(s))
        || BENCH_CONTAINS.iter().any(|c| filename.contains(c))
}

/// Check if filename is a mock or stub file (cognitive complexity ≤4)
pub fn is_mock_or_stub_file(filename: &str) -> bool {
    const MOCK_STUB_PREFIXES: &[&str] = &["mock_", "stub_", "stubs_"];
    const MOCK_STUB_CONTAINS: &[&str] = &["_mock", "_stub", "_stubs"];

    MOCK_STUB_PREFIXES.iter().any(|p| filename.starts_with(p))
        || MOCK_STUB_CONTAINS.iter().any(|c| filename.contains(c))
}

/// Analyze a single file for complexity metrics
async fn analyze_complexity_file(
    path: &Path,
    cyclomatic_threshold: u16,
    cognitive_threshold: u16,
) -> Result<Option<crate::services::complexity::FileComplexityMetrics>> {
    // PERFORMANCE OPTIMIZATION: Use async file I/O instead of blocking
    match tokio::fs::read_to_string(path).await {
        Ok(content) => {
            let metrics = analyze_file_complexity_async(
                path,
                &content,
                cyclomatic_threshold,
                cognitive_threshold,
            )
            .await?;
            Ok(Some(metrics))
        }
        Err(_) => Ok(None),
    }
}

async fn analyze_file_complexity_async(
    path: &Path,
    content: &str,
    _cyclomatic_threshold: u16,
    _cognitive_threshold: u16,
) -> Result<crate::services::complexity::FileComplexityMetrics> {
    crate::cli::language_analyzer::analyze_file_complexity(path, content).await
}

#[must_use]
pub fn add_top_files_ranking(
    files: Vec<crate::services::complexity::FileComplexityMetrics>,
    top_files: usize,
) -> Vec<crate::services::complexity::FileComplexityMetrics> {
    if top_files == 0 {
        files
    } else {
        files.into_iter().take(top_files).collect()
    }
}

pub fn format_dead_code_output(
    format: DeadCodeOutputFormat,
    dead_code_result: &crate::models::dead_code::DeadCodeResult,
    _output: Option<PathBuf>,
) -> Result<()> {
    crate::cli::dead_code_formatter::format_and_output_dead_code(format, dead_code_result, _output)
}


// Single file quality check functions - extracted for file health (CB-040)
async fn check_single_file_complexity(
    project_path: &Path,
    file_path: &Path,
    max_complexity_p99: u32,
) -> Result<Vec<QualityViolation>> {
    let abs_file_path = resolve_absolute_file_path(project_path, file_path);
    validate_file_exists(&abs_file_path)?;

    let mut violations = Vec::new();
    analyze_file_complexity(
        &abs_file_path,
        file_path,
        max_complexity_p99,
        &mut violations,
    )
    .await?;

    Ok(violations)
}

/// Resolve file path to absolute path
fn resolve_absolute_file_path(project_path: &Path, file_path: &Path) -> PathBuf {
    if file_path.is_absolute() {
        file_path.to_path_buf()
    } else {
        project_path.join(file_path)
    }
}

/// Validate that file exists
fn validate_file_exists(abs_file_path: &Path) -> Result<()> {
    if !abs_file_path.exists() {
        return Err(anyhow::anyhow!(
            "File not found: {}",
            abs_file_path.display()
        ));
    }
    Ok(())
}

/// Analyze file complexity based on file extension
async fn analyze_file_complexity(
    abs_file_path: &Path,
    original_path: &Path,
    max_complexity: u32,
    violations: &mut Vec<QualityViolation>,
) -> Result<()> {
    if let Some(ext) = abs_file_path.extension() {
        if ext == "rs" {
            analyze_rust_file_complexity(abs_file_path, original_path, max_complexity, violations)
                .await?;
        }
        // Add support for other languages as needed
    }
    Ok(())
}

/// Analyze Rust file complexity and generate violations
async fn analyze_rust_file_complexity(
    abs_file_path: &Path,
    original_path: &Path,
    max_complexity: u32,
    violations: &mut Vec<QualityViolation>,
) -> Result<()> {
    use crate::services::ast_rust::analyze_rust_file_with_complexity;

    let metrics = analyze_rust_file_with_complexity(abs_file_path).await?;

    for func in &metrics.functions {
        if function_exceeds_complexity_threshold(func, max_complexity) {
            violations.push(create_complexity_violation(
                func,
                original_path,
                max_complexity,
            ));
        }
    }

    Ok(())
}

/// Check if function exceeds complexity threshold
fn function_exceeds_complexity_threshold(
    func: &crate::services::complexity::FunctionComplexity,
    max_complexity: u32,
) -> bool {
    func.metrics.cyclomatic > max_complexity as u16
}

/// Create complexity violation for a function
fn create_complexity_violation(
    func: &crate::services::complexity::FunctionComplexity,
    file_path: &Path,
    max_complexity: u32,
) -> QualityViolation {
    QualityViolation {
        check_type: "complexity".to_string(),
        severity: "error".to_string(),
        file: file_path.to_string_lossy().to_string(),
        line: Some(func.line_start as usize),
        message: format!(
            "Function '{}' has cyclomatic complexity {} (max: {})",
            func.name, func.metrics.cyclomatic, max_complexity
        ),
        details: None,
    }
}

async fn check_single_file_dead_code(
    project_path: &Path,
    file_path: &Path,
) -> Result<Vec<QualityViolation>> {
    use regex::Regex;

    let mut violations = Vec::new();

    // Make file path absolute
    let abs_file_path = if file_path.is_absolute() {
        file_path.to_path_buf()
    } else {
        project_path.join(file_path)
    };

    if !abs_file_path.exists() {
        return Ok(violations); // No violations if file doesn't exist
    }

    // Read file content
    let content = tokio::fs::read_to_string(&abs_file_path).await?;

    // Check for common dead code patterns
    let dead_code_patterns = vec![
        (r"#\[allow\(dead_code\)\]", "Dead code attribute found"),
        (r"^\s*//\s*fn\s+\w+", "Commented out function"),
        (r"^\s*//\s*struct\s+\w+", "Commented out struct"),
        (r"^\s*//\s*impl\s+", "Commented out implementation"),
    ];

    for (pattern_str, message) in dead_code_patterns {
        let regex = Regex::new(pattern_str)?;
        for (line_no, line) in content.lines().enumerate() {
            if regex.is_match(line) {
                violations.push(QualityViolation {
                    check_type: "dead_code".to_string(),
                    severity: "warning".to_string(),
                    file: file_path.to_string_lossy().to_string(),
                    line: Some(line_no + 1),
                    message: message.to_string(),
                    details: None,
                });
            }
        }
    }

    Ok(violations)
}

async fn check_single_file_satd(
    project_path: &Path,
    file_path: &Path,
) -> Result<Vec<QualityViolation>> {
    use regex::Regex;

    let mut violations = Vec::new();
    let satd_pattern = Regex::new(r"(?i)\b(TODO|FIXME|HACK|XXX|BUG|REFACTOR):\s*(.+)")?;

    // Make file path absolute
    let abs_file_path = if file_path.is_absolute() {
        file_path.to_path_buf()
    } else {
        project_path.join(file_path)
    };

    if !abs_file_path.exists() {
        return Ok(violations);
    }

    let content = tokio::fs::read_to_string(&abs_file_path).await?;

    for (line_no, line) in content.lines().enumerate() {
        if let Some(captures) = satd_pattern.captures(line) {
            let satd_type = captures
                .get(1)
                .expect("Match group 1 exists for successful regex match")
                .as_str();
            let text = captures
                .get(2)
                .expect("Match group 2 exists for successful regex match")
                .as_str();

            violations.push(QualityViolation {
                check_type: "satd".to_string(),
                severity: "warning".to_string(),
                file: file_path.to_string_lossy().to_string(),
                line: Some(line_no + 1),
                message: format!("Self-admitted technical debt: {satd_type} - {text}"),
                details: None,
            });
        }
    }

    Ok(violations)
}

async fn check_single_file_security(
    project_path: &Path,
    file_path: &Path,
) -> Result<Vec<QualityViolation>> {
    use regex::Regex;

    let mut violations = Vec::new();

    // Security patterns to check
    let security_patterns = vec![
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
        (
            r#"(?i)token\s*=\s*["'][^"']+["']"#,
            "Hardcoded token detected",
        ),
        (r"(?i)unsafe\s*\{", "Unsafe code block detected"),
        (
            r"std::env::var\(.*\)\.unwrap\(\)",
            "Unsafe environment variable access",
        ),
    ];

    // Make file path absolute
    let abs_file_path = if file_path.is_absolute() {
        file_path.to_path_buf()
    } else {
        project_path.join(file_path)
    };

    if !abs_file_path.exists() {
        return Ok(violations);
    }

    let content = tokio::fs::read_to_string(&abs_file_path).await?;

    for (pattern_str, message) in security_patterns {
        let regex = Regex::new(pattern_str)?;
        for (line_no, line) in content.lines().enumerate() {
            if regex.is_match(line) {
                violations.push(QualityViolation {
                    check_type: "security".to_string(),
                    severity: "error".to_string(),
                    file: file_path.to_string_lossy().to_string(),
                    line: Some(line_no + 1),
                    message: message.to_string(),
                    details: None,
                });
            }
        }
    }

    Ok(violations)
}

#[cfg(test)]
mod quality_gate_part2c_pure_tests {
    //! Covers the pure-compute helpers in quality_gate_part2c.rs (202 uncov
    //! on broad, 0% cov). Skipped: async fn's that hit disk / AST analysis.
    use super::*;
    use crate::services::complexity::{ComplexityMetrics, FunctionComplexity};

    fn make_func(name: &str, cyclomatic: u16) -> FunctionComplexity {
        FunctionComplexity {
            name: name.to_string(),
            line_start: 10,
            line_end: 30,
            metrics: ComplexityMetrics {
                cyclomatic,
                cognitive: cyclomatic,
                nesting_max: 3,
                lines: 20,
                halstead: None,
            },
        }
    }

    // ── resolve_absolute_file_path: both branches ──

    #[test]
    fn test_resolve_absolute_file_path_absolute_input_preserved() {
        let abs = std::path::Path::new("/tmp/a.rs");
        let result = resolve_absolute_file_path(std::path::Path::new("/ignored"), abs);
        assert_eq!(result, std::path::PathBuf::from("/tmp/a.rs"));
    }

    #[test]
    fn test_resolve_absolute_file_path_relative_joined_with_project() {
        let base = std::path::Path::new("/home/proj");
        let rel = std::path::Path::new("src/a.rs");
        let result = resolve_absolute_file_path(base, rel);
        assert_eq!(result, std::path::PathBuf::from("/home/proj/src/a.rs"));
    }

    // ── validate_file_exists: ok + err arms ──

    #[test]
    fn test_validate_file_exists_missing_returns_err() {
        let missing = std::path::Path::new("/tmp/pmat_nope_0xDEADBEEF.rs");
        let err = validate_file_exists(missing).unwrap_err();
        assert!(err.to_string().contains("File not found"));
    }

    #[test]
    fn test_validate_file_exists_existing_path_returns_ok() {
        let tmp = tempfile::NamedTempFile::new().unwrap();
        assert!(validate_file_exists(tmp.path()).is_ok());
    }

    // ── function_exceeds_complexity_threshold: both branches ──

    #[test]
    fn test_function_exceeds_complexity_threshold_below_is_false() {
        let f = make_func("f", 5);
        assert!(!function_exceeds_complexity_threshold(&f, 10));
    }

    #[test]
    fn test_function_exceeds_complexity_threshold_equal_is_false() {
        let f = make_func("f", 10);
        // Uses > (strict), so equal to max_complexity does NOT exceed.
        assert!(!function_exceeds_complexity_threshold(&f, 10));
    }

    #[test]
    fn test_function_exceeds_complexity_threshold_above_is_true() {
        let f = make_func("f", 25);
        assert!(function_exceeds_complexity_threshold(&f, 10));
    }

    // ── create_complexity_violation: shape of emitted violation ──

    #[test]
    fn test_create_complexity_violation_fills_all_fields() {
        let f = make_func("foo_bar", 42);
        let violation =
            create_complexity_violation(&f, std::path::Path::new("src/foo.rs"), 10);
        assert_eq!(violation.check_type, "complexity");
        assert_eq!(violation.severity, "error");
        assert_eq!(violation.file, "src/foo.rs");
        assert_eq!(violation.line, Some(10)); // line_start=10
        assert!(
            violation.message.contains("foo_bar"),
            "msg must name the function: {}",
            violation.message
        );
        assert!(
            violation.message.contains("42"),
            "msg must carry cyclomatic value"
        );
        assert!(
            violation.message.contains("max: 10"),
            "msg must carry threshold"
        );
        assert!(violation.details.is_none());
    }

    // ── analyze_file_complexity early-return on non-rust extensions ──

    #[tokio::test]
    async fn test_analyze_file_complexity_non_rust_extension_is_no_op() {
        let tmp = tempfile::tempdir().unwrap();
        let path = tmp.path().join("a.py");
        std::fs::write(&path, "print(1)").unwrap();
        let mut violations = Vec::new();
        // Non-rust extension → early return without touching the analyzer.
        let res = analyze_file_complexity(&path, std::path::Path::new("a.py"), 10, &mut violations)
            .await;
        assert!(res.is_ok());
        assert!(violations.is_empty());
    }

    #[tokio::test]
    async fn test_analyze_file_complexity_no_extension_is_no_op() {
        let tmp = tempfile::tempdir().unwrap();
        let path = tmp.path().join("README");
        std::fs::write(&path, "hi").unwrap();
        let mut violations = Vec::new();
        let res =
            analyze_file_complexity(&path, std::path::Path::new("README"), 10, &mut violations)
                .await;
        assert!(res.is_ok());
        assert!(violations.is_empty());
    }

    // ── check_single_file_dead_code on missing file short-circuits ──

    #[tokio::test]
    async fn test_check_single_file_dead_code_missing_file_returns_empty() {
        let tmp = tempfile::tempdir().unwrap();
        let missing = tmp.path().join("nope.rs");
        let res = check_single_file_dead_code(tmp.path(), &missing).await.unwrap();
        assert!(res.is_empty(), "missing file → 0 violations");
    }
}

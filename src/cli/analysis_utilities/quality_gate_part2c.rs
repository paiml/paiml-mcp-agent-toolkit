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

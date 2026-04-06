// CB-400/401/402: Shell & Makefile quality (bashrs integration)
// Included from quality_checks.rs - shares parent module scope (no `use` imports)

/// CB-400: Check git hooks with bashrs
pub fn detect_cb400_git_hooks_quality(project_path: &Path) -> Vec<CbPatternViolation> {
    debug_assert!(project_path.exists(), "project_path must exist: {}", project_path.display());
    let hooks_dir = project_path.join(".git/hooks");
    if !hooks_dir.exists() {
        return Vec::new();
    }
    ["pre-commit", "pre-push", "commit-msg", "post-commit"]
        .iter()
        .flat_map(|name| lint_single_hook(&hooks_dir, name))
        .collect()
}

fn lint_single_hook(hooks_dir: &Path, hook_name: &str) -> Vec<CbPatternViolation> {
    debug_assert!(hooks_dir.exists(), "hooks_dir must exist: {}", hooks_dir.display());
    let hook_path = hooks_dir.join(hook_name);
    if !hook_path.exists() || hook_path.to_string_lossy().ends_with(".sample") {
        return Vec::new();
    }
    let file = format!(".git/hooks/{hook_name}");
    match run_bashrs_lint(&hook_path) {
        Ok(issues) => issues
            .into_iter()
            .map(|issue| CbPatternViolation {
                pattern_id: format!("CB-400-{}", issue.code),
                file: file.clone(),
                line: issue.line,
                description: format!("{}: {}", issue.code, issue.message),
                severity: match issue.severity.as_str() {
                    "error" => Severity::Error,
                    "warning" => Severity::Warning,
                    _ => Severity::Info,
                },
            })
            .collect(),
        Err(e) if !e.contains("not found") => vec![CbPatternViolation {
            pattern_id: "CB-400".to_string(),
            file,
            line: 0,
            description: format!("bashrs lint error: {e}"),
            severity: Severity::Warning,
        }],
        Err(_) => Vec::new(),
    }
}

/// CB-401: Check Makefile with bashrs
pub fn detect_cb401_makefile_quality(project_path: &Path) -> Vec<CbPatternViolation> {
    debug_assert!(project_path.exists(), "project_path must exist: {}", project_path.display());
    let mut violations = Vec::new();
    let makefile_path = project_path.join("Makefile");

    if !makefile_path.exists() {
        return violations;
    }

    // Run bashrs make lint on Makefile
    match run_bashrs_make_lint(&makefile_path) {
        Ok(issues) if !issues.is_empty() => {
            for issue in issues {
                violations.push(CbPatternViolation {
                    pattern_id: format!("CB-401-{}", issue.code),
                    file: "Makefile".to_string(),
                    line: issue.line,
                    description: format!("{}: {}", issue.code, issue.message),
                    severity: match issue.severity.as_str() {
                        "error" => Severity::Error,
                        "warning" => Severity::Warning,
                        _ => Severity::Info,
                    },
                });
            }
        }
        Ok(_) => {} // No issues
        Err(e) => {
            if !e.contains("not found") {
                violations.push(CbPatternViolation {
                    pattern_id: "CB-401".to_string(),
                    file: "Makefile".to_string(),
                    line: 0,
                    description: format!("bashrs make lint error: {}", e),
                    severity: Severity::Warning,
                });
            }
        }
    }

    violations
}

/// CB-402: Check shell scripts with bashrs
pub fn detect_cb402_shell_script_quality(project_path: &Path) -> Vec<CbPatternViolation> {
    debug_assert!(project_path.exists(), "project_path must exist: {}", project_path.display());
    let mut violations = Vec::new();

    // Find all .sh files (limit to reasonable depth)
    let sh_files: Vec<_> = walkdir::WalkDir::new(project_path)
        .max_depth(4)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| {
            let path = e.path();
            path.extension().is_some_and(|ext| ext == "sh")
                && !path.to_string_lossy().contains("target/")
                && !path.to_string_lossy().contains("node_modules/")
        })
        .take(20) // Limit to avoid slow scans
        .collect();

    for entry in sh_files {
        match run_bashrs_lint(entry.path()) {
            Ok(issues) if !issues.is_empty() => {
                for issue in issues {
                    violations.push(CbPatternViolation {
                        pattern_id: format!("CB-402-{}", issue.code),
                        file: entry
                            .path()
                            .strip_prefix(project_path)
                            .map(|p| p.display().to_string())
                            .unwrap_or_else(|_| entry.path().display().to_string()),
                        line: issue.line,
                        description: format!("{}: {}", issue.code, issue.message),
                        severity: match issue.severity.as_str() {
                            "error" => Severity::Error,
                            "warning" => Severity::Warning,
                            _ => Severity::Info,
                        },
                    });
                }
            }
            Ok(_) => {}  // No issues
            Err(_) => {} // Skip silently for shell scripts
        }
    }

    violations
}

/// Run bashrs lint on a file and parse results
pub(super) fn run_bashrs_lint(path: &Path) -> Result<Vec<BashrsIssue>, String> {
    debug_assert!(path.exists(), "path must exist: {}", path.display());
    use std::process::Command;

    let output = Command::new("bashrs")
        .args(["lint", "--format", "json", "--level", "warning"])
        .arg(path)
        .output()
        .map_err(|e| format!("bashrs not found: {}", e))?;

    if output.status.success() {
        // No issues
        return Ok(Vec::new());
    }

    // Parse JSON output
    let stdout = String::from_utf8_lossy(&output.stdout);
    parse_bashrs_json_output(&stdout)
}

/// Run bashrs make lint on Makefile
pub(super) fn run_bashrs_make_lint(path: &Path) -> Result<Vec<BashrsIssue>, String> {
    debug_assert!(path.exists(), "path must exist: {}", path.display());
    use std::process::Command;

    let output = Command::new("bashrs")
        .args(["make", "lint", "--format", "json"])
        .arg(path)
        .output()
        .map_err(|e| format!("bashrs not found: {}", e))?;

    if output.status.success() {
        return Ok(Vec::new());
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    parse_bashrs_json_output(&stdout)
}

/// Parse bashrs JSON output into issues
pub(super) fn parse_bashrs_json_output(json_str: &str) -> Result<Vec<BashrsIssue>, String> {
    // bashrs outputs JSON array of diagnostics
    #[derive(serde::Deserialize)]
    struct BashrsOutput {
        #[serde(default)]
        diagnostics: Vec<BashrsDiagnostic>,
    }

    #[derive(serde::Deserialize)]
    struct BashrsDiagnostic {
        code: String,
        message: String,
        #[serde(default)]
        line: usize,
        #[serde(default)]
        severity: String,
    }

    // Try to parse as array first, then as object
    if let Ok(diagnostics) = serde_json::from_str::<Vec<BashrsDiagnostic>>(json_str) {
        return Ok(diagnostics
            .into_iter()
            .map(|d| BashrsIssue {
                code: d.code,
                message: d.message,
                line: d.line,
                severity: d.severity,
            })
            .collect());
    }

    if let Ok(output) = serde_json::from_str::<BashrsOutput>(json_str) {
        return Ok(output
            .diagnostics
            .into_iter()
            .map(|d| BashrsIssue {
                code: d.code,
                message: d.message,
                line: d.line,
                severity: d.severity,
            })
            .collect());
    }

    // If JSON parsing fails, return empty (graceful degradation)
    Ok(Vec::new())
}

// CB-400/401/402: Shell & Makefile quality (bashrs integration)
// Included from quality_checks.rs - shares parent module scope (no `use` imports)

/// CB-400: Check git hooks with bashrs
#[provable_contracts_macros::contract("pmat-core.yaml", equation = "path_exists")]
pub fn detect_cb400_git_hooks_quality(project_path: &Path) -> Vec<CbPatternViolation> {
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
#[provable_contracts_macros::contract("pmat-core.yaml", equation = "path_exists")]
pub fn detect_cb401_makefile_quality(project_path: &Path) -> Vec<CbPatternViolation> {
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

/// How many shell scripts CB-402 will lint, and how deep it will look.
///
/// Both are real limits and both are now DISCLOSED when they bite: infra has
/// 104 `.sh` files, 95 of them within depth 4, so a silent `.take(20)` meant
/// the check spoke for 20 files while its message claimed "All shell scripts".
const SHELL_SCAN_LIMIT: usize = 20;
const SHELL_SCAN_DEPTH: usize = 4;

/// CB-402: Check shell scripts with bashrs
#[provable_contracts_macros::contract("pmat-core.yaml", equation = "path_exists")]
pub fn detect_cb402_shell_script_quality(project_path: &Path) -> Vec<CbPatternViolation> {
    let mut violations = Vec::new();

    // Find all .sh files (limit to reasonable depth)
    let candidates: Vec<_> = walkdir::WalkDir::new(project_path)
        .max_depth(SHELL_SCAN_DEPTH)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| {
            let path = e.path();
            path.extension().is_some_and(|ext| ext == "sh")
                && !path.to_string_lossy().contains("target/")
                && !path.to_string_lossy().contains("node_modules/")
        })
        .collect();
    let truncated = candidates.len().saturating_sub(SHELL_SCAN_LIMIT);
    let sh_files = candidates.into_iter().take(SHELL_SCAN_LIMIT);

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
            Ok(_) => {} // No issues
            // A script bashrs could not lint is NOT a script that passed.
            // This was `Err(_) => {}` with the comment "Skip silently for shell
            // scripts", which is how a broken bashrs invocation became a clean
            // bill of health for the whole tree.
            Err(e) => violations.push(CbPatternViolation {
                pattern_id: "CB-402-UNMEASURED".to_string(),
                file: entry
                    .path()
                    .strip_prefix(project_path)
                    .map(|p| p.display().to_string())
                    .unwrap_or_else(|_| entry.path().display().to_string()),
                line: 0,
                description: format!("not measured: {e}"),
                severity: Severity::Warning,
            }),
        }
    }

    // The scan is capped (see `SHELL_SCAN_LIMIT`). Say so, rather than letting
    // 20-of-104 read as 104-of-104.
    if truncated > 0 {
        violations.push(CbPatternViolation {
            pattern_id: "CB-402-TRUNCATED".to_string(),
            file: String::new(),
            line: 0,
            description: format!(
                "{truncated} shell script(s) were NOT examined: the scan stops at \
                 {SHELL_SCAN_LIMIT} files and depth {SHELL_SCAN_DEPTH}. This result \
                 describes the files it reached, not the repository."
            ),
            severity: Severity::Warning,
        });
    }

    violations
}

/// Run bashrs lint on a file and parse results
pub(super) fn run_bashrs_lint(path: &Path) -> Result<Vec<BashrsIssue>, String> {
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

    // Try to parse as array first, then as object. Both read the JSON BODY:
    // bashrs prefixes its stdout with log lines (see `json_body`).
    let json_str = json_body(json_str);
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

    // Unparseable output is NOT a clean result.
    //
    // This returned `Ok(Vec::new())` under the comment "graceful degradation",
    // and that one line turned CB-400 into a check that could not fail. bashrs
    // 6.66.2 writes an ANSI-coloured log line to STDOUT before its JSON:
    //
    //     \u{1b}[2m2026-…Z\u{1b}[0m \u{1b}[32m INFO\u{1b}[0m … Linting ./scripts/x.sh
    //     {
    //       "file": "./scripts/x.sh",
    //       "diagnostics": [ … ]
    //
    // so `serde_json` fails on column 1, the empty vec propagates as "no
    // violations", and CB-400 reports `Pass: bashrs: All shell scripts and
    // Makefiles pass quality checks` for a tree where bashrs itself exits 2
    // with 948 errors and 1493 warnings across 59 of 104 scripts.
    //
    // Degrading gracefully is right; degrading SILENTLY into the answer the
    // caller most wants to hear is not. The caller now learns it got nothing.
    Err(format!(
        "bashrs output was not JSON ({} bytes); the first non-log line was: {}",
        json_str.len(),
        json_str
            .lines()
            .find(|l| !l.trim().is_empty())
            .map_or("<empty>", |l| l.trim())
            .chars()
            .take(120)
            .collect::<String>()
    ))
}

/// The JSON body of a bashrs response, with any leading log preamble removed.
///
/// bashrs emits human-readable log lines on stdout ahead of its machine-readable
/// payload, so the payload starts at the first `{` or `[` rather than at byte 0.
/// Skipping to it is what lets the diagnostics actually be read; without this the
/// parse fails and every script looks clean.
fn json_body(raw: &str) -> &str {
    // Anchor on a LINE that opens the payload, not on the first `{`/`[` byte:
    // the log preamble is ANSI-coloured, and every escape sequence contains a
    // literal `[` (`ESC[2m`), so a byte search lands inside the colour code and
    // "fixes" nothing. Found by running it.
    let mut offset = 0usize;
    for line in raw.split_inclusive('\n') {
        if matches!(line.trim_start().as_bytes().first(), Some(b'{' | b'[')) {
            return &raw[offset..];
        }
        offset += line.len();
    }
    raw
}

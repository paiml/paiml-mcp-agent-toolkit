#![cfg_attr(coverage_nightly, coverage(off))]
use super::types::*;
use std::fs;
use std::path::Path;

// =============================================================================
// CB-125, CB-126, CB-127: Coverage Quality & Test Performance (v2.2)
// Per improve-pmat-comply.md v2.2.0 specification
// =============================================================================

/// CB-125: Detect coverage exclusion gaming
/// Per [GAME-001] Popper: Unfalsifiable claims are unscientific
/// Per [GAME-002] Google TAP: >20% exclusion indicates gaming
/// Thresholds:
/// - >10 exclusion patterns = Warning (complexity suggests gaming)
/// - >20% LOC excluded = Error (significant coverage blind spot)
/// - >50% LOC excluded = Critical (coverage metric meaningless)
pub fn detect_cb125_coverage_exclusion_gaming(project_path: &Path) -> Vec<CbPatternViolation> {
    let makefile_path = project_path.join("Makefile");
    let content = match fs::read_to_string(&makefile_path) {
        Ok(c) => c,
        Err(_) => return Vec::new(),
    };

    let (exclusion_count, exclusion_line) = count_exclusion_patterns(&content);
    classify_exclusion_severity(exclusion_count, exclusion_line, &makefile_path)
}

/// Count pipe-separated patterns in --ignore-filename-regex lines.
fn count_exclusion_patterns(content: &str) -> (usize, usize) {
    let mut count = 0;
    let mut last_line = 0;
    for (line_num, line) in content.lines().enumerate() {
        if !line.contains("--ignore-filename-regex") {
            continue;
        }
        last_line = line_num + 1;
        let start = line.find('\'').unwrap_or(0);
        let end = line.rfind('\'').unwrap_or(0);
        if start < end {
            count += line[start + 1..end].matches('|').count() + 1;
        }
    }
    (count, last_line)
}

/// Map exclusion pattern count to CB-125 severity tier.
fn classify_exclusion_severity(
    count: usize,
    line: usize,
    makefile_path: &Path,
) -> Vec<CbPatternViolation> {
    let file = makefile_path.display().to_string();
    let (pattern_id, desc, severity) = if count > 50 {
        ("CB-125-C", format!(
            "CRITICAL: {count} coverage exclusion patterns detected. Coverage metric is meaningless. \
            Per [GAME-001] Popper: unfalsifiable coverage claims are unscientific. \
            Reduce to ≤10 patterns (binary entry points only)"
        ), Severity::Critical)
    } else if count > 20 {
        ("CB-125-B", format!(
            "{count} coverage exclusion patterns exceed 20% budget per [GAME-002] Google TAP. \
            Significant coverage blind spot. Reduce exclusions or document technical debt"
        ), Severity::Error)
    } else if count > 10 {
        ("CB-125-A", format!(
            "{count} coverage exclusion patterns suggests complexity. \
            Consider reducing to ≤10 patterns (binary entry points only)"
        ), Severity::Warning)
    } else {
        return Vec::new();
    };
    vec![CbPatternViolation {
        pattern_id: pattern_id.to_string(),
        file,
        line,
        description: desc,
        severity,
    }]
}

/// Check sleep duration and return violation if threshold exceeded
pub(super) fn check_sleep_violation(
    duration: f64,
    file: &str,
    line: usize,
) -> Option<CbPatternViolation> {
    let (pattern_id, desc, severity) = if duration > 300.0 {
        ("CB-126-C", "Test sleep exceeds 300s critical threshold", Severity::Critical)
    } else if duration > 60.0 {
        ("CB-126-B", "Test sleep exceeds 60s Tier 2 threshold", Severity::Error)
    } else if duration > 5.0 {
        ("CB-126-A", "Test sleep exceeds 5s Tier 1 threshold", Severity::Warning)
    } else {
        return None;
    };
    Some(CbPatternViolation {
        pattern_id: pattern_id.to_string(),
        file: file.to_string(),
        line,
        description: desc.to_string(),
        severity,
    })
}

/// CB-126: Detect slow tests that violate tiered TDD feedback requirements
pub fn detect_cb126_slow_tests(project_path: &Path) -> Vec<CbPatternViolation> {
    let mut violations = Vec::new();
    violations.extend(check_makefile_test_targets(project_path));
    violations.extend(check_sleep_durations(project_path));
    violations
}

pub(super) fn check_makefile_test_targets(project_path: &Path) -> Vec<CbPatternViolation> {
    let makefile_path = project_path.join("Makefile");
    let content = match fs::read_to_string(&makefile_path) {
        Ok(c) => c,
        Err(_) => return Vec::new(),
    };
    find_test_targets_missing_proptest(&content, &makefile_path.display().to_string())
}

fn find_test_targets_missing_proptest(content: &str, file_path: &str) -> Vec<CbPatternViolation> {
    let mut violations = Vec::new();
    let mut in_test_target = false;
    let mut target_line = 0;
    let mut has_proptest = false;
    let mut has_cargo_test = false;

    for (line_num, line) in content.lines().enumerate() {
        if line.starts_with("test") && line.contains(':') {
            in_test_target = true;
            target_line = line_num + 1;
            has_proptest = false;
            has_cargo_test = false;
        }
        if !in_test_target {
            continue;
        }
        has_proptest |= line.contains("PROPTEST_CASES") || line.contains("QUICKCHECK_TESTS");
        has_cargo_test |= line.contains("cargo test") || line.contains("cargo +nightly llvm-cov test");

        if is_end_of_makefile_target_generic(line, "test") {
            if has_cargo_test && !has_proptest {
                violations.push(CbPatternViolation {
                    pattern_id: "CB-126-D".to_string(),
                    file: file_path.to_string(),
                    line: target_line,
                    description: "Test target missing PROPTEST_CASES/QUICKCHECK_TESTS".to_string(),
                    severity: Severity::Warning,
                });
            }
            in_test_target = false;
        }
    }
    violations
}

pub(super) fn is_end_of_makefile_target_generic(line: &str, target_prefix: &str) -> bool {
    line.is_empty()
        || (line.chars().next().map(|c| !c.is_whitespace()).unwrap_or(false)
            && !line.starts_with('\t')
            && !line.starts_with(target_prefix))
}

pub(super) fn check_sleep_durations(project_path: &Path) -> Vec<CbPatternViolation> {
    let src_dir = project_path.join("src");
    let entries = match walkdir_rs_files(&src_dir) {
        Ok(e) => e,
        Err(_) => return Vec::new(),
    };
    entries
        .iter()
        .flat_map(|entry| scan_file_for_sleep_violations(entry))
        .collect()
}

fn scan_file_for_sleep_violations(path: &Path) -> Vec<CbPatternViolation> {
    let content = match fs::read_to_string(path) {
        Ok(c) => c,
        Err(_) => return Vec::new(),
    };
    let file_path = path.display().to_string();
    content
        .lines()
        .enumerate()
        .filter(|(_, line)| line.contains("thread::sleep") && line.contains("Duration::from_secs"))
        .filter_map(|(i, line)| {
            extract_sleep_duration(line)
                .and_then(|dur| check_sleep_violation(dur, &file_path, i + 1))
        })
        .collect()
}

/// Helper to extract sleep duration from a line containing sleep calls
pub(super) fn extract_sleep_duration(line: &str) -> Option<f64> {
    if let Some(start) = line.find("from_secs(") {
        let after = &line[start + 10..];
        if let Some(end) = after.find(')') {
            let num_str = &after[..end];
            return num_str.trim().parse::<f64>().ok();
        }
    }
    if let Some(start) = line.find("from_millis(") {
        let after = &line[start + 12..];
        if let Some(end) = after.find(')') {
            let num_str = &after[..end];
            if let Ok(millis) = num_str.trim().parse::<f64>() {
                return Some(millis / 1000.0);
            }
        }
    }
    None
}

/// State for tracking coverage target parsing
#[derive(Default)]
pub(super) struct CoverageTargetState {
    pub(super) active: bool,
    pub(super) line: usize,
    pub(super) has_nextest: bool,
    pub(super) has_llvm_cov: bool,
    pub(super) has_proptest_cases: bool,
    pub(super) has_lib_flag: bool,
    /// Whether this target actually runs cargo tests (vs. report/clean/alias/deno)
    pub(super) runs_cargo_tests: bool,
}

impl CoverageTargetState {
    pub(super) fn reset(&mut self, line: usize) {
        self.active = true;
        self.line = line;
        self.has_nextest = false;
        self.has_llvm_cov = false;
        self.has_proptest_cases = false;
        self.has_lib_flag = false;
        self.runs_cargo_tests = false;
    }

    pub(super) fn update_from_line(&mut self, line: &str) {
        let trimmed = line.trim();
        // Skip comments and echo statements
        if trimmed.starts_with('#') || trimmed.starts_with("@#") {
            return;
        }
        let is_echo = trimmed.starts_with("@echo") || trimmed.starts_with("echo");
        if !is_echo && line.contains("nextest") {
            self.has_nextest = true;
            self.runs_cargo_tests = true;
        }
        if line.contains("llvm-cov") || line.contains("cargo-llvm-cov") {
            self.has_llvm_cov = true;
        }
        // Detect actual test execution: `cargo test` or `cargo llvm-cov test`
        // Exclude report-only commands like `cargo llvm-cov report`
        if !is_echo && (line.contains("cargo test") || line.contains("cargo llvm-cov test")) {
            self.runs_cargo_tests = true;
        }
        if line.contains("PROPTEST_CASES") || line.contains("QUICKCHECK_TESTS") {
            self.has_proptest_cases = true;
        }
        if line.contains("--lib") {
            self.has_lib_flag = true;
        }
    }

    pub(super) fn collect_violations(&self, file_path: &str) -> Vec<CbPatternViolation> {
        let mut violations = Vec::new();

        // Only flag targets that actually run cargo tests.
        // Skip: alias/delegate targets, report-only, clean, open, invalidate, deno targets.
        if !self.runs_cargo_tests {
            return violations;
        }

        if self.has_nextest && self.has_llvm_cov {
            violations.push(CbPatternViolation {
                pattern_id: "CB-127-A".to_string(),
                file: file_path.to_string(),
                line: self.line,
                description: "CRITICAL: nextest + llvm-cov causes profraw explosion. \
                    Use 'cargo llvm-cov test' instead".to_string(),
                severity: Severity::Error,
            });
        }
        if !self.has_proptest_cases {
            violations.push(CbPatternViolation {
                pattern_id: "CB-127-B".to_string(),
                file: file_path.to_string(),
                line: self.line,
                description: "Coverage target missing PROPTEST_CASES/QUICKCHECK_TESTS".to_string(),
                severity: Severity::Warning,
            });
        }
        if !self.has_lib_flag && self.has_llvm_cov {
            violations.push(CbPatternViolation {
                pattern_id: "CB-127-C".to_string(),
                file: file_path.to_string(),
                line: self.line,
                description: "Coverage target missing --lib flag".to_string(),
                severity: Severity::Warning,
            });
        }
        violations
    }
}

pub(super) fn is_end_of_makefile_target(line: &str) -> bool {
    line.is_empty()
        || (line.chars().next().map(|c| !c.is_whitespace()).unwrap_or(false)
            && !line.starts_with('\t')
            && !line.starts_with("coverage"))
}

/// CB-127: Detect slow coverage configurations
/// Per [PERF-001] certeza: coverage budget <2min for Tier 2
pub fn detect_cb127_slow_coverage(project_path: &Path) -> Vec<CbPatternViolation> {
    let mut violations = Vec::new();
    let makefile_path = project_path.join("Makefile");

    let content = match fs::read_to_string(&makefile_path) {
        Ok(c) => c,
        Err(_) => return violations,
    };

    let mut state = CoverageTargetState::default();
    let file_path = makefile_path.display().to_string();

    for (line_num, line) in content.lines().enumerate() {
        // Detect coverage target start
        if (line.starts_with("coverage") || line.starts_with("coverage-")) && line.contains(':') {
            state.reset(line_num + 1);
            continue;
        }

        if state.active {
            if is_end_of_makefile_target(line) {
                violations.extend(state.collect_violations(&file_path));
                state.active = false;
            } else {
                state.update_from_line(line);
            }
        }
    }

    violations
}

// =============================================================================
// CB-400: Shell & Makefile Quality (bashrs integration)
// Uses bashrs for deterministic, idempotent, and safe shell scripting.
//
// Sub-checks:
// - CB-400: Git hooks quality (pre-commit, pre-push, etc.)
// - CB-401: Makefile quality
// - CB-402: Shell script quality (*.sh)
// =============================================================================

/// Result of bashrs lint check
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct BashrsLintResult {
    pub file: String,
    pub issues: Vec<BashrsIssue>,
    pub passed: bool,
}

/// Individual bashrs issue
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct BashrsIssue {
    pub code: String,
    pub message: String,
    pub line: usize,
    pub severity: String,
}

/// CB-400: Check git hooks with bashrs
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

/// CB-402: Check shell scripts with bashrs
pub fn detect_cb402_shell_script_quality(project_path: &Path) -> Vec<CbPatternViolation> {
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
                        file: entry.path().strip_prefix(project_path)
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
            Err(_) => {} // Skip silently for shell scripts
        }
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

    // Try to parse as array first, then as object
    if let Ok(diagnostics) = serde_json::from_str::<Vec<BashrsDiagnostic>>(json_str) {
        return Ok(diagnostics.into_iter().map(|d| BashrsIssue {
            code: d.code,
            message: d.message,
            line: d.line,
            severity: d.severity,
        }).collect());
    }

    if let Ok(output) = serde_json::from_str::<BashrsOutput>(json_str) {
        return Ok(output.diagnostics.into_iter().map(|d| BashrsIssue {
            code: d.code,
            message: d.message,
            line: d.line,
            severity: d.severity,
        }).collect());
    }

    // If JSON parsing fails, return empty (graceful degradation)
    Ok(Vec::new())
}

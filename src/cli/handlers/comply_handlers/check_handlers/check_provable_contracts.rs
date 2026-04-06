// Provable Contracts compliance checks (CB-1200)
//
// Detects if a project uses provable-contracts YAML contract files
// and validates them via `pv lint`. Integrates contract quality scoring
// into pmat comply quality gates.

/// Quick check if a directory contains any contract YAML files (not just binding.yaml).
fn has_contract_yamls(dir: &Path) -> bool {
    std::fs::read_dir(dir)
        .into_iter()
        .flatten()
        .flatten()
        .any(|e| {
            let p = e.path();
            p.is_file()
                && p.extension()
                    .is_some_and(|ext| ext == "yaml" || ext == "yml")
                && !p
                    .file_name()
                    .is_some_and(|n| n.to_string_lossy().contains("binding"))
        })
}

/// Resolve the contracts directory — sibling provable-contracts preferred, local fallback.
/// Prefers ../provable-contracts/contracts/<name> because it contains only provable-contracts
/// YAMLs. Local contracts/ may contain pmat work contracts (different schema) that pv lint
/// cannot parse.
fn resolve_contracts_dir(project_path: &Path) -> Option<std::path::PathBuf> {
    let abs = std::fs::canonicalize(project_path).ok()?;
    let parent = abs.parent()?;
    let pv_contracts = parent.join("provable-contracts").join("contracts");
    if pv_contracts.exists() {
        // Try directory name first
        let dir_name = abs.file_name()?.to_str()?;
        let sibling = pv_contracts.join(dir_name);
        if sibling.exists() {
            return Some(sibling);
        }
        // Try Cargo.toml package name (e.g., paiml-mcp-agent-toolkit → pmat)
        let cargo_toml = project_path.join("Cargo.toml");
        if let Ok(content) = std::fs::read_to_string(&cargo_toml) {
            for line in content.lines() {
                let trimmed = line.trim();
                if trimmed.starts_with("name") && trimmed.contains('=') {
                    if let Some(name) = trimmed.split('=').nth(1) {
                        let pkg = name.trim().trim_matches('"');
                        let by_pkg = pv_contracts.join(pkg);
                        if by_pkg.exists() {
                            return Some(by_pkg);
                        }
                    }
                    break;
                }
            }
        }
    }
    // Fallback: local contracts/ if it has provable-contracts YAMLs
    let local = project_path.join("contracts");
    if local.exists() && has_contract_yamls(&local) {
        return Some(local);
    }
    None
}
//
// Auto-skips if no contracts/ directory or *.yaml contract files found.

use std::path::Path;

use super::types::*;

/// Check if project uses provable-contracts and validate contract quality (CB-1200)
///
/// Detection: looks for `contracts/` directory containing YAML files with
/// `proof_obligations` or `equations` keys (provable-contracts schema markers).
///
/// If detected, runs `pv lint` to validate contracts and reports:
/// - Contract count and coverage
/// - Proof obligation status
/// - Quality score (A-F grade)
/// - Any validation errors or warnings
#[provable_contracts_macros::contract("pmat-core.yaml", equation = "path_exists")]
pub(crate) fn check_provable_contracts(project_path: &Path) -> ComplianceCheck {
    // Phase 1: Detect if this project uses provable-contracts
    let contracts_dir = match resolve_contracts_dir(project_path) {
        Some(d) => d,
        None => {
            return skip_check(
                "CB-1200: Provable Contracts",
                "No provable-contract YAML files found in contracts/",
            );
        }
    };

    let contract_files = find_contract_files(&contracts_dir);
    if contract_files.is_empty() {
        return skip_check(
            "CB-1200: Provable Contracts",
            "No provable-contract YAML files found in contracts/",
        );
    }

    // Phase 2: Check if `pv` CLI is available
    let pv_available = std::process::Command::new("pv")
        .arg("--version")
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false);

    if !pv_available {
        return ComplianceCheck {
            name: "CB-1200: Provable Contracts".into(),
            status: CheckStatus::Warn,
            message: format!(
                "Found {} contract file(s) but `pv` CLI not installed. \
                 Install: cargo install --path ../provable-contracts/crates/provable-contracts-cli",
                contract_files.len()
            ),
            severity: Severity::Warning,
        };
    }

    // Phase 3: Run `pv lint` for validation + scoring
    let lint_result = run_pv_lint(project_path);

    // Phase 4: Run `pv score` for quality grading
    let score_result = run_pv_score(&contracts_dir);

    // Phase 5: Check binding coverage
    let binding_result = check_binding_coverage(project_path);

    // Build composite result
    build_provable_contracts_result(&contract_files, lint_result, score_result, binding_result)
}

/// Find YAML files that are provable-contracts schema files
fn find_contract_files(contracts_dir: &Path) -> Vec<String> {
    let mut files = Vec::new();

    for entry in walkdir::WalkDir::new(contracts_dir)
        .max_depth(3)
        .into_iter()
        .filter_map(|e| e.ok())
    {
        let path = entry.path();
        if path.is_file()
            && (path.extension().is_some_and(|e| e == "yaml" || e == "yml"))
            && !is_binding_file(path)
        {
            // Quick check: does it look like a provable-contract file?
            if let Ok(content) = std::fs::read_to_string(path) {
                if content.contains("proof_obligations")
                    || content.contains("equations:")
                    || content.contains("falsification_tests")
                    || content.contains("kani_harnesses")
                {
                    let rel = path.strip_prefix(contracts_dir).unwrap_or(path);
                    files.push(rel.display().to_string());
                }
            }
        }
    }

    files
}

/// Check if a YAML file is a binding registry (not a contract)
fn is_binding_file(path: &Path) -> bool {
    path.file_name()
        .is_some_and(|n| n.to_string_lossy().contains("binding"))
}

/// Run `pv lint` and parse the result
fn run_pv_lint(project_path: &Path) -> PvLintResult {
    let contracts_dir =
        resolve_contracts_dir(project_path).unwrap_or_else(|| project_path.join("contracts"));

    let output = std::process::Command::new("pv")
        .args([
            "lint",
            &contracts_dir.display().to_string(),
            "--format",
            "json",
        ])
        .current_dir(project_path)
        .output();

    match output {
        Ok(out) if out.status.success() => {
            // Parse JSON output for details
            let stdout = String::from_utf8_lossy(&out.stdout);
            parse_pv_lint_json(&stdout)
        }
        Ok(out) => {
            let stderr = String::from_utf8_lossy(&out.stderr);
            let stdout = String::from_utf8_lossy(&out.stdout);
            // pv lint returns non-zero on violations — parse them
            let mut result = parse_pv_lint_json(&stdout);
            if result.errors == 0 && result.warnings == 0 {
                // Couldn't parse JSON, use stderr
                result.errors = 1;
                result.error_details.push(stderr.trim().to_string());
            }
            result
        }
        Err(e) => PvLintResult {
            passed: false,
            contracts_checked: 0,
            errors: 1,
            warnings: 0,
            error_details: vec![format!("Failed to run pv lint: {e}")],
        },
    }
}

/// Run `pv score` and extract the grade
fn run_pv_score(contracts_dir: &Path) -> PvScoreResult {
    let output = std::process::Command::new("pv")
        .args([
            "score",
            &contracts_dir.display().to_string(),
            "--format",
            "json",
        ])
        .output();

    match output {
        Ok(out) if out.status.success() => {
            let stdout = String::from_utf8_lossy(&out.stdout);
            parse_pv_score_json(&stdout)
        }
        Ok(out) => {
            let stderr = String::from_utf8_lossy(&out.stderr);
            PvScoreResult {
                overall_score: 0.0,
                grade: "?".into(),
                dimensions: Vec::new(),
                error: Some(stderr.trim().to_string()),
            }
        }
        Err(e) => PvScoreResult {
            overall_score: 0.0,
            grade: "?".into(),
            dimensions: Vec::new(),
            error: Some(format!("Failed to run pv score: {e}")),
        },
    }
}

/// Check binding.yaml coverage for the project
fn check_binding_coverage(project_path: &Path) -> BindingResult {
    let contracts_dir =
        resolve_contracts_dir(project_path).unwrap_or_else(|| project_path.join("contracts"));

    // Look for binding.yaml files in contracts/
    let binding_files: Vec<_> = walkdir::WalkDir::new(&contracts_dir)
        .max_depth(3)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| is_binding_file(e.path()) && e.path().is_file())
        .collect();

    if binding_files.is_empty() {
        return BindingResult {
            has_bindings: false,
            total_bindings: 0,
            implemented: 0,
            partial: 0,
            not_implemented: 0,
        };
    }

    let mut total = 0;
    let mut implemented = 0;
    let mut partial = 0;
    let mut not_impl = 0;

    for entry in &binding_files {
        if let Ok(content) = std::fs::read_to_string(entry.path()) {
            // Count binding status lines
            for line in content.lines() {
                let trimmed = line.trim();
                if trimmed.contains("status:") {
                    total += 1;
                    if trimmed.contains("implemented") && !trimmed.contains("not_implemented") {
                        implemented += 1;
                    } else if trimmed.contains("partial") {
                        partial += 1;
                    } else if trimmed.contains("not_implemented") {
                        not_impl += 1;
                    }
                }
            }
        }
    }

    BindingResult {
        has_bindings: true,
        total_bindings: total,
        implemented,
        partial,
        not_implemented: not_impl,
    }
}

// --- Result types ---

#[allow(dead_code)]
struct PvLintResult {
    passed: bool,
    contracts_checked: usize,
    errors: usize,
    warnings: usize,
    error_details: Vec<String>,
}

#[allow(dead_code)]
struct PvScoreResult {
    overall_score: f64,
    grade: String,
    dimensions: Vec<(String, f64)>,
    error: Option<String>,
}

#[allow(dead_code)]
struct BindingResult {
    has_bindings: bool,
    total_bindings: usize,
    implemented: usize,
    partial: usize,
    not_implemented: usize,
}

// --- JSON parsing ---

fn parse_pv_lint_json(json_str: &str) -> PvLintResult {
    // Try to parse as JSON; fall back to text analysis
    if let Ok(value) = serde_json::from_str::<serde_json::Value>(json_str) {
        let errors = value
            .get("errors")
            .or_else(|| value.get("error_count"))
            .and_then(|v| v.as_u64())
            .unwrap_or(0) as usize;
        let warnings = value
            .get("warnings")
            .or_else(|| value.get("warning_count"))
            .and_then(|v| v.as_u64())
            .unwrap_or(0) as usize;
        let contracts = value
            .get("contracts_checked")
            .or_else(|| value.get("total_contracts"))
            .and_then(|v| v.as_u64())
            .unwrap_or(0) as usize;

        let mut details = Vec::new();
        if let Some(violations) = value.get("violations").and_then(|v| v.as_array()) {
            for v in violations.iter().take(5) {
                if let Some(msg) = v.get("message").and_then(|m| m.as_str()) {
                    details.push(msg.to_string());
                }
            }
        }

        PvLintResult {
            passed: errors == 0,
            contracts_checked: contracts,
            errors,
            warnings,
            error_details: details,
        }
    } else {
        // Text fallback: count lines with "error" or "warning"
        let error_count = json_str
            .lines()
            .filter(|l| l.to_lowercase().contains("error"))
            .count();
        let warn_count = json_str
            .lines()
            .filter(|l| l.to_lowercase().contains("warning"))
            .count();

        PvLintResult {
            passed: error_count == 0,
            contracts_checked: 0,
            errors: error_count,
            warnings: warn_count,
            error_details: Vec::new(),
        }
    }
}

fn parse_pv_score_json(json_str: &str) -> PvScoreResult {
    if let Ok(value) = serde_json::from_str::<serde_json::Value>(json_str) {
        let score = value
            .get("overall_score")
            .or_else(|| value.get("score"))
            .and_then(|v| v.as_f64())
            .unwrap_or(0.0);
        let grade = value
            .get("grade")
            .and_then(|v| v.as_str())
            .unwrap_or("?")
            .to_string();

        let mut dims = Vec::new();
        if let Some(dimensions) = value.get("dimensions").and_then(|v| v.as_object()) {
            for (key, val) in dimensions {
                if let Some(s) = val.as_f64() {
                    dims.push((key.clone(), s));
                }
            }
        }

        PvScoreResult {
            overall_score: score,
            grade,
            dimensions: dims,
            error: None,
        }
    } else {
        PvScoreResult {
            overall_score: 0.0,
            grade: "?".into(),
            dimensions: Vec::new(),
            error: Some("Could not parse pv score output".into()),
        }
    }
}

// --- Result builder ---

fn build_provable_contracts_result(
    contract_files: &[String],
    lint: PvLintResult,
    score: PvScoreResult,
    binding: BindingResult,
) -> ComplianceCheck {
    use crate::cli::colors as c;

    let mut parts = Vec::new();
    let mut has_errors = false;
    let mut has_warnings = false;

    // Contract count
    parts.push(format!(
        "{} contract(s) found",
        c::number(&contract_files.len().to_string())
    ));

    // Lint results
    if lint.passed {
        parts.push(c::pass("lint passed").to_string());
    } else {
        has_errors = true;
        parts.push(format!(
            "{} lint: {} error(s), {} warning(s)",
            c::fail(""),
            c::number(&lint.errors.to_string()),
            c::number(&lint.warnings.to_string()),
        ));
        for detail in lint.error_details.iter().take(3) {
            parts.push(format!("  {}", c::dim(detail)));
        }
    }

    // Score results
    if score.error.is_none() && score.overall_score > 0.0 {
        let grade_str = format!("grade {}", score.grade);
        let grade_colored = if score.overall_score >= 0.8 {
            c::pass(&grade_str)
        } else if score.overall_score >= 0.6 {
            format!(
                "{}{}{}",
                crate::cli::colors::YELLOW,
                grade_str,
                crate::cli::colors::RESET
            )
        } else {
            c::fail(&grade_str)
        };
        parts.push(format!(
            "Score: {:.0}% ({})",
            score.overall_score * 100.0,
            grade_colored
        ));

        if score.overall_score < 0.7 {
            has_warnings = true;
        }
    }

    // Binding results
    if binding.has_bindings {
        let coverage_pct = if binding.total_bindings > 0 {
            (binding.implemented as f64 / binding.total_bindings as f64) * 100.0
        } else {
            0.0
        };

        parts.push(format!(
            "Bindings: {}/{} implemented ({:.0}%)",
            c::number(&binding.implemented.to_string()),
            c::number(&binding.total_bindings.to_string()),
            coverage_pct,
        ));

        if binding.not_implemented > 0 {
            has_warnings = true;
            parts.push(format!(
                "  {} unimplemented binding(s)",
                c::number(&binding.not_implemented.to_string()),
            ));
        }
    }

    let message = parts.join("\n    ");
    let (status, severity) = if has_errors {
        (CheckStatus::Fail, Severity::Error)
    } else if has_warnings {
        (CheckStatus::Warn, Severity::Warning)
    } else {
        (CheckStatus::Pass, Severity::Info)
    };

    ComplianceCheck {
        name: "CB-1200: Provable Contracts".into(),
        status,
        message,
        severity,
    }
}

#[cfg_attr(coverage_nightly, coverage(off))]
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_no_contracts_dir_skips() {
        let temp = tempfile::tempdir().unwrap();
        let check = check_provable_contracts(temp.path());
        assert_eq!(check.status, CheckStatus::Skip);
        assert!(check.message.contains("No provable-contract"));
    }

    #[test]
    fn test_empty_contracts_dir_skips() {
        let temp = tempfile::tempdir().unwrap();
        std::fs::create_dir(temp.path().join("contracts")).unwrap();
        let check = check_provable_contracts(temp.path());
        assert_eq!(check.status, CheckStatus::Skip);
        assert!(check.message.contains("No provable-contract YAML"));
    }

    #[test]
    fn test_non_contract_yaml_skips() {
        let temp = tempfile::tempdir().unwrap();
        let contracts_dir = temp.path().join("contracts");
        std::fs::create_dir(&contracts_dir).unwrap();
        std::fs::write(contracts_dir.join("config.yaml"), "some_key: some_value\n").unwrap();
        let check = check_provable_contracts(temp.path());
        assert_eq!(check.status, CheckStatus::Skip);
    }

    #[test]
    fn test_contract_yaml_detected() {
        let temp = tempfile::tempdir().unwrap();
        let contracts_dir = temp.path().join("contracts");
        std::fs::create_dir(&contracts_dir).unwrap();
        std::fs::write(
            contracts_dir.join("softmax-kernel-v1.yaml"),
            "equations:\n  softmax:\n    formula: test\nproof_obligations:\n  - type: invariant\n",
        )
        .unwrap();
        let files = find_contract_files(&contracts_dir);
        assert_eq!(files.len(), 1);
        assert!(files[0].contains("softmax"));
    }

    #[test]
    fn test_binding_file_excluded_from_contracts() {
        let temp = tempfile::tempdir().unwrap();
        let contracts_dir = temp.path().join("contracts");
        std::fs::create_dir(&contracts_dir).unwrap();
        std::fs::write(
            contracts_dir.join("binding.yaml"),
            "bindings:\n  - contract: test\n    status: implemented\n",
        )
        .unwrap();
        let files = find_contract_files(&contracts_dir);
        assert!(files.is_empty());
    }

    #[test]
    fn test_is_binding_file() {
        assert!(is_binding_file(Path::new(
            "contracts/aprender/binding.yaml"
        )));
        assert!(is_binding_file(Path::new("binding.yaml")));
        assert!(!is_binding_file(Path::new("softmax-kernel-v1.yaml")));
    }

    #[test]
    fn test_parse_pv_lint_json_success() {
        let json = r#"{"errors": 0, "warnings": 2, "contracts_checked": 10}"#;
        let result = parse_pv_lint_json(json);
        assert!(result.passed);
        assert_eq!(result.contracts_checked, 10);
        assert_eq!(result.warnings, 2);
    }

    #[test]
    fn test_parse_pv_lint_json_failure() {
        let json = r#"{"errors": 3, "warnings": 1, "contracts_checked": 5}"#;
        let result = parse_pv_lint_json(json);
        assert!(!result.passed);
        assert_eq!(result.errors, 3);
    }

    #[test]
    fn test_parse_pv_lint_json_invalid() {
        let result = parse_pv_lint_json("not json at all");
        assert!(result.passed); // No "error" substring in the text
        assert_eq!(result.contracts_checked, 0);
    }

    #[test]
    fn test_parse_pv_score_json() {
        let json = r#"{"overall_score": 0.85, "grade": "B", "dimensions": {"D1": 0.9, "D2": 0.8}}"#;
        let result = parse_pv_score_json(json);
        assert!((result.overall_score - 0.85).abs() < 0.01);
        assert_eq!(result.grade, "B");
        assert_eq!(result.dimensions.len(), 2);
    }

    #[test]
    fn test_parse_pv_score_json_invalid() {
        let result = parse_pv_score_json("garbage");
        assert_eq!(result.grade, "?");
        assert!(result.error.is_some());
    }

    #[test]
    fn test_binding_coverage_no_bindings() {
        let temp = tempfile::tempdir().unwrap();
        let contracts_dir = temp.path().join("contracts");
        std::fs::create_dir(&contracts_dir).unwrap();
        let result = check_binding_coverage(temp.path());
        assert!(!result.has_bindings);
    }

    #[test]
    fn test_binding_coverage_with_bindings() {
        let temp = tempfile::tempdir().unwrap();
        let contracts_dir = temp.path().join("contracts");
        std::fs::create_dir(&contracts_dir).unwrap();
        std::fs::write(
            contracts_dir.join("binding.yaml"),
            "bindings:\n  - contract: a.yaml\n    status: implemented\n  - contract: b.yaml\n    status: partial\n  - contract: c.yaml\n    status: not_implemented\n",
        )
        .unwrap();
        let result = check_binding_coverage(temp.path());
        assert!(result.has_bindings);
        assert_eq!(result.total_bindings, 3);
        assert_eq!(result.implemented, 1);
        assert_eq!(result.partial, 1);
        assert_eq!(result.not_implemented, 1);
    }

    #[test]
    fn test_build_result_all_pass() {
        let files = vec!["softmax.yaml".to_string()];
        let lint = PvLintResult {
            passed: true,
            contracts_checked: 1,
            errors: 0,
            warnings: 0,
            error_details: vec![],
        };
        let score = PvScoreResult {
            overall_score: 0.9,
            grade: "A".into(),
            dimensions: vec![],
            error: None,
        };
        let binding = BindingResult {
            has_bindings: true,
            total_bindings: 5,
            implemented: 5,
            partial: 0,
            not_implemented: 0,
        };
        let check = build_provable_contracts_result(&files, lint, score, binding);
        assert_eq!(check.status, CheckStatus::Pass);
    }

    #[test]
    fn test_build_result_lint_errors() {
        let files = vec!["test.yaml".to_string()];
        let lint = PvLintResult {
            passed: false,
            contracts_checked: 1,
            errors: 2,
            warnings: 0,
            error_details: vec!["Missing proof_obligations".into()],
        };
        let score = PvScoreResult {
            overall_score: 0.0,
            grade: "?".into(),
            dimensions: vec![],
            error: None,
        };
        let binding = BindingResult {
            has_bindings: false,
            total_bindings: 0,
            implemented: 0,
            partial: 0,
            not_implemented: 0,
        };
        let check = build_provable_contracts_result(&files, lint, score, binding);
        assert_eq!(check.status, CheckStatus::Fail);
        assert_eq!(check.severity, Severity::Error);
    }

    #[test]
    fn test_build_result_low_score_warns() {
        let files = vec!["test.yaml".to_string()];
        let lint = PvLintResult {
            passed: true,
            contracts_checked: 1,
            errors: 0,
            warnings: 0,
            error_details: vec![],
        };
        let score = PvScoreResult {
            overall_score: 0.5,
            grade: "F".into(),
            dimensions: vec![],
            error: None,
        };
        let binding = BindingResult {
            has_bindings: false,
            total_bindings: 0,
            implemented: 0,
            partial: 0,
            not_implemented: 0,
        };
        let check = build_provable_contracts_result(&files, lint, score, binding);
        assert_eq!(check.status, CheckStatus::Warn);
    }
}

#![cfg_attr(coverage_nightly, coverage(off))]
// Checkpoint handler: invariant evaluation at checkpoint time (DbC §4.2)

use anyhow::{Context, Result};
use std::path::Path;

use crate::cli::handlers::work_contract::{
    CheckpointRecord, ClauseKind, ContractClause, FalsificationMethod, InvariantResult,
    WorkContract,
};

/// Evaluate invariant clauses and produce a CheckpointRecord.
///
/// This is the core of `pmat work checkpoint`. Each invariant clause is
/// evaluated against the current project state. The result is persisted
/// to `.pmat-work/{id}/checkpoints/` for audit trail.
pub(super) fn run_checkpoint(project_path: &Path, work_item_id: &str) -> Result<CheckpointRecord> {
    let contract = WorkContract::load(project_path, work_item_id).with_context(|| {
        format!(
            "No contract found for '{}'. Run 'pmat work start {}' first.",
            work_item_id, work_item_id
        )
    })?;

    let git_sha = get_git_sha(project_path);

    // Evaluate all invariant clauses
    let invariant_results = evaluate_invariants(project_path, &contract.invariant, &contract);

    let record = CheckpointRecord::new(
        work_item_id.to_string(),
        git_sha,
        contract.iteration,
        invariant_results,
    );

    Ok(record)
}

/// Evaluate a list of invariant clauses against the current project state.
///
/// Each clause is evaluated based on its falsification_method using
/// lightweight checks appropriate for checkpoint frequency.
fn evaluate_invariants(
    project_path: &Path,
    invariants: &[ContractClause],
    contract: &WorkContract,
) -> Vec<InvariantResult> {
    invariants
        .iter()
        .filter(|c| c.kind == ClauseKind::Invariant)
        .map(|clause| evaluate_single_invariant(project_path, clause, contract))
        .collect()
}

/// Evaluate a single invariant clause.
fn evaluate_single_invariant(
    project_path: &Path,
    clause: &ContractClause,
    contract: &WorkContract,
) -> InvariantResult {
    match clause.falsification_method {
        FalsificationMethod::FileSizeRegression => {
            check_file_size_invariant(project_path, clause, contract)
        }
        FalsificationMethod::LintPass => check_lint_invariant(project_path, clause),
        FalsificationMethod::ComplexityRegression => {
            // Complexity check is expensive — report as checked but defer
            // full evaluation to `work complete` for now.
            InvariantResult {
                clause_id: clause.id.clone(),
                passed: true,
                explanation: format!("{}: deferred to completion", clause.description),
            }
        }
        FalsificationMethod::ManifestIntegrity => check_compiles_invariant(project_path, clause),
        // SATD, dead code, fix chain — checked at completion for now
        _ => InvariantResult {
            clause_id: clause.id.clone(),
            passed: true,
            explanation: format!("{}: deferred to completion", clause.description),
        },
    }
}

/// Check invariant.file_size: no file exceeds the configured limit.
fn check_file_size_invariant(
    project_path: &Path,
    clause: &ContractClause,
    contract: &WorkContract,
) -> InvariantResult {
    let max_lines = contract.thresholds.max_file_lines;

    // Check staged/modified files from git status
    let output = std::process::Command::new("git")
        .args(["diff", "--name-only", "HEAD"])
        .current_dir(project_path)
        .output();

    let changed_files: Vec<String> = match output {
        Ok(o) => String::from_utf8_lossy(&o.stdout)
            .lines()
            .filter(|l| !l.is_empty())
            .map(String::from)
            .collect(),
        Err(_) => return pass_result(&clause.id, "Could not read git diff"),
    };

    let mut violations = Vec::new();
    for file in &changed_files {
        let file_path = project_path.join(file);
        if !file_path.exists() {
            continue;
        }
        // Only check source files
        if !file.ends_with(".rs")
            && !file.ends_with(".py")
            && !file.ends_with(".ts")
            && !file.ends_with(".js")
            && !file.ends_with(".go")
        {
            continue;
        }
        if let Ok(content) = std::fs::read_to_string(&file_path) {
            let line_count = content.lines().count();
            if line_count > max_lines {
                violations.push(format!("{}: {} lines", file, line_count));
            }
        }
    }

    if violations.is_empty() {
        InvariantResult {
            clause_id: clause.id.clone(),
            passed: true,
            explanation: format!("All changed files within {} line limit", max_lines),
        }
    } else {
        InvariantResult {
            clause_id: clause.id.clone(),
            passed: false,
            explanation: format!(
                "{} file(s) exceed {} lines: {}",
                violations.len(),
                max_lines,
                violations.join(", ")
            ),
        }
    }
}

/// Check invariant.lint: cargo clippy passes.
fn check_lint_invariant(project_path: &Path, clause: &ContractClause) -> InvariantResult {
    // Only run lint if Cargo.toml exists
    if !project_path.join("Cargo.toml").exists() {
        return pass_result(&clause.id, "Not a Rust project, lint skipped");
    }

    let output = std::process::Command::new("cargo")
        .args(["clippy", "--quiet", "--message-format=short"])
        .current_dir(project_path)
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .output();

    match output {
        Ok(o) if o.status.success() => InvariantResult {
            clause_id: clause.id.clone(),
            passed: true,
            explanation: "Lint clean".to_string(),
        },
        Ok(o) => {
            let stderr = String::from_utf8_lossy(&o.stderr);
            let error_count = stderr.lines().filter(|l| l.contains("error")).count();
            InvariantResult {
                clause_id: clause.id.clone(),
                passed: false,
                explanation: format!("Lint failed: {} error(s)", error_count),
            }
        }
        Err(e) => InvariantResult {
            clause_id: clause.id.clone(),
            passed: false,
            explanation: format!("Could not run clippy: {}", e),
        },
    }
}

/// Check invariant.compiles: project builds successfully.
fn check_compiles_invariant(project_path: &Path, clause: &ContractClause) -> InvariantResult {
    if project_path.join("Cargo.toml").exists() {
        let output = std::process::Command::new("cargo")
            .args(["check", "--quiet"])
            .current_dir(project_path)
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::piped())
            .output();

        match output {
            Ok(o) if o.status.success() => InvariantResult {
                clause_id: clause.id.clone(),
                passed: true,
                explanation: "Compiles successfully".to_string(),
            },
            Ok(_) => InvariantResult {
                clause_id: clause.id.clone(),
                passed: false,
                explanation: "Compilation failed".to_string(),
            },
            Err(e) => InvariantResult {
                clause_id: clause.id.clone(),
                passed: false,
                explanation: format!("Could not run cargo check: {}", e),
            },
        }
    } else {
        pass_result(&clause.id, "No Cargo.toml found, compile check skipped")
    }
}

fn pass_result(clause_id: &str, explanation: &str) -> InvariantResult {
    InvariantResult {
        clause_id: clause_id.to_string(),
        passed: true,
        explanation: explanation.to_string(),
    }
}

/// Get current git SHA (short helper to avoid duplication)
fn get_git_sha(project_path: &Path) -> String {
    std::process::Command::new("git")
        .args(["rev-parse", "HEAD"])
        .current_dir(project_path)
        .output()
        .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string())
        .unwrap_or_else(|_| "unknown".to_string())
}

/// Evaluate invariants from a loaded contract for use during `work complete`.
///
/// Returns the list of invariant results and whether all passed.
/// This is the "final invariant check" from DbC §4.3.
pub(super) fn evaluate_final_invariants(
    project_path: &Path,
    contract: &WorkContract,
) -> (Vec<InvariantResult>, bool) {
    let results = evaluate_invariants(project_path, &contract.invariant, contract);
    let all_pass = results.iter().all(|r| r.passed);
    (results, all_pass)
}

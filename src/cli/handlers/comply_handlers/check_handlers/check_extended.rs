// Extended compliance checks - CB-300+, CB-081, file health, sovereign stack, PAIML deps
//
// Originally from check_handlers.rs and migrate_handlers.rs,
// contains the more specialized compliance check functions.

use crate::cli::handlers::comply_cb_detect::{
    detect_cb081_dependency_count, DependencyCountReport,
};
use crate::services::file_health::{
    scan_directory, FileHealthMetrics, FileHealthReport, DEFAULT_EXCLUDE_PATTERNS, RUST_EXTENSIONS,
};
use anyhow::Result;
use std::path::Path;

use super::types::*;

/// CB-300: Muda Waste Score (COMPLY-040)
#[provable_contracts_macros::contract("pmat-core.yaml", equation = "path_exists")]
pub(crate) fn check_muda_waste_score(project_path: &Path) -> ComplianceCheck {
    use crate::cli::handlers::comply_handlers::muda_handlers;
    let report = muda_handlers::calculate_muda_score(project_path);
    let mut message = format!(
        "Muda Score: {:.1}/100 ({}) - Over:{:.0} Wait:{:.0} Inv:{:.0} Proc:{:.0} Def:{:.0}",
        report.total_score,
        report.grade,
        report.overproduction,
        report.waiting,
        report.inventory,
        report.over_processing,
        report.defects
    );

    // Append file-level details for non-zero categories
    for (category, files) in &report.file_details {
        if !files.is_empty() {
            message.push_str(&format!("\n    {}: {}", category, files.join(", ")));
        }
    }

    let (status, severity) = match report.grade {
        muda_handlers::MudaGrade::Lean | muda_handlers::MudaGrade::Efficient => {
            (CheckStatus::Pass, Severity::Info)
        }
        muda_handlers::MudaGrade::Moderate => (CheckStatus::Warn, Severity::Warning),
        muda_handlers::MudaGrade::High | muda_handlers::MudaGrade::Critical => {
            (CheckStatus::Fail, Severity::Error)
        }
    };
    ComplianceCheck {
        name: "CB-300: Muda Waste Score".into(),
        status,
        message,
        severity,
    }
}

/// CB-301: Reproducibility Level Check (COMPLY-041)
#[provable_contracts_macros::contract("pmat-core.yaml", equation = "path_exists")]
pub(crate) fn check_reproducibility_level(project_path: &Path) -> ComplianceCheck {
    use crate::cli::handlers::comply_handlers::reproducibility_handlers;
    let report = reproducibility_handlers::check_reproducibility(project_path);
    let detail_summary: String = report
        .details
        .iter()
        .take(3)
        .cloned()
        .collect::<Vec<_>>()
        .join("; ");
    let message = format!("Reproducibility: {} - {}", report.level, detail_summary);
    let (status, severity) = match report.level {
        reproducibility_handlers::ReproducibilityLevel::Gold
        | reproducibility_handlers::ReproducibilityLevel::Silver => {
            (CheckStatus::Pass, Severity::Info)
        }
        reproducibility_handlers::ReproducibilityLevel::Bronze => {
            (CheckStatus::Warn, Severity::Warning)
        }
        reproducibility_handlers::ReproducibilityLevel::None => {
            (CheckStatus::Fail, Severity::Error)
        }
    };
    ComplianceCheck {
        name: "CB-301: Reproducibility Level".into(),
        status,
        message,
        severity,
    }
}

/// CB-302: Golden Trace Drift Detection (COMPLY-042)
#[provable_contracts_macros::contract("pmat-core.yaml", equation = "path_exists")]
pub(crate) fn check_golden_trace_drift(project_path: &Path) -> ComplianceCheck {
    use crate::cli::handlers::comply_handlers::reproducibility_handlers;
    match reproducibility_handlers::check_golden_trace_drift(project_path) {
        None => ComplianceCheck {
            name: "CB-302: Golden Trace Drift".into(),
            status: CheckStatus::Skip,
            message: "No renacer.toml configured - golden tracing not enabled".into(),
            severity: Severity::Info,
        },
        Some(true) => ComplianceCheck {
            name: "CB-302: Golden Trace Drift".into(),
            status: CheckStatus::Pass,
            message: "Golden traces valid - no drift detected".into(),
            severity: Severity::Info,
        },
        Some(false) => ComplianceCheck {
            name: "CB-302: Golden Trace Drift".into(),
            status: CheckStatus::Fail,
            message: "Golden trace drift detected - run 'renacer validate' to investigate".into(),
            severity: Severity::Error,
        },
    }
}

/// CB-303: Equation-Driven Development Compliance (COMPLY-043)
#[provable_contracts_macros::contract("pmat-core.yaml", equation = "path_exists")]
pub(crate) fn check_edd_compliance(project_path: &Path) -> ComplianceCheck {
    use crate::cli::handlers::comply_handlers::edd_handlers;
    let report = edd_handlers::check_edd_compliance(project_path);
    if !report.is_simulation_project {
        return ComplianceCheck {
            name: "CB-303: EDD Compliance".into(),
            status: CheckStatus::Skip,
            message: "Not a simulation project (no simular/trueno-sim dependency)".into(),
            severity: Severity::Info,
        };
    }
    let violation_count = report.undocumented_fns.len();
    let message = format!(
        "EDD: {:.0}% ({}/{} pub fns documented with math)",
        report.compliance_pct, report.documented_fns, report.total_simulation_fns
    );
    if report.compliance_pct >= 80.0 {
        ComplianceCheck {
            name: "CB-303: EDD Compliance".into(),
            status: CheckStatus::Pass,
            message,
            severity: Severity::Info,
        }
    } else if violation_count > 0 {
        ComplianceCheck {
            name: "CB-303: EDD Compliance".into(),
            status: CheckStatus::Warn,
            message: format!(
                "{} - {} functions missing mathematical models",
                message, violation_count
            ),
            severity: Severity::Warning,
        }
    } else {
        ComplianceCheck {
            name: "CB-303: EDD Compliance".into(),
            status: CheckStatus::Pass,
            message,
            severity: Severity::Info,
        }
    }
}

/// CB-304: Dead Code Percentage
#[provable_contracts_macros::contract("pmat-core.yaml", equation = "path_exists")]
pub(crate) fn check_dead_code_percentage(project_path: &Path) -> ComplianceCheck {
    let config = crate::models::deep_context_config::DeepContextConfig::default();
    let threshold_pct = config.dead_code_threshold * 100.0;
    let source_dirs: Vec<std::path::PathBuf> = ["src", "crates", "lean", "lib"]
        .iter()
        .map(|d| project_path.join(d))
        .filter(|d| d.exists() && d.is_dir())
        .collect();
    if source_dirs.is_empty() {
        return ComplianceCheck {
            name: "CB-304: Dead Code Percentage".into(),
            status: CheckStatus::Skip,
            message: "No source directory found (checked src/, crates/, lean/, lib/)".into(),
            severity: Severity::Info,
        };
    }
    let (mut total_items, mut dead_items, mut total_lines, mut dead_lines) = (0, 0, 0, 0);
    for src_dir in &source_dirs {
        let (ti, di, tl, dl) = super::check_dead_code::scan_dead_code_indicators(src_dir);
        total_items += ti;
        dead_items += di;
        total_lines += tl;
        dead_lines += dl;
    }
    if total_items == 0 {
        return ComplianceCheck {
            name: "CB-304: Dead Code Percentage".into(),
            status: CheckStatus::Pass,
            message: "No code items found to analyze".into(),
            severity: Severity::Info,
        };
    }
    let dead_pct = if total_lines > 0 && dead_lines > 0 {
        (dead_lines as f64 / total_lines as f64) * 100.0
    } else {
        (dead_items as f64 / total_items as f64) * 100.0
    };
    let message = format!(
        "Dead code: {:.1}% ({} dead items/{} total, ~{} dead lines/{} total) [threshold: {:.0}%]",
        dead_pct, dead_items, total_items, dead_lines, total_lines, threshold_pct
    );
    if dead_pct <= threshold_pct {
        ComplianceCheck {
            name: "CB-304: Dead Code Percentage".into(),
            status: CheckStatus::Pass,
            message,
            severity: Severity::Info,
        }
    } else if dead_pct <= threshold_pct * 2.0 {
        ComplianceCheck {
            name: "CB-304: Dead Code Percentage".into(),
            status: CheckStatus::Warn,
            message: format!("{message} - exceeds threshold"),
            severity: Severity::Warning,
        }
    } else {
        ComplianceCheck {
            name: "CB-304: Dead Code Percentage".into(),
            status: CheckStatus::Fail,
            message: format!("{message} - significantly exceeds threshold"),
            severity: Severity::Error,
        }
    }
}

fn format_dependency_message(report: &DependencyCountReport) -> String {
    let transitive_display = if let Some(prod) = report.prod_transitive_count {
        format!(
            "{} prod transitive ({} total w/dev)",
            prod, report.transitive_count
        )
    } else {
        format!("{} transitive", report.transitive_count)
    };
    let mut details = vec![format!(
        "{} direct, {}",
        report.direct_count, transitive_display
    )];
    if let Some(ref trend) = report.trend {
        if trend.direct_delta != 0 || trend.transitive_delta != 0 {
            details.push(format!(
                "\u{0394} {:+}/{:+} since last",
                trend.direct_delta, trend.transitive_delta
            ));
        }
    }
    if !report.duplicate_crates.is_empty() {
        details.push(format!("{} duplicates", report.duplicate_crates.len()));
    }
    details.push(format!("{:.0}% feature-gated", report.feature_gated_pct));
    if report.sovereign_bonus > 0 {
        details.push(format!(
            "+{} sovereign ({})",
            report.sovereign_bonus,
            report.sovereign_crates.join(", ")
        ));
    }
    format!("Score: {}/5 | {}", report.score, details.join(" | "))
}

fn append_violation_details(msg: &mut String, report: &DependencyCountReport, limit: usize) {
    for v in report.violations.iter().take(limit) {
        let icon = if v.severity == crate::cli::handlers::comply_cb_detect::Severity::Error {
            "\u{2717}"
        } else {
            "\u{26a0}"
        };
        msg.push_str(&format!("\n    {} {}", icon, v.description));
    }
}

/// CB-081: Dependency Count Check
///
/// Refuses to run when the project has no `Cargo.lock`. The transitive count
/// comes from `cargo tree`, and cargo RESOLVES the dependency graph to answer
/// it — which writes a `Cargo.lock` into a tree that had none. That is how
/// `comply check` came to satisfy its own findings: run 1 reported
/// "Cargo.lock Present: Fail" and "CB-301: Reproducibility: None — Fail",
/// wrote the lockfile as a side effect of measuring, and run 2 of the identical
/// command on byte-identical source reported 0 failures and exit 2 instead of 1
/// (#939). An auditor may not create the artifact it is auditing, so an
/// unresolved project is reported as unmeasured, not as healthy.
#[provable_contracts_macros::contract("pmat-core.yaml", equation = "path_exists")]
pub(crate) fn check_dependency_count(project_path: &Path) -> ComplianceCheck {
    if !project_path.join("Cargo.toml").exists() {
        return ComplianceCheck {
            name: "CB-081: Dependency Health".into(),
            status: CheckStatus::Skip,
            message: "Not a Rust project (no Cargo.toml)".into(),
            severity: Severity::Info,
        };
    }
    if !project_path.join("Cargo.lock").exists() {
        return ComplianceCheck {
            name: "CB-081: Dependency Health".into(),
            status: CheckStatus::Skip,
            message: "Not measured: no Cargo.lock, and resolving the dependency graph would \
                      write one into the audited project - run 'cargo generate-lockfile' first"
                .into(),
            severity: Severity::Info,
        };
    }
    let report = detect_cb081_dependency_count(project_path);
    let message = format_dependency_message(&report);
    let has_critical = report
        .violations
        .iter()
        .any(|v| v.severity == crate::cli::handlers::comply_cb_detect::Severity::Error);
    if report.score >= 4 && !has_critical {
        ComplianceCheck {
            name: "CB-081: Dependency Health".into(),
            status: CheckStatus::Pass,
            message,
            severity: Severity::Info,
        }
    } else if report.score >= 2 && !has_critical {
        let mut msg = message;
        append_violation_details(&mut msg, &report, 1);
        ComplianceCheck {
            name: "CB-081: Dependency Health".into(),
            status: CheckStatus::Warn,
            message: msg,
            severity: Severity::Warning,
        }
    } else {
        let mut msg = message;
        append_violation_details(&mut msg, &report, 3);
        ComplianceCheck {
            name: "CB-081: Dependency Health".into(),
            status: CheckStatus::Fail,
            message: msg,
            severity: Severity::Error,
        }
    }
}

/// CB-081-F: a workspace member pulled from crates.io by a sibling.
///
/// Split out of CB-081's duplicate COUNT because the remedy is different and
/// unconditional. CB-081-B reports every crate resolving to more than one
/// version as one undifferentiated list, and that list is dominated by
/// third-party major conflicts a project often cannot fix — so the subset it
/// can always fix is invisible inside it.
///
/// paiml/aprender (78 crates) resolved `trueno` at 0.16, 0.16.5 and the in-tree
/// 0.63.0 at once — three compilations of the SIMD kernels in one binary —
/// while `jugar-probar` spanned seven declared versions. `comply check` did see
/// it: `trueno` sat inside "176 duplicate crates: codespan-reporting, syn,
/// schemars, …" under "run cargo tree --duplicates". The signal was there and
/// the severity was not (#989).
///
/// Skipped, not passed, on a non-workspace: a single crate has no members, so
/// there is nothing to measure and nothing to claim.
#[provable_contracts_macros::contract("pmat-core.yaml", equation = "path_exists")]
pub(crate) fn check_workspace_member_registry_deps(project_path: &Path) -> ComplianceCheck {
    let name = "CB-081-F: Workspace Member From Registry";
    if !project_path.join("Cargo.toml").exists() {
        return ComplianceCheck {
            name: name.into(),
            status: CheckStatus::Skip,
            message: "Not a Rust project (no Cargo.toml)".into(),
            severity: Severity::Info,
        };
    }
    let members = crate::cli::handlers::comply_cb_detect::workspace_member_count(project_path);
    if members == 0 {
        return ComplianceCheck {
            name: name.into(),
            status: CheckStatus::Skip,
            message: "Not a workspace: no members to pull from the registry".into(),
            severity: Severity::Info,
        };
    }
    let found = crate::cli::handlers::comply_cb_detect::detect_workspace_members_from_registry(
        project_path,
    );
    if found.is_empty() {
        return ComplianceCheck {
            name: name.into(),
            status: CheckStatus::Pass,
            message: format!(
                "{members} workspace member(s); every sibling dependency uses a path or \
                 workspace inheritance"
            ),
            severity: Severity::Info,
        };
    }
    ComplianceCheck {
        name: name.into(),
        status: CheckStatus::Fail,
        message: crate::cli::handlers::comply_cb_detect::format_registry_member_deps(&found),
        severity: Severity::Error,
    }
}

/// Detect project type and discover source files across all source directories.
#[provable_contracts_macros::contract("pmat-core.yaml", equation = "path_exists")]
pub(crate) fn discover_source_files(
    project_path: &Path,
) -> Result<Vec<std::path::PathBuf>, String> {
    let is_rust = project_path.join("Cargo.toml").exists();
    let is_lean = project_path.join("lakefile.lean").exists()
        || project_path.join("lean-toolchain").exists()
        || project_path.join("lean").join("lakefile.lean").exists()
        || project_path.join("lean").join("lean-toolchain").exists();
    if !is_rust && !is_lean {
        return Err("No Cargo.toml or lakefile.lean found".into());
    }
    let source_dirs: Vec<std::path::PathBuf> = ["src", "crates", "lean", "lib"]
        .iter()
        .map(|d| project_path.join(d))
        .filter(|d| d.exists() && d.is_dir())
        .collect();
    if source_dirs.is_empty() {
        return Err("No source directory found (checked src/, crates/, lean/, lib/)".into());
    }
    let extensions: &[&str] = if is_lean && !is_rust {
        &["lean"]
    } else if is_rust && is_lean {
        &["rs", "lean"]
    } else {
        RUST_EXTENSIONS
    };
    let mut files = Vec::new();
    for src_dir in &source_dirs {
        files.extend(scan_directory(
            src_dir,
            extensions,
            DEFAULT_EXCLUDE_PATTERNS,
        ));
    }
    // #986: the project's own excludes apply HERE, not at one call site.
    // `discover_source_files` has three callers and all three are file health
    // — the CB-040 check, `--include-project` cross-stack health, and the
    // ratchet baseline. Only the first honoured the project's configured
    // excludes, so the same generated file was excluded from the check that
    // fails the build and included in the baseline the check ratchets against.
    // One rule, one place.
    let configured = load_file_health_excludes(project_path);
    if !configured.is_empty() {
        files.retain(|f| !matches_exclude_pattern(f, &configured));
    }
    Ok(files)
}

/// Check file health across the project (CB-040)
/// Analyze a single file for health metrics, returning (critical, problem, over_500) increments
fn classify_file_health(file_path: &std::path::PathBuf, content: &str) -> (usize, usize, usize) {
    let lines = content.lines().count();
    let is_test_file = file_path.to_string_lossy().contains("/tests/")
        || file_path
            .file_name()
            .map(|f| {
                let name = f.to_string_lossy();
                name.starts_with("test") || name.ends_with("_tests.rs")
            })
            .unwrap_or(false);
    let critical_threshold = if is_test_file { 4000 } else { 2000 };
    let problem_threshold = if is_test_file { 2000 } else { 1000 };
    let critical = usize::from(lines > critical_threshold);
    let problem = usize::from(lines <= critical_threshold && lines > problem_threshold);
    let over_500 = usize::from(lines > 500);
    (critical, problem, over_500)
}

/// Build the final ComplianceCheck from file health counts
fn build_file_health_check(
    report: &FileHealthReport,
    critical_count: usize,
    problem_count: usize,
    over_500_count: usize,
) -> ComplianceCheck {
    if critical_count > 0 {
        ComplianceCheck { name: "CB-040: File Health".into(), status: CheckStatus::Fail, message: format!("CRITICAL: {} files >2000 lines, {} files >1000 lines, {} files >500 lines (avg health: {}%, grade: {})", critical_count, problem_count, over_500_count, report.average_health, report.average_grade.as_str()), severity: Severity::Critical }
    } else if problem_count > 0 || over_500_count > 5 {
        ComplianceCheck {
            name: "CB-040: File Health".into(),
            status: CheckStatus::Warn,
            message: format!(
                "{} files >1000 lines, {} files >500 lines (avg health: {}%, grade: {})",
                problem_count,
                over_500_count,
                report.average_health,
                report.average_grade.as_str()
            ),
            severity: Severity::Warning,
        }
    } else {
        ComplianceCheck { name: "CB-040: File Health".into(), status: CheckStatus::Pass, message: format!("{} files analyzed, avg health: {}%, grade: {} (all files <500 lines or within tolerance)", report.total_files, report.average_health, report.average_grade.as_str()), severity: Severity::Info }
    }
}

#[provable_contracts_macros::contract("pmat-core.yaml", equation = "path_exists")]
/// GH-292: Load file health exclude patterns. Merges entries from
/// `.pmat-gates.toml [file_health] exclude` and
/// `.pmat.yaml comply.thresholds.file_health_exclude`. Either source alone
/// is sufficient; both may coexist.
fn load_file_health_excludes(project_path: &Path) -> Vec<String> {
    let mut out = load_file_health_excludes_from_gates(project_path);
    let yaml = crate::models::comply_config::PmatYamlConfig::load(project_path).unwrap_or_default();
    for pat in &yaml.comply.thresholds.file_health_exclude {
        if !out.iter().any(|p| p == pat) {
            out.push(pat.clone());
        }
    }
    out
}

fn load_file_health_excludes_from_gates(project_path: &Path) -> Vec<String> {
    let toml_path = project_path.join(".pmat-gates.toml");
    let content = match std::fs::read_to_string(&toml_path) {
        Ok(c) => c,
        Err(_) => return Vec::new(),
    };
    let table: toml::Table = match content.parse() {
        Ok(t) => t,
        Err(_) => return Vec::new(),
    };
    // Read the general exclude keys as well as the File-Health-specific one.
    //
    // #986: a project that had configured `.pmat-gates.toml [exclude] paths`
    // — the key `quality_gate_config.rs::extract_excludes_from_table` honours —
    // found File Health reporting CRITICAL on generated files it had already
    // excluded. Both readers parse the SAME file; they simply looked at
    // different keys, so the config was honoured or ignored depending on which
    // check you happened to run. One file, one meaning.
    //
    // Union, never replace: `[file_health] exclude` stays authoritative for
    // anyone already using it, and the general keys now apply too.
    let mut out: Vec<String> = table
        .get("file_health")
        .and_then(|fh| fh.get("exclude"))
        .and_then(|v| v.as_array())
        .map(|a| {
            a.iter()
                .filter_map(|v| v.as_str().map(String::from))
                .collect()
        })
        .unwrap_or_default();

    let general = table
        .get("exclude")
        .and_then(|t| t.get("paths"))
        .and_then(|v| v.as_array())
        .or_else(|| table.get("exclude_paths").and_then(|v| v.as_array()))
        .or_else(|| {
            table
                .get("quality-gates")
                .and_then(|t| t.get("exclude"))
                .and_then(|v| v.as_array())
        });
    if let Some(arr) = general {
        for pat in arr.iter().filter_map(|v| v.as_str()) {
            if !out.iter().any(|p| p == pat) {
                out.push(pat.to_string());
            }
        }
    }
    warn_on_unread_exclude_keys(&table, &toml_path);
    out
}

/// Every recognised place an exclude list may live in `.pmat-gates.toml`.
const GATES_EXCLUDE_KEYS: [&str; 4] = [
    "[exclude] paths",
    "exclude_paths",
    "[quality-gates] exclude",
    "[file_health] exclude",
];

/// Say so when a table that exists only to hold excludes holds none we read.
///
/// #986, second half: a project wrote
///
/// ```toml
/// [exclude]
/// patterns = ["…"]
/// ```
///
/// `patterns` is not one of the keys any reader looks at. The file parsed, the
/// run was green, and three exclusions did nothing for as long as the file had
/// existed — configuration absent, rendered as configuration honoured. Nothing
/// in the tree could tell the author, because a key nobody reads is
/// indistinguishable from a key that matched no file. It is distinguishable at
/// the point of reading, so that is where it is now said.
fn warn_on_unread_exclude_keys(table: &toml::Table, toml_path: &Path) {
    let mut unread: Vec<String> = Vec::new();
    if let Some(t) = table.get("exclude").and_then(toml::Value::as_table) {
        for key in t.keys() {
            if key != "paths" {
                unread.push(format!("[exclude] {key}"));
            }
        }
    }
    if let Some(t) = table.get("file_health").and_then(toml::Value::as_table) {
        for key in t.keys() {
            if key != "exclude" {
                unread.push(format!("[file_health] {key}"));
            }
        }
    }
    if unread.is_empty() {
        return;
    }
    eprintln!(
        "warning: {}: {} is not read by any pmat check and has no effect. \
         Exclusions are read from: {}.",
        toml_path.display(),
        unread.join(", "),
        GATES_EXCLUDE_KEYS.join(", ")
    );
}

/// Check if a file path matches any exclude pattern (glob-style)
fn matches_exclude_pattern(file_path: &std::path::PathBuf, patterns: &[String]) -> bool {
    let path_str = file_path.to_string_lossy();
    let file_name = file_path.file_name().and_then(|n| n.to_str()).unwrap_or("");
    for pattern in patterns {
        if let Some(suffix) = pattern.strip_prefix("**/") {
            // ** prefix: match filename OR any path containing the suffix
            if suffix.contains('/') {
                // e.g. "**/examples/**" — check if path contains the segment
                let segment = suffix.trim_end_matches("/**").trim_end_matches("/*");
                if path_str.contains(segment) {
                    return true;
                }
            } else if suffix.contains('*') {
                // e.g. "**/*_tests.rs" — glob match on filename
                if glob_match_simple(file_name, suffix) {
                    return true;
                }
            } else {
                // e.g. "**/generated_contracts.rs" — exact filename match
                if file_name == suffix {
                    return true;
                }
            }
        } else if pattern.ends_with("/**") {
            // Directory prefix: e.g. "crates/aprender-test-lib/**"
            let dir = pattern.trim_end_matches("/**");
            if path_str.contains(dir) {
                return true;
            }
        } else if pattern.contains('/') {
            // Path pattern: check if path contains it
            if path_str.contains(pattern.as_str()) {
                return true;
            }
        } else {
            // Simple filename match
            if file_name == pattern || glob_match_simple(file_name, pattern) {
                return true;
            }
        }
    }
    false
}

fn glob_match_simple(name: &str, pattern: &str) -> bool {
    if let Some(star_pos) = pattern.find('*') {
        let prefix = &pattern[..star_pos];
        let suffix = &pattern[star_pos + 1..];
        name.starts_with(prefix) && name.ends_with(suffix)
    } else {
        name == pattern
    }
}

pub(crate) fn check_file_health(project_path: &Path) -> ComplianceCheck {
    let files = match discover_source_files(project_path) {
        Ok(f) => f,
        Err(msg) => {
            return ComplianceCheck {
                name: "CB-040: File Health".into(),
                status: CheckStatus::Skip,
                message: msg,
                severity: Severity::Info,
            }
        }
    };
    if files.is_empty() {
        return ComplianceCheck {
            name: "CB-040: File Health".into(),
            status: CheckStatus::Pass,
            message: "No source files found".into(),
            severity: Severity::Info,
        };
    }
    // Excludes are applied by `discover_source_files` (#986) — one place, so
    // the check, the cross-stack report and the ratchet baseline see the same
    // file set. Filtering again here would just be a second implementation of
    // the same rule.
    let mut metrics: Vec<FileHealthMetrics> = Vec::new();
    let (mut critical_count, mut problem_count, mut over_500_count) = (0, 0, 0);
    for file_path in &files {
        let content = match std::fs::read_to_string(file_path) {
            Ok(c) => c,
            Err(_) => continue,
        };
        let (c, p, o) = classify_file_health(file_path, &content);
        critical_count += c;
        problem_count += p;
        over_500_count += o;
        let lines = content.lines().count();
        let test_lines = estimate_test_lines(&content);
        let avg_complexity = estimate_avg_complexity(&content);
        metrics.push(FileHealthMetrics::calculate(
            file_path.clone(),
            lines,
            test_lines,
            avg_complexity,
            0,
        ));
    }
    let report = FileHealthReport::from_files(project_path.to_path_buf(), metrics);
    build_file_health_check(&report, critical_count, problem_count, over_500_count)
}

#[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
pub(crate) fn estimate_test_lines(content: &str) -> usize {
    let mut test_lines = 0;
    let mut in_test_module = false;
    let mut brace_depth = 0;
    for line in content.lines() {
        let trimmed = line.trim();
        if trimmed.contains("#[cfg(test)]") {
            in_test_module = true;
        }
        if in_test_module {
            brace_depth += trimmed.matches('{').count();
            brace_depth = brace_depth.saturating_sub(trimmed.matches('}').count());
            test_lines += 1;
            if brace_depth == 0 && test_lines > 1 {
                in_test_module = false;
            }
        }
        if trimmed.contains("#[test]") || trimmed.contains("#[tokio::test]") {
            test_lines += 10;
        }
    }
    test_lines
}

fn count_line_complexity(trimmed: &str) -> u32 {
    let flow_keywords: &[(&str, &str)] = &[
        ("if ", " if "),
        ("else if ", "} else if "),
        ("match ", " match "),
        ("for ", " for "),
        ("while ", " while "),
        ("loop ", " loop "),
    ];
    let mut count = 0u32;
    for &(prefix, infix) in flow_keywords {
        if trimmed.starts_with(prefix) || trimmed.contains(infix) {
            count += 1;
        }
    }
    if trimmed.contains("&&") || trimmed.contains("||") {
        count += 1;
    }
    if trimmed.contains('?') && !trimmed.contains("//") {
        count += 1;
    }
    count
}

#[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
pub(crate) fn estimate_avg_complexity(content: &str) -> f32 {
    let mut total_complexity = 1u32;
    let mut function_count = 0u32;
    for line in content.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with("fn ")
            || trimmed.starts_with("pub fn ")
            || trimmed.starts_with("async fn ")
            || trimmed.starts_with("pub async fn ")
        {
            function_count += 1;
        }
        total_complexity += count_line_complexity(trimmed);
    }
    if function_count == 0 {
        return total_complexity as f32;
    }
    total_complexity as f32 / function_count as f32
}

// Sovereign stack patterns and PAIML deps checks (from migrate_handlers.rs)
pub(crate) use super::check_sovereign::*;

#[cfg(test)]
mod check_extended_tests {
    //! Covers pure-compute helpers in check_extended.rs (207 uncov on broad,
    //! 0% cov). Skips fs-walking check_muda_waste / check_reproducibility /
    //! check_dead_code_percentage / check_dependency_count happy paths.
    use super::*;
    use crate::cli::handlers::comply_cb_detect::{
        CbPatternViolation, DependencyCountReport, Severity as CbSev,
    };

    fn make_report() -> DependencyCountReport {
        DependencyCountReport {
            direct_count: 50,
            transitive_count: 200,
            prod_transitive_count: Some(180),
            score: 4,
            duplicate_crates: vec![],
            feature_gated_count: 30,
            feature_gated_pct: 60.0,
            sovereign_crates: vec![],
            sovereign_bonus: 0,
            trend: None,
            violations: vec![],
        }
    }

    // ── format_dependency_message: 4 conditional branches ──

    #[test]
    fn test_format_dependency_message_with_prod_transitive() {
        let r = make_report();
        let msg = format_dependency_message(&r);
        assert!(msg.contains("Score: 4/5"));
        assert!(msg.contains("50 direct"));
        assert!(msg.contains("180 prod transitive"));
        assert!(msg.contains("200 total w/dev"));
        assert!(msg.contains("60% feature-gated"));
    }

    #[test]
    fn test_format_dependency_message_without_prod_transitive() {
        let mut r = make_report();
        r.prod_transitive_count = None;
        let msg = format_dependency_message(&r);
        // Falls back to "{} transitive" without the prod/total split.
        assert!(msg.contains("200 transitive"));
        assert!(!msg.contains("prod transitive"));
    }

    #[test]
    fn test_format_dependency_message_with_trend_delta() {
        use crate::cli::handlers::comply_cb_detect::DependencyTrend;
        let mut r = make_report();
        r.trend = Some(DependencyTrend {
            direct_delta: 5,
            transitive_delta: -3,
            previous_timestamp: "2024-01-01".to_string(),
        });
        let msg = format_dependency_message(&r);
        assert!(msg.contains("+5") || msg.contains("-3"));
    }

    #[test]
    fn test_format_dependency_message_with_zero_trend_skipped() {
        use crate::cli::handlers::comply_cb_detect::DependencyTrend;
        let mut r = make_report();
        r.trend = Some(DependencyTrend {
            direct_delta: 0,
            transitive_delta: 0,
            previous_timestamp: "2024-01-01".to_string(),
        });
        let msg = format_dependency_message(&r);
        // Zero deltas → trend section should not appear.
        assert!(!msg.contains("\u{0394}"));
    }

    #[test]
    fn test_format_dependency_message_with_duplicate_crates() {
        use crate::cli::handlers::comply_cb_detect::DuplicateCrate;
        let mut r = make_report();
        r.duplicate_crates = vec![DuplicateCrate {
            name: "rand".into(),
            versions: vec!["0.8".into(), "0.9".into()],
        }];
        let msg = format_dependency_message(&r);
        assert!(msg.contains("1 duplicates"));
    }

    #[test]
    fn test_format_dependency_message_with_sovereign_bonus() {
        let mut r = make_report();
        r.sovereign_bonus = 2;
        r.sovereign_crates = vec!["trueno".into(), "aprender".into()];
        let msg = format_dependency_message(&r);
        assert!(msg.contains("+2 sovereign"));
        assert!(msg.contains("trueno") || msg.contains("aprender"));
    }

    // ── append_violation_details ──

    #[test]
    fn test_append_violation_details_error_uses_x_icon() {
        let r = DependencyCountReport {
            violations: vec![CbPatternViolation {
                pattern_id: "CB-081".into(),
                file: "Cargo.toml".into(),
                line: 1,
                description: "too many deps".into(),
                severity: CbSev::Error,
            }],
            ..make_report()
        };
        let mut msg = String::new();
        append_violation_details(&mut msg, &r, 5);
        assert!(msg.contains("\u{2717}"), "error → ✗ icon");
        assert!(msg.contains("too many deps"));
    }

    #[test]
    fn test_append_violation_details_warning_uses_warning_icon() {
        let r = DependencyCountReport {
            violations: vec![CbPatternViolation {
                pattern_id: "CB-081".into(),
                file: "Cargo.toml".into(),
                line: 1,
                description: "warning desc".into(),
                severity: CbSev::Warning,
            }],
            ..make_report()
        };
        let mut msg = String::new();
        append_violation_details(&mut msg, &r, 5);
        assert!(msg.contains("\u{26a0}"), "warning → ⚠ icon");
    }

    #[test]
    fn test_append_violation_details_respects_limit() {
        let r = DependencyCountReport {
            violations: (0..10)
                .map(|i| CbPatternViolation {
                    pattern_id: format!("CB-{i}"),
                    file: "Cargo.toml".into(),
                    line: i,
                    description: format!("violation {i}"),
                    severity: CbSev::Warning,
                })
                .collect(),
            ..make_report()
        };
        let mut msg = String::new();
        append_violation_details(&mut msg, &r, 3);
        // Only 3 lines should appear.
        assert_eq!(msg.matches("violation").count(), 3);
    }

    #[test]
    fn test_append_violation_details_empty_violations_no_change() {
        let r = make_report();
        let mut msg = "existing".to_string();
        append_violation_details(&mut msg, &r, 5);
        assert_eq!(msg, "existing");
    }

    // ── check_dependency_count: no-Cargo.toml skip path ──

    #[test]
    fn test_check_dependency_count_no_cargo_toml_skips() {
        let tmp = tempfile::tempdir().unwrap();
        let check = check_dependency_count(tmp.path());
        assert!(matches!(check.status, CheckStatus::Skip));
        assert!(check.message.contains("Not a Rust project"));
    }

    // ── Wave 39 PR22: classify_file_health + build_file_health_check ────────

    fn make_health_report(total: usize, avg_health: u8) -> FileHealthReport {
        FileHealthReport {
            project_path: std::path::PathBuf::from("/tmp"),
            total_files: total,
            total_lines: total * 100,
            average_health: avg_health,
            average_grade: crate::services::file_health::HealthGrade::from_score(avg_health),
            critical_files: vec![],
            problem_files: vec![],
            warning_files: vec![],
            healthy_files_count: total,
            is_compliant: true,
            recommendations: vec![],
        }
    }

    // ── classify_file_health ────────────────────────────────────────────────

    #[test]
    fn test_classify_file_health_normal_file_under_500() {
        let path = std::path::PathBuf::from("src/foo.rs");
        let content: String = "fn x() {}\n".repeat(100); // 100 lines
        let (critical, problem, over_500) = classify_file_health(&path, &content);
        assert_eq!(critical, 0);
        assert_eq!(problem, 0);
        assert_eq!(over_500, 0);
    }

    #[test]
    fn test_classify_file_health_normal_file_500_to_1000() {
        let path = std::path::PathBuf::from("src/foo.rs");
        let content: String = "fn x() {}\n".repeat(700);
        let (critical, problem, over_500) = classify_file_health(&path, &content);
        assert_eq!(critical, 0);
        assert_eq!(problem, 0);
        assert_eq!(over_500, 1);
    }

    #[test]
    fn test_classify_file_health_normal_file_problem_threshold() {
        // PIN: non-test file, problem threshold = 1000 (lines > 1000 AND <= 2000).
        let path = std::path::PathBuf::from("src/foo.rs");
        let content: String = "fn x() {}\n".repeat(1500);
        let (critical, problem, over_500) = classify_file_health(&path, &content);
        assert_eq!(critical, 0);
        assert_eq!(problem, 1);
        assert_eq!(over_500, 1);
    }

    #[test]
    fn test_classify_file_health_normal_file_critical_threshold() {
        // PIN: non-test file, critical threshold = 2000 (lines > 2000).
        let path = std::path::PathBuf::from("src/foo.rs");
        let content: String = "fn x() {}\n".repeat(2500);
        let (critical, problem, _) = classify_file_health(&path, &content);
        assert_eq!(critical, 1);
        assert_eq!(problem, 0);
    }

    #[test]
    fn test_classify_file_health_test_file_doubled_thresholds() {
        // PIN: test files (path with /tests/ OR name starts with "test" OR ends "_tests.rs")
        // get DOUBLED thresholds: critical=4000, problem=2000.
        let path = std::path::PathBuf::from("src/foo_tests.rs");
        let content: String = "fn x() {}\n".repeat(1500); // 1500 lines
        let (critical, problem, _) = classify_file_health(&path, &content);
        // Below problem threshold (2000) for test files.
        assert_eq!(critical, 0);
        assert_eq!(problem, 0);
    }

    #[test]
    fn test_classify_file_health_test_file_at_problem_threshold() {
        let path = std::path::PathBuf::from("src/foo_tests.rs");
        let content: String = "fn x() {}\n".repeat(2500);
        let (critical, problem, _) = classify_file_health(&path, &content);
        // 2500 > 2000 (test problem threshold) AND <= 4000 (test critical) → problem
        assert_eq!(critical, 0);
        assert_eq!(problem, 1);
    }

    #[test]
    fn test_classify_file_health_test_file_at_critical_threshold() {
        let path = std::path::PathBuf::from("src/foo_tests.rs");
        let content: String = "fn x() {}\n".repeat(4500);
        let (critical, problem, _) = classify_file_health(&path, &content);
        assert_eq!(critical, 1);
        assert_eq!(problem, 0);
    }

    #[test]
    fn test_classify_file_health_tests_subdirectory_treated_as_test() {
        // PIN: any path containing "/tests/" gets test thresholds.
        let path = std::path::PathBuf::from("src/foo/tests/integration.rs");
        let content: String = "fn x() {}\n".repeat(2500);
        let (critical, problem, _) = classify_file_health(&path, &content);
        // 2500 > 2000 (problem) AND <= 4000 (critical) → problem
        assert_eq!(problem, 1);
        assert_eq!(critical, 0);
    }

    #[test]
    fn test_classify_file_health_starts_with_test_prefix() {
        // PIN: filename starting with "test" (e.g. test_foo.rs) → test thresholds.
        let path = std::path::PathBuf::from("src/test_foo.rs");
        let content: String = "fn x() {}\n".repeat(2500);
        let (_, problem, _) = classify_file_health(&path, &content);
        assert_eq!(problem, 1);
    }

    // ── build_file_health_check ─────────────────────────────────────────────

    #[test]
    fn test_build_file_health_check_critical_count_emits_critical() {
        let report = make_health_report(10, 80);
        let check = build_file_health_check(&report, 1, 0, 0);
        assert!(matches!(check.status, CheckStatus::Fail));
        assert_eq!(check.severity, Severity::Critical);
        assert!(check.message.contains("CRITICAL"));
    }

    #[test]
    fn test_build_file_health_check_problem_count_emits_warning() {
        let report = make_health_report(10, 70);
        let check = build_file_health_check(&report, 0, 1, 0);
        assert!(matches!(check.status, CheckStatus::Warn));
        assert_eq!(check.severity, Severity::Warning);
    }

    #[test]
    fn test_build_file_health_check_over_500_above_5_emits_warning() {
        // PIN: over_500_count > 5 alone (no problem/critical) emits Warn.
        let report = make_health_report(20, 80);
        let check = build_file_health_check(&report, 0, 0, 6);
        assert!(matches!(check.status, CheckStatus::Warn));
    }

    #[test]
    fn test_build_file_health_check_over_500_at_5_passes() {
        // PIN: over_500_count == 5 (boundary) does NOT trigger warning.
        let report = make_health_report(20, 90);
        let check = build_file_health_check(&report, 0, 0, 5);
        assert!(matches!(check.status, CheckStatus::Pass));
    }

    #[test]
    fn test_build_file_health_check_clean_passes() {
        let report = make_health_report(10, 95);
        let check = build_file_health_check(&report, 0, 0, 0);
        assert!(matches!(check.status, CheckStatus::Pass));
        assert_eq!(check.severity, Severity::Info);
        assert!(check.message.contains("all files <500 lines"));
    }

    /// #939: CB-081 measured the transitive dependency graph with `cargo tree`,
    /// and cargo WRITES `Cargo.lock` to answer. So `comply check` created the
    /// very file its own "Cargo.lock Present" check was about to look for: run
    /// 1 said Fail, run 2 of the identical command said Pass. Measuring must
    /// not create the evidence.
    #[test]
    fn cb081_does_not_create_a_lockfile_in_the_audited_project() {
        let dir = tempfile::TempDir::new().expect("tempdir");
        std::fs::create_dir_all(dir.path().join("src")).expect("mkdir");
        std::fs::write(
            dir.path().join("Cargo.toml"),
            "[package]\nname = \"idem\"\nversion = \"0.1.0\"\nedition = \"2021\"\n\n[dependencies]\n",
        )
        .expect("write manifest");
        std::fs::write(dir.path().join("src/lib.rs"), "//! x\n").expect("write lib");

        let check = check_dependency_count(dir.path());

        assert!(
            !dir.path().join("Cargo.lock").exists(),
            "comply check must not write Cargo.lock into the project it audits"
        );
        assert!(
            !dir.path().join(".pmat").exists(),
            "comply check must not write .pmat/ into the project it audits"
        );
        assert_eq!(check.status, CheckStatus::Skip);
        assert!(
            check.message.contains("Not measured"),
            "an unresolved project is unmeasured, not healthy: {}",
            check.message
        );
    }

    /// The measurement itself is unchanged when the project HAS resolved its
    /// dependencies: a lockfile present means CB-081 runs as before.
    #[test]
    fn cb081_still_scores_a_project_that_has_a_lockfile() {
        let dir = tempfile::TempDir::new().expect("tempdir");
        std::fs::write(
            dir.path().join("Cargo.toml"),
            "[package]\nname = \"idem\"\nversion = \"0.1.0\"\n\n[dependencies]\n",
        )
        .expect("write manifest");
        std::fs::write(
            dir.path().join("Cargo.lock"),
            "# This file is automatically @generated by Cargo.\nversion = 4\n",
        )
        .expect("write lock");

        let check = check_dependency_count(dir.path());
        assert_ne!(check.status, CheckStatus::Skip, "{}", check.message);
    }
}

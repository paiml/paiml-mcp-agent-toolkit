// Compliance check logic - handle_check and individual check functions
//
// This is the core compliance checking module, containing handle_check
// and basic check_* functions (version, config, hooks, quality, CB patterns).

use crate::cli::commands::ComplyOutputFormat;
use crate::cli::handlers::comply_cb_detect::{
    detect_bricks_without_assertions, detect_cb001_wgsl_no_bounds_check,
    detect_cb002_wgsl_barrier_divergence, detect_cb020_unsafe_without_safety,
    detect_cb021_simd_without_target_feature, detect_cb120_nan_unsafe_comparison,
    detect_cb121_lock_poisoning, detect_cb122_serde_safety, detect_cb123_undocumented_ignore,
    detect_cb124_coverage_threshold, detect_cb125_coverage_exclusion_gaming,
    detect_cb126_slow_tests, detect_cb127_slow_coverage, detect_profiler_anomalies,
};
use crate::models::comply_config::{ComplyThresholds, PmatYamlConfig};
use anyhow::Result;
use chrono::Utc;
use std::fs;
use std::path::Path;

use super::check_best_practices::{
    check_agent_context_adoption, check_custom_scores, check_lean_best_practices_with_config,
    check_lua_best_practices_with_config, check_markdown_best_practices_with_config,
    check_model_quality_with_config, check_rust_best_practices_with_config,
    check_scala_best_practices_with_config, check_shell_makefile_quality,
    check_sql_best_practices_with_config, check_tdg_grade_gate,
    check_yaml_best_practices_with_config,
};
use super::check_extended::{
    check_dead_code_percentage, check_dependency_count, check_edd_compliance, check_file_health,
    check_golden_trace_drift, check_muda_waste_score, check_paiml_deps_workspace,
    check_reproducibility_level, check_sovereign_stack_patterns,
    check_workspace_member_registry_deps,
};
use super::check_mono_spec::{
    check_memory_profiling, check_mono_spec_structure, check_swe_ci_evoscore,
};
use super::types::*;

/// Check project compliance with current PMAT version.
///
/// Read-only by construction: nothing on this path writes into the project
/// being audited. It used to end with `update_last_check_timestamp`, which
/// CREATED `.pmat/project.toml` (and the `.pmat/` directory) in a tree that had
/// neither — so the audit manufactured the pin it reported one run later
/// (#939). Recording a project's pinned version is `pmat comply init`'s job.
#[provable_contracts_macros::contract("pmat-core.yaml", equation = "path_exists")]
pub(crate) async fn handle_check(
    project_path: &Path,
    strict: bool,
    failures_only: bool,
    format: ComplyOutputFormat,
) -> Result<()> {
    let report = compute_compliance_report(project_path, failures_only)?;

    output_compliance_report(&report, format, project_path)?;

    apply_exit_policy(&report, strict)
}

/// Reject an input no compliance verdict can honestly be given for.
///
/// A `debug_assert!` is not a guard in a release build: `comply check -p
/// /does/not/exist` sailed past it, and the first thing that touched the
/// filesystem was `load_or_create_project_config`, which tried to
/// `create_dir_all("/does/not/exist/.pmat")`. That surfaced as "Permission
/// denied (os error 13)" and exit 126 — an error about the wrong thing,
/// pointing at a permissions problem the user does not have.
///
/// A path that exists is not yet a project either. An EMPTY directory came back
/// "Project Version: 3.30.0 / Versions Behind: 0 / Status: COMPLIANT",
/// "154 checks (0 fail)", exit 0 — the same headline check count this
/// repository reports, and a better verdict than this repository gets. Every
/// one of those 154 checks had skipped for want of anything to look at, and
/// `load_or_create_project_config` had meanwhile WRITTEN a `.pmat/` directory
/// into the empty tree to invent the version it then reported as current.
fn guard_analysable_project(project_path: &Path) -> Result<()> {
    if !project_path.exists() {
        anyhow::bail!("Path not found: {}", project_path.display());
    }
    if let Some(reason) = no_project_here(project_path) {
        anyhow::bail!(
            "No project found at {}: {reason} — comply has nothing to check here, and an unmeasured project is not a compliant one",
            project_path.display()
        );
    }
    Ok(())
}

/// THE compliance computation. `comply check` and `comply report` are two
/// renderings of this one result, not two implementations of it.
///
/// `comply report` used to run a five-check stub of its own and finish it with
/// a literal `recommendations: vec![]`, while `comply check` computed that same
/// schema field from the same config a few hundred lines away — so on the very
/// inputs where check returned one and two recommendations, report returned
/// none. Worse, report's five checks contained the only ones that cannot fail,
/// so report answered COMPLIANT on a tree where check had just reported a Fail,
/// and report's JSON was byte-identical for an empty project and a 121-file
/// defect-ridden one bar the timestamp. Both commands now share this function,
/// so there is exactly one answer to give.
pub(crate) fn compute_compliance_report(
    project_path: &Path,
    failures_only: bool,
) -> Result<ComplianceReport> {
    guard_analysable_project(project_path)?;

    crate::status_eprintln!("Checking PMAT compliance for {}", project_path.display());

    let yaml_config = PmatYamlConfig::load(project_path).unwrap_or_default();
    let comply_config = &yaml_config.comply;
    announce_suppressions(project_path, comply_config);

    let (config, version_source) = load_project_config_with_source(project_path)?;
    let project_version = &config.pmat.version;

    let checks = build_all_compliance_checks(project_path, comply_config, project_version);
    Ok(build_compliance_report(
        checks,
        project_version,
        version_source,
        failures_only,
    ))
}

/// Why this path holds no project, or `None` if it plausibly does.
///
/// Deliberately permissive — one manifest, one VCS directory or one source file
/// anywhere in the tree is enough. The case being refused is the one where a
/// verdict is pure fabrication: nothing to read, so nothing measured.
fn no_project_here(project_path: &Path) -> Option<String> {
    if project_path.is_file() {
        return None;
    }

    const MANIFESTS: [&str; 12] = [
        "Cargo.toml",
        "package.json",
        "pyproject.toml",
        "setup.py",
        "go.mod",
        "build.sbt",
        "pom.xml",
        "build.gradle",
        "Makefile",
        "lakefile.lean",
        ".pmat.yaml",
        ".git",
    ];
    if MANIFESTS.iter().any(|m| project_path.join(m).exists()) {
        return None;
    }

    // No manifest is not decisive on its own: comply checks Markdown, shell and
    // SQL too. Any file the tool could read counts, `.pmat/` excepted — comply
    // writes that itself, so accepting it would let the command manufacture its
    // own evidence of a project on the second run.
    let mut saw_file = false;
    if let Ok(entries) = std::fs::read_dir(project_path) {
        for entry in entries.flatten() {
            let name = entry.file_name();
            if name == ".pmat" {
                continue;
            }
            saw_file = true;
            break;
        }
    }
    if saw_file {
        return None;
    }

    Some(
        "no manifest (Cargo.toml, package.json, pyproject.toml, go.mod, …), no .git, and no files"
            .to_string(),
    )
}

/// One-shot log summarizing the active `.pmat.yaml` configuration.
fn announce_suppressions(
    project_path: &Path,
    comply_config: &crate::models::comply_config::ComplyConfig,
) {
    let config_path = project_path.join(".pmat.yaml");
    if !config_path.exists() {
        return;
    }
    crate::status_eprintln!("  Using configuration from .pmat.yaml");
    if !comply_config.suppressions.is_empty() {
        crate::status_eprintln!(
            "  {} suppression rule(s) loaded",
            comply_config.suppressions.len()
        );
    }
}

/// The exit code `comply check` will use, and the reason for it.
///
/// Pure, so the policy is testable; `apply_exit_policy` is the only thing that
/// calls `std::process::exit`.
#[derive(Debug, PartialEq, Eq)]
pub(crate) struct ExitPolicy {
    pub(crate) code: i32,
    /// Printed to stderr when the code is not 0. `None` means "nothing to
    /// explain": the run was clean.
    pub(crate) reason: Option<String>,
}

/// Decide the exit code: 0 clean, 1 on failures, 2 on `--strict` with
/// warnings but no failures.
///
/// Two things were wrong here (#945). First, the counts came from
/// `report.checks`, which `--failures-only` has already filtered — so
/// `comply check --strict --failures-only` silently dropped every warning and
/// exited 0 where the same run without `--failures-only` exited 2. The counts
/// now come from `report.summary`, which is tallied before that filter, so a
/// display flag can no longer move the exit code.
///
/// Second, code 2 was silent. A compliant project (`is_compliant: true`,
/// `fail: 0`) exited 2 with nothing anywhere saying why, against a `--help`
/// that promises only "exit with error if non-compliant". The escalation is
/// kept — `--strict` means warnings are errors, and that is the flag's only
/// effect — but it now names itself, its counts, and the code the same run
/// would have produced without the flag.
pub(crate) fn exit_policy(report: &ComplianceReport, strict: bool) -> ExitPolicy {
    let failures = report.summary.fail;
    let warnings = report.summary.warn;

    if !report.is_compliant {
        return ExitPolicy {
            code: 1,
            reason: Some(format!(
                "comply: {failures} check(s) failed -> exit 1 (see the Fail entries above)"
            )),
        };
    }
    if strict && warnings > 0 && failures == 0 {
        return ExitPolicy {
            code: 2,
            reason: Some(format!(
                "comply: the report is COMPLIANT ({failures} failures), but --strict treats \
                 warnings as errors: {warnings} warning(s) -> exit 2. \
                 Exit codes: 0 = no failures and no warnings, 1 = failures, \
                 2 = --strict with warnings only. Without --strict this run exits 0."
            )),
        };
    }
    ExitPolicy {
        code: 0,
        reason: None,
    }
}

/// Apply the report's exit policy, explaining any non-zero code on stderr.
fn apply_exit_policy(report: &ComplianceReport, strict: bool) -> Result<()> {
    let policy = exit_policy(report, strict);
    if let Some(reason) = &policy.reason {
        eprintln!("{reason}");
    }
    if policy.code != 0 {
        std::process::exit(policy.code);
    }
    Ok(())
}

/// A named compliance-check group and the thunk that produces its checks.
type CheckGroup<'a> = (
    &'static str,
    Box<dyn Fn() -> Vec<ComplianceCheck> + Send + Sync + 'a>,
);

fn build_all_compliance_checks(
    project_path: &Path,
    comply_config: &crate::models::comply_config::ComplyConfig,
    project_version: &str,
) -> Vec<ComplianceCheck> {
    // Data-driven group list: each entry is independent and side-effect-free
    // w.r.t. the others, which lets `run_check_groups` both report live
    // per-group progress AND run the groups concurrently.
    let groups: Vec<CheckGroup> = vec![
        (
            "foundation",
            Box::new(move || build_foundation_checks(project_path, comply_config, project_version)),
        ),
        (
            "language",
            Box::new(move || build_language_best_practices(project_path, comply_config)),
        ),
        (
            "custom-score",
            Box::new(move || build_custom_score_checks(project_path, comply_config)),
        ),
        (
            "provable-contracts",
            Box::new(move || build_provable_contract_checks(project_path, comply_config)),
        ),
        (
            "contract-surfaces",
            Box::new(move || build_contract_surface_checks(project_path, comply_config)),
        ),
        (
            "agent-contracts",
            Box::new(move || build_agent_contract_checks(project_path, comply_config)),
        ),
        (
            "commit-enforcement",
            Box::new(move || build_commit_enforcement_checks(project_path, comply_config)),
        ),
        (
            "binding-scope",
            Box::new(move || build_binding_scope_checks(project_path, comply_config)),
        ),
        (
            "work-ladder",
            Box::new(move || build_work_ladder_checks(project_path, comply_config)),
        ),
        (
            "falsification",
            Box::new(move || build_falsification_unification_checks(project_path, comply_config)),
        ),
        (
            "codegen",
            Box::new(move || build_codegen_checks(project_path, comply_config)),
        ),
        (
            "cot-proof",
            Box::new(move || build_cot_proof_checks(project_path, comply_config)),
        ),
        (
            "macs",
            Box::new(move || build_macs_checks(project_path, comply_config)),
        ),
    ];
    run_check_groups(groups)
}

/// Peak RSS one concurrent comply worker costs, in bytes.
///
/// Measured on this repo, `/usr/bin/time -v pmat comply check`, varying only
/// `RAYON_NUM_THREADS` — memory scales LINEARLY with concurrency because each
/// check walks and reads the tree into its own buffers and nothing is shared:
///
/// | threads | peak RSS | wall  |
/// |---------|----------|-------|
/// | 1       |  4.1 GB  | 2:23  |
/// | 4       | 15.5 GB  | 1:12  |
/// | default | 58.7 GB  | 0:38  |
///
/// 4.1 GB x threads predicts 16.4 and 57.4 against 15.5 and 58.7 observed.
const COMPLY_BYTES_PER_WORKER: u64 = 4 * 1024 * 1024 * 1024;

/// Fraction of *available* RAM comply may plan to use (1/8).
///
/// Sizing to all-of-memory-minus-headroom is what an unbounded run effectively
/// did: on a 125 GB workstation that authorised 24 workers and ~96 GB, which is
/// hostile to every other build on the box even though it technically fits.
/// A compliance check is a background chore, not the machine's purpose.
const COMPLY_MEMORY_DIVISOR: u64 = 8;

/// Default ceiling on concurrent groups, regardless of how big the machine is.
///
/// Past ~4 workers the wall-clock return falls off fast while memory keeps
/// climbing linearly: 1 -> 2:23 at 4.1 GB, 4 -> 1:12 at 15.5 GB, unbounded ->
/// 0:38 at 58.7 GB. The second halving costs 11 GB; the third costs 43 GB.
/// Anyone who wants that trade can take it with `PMAT_COMPLY_JOBS`.
const COMPLY_MAX_DEFAULT_JOBS: usize = 4;

/// How many comply groups may run at once.
///
/// Concurrency used to be rayon's default — one worker per CPU — which sizes
/// the run by the wrong resource. Comply's binding constraint is MEMORY, not
/// CPU: at ~4 GB per worker a 16-core box peaks near 64 GB and a 64-core box
/// would ask for 256 GB. On this workstation an unbounded run reached 58.7 GB
/// against pmat itself and ~94 GB against aprender, driving load average to 75,
/// starving every other build, and tripping the OOM guard.
///
/// The bound is whichever runs out first of: available RAM / 8, the CPU count,
/// the number of groups (more workers than groups is pure waste), and a default
/// ceiling. `PMAT_COMPLY_JOBS` overrides all of it.
fn comply_concurrency(groups: usize) -> usize {
    if let Ok(raw) = std::env::var("PMAT_COMPLY_JOBS") {
        if let Ok(n) = raw.trim().parse::<usize>() {
            if n > 0 {
                return n;
            }
        }
    }
    let cpus = num_cpus::get().max(1);
    // Never 0: one worker always runs, even on a machine with no headroom,
    // because refusing to check anything is worse than checking slowly.
    let by_memory = available_memory_bytes()
        .map(|avail| ((avail / COMPLY_MEMORY_DIVISOR) / COMPLY_BYTES_PER_WORKER).max(1) as usize)
        .unwrap_or(COMPLY_MAX_DEFAULT_JOBS);
    by_memory
        .min(cpus)
        .min(groups.max(1))
        .min(COMPLY_MAX_DEFAULT_JOBS)
        .max(1)
}

/// Available RAM in bytes, or `None` where it cannot be read.
///
/// `MemAvailable` rather than `MemFree`: the kernel's own estimate of what a
/// new workload can claim without swapping, which is the number this decision
/// actually needs. Linux only — every other platform gets `None` and the
/// CPU-count fallback, which is no worse than the behaviour this replaces.
fn available_memory_bytes() -> Option<u64> {
    let meminfo = std::fs::read_to_string("/proc/meminfo").ok()?;
    let line = meminfo.lines().find(|l| l.starts_with("MemAvailable:"))?;
    let kb: u64 = line.split_whitespace().nth(1)?.parse().ok()?;
    Some(kb * 1024)
}

/// Run each check group concurrently, emitting a live per-group status line
/// (name · count · fail/warn tally · elapsed) to stderr as each completes, plus
/// a final summary. Replaces the previous silent sequential scan: a long
/// compliance run is now observable (you can see which group is slow) and
/// wall-time drops from Σ(group) to max(group). Groups are independent and
/// each reads its own inputs, so concurrency is safe.
fn run_check_groups(groups: Vec<CheckGroup>) -> Vec<ComplianceCheck> {
    use rayon::prelude::*;
    use std::sync::atomic::{AtomicUsize, Ordering};

    let total = groups.len();
    let jobs = comply_concurrency(total);
    crate::status_eprintln!(
        "  comply: {total} group(s), {jobs} at a time (~{} GB peak; \
         PMAT_COMPLY_JOBS overrides)",
        (jobs as u64 * COMPLY_BYTES_PER_WORKER) / (1024 * 1024 * 1024)
    );
    let overall = std::time::Instant::now();
    let done = AtomicUsize::new(0);

    // A DEDICATED pool, not the global one. Comply nests rayon — groups run in
    // parallel and `run_checks_parallel` splits the checks inside a group again
    // — so capping only the outer loop would let the inner level re-expand to
    // one worker per CPU and put the memory straight back. Both levels run
    // inside `pool.install`, so they draw from the same bounded set of threads.
    //
    // If the pool cannot be built we fall back to the global one rather than
    // failing the run: an unbounded compliance check is bad, refusing to check
    // at all is worse.
    let work = || {
        let mut grouped: Vec<(usize, Vec<ComplianceCheck>)> = groups
            .into_par_iter()
            .enumerate()
            .map(|(idx, (name, run))| {
                let start = std::time::Instant::now();
                let checks = run();
                let fails = checks
                    .iter()
                    .filter(|c| c.status == CheckStatus::Fail)
                    .count();
                let warns = checks
                    .iter()
                    .filter(|c| c.status == CheckStatus::Warn)
                    .count();
                let n = done.fetch_add(1, Ordering::Relaxed) + 1;
                crate::status_eprintln!(
                    "  [{n:>2}/{total}] {name:<19} {:>3} checks · {fails} fail · {warns} warn · {:.1}s",
                    checks.len(),
                    start.elapsed().as_secs_f64()
                );
                (idx, checks)
            })
            .collect();
        grouped.sort_by_key(|(idx, _)| *idx);
        grouped
    };

    let grouped = match rayon::ThreadPoolBuilder::new()
        .num_threads(jobs)
        .thread_name(|i| format!("comply-{i}"))
        .build()
    {
        Ok(pool) => pool.install(work),
        Err(_) => work(),
    };

    // Declaration order is restored inside `work` so report output is
    // deterministic regardless of completion order.
    let all: Vec<ComplianceCheck> = grouped.into_iter().flat_map(|(_, c)| c).collect();

    let fails = all.iter().filter(|c| c.status == CheckStatus::Fail).count();
    crate::status_eprintln!(
        "  ── comply: {} checks in {:.1}s ({fails} fail) ──",
        all.len(),
        overall.elapsed().as_secs_f64()
    );
    all
}

fn build_compliance_report(
    checks: Vec<ComplianceCheck>,
    project_version: &str,
    version_source: VersionSource,
    failures_only: bool,
) -> ComplianceReport {
    debug_assert!(
        !project_version.is_empty(),
        "project_version must not be empty"
    );
    let failures = checks
        .iter()
        .filter(|c| c.status == CheckStatus::Fail)
        .count();
    let breaking_changes = get_breaking_changes_since(project_version);
    let versions_behind = calculate_versions_behind(project_version);

    let mut recommendations = vec![];
    if versions_behind > 0 {
        recommendations.push(format!(
            "Run 'pmat comply migrate' to update to v{}",
            PMAT_VERSION
        ));
    }
    if !breaking_changes.is_empty() {
        recommendations.push("Review breaking changes with 'pmat comply diff'".to_string());
    }

    ComplianceReport {
        project_version: project_version.to_string(),
        project_version_source: version_source,
        current_version: PMAT_VERSION.to_string(),
        is_compliant: failures == 0,
        versions_behind,
        summary: CheckSummary::tally(&checks),
        checks: if failures_only {
            checks
                .into_iter()
                .filter(|c| c.status == CheckStatus::Fail)
                .collect()
        } else {
            checks
        },
        breaking_changes,
        recommendations,
        timestamp: Utc::now(),
        // `comply check` has no --include-history; only `comply report` does.
        history: None,
    }
}

fn output_compliance_report(
    report: &ComplianceReport,
    format: ComplyOutputFormat,
    project_path: &Path,
) -> Result<()> {
    match format {
        ComplyOutputFormat::Text => print_compliance_text(report),
        ComplyOutputFormat::Json => println!("{}", serde_json::to_string_pretty(report)?),
        ComplyOutputFormat::Markdown => print_compliance_markdown(report),
        ComplyOutputFormat::Sarif => output_sarif_or_fallback(report, project_path)?,
    }
    Ok(())
}

/// `-f sarif` emits SARIF — pmat's own compliance findings, always.
///
/// This used to try the external `pv` binary and, when that was unavailable
/// (which is the normal case: it needs a sibling contracts directory AND `pv`
/// on PATH), silently print pmat's plain JSON report instead. The document
/// carried no `$schema`, no `version` and no `runs[]`, so GitHub's
/// upload-sarif action rejected it — and it was byte-identical to `-f json`
/// apart from the timestamp, so nothing in the output said the fallback had
/// happened. Even on the happy path the SARIF described pv's YAML contract
/// lint and never the 154 compliance checks the command was run for; pv's runs
/// are now MERGED into pmat's document rather than replacing it.
fn output_sarif_or_fallback(report: &ComplianceReport, project_path: &Path) -> Result<()> {
    debug_assert!(
        project_path.exists(),
        "project_path must exist: {}",
        project_path.display()
    );
    let mut sarif = build_sarif(report, project_path);
    if let Some(pv_runs) = pv_lint_runs(project_path) {
        if let Some(runs) = sarif["runs"].as_array_mut() {
            runs.extend(pv_runs);
        }
    }
    println!("{}", serde_json::to_string_pretty(&sarif)?);
    Ok(())
}

/// The SARIF rule id for a check: its `CB-nnn` clause when it has one, else a
/// slug of its name, so a consumer can suppress a single finding.
fn sarif_rule_id(name: &str) -> String {
    for token in name.split(|c: char| !(c.is_ascii_alphanumeric() || c == '-')) {
        let upper = token.to_ascii_uppercase();
        if upper.starts_with("CB-") && upper[3..].chars().all(|c| c.is_ascii_digit()) {
            return upper;
        }
    }
    let slug: String = name
        .chars()
        .map(|c| {
            if c.is_ascii_alphanumeric() {
                c.to_ascii_lowercase()
            } else {
                '-'
            }
        })
        .collect();
    format!("pmat/{}", slug.trim_matches('-'))
}

/// SARIF level for a check result. Skipped and passing checks produce no
/// result at all — a SARIF run reports findings, not a roll call.
fn sarif_level(check: &ComplianceCheck) -> Option<&'static str> {
    match check.status {
        CheckStatus::Fail => Some(match check.severity {
            Severity::Critical | Severity::Error => "error",
            Severity::Warning => "warning",
            Severity::Info => "note",
        }),
        CheckStatus::Warn => Some("warning"),
        CheckStatus::Pass | CheckStatus::Skip => None,
    }
}

/// A SARIF 2.1.0 document for pmat's own compliance report.
fn build_sarif(report: &ComplianceReport, project_path: &Path) -> serde_json::Value {
    let uri = project_path.display().to_string();

    let rules: Vec<serde_json::Value> = report
        .checks
        .iter()
        .map(|check| {
            serde_json::json!({
                "id": sarif_rule_id(&check.name),
                "name": check.name,
                "shortDescription": { "text": check.name },
            })
        })
        .collect();

    let results: Vec<serde_json::Value> = report
        .checks
        .iter()
        .filter_map(|check| {
            let level = sarif_level(check)?;
            Some(serde_json::json!({
                "ruleId": sarif_rule_id(&check.name),
                "level": level,
                "message": { "text": format!("{}: {}", check.name, check.message) },
                "locations": [{
                    "physicalLocation": {
                        "artifactLocation": { "uri": uri }
                    }
                }],
            }))
        })
        .collect();

    serde_json::json!({
        "$schema": "https://json.schemastore.org/sarif-2.1.0.json",
        "version": "2.1.0",
        "runs": [{
            "tool": {
                "driver": {
                    "name": "pmat",
                    "version": PMAT_VERSION,
                    "informationUri": "https://github.com/paiml/paiml-mcp-agent-toolkit",
                    "rules": rules,
                }
            },
            "results": results,
        }]
    })
}

/// pv's SARIF `runs[]`, when the external contract linter is available and
/// produced a SARIF document. Anything else is reported on stderr rather than
/// swallowed.
fn pv_lint_runs(project_path: &Path) -> Option<Vec<serde_json::Value>> {
    let raw = try_pv_lint_sarif(project_path)?;
    match serde_json::from_str::<serde_json::Value>(&raw) {
        Ok(value) => value.get("runs").and_then(|runs| runs.as_array()).cloned(),
        Err(e) => {
            eprintln!("Warning: `pv lint --format sarif` output was not JSON ({e}); reporting pmat's compliance checks only");
            None
        }
    }
}

fn try_pv_lint_sarif(project_path: &Path) -> Option<String> {
    debug_assert!(
        project_path.exists(),
        "project_path must exist: {}",
        project_path.display()
    );
    let contracts_dir = resolve_contracts_dir(project_path)?;
    let output = std::process::Command::new("pv")
        .args([
            "lint",
            &contracts_dir.display().to_string(),
            "--format",
            "sarif",
        ])
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::null())
        .output()
        .ok()?;
    let sarif = String::from_utf8(output.stdout).ok()?;
    if sarif.is_empty() {
        None
    } else {
        Some(sarif)
    }
}

#[cfg(test)]
mod build_compliance_report_tests {
    use super::*;

    fn check(name: &str, status: CheckStatus) -> ComplianceCheck {
        ComplianceCheck {
            name: name.into(),
            status,
            message: format!("msg for {name}"),
            severity: Severity::Info,
        }
    }

    #[test]
    fn test_compliant_when_no_failures() {
        let checks = vec![check("a", CheckStatus::Pass), check("b", CheckStatus::Warn)];
        let report =
            build_compliance_report(checks, "1.0.0", VersionSource::PinnedByProject, false);
        assert!(report.is_compliant);
        assert_eq!(report.checks.len(), 2);
    }

    #[test]
    fn test_non_compliant_when_any_fail() {
        let checks = vec![check("a", CheckStatus::Pass), check("b", CheckStatus::Fail)];
        let report =
            build_compliance_report(checks, "1.0.0", VersionSource::PinnedByProject, false);
        assert!(!report.is_compliant);
    }

    #[test]
    fn test_failures_only_filter_drops_non_fail_checks() {
        let checks = vec![
            check("p", CheckStatus::Pass),
            check("w", CheckStatus::Warn),
            check("s", CheckStatus::Skip),
            check("f", CheckStatus::Fail),
        ];
        let report = build_compliance_report(checks, "1.0.0", VersionSource::PinnedByProject, true);
        assert_eq!(report.checks.len(), 1);
        assert_eq!(report.checks[0].name, "f");
    }

    #[test]
    fn test_failures_only_false_keeps_all_checks() {
        let checks = vec![
            check("p", CheckStatus::Pass),
            check("w", CheckStatus::Warn),
            check("f", CheckStatus::Fail),
        ];
        let report =
            build_compliance_report(checks, "1.0.0", VersionSource::PinnedByProject, false);
        assert_eq!(report.checks.len(), 3);
    }

    #[test]
    fn test_project_version_propagates() {
        let report =
            build_compliance_report(vec![], "2.5.0", VersionSource::PinnedByProject, false);
        assert_eq!(report.project_version, "2.5.0");
        assert!(!report.current_version.is_empty());
    }

    #[test]
    fn test_empty_checks_compliant() {
        // No failures = compliant by definition
        let report =
            build_compliance_report(vec![], "1.0.0", VersionSource::PinnedByProject, false);
        assert!(report.is_compliant);
        assert_eq!(report.checks.len(), 0);
    }

    #[test]
    fn test_apply_exit_policy_returns_ok_when_compliant_no_warnings() {
        let report = ComplianceReport {
            project_version: "1.0".into(),
            project_version_source: VersionSource::PinnedByProject,
            summary: CheckSummary::default(),
            current_version: "1.0".into(),
            is_compliant: true,
            versions_behind: 0,
            checks: vec![],
            breaking_changes: vec![],
            recommendations: vec![],
            timestamp: Utc::now(),
            history: None,
        };
        // strict=false, no warnings → Ok
        assert!(apply_exit_policy(&report, false).is_ok());
    }

    #[test]
    fn test_apply_exit_policy_returns_ok_when_strict_but_no_warnings() {
        let report = ComplianceReport {
            project_version: "1.0".into(),
            project_version_source: VersionSource::PinnedByProject,
            summary: CheckSummary::default(),
            current_version: "1.0".into(),
            is_compliant: true,
            versions_behind: 0,
            checks: vec![check("p", CheckStatus::Pass)],
            breaking_changes: vec![],
            recommendations: vec![],
            timestamp: Utc::now(),
            history: None,
        };
        // is_compliant=true, no warnings even with strict → Ok (no exit)
        assert!(apply_exit_policy(&report, true).is_ok());
    }

    fn report_with(summary: CheckSummary, checks: Vec<ComplianceCheck>) -> ComplianceReport {
        ComplianceReport {
            project_version: "1.0".into(),
            project_version_source: VersionSource::PinnedByProject,
            is_compliant: summary.fail == 0,
            summary,
            current_version: "1.0".into(),
            versions_behind: 0,
            checks,
            breaking_changes: vec![],
            recommendations: vec![],
            timestamp: Utc::now(),
            history: None,
        }
    }

    /// #945: exit 2 on a COMPLIANT project was silent, against a `--help` that
    /// promises only "exit with error if non-compliant". The escalation stays —
    /// it is `--strict`'s only effect — but it must say so.
    #[test]
    fn strict_warning_escalation_explains_itself() {
        let summary = CheckSummary {
            total: 154,
            pass: 27,
            warn: 12,
            fail: 0,
            skip: 115,
        };
        let report = report_with(summary, vec![check("w", CheckStatus::Warn)]);

        let strict = exit_policy(&report, true);
        assert_eq!(strict.code, 2);
        let reason = strict.reason.expect("exit 2 must explain itself");
        assert!(reason.contains("--strict"), "{reason}");
        assert!(reason.contains("12 warning(s)"), "{reason}");
        assert!(reason.contains("COMPLIANT"), "{reason}");
        assert!(
            reason.contains("Without --strict this run exits 0"),
            "{reason}"
        );

        // The same report without --strict is a clean, silent 0.
        assert_eq!(
            exit_policy(&report, false),
            ExitPolicy {
                code: 0,
                reason: None
            }
        );
    }

    /// #945: the counts came from `report.checks`, which `--failures-only` has
    /// already filtered — so `comply check --strict --failures-only` exited 0
    /// on a tree where `comply check --strict` exited 2 (verified on the 3.30.0
    /// binary: rc 0 vs rc 2, both reporting `warn: 12`). A display flag must
    /// not move the exit code.
    #[test]
    fn failures_only_does_not_change_the_exit_code() {
        let summary = CheckSummary {
            total: 154,
            pass: 27,
            warn: 12,
            fail: 0,
            skip: 115,
        };
        // What `--failures-only` leaves behind: no failures, so no checks.
        let filtered = report_with(summary, vec![]);
        let unfiltered = report_with(summary, vec![check("w", CheckStatus::Warn)]);

        assert_eq!(exit_policy(&filtered, true).code, 2);
        assert_eq!(
            exit_policy(&filtered, true).code,
            exit_policy(&unfiltered, true).code
        );
    }

    #[test]
    fn failures_exit_1_whether_or_not_strict() {
        let summary = CheckSummary {
            total: 3,
            pass: 1,
            warn: 1,
            fail: 1,
            skip: 0,
        };
        let report = report_with(summary, vec![check("f", CheckStatus::Fail)]);
        for strict in [false, true] {
            let policy = exit_policy(&report, strict);
            assert_eq!(policy.code, 1, "strict={strict}");
            assert!(policy.reason.expect("reason").contains("1 check(s) failed"));
        }
    }

    /// `comply check -f sarif` used to print pmat's plain JSON report whenever
    /// the external `pv` linter was unavailable: no `$schema`, no `version`, no
    /// `runs[]` — a document GitHub's upload-sarif action rejects, and
    /// byte-identical to `-f json` apart from the timestamp.
    #[test]
    fn sarif_is_sarif_and_carries_pmats_own_checks() {
        let report = ComplianceReport {
            project_version: "1.0".into(),
            project_version_source: VersionSource::PinnedByProject,
            summary: CheckSummary::default(),
            current_version: "1.0".into(),
            is_compliant: false,
            versions_behind: 0,
            checks: vec![
                check("Version Currency", CheckStatus::Pass),
                ComplianceCheck {
                    name: "CB-030: O(1) Hooks".into(),
                    status: CheckStatus::Fail,
                    message: "hook missing".into(),
                    severity: Severity::Error,
                },
                ComplianceCheck {
                    name: "Quality Thresholds".into(),
                    status: CheckStatus::Warn,
                    message: "close to the limit".into(),
                    severity: Severity::Warning,
                },
                check("Disabled Check", CheckStatus::Skip),
            ],
            breaking_changes: vec![],
            recommendations: vec![],
            timestamp: Utc::now(),
            history: None,
        };

        let sarif = build_sarif(&report, Path::new("."));
        assert_eq!(sarif["version"], "2.1.0");
        assert!(sarif["$schema"].is_string(), "SARIF needs a $schema");
        let runs = sarif["runs"].as_array().expect("SARIF needs runs[]");
        assert_eq!(runs.len(), 1);
        assert_eq!(runs[0]["tool"]["driver"]["name"], "pmat");

        // One result per Fail/Warn, none for Pass/Skip.
        let results = runs[0]["results"].as_array().expect("results[]");
        assert_eq!(results.len(), 2, "{results:#?}");
        assert_eq!(results[0]["ruleId"], "CB-030");
        assert_eq!(results[0]["level"], "error");
        assert!(results[0]["message"]["text"]
            .as_str()
            .expect("message")
            .contains("hook missing"));
        assert_eq!(results[1]["ruleId"], "pmat/quality-thresholds");
        assert_eq!(results[1]["level"], "warning");

        // The document must NOT be the plain compliance report.
        assert!(sarif.get("checks").is_none());
        assert!(sarif.get("is_compliant").is_none());
    }

    /// #939: resolving the project's pinned version CREATED
    /// `.pmat/project.toml`, so `comply check` reported
    /// "not pinned by this project" on run 1 and "pinned in .pmat/project.toml"
    /// on run 2 of the identical command. Reading a pin must not write one.
    #[test]
    fn resolving_the_pinned_version_writes_nothing() {
        let dir = tempfile::TempDir::new().expect("tempdir");
        std::fs::write(dir.path().join("Cargo.toml"), "[package]\nname = \"x\"\n").expect("write");

        let (config, source) = load_project_config_with_source(dir.path()).expect("load");
        assert_eq!(source, VersionSource::InstalledPmatDefault);
        assert!(!config.pmat.version.is_empty());
        assert!(
            !dir.path().join(".pmat").exists(),
            "comply check must not create .pmat/ in the project it audits"
        );

        // And the verdict is stable across repeated reads.
        let (_, again) = load_project_config_with_source(dir.path()).expect("load again");
        assert_eq!(source, again);
    }

    /// #939, end to end: the whole compliance run on unchanged source must
    /// return the identical verdict twice. It did not — run 1 reported
    /// `fail: 2` (`Cargo.lock Present`, `CB-301 Reproducibility`) and run 2 of
    /// the identical command reported `fail: 0`, because run 1 had written the
    /// lockfile and the version pin it then read back as evidence.
    #[test]
    fn a_second_run_on_unchanged_source_returns_the_identical_verdict() {
        let dir = tempfile::TempDir::new().expect("tempdir");
        std::fs::create_dir_all(dir.path().join("src")).expect("mkdir");
        std::fs::write(
            dir.path().join("Cargo.toml"),
            "[package]\nname = \"idem\"\nversion = \"0.1.0\"\nedition = \"2021\"\n\n[dependencies]\n",
        )
        .expect("write manifest");
        std::fs::write(dir.path().join("src/lib.rs"), "//! x\n").expect("write lib");

        let first = compute_compliance_report(dir.path(), false).expect("run 1");
        let second = compute_compliance_report(dir.path(), false).expect("run 2");

        assert_eq!(first.summary, second.summary, "verdict moved on its own");
        assert_eq!(first.is_compliant, second.is_compliant);
        assert_eq!(
            first.project_version_source, second.project_version_source,
            "the audit pinned the version it then reported as pinned"
        );
        assert!(
            !dir.path().join("Cargo.lock").exists(),
            "comply check must not write Cargo.lock into the audited project"
        );
        assert!(
            !dir.path().join(".pmat/project.toml").exists(),
            "comply check must not pin the audited project - that is `comply init`"
        );
    }

    #[test]
    fn a_real_pin_is_reported_as_pinned() {
        let dir = tempfile::TempDir::new().expect("tempdir");
        std::fs::create_dir_all(dir.path().join(".pmat")).expect("mkdir");
        std::fs::write(
            dir.path().join(".pmat/project.toml"),
            "[pmat]\nversion = \"1.2.3\"\n",
        )
        .expect("write");

        let (config, source) = load_project_config_with_source(dir.path()).expect("load");
        assert_eq!(source, VersionSource::PinnedByProject);
        assert_eq!(config.pmat.version, "1.2.3");
    }

    #[test]
    fn sarif_rule_ids_prefer_the_cb_clause() {
        assert_eq!(sarif_rule_id("CB-1204: Verification Ladder"), "CB-1204");
        assert_eq!(
            sarif_rule_id("Cargo.lock Present"),
            "pmat/cargo-lock-present"
        );
        assert_eq!(
            sarif_rule_id("OIP Tarantula Patterns (CB-120 to CB-124)"),
            "CB-120"
        );
    }
}

// Provable-contracts enforcement helpers (shared by CB-1201 through CB-1209)
include!("check_pv_enforcement_helpers.rs");
// Provable-contracts enforcement checks (CB-1201, CB-1203)
include!("check_pv_enforcement.rs");
// Provable-contracts verification ladder (CB-1204 through CB-1207)
include!("check_pv_verification_ladder.rs");
// Provable-contracts quality gate (CB-1202, CB-1208, CB-1209)
include!("check_pv_quality_gate.rs");
include!("check_pv_quality.rs");
include!("check_contract_surfaces.rs");
include!("check_agent_contracts.rs");
include!("check_agent_iteration.rs");
include!("check_agent_autonomous.rs");
include!("check_commit_enforcement.rs");
include!("check_commit_enforcement_p2.rs");
include!("check_commit_enforcement_p3.rs");
include!("check_commit_enforcement_p4.rs");
include!("check_commit_enforcement_p5.rs");
include!("check_commit_enforcement_p6.rs");
include!("check_commit_enforcement_p7.rs");
include!("check_commit_enforcement_p8.rs");
include!("check_commit_enforcement_p9.rs");
include!("check_commit_enforcement_p10.rs");

// Split into submodule files for file_health (CB-040) compliance
include!("check_builders_foundation.rs");
include!("check_builders_contracts.rs");
include!("check_builders_commits.rs");
include!("check_builders_work.rs");
include!("check_individual_basic.rs");
include!("check_individual_cb.rs");
include!("check_individual_ci.rs");

include!("check_handlers_tests_inline.rs");
include!("check_pv_enforcement_helpers_tests.rs");

include!("check_path_guard_tests.rs");
include!("check_empty_project_guard_tests.rs");
include!("check_readonly_and_exemption_tests.rs");

#[cfg_attr(coverage_nightly, coverage(off))]
#[cfg(test)]
mod comply_concurrency_tests {
    use super::*;
    use serial_test::serial;

    const KEY: &str = "PMAT_COMPLY_JOBS";

    /// Restores `PMAT_COMPLY_JOBS` on drop, so a failing assertion cannot leak
    /// the override into the rest of the suite. `#[serial]` on top of it because
    /// the process environment is shared state and these tests write to it —
    /// without both, a green run proves nothing about which value was read.
    struct JobsEnvGuard(Option<String>);

    impl JobsEnvGuard {
        fn set(value: &str) -> Self {
            let prev = std::env::var(KEY).ok();
            std::env::set_var(KEY, value);
            Self(prev)
        }
    }

    impl Drop for JobsEnvGuard {
        fn drop(&mut self) {
            match self.0.take() {
                Some(v) => std::env::set_var(KEY, v),
                None => std::env::remove_var(KEY),
            }
        }
    }

    /// The bound exists because an unbounded run peaked at 58.7 GB against this
    /// repo and ~94 GB against aprender, drove load average to 75 and tripped
    /// the machine's OOM guard. Memory scales LINEARLY with workers (~4 GB
    /// each), so "one worker per CPU" asks a 64-core box for 256 GB.
    #[test]
    #[serial]
    fn concurrency_is_bounded_regardless_of_machine_size() {
        let _guard = JobsEnvGuard::set("");
        std::env::remove_var(KEY);
        let jobs = comply_concurrency(13);
        assert!(
            jobs >= 1,
            "at least one worker must always run — refusing to check is worse than checking slowly"
        );
        assert!(
            jobs <= COMPLY_MAX_DEFAULT_JOBS,
            "default concurrency {jobs} exceeded the ceiling {COMPLY_MAX_DEFAULT_JOBS}; \
             on this machine that is ~{} GB of peak RSS",
            (jobs as u64 * COMPLY_BYTES_PER_WORKER) / (1024 * 1024 * 1024)
        );
    }

    /// More workers than groups is pure waste — they would idle while still
    /// costing their share of memory if anything were scheduled onto them.
    #[test]
    #[serial]
    fn concurrency_never_exceeds_the_group_count() {
        let _guard = JobsEnvGuard::set("");
        std::env::remove_var(KEY);
        assert_eq!(comply_concurrency(1), 1);
        assert!(comply_concurrency(2) <= 2);
    }

    /// The estimate is a default, not a policy. Anyone who knows their machine
    /// can raise or lower it.
    #[test]
    #[serial]
    fn env_override_wins() {
        let _guard = JobsEnvGuard::set("7");
        assert_eq!(
            comply_concurrency(13),
            7,
            "PMAT_COMPLY_JOBS must override the computed bound"
        );
    }

    /// A garbage or zero override falls back to the computed bound rather than
    /// producing a zero-thread pool.
    #[test]
    #[serial]
    fn zero_or_garbage_override_falls_back() {
        for bad in ["0", "not-a-number", ""] {
            let _guard = JobsEnvGuard::set(bad);
            let jobs = comply_concurrency(13);
            assert!(
                (1..=COMPLY_MAX_DEFAULT_JOBS).contains(&jobs),
                "override {bad:?} produced {jobs} workers"
            );
        }
    }
}

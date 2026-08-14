#![cfg_attr(coverage_nightly, coverage(off))]
// CB-140: Mono-spec structure enforcement
// CB-141: Memory profiling infrastructure
// CB-142: SWE-CI EvoScore

use super::types::*;
use std::path::Path;

/// Line ceiling every mono-spec markdown file is held to.
const MAX_SPEC_LINES: usize = 500;

/// Build a CB-140 result. Every return path of [`check_mono_spec_structure`]
/// used to repeat the name/status/message/severity literal.
fn mono_spec_check(status: CheckStatus, severity: Severity, message: String) -> ComplianceCheck {
    ComplianceCheck {
        name: "CB-140: Mono-Spec Structure".into(),
        status,
        message,
        severity,
    }
}

/// True for a regular `*.md` file (directories named `foo.md` do not count).
fn is_markdown_file(path: &Path) -> bool {
    path.is_file() && path.extension().and_then(|e| e.to_str()) == Some("md")
}

/// The single "this spec file is too long" rule.
///
/// It used to be written out twice — once for component files and once for the
/// root spec — with the root copy hard-coding the file name. One rule, one
/// implementation: an unreadable file yields `None` (no issue), exactly as
/// both copies did.
fn oversized_spec_issue(path: &Path) -> Option<String> {
    let content = std::fs::read_to_string(path).ok()?;
    let line_count = content.lines().count();
    if line_count <= MAX_SPEC_LINES {
        return None;
    }
    let name = path.file_name().and_then(|n| n.to_str())?;
    Some(format!("{name}: {line_count} lines (max {MAX_SPEC_LINES})"))
}

/// Check 3: only `pmat-spec.md` may sit at the root of `docs/specifications/`.
fn loose_spec_issues(spec_dir: &Path) -> Vec<String> {
    let Ok(entries) = std::fs::read_dir(spec_dir) else {
        return Vec::new();
    };
    entries
        .flatten()
        .filter_map(|entry| {
            let path = entry.path();
            if !is_markdown_file(&path) {
                return None;
            }
            let name = path.file_name()?.to_str()?;
            (name != "pmat-spec.md")
                .then(|| format!("Loose spec file: {name} (should be in components/)"))
        })
        .collect()
}

/// Check 4: every component spec under the ceiling, and at least one of them.
///
/// An absent (or unreadable) `components/` directory yields no issues here —
/// that case is reported once by check 2 rather than twice.
fn component_spec_issues(components_dir: &Path) -> Vec<String> {
    let Ok(entries) = std::fs::read_dir(components_dir) else {
        return Vec::new();
    };
    let mut issues = Vec::new();
    let mut component_count = 0usize;
    for entry in entries.flatten() {
        let path = entry.path();
        if !is_markdown_file(&path) {
            continue;
        }
        component_count += 1;
        issues.extend(oversized_spec_issue(&path));
    }
    if component_count == 0 {
        issues.push("No component spec files found in components/".to_string());
    }
    issues
}

/// CB-140: Validate mono-spec structure
///
/// Checks:
/// 1. docs/specifications/pmat-spec.md exists
/// 2. docs/specifications/components/ directory exists with sub-specs
/// 3. No loose spec files in docs/specifications/ (only pmat-spec.md)
/// 4. All component files (and the root spec) are under 500 lines
#[provable_contracts_macros::contract("pmat-core.yaml", equation = "path_exists")]
pub(crate) fn check_mono_spec_structure(project_path: &Path) -> ComplianceCheck {
    let spec_dir = project_path.join("docs").join("specifications");
    let root_spec = spec_dir.join("pmat-spec.md");
    let components_dir = spec_dir.join("components");

    // Check 1: Root spec exists
    if !root_spec.exists() {
        return mono_spec_check(
            CheckStatus::Skip,
            Severity::Info,
            "No docs/specifications/pmat-spec.md found (not a spec-managed project)".to_string(),
        );
    }

    let mut issues = Vec::new();
    // Check 2: Components directory exists (`is_dir` is false when absent).
    if !components_dir.is_dir() {
        issues.push("Missing docs/specifications/components/ directory".to_string());
    }
    issues.extend(loose_spec_issues(&spec_dir));
    issues.extend(component_spec_issues(&components_dir));
    issues.extend(oversized_spec_issue(&root_spec));

    if issues.is_empty() {
        mono_spec_check(
            CheckStatus::Pass,
            Severity::Info,
            "Mono-spec structure valid (root spec + components, all under 500 lines)".to_string(),
        )
    } else {
        mono_spec_check(
            CheckStatus::Warn,
            Severity::Warning,
            format!(
                "{} issue(s):\n{}",
                issues.len(),
                format_violation_list(&issues)
            ),
        )
    }
}

/// CB-141: Check for memory profiling infrastructure
///
/// Checks:
/// 1. dhat (or similar profiler) in Cargo.toml dev-dependencies
/// 2. Profile binary exists (examples/dhat_profile.rs or similar)
/// 3. Memory baseline file exists (.pmat-metrics/memory-baseline.json)
#[provable_contracts_macros::contract("pmat-core.yaml", equation = "path_exists")]
pub(crate) fn check_memory_profiling(project_path: &Path) -> ComplianceCheck {
    let cargo_toml = project_path.join("Cargo.toml");

    // Skip if not a Rust project
    if !cargo_toml.exists() {
        return ComplianceCheck {
            name: "CB-141: Memory Profiling".into(),
            status: CheckStatus::Skip,
            message: "Not a Rust project (no Cargo.toml)".into(),
            severity: Severity::Info,
        };
    }

    let cargo_content = std::fs::read_to_string(&cargo_toml).unwrap_or_default();
    let mut issues = Vec::new();
    let mut score_penalty = 0;

    // Check 1: Profiler dependency
    let has_profiler = cargo_content.contains("dhat")
        || cargo_content.contains("jemalloc")
        || cargo_content.contains("heaptrack")
        || cargo_content.contains("mimalloc");

    if !has_profiler {
        issues.push("No memory profiler dependency (dhat, jemalloc, heaptrack)".to_string());
        score_penalty += 5;
    }

    // Check 2: Profile binary or benchmark
    let has_profile_binary = project_path
        .join("examples")
        .join("dhat_profile.rs")
        .exists()
        || project_path.join("benches").join("memory.rs").exists()
        || has_profile_example_in_dir(project_path);

    if !has_profile_binary {
        issues.push(
            "No memory profile binary (examples/dhat_profile.rs or benches/memory.rs)".to_string(),
        );
        score_penalty += 3;
    }

    // Check 3: Baseline file
    let has_baseline = project_path
        .join(".pmat-metrics")
        .join("memory-baseline.json")
        .exists();

    if !has_baseline {
        issues.push("No memory baseline (.pmat-metrics/memory-baseline.json)".to_string());
        score_penalty += 2;
    }

    if issues.is_empty() {
        ComplianceCheck {
            name: "CB-141: Memory Profiling".into(),
            status: CheckStatus::Pass,
            message: "Memory profiling infrastructure present (profiler + binary + baseline)"
                .into(),
            severity: Severity::Info,
        }
    } else {
        ComplianceCheck {
            name: "CB-141: Memory Profiling".into(),
            status: CheckStatus::Warn,
            message: format!(
                "-{} points penalty:\n{}",
                score_penalty,
                format_violation_list(&issues)
            ),
            severity: Severity::Warning,
        }
    }
}

/// Check if any example file contains dhat or memory profiling patterns
fn has_profile_example_in_dir(project_path: &Path) -> bool {
    let examples_dir = project_path.join("examples");
    if !examples_dir.exists() {
        return false;
    }
    if let Ok(entries) = std::fs::read_dir(&examples_dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.extension().and_then(|e| e.to_str()) == Some("rs") {
                if let Ok(content) = std::fs::read_to_string(&path) {
                    if content.contains("dhat::") || content.contains("global_allocator") {
                        return true;
                    }
                }
            }
        }
    }
    false
}

/// One recorded commit: `(commit, pass, total)`.
type CommitTests = (String, u64, u64);

/// Recency weighting for [`evoscore`]. Later commits count for more.
const EVOSCORE_GAMMA: f64 = 1.5;

/// Fewer than this many recorded commits and there is nothing to trend.
const EVOSCORE_MIN_COMMITS: usize = 3;

/// Build a CB-142 result.
fn evoscore_check(status: CheckStatus, severity: Severity, message: String) -> ComplianceCheck {
    ComplianceCheck {
        name: "CB-142: SWE-CI EvoScore".into(),
        status,
        message,
        severity,
    }
}

/// `commit-*<suffix>` files in `metrics_dir`, sorted by name so the series is
/// chronological. Missing/unreadable directory yields an empty series.
fn sorted_metric_files(metrics_dir: &Path, suffix: &str) -> Vec<std::path::PathBuf> {
    let Ok(entries) = std::fs::read_dir(metrics_dir) else {
        return Vec::new();
    };
    let mut files: Vec<std::path::PathBuf> = entries
        .flatten()
        .map(|e| e.path())
        .filter(|p| {
            p.file_name()
                .and_then(|n| n.to_str())
                .is_some_and(|n| n.starts_with("commit-") && n.ends_with(suffix))
        })
        .collect();
    files.sort();
    files
}

/// Parse one metric file into JSON; unreadable or malformed files are skipped.
fn read_metric_json(path: &Path) -> Option<serde_json::Value> {
    serde_json::from_str(&std::fs::read_to_string(path).ok()?).ok()
}

/// Pull `(commit, pass, total)` out of one record.
///
/// `record` carries the commit id; `counts` carries `pass`/`total` — the same
/// object for `-tests.json`, the nested `tests` object for `-meta.json`.
/// `total == 0` means nothing was measured for that commit, so it is dropped
/// rather than counted as a 0/0 data point.
fn commit_tests_from(
    record: &serde_json::Value,
    counts: &serde_json::Value,
) -> Option<CommitTests> {
    let total = counts["total"].as_u64().unwrap_or(0);
    if total == 0 {
        return None;
    }
    let pass = counts["pass"].as_u64().unwrap_or(0);
    let commit = record["commit"].as_str().unwrap_or("unknown").to_string();
    Some((commit, pass, total))
}

/// Series recorded by `pmat test --record` as `commit-*-tests.json`.
fn read_tests_series(metrics_dir: &Path) -> Vec<CommitTests> {
    sorted_metric_files(metrics_dir, "-tests.json")
        .iter()
        .filter_map(|path| {
            let data = read_metric_json(path)?;
            commit_tests_from(&data, &data)
        })
        .collect()
}

/// Fallback series: `commit-*-meta.json` files that carry a `tests` object.
fn read_meta_series(metrics_dir: &Path) -> Vec<CommitTests> {
    sorted_metric_files(metrics_dir, "-meta.json")
        .iter()
        .filter_map(|path| {
            let data = read_metric_json(path)?;
            let tests = data.get("tests")?;
            commit_tests_from(&data, tests)
        })
        .collect()
}

/// Attainment of one commit: progress from the baseline toward the best pass
/// count ever observed, or the relative regression when it fell below baseline.
fn commit_attainment(current_pass: f64, base_pass: f64, oracle_pass: f64) -> f64 {
    if current_pass >= base_pass {
        let gap = oracle_pass - base_pass;
        if gap > 0.0 {
            (current_pass - base_pass) / gap
        } else {
            1.0 // Already at oracle level
        }
    } else if base_pass > 0.0 {
        (current_pass - base_pass) / base_pass
    } else {
        0.0
    }
}

/// Recency-weighted mean attainment over the series (weights `gamma^i`).
///
/// The first commit is the baseline and is not scored against itself.
fn evoscore(series: &[CommitTests], gamma: f64) -> f64 {
    let Some(base) = series.first() else {
        return 0.0;
    };
    let base_pass = base.1 as f64;
    // Oracle: best observed pass count (proxy for ideal)
    let oracle_pass = series.iter().map(|(_, p, _)| *p).max().unwrap_or(0) as f64;

    let mut weighted_sum = 0.0;
    let mut weight_total = 0.0;
    for (i, (_commit, pass, _total)) in series.iter().enumerate().skip(1) {
        let weight = gamma.powi(i as i32);
        weighted_sum += weight * commit_attainment(*pass as f64, base_pass, oracle_pass);
        weight_total += weight;
    }

    if weight_total > 0.0 {
        weighted_sum / weight_total
    } else {
        0.0
    }
}

/// Verdict and human-readable trend for a computed EvoScore.
fn evoscore_verdict(score: f64) -> (CheckStatus, Severity, &'static str) {
    if score >= 0.5 {
        (
            CheckStatus::Pass,
            Severity::Info,
            "Consistent improvement trend",
        )
    } else if score >= 0.0 {
        (
            CheckStatus::Warn,
            Severity::Warning,
            "Mixed improvement/regression trend",
        )
    } else {
        (CheckStatus::Fail, Severity::Error, "Net regression trend")
    }
}

/// CB-142: SWE-CI EvoScore from recorded per-commit test results
///
/// Computes evolution score from test pass/fail data across commits.
/// Returns Skip if insufficient data.
#[provable_contracts_macros::contract("pmat-core.yaml", equation = "path_exists")]
pub(crate) fn check_swe_ci_evoscore(project_path: &Path) -> ComplianceCheck {
    let metrics_dir = project_path.join(".pmat-metrics");

    // `-tests.json` is the primary record; `-meta.json` is consulted only when
    // no tests record produced a single usable data point.
    let mut test_data = read_tests_series(&metrics_dir);
    if test_data.is_empty() {
        test_data = read_meta_series(&metrics_dir);
    }

    if test_data.len() < EVOSCORE_MIN_COMMITS {
        return evoscore_check(
            CheckStatus::Skip,
            Severity::Info,
            format!(
                "Insufficient commit test data ({} commits, need >= {EVOSCORE_MIN_COMMITS}). \
                 Record test results with: pmat test --record",
                test_data.len()
            ),
        );
    }

    let score = evoscore(&test_data, EVOSCORE_GAMMA);
    let (status, severity, trend) = evoscore_verdict(score);
    evoscore_check(
        status,
        severity,
        format!(
            "EvoScore: {score:.3} (gamma={EVOSCORE_GAMMA:.1}, {} commits). {trend}",
            test_data.len()
        ),
    )
}

#[cfg_attr(coverage_nightly, coverage(off))]
#[cfg(test)]
mod tests_mono_spec {
    use super::*;

    #[test]
    fn test_mono_spec_skip_no_spec_dir() {
        let tmp = tempfile::tempdir().expect("create tempdir");
        let result = check_mono_spec_structure(tmp.path());
        assert_eq!(result.status, CheckStatus::Skip);
    }

    #[test]
    fn test_mono_spec_pass() {
        let tmp = tempfile::tempdir().expect("create tempdir");
        let spec_dir = tmp.path().join("docs").join("specifications");
        let comp_dir = spec_dir.join("components");
        std::fs::create_dir_all(&comp_dir).unwrap();
        std::fs::write(spec_dir.join("pmat-spec.md"), "# Spec\n\nTOC here\n").unwrap();
        std::fs::write(comp_dir.join("quality.md"), "# Quality\n\nDetails\n").unwrap();
        let result = check_mono_spec_structure(tmp.path());
        assert_eq!(result.status, CheckStatus::Pass);
    }

    #[test]
    fn test_mono_spec_loose_files() {
        let tmp = tempfile::tempdir().expect("create tempdir");
        let spec_dir = tmp.path().join("docs").join("specifications");
        let comp_dir = spec_dir.join("components");
        std::fs::create_dir_all(&comp_dir).unwrap();
        std::fs::write(spec_dir.join("pmat-spec.md"), "# Spec\n").unwrap();
        std::fs::write(spec_dir.join("old-spec.md"), "# Old\n").unwrap();
        std::fs::write(comp_dir.join("quality.md"), "# Quality\n").unwrap();
        let result = check_mono_spec_structure(tmp.path());
        assert_eq!(result.status, CheckStatus::Warn);
        assert!(result.message.contains("Loose spec file"));
    }

    #[test]
    fn test_mono_spec_oversized_component() {
        let tmp = tempfile::tempdir().expect("create tempdir");
        let spec_dir = tmp.path().join("docs").join("specifications");
        let comp_dir = spec_dir.join("components");
        std::fs::create_dir_all(&comp_dir).unwrap();
        std::fs::write(spec_dir.join("pmat-spec.md"), "# Spec\n").unwrap();
        // Create a 501-line file
        let big_content: String = (0..501).map(|i| format!("line {}\n", i)).collect();
        std::fs::write(comp_dir.join("big.md"), &big_content).unwrap();
        let result = check_mono_spec_structure(tmp.path());
        assert_eq!(result.status, CheckStatus::Warn);
        assert!(result.message.contains("501 lines"));
    }

    #[test]
    fn test_memory_profiling_skip_non_rust() {
        let tmp = tempfile::tempdir().expect("create tempdir");
        let result = check_memory_profiling(tmp.path());
        assert_eq!(result.status, CheckStatus::Skip);
    }

    #[test]
    fn test_memory_profiling_warn_no_infra() {
        let tmp = tempfile::tempdir().expect("create tempdir");
        std::fs::write(
            tmp.path().join("Cargo.toml"),
            "[package]\nname = \"test\"\n",
        )
        .unwrap();
        let result = check_memory_profiling(tmp.path());
        assert_eq!(result.status, CheckStatus::Warn);
        assert!(result.message.contains("-10 points"));
    }

    #[test]
    fn test_memory_profiling_pass_with_dhat() {
        let tmp = tempfile::tempdir().expect("create tempdir");
        std::fs::write(
            tmp.path().join("Cargo.toml"),
            "[package]\nname = \"test\"\n[dev-dependencies]\ndhat = \"0.3\"\n",
        )
        .unwrap();
        let examples_dir = tmp.path().join("examples");
        std::fs::create_dir_all(&examples_dir).unwrap();
        std::fs::write(
            examples_dir.join("dhat_profile.rs"),
            "fn main() { let _p = dhat::Profiler::new_heap(); }",
        )
        .unwrap();
        let metrics_dir = tmp.path().join(".pmat-metrics");
        std::fs::create_dir_all(&metrics_dir).unwrap();
        std::fs::write(
            metrics_dir.join("memory-baseline.json"),
            r#"{"peak_memory_bytes": 100000}"#,
        )
        .unwrap();
        let result = check_memory_profiling(tmp.path());
        assert_eq!(result.status, CheckStatus::Pass);
    }

    #[test]
    fn test_evoscore_skip_no_data() {
        let tmp = tempfile::tempdir().expect("create tempdir");
        let result = check_swe_ci_evoscore(tmp.path());
        assert_eq!(result.status, CheckStatus::Skip);
    }

    #[test]
    fn test_evoscore_pass_improving() {
        let tmp = tempfile::tempdir().expect("create tempdir");
        let metrics_dir = tmp.path().join(".pmat-metrics");
        std::fs::create_dir_all(&metrics_dir).unwrap();
        // Simulate improving test pass counts across 4 commits
        for (i, pass) in [80, 85, 90, 95].iter().enumerate() {
            std::fs::write(
                metrics_dir.join(format!("commit-{:04}-tests.json", i)),
                format!(r#"{{"commit":"abc{}","pass":{},"total":100}}"#, i, pass),
            )
            .unwrap();
        }
        let result = check_swe_ci_evoscore(tmp.path());
        assert_eq!(result.status, CheckStatus::Pass);
        assert!(result.message.contains("EvoScore"));
    }

    #[test]
    fn test_evoscore_fail_regressing() {
        let tmp = tempfile::tempdir().expect("create tempdir");
        let metrics_dir = tmp.path().join(".pmat-metrics");
        std::fs::create_dir_all(&metrics_dir).unwrap();
        // Simulate regressing test pass counts
        for (i, pass) in [90, 80, 70, 60].iter().enumerate() {
            std::fs::write(
                metrics_dir.join(format!("commit-{:04}-tests.json", i)),
                format!(r#"{{"commit":"abc{}","pass":{},"total":100}}"#, i, pass),
            )
            .unwrap();
        }
        let result = check_swe_ci_evoscore(tmp.path());
        assert_eq!(result.status, CheckStatus::Fail);
    }

    // ==================================================================
    // #970/#971/#972: these three checks must stay decomposed.
    //
    // RED before the refactor: check_swe_ci_evoscore measured cognitive 85 and
    // check_mono_spec_structure 62, against the analyzer's own error threshold
    // of 30 (`pmat analyze complexity --file`). check_wasm_ffi_contracts is
    // here because #970 reported it at 179; it was decomposed in e430b397c and
    // this pins it so it cannot grow back unnoticed.
    // ==================================================================

    /// The `error`-severity cognitive ceiling `pmat analyze complexity` applies.
    const COGNITIVE_ERROR_THRESHOLD: u32 = 30;

    async fn cognitive_of(rel_path: &str, function: &str) -> u32 {
        use crate::services::accurate_complexity_analyzer::AccurateComplexityAnalyzer;
        let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join(rel_path);
        let result = AccurateComplexityAnalyzer::new()
            .analyze_file(&path)
            .await
            .unwrap_or_else(|e| panic!("analyze {rel_path}: {e}"));
        result
            .functions
            .iter()
            .find(|f| f.name == function)
            .unwrap_or_else(|| panic!("{function} not found in {rel_path}"))
            .cognitive_complexity
    }

    #[tokio::test]
    async fn test_comply_checks_stay_under_cognitive_ceiling() {
        const TARGETS: &[(&str, &str)] = &[
            (
                "src/cli/handlers/comply_handlers/check_handlers/check_mono_spec.rs",
                "check_mono_spec_structure",
            ),
            (
                "src/cli/handlers/comply_handlers/check_handlers/check_mono_spec.rs",
                "check_swe_ci_evoscore",
            ),
            (
                "src/cli/handlers/comply_handlers/check_handlers/check_contract_surfaces.rs",
                "check_wasm_ffi_contracts",
            ),
        ];
        let mut over = Vec::new();
        for (file, function) in TARGETS {
            let cognitive = cognitive_of(file, function).await;
            if cognitive > COGNITIVE_ERROR_THRESHOLD {
                over.push(format!("{function}: cognitive {cognitive}"));
            }
        }
        assert!(
            over.is_empty(),
            "cognitive complexity ceiling is {COGNITIVE_ERROR_THRESHOLD}; over: {over:?}"
        );
    }

    // ==================================================================
    // CB-140 behaviour pinning (refactor of #972 must not move a verdict)
    // ==================================================================

    fn mono_fixture(kind: &str) -> tempfile::TempDir {
        let tmp = tempfile::tempdir().expect("create tempdir");
        let spec = tmp.path().join("docs").join("specifications");
        let comp = spec.join("components");
        let big: String = (0..501).map(|i| format!("l {i}\n")).collect();
        std::fs::create_dir_all(&spec).unwrap();
        match kind {
            "no_components" => {
                std::fs::write(spec.join("pmat-spec.md"), "# Spec\n").unwrap();
            }
            "empty_components" => {
                std::fs::create_dir_all(&comp).unwrap();
                std::fs::write(spec.join("pmat-spec.md"), "# Spec\n").unwrap();
            }
            "big_root" => {
                std::fs::create_dir_all(&comp).unwrap();
                std::fs::write(spec.join("pmat-spec.md"), &big).unwrap();
                std::fs::write(comp.join("a.md"), "# A\n").unwrap();
            }
            "exactly_500" => {
                std::fs::create_dir_all(&comp).unwrap();
                std::fs::write(spec.join("pmat-spec.md"), "# Spec\n").unwrap();
                let exact: String = (0..500).map(|i| format!("l {i}\n")).collect();
                std::fs::write(comp.join("edge.md"), &exact).unwrap();
            }
            "components_is_file" => {
                std::fs::write(spec.join("pmat-spec.md"), "# Spec\n").unwrap();
                std::fs::write(spec.join("components"), "not a dir").unwrap();
            }
            "non_md_noise" => {
                std::fs::create_dir_all(&comp).unwrap();
                std::fs::write(spec.join("pmat-spec.md"), "# Spec\n").unwrap();
                std::fs::write(spec.join("notes.txt"), "x").unwrap();
                std::fs::create_dir_all(spec.join("subdir")).unwrap();
                std::fs::write(comp.join("a.md"), "# A\n").unwrap();
                std::fs::write(comp.join("b.txt"), "x").unwrap();
            }
            "nested_only" => {
                std::fs::create_dir_all(comp.join("sub")).unwrap();
                std::fs::write(spec.join("pmat-spec.md"), "# Spec\n").unwrap();
                std::fs::write(comp.join("sub").join("deep.md"), "# D\n").unwrap();
            }
            other => panic!("unknown fixture {other}"),
        }
        tmp
    }

    #[test]
    fn test_mono_spec_missing_components_dir() {
        let tmp = mono_fixture("no_components");
        let r = check_mono_spec_structure(tmp.path());
        assert_eq!(r.status, CheckStatus::Warn);
        assert!(r.message.contains("1 issue(s):"), "{}", r.message);
        assert!(r
            .message
            .contains("Missing docs/specifications/components/ directory"));
    }

    #[test]
    fn test_mono_spec_empty_components_dir() {
        let tmp = mono_fixture("empty_components");
        let r = check_mono_spec_structure(tmp.path());
        assert_eq!(r.status, CheckStatus::Warn);
        assert!(r
            .message
            .contains("No component spec files found in components/"));
        // The directory exists, so check 2 must NOT also fire.
        assert!(!r
            .message
            .contains("Missing docs/specifications/components/"));
    }

    #[test]
    fn test_mono_spec_oversized_root_spec() {
        let tmp = mono_fixture("big_root");
        let r = check_mono_spec_structure(tmp.path());
        assert_eq!(r.status, CheckStatus::Warn);
        // Root spec and component files share ONE 500-line rule and one message.
        assert!(
            r.message.contains("pmat-spec.md: 501 lines (max 500)"),
            "{}",
            r.message
        );
    }

    #[test]
    fn test_mono_spec_exactly_500_lines_is_not_a_violation() {
        let tmp = mono_fixture("exactly_500");
        assert_eq!(
            check_mono_spec_structure(tmp.path()).status,
            CheckStatus::Pass
        );
    }

    #[test]
    fn test_mono_spec_components_path_that_is_a_file() {
        let tmp = mono_fixture("components_is_file");
        let r = check_mono_spec_structure(tmp.path());
        assert_eq!(r.status, CheckStatus::Warn);
        assert!(r
            .message
            .contains("Missing docs/specifications/components/ directory"));
        // A non-directory `components` must not also be counted as a loose
        // spec file nor as an empty components directory.
        assert!(r.message.contains("1 issue(s):"), "{}", r.message);
    }

    #[test]
    fn test_mono_spec_ignores_non_markdown_and_subdirs() {
        let tmp = mono_fixture("non_md_noise");
        assert_eq!(
            check_mono_spec_structure(tmp.path()).status,
            CheckStatus::Pass
        );
    }

    #[test]
    fn test_mono_spec_component_scan_is_not_recursive() {
        // Nested component files do not count; the check looks one level deep.
        let tmp = mono_fixture("nested_only");
        let r = check_mono_spec_structure(tmp.path());
        assert_eq!(r.status, CheckStatus::Warn);
        assert!(r
            .message
            .contains("No component spec files found in components/"));
    }

    // ==================================================================
    // CB-142 behaviour pinning (refactor of #971 must not move a verdict)
    // ==================================================================

    fn evo_dir() -> tempfile::TempDir {
        let tmp = tempfile::tempdir().expect("create tempdir");
        std::fs::create_dir_all(tmp.path().join(".pmat-metrics")).unwrap();
        tmp
    }

    fn write_tests_record(tmp: &tempfile::TempDir, i: usize, pass: u64, total: u64) {
        std::fs::write(
            tmp.path()
                .join(".pmat-metrics")
                .join(format!("commit-{i:04}-tests.json")),
            format!(r#"{{"commit":"c{i}","pass":{pass},"total":{total}}}"#),
        )
        .unwrap();
    }

    fn write_meta_record(tmp: &tempfile::TempDir, i: usize, body: &str) {
        std::fs::write(
            tmp.path()
                .join(".pmat-metrics")
                .join(format!("commit-{i:04}-meta.json")),
            body,
        )
        .unwrap();
    }

    #[test]
    fn test_evoscore_flat_series_scores_one() {
        // Oracle == baseline, so the gap is zero and every commit is "at
        // oracle level" — 1.000, not a divide-by-zero NaN.
        let tmp = evo_dir();
        for i in 0..4 {
            write_tests_record(&tmp, i, 50, 100);
        }
        let r = check_swe_ci_evoscore(tmp.path());
        assert_eq!(r.status, CheckStatus::Pass);
        assert!(r.message.contains("EvoScore: 1.000"), "{}", r.message);
    }

    #[test]
    fn test_evoscore_improving_series_exact_score() {
        let tmp = evo_dir();
        for (i, p) in [80u64, 85, 90, 95].iter().enumerate() {
            write_tests_record(&tmp, i, *p, 100);
        }
        let r = check_swe_ci_evoscore(tmp.path());
        assert_eq!(r.status, CheckStatus::Pass);
        assert!(
            r.message
                .contains("EvoScore: 0.754 (gamma=1.5, 4 commits). Consistent improvement trend"),
            "{}",
            r.message
        );
    }

    #[test]
    fn test_evoscore_regressing_series_exact_score() {
        let tmp = evo_dir();
        for (i, p) in [90u64, 80, 70, 60].iter().enumerate() {
            write_tests_record(&tmp, i, *p, 100);
        }
        let r = check_swe_ci_evoscore(tmp.path());
        assert_eq!(r.status, CheckStatus::Fail);
        assert!(
            r.message
                .contains("EvoScore: -0.251 (gamma=1.5, 4 commits). Net regression trend"),
            "{}",
            r.message
        );
    }

    #[test]
    fn test_evoscore_zero_total_records_are_dropped_not_counted() {
        // A record with total == 0 measured nothing; counting it would let a
        // project reach the 3-commit minimum without any test data.
        let tmp = evo_dir();
        write_tests_record(&tmp, 0, 0, 0);
        write_tests_record(&tmp, 1, 10, 100);
        write_tests_record(&tmp, 2, 20, 100);
        let r = check_swe_ci_evoscore(tmp.path());
        assert_eq!(r.status, CheckStatus::Skip);
        assert!(
            r.message.contains("(2 commits, need >= 3)"),
            "{}",
            r.message
        );
    }

    #[test]
    fn test_evoscore_malformed_record_is_skipped() {
        let tmp = evo_dir();
        std::fs::write(
            tmp.path()
                .join(".pmat-metrics")
                .join("commit-0000-tests.json"),
            "{not json",
        )
        .unwrap();
        for (i, p) in [10u64, 20, 30].iter().enumerate() {
            write_tests_record(&tmp, i + 1, *p, 100);
        }
        let r = check_swe_ci_evoscore(tmp.path());
        assert_eq!(r.status, CheckStatus::Pass);
        assert!(r.message.contains("3 commits"), "{}", r.message);
    }

    #[test]
    fn test_evoscore_meta_fallback_used_when_no_tests_records() {
        let tmp = evo_dir();
        for (i, p) in [10u64, 40, 70, 90].iter().enumerate() {
            write_meta_record(
                &tmp,
                i,
                &format!(r#"{{"commit":"m{i}","tests":{{"pass":{p},"total":100}}}}"#),
            );
        }
        let r = check_swe_ci_evoscore(tmp.path());
        assert_eq!(r.status, CheckStatus::Pass);
        assert!(r.message.contains("4 commits"), "{}", r.message);
    }

    #[test]
    fn test_evoscore_meta_without_tests_key_yields_no_data() {
        let tmp = evo_dir();
        for i in 0..4 {
            write_meta_record(&tmp, i, &format!(r#"{{"commit":"m{i}"}}"#));
        }
        let r = check_swe_ci_evoscore(tmp.path());
        assert_eq!(r.status, CheckStatus::Skip);
        assert!(
            r.message.contains("(0 commits, need >= 3)"),
            "{}",
            r.message
        );
    }

    #[test]
    fn test_evoscore_meta_ignored_when_any_tests_record_parsed() {
        // Two usable tests.json records: the series is non-empty, so the meta
        // fallback stays shut and the result is Skip at 2 — NOT 5.
        let tmp = evo_dir();
        write_tests_record(&tmp, 0, 10, 100);
        write_tests_record(&tmp, 1, 20, 100);
        for (i, p) in [30u64, 40, 50].iter().enumerate() {
            write_meta_record(
                &tmp,
                i,
                &format!(r#"{{"commit":"m{i}","tests":{{"pass":{p},"total":100}}}}"#),
            );
        }
        let r = check_swe_ci_evoscore(tmp.path());
        assert_eq!(r.status, CheckStatus::Skip);
        assert!(
            r.message.contains("(2 commits, need >= 3)"),
            "{}",
            r.message
        );
    }

    #[test]
    fn test_evoscore_ignores_unrelated_metric_files() {
        let tmp = evo_dir();
        std::fs::write(tmp.path().join(".pmat-metrics").join("lint.json"), "{}").unwrap();
        std::fs::write(
            tmp.path()
                .join(".pmat-metrics")
                .join("commit-0000-other.json"),
            "{}",
        )
        .unwrap();
        for (i, p) in [10u64, 20, 30].iter().enumerate() {
            write_tests_record(&tmp, i + 1, *p, 100);
        }
        let r = check_swe_ci_evoscore(tmp.path());
        assert_eq!(r.status, CheckStatus::Pass);
        assert!(r.message.contains("3 commits"), "{}", r.message);
    }

    #[test]
    fn test_evoscore_series_is_ordered_by_filename_not_readdir() {
        // Written in reverse; the score must match the ascending-name order.
        let tmp = evo_dir();
        for (i, p) in [80u64, 85, 90, 95].iter().enumerate().rev() {
            write_tests_record(&tmp, i, *p, 100);
        }
        let r = check_swe_ci_evoscore(tmp.path());
        assert!(r.message.contains("EvoScore: 0.754"), "{}", r.message);
    }

    #[test]
    fn test_evoscore_no_metrics_dir_reports_zero_commits() {
        let tmp = tempfile::tempdir().expect("create tempdir");
        let r = check_swe_ci_evoscore(tmp.path());
        assert_eq!(r.status, CheckStatus::Skip);
        assert!(
            r.message.contains("(0 commits, need >= 3)"),
            "{}",
            r.message
        );
    }
}

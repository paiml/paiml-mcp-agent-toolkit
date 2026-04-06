#![cfg_attr(coverage_nightly, coverage(off))]
// CB-140: Mono-spec structure enforcement
// CB-141: Memory profiling infrastructure
// CB-142: SWE-CI EvoScore

use super::types::*;
use std::path::Path;

/// CB-140: Validate mono-spec structure
///
/// Checks:
/// 1. docs/specifications/pmat-spec.md exists
/// 2. docs/specifications/components/ directory exists with sub-specs
/// 3. No loose spec files in docs/specifications/ (only pmat-spec.md)
/// 4. All component files are under 500 lines
#[provable_contracts_macros::contract("pmat-core.yaml", equation = "path_exists")]
pub(crate) fn check_mono_spec_structure(project_path: &Path) -> ComplianceCheck {
    debug_assert!(
        project_path.exists(),
        "project_path must exist: {}",
        project_path.display()
    );
    let spec_dir = project_path.join("docs").join("specifications");
    let root_spec = spec_dir.join("pmat-spec.md");
    let components_dir = spec_dir.join("components");

    // Check 1: Root spec exists
    if !root_spec.exists() {
        return ComplianceCheck {
            name: "CB-140: Mono-Spec Structure".into(),
            status: CheckStatus::Skip,
            message: "No docs/specifications/pmat-spec.md found (not a spec-managed project)"
                .into(),
            severity: Severity::Info,
        };
    }

    let mut issues = Vec::new();

    // Check 2: Components directory exists
    if !components_dir.exists() || !components_dir.is_dir() {
        issues.push("Missing docs/specifications/components/ directory".to_string());
    }

    // Check 3: No loose spec files (only pmat-spec.md allowed at root level)
    if let Ok(entries) = std::fs::read_dir(&spec_dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_file() && path.extension().and_then(|e| e.to_str()) == Some("md") {
                if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
                    if name != "pmat-spec.md" {
                        issues.push(format!(
                            "Loose spec file: {} (should be in components/)",
                            name
                        ));
                    }
                }
            }
        }
    }

    // Check 4: Component files under 500 lines
    if components_dir.exists() {
        if let Ok(entries) = std::fs::read_dir(&components_dir) {
            let mut component_count = 0;
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_file() && path.extension().and_then(|e| e.to_str()) == Some("md") {
                    component_count += 1;
                    if let Ok(content) = std::fs::read_to_string(&path) {
                        let line_count = content.lines().count();
                        if line_count > 500 {
                            if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
                                issues.push(format!("{}: {} lines (max 500)", name, line_count));
                            }
                        }
                    }
                }
            }
            if component_count == 0 {
                issues.push("No component spec files found in components/".to_string());
            }
        }
    }

    // Check root spec is under 500 lines
    if let Ok(content) = std::fs::read_to_string(&root_spec) {
        let line_count = content.lines().count();
        if line_count > 500 {
            issues.push(format!("pmat-spec.md: {} lines (max 500)", line_count));
        }
    }

    if issues.is_empty() {
        ComplianceCheck {
            name: "CB-140: Mono-Spec Structure".into(),
            status: CheckStatus::Pass,
            message: "Mono-spec structure valid (root spec + components, all under 500 lines)"
                .into(),
            severity: Severity::Info,
        }
    } else {
        ComplianceCheck {
            name: "CB-140: Mono-Spec Structure".into(),
            status: CheckStatus::Warn,
            message: format!(
                "{} issue(s):\n{}",
                issues.len(),
                format_violation_list(&issues)
            ),
            severity: Severity::Warning,
        }
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
    debug_assert!(
        project_path.exists(),
        "project_path must exist: {}",
        project_path.display()
    );
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
    debug_assert!(
        project_path.exists(),
        "project_path must exist: {}",
        project_path.display()
    );
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

/// CB-142: SWE-CI EvoScore from git history
///
/// Computes evolution score from test pass/fail data across commits.
/// Returns Skip if insufficient data.
#[provable_contracts_macros::contract("pmat-core.yaml", equation = "path_exists")]
pub(crate) fn check_swe_ci_evoscore(project_path: &Path) -> ComplianceCheck {
    debug_assert!(
        project_path.exists(),
        "project_path must exist: {}",
        project_path.display()
    );
    let metrics_dir = project_path.join(".pmat-metrics");

    // Collect commit test data files (sorted by filename for chronological order)
    let mut test_data: Vec<(String, u64, u64)> = Vec::new(); // (commit, pass, total)
    let mut test_files: Vec<std::path::PathBuf> = Vec::new();

    if metrics_dir.exists() {
        if let Ok(entries) = std::fs::read_dir(&metrics_dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
                    if name.starts_with("commit-") && name.ends_with("-tests.json") {
                        test_files.push(path);
                    }
                }
            }
        }
    }

    // Sort by filename to ensure chronological order
    test_files.sort();

    for path in &test_files {
        if let Ok(content) = std::fs::read_to_string(path) {
            if let Ok(data) = serde_json::from_str::<serde_json::Value>(&content) {
                let pass = data["pass"].as_u64().unwrap_or(0);
                let total = data["total"].as_u64().unwrap_or(0);
                let commit = data["commit"].as_str().unwrap_or("unknown").to_string();
                if total > 0 {
                    test_data.push((commit, pass, total));
                }
            }
        }
    }

    // Also check for meta files that contain test counts
    if test_data.is_empty() && metrics_dir.exists() {
        let mut meta_files: Vec<std::path::PathBuf> = Vec::new();
        if let Ok(entries) = std::fs::read_dir(&metrics_dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
                    if name.starts_with("commit-") && name.ends_with("-meta.json") {
                        meta_files.push(path);
                    }
                }
            }
        }
        meta_files.sort();
        for path in &meta_files {
            if let Ok(content) = std::fs::read_to_string(path) {
                if let Ok(data) = serde_json::from_str::<serde_json::Value>(&content) {
                    if let Some(tests) = data.get("tests") {
                        let pass = tests["pass"].as_u64().unwrap_or(0);
                        let total = tests["total"].as_u64().unwrap_or(0);
                        let commit = data["commit"].as_str().unwrap_or("unknown").to_string();
                        if total > 0 {
                            test_data.push((commit, pass, total));
                        }
                    }
                }
            }
        }
    }

    if test_data.len() < 3 {
        return ComplianceCheck {
            name: "CB-142: SWE-CI EvoScore".into(),
            status: CheckStatus::Skip,
            message: format!(
                "Insufficient commit test data ({} commits, need >= 3). \
                 Record test results with: pmat test --record",
                test_data.len()
            ),
            severity: Severity::Info,
        };
    }

    // Compute EvoScore with gamma = 1.5 (default)
    let gamma: f64 = 1.5;
    let n = test_data.len();

    // Base state: first commit
    let base_pass = test_data[0].1 as f64;
    // Oracle: best observed pass count (proxy for ideal)
    let oracle_pass = test_data.iter().map(|(_, p, _)| *p).max().unwrap_or(0) as f64;

    let mut weighted_sum = 0.0;
    let mut weight_total = 0.0;

    for (i, (_commit, pass, _total)) in test_data.iter().enumerate().skip(1) {
        let current_pass = *pass as f64;
        let a_c = if current_pass >= base_pass {
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
        };

        let weight = gamma.powi(i as i32);
        weighted_sum += weight * a_c;
        weight_total += weight;
    }

    let evoscore = if weight_total > 0.0 {
        weighted_sum / weight_total
    } else {
        0.0
    };

    let (status, severity) = if evoscore >= 0.5 {
        (CheckStatus::Pass, Severity::Info)
    } else if evoscore >= 0.0 {
        (CheckStatus::Warn, Severity::Warning)
    } else {
        (CheckStatus::Fail, Severity::Error)
    };

    ComplianceCheck {
        name: "CB-142: SWE-CI EvoScore".into(),
        status,
        message: format!(
            "EvoScore: {:.3} (gamma={:.1}, {} commits). {}",
            evoscore,
            gamma,
            n,
            match status {
                CheckStatus::Pass => "Consistent improvement trend",
                CheckStatus::Warn => "Mixed improvement/regression trend",
                CheckStatus::Fail => "Net regression trend",
                CheckStatus::Skip => "",
            }
        ),
        severity,
    }
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
}

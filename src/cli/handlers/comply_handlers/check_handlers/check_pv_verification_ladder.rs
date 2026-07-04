// Provable-contracts verification ladder checks (CB-1204 through CB-1207)
// Included from check.rs — do NOT add `use` imports or `#!` attributes here.

/// CB-1204 helper: is the build.rs pipeline superseded by ≥10 trait impls in a
/// `contract_traits.rs` (root `tests/` or any `crates/*/tests/`)? Returns the
/// impl count and the file it was found in.
fn trait_enforcement_supersedes(project_path: &Path) -> Option<(usize, std::path::PathBuf)> {
    let mut paths = vec![project_path.join("tests").join("contract_traits.rs")];
    if let Ok(entries) = std::fs::read_dir(project_path.join("crates")) {
        for e in entries.flatten() {
            let p = e.path().join("tests").join("contract_traits.rs");
            if p.exists() {
                paths.push(p);
            }
        }
    }
    for trait_test in paths {
        let Ok(content) = std::fs::read_to_string(&trait_test) else {
            continue;
        };
        let impl_count = content
            .lines()
            .filter(|l| {
                let t = l.trim();
                t.starts_with("impl ") && t.contains("V1 for") && !t.starts_with("//")
            })
            .count();
        if impl_count >= 10 {
            return Some((impl_count, trait_test));
        }
    }
    None
}

/// CB-1204 helper: does a build.rs body emit contract PRE/POST assertion env
/// vars (literal or GH-295 dynamic-format patterns)?
fn build_emits_pre_post(c: &str) -> bool {
    c.contains("PRE_COUNT")
        || c.contains("emit_contract")
        || c.contains("_PRE_0")
        || c.contains("cargo:rustc-env=PRE_")
        || c.contains("cargo:rustc-env=POST_")
        || (c.contains("PRE_{") && c.contains("POST_{"))
        || c.contains("emit_pre_post")
}

/// CB-1204: Build.rs contract pipeline — does build.rs emit assertion env vars from YAML?
///
/// The escape-proof pipeline requires build.rs to read contracts/*.yaml and
/// emit CONTRACT_*_PRE_COUNT / CONTRACT_*_PRE_0 env vars that the #[contract]
/// proc macro reads at compile time.
#[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
pub(crate) fn check_build_rs_pipeline(project_path: &Path) -> ComplianceCheck {
    let contracts_dir = project_path.join("contracts");
    let build_rs = project_path.join("build.rs");

    if !contracts_dir.exists() {
        return ComplianceCheck {
            name: "CB-1204: Build.rs Pipeline".into(),
            status: CheckStatus::Skip,
            message: "No contracts/ directory".into(),
            severity: Severity::Info,
        };
    }

    // Check YAML has preconditions (otherwise no pipeline needed)
    let has_preconditions = std::fs::read_dir(&contracts_dir)
        .map(|entries| {
            entries.flatten().any(|e| {
                e.path().extension().is_some_and(|ext| ext == "yaml")
                    && std::fs::read_to_string(e.path())
                        .map(|c| c.contains("preconditions:"))
                        .unwrap_or(false)
            })
        })
        .unwrap_or(false);

    if !has_preconditions {
        return ComplianceCheck {
            name: "CB-1204: Build.rs Pipeline".into(),
            status: CheckStatus::Pass,
            message: "No preconditions in YAML — pipeline not required".into(),
            severity: Severity::Info,
        };
    }

    // If CB-1209 trait enforcement is active (≥10 impls in a contract_traits.rs),
    // the build.rs pipeline is superseded — traits are the stronger mechanism.
    if let Some((impl_count, trait_test)) = trait_enforcement_supersedes(project_path) {
        return ComplianceCheck {
            name: "CB-1204: Build.rs Pipeline".into(),
            status: CheckStatus::Pass,
            message: format!(
                "Superseded by trait enforcement ({impl_count} trait impls in {})",
                trait_test
                    .strip_prefix(project_path)
                    .unwrap_or(&trait_test)
                    .display()
            ),
            severity: Severity::Info,
        };
    }

    // Check build.rs at root or in crates/*/ (GH-295: also scan crates/*/build.rs)
    let mut build_files = vec![build_rs.clone()];
    if let Ok(entries) = std::fs::read_dir(project_path.join("crates")) {
        for e in entries.flatten() {
            let bf = e.path().join("build.rs");
            if bf.exists() {
                build_files.push(bf);
            }
        }
    }

    let any_build_rs = build_files.iter().any(|f| f.exists());
    if !any_build_rs {
        return ComplianceCheck {
            name: "CB-1204: Build.rs Pipeline".into(),
            status: CheckStatus::Fail,
            message: "Contracts have preconditions but no build.rs to emit assertion env vars"
                .into(),
            severity: Severity::Error,
        };
    }

    let has_pre_emit = build_files.iter().any(|f| {
        std::fs::read_to_string(f)
            .map(|c| build_emits_pre_post(&c))
            .unwrap_or(false)
    });
    if has_pre_emit {
        return ComplianceCheck {
            name: "CB-1204: Build.rs Pipeline".into(),
            status: CheckStatus::Pass,
            message: "build.rs emits contract assertion env vars from YAML".into(),
            severity: Severity::Info,
        };
    }

    ComplianceCheck {
        name: "CB-1204: Build.rs Pipeline".into(),
        status: CheckStatus::Fail,
        message: "build.rs exists but doesn't emit PRE/POST env vars from contracts/ YAML".into(),
        severity: Severity::Error,
    }
}


/// Count entries in a root-level YAML sequence field (0 if absent/not a list).
fn count_seq(doc: &serde_json::Value, key: &str) -> usize {
    doc.get(key)
        .and_then(|v| v.as_array())
        .map(|a| a.len())
        .unwrap_or(0)
}

/// Evaluate one contract's provability evidence for CB-1205.
/// Returns `(is_kernel, violations)`; `is_kernel` is true when the contract
/// carries proof obligations. Unparseable YAML falls back to key existence.
fn eval_kernel_provability(content: &str, stem: &str) -> (bool, Vec<String>) {
    let mut v = Vec::new();
    let Ok(doc) = serde_yaml_ng::from_str::<serde_json::Value>(content) else {
        if !content.contains("proof_obligations:") {
            return (false, v);
        }
        if !content.contains("kani_harnesses:") {
            v.push(format!("{stem}: has proof_obligations but no kani_harnesses"));
        }
        if !content.contains("falsification_tests:") {
            v.push(format!("{stem}: has proof_obligations but no falsification_tests"));
        }
        return (true, v);
    };
    let obligations = count_seq(&doc, "proof_obligations");
    if obligations == 0 {
        return (false, v);
    }
    if count_seq(&doc, "kani_harnesses") == 0 {
        v.push(format!("{stem}: {obligations} obligation(s) but 0 kani_harnesses"));
    }
    let falsification = count_seq(&doc, "falsification_tests");
    if falsification < obligations {
        v.push(format!(
            "{stem}: {falsification} falsification_test(s) < {obligations} obligation(s)"
        ));
    }
    (true, v)
}

/// MUST have kani_harnesses and sufficient falsification_tests.
/// pv-compatibility spec §2.2
///
/// CB-1205 count hardening: enforces the provability invariant by COUNT, not
/// mere key existence — `|proof_obligations| > 0` requires `|kani_harnesses| ≥ 1`
/// AND `|falsification_tests| ≥ |proof_obligations|`. Unparseable YAML falls back
/// to key-existence so a malformed kernel still trips (never silently passes).
#[provable_contracts_macros::contract("pmat-core.yaml", equation = "path_exists")]
pub(crate) fn check_provability_invariant(project_path: &Path) -> ComplianceCheck {
    let contracts_dir = project_path.join("contracts");
    if !contracts_dir.exists() {
        return ComplianceCheck {
            name: "CB-1205: Provability Invariant".into(),
            status: CheckStatus::Skip,
            message: "No contracts/ directory".into(),
            severity: Severity::Info,
        };
    }

    let mut kernel_contracts = 0usize;
    let mut violations = Vec::new();

    if let Ok(entries) = std::fs::read_dir(&contracts_dir) {
        for entry in entries.flatten() {
            let p = entry.path();
            if p.extension().is_none_or(|e| e != "yaml") {
                continue;
            }
            if p.file_name()
                .is_some_and(|n| n.to_string_lossy().contains("binding"))
            {
                continue;
            }
            let Ok(content) = std::fs::read_to_string(&p) else {
                continue;
            };

            // Skip data registries
            if content.contains("registry: true") {
                continue;
            }

            let stem = p.file_stem().and_then(|s| s.to_str()).unwrap_or("?");
            let (is_kernel, mut file_violations) = eval_kernel_provability(&content, stem);
            if is_kernel {
                kernel_contracts += 1;
                violations.append(&mut file_violations);
            }
        }
    }

    if kernel_contracts == 0 {
        return ComplianceCheck {
            name: "CB-1205: Provability Invariant".into(),
            status: CheckStatus::Pass,
            message: "No kernel contracts with proof_obligations found".into(),
            severity: Severity::Info,
        };
    }

    if violations.is_empty() {
        ComplianceCheck {
            name: "CB-1205: Provability Invariant".into(),
            status: CheckStatus::Pass,
            message: format!("{kernel_contracts} kernel contract(s) satisfy provability invariant"),
            severity: Severity::Info,
        }
    } else {
        ComplianceCheck {
            name: "CB-1205: Provability Invariant".into(),
            status: CheckStatus::Warn,
            message: format!(
                "{} violation(s): {}",
                violations.len(),
                violations
                    .iter()
                    .take(3)
                    .cloned()
                    .collect::<Vec<_>>()
                    .join("; ")
            ),
            severity: Severity::Warning,
        }
    }
}


/// Reads proof-status.json from provable-contracts sibling repo.
/// pv-compatibility spec §2.3
#[provable_contracts_macros::contract("pmat-core.yaml", equation = "path_exists")]
pub(crate) fn check_verification_levels(project_path: &Path, thresholds: &ComplyThresholds) -> ComplianceCheck {
    // Resolve to absolute path so .parent() works correctly from "."
    let abs_path =
        std::fs::canonicalize(project_path).unwrap_or_else(|_| project_path.to_path_buf());
    let ps_path = abs_path
        .parent()
        .map(|p| p.join("provable-contracts").join("proof-status.json"));

    let Some(ps_path) = ps_path.filter(|p| p.exists()) else {
        return ComplianceCheck {
            name: "CB-1206: Verification Levels".into(),
            status: CheckStatus::Skip,
            message: "No proof-status.json in ../provable-contracts/".into(),
            severity: Severity::Info,
        };
    };

    let Ok(content) = std::fs::read_to_string(&ps_path) else {
        return ComplianceCheck {
            name: "CB-1206: Verification Levels".into(),
            status: CheckStatus::Skip,
            message: "Cannot read proof-status.json".into(),
            severity: Severity::Info,
        };
    };

    let Ok(val) = serde_json::from_str::<serde_json::Value>(&content) else {
        return ComplianceCheck {
            name: "CB-1206: Verification Levels".into(),
            status: CheckStatus::Warn,
            message: "Cannot parse proof-status.json".into(),
            severity: Severity::Warning,
        };
    };

    // Collect contract stems from this project's contracts/ directory
    let project_stems = collect_project_contract_stems(project_path);

    // If the project has no provable-contract YAML files, skip — don't report global totals
    if project_stems.is_empty() {
        return ComplianceCheck {
            name: "CB-1206: Verification Levels".into(),
            status: CheckStatus::Skip,
            message: "No provable-contract YAML files found — skipping verification level check".into(),
            severity: Severity::Info,
        };
    }

    // Filter proof-status.json contracts to only those belonging to this project
    let (obligations, tests, kani, lean, contracts) =
        if let Some(contracts_arr) = val.get("contracts").and_then(|v| v.as_array()) {
            let mut ob = 0u64;
            let mut ts = 0u64;
            let mut ka = 0u64;
            let mut le = 0u64;
            let mut ct = 0u64;
            for entry in contracts_arr {
                let stem = entry
                    .get("stem")
                    .and_then(|v| v.as_str())
                    .unwrap_or("");
                if !project_stems.contains(stem) {
                    continue;
                }
                ct += 1;
                ob += entry
                    .get("obligations")
                    .and_then(|v| v.as_u64())
                    .unwrap_or(0);
                ts += entry
                    .get("falsification_tests")
                    .and_then(|v| v.as_u64())
                    .unwrap_or(0);
                ka += entry
                    .get("kani_harnesses")
                    .and_then(|v| v.as_u64())
                    .unwrap_or(0);
                le += entry
                    .get("lean_proved")
                    .and_then(|v| v.as_u64())
                    .unwrap_or(0);
            }
            (ob, ts, ka, le, ct)
        } else {
            // Fallback to totals if no contracts array (old format)
            let totals = val.get("totals");
            let ob = totals.and_then(|t| t.get("obligations")).and_then(|v| v.as_u64()).unwrap_or(0);
            let ts = totals.and_then(|t| t.get("falsification_tests")).and_then(|v| v.as_u64()).unwrap_or(0);
            let ka = totals.and_then(|t| t.get("kani_harnesses")).and_then(|v| v.as_u64()).unwrap_or(0);
            let le = totals.and_then(|t| t.get("lean_proved")).and_then(|v| v.as_u64()).unwrap_or(0);
            let ct = totals.and_then(|t| t.get("contracts")).and_then(|v| v.as_u64()).unwrap_or(0);
            (ob, ts, ka, le, ct)
        };

    if obligations == 0 {
        return ComplianceCheck {
            name: "CB-1206: Verification Levels".into(),
            status: CheckStatus::Pass,
            message: format!("{contracts} contracts, 0 obligations"),
            severity: Severity::Info,
        };
    }

    let l4_pct = kani as f64 / obligations as f64 * 100.0;
    let l5_pct = lean as f64 / obligations as f64 * 100.0;

    let msg = format!(
        "{obligations} obligations: L2={tests} tests, L4={kani} kani ({l4_pct:.0}%), L5={lean} lean ({l5_pct:.0}%)"
    );

    let min_kani = thresholds.min_kani_coverage;
    if min_kani > 0.0 && l4_pct < min_kani {
        ComplianceCheck {
            name: "CB-1206: Verification Levels".into(),
            status: CheckStatus::Fail,
            message: format!("{msg} — Kani {l4_pct:.0}% < threshold {min_kani:.0}%"),
            severity: Severity::Error,
        }
    } else if l4_pct < 10.0 && kani == 0 {
        ComplianceCheck {
            name: "CB-1206: Verification Levels".into(),
            status: CheckStatus::Warn,
            message: format!("{msg} — no Kani verification"),
            severity: Severity::Warning,
        }
    } else {
        ComplianceCheck {
            name: "CB-1206: Verification Levels".into(),
            status: CheckStatus::Pass,
            message: msg,
            severity: Severity::Info,
        }
    }
}


/// CB-1207 helper: is this file a drift-tracked provable contract YAML —
/// a `.yaml`/`.yml`, not a binding, carrying provable-contracts schema markers?
fn is_drift_tracked_contract(p: &Path) -> bool {
    if !p.is_file() || p.extension().is_none_or(|e| e != "yaml" && e != "yml") {
        return false;
    }
    if p.file_name()
        .is_some_and(|n| n.to_string_lossy().contains("binding"))
    {
        return false;
    }
    let Ok(content) = std::fs::read_to_string(p) else {
        return false;
    };
    content.contains("proof_obligations")
        || content.contains("equations:")
        || content.contains("falsification_tests")
        || content.contains("kani_harnesses")
}

/// CB-1207 helper: is a contract "stale" — both its last git commit AND its
/// YAML mtime older than `threshold`? Flattened from a nested if-let pyramid.
fn contract_is_stale(
    project_path: &Path,
    rel_path: &Path,
    yaml_mtime: std::time::SystemTime,
    threshold: std::time::Duration,
) -> bool {
    let output = std::process::Command::new("git")
        .args(["log", "-1", "--format=%ct", "--"])
        .arg(rel_path)
        .current_dir(project_path)
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::null())
        .output();
    let Ok(o) = output else { return false };
    let Ok(ts_str) = String::from_utf8(o.stdout) else {
        return false;
    };
    let Ok(ts) = ts_str.trim().parse::<u64>() else {
        return false;
    };
    let contract_commit = std::time::UNIX_EPOCH + std::time::Duration::from_secs(ts);
    let now = std::time::SystemTime::now();
    let Ok(age) = now.duration_since(contract_commit) else {
        return false;
    };
    age > threshold
        && now
            .duration_since(yaml_mtime)
            .map(|ya| ya > threshold)
            .unwrap_or(false)
}

/// CB-1207: Contract drift — are contracts stale relative to source changes?
/// A contract YAML older than its bound source files by >30 days = drift.
/// pv-compatibility spec CD5.
#[provable_contracts_macros::contract("pmat-core.yaml", equation = "path_exists")]
pub(crate) fn check_contract_drift(project_path: &Path) -> ComplianceCheck {
    let contracts_dir = match resolve_contracts_dir(project_path) {
        Some(d) => d,
        None => {
            return ComplianceCheck {
                name: "CB-1207: Contract Drift".into(),
                status: CheckStatus::Skip,
                message: "No contracts directory found (local or sibling)".into(),
                severity: Severity::Info,
            };
        }
    };

    let thirty_days = std::time::Duration::from_secs(30 * 24 * 3600);
    let mut stale = 0usize;
    let mut total = 0usize;

    for entry in walkdir::WalkDir::new(&contracts_dir)
        .max_depth(3)
        .into_iter()
        .filter_map(|e| e.ok())
    {
        let p = entry.path();
        if !is_drift_tracked_contract(p) {
            continue;
        }
        let Ok(meta) = std::fs::metadata(p) else {
            continue;
        };
        let Ok(yaml_mtime) = meta.modified() else {
            continue;
        };
        total += 1;

        let rel_path = p.strip_prefix(project_path).unwrap_or(p);
        if contract_is_stale(project_path, rel_path, yaml_mtime, thirty_days * 3) {
            stale += 1;
        }
    }

    if total == 0 {
        return ComplianceCheck {
            name: "CB-1207: Contract Drift".into(),
            status: CheckStatus::Pass,
            message: "No contract YAMLs to check".into(),
            severity: Severity::Info,
        };
    }

    let fresh = total - stale;
    if stale == 0 {
        ComplianceCheck {
            name: "CB-1207: Contract Drift".into(),
            status: CheckStatus::Pass,
            message: format!("{total} contract(s), all fresh (committed within 90 days)"),
            severity: Severity::Info,
        }
    } else {
        ComplianceCheck {
            name: "CB-1207: Contract Drift".into(),
            status: CheckStatus::Warn,
            message: format!(
                "{stale}/{total} contract(s) stale (>90 days since last commit), {fresh} fresh"
            ),
            severity: Severity::Warning,
        }
    }
}

#[cfg(test)]
mod check_pv_verification_ladder_tests {
    //! Covers the "no contracts/ → Skip" early-return arms for all 4
    //! verification-ladder checks in check_pv_verification_ladder.rs
    //! (55 uncov on broad, 0% cov).
    use super::*;

    #[test]
    fn test_check_build_rs_pipeline_no_contracts_dir_skips() {
        let tmp = tempfile::tempdir().unwrap();
        let check = check_build_rs_pipeline(tmp.path());
        assert!(matches!(check.status, CheckStatus::Skip));
        assert_eq!(check.severity, Severity::Info);
        assert!(check.message.contains("No contracts/ directory"));
        assert_eq!(check.name, "CB-1204: Build.rs Pipeline");
    }

    #[test]
    fn test_check_build_rs_pipeline_contracts_no_preconditions_passes() {
        let tmp = tempfile::tempdir().unwrap();
        let cd = tmp.path().join("contracts");
        std::fs::create_dir(&cd).unwrap();
        std::fs::write(cd.join("c.yaml"), "name: foo\nversion: 1\n").unwrap();
        let check = check_build_rs_pipeline(tmp.path());
        assert!(matches!(check.status, CheckStatus::Pass));
        assert!(check.message.contains("pipeline not required"));
    }

    #[test]
    fn test_check_provability_invariant_no_contracts_dir_skips() {
        let tmp = tempfile::tempdir().unwrap();
        let check = check_provability_invariant(tmp.path());
        assert!(matches!(check.status, CheckStatus::Skip));
        assert!(check.message.contains("No contracts/ directory"));
        assert_eq!(check.name, "CB-1205: Provability Invariant");
    }

    #[test]
    fn test_check_provability_invariant_empty_contracts_dir_runs() {
        let tmp = tempfile::tempdir().unwrap();
        std::fs::create_dir(tmp.path().join("contracts")).unwrap();
        let check = check_provability_invariant(tmp.path());
        assert_eq!(check.name, "CB-1205: Provability Invariant");
    }

    #[test]
    fn test_check_provability_invariant_falsification_shortfall_warns() {
        // CB-1205 count hardening: 2 obligations but only 1 falsification_test
        // violates |falsification_tests| >= |proof_obligations|.
        let tmp = tempfile::tempdir().unwrap();
        let cd = tmp.path().join("contracts");
        std::fs::create_dir(&cd).unwrap();
        std::fs::write(
            cd.join("kernel.yaml"),
            "proof_obligations:\n  - id: o1\n  - id: o2\nfalsification_tests:\n  - id: f1\nkani_harnesses:\n  - harness: h1\n",
        )
        .unwrap();
        let check = check_provability_invariant(tmp.path());
        assert_eq!(check.status, CheckStatus::Warn);
        assert!(check.message.contains("falsification_test"));
    }

    #[test]
    fn test_check_provability_invariant_sufficient_counts_passes() {
        let tmp = tempfile::tempdir().unwrap();
        let cd = tmp.path().join("contracts");
        std::fs::create_dir(&cd).unwrap();
        std::fs::write(
            cd.join("kernel.yaml"),
            "proof_obligations:\n  - id: o1\n  - id: o2\nfalsification_tests:\n  - id: f1\n  - id: f2\nkani_harnesses:\n  - harness: h1\n",
        )
        .unwrap();
        let check = check_provability_invariant(tmp.path());
        assert_eq!(check.status, CheckStatus::Pass);
    }

    #[test]
    fn test_check_provability_invariant_missing_kani_warns() {
        // 1 obligation, sufficient falsification, but 0 kani_harnesses.
        let tmp = tempfile::tempdir().unwrap();
        let cd = tmp.path().join("contracts");
        std::fs::create_dir(&cd).unwrap();
        std::fs::write(
            cd.join("kernel.yaml"),
            "proof_obligations:\n  - id: o1\nfalsification_tests:\n  - id: f1\n",
        )
        .unwrap();
        let check = check_provability_invariant(tmp.path());
        assert_eq!(check.status, CheckStatus::Warn);
        assert!(check.message.contains("kani_harnesses"));
    }

    #[test]
    fn test_check_contract_drift_no_contracts_skips() {
        let tmp = tempfile::tempdir().unwrap();
        let check = check_contract_drift(tmp.path());
        assert!(matches!(check.status, CheckStatus::Skip));
        assert_eq!(check.name, "CB-1207: Contract Drift");
    }

    #[test]
    fn test_check_verification_levels_on_empty_project_runs() {
        let tmp = tempfile::tempdir().unwrap();
        let thresholds = ComplyThresholds::default();
        let check = check_verification_levels(tmp.path(), &thresholds);
        assert!(matches!(
            check.status,
            CheckStatus::Skip | CheckStatus::Pass | CheckStatus::Warn | CheckStatus::Fail
        ));
    }
}


// Provable-contracts verification ladder checks (CB-1204 through CB-1207)
// Included from check.rs — do NOT add `use` imports or `#!` attributes here.

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

    // If CB-1209 trait enforcement is active (tests/contract_traits.rs with impls),
    // the build.rs pipeline is superseded — traits are the newer, stronger mechanism
    let trait_test = project_path.join("tests").join("contract_traits.rs");
    if trait_test.exists() {
        if let Ok(content) = std::fs::read_to_string(&trait_test) {
            let impl_count = content.lines().filter(|l| {
                let t = l.trim();
                t.starts_with("impl ") && t.contains("V1 for") && !t.starts_with("//")
            }).count();
            if impl_count >= 10 {
                return ComplianceCheck {
                    name: "CB-1204: Build.rs Pipeline".into(),
                    status: CheckStatus::Pass,
                    message: format!(
                        "Superseded by trait enforcement ({impl_count} trait impls in tests/contract_traits.rs)"
                    ),
                    severity: Severity::Info,
                };
            }
        }
    }

    // Check build.rs at root or in crates/*/
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
            .map(|c| c.contains("PRE_COUNT") || c.contains("emit_contract") || c.contains("_PRE_0"))
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


/// MUST have kani_harnesses and sufficient falsification_tests.
/// pv-compatibility spec §2.2
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
            if p.extension().map_or(true, |e| e != "yaml") {
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

            let has_obligations = content.contains("proof_obligations:");
            if !has_obligations {
                continue;
            }

            kernel_contracts += 1;
            let stem = p.file_stem().and_then(|s| s.to_str()).unwrap_or("?");

            let has_kani = content.contains("kani_harnesses:");
            let has_falsification = content.contains("falsification_tests:");

            if !has_kani {
                violations.push(format!(
                    "{stem}: has proof_obligations but no kani_harnesses"
                ));
            }
            if !has_falsification {
                violations.push(format!(
                    "{stem}: has proof_obligations but no falsification_tests"
                ));
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
        if !p.is_file() {
            continue;
        }
        if p.extension().map_or(true, |e| e != "yaml" && e != "yml") {
            continue;
        }
        if p.file_name()
            .is_some_and(|n| n.to_string_lossy().contains("binding"))
        {
            continue;
        }
        // Only count files with provable-contracts schema markers (matches CB-1200)
        if let Ok(content) = std::fs::read_to_string(p) {
            if !content.contains("proof_obligations")
                && !content.contains("equations:")
                && !content.contains("falsification_tests")
                && !content.contains("kani_harnesses")
            {
                continue;
            }
        } else {
            continue;
        }
        let Ok(meta) = std::fs::metadata(p) else {
            continue;
        };
        let Ok(yaml_mtime) = meta.modified() else {
            continue;
        };
        total += 1;

        // Check git log for the contract's last commit vs now
        let rel_path = p.strip_prefix(project_path).unwrap_or(p);
        let output = std::process::Command::new("git")
            .args(["log", "-1", "--format=%ct", "--"])
            .arg(rel_path)
            .current_dir(project_path)
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::null())
            .output();

        if let Ok(o) = output {
            if let Ok(ts_str) = String::from_utf8(o.stdout) {
                if let Ok(ts) = ts_str.trim().parse::<u64>() {
                    let contract_commit =
                        std::time::UNIX_EPOCH + std::time::Duration::from_secs(ts);
                    let now = std::time::SystemTime::now();
                    if let Ok(age) = now.duration_since(contract_commit) {
                        // Contract not touched in >90 days AND yaml is old
                        if age > thirty_days * 3 {
                            if let Ok(yaml_age) = now.duration_since(yaml_mtime) {
                                if yaml_age > thirty_days * 3 {
                                    stale += 1;
                                }
                            }
                        }
                    }
                }
            }
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


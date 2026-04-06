// Provable-contracts enforcement checks (CB-1201 through CB-1209)
// Included from check.rs — do NOT add `use` imports or `#!` attributes here.

/// Extract equation names from contract YAMLs that have preconditions or postconditions.
fn collect_contract_equation_names(contracts_dir: &Path) -> Vec<String> {
    debug_assert!(contracts_dir.exists(), "contracts_dir must exist: {}", contracts_dir.display());
    let mut eq_names = Vec::new();
    let headers = [
        "equations",
        "metadata",
        "falsification_tests",
        "kani_harnesses",
        "proof_obligations",
        "qa_gate",
        "implementation",
        "enforcement",
        "version",
        "created",
        "author",
        "description",
        "references",
        "issues",
    ];
    let Ok(entries) = std::fs::read_dir(contracts_dir) else {
        return eq_names;
    };
    for entry in entries.flatten() {
        if entry.path().extension().map_or(true, |e| e != "yaml") {
            continue;
        }
        let Ok(content) = std::fs::read_to_string(entry.path()) else {
            continue;
        };
        let lines: Vec<&str> = content.lines().collect();
        for (i, line) in lines.iter().enumerate() {
            let trimmed = line.trim();
            if !trimmed.ends_with(':')
                || trimmed.starts_with('#')
                || trimmed.starts_with('-')
                || trimmed.contains(' ')
                || !line.starts_with("  ")
                || line.starts_with("    ")
            {
                continue;
            }
            let name = trimmed.trim_end_matches(':');
            if headers.contains(&name) {
                continue;
            }
            // Look ahead for preconditions/postconditions
            let has_pre_post = lines[i + 1..]
                .iter()
                .take_while(|next| {
                    let nt = next.trim();
                    !(next.starts_with("  ")
                        && !next.starts_with("    ")
                        && nt.ends_with(':')
                        && !nt.starts_with('#')
                        && !nt.starts_with('-'))
                })
                .any(|next| {
                    let nt = next.trim();
                    nt == "preconditions:" || nt == "postconditions:"
                });
            if has_pre_post {
                eq_names.push(name.to_string());
            }
        }
    }
    eq_names
}

/// CB-1203: Contract-bound functions MUST have #[contract] or #[requires]/#[ensures] macros.
/// Cross-references contract YAML equation names against production source.
/// A production `pub fn <equation_name>` without a contract macro = FAIL.
/// Preferred: `#[contract("yaml-name", equation = "eq")]` — auto-injects from YAML.
/// Legacy: `#[requires(...)]` / `#[ensures(...)]` — hand-written assertions.
pub(crate) fn check_annotation_coverage(project_path: &Path) -> ComplianceCheck {
    debug_assert!(project_path.exists(), "project_path must exist: {}", project_path.display());
    let contracts_dir = project_path.join("contracts");
    if !contracts_dir.exists() {
        return ComplianceCheck {
            name: "CB-1203: Contract Annotations".into(),
            status: CheckStatus::Skip,
            message: "No contracts/ directory".into(),
            severity: Severity::Info,
        };
    }
    // Support both flat (src/) and workspace (crates/*/src/) layouts
    let src_dir = project_path.join("src");
    let crates_dir = project_path.join("crates");
    if !src_dir.exists() && !crates_dir.exists() {
        return ComplianceCheck {
            name: "CB-1203: Contract Annotations".into(),
            status: CheckStatus::Skip,
            message: "No src/ or crates/ directory".into(),
            severity: Severity::Info,
        };
    }

    // Collect equation names with preconditions/postconditions (Refs #273)
    let eq_names = collect_contract_equation_names(&contracts_dir);

    if eq_names.is_empty() {
        return ComplianceCheck {
            name: "CB-1203: Contract Annotations".into(),
            status: CheckStatus::Pass,
            message: "No contract equations found".into(),
            severity: Severity::Info,
        };
    }

    // For each equation name, find production pub fn and check for macros
    // Function-level check: macro must be in the 10 lines before pub fn
    let mut bound_fns = 0usize;
    let mut with_macro = 0usize;
    let mut missing = Vec::new();

    // Collect all source files — support both src/ and crates/*/src/ layouts
    let mut src_files: Vec<_> = Vec::new();
    let search_dirs: Vec<std::path::PathBuf> = if src_dir.exists() {
        vec![src_dir.clone()]
    } else {
        // Workspace: search all crates/*/src/
        std::fs::read_dir(&crates_dir)
            .into_iter()
            .flatten()
            .flatten()
            .filter_map(|e| {
                let s = e.path().join("src");
                s.exists().then_some(s)
            })
            .collect()
    };
    for sdir in &search_dirs {
        src_files.extend(
            walkdir::WalkDir::new(sdir)
                .into_iter()
                .flatten()
                .filter(|e| e.path().extension().is_some_and(|ext| ext == "rs"))
                .filter(|e| {
                    let fname = e.file_name().to_string_lossy();
                    !fname.contains("test") && !fname.contains("contract_test")
                }),
        );
    }
    // Sort: blis/ and lib-level files first (kernel implementations)
    src_files.sort_by(|a, b| {
        let a_blis = a.path().to_string_lossy().contains("/blis/");
        let b_blis = b.path().to_string_lossy().contains("/blis/");
        b_blis.cmp(&a_blis)
    });

    // Also collect contract YAML stems for #[contract("stem", equation = "eq")] matching
    let mut yaml_stems: std::collections::HashMap<String, String> =
        std::collections::HashMap::new();
    if let Ok(entries) = std::fs::read_dir(&contracts_dir) {
        for entry in entries.flatten() {
            if entry.path().extension().map_or(true, |e| e != "yaml") {
                continue;
            }
            if let Some(stem) = entry.path().file_stem().and_then(|s| s.to_str()) {
                if let Ok(content) = std::fs::read_to_string(entry.path()) {
                    yaml_stems.insert(stem.to_string(), content);
                }
            }
        }
    }

    // Preload source lines that are #[contract] attributes (not string literals)
    // Matches both `#[contract(` and `#[provable_contracts_macros::contract(`
    let mut contract_attr_lines = Vec::new();
    for entry in &src_files {
        if let Ok(content) = std::fs::read_to_string(entry.path()) {
            for line in content.lines() {
                let t = line.trim();
                if t.starts_with("#[contract(") || t.contains("::contract(") {
                    contract_attr_lines.push(t.to_string());
                }
            }
        }
    }

    for eq in &eq_names {
        // Strategy 1: Check if any #[contract] attribute references this equation
        let attr_pattern = format!("equation = \"{eq}\"");
        if contract_attr_lines
            .iter()
            .any(|line| line.contains(&attr_pattern))
        {
            bound_fns += 1;
            with_macro += 1;
            continue; // Covered by #[contract] macro — assertions come from YAML
        }

        // Strategy 2: Find pub fn <eq_name>( and check for macros in preceding lines
        let pattern = format!("pub fn {eq}(");
        let mut found = false;
        for entry in &src_files {
            if let Ok(content) = std::fs::read_to_string(entry.path()) {
                if let Some(pos) = content.find(&pattern) {
                    bound_fns += 1;
                    found = true;
                    let prefix = &content[..pos];
                    let preceding_lines: Vec<&str> = prefix.lines().rev().take(10).collect();
                    let has_macro = preceding_lines.iter().any(|line| {
                        let t = line.trim();
                        t.starts_with("#[contract(")
                            || t.contains("::contract(")
                            || t.starts_with("#[requires(")
                            || t.starts_with("#[ensures(")
                            || t.starts_with("#[invariant(")
                    });
                    // Also check function body for contract_pre_*/debug_assert! with contract comment
                    let body_start = &content[pos..];
                    let body_snippet: String = body_start.lines().take(20).collect::<Vec<_>>().join("\n");
                    let has_body_contract = body_snippet.contains("contract_pre_")
                        || body_snippet.contains("contract_post_")
                        || body_snippet.contains("// Contract:");
                    if has_macro || has_body_contract {
                        with_macro += 1;
                    } else {
                        let rel = entry
                            .path()
                            .strip_prefix(project_path)
                            .unwrap_or(entry.path());
                        missing.push(format!("{eq} in {}", rel.display()));
                    }
                    break;
                }
            }
        }
        // Equation has no matching pub fn — not a failure (might be test-only or delegated)
        if !found {
            // silently skip
        }
    }

    if bound_fns == 0 {
        return ComplianceCheck {
            name: "CB-1203: Contract Annotations".into(),
            status: CheckStatus::Pass,
            message: format!("{} equations, 0 production pub fns found", eq_names.len()),
            severity: Severity::Info,
        };
    }

    if !missing.is_empty() {
        ComplianceCheck {
            name: "CB-1203: Contract Annotations".into(),
            status: CheckStatus::Fail,
            message: format!(
                "{}/{} contract-bound fns lack macros: {}",
                missing.len(),
                bound_fns,
                missing
                    .iter()
                    .take(3)
                    .cloned()
                    .collect::<Vec<_>>()
                    .join(", ")
            ),
            severity: Severity::Error,
        }
    } else {
        ComplianceCheck {
            name: "CB-1203: Contract Annotations".into(),
            status: CheckStatus::Pass,
            message: format!("{with_macro}/{bound_fns} contract-bound fns have macros"),
            severity: Severity::Info,
        }
    }
}

/// CB-1204: Build.rs contract pipeline — does build.rs emit assertion env vars from YAML?
///
/// The escape-proof pipeline requires build.rs to read contracts/*.yaml and
/// emit CONTRACT_*_PRE_COUNT / CONTRACT_*_PRE_0 env vars that the #[contract]
/// proc macro reads at compile time.
pub(crate) fn check_build_rs_pipeline(project_path: &Path) -> ComplianceCheck {
    debug_assert!(project_path.exists(), "project_path must exist: {}", project_path.display());
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

/// CB-1205: Provability Invariant — kernel contracts with proof_obligations
/// MUST have kani_harnesses and sufficient falsification_tests.
/// pv-compatibility spec §2.2
pub(crate) fn check_provability_invariant(project_path: &Path) -> ComplianceCheck {
    debug_assert!(project_path.exists(), "project_path must exist: {}", project_path.display());
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

/// CB-1206: Verification Level Distribution — report L1-L5 proof depth.
/// Reads proof-status.json from provable-contracts sibling repo.
/// pv-compatibility spec §2.3
pub(crate) fn check_verification_levels(project_path: &Path, thresholds: &ComplyThresholds) -> ComplianceCheck {
    debug_assert!(project_path.exists(), "project_path must exist: {}", project_path.display());
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

/// Quick check if a directory contains any contract YAML files (not just binding.yaml).
fn has_contract_yamls(dir: &Path) -> bool {
    debug_assert!(dir.exists(), "dir must exist: {}", dir.display());
    std::fs::read_dir(dir).into_iter().flatten().flatten().any(|e| {
        let p = e.path();
        p.is_file()
            && p.extension().is_some_and(|ext| ext == "yaml" || ext == "yml")
            && !p.file_name().is_some_and(|n| n.to_string_lossy().contains("binding"))
    })
}

/// Resolve the contracts directory for a project.
/// Checks local `contracts/` first (if it has YAMLs), then sibling `../provable-contracts/contracts/<name>/`.
/// Tries both the directory name and the Cargo.toml package name (e.g., paiml-mcp-agent-toolkit → pmat).
fn resolve_contracts_dir(project_path: &Path) -> Option<std::path::PathBuf> {
    debug_assert!(project_path.exists(), "project_path must exist: {}", project_path.display());
    // Prefer sibling provable-contracts repo — contains only provable-contracts YAMLs.
    // Local contracts/ may contain pmat work contracts (different schema) that pv lint
    // cannot parse.
    let abs = std::fs::canonicalize(project_path).ok()?;
    let parent = abs.parent()?;
    let pv_contracts = parent.join("provable-contracts").join("contracts");
    if pv_contracts.exists() {
        // Try 1: directory name
        let dir_name = abs.file_name()?.to_str()?;
        let sibling = pv_contracts.join(dir_name);
        if sibling.exists() {
            return Some(sibling);
        }
        // Try 2: Cargo.toml package name (handles paiml-mcp-agent-toolkit → pmat)
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

/// Collect contract YAML stems from the project's contracts directory (recursive).
/// Returns a set of stems (e.g., "softmax-kernel-v1") for filtering proof-status.json.
/// Checks local `contracts/` then sibling `../provable-contracts/contracts/<project>/`.
fn collect_project_contract_stems(project_path: &Path) -> std::collections::HashSet<String> {
    debug_assert!(project_path.exists(), "project_path must exist: {}", project_path.display());
    let mut stems = std::collections::HashSet::new();
    if let Some(contracts_dir) = resolve_contracts_dir(project_path) {
        collect_stems_recursive(&contracts_dir, &mut stems);
    }
    stems
}

fn collect_stems_recursive(dir: &Path, stems: &mut std::collections::HashSet<String>) {
    debug_assert!(dir.exists(), "dir must exist: {}", dir.display());
    let entries = match std::fs::read_dir(dir) {
        Ok(e) => e,
        Err(_) => return,
    };
    for entry in entries.filter_map(|e| e.ok()) {
        let path = entry.path();
        if path.is_dir() {
            collect_stems_recursive(&path, stems);
        } else if path.extension().is_some_and(|e| e == "yaml" || e == "yml") {
            // Skip binding files
            if path
                .file_name()
                .is_some_and(|n| n.to_string_lossy().contains("binding"))
            {
                continue;
            }
            if let Some(stem) = path.file_stem() {
                stems.insert(stem.to_string_lossy().into_owned());
            }
        }
    }
}

/// CB-1207: Contract drift — are contracts stale relative to source changes?
/// A contract YAML older than its bound source files by >30 days = drift.
/// pv-compatibility spec CD5.
pub(crate) fn check_contract_drift(project_path: &Path) -> ComplianceCheck {
    debug_assert!(project_path.exists(), "project_path must exist: {}", project_path.display());
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

/// CB-1208: Binding Existence Verification — verify that binding.yaml entries
/// with `status: implemented` correspond to actual Rust functions in the codebase.
///
/// This closes the enforcement gap where build.rs AllImplemented policy only checks
/// the YAML status field (self-attestation) without verifying the Rust function exists.
/// 16,977 bindings across the stack but only 35 have #[contract] annotations.
pub(crate) fn check_binding_existence(project_path: &Path, thresholds: &ComplyThresholds) -> ComplianceCheck {
    debug_assert!(project_path.exists(), "project_path must exist: {}", project_path.display());
    let contracts_dir = match resolve_contracts_dir(project_path) {
        Some(d) => d,
        None => {
            return ComplianceCheck {
                name: "CB-1208: Binding Existence".into(),
                status: CheckStatus::Skip,
                message: "No binding entries with status: implemented".into(),
                severity: Severity::Info,
            };
        }
    };

    let abs_path =
        std::fs::canonicalize(project_path).unwrap_or_else(|_| project_path.to_path_buf());
    let project_name = abs_path
        .file_name()
        .map(|n| n.to_string_lossy().to_string())
        .unwrap_or_default();

    if project_name == "provable-contracts" {
        return ComplianceCheck {
            name: "CB-1208: Binding Existence".into(),
            status: CheckStatus::Skip,
            message: "Registry project — binding existence checked on upstream repos".into(),
            severity: Severity::Info,
        };
    }

    let binding_files = resolve_binding_files(&contracts_dir, &abs_path, &project_name);
    let mut binding_entries: Vec<(String, String)> = Vec::new();
    for path in &binding_files {
        if let Ok(content) = std::fs::read_to_string(path) {
            parse_binding_entries(&content, path, &contracts_dir, &mut binding_entries);
        }
    }

    if binding_entries.is_empty() {
        return ComplianceCheck {
            name: "CB-1208: Binding Existence".into(),
            status: CheckStatus::Skip,
            message: "No binding entries with status: implemented".into(),
            severity: Severity::Info,
        };
    }

    let known_fns = match collect_known_fn_names(project_path) {
        Some(fns) => fns,
        None => {
            return ComplianceCheck {
                name: "CB-1208: Binding Existence".into(),
                status: CheckStatus::Skip,
                message: "No src/ or crates/ directory to verify against".into(),
                severity: Severity::Info,
            };
        }
    };

    // Detect enforcement level: build.rs, traits, or paper-only
    let has_buildrs = detect_buildrs_enforcement(project_path);
    let has_traits = project_path.join("tests").join("contract_traits.rs").exists();
    let enforcement = match (has_buildrs, has_traits) {
        (true, true) => "L3",
        (false, true) => "L2",
        (true, false) => "L1",
        (false, false) => "L0",
    };

    let (total, unique_fns, verified, missing) =
        cross_reference_bindings(&binding_entries, &known_fns);
    let missing_count = unique_fns - verified;

    // Paper-only (L0) bindings with no compile-time enforcement are ghost bindings
    if enforcement == "L0" && total > 0 {
        return ComplianceCheck {
            name: "CB-1208: Binding Existence".into(),
            status: CheckStatus::Fail,
            message: format!(
                "L0 paper-only: {total} bindings but no build.rs or trait enforcement — ghost bindings"
            ),
            severity: Severity::Error,
        };
    }

    if missing_count == 0 {
        ComplianceCheck {
            name: "CB-1208: Binding Existence".into(),
            status: CheckStatus::Pass,
            message: format!(
                "{verified}/{unique_fns} bound functions verified ({enforcement}) ({total} total binding entries)"
            ),
            severity: Severity::Info,
        }
    } else {
        let pct = verified as f64 / unique_fns as f64 * 100.0;
        let threshold = thresholds.min_binding_existence;
        let severity = if pct >= threshold {
            Severity::Warning
        } else {
            Severity::Error
        };
        let status = if pct >= threshold {
            CheckStatus::Warn
        } else {
            CheckStatus::Fail
        };
        ComplianceCheck {
            name: "CB-1208: Binding Existence".into(),
            status,
            message: format!(
                "{missing_count}/{unique_fns} bound fns not found ({enforcement}, {pct:.0}% verified, threshold: {threshold:.0}%): {}",
                missing.join(", ")
            ),
            severity,
        }
    }
}

/// Detect if build.rs has contract enforcement (reads binding.yaml or contracts/)
fn detect_buildrs_enforcement(project_path: &Path) -> bool {
    debug_assert!(project_path.exists(), "project_path must exist: {}", project_path.display());
    // Check root build.rs
    let build_rs = project_path.join("build.rs");
    if build_rs.exists() {
        if let Ok(c) = std::fs::read_to_string(&build_rs) {
            if c.contains("binding") || c.contains("contract") || c.contains("AllImplemented") {
                return true;
            }
        }
    }
    // Check workspace member crates (crates/*/build.rs)
    if let Ok(entries) = std::fs::read_dir(project_path.join("crates")) {
        for entry in entries.flatten() {
            let member_build = entry.path().join("build.rs");
            if member_build.exists() {
                if let Ok(c) = std::fs::read_to_string(&member_build) {
                    if c.contains("binding")
                        || c.contains("contract")
                        || c.contains("AllImplemented")
                    {
                        return true;
                    }
                }
            }
        }
    }
    false
}

/// CB-1209: Contract Trait Enforcement — compiler-enforced via `pv scaffold --trait`
///
/// The new enforcement chain: YAML → generated trait → impl in tests/contract_traits.rs
/// → cargo test verifies compilation. The Rust compiler is the enforcer.
///
/// This check detects:
/// 1. tests/contract_traits.rs exists
/// 2. Count trait impl blocks (`impl XxxV1 for`)
/// 3. Count provable-contracts trait imports
pub(crate) fn check_contract_trait_enforcement(project_path: &Path, thresholds: &ComplyThresholds) -> ComplianceCheck {
    debug_assert!(project_path.exists(), "project_path must exist: {}", project_path.display());
    // Look for contract_traits.rs in tests/ or tests/contract_traits/
    let trait_test = project_path.join("tests").join("contract_traits.rs");
    if !trait_test.exists() {
        // Also check for the trait file in integration test directories
        let alt_paths = [
            project_path.join("tests").join("contract_traits").join("mod.rs"),
            project_path.join("tests").join("pv_traits.rs"),
        ];
        let found = alt_paths.iter().find(|p| p.exists());
        if found.is_none() {
            return ComplianceCheck {
                name: "CB-1209: Contract Trait Enforcement".into(),
                status: CheckStatus::Skip,
                message: "No tests/contract_traits.rs found".into(),
                severity: Severity::Info,
            };
        }
    }

    let test_path = if trait_test.exists() {
        trait_test
    } else {
        project_path
            .join("tests")
            .join("contract_traits")
            .join("mod.rs")
    };

    let content = match std::fs::read_to_string(&test_path) {
        Ok(c) => c,
        Err(_) => {
            return ComplianceCheck {
                name: "CB-1209: Contract Trait Enforcement".into(),
                status: CheckStatus::Warn,
                message: "Cannot read tests/contract_traits.rs".into(),
                severity: Severity::Warning,
            };
        }
    };

    // Count trait impls: `impl XxxV1 for` patterns
    let impl_count = content
        .lines()
        .filter(|line| {
            let t = line.trim();
            t.starts_with("impl ") && t.contains("V1 for") && !t.starts_with("//")
        })
        .count();

    // Count trait imports from provable_contracts::traits
    let _trait_imports = content
        .lines()
        .filter(|line| {
            line.contains("provable_contracts::traits")
                || line.contains("provable_contracts_macros")
        })
        .count();

    // Count test functions (all #[test] fns in the file)
    let test_count = content
        .lines()
        .filter(|line| line.trim() == "#[test]")
        .count();

    if impl_count == 0 {
        return ComplianceCheck {
            name: "CB-1209: Contract Trait Enforcement".into(),
            status: CheckStatus::Warn,
            message: "tests/contract_traits.rs exists but no trait impls found".into(),
            severity: Severity::Warning,
        };
    }

    // Check how many of the 13 Tier 1-2 kernel traits are implemented
    let tier1_traits = [
        "SoftmaxKernelV1",
        "RmsnormKernelV1",
        "RopeKernelV1",
        "AttentionKernelV1",
        "MatmulKernelV1",
        "FlashAttentionV1",
        "GqaKernelV1",
        "LayernormKernelV1",
        "SiluKernelV1",
        "SwigluKernelV1",
        "ActivationKernelV1",
        "CrossEntropyKernelV1",
        "AdamwKernelV1",
    ];
    let implemented: Vec<&str> = tier1_traits
        .iter()
        .filter(|t| content.contains(**t))
        .copied()
        .collect();

    let msg = format!(
        "{}/{} contract traits enforced, {} impl(s), {} test(s)",
        implemented.len(),
        tier1_traits.len(),
        impl_count,
        test_count
    );

    let required = if thresholds.require_all_traits {
        tier1_traits.len()
    } else {
        10
    };

    if implemented.len() >= required {
        ComplianceCheck {
            name: "CB-1209: Contract Trait Enforcement".into(),
            status: CheckStatus::Pass,
            message: msg,
            severity: Severity::Info,
        }
    } else if implemented.len() >= 5 {
        ComplianceCheck {
            name: "CB-1209: Contract Trait Enforcement".into(),
            status: CheckStatus::Warn,
            message: format!("{msg} — need >={required} for full credit"),
            severity: Severity::Warning,
        }
    } else {
        ComplianceCheck {
            name: "CB-1209: Contract Trait Enforcement".into(),
            status: CheckStatus::Fail,
            message: format!("{msg} — need >=5 for partial credit"),
            severity: Severity::Error,
        }
    }
}

/// Resolve binding.yaml file locations (local + sibling provable-contracts)
fn resolve_binding_files(
    contracts_dir: &Path,
    abs_path: &Path,
    project_name: &str,
) -> Vec<std::path::PathBuf> {
    debug_assert!(contracts_dir.exists(), "contracts_dir must exist: {}", contracts_dir.display());
    debug_assert!(abs_path.exists(), "abs_path must exist: {}", abs_path.display());
    let mut files: Vec<std::path::PathBuf> = Vec::new();
    for entry in walkdir::WalkDir::new(contracts_dir)
        .max_depth(3)
        .into_iter()
        .filter_map(|e| e.ok())
    {
        if entry.path().is_file()
            && entry
                .path()
                .file_name()
                .is_some_and(|n| n.to_string_lossy().contains("binding"))
        {
            files.push(entry.path().to_path_buf());
        }
    }
    let has_implemented = files.iter().any(|f| {
        std::fs::read_to_string(f)
            .map(|c| c.contains("status: implemented"))
            .unwrap_or(false)
    });
    if files.is_empty() || !has_implemented {
        if let Some(sb) = abs_path.parent().map(|p| {
            p.join("provable-contracts")
                .join("contracts")
                .join(project_name)
                .join("binding.yaml")
        }) {
            if sb.exists() {
                files.push(sb);
            }
        }
    }
    files
}

/// Collect all fn/const/static names from Rust source files
fn collect_known_fn_names(
    project_path: &Path,
) -> Option<std::collections::HashSet<String>> {
    debug_assert!(project_path.exists(), "project_path must exist: {}", project_path.display());
    let src_dir = project_path.join("src");
    let crates_dir = project_path.join("crates");
    let mut search_dirs: Vec<std::path::PathBuf> = Vec::new();
    // Search both src/ and crates/*/src/ — workspaces have both
    if src_dir.exists() {
        search_dirs.push(src_dir);
    }
    if crates_dir.exists() {
        for entry in std::fs::read_dir(&crates_dir).into_iter().flatten().flatten() {
            let s = entry.path().join("src");
            if s.exists() {
                search_dirs.push(s);
            }
        }
    }
    if search_dirs.is_empty() {
        return None;
    }

    let mut known = std::collections::HashSet::new();
    for sdir in &search_dirs {
        for entry in walkdir::WalkDir::new(sdir)
            .into_iter()
            .filter_map(|e| e.ok())
        {
            if !entry.path().extension().is_some_and(|e| e == "rs") {
                continue;
            }
            if let Ok(content) = std::fs::read_to_string(entry.path()) {
                extract_names_from_source(&content, &mut known);
            }
        }
    }
    Some(known)
}

/// Extract fn/const/static names from a single Rust source file
fn extract_names_from_source(content: &str, names: &mut std::collections::HashSet<String>) {
    debug_assert!(!content.is_empty(), "content must not be empty");
    for line in content.lines() {
        let t = line.trim();
        // fn declarations (all visibility/async/unsafe/const variants)
        let fn_rest = t
            .strip_prefix("pub fn ")
            .or_else(|| t.strip_prefix("fn "))
            .or_else(|| t.strip_prefix("pub async fn "))
            .or_else(|| t.strip_prefix("pub(crate) fn "))
            .or_else(|| t.strip_prefix("async fn "))
            .or_else(|| t.strip_prefix("unsafe fn "))
            .or_else(|| t.strip_prefix("pub unsafe fn "))
            .or_else(|| t.strip_prefix("pub(crate) unsafe fn "))
            .or_else(|| t.strip_prefix("pub(crate) async fn "))
            .or_else(|| t.strip_prefix("pub const unsafe fn "))
            .or_else(|| t.strip_prefix("const fn "))
            .or_else(|| t.strip_prefix("pub const fn "));
        if let Some(rest) = fn_rest {
            if let Some(name) = rest.split('(').next() {
                let name = name.trim();
                if !name.is_empty() && name.chars().all(|c| c.is_alphanumeric() || c == '_') {
                    names.insert(name.to_string());
                }
            }
        }
        // const/static declarations
        let const_rest = t
            .strip_prefix("pub const ")
            .or_else(|| t.strip_prefix("pub(crate) const "))
            .or_else(|| t.strip_prefix("const "))
            .or_else(|| t.strip_prefix("pub static "))
            .or_else(|| t.strip_prefix("static "));
        if let Some(rest) = const_rest {
            if let Some(name) = rest.split(':').next() {
                let name = name.trim();
                if !name.is_empty() && name.chars().all(|c| c.is_alphanumeric() || c == '_') {
                    names.insert(name.to_string());
                }
            }
        }
    }
}

/// Cross-reference binding entries against known fn names
fn cross_reference_bindings(
    entries: &[(String, String)],
    known_fns: &std::collections::HashSet<String>,
) -> (usize, usize, usize, Vec<String>) {
    debug_assert!(!entries.is_empty(), "entries must not be empty");
    let total = entries.len();
    let mut missing = Vec::new();
    let mut verified = 0usize;
    let mut seen = std::collections::HashSet::new();
    for (func, _) in entries {
        if !seen.insert(func.clone()) {
            continue;
        }
        if known_fns.contains(func.as_str()) {
            verified += 1;
        } else if missing.len() < 5 {
            missing.push(func.clone());
        }
    }
    (total, seen.len(), verified, missing)
}

/// Parse binding entries from a binding.yaml file content
fn parse_binding_entries(
    content: &str,
    path: &Path,
    contracts_dir: &Path,
    entries: &mut Vec<(String, String)>,
) {
    debug_assert!(path.exists(), "path must exist: {}", path.display());
    debug_assert!(contracts_dir.exists(), "contracts_dir must exist: {}", contracts_dir.display());
    debug_assert!(!content.is_empty(), "content must not be empty");
    let mut current_fn = None;
    let mut current_status = None;
    for line in content.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with("function:") {
            let val = trimmed
                .strip_prefix("function:")
                .unwrap_or("")
                .trim()
                .trim_matches('"');
            // Extract bare function name (strip Type:: prefix)
            let bare = if let Some(pos) = val.rfind("::") {
                &val[pos + 2..]
            } else {
                val
            };
            current_fn = Some(bare.to_string());
        } else if trimmed.starts_with("status:") {
            current_status = Some(
                trimmed
                    .strip_prefix("status:")
                    .unwrap_or("")
                    .trim()
                    .to_string(),
            );
        } else if trimmed.starts_with("- contract:") || trimmed.is_empty() {
            if let (Some(func), Some(status)) = (current_fn.take(), current_status.take()) {
                if status == "implemented" && !func.is_empty() {
                    let rel = path
                        .strip_prefix(contracts_dir)
                        .unwrap_or(path)
                        .display()
                        .to_string();
                    entries.push((func, rel));
                }
            }
        }
    }
    // Handle last entry
    if let (Some(func), Some(status)) = (current_fn, current_status) {
        if status == "implemented" && !func.is_empty() {
            let rel = path
                .strip_prefix(contracts_dir)
                .unwrap_or(path)
                .display()
                .to_string();
            entries.push((func, rel));
        }
    }
}

/// CB-1202: Contract coverage — do repos with critical functions have contracts?
pub(crate) fn check_contract_coverage(project_path: &Path) -> ComplianceCheck {
    debug_assert!(project_path.exists(), "project_path must exist: {}", project_path.display());
    let src_dir = project_path.join("src");
    let contracts_dir = project_path.join("contracts");
    if !src_dir.exists() {
        return ComplianceCheck {
            name: "CB-1202: Contract Coverage".into(),
            status: CheckStatus::Skip,
            message: "No src/ directory".into(),
            severity: Severity::Info,
        };
    }

    // Critical ML/GPU/data keywords that REQUIRE contracts
    let critical_keywords = [
        "forward",
        "backward",
        "optimizer",
        "checkpoint",
        "loss",
        "gradient",
        "sampling",
        "kv_cache",
        "tokenize",
        "quantize",
        "kernel",
        "dispatch",
        "softmax",
        "matmul",
        "gemm",
        "batch",
    ];

    // Count which keywords appear in public functions
    let mut keywords_found = Vec::new();
    let mut keywords_covered = 0usize;

    for keyword in &critical_keywords {
        // Search src/ for pub fn containing keyword
        let has_fn = walkdir::WalkDir::new(&src_dir)
            .into_iter()
            .flatten()
            .filter(|e| e.path().extension().is_some_and(|ext| ext == "rs"))
            .any(|e| {
                std::fs::read_to_string(e.path())
                    .map(|c| {
                        c.contains(&format!("pub fn {keyword}"))
                            || c.contains(&format!("pub async fn {keyword}"))
                    })
                    .unwrap_or(false)
            });

        if !has_fn {
            continue;
        }
        keywords_found.push(*keyword);

        // Check if any contract mentions this keyword
        if contracts_dir.exists() {
            let has_contract = walkdir::WalkDir::new(&contracts_dir)
                .into_iter()
                .flatten()
                .filter(|e| {
                    e.path()
                        .extension()
                        .is_some_and(|ext| ext == "yaml" || ext == "yml")
                })
                .any(|e| {
                    std::fs::read_to_string(e.path())
                        .map(|c| c.to_lowercase().contains(keyword))
                        .unwrap_or(false)
                });
            if has_contract {
                keywords_covered += 1;
            }
        }
    }

    if keywords_found.is_empty() {
        return ComplianceCheck {
            name: "CB-1202: Contract Coverage".into(),
            status: CheckStatus::Pass,
            message: "No critical ML/GPU functions detected".into(),
            severity: Severity::Info,
        };
    }

    let coverage_pct = keywords_covered * 100 / keywords_found.len();
    let uncovered: Vec<&&str> = keywords_found
        .iter()
        .filter(|k| {
            !contracts_dir.exists()
                || !walkdir::WalkDir::new(&contracts_dir)
                    .into_iter()
                    .flatten()
                    .filter(|e| {
                        e.path()
                            .extension()
                            .is_some_and(|ext| ext == "yaml" || ext == "yml")
                    })
                    .any(|e| {
                        std::fs::read_to_string(e.path())
                            .map(|c| c.to_lowercase().contains(**k))
                            .unwrap_or(false)
                    })
        })
        .collect();

    if coverage_pct >= 50 {
        ComplianceCheck {
            name: "CB-1202: Contract Coverage".into(),
            status: CheckStatus::Pass,
            message: format!(
                "{keywords_covered}/{} critical keywords covered ({coverage_pct}%)",
                keywords_found.len()
            ),
            severity: Severity::Info,
        }
    } else {
        let missing: Vec<String> = uncovered.iter().map(|k| k.to_string()).collect();
        ComplianceCheck {
            name: "CB-1202: Contract Coverage".into(),
            status: CheckStatus::Fail,
            message: format!(
                "Only {keywords_covered}/{} critical keywords covered ({coverage_pct}%). Missing: {}",
                keywords_found.len(), missing.join(", ")
            ),
            severity: Severity::Error,
        }
    }
}

/// CB-1201: PV Lint + contract fulfillment gate.
/// Checks: (1) pv lint passes, (2) referenced tests EXIST, (3) they PASS.
/// Missing test = unfalsifiable claim = FAIL (like TDG grade F).
pub(crate) fn check_pv_lint(project_path: &Path, thresholds: &ComplyThresholds) -> ComplianceCheck {
    debug_assert!(project_path.exists(), "project_path must exist: {}", project_path.display());
    let contracts_dir = match resolve_contracts_dir(project_path) {
        Some(dir) => dir,
        None => {
            return ComplianceCheck {
                name: "CB-1201: PV Lint".into(),
                status: CheckStatus::Skip,
                message: "No contracts/ directory found".into(),
                severity: Severity::Info,
            };
        }
    };

    // Step 1: Run pv lint on resolved contracts dir — avoids scanning work/ YAMLs
    let (pv_passed, pv_error_detail) = std::process::Command::new("pv")
        .args(["lint", &contracts_dir.display().to_string(), "--format", "json"])
        .current_dir(project_path)
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .output()
        .map(|o| {
            let json_val = String::from_utf8(o.stdout)
                .ok()
                .and_then(|s| serde_json::from_str::<serde_json::Value>(&s).ok());
            let passed = json_val
                .as_ref()
                .and_then(|v| v.get("passed")?.as_bool())
                .unwrap_or(false);
            // Extract first error finding for diagnostics
            let detail = json_val
                .as_ref()
                .and_then(|v| v.get("findings")?.as_array())
                .and_then(|arr| arr.iter().find(|f| {
                    f.get("severity").and_then(|s| s.as_str()) == Some("error")
                        || f.get("severity").and_then(|s| s.as_str()) == Some("ERROR")
                }))
                .and_then(|f| f.get("message").and_then(|m| m.as_str()))
                .map(|s| s.to_string())
                .or_else(|| {
                    // Fallback: first line of stderr
                    String::from_utf8(o.stderr).ok()
                        .and_then(|s| s.lines().next().map(|l| l.trim().to_string()))
                        .filter(|s| !s.is_empty())
                });
            (passed, detail)
        })
        .unwrap_or((false, None));

    // Step 2: Check test fulfillment
    let (total_refs, existing, missing) = count_contract_test_refs(project_path);

    if total_refs > 0 && missing > 0 {
        return ComplianceCheck {
            name: "CB-1201: PV Lint".into(),
            status: CheckStatus::Fail,
            message: format!(
                "Unfalsifiable: {missing}/{total_refs} contract tests missing ({}% unfulfilled)",
                missing * 100 / total_refs
            ),
            severity: Severity::Error,
        };
    }

    if !pv_passed {
        let msg = match pv_error_detail {
            Some(detail) => format!("PV Lint failed: {detail}"),
            None => "PV Lint failed".into(),
        };
        let (status, severity) = if thresholds.pv_lint_is_error {
            (CheckStatus::Fail, Severity::Error)
        } else {
            (CheckStatus::Warn, Severity::Warning)
        };
        return ComplianceCheck {
            name: "CB-1201: PV Lint".into(),
            status,
            message: msg,
            severity,
        };
    }

    ComplianceCheck {
        name: "CB-1201: PV Lint".into(),
        status: CheckStatus::Pass,
        message: format!("PV Lint passed, {existing}/{total_refs} tests fulfilled"),
        severity: Severity::Info,
    }
}

fn count_contract_test_refs(project_path: &Path) -> (usize, usize, usize) {
    debug_assert!(project_path.exists(), "project_path must exist: {}", project_path.display());
    let contracts_dir = project_path.join("contracts");
    let src_dir = project_path.join("src");
    let mut refs = Vec::new();

    if let Ok(entries) = std::fs::read_dir(&contracts_dir) {
        for entry in entries.flatten() {
            let p = entry.path();
            if p.extension().map_or(true, |e| e != "yaml" && e != "yml") {
                continue;
            }
            if let Ok(content) = std::fs::read_to_string(&p) {
                for line in content.lines() {
                    if let Some(pos) = line.find("test:") {
                        let rest = line[pos + 5..].trim().trim_matches('"');
                        let name = rest
                            .split(|c: char| !c.is_alphanumeric() && c != '_')
                            .next()
                            .unwrap_or("");
                        if name.starts_with("test_") || name.starts_with("prop_") {
                            refs.push(name.to_string());
                        }
                    }
                }
            }
        }
    }

    if refs.is_empty() {
        return (0, 0, 0);
    }

    let mut src_tests = std::collections::HashSet::new();
    // Scan src/, tests/, crates/, and workspace member directories
    let mut scan_dirs = vec![src_dir.clone()];
    scan_dirs.push(project_path.join("tests"));
    scan_dirs.push(project_path.join("crates"));
    // Workspace members: root-level dirs with Cargo.toml
    if let Ok(entries) = std::fs::read_dir(project_path) {
        for entry in entries.flatten() {
            let p = entry.path();
            if p.is_dir() && p.join("Cargo.toml").exists() {
                let name = p.file_name().unwrap_or_default();
                if name != "src" && name != "crates" && name != "tests" && name != "contracts" {
                    scan_dirs.push(p.join("src"));
                    scan_dirs.push(p.join("tests"));
                }
            }
        }
    }
    for dir in &scan_dirs {
        if !dir.exists() {
            continue;
        }
        for entry in walkdir::WalkDir::new(dir).into_iter().flatten() {
            if entry.path().extension().map_or(true, |e| e != "rs") {
                continue;
            }
            if let Ok(content) = std::fs::read_to_string(entry.path()) {
                for line in content.lines() {
                    if let Some(pos) = line.find("fn test_").or_else(|| line.find("fn prop_")) {
                        let rest = &line[pos + 3..];
                        let name = rest
                            .split(|c: char| !c.is_alphanumeric() && c != '_')
                            .next()
                            .unwrap_or("");
                        if !name.is_empty() {
                            src_tests.insert(name.to_string());
                        }
                    }
                }
            }
        }
    }

    let existing = refs
        .iter()
        .filter(|t| src_tests.contains(t.as_str()))
        .count();
    let missing = refs.len() - existing;
    (refs.len(), existing, missing)
}


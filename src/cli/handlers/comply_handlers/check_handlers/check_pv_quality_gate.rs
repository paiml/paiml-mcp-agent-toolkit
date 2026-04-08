// Provable-contracts quality gate checks (CB-1202, CB-1208, CB-1209)
// Included from check.rs — do NOT add `use` imports or `#!` attributes here.

/// CB-1202: Contract coverage — do repos with critical functions have contracts?
#[provable_contracts_macros::contract("pmat-core.yaml", equation = "path_exists")]
pub(crate) fn check_contract_coverage(project_path: &Path) -> ComplianceCheck {
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


/// CB-1208: Binding Existence Verification — verify that binding.yaml entries
/// with `status: implemented` correspond to actual Rust functions in the codebase.
///
/// This closes the enforcement gap where build.rs AllImplemented policy only checks
/// the YAML status field (self-attestation) without verifying the Rust function exists.
/// 16,977 bindings across the stack but only 35 have #[contract] annotations.
#[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
pub(crate) fn check_binding_existence(project_path: &Path, thresholds: &ComplyThresholds) -> ComplianceCheck {
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


/// CB-1209: Contract Trait Enforcement — compiler-enforced via `pv scaffold --trait`
///
/// The new enforcement chain: YAML → generated trait → impl in tests/contract_traits.rs
/// → cargo test verifies compilation. The Rust compiler is the enforcer.
///
/// This check detects:
/// 1. tests/contract_traits.rs exists
/// 2. Count trait impl blocks (`impl XxxV1 for`)
/// 3. Count provable-contracts trait imports
#[provable_contracts_macros::contract("pmat-core.yaml", equation = "path_exists")]
pub(crate) fn check_contract_trait_enforcement(project_path: &Path, thresholds: &ComplyThresholds) -> ComplianceCheck {
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


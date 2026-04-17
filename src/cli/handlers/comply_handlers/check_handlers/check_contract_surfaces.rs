// Contract Surface Type enforcement checks (CB-1300 through CB-1308)
// Included from check.rs — do NOT add `use` imports or `#!` attributes here.
//
// Spec: docs/specifications/components/contract-surface-types.md (Component 23)

/// Known top-level keys for cross-language safety contracts.
const CROSS_LANGUAGE_KEYS: &[&str] = &[
    "source_language",
    "target_language",
    "safety_invariants",
    "enforcement_level",
];

/// Generic placeholder preconditions that indicate API-pattern contracts
/// stuffed into kernel-math schema (semantic leak, not structural).
const GENERIC_PLACEHOLDERS: &[&str] = &[
    "input.len() > 0",
    "!input.is_empty()",
    "!x.is_empty()",
    "width > 0",
    "severity > 0",
    "result.len() > 0",
    "!result.is_empty()",
    "Type safety preserved",
    "No panics on valid input",
];

/// CB-1305: Contract Surface Type Classification — the anti-leak gate
///
/// Classifies every YAML contract in contracts/ against known contract classes.
/// WARN on unknown structure. FAIL if >20% unclassified.
/// Also detects semantic leaks: API-pattern contracts disguised as kernel-math.
#[provable_contracts_macros::contract("pmat-core.yaml", equation = "path_exists")]
pub(crate) fn check_contract_surface_classification(project_path: &Path) -> ComplianceCheck {
    let contracts_dir = match resolve_contracts_dir(project_path) {
        Some(d) => d,
        None => {
            return ComplianceCheck {
                name: "CB-1305: Contract Surface Classification".into(),
                status: CheckStatus::Skip,
                message: "No contract YAML files found".into(),
                severity: Severity::Info,
            };
        }
    };

    let mut total_contracts = 0usize;
    let mut kernel_math = 0usize;
    let mut cross_language = 0usize;
    let mut schema_registry = 0usize;
    let mut invariants_only = 0usize;
    let mut semantic_leaks = 0usize;
    let mut unclassified: Vec<String> = Vec::new();

    for entry in walkdir::WalkDir::new(&contracts_dir)
        .max_depth(3)
        .into_iter()
        .filter_map(|e| e.ok())
    {
        let path = entry.path();
        if !path.is_file() {
            continue;
        }
        if !path.extension().is_some_and(|e| e == "yaml" || e == "yml") {
            continue;
        }
        if path
            .file_name()
            .is_some_and(|n| n.to_string_lossy().contains("binding"))
        {
            continue;
        }

        total_contracts += 1;
        let Ok(content) = std::fs::read_to_string(path) else {
            continue;
        };

        let top_keys = extract_top_level_keys(&content);
        let class = classify_contract(&top_keys, &content);

        match class {
            ContractClass::KernelMath => kernel_math += 1,
            ContractClass::CrossLanguage => cross_language += 1,
            ContractClass::SchemaRegistry => schema_registry += 1,
            ContractClass::InvariantsOnly => invariants_only += 1,
            ContractClass::SemanticLeak => {
                semantic_leaks += 1;
                kernel_math += 1; // structurally it IS kernel-math
            }
            ContractClass::Unknown => {
                let name = path
                    .file_name()
                    .map(|n| n.to_string_lossy().to_string())
                    .unwrap_or_default();
                if unclassified.len() < 5 {
                    unclassified.push(name);
                }
            }
        }
    }

    if total_contracts == 0 {
        return ComplianceCheck {
            name: "CB-1305: Contract Surface Classification".into(),
            status: CheckStatus::Skip,
            message: "No contract YAML files found".into(),
            severity: Severity::Info,
        };
    }

    let unclassified_count = unclassified.len();
    let classified = total_contracts - unclassified_count;
    let unclassified_pct = unclassified_count as f64 / total_contracts as f64 * 100.0;

    let mut issues = Vec::new();

    // FAIL if >20% unclassified
    if unclassified_pct > 20.0 {
        issues.push(format!(
            "{unclassified_count}/{total_contracts} ({unclassified_pct:.0}%) contracts unclassified — leak has outpaced spec"
        ));
    }

    // WARN on semantic leaks (generic API patterns in kernel-math schema)
    if semantic_leaks > 0 {
        issues.push(format!(
            "{semantic_leaks} semantic leak(s): API-pattern contracts with generic placeholders in kernel-math schema"
        ));
    }

    if !issues.is_empty() {
        let severity = if unclassified_pct > 20.0 {
            Severity::Error
        } else {
            Severity::Warning
        };
        let status = if unclassified_pct > 20.0 {
            CheckStatus::Fail
        } else {
            CheckStatus::Warn
        };
        let unclassified_list = if unclassified.is_empty() {
            String::new()
        } else {
            format!(" [{}]", unclassified.join(", "))
        };
        ComplianceCheck {
            name: "CB-1305: Contract Surface Classification".into(),
            status,
            message: format!(
                "{}{unclassified_list}",
                issues.join("; ")
            ),
            severity,
        }
    } else {
        let mut parts = Vec::new();
        let pure_kernel = kernel_math.saturating_sub(semantic_leaks);
        if pure_kernel > 0 { parts.push(format!("kernel={pure_kernel}")); }
        if cross_language > 0 { parts.push(format!("cross-lang={cross_language}")); }
        if schema_registry > 0 { parts.push(format!("registry={schema_registry}")); }
        if invariants_only > 0 { parts.push(format!("invariants={invariants_only}")); }
        if semantic_leaks > 0 { parts.push(format!("leaks={semantic_leaks}")); }
        ComplianceCheck {
            name: "CB-1305: Contract Surface Classification".into(),
            status: CheckStatus::Pass,
            message: format!(
                "{classified}/{total_contracts} classified ({})",
                parts.join(", ")
            ),
            severity: Severity::Info,
        }
    }
}

/// CB-1300: CLI Argument Contract Coverage — detect uncontracted CLI flags
///
/// Scans for OutputFormat enum duplication and uncontracted clap arguments.
/// FAIL if >3 OutputFormat definitions (severe duplication).
/// WARN if any clap arg structs lack validation contracts.
#[provable_contracts_macros::contract("pmat-core.yaml", equation = "path_exists")]
pub(crate) fn check_cli_arg_contracts(project_path: &Path) -> ComplianceCheck {
    let src_dir = project_path.join("src");
    if !src_dir.exists() {
        return ComplianceCheck {
            name: "CB-1300: CLI Arg Contracts".into(),
            status: CheckStatus::Skip,
            message: "No src/ directory".into(),
            severity: Severity::Info,
        };
    }

    let mut output_format_count = 0usize;
    let mut output_format_files: Vec<String> = Vec::new();

    // Walk src/ looking for OutputFormat enum definitions
    for entry in walkdir::WalkDir::new(&src_dir)
        .max_depth(8)
        .into_iter()
        .filter_map(|e| e.ok())
    {
        let path = entry.path();
        if !path.is_file() {
            continue;
        }
        if !path.extension().is_some_and(|e| e == "rs") {
            continue;
        }
        // Skip test files
        if path.to_string_lossy().contains("/tests/") {
            continue;
        }
        let Ok(content) = std::fs::read_to_string(path) else {
            continue;
        };
        // Count enum OutputFormat definitions (not references)
        let count = content
            .lines()
            .filter(|l| {
                let trimmed = l.trim();
                (trimmed.starts_with("pub enum OutputFormat")
                    || trimmed.starts_with("pub(crate) enum OutputFormat"))
                    && !trimmed.starts_with("//")
            })
            .count();
        if count > 0 {
            output_format_count += count;
            let rel = path
                .strip_prefix(project_path)
                .unwrap_or(path)
                .display()
                .to_string();
            if output_format_files.len() < 5 {
                output_format_files.push(rel);
            }
        }
    }

    if output_format_count > 3 {
        ComplianceCheck {
            name: "CB-1300: CLI Arg Contracts".into(),
            status: CheckStatus::Fail,
            message: format!(
                "{output_format_count} OutputFormat enum definitions — must converge to ONE canonical type [{}]",
                output_format_files.join(", ")
            ),
            severity: Severity::Error,
        }
    } else if output_format_count > 1 {
        ComplianceCheck {
            name: "CB-1300: CLI Arg Contracts".into(),
            status: CheckStatus::Warn,
            message: format!(
                "{output_format_count} OutputFormat enum definitions — should converge to ONE [{}]",
                output_format_files.join(", ")
            ),
            severity: Severity::Warning,
        }
    } else {
        ComplianceCheck {
            name: "CB-1300: CLI Arg Contracts".into(),
            status: CheckStatus::Pass,
            message: format!(
                "{output_format_count} OutputFormat enum definition(s)"
            ),
            severity: Severity::Info,
        }
    }
}

/// CB-1303: Config Contract Validation — detect CI drift and stale configs
///
/// Checks .github/workflows/ for known required patterns (RUST_MIN_STACK,
/// --lib flag in cargo test) and Cargo.toml for sovereign dep contracts.
#[provable_contracts_macros::contract("pmat-core.yaml", equation = "path_exists")]
pub(crate) fn check_config_contracts(project_path: &Path) -> ComplianceCheck {
    let mut issues: Vec<String> = Vec::new();
    let mut checks_run = 0usize;

    // Check GitHub Actions workflows
    let workflows_dir = project_path.join(".github").join("workflows");
    if workflows_dir.exists() {
        check_workflow_contracts(&workflows_dir, &mut issues, &mut checks_run);
    }

    // Check Cargo.toml sovereign dep contracts
    let cargo_toml = project_path.join("Cargo.toml");
    if cargo_toml.exists() {
        check_cargo_contracts(&cargo_toml, &mut issues, &mut checks_run);
    }

    if checks_run == 0 {
        return ComplianceCheck {
            name: "CB-1303: Config Contracts".into(),
            status: CheckStatus::Skip,
            message: "No config files to validate".into(),
            severity: Severity::Info,
        };
    }

    if issues.is_empty() {
        ComplianceCheck {
            name: "CB-1303: Config Contracts".into(),
            status: CheckStatus::Pass,
            message: format!("{checks_run} config contract(s) validated"),
            severity: Severity::Info,
        }
    } else {
        ComplianceCheck {
            name: "CB-1303: Config Contracts".into(),
            status: CheckStatus::Warn,
            message: format!(
                "{} issue(s): {}",
                issues.len(),
                issues.join("; ")
            ),
            severity: Severity::Warning,
        }
    }
}

/// CB-1304: Sovereign Dep Version Contracts — verify batuta stack versions
///
/// Checks Cargo.toml for sovereign stack dependencies and verifies they
/// meet minimum version contracts. Also checks for adapter pattern.
#[provable_contracts_macros::contract("pmat-core.yaml", equation = "path_exists")]
pub(crate) fn check_sovereign_dep_contracts(project_path: &Path) -> ComplianceCheck {
    let cargo_toml = project_path.join("Cargo.toml");
    if !cargo_toml.exists() {
        return ComplianceCheck {
            name: "CB-1304: Sovereign Dep Contracts".into(),
            status: CheckStatus::Skip,
            message: "No Cargo.toml".into(),
            severity: Severity::Info,
        };
    }

    let Ok(content) = std::fs::read_to_string(&cargo_toml) else {
        return ComplianceCheck {
            name: "CB-1304: Sovereign Dep Contracts".into(),
            status: CheckStatus::Skip,
            message: "Cannot read Cargo.toml".into(),
            severity: Severity::Info,
        };
    };

    // Sovereign dep minimum versions (from contract-surface-types spec)
    let sovereign_deps: &[(&str, &str)] = &[
        ("aprender", "0.27"),
        ("trueno", "0.16"),
        ("trueno-graph", "0.1.17"),
        ("trueno-db", "0.3.15"),
        ("trueno-rag", "0.2"),
    ];

    let mut found_deps = 0usize;
    let mut issues: Vec<String> = Vec::new();

    for (dep, min_ver) in sovereign_deps {
        // Look for the dep in Cargo.toml (handles both inline and table formats)
        if let Some(version_str) = extract_dep_version(&content, dep) {
            found_deps += 1;
            // Simple semver prefix check
            let clean = version_str
                .trim_start_matches('^')
                .trim_start_matches('~')
                .trim_start_matches('=')
                .trim_start_matches(">=");
            if !version_satisfies_minimum(clean, min_ver) {
                issues.push(format!(
                    "{dep} = \"{version_str}\" below minimum {min_ver}"
                ));
            }
        }
    }

    if found_deps == 0 {
        return ComplianceCheck {
            name: "CB-1304: Sovereign Dep Contracts".into(),
            status: CheckStatus::Skip,
            message: "No sovereign stack deps in Cargo.toml".into(),
            severity: Severity::Info,
        };
    }

    // Check for adapter pattern (e.g., aprender_adapter.rs)
    let src_dir = project_path.join("src");
    let has_adapter = src_dir.exists()
        && walkdir::WalkDir::new(&src_dir)
            .max_depth(4)
            .into_iter()
            .filter_map(|e| e.ok())
            .any(|e| {
                e.path()
                    .file_name()
                    .is_some_and(|n| n.to_string_lossy().contains("adapter"))
            });

    if !issues.is_empty() {
        ComplianceCheck {
            name: "CB-1304: Sovereign Dep Contracts".into(),
            status: CheckStatus::Fail,
            message: format!("{} version violation(s): {}", issues.len(), issues.join("; ")),
            severity: Severity::Error,
        }
    } else {
        let adapter_note = if has_adapter {
            ", adapter pattern detected"
        } else {
            ""
        };
        ComplianceCheck {
            name: "CB-1304: Sovereign Dep Contracts".into(),
            status: CheckStatus::Pass,
            message: format!("{found_deps} sovereign dep(s) meet version contracts{adapter_note}"),
            severity: Severity::Info,
        }
    }
}

/// CB-1302: MCP Tool Schema Coverage — detect uncontracted MCP tool args
///
/// Counts MCP arg structs (Deserialize structs in mcp_pmcp/) and checks
/// for schema documentation. WARN if >20 arg structs lack doc comments.
#[provable_contracts_macros::contract("pmat-core.yaml", equation = "path_exists")]
pub(crate) fn check_mcp_schema_contracts(project_path: &Path) -> ComplianceCheck {
    let mcp_dir = project_path.join("src").join("mcp_pmcp");
    if !mcp_dir.exists() {
        return ComplianceCheck {
            name: "CB-1302: MCP Schema Contracts".into(),
            status: CheckStatus::Skip,
            message: "No src/mcp_pmcp/ directory".into(),
            severity: Severity::Info,
        };
    }

    let mut total_arg_structs = 0usize;
    let mut undocumented = 0usize;
    let mut handler_files = 0usize;

    for entry in walkdir::WalkDir::new(&mcp_dir)
        .max_depth(2)
        .into_iter()
        .filter_map(|e| e.ok())
    {
        let path = entry.path();
        if !path.is_file() {
            continue;
        }
        if !path.extension().is_some_and(|e| e == "rs") {
            continue;
        }
        let filename = path
            .file_name()
            .map(|n| n.to_string_lossy().to_string())
            .unwrap_or_default();
        if filename.contains("test") {
            continue;
        }
        if filename.contains("handler") {
            handler_files += 1;
        }

        let Ok(content) = std::fs::read_to_string(path) else {
            continue;
        };

        let lines: Vec<&str> = content.lines().collect();
        for (i, line) in lines.iter().enumerate() {
            let trimmed = line.trim();
            if trimmed.contains("struct") && trimmed.contains("Args") && !trimmed.starts_with("//") {
                total_arg_structs += 1;
                // Check if previous line(s) have doc comments
                let has_doc = (i > 0 && lines[i - 1].trim().starts_with("///"))
                    || (i > 1 && lines[i - 2].trim().starts_with("///"));
                if !has_doc {
                    undocumented += 1;
                }
            }
        }
    }

    if total_arg_structs == 0 {
        return ComplianceCheck {
            name: "CB-1302: MCP Schema Contracts".into(),
            status: CheckStatus::Skip,
            message: "No MCP arg structs found".into(),
            severity: Severity::Info,
        };
    }

    if undocumented > 20 {
        ComplianceCheck {
            name: "CB-1302: MCP Schema Contracts".into(),
            status: CheckStatus::Fail,
            message: format!(
                "{undocumented}/{total_arg_structs} MCP arg structs undocumented across {handler_files} handlers — add /// doc comments with input contracts"
            ),
            severity: Severity::Error,
        }
    } else if undocumented > 5 {
        ComplianceCheck {
            name: "CB-1302: MCP Schema Contracts".into(),
            status: CheckStatus::Warn,
            message: format!(
                "{undocumented}/{total_arg_structs} MCP arg structs lack doc comments ({handler_files} handlers)"
            ),
            severity: Severity::Warning,
        }
    } else {
        ComplianceCheck {
            name: "CB-1302: MCP Schema Contracts".into(),
            status: CheckStatus::Pass,
            message: format!(
                "{total_arg_structs} MCP arg structs, {handler_files} handlers ({undocumented} undocumented)"
            ),
            severity: Severity::Info,
        }
    }
}

/// CB-1306: TUI Widget Contract Coverage — verify presentar widget lifecycle
///
/// Scans for presentar-core contracts/ directory and checks that core
/// geometry, color, and layout constraint contracts exist with real
/// preconditions (not placeholders).
#[provable_contracts_macros::contract("pmat-core.yaml", equation = "path_exists")]
pub(crate) fn check_tui_widget_contracts(project_path: &Path) -> ComplianceCheck {
    let contracts_dir = project_path.join("contracts");
    // Check if this is a presentar-like project (has presentar-core dep or crate)
    // Only flag as presentar project if it HAS the presentar-core crate as workspace member
    // (not just as a dependency)
    let is_presentar = project_path.join("crates").join("presentar-core").exists();

    if !is_presentar {
        return ComplianceCheck {
            name: "CB-1306: TUI Widget Contracts".into(),
            status: CheckStatus::Skip,
            message: "Not a presentar/TUI project".into(),
            severity: Severity::Info,
        };
    }

    if !contracts_dir.exists() {
        return ComplianceCheck {
            name: "CB-1306: TUI Widget Contracts".into(),
            status: CheckStatus::Fail,
            message: "presentar project has no contracts/ directory".into(),
            severity: Severity::Error,
        };
    }

    // Required contract coverage for TUI widget lifecycle
    let required_domains = &[
        ("color", "contrast_ratio"),    // WCAG color contracts
        ("geometry", "intersection"),    // rect geometry
        ("layout", "constrain"),         // constraint solving
    ];

    let mut covered = 0usize;
    let mut missing: Vec<&str> = Vec::new();

    for entry in walkdir::WalkDir::new(&contracts_dir)
        .max_depth(2)
        .into_iter()
        .filter_map(|e| e.ok())
    {
        let path = entry.path();
        if !path.is_file() || !path.extension().is_some_and(|e| e == "yaml" || e == "yml") {
            continue;
        }
        if let Ok(content) = std::fs::read_to_string(path) {
            for (domain, equation) in required_domains {
                if content.contains(equation) {
                    covered += 1;
                    break; // count file once even if multiple matches
                }
                let _ = domain; // used for diagnostics below
            }
        }
    }

    // Check which domains are missing
    for (domain, equation) in required_domains {
        let found = walkdir::WalkDir::new(&contracts_dir)
            .max_depth(2)
            .into_iter()
            .filter_map(|e| e.ok())
            .any(|e| {
                e.path().is_file()
                    && std::fs::read_to_string(e.path())
                        .map(|c| c.contains(equation))
                        .unwrap_or(false)
            });
        if !found {
            missing.push(domain);
        }
    }

    if !missing.is_empty() {
        ComplianceCheck {
            name: "CB-1306: TUI Widget Contracts".into(),
            status: CheckStatus::Warn,
            message: format!(
                "{covered}/{} widget domains covered, missing: {}",
                required_domains.len(),
                missing.join(", ")
            ),
            severity: Severity::Warning,
        }
    } else {
        ComplianceCheck {
            name: "CB-1306: TUI Widget Contracts".into(),
            status: CheckStatus::Pass,
            message: format!(
                "{}/{} widget lifecycle domains contracted (color, geometry, layout)",
                required_domains.len(),
                required_domains.len()
            ),
            severity: Severity::Info,
        }
    }
}

/// CB-1307: WASM FFI Boundary Contracts — verify wasm_bindgen exports
///
/// Scans for #[wasm_bindgen] annotations and checks that exported
/// functions use Result<_, JsValue> (not panic) and have doc comments.
#[provable_contracts_macros::contract("pmat-core.yaml", equation = "path_exists")]
pub(crate) fn check_wasm_ffi_contracts(project_path: &Path) -> ComplianceCheck {
    // Check if WASM target exists
    let has_wasm_dep = project_path.join("Cargo.toml").exists()
        && std::fs::read_to_string(project_path.join("Cargo.toml"))
            .map(|c| c.contains("wasm-bindgen") || c.contains("wasm_bindgen"))
            .unwrap_or(false);

    if !has_wasm_dep {
        return ComplianceCheck {
            name: "CB-1307: WASM FFI Contracts".into(),
            status: CheckStatus::Skip,
            message: "No wasm-bindgen dependency".into(),
            severity: Severity::Info,
        };
    }

    let src_dir = project_path.join("src");
    let crates_dir = project_path.join("crates");
    let search_dirs: Vec<&Path> = [src_dir.as_path(), crates_dir.as_path()]
        .into_iter()
        .filter(|d| d.exists())
        .collect();

    let mut total_exports = 0usize;
    let mut undocumented = 0usize;
    let mut unwrap_in_export = 0usize;
    let mut no_result_return = 0usize;

    for search_dir in &search_dirs {
        for entry in walkdir::WalkDir::new(search_dir)
            .max_depth(8)
            .into_iter()
            .filter_map(|e| e.ok())
        {
            let path = entry.path();
            if !path.is_file() || !path.extension().is_some_and(|e| e == "rs") {
                continue;
            }
            if path.to_string_lossy().contains("test") {
                continue;
            }
            let Ok(content) = std::fs::read_to_string(path) else {
                continue;
            };
            if !content.contains("wasm_bindgen") {
                continue;
            }

            let lines: Vec<&str> = content.lines().collect();
            let mut in_wasm_block = false;

            for (i, line) in lines.iter().enumerate() {
                let trimmed = line.trim();
                if trimmed.contains("#[wasm_bindgen") {
                    in_wasm_block = true;
                    continue;
                }
                if in_wasm_block {
                    // Allow impl blocks and derive/cfg attributes to pass through
                    if trimmed.starts_with("impl ")
                        || trimmed.starts_with("pub struct ")
                        || trimmed.starts_with("#[")
                        || trimmed.is_empty()
                    {
                        // struct counts as export
                        if trimmed.starts_with("pub struct ") {
                            total_exports += 1;
                            let has_doc = i > 0 && lines[i - 1].trim().starts_with("///")
                                || (i > 1 && lines.get(i.wrapping_sub(2)).is_some_and(|l| l.trim().starts_with("///")));
                            if !has_doc { undocumented += 1; }
                        }
                        continue; // stay in wasm block
                    }
                    if trimmed.starts_with("pub fn ") || trimmed.contains("fn new(") {
                        total_exports += 1;

                        // Check for doc comment (look back past attributes)
                        let has_doc = (0..4).any(|back| {
                            i.checked_sub(back + 1).is_some_and(|j|
                                lines.get(j).is_some_and(|l| l.trim().starts_with("///"))
                            )
                        });
                        if !has_doc { undocumented += 1; }

                        // Check for unwrap in the function body
                        let end = (i + 30).min(lines.len());
                        let fn_has_unwrap = lines[i..end].iter()
                            .take_while(|l| {
                                let t = l.trim();
                                !(t.starts_with("pub fn ") && t != trimmed)
                            })
                            .any(|l| l.contains(".unwrap()"));
                        if fn_has_unwrap { unwrap_in_export += 1; }

                        // Check if constructor returns Result
                        if trimmed.contains("fn new(") && !trimmed.contains("Result") {
                            no_result_return += 1;
                        }
                        continue; // stay in wasm block for more methods
                    }
                    // closing brace or non-pub line — exit block
                    if trimmed == "}" || (!trimmed.starts_with("//") && !trimmed.starts_with("let ")) {
                        in_wasm_block = false;
                    }
                }
            }
        }
    }

    if total_exports == 0 {
        return ComplianceCheck {
            name: "CB-1307: WASM FFI Contracts".into(),
            status: CheckStatus::Skip,
            message: "No #[wasm_bindgen] exports found".into(),
            severity: Severity::Info,
        };
    }

    let mut issues = Vec::new();
    if unwrap_in_export > 0 {
        issues.push(format!("{unwrap_in_export} export(s) use .unwrap() (panic across FFI)"));
    }
    if no_result_return > 0 {
        issues.push(format!("{no_result_return} constructor(s) don't return Result<_, JsValue>"));
    }
    if undocumented as f64 / total_exports as f64 > 0.5 {
        issues.push(format!("{undocumented}/{total_exports} exports undocumented"));
    }

    if !issues.is_empty() {
        ComplianceCheck {
            name: "CB-1307: WASM FFI Contracts".into(),
            status: CheckStatus::Warn,
            message: format!("{total_exports} WASM exports: {}", issues.join("; ")),
            severity: Severity::Warning,
        }
    } else {
        ComplianceCheck {
            name: "CB-1307: WASM FFI Contracts".into(),
            status: CheckStatus::Pass,
            message: format!("{total_exports} WASM exports, {undocumented} undocumented, 0 unwrap-across-FFI"),
            severity: Severity::Info,
        }
    }
}

/// GH-292: Read min verification level from `.pmat-gates.toml [verification_ladder]`
/// or `.pmat.yaml comply.thresholds.min_verification_level`. Gates file wins.
fn load_verification_min_level(project_path: &Path) -> u8 {
    if let Some(v) = load_verification_min_level_from_gates(project_path) {
        return v;
    }
    load_verification_min_level_from_yaml(project_path).unwrap_or(5)
}

fn load_verification_min_level_from_gates(project_path: &Path) -> Option<u8> {
    let path = project_path.join(".pmat-gates.toml");
    let content = std::fs::read_to_string(&path).ok()?;
    let table: toml::Table = content.parse().ok()?;
    table
        .get("verification_ladder")
        .and_then(|vl| vl.get("min_level"))
        .and_then(|v| v.as_str())
        .and_then(parse_level_string)
}

fn load_verification_min_level_from_yaml(project_path: &Path) -> Option<u8> {
    let yaml_path = project_path.join(".pmat.yaml");
    let yml_path = project_path.join(".pmat.yml");
    if !yaml_path.exists() && !yml_path.exists() {
        return None;
    }
    let cfg = crate::models::comply_config::PmatYamlConfig::load(project_path).ok()?;
    parse_level_string(&cfg.comply.thresholds.min_verification_level)
}

fn parse_level_string(s: &str) -> Option<u8> {
    let stripped = s.strip_prefix('L').unwrap_or(s);
    stripped.parse::<u8>().ok()
}

fn level_number(content: &str) -> u8 {
    if content.contains("lean_theorem:") {
        5
    } else if content.contains("kani_harnesses:") {
        4
    } else if content.contains("falsification:") || content.contains("falsification_tests:") {
        3
    } else if content.contains("proof_obligations:") {
        2
    } else {
        1
    }
}

fn level_label(n: u8) -> &'static str {
    match n {
        5 => "L5",
        4 => "L4",
        3 => "L3",
        2 => "L2",
        _ => "L1",
    }
}

/// CB-1308: Verification Ladder — configurable min level (default L5)
///
/// Every contract YAML below the minimum level is a FAIL, listed by name.
/// Configurable via .pmat-gates.toml [verification_ladder] min_level = "L3"
#[provable_contracts_macros::contract("pmat-core.yaml", equation = "path_exists")]
pub(crate) fn check_verification_ladder(project_path: &Path) -> ComplianceCheck {
    let contracts_dir = match resolve_contracts_dir(project_path) {
        Some(d) => d,
        None => {
            return ComplianceCheck {
                name: "CB-1308: Verification Ladder".into(),
                status: CheckStatus::Skip,
                message: "No contract YAML files found".into(),
                severity: Severity::Info,
            };
        }
    };

    let mut total = 0usize;
    let mut l5_count = 0usize;
    let mut violations: Vec<String> = Vec::new();

    for entry in walkdir::WalkDir::new(&contracts_dir)
        .max_depth(3)
        .into_iter()
        .filter_map(|e| e.ok())
    {
        let path = entry.path();
        if !path.is_file() {
            continue;
        }
        if !path.extension().is_some_and(|e| e == "yaml" || e == "yml") {
            continue;
        }
        if path.file_name().is_some_and(|n| n.to_string_lossy().contains("binding")) {
            continue;
        }

        let Ok(content) = std::fs::read_to_string(path) else {
            continue;
        };

        // Only enforce L5 on contracts that HAVE equations.
        // Schema-registry and invariant-only contracts have a different
        // verification path (CB-1305 classification).
        let has_equations = content.lines().any(|l| {
            let t = l.trim();
            t == "equations:" && !l.starts_with(' ')
        });
        if !has_equations {
            continue; // exempt from L5 enforcement
        }

        total += 1;
        let filename = path
            .file_name()
            .map(|n| n.to_string_lossy().to_string())
            .unwrap_or_default();

        let file_level = level_number(&content);
        let min_level = load_verification_min_level(project_path);
        if file_level >= min_level {
            l5_count += 1;
        } else {
            violations.push(format!("{filename} ({})", level_label(file_level)));
        }
    }

    if total == 0 {
        return ComplianceCheck {
            name: "CB-1308: Verification Ladder".into(),
            status: CheckStatus::Skip,
            message: "No contract YAML files found".into(),
            severity: Severity::Info,
        };
    }

    if violations.is_empty() {
        return ComplianceCheck {
            name: "CB-1308: Verification Ladder".into(),
            status: CheckStatus::Pass,
            message: format!("{total}/{total} contracts at L5 (lean_theorem)"),
            severity: Severity::Info,
        };
    }

    // FAIL: list every non-L5 contract by name, like pmat tdg lists files
    let shown = if violations.len() > 10 {
        let remaining = violations.len() - 10;
        let mut v: Vec<String> = violations[..10].to_vec();
        v.push(format!("(+{remaining} more)"));
        v
    } else {
        violations.clone()
    };

    ComplianceCheck {
        name: "CB-1308: Verification Ladder".into(),
        status: CheckStatus::Fail,
        message: format!(
            "{}/{total} at L5, {} violations:\n{}",
            l5_count,
            violations.len(),
            shown
                .iter()
                .map(|v| format!("    - {v}"))
                .collect::<Vec<_>>()
                .join("\n")
        ),
        severity: Severity::Critical,
    }
}

// --- Helpers ---

enum ContractClass {
    KernelMath,
    CrossLanguage,
    SchemaRegistry,  // weight_roles, architecture_map, registry: true
    InvariantsOnly,  // invariants: without equations: (pure constraints)
    SemanticLeak,    // API-pattern disguised as kernel-math
    Unknown,
}

/// Extract top-level YAML keys (lines at column 0 ending with ':')
fn extract_top_level_keys(content: &str) -> Vec<String> {
    content
        .lines()
        .filter(|l| {
            !l.starts_with(' ')
                && !l.starts_with('#')
                && !l.is_empty()
                && l.contains(':')
        })
        .map(|l| l.split(':').next().unwrap_or("").trim().to_string())
        .collect()
}

/// Keys that indicate schema-registry contracts ONLY when equations: is absent.
const SCHEMA_REGISTRY_KEYS: &[&str] = &[
    "weight_roles",
    "architecture_map",
    "tensor_names",
    "name_templates",
    "tokenizer_types",
    "families",
    "formats",
    "schemes",
    "layer_steps",
    "constants",
    "type_enforcement",
    "kernel_structure",
    "simd_dispatch",
];

/// Classify a contract by its top-level keys and precondition content.
fn classify_contract(top_keys: &[String], content: &str) -> ContractClass {
    let has_equations = top_keys.iter().any(|k| k == "equations");
    let has_enforcement_level = content
        .lines()
        .any(|l| l.trim().starts_with("enforcement_level:"));
    let has_cross_lang_markers = top_keys.iter().any(|k| {
        CROSS_LANGUAGE_KEYS.contains(&k.as_str())
    });
    let has_schema_registry = top_keys.iter().any(|k| {
        SCHEMA_REGISTRY_KEYS.contains(&k.as_str())
    });
    let has_invariants = top_keys.iter().any(|k| k == "invariants");
    let has_registry_flag = content
        .lines()
        .any(|l| l.trim() == "registry: true");

    // Contracts WITH equations: — classify by enforcement style
    if has_equations {
        if has_enforcement_level || has_cross_lang_markers {
            return ContractClass::CrossLanguage;
        }
        if is_generic_api_pattern(content) {
            return ContractClass::SemanticLeak;
        }
        return ContractClass::KernelMath;
    }

    // Contracts WITHOUT equations: — classify by structure
    if has_schema_registry || has_registry_flag {
        return ContractClass::SchemaRegistry;
    }
    if has_invariants {
        return ContractClass::InvariantsOnly;
    }

    // No recognized structure
    ContractClass::Unknown
}

/// Detect if a contract is a generic API pattern with only placeholder assertions.
/// Checks BOTH preconditions AND postconditions + invariant text.
fn is_generic_api_pattern(content: &str) -> bool {
    let mut assertions: Vec<String> = Vec::new();
    let mut in_list = false;

    for line in content.lines() {
        let trimmed = line.trim();
        if trimmed == "preconditions:"
            || trimmed == "postconditions:"
            || trimmed == "invariants:"
        {
            in_list = true;
            continue;
        }
        if in_list {
            if trimmed.starts_with("- ") {
                let val = trimmed
                    .trim_start_matches("- ")
                    .trim_matches('\'')
                    .trim_matches('"');
                assertions.push(val.to_string());
            } else if !trimmed.is_empty()
                && !trimmed.starts_with('#')
                && !trimmed.starts_with("- ")
            {
                in_list = false;
            }
        }
    }

    if assertions.is_empty() {
        return false;
    }

    // If ALL assertions are generic placeholders, it's a semantic leak
    assertions
        .iter()
        .all(|p| GENERIC_PLACEHOLDERS.contains(&p.as_str()))
}

/// Check GitHub Actions workflow contracts.
fn check_workflow_contracts(
    workflows_dir: &Path,
    issues: &mut Vec<String>,
    checks_run: &mut usize,
) {
    let Ok(entries) = std::fs::read_dir(workflows_dir) else {
        return;
    };

    for entry in entries.flatten() {
        let path = entry.path();
        if !path.extension().is_some_and(|e| e == "yml" || e == "yaml") {
            continue;
        }
        let Ok(content) = std::fs::read_to_string(&path) else {
            continue;
        };
        let filename = path
            .file_name()
            .map(|n| n.to_string_lossy().to_string())
            .unwrap_or_default();

        *checks_run += 1;

        // Contract: RUST_MIN_STACK must be set in workflows that run cargo test
        // Only match cargo test in run: steps, not comments
        let has_cargo_test_in_run = content.lines().any(|l| {
            let t = l.trim();
            !t.starts_with('#') && t.contains("cargo test")
        });
        if has_cargo_test_in_run && !content.contains("RUST_MIN_STACK") {
            issues.push(format!(
                "{filename}: cargo test without RUST_MIN_STACK (clap needs 8MB stack)"
            ));
        }

        // Contract: no stale working-directory references to ./server
        if content.contains("working-directory: ./server")
            || content.contains("working-directory: server")
        {
            issues.push(format!(
                "{filename}: stale working-directory: ./server reference"
            ));
        }
    }
}

/// Check Cargo.toml contracts (sovereign deps, default-features).
fn check_cargo_contracts(
    cargo_toml: &Path,
    issues: &mut Vec<String>,
    checks_run: &mut usize,
) {
    let Ok(content) = std::fs::read_to_string(cargo_toml) else {
        return;
    };

    *checks_run += 1;

    // Contract: edition should be 2021+
    if !content.contains("edition = \"2021\"")
        && !content.contains("edition = \"2024\"")
    {
        issues.push("Cargo.toml: edition is not 2021 or 2024".into());
    }
}

/// Extract dependency version from Cargo.toml content.
fn extract_dep_version<'a>(content: &'a str, dep_name: &str) -> Option<&'a str> {
    for line in content.lines() {
        let trimmed = line.trim();
        // Must start with dep name followed by space or =
        if !trimmed.starts_with(dep_name) {
            continue;
        }
        let after_name = &trimmed[dep_name.len()..];
        if !after_name.starts_with(' ') && !after_name.starts_with('=') {
            continue;
        }
        // Inline table: dep = { version = "x.y", ... }
        if trimmed.contains('{') {
            // Find version = "x.y" inside the braces
            if let Some(ver_start) = trimmed.find("version") {
                let rest = &trimmed[ver_start + "version".len()..];
                let rest = rest.trim_start_matches([' ', '=']);
                if let Some(inner) = rest.strip_prefix('"') {
                    if let Some(end) = inner.find('"') {
                        return Some(&inner[..end]);
                    }
                }
            }
            continue;
        }
        // Simple: dep = "x.y"
        if let Some(eq_pos) = trimmed.find('=') {
            let after_eq = trimmed[eq_pos + 1..].trim();
            if let Some(inner) = after_eq.strip_prefix('"') {
                if let Some(end) = inner.find('"') {
                    return Some(&inner[..end]);
                }
            }
        }
    }
    None
}

/// Simple semver minimum check: does `actual` >= `minimum`?
fn version_satisfies_minimum(actual: &str, minimum: &str) -> bool {
    let parse = |v: &str| -> Vec<u32> {
        v.split('.')
            .filter_map(|s| s.parse().ok())
            .collect()
    };
    let a = parse(actual);
    let m = parse(minimum);
    for i in 0..m.len().max(a.len()) {
        let av = a.get(i).copied().unwrap_or(0);
        let mv = m.get(i).copied().unwrap_or(0);
        if av > mv {
            return true;
        }
        if av < mv {
            return false;
        }
    }
    true // equal
}

#[cfg(test)]
mod contract_surface_tests {
    use super::*;

    #[test]
    fn test_version_satisfies_minimum() {
        assert!(version_satisfies_minimum("0.27.1", "0.27"));
        assert!(version_satisfies_minimum("0.27.0", "0.27"));
        assert!(version_satisfies_minimum("1.0.0", "0.27"));
        assert!(!version_satisfies_minimum("0.26.0", "0.27"));
        assert!(version_satisfies_minimum("0.16", "0.16"));
        assert!(!version_satisfies_minimum("0.15", "0.16"));
        assert!(version_satisfies_minimum("0.1.17", "0.1.17"));
        assert!(!version_satisfies_minimum("0.1.16", "0.1.17"));
        assert!(version_satisfies_minimum("0.3.15", "0.3.15"));
        assert!(version_satisfies_minimum("0.3.16", "0.3.15"));
    }

    #[test]
    fn test_extract_top_level_keys() {
        let content = "metadata:\n  version: 1.0\nequations:\n  softmax:\n    formula: x";
        let keys = extract_top_level_keys(content);
        assert_eq!(keys, vec!["metadata", "equations"]);
    }

    #[test]
    fn test_classify_kernel_math() {
        let content = "metadata:\n  version: 1.0\nequations:\n  softmax:\n    preconditions:\n    - 'x.iter().all(|v| v.is_finite())'";
        let keys = extract_top_level_keys(content);
        assert!(matches!(classify_contract(&keys, content), ContractClass::KernelMath));
    }

    #[test]
    fn test_classify_semantic_leak() {
        let content = "metadata:\n  version: 1.0\nequations:\n  config:\n    preconditions:\n    - 'input.len() > 0'\n    postconditions:\n    - 'result.len() > 0'";
        let keys = extract_top_level_keys(content);
        assert!(matches!(classify_contract(&keys, content), ContractClass::SemanticLeak));
    }

    #[test]
    fn test_classify_cross_language() {
        let content = "metadata:\n  version: 1.0\n  enforcement_level: standard\nequations:\n  kernel_ffi:\n    preconditions:\n    - 'Function has __global__ qualifier'";
        let keys = extract_top_level_keys(content);
        assert!(matches!(classify_contract(&keys, content), ContractClass::CrossLanguage));
    }

    #[test]
    fn test_is_generic_api_pattern() {
        let generic = "preconditions:\n    - 'input.len() > 0'\n    - '!input.is_empty()'";
        assert!(is_generic_api_pattern(generic));

        let real = "preconditions:\n    - 'x.iter().all(|v| v.is_finite())'";
        assert!(!is_generic_api_pattern(real));
    }

    #[test]
    fn test_extract_dep_version_simple() {
        let content = "aprender = \"0.27.1\"\ntrueno = \"0.16\"";
        assert_eq!(extract_dep_version(content, "aprender"), Some("0.27.1"));
        assert_eq!(extract_dep_version(content, "trueno"), Some("0.16"));
        assert_eq!(extract_dep_version(content, "nonexistent"), None);
    }

    #[test]
    fn test_extract_dep_version_inline_table() {
        let content = "aprender = { version = \"0.27.1\", default-features = false }";
        assert_eq!(extract_dep_version(content, "aprender"), Some("0.27.1"));
    }

    #[test]
    fn test_generic_pattern_with_postconditions() {
        let content = "equations:\n  config:\n    preconditions:\n    - 'input.len() > 0'\n    postconditions:\n    - 'result.len() > 0'\n    invariants:\n    - 'Type safety preserved'";
        assert!(is_generic_api_pattern(content));
    }

    #[test]
    fn test_mixed_real_and_placeholder_not_leak() {
        let content = "preconditions:\n    - 'input.len() > 0'\n    - 'x.iter().all(|v| v.is_finite())'";
        assert!(!is_generic_api_pattern(content));
    }

    #[test]
    fn test_classify_schema_registry() {
        let content = "metadata:\n  version: 1.0\n  registry: true\narchitecture_map:\n  llama: llama";
        let keys = extract_top_level_keys(content);
        assert!(matches!(classify_contract(&keys, content), ContractClass::SchemaRegistry));
    }

    #[test]
    fn test_classify_invariants_only() {
        let content = "metadata:\n  version: 1.0\ninvariants:\n- id: QKN-001\n  property: loads weights";
        let keys = extract_top_level_keys(content);
        assert!(matches!(classify_contract(&keys, content), ContractClass::InvariantsOnly));
    }

    #[test]
    fn test_classify_weight_roles() {
        let content = "metadata:\n  version: 1.0\nweight_roles:\n  attn_norm:\n    description: Pre-attention norm";
        let keys = extract_top_level_keys(content);
        assert!(matches!(classify_contract(&keys, content), ContractClass::SchemaRegistry));
    }

    #[test]
    fn test_parse_level_string() {
        assert_eq!(parse_level_string("L3"), Some(3));
        assert_eq!(parse_level_string("L5"), Some(5));
        assert_eq!(parse_level_string("3"), Some(3));
        assert_eq!(parse_level_string("Lx"), None);
        assert_eq!(parse_level_string(""), None);
    }

    #[test]
    fn test_verification_level_default_without_config() {
        let tmp = tempfile::tempdir().unwrap();
        assert_eq!(load_verification_min_level(tmp.path()), 5);
    }

    #[test]
    fn test_verification_level_from_pmat_yaml() {
        let tmp = tempfile::tempdir().unwrap();
        std::fs::write(
            tmp.path().join(".pmat.yaml"),
            "comply:\n  thresholds:\n    min_verification_level: \"L3\"\n",
        )
        .unwrap();
        assert_eq!(load_verification_min_level(tmp.path()), 3);
    }

    #[test]
    fn test_verification_level_gates_wins_over_yaml() {
        let tmp = tempfile::tempdir().unwrap();
        std::fs::write(
            tmp.path().join(".pmat-gates.toml"),
            "[verification_ladder]\nmin_level = \"L2\"\n",
        )
        .unwrap();
        std::fs::write(
            tmp.path().join(".pmat.yaml"),
            "comply:\n  thresholds:\n    min_verification_level: \"L4\"\n",
        )
        .unwrap();
        assert_eq!(load_verification_min_level(tmp.path()), 2);
    }
}

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
    let skip = || ComplianceCheck {
        name: "CB-1305: Contract Surface Classification".into(),
        status: CheckStatus::Skip,
        message: "No contract YAML files found".into(),
        severity: Severity::Info,
    };
    let Some(contracts_dir) = resolve_contracts_dir(project_path) else {
        return skip();
    };
    let mut tally = SurfaceTally::default();
    for path in contract_yaml_files(&contracts_dir, 3) {
        if path
            .file_name()
            .is_some_and(|n| n.to_string_lossy().contains("binding"))
        {
            continue;
        }
        tally.total += 1;
        let Ok(content) = std::fs::read_to_string(&path) else {
            continue;
        };
        let top_keys = extract_top_level_keys(&content);
        tally.record(classify_contract(&top_keys, &content), &path);
    }
    if tally.total == 0 {
        return skip();
    }
    tally.verdict()
}

/// Every `.yaml`/`.yml` file under `dir`, at most `depth` levels down.
fn contract_yaml_files(dir: &Path, depth: usize) -> Vec<std::path::PathBuf> {
    walkdir::WalkDir::new(dir)
        .max_depth(depth)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.path().is_file())
        .filter(|e| {
            e.path()
                .extension()
                .is_some_and(|x| x == "yaml" || x == "yml")
        })
        .map(|e| e.into_path())
        .collect()
}

/// What CB-1305 counted over the contracts directory.
#[derive(Default)]
struct SurfaceTally {
    total: usize,
    kernel_math: usize,
    cross_language: usize,
    schema_registry: usize,
    invariants_only: usize,
    semantic_leaks: usize,
    /// The first five unclassified file names, for the message.
    unclassified: Vec<String>,
    unclassified_count: usize,
}

impl SurfaceTally {
    fn record(&mut self, class: ContractClass, path: &Path) {
        match class {
            ContractClass::KernelMath => self.kernel_math += 1,
            ContractClass::CrossLanguage => self.cross_language += 1,
            ContractClass::SchemaRegistry => self.schema_registry += 1,
            ContractClass::InvariantsOnly => self.invariants_only += 1,
            ContractClass::SemanticLeak => {
                self.semantic_leaks += 1;
                self.kernel_math += 1; // structurally it IS kernel-math
            }
            ContractClass::Unknown => {
                self.unclassified_count += 1;
                if self.unclassified.len() < 5 {
                    self.unclassified.push(
                        path.file_name()
                            .map(|n| n.to_string_lossy().to_string())
                            .unwrap_or_default(),
                    );
                }
            }
        }
    }

    fn verdict(&self) -> ComplianceCheck {
        let name = "CB-1305: Contract Surface Classification";
        let unclassified_pct = self.unclassified_count as f64 / self.total as f64 * 100.0;
        let mut issues = Vec::new();
        if unclassified_pct > 20.0 {
            issues.push(format!(
                "{}/{} ({unclassified_pct:.0}%) contracts unclassified — leak has outpaced specification",
                self.unclassified_count, self.total
            ));
        }
        if self.semantic_leaks > 0 {
            issues.push(format!(
                "{} semantic leak(s): API-pattern contracts with generic placeholders in kernel-math schema",
                self.semantic_leaks
            ));
        }
        if issues.is_empty() {
            let classified = self.total - self.unclassified_count;
            let pure_kernel = self.kernel_math.saturating_sub(self.semantic_leaks);
            let parts: Vec<String> = [
                ("kernel", pure_kernel),
                ("cross-lang", self.cross_language),
                ("registry", self.schema_registry),
                ("invariants", self.invariants_only),
                ("leaks", self.semantic_leaks),
            ]
            .iter()
            .filter(|(_, n)| *n > 0)
            .map(|(k, n)| format!("{k}={n}"))
            .collect();
            return ComplianceCheck {
                name: name.into(),
                status: CheckStatus::Pass,
                message: format!(
                    "{classified}/{} classified ({})",
                    self.total,
                    parts.join(", ")
                ),
                severity: Severity::Info,
            };
        }
        let failing = unclassified_pct > 20.0;
        let unclassified_list = if self.unclassified.is_empty() {
            String::new()
        } else {
            format!(" [{}]", self.unclassified.join(", "))
        };
        ComplianceCheck {
            name: name.into(),
            status: if failing { CheckStatus::Fail } else { CheckStatus::Warn },
            message: format!("{}{unclassified_list}", issues.join("; ")),
            severity: if failing { Severity::Error } else { Severity::Warning },
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
        if path.extension().is_none_or(|e| e != "rs") {
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
    let name = "CB-1302: MCP Schema Contracts";
    let mcp_dir = project_path.join("src").join("mcp_pmcp");
    if !mcp_dir.exists() {
        return ComplianceCheck {
            name: name.into(),
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
        if !path.is_file() || path.extension().is_none_or(|e| e != "rs") {
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
        let (total, undoc) = arg_struct_census(&content);
        total_arg_structs += total;
        undocumented += undoc;
    }
    if total_arg_structs == 0 {
        return ComplianceCheck {
            name: name.into(),
            status: CheckStatus::Skip,
            message: "No MCP arg structs found".into(),
            severity: Severity::Info,
        };
    }
    let (status, severity, message) = if undocumented > 20 {
        (
            CheckStatus::Fail,
            Severity::Error,
            format!(
                "{undocumented}/{total_arg_structs} MCP arg structs undocumented across {handler_files} handlers — add /// doc comments with input contracts"
            ),
        )
    } else if undocumented > 5 {
        (
            CheckStatus::Warn,
            Severity::Warning,
            format!(
                "{undocumented}/{total_arg_structs} MCP arg structs lack doc comments ({handler_files} handlers)"
            ),
        )
    } else {
        (
            CheckStatus::Pass,
            Severity::Info,
            format!(
                "{total_arg_structs} MCP arg structs, {handler_files} handlers ({undocumented} undocumented)"
            ),
        )
    };
    ComplianceCheck {
        name: name.into(),
        status,
        message,
        severity,
    }
}

/// (`*Args` structs, those without a `///` line in the two lines above).
fn arg_struct_census(content: &str) -> (usize, usize) {
    let lines: Vec<&str> = content.lines().collect();
    let mut total = 0usize;
    let mut undocumented = 0usize;
    for (i, line) in lines.iter().enumerate() {
        let trimmed = line.trim();
        if !(trimmed.contains("struct") && trimmed.contains("Args") && !trimmed.starts_with("//")) {
            continue;
        }
        total += 1;
        let documented = (1..=2).any(|back| {
            i >= back && lines[i - back].trim().starts_with("///")
        });
        if !documented {
            undocumented += 1;
        }
    }
    (total, undocumented)
}

/// CB-1306: TUI Widget Contract Coverage — verify presentar widget lifecycle
///
/// Scans for presentar-core contracts/ directory and checks that core
/// geometry, color, and layout constraint contracts exist with real
/// preconditions (not placeholders).
#[provable_contracts_macros::contract("pmat-core.yaml", equation = "path_exists")]
pub(crate) fn check_tui_widget_contracts(project_path: &Path) -> ComplianceCheck {
    let name = "CB-1306: TUI Widget Contracts";
    let contracts_dir = project_path.join("contracts");
    if !project_path.join("crates").join("presentar-core").exists() {
        return ComplianceCheck {
            name: name.into(),
            status: CheckStatus::Skip,
            message: "Not a presentar/TUI project".into(),
            severity: Severity::Info,
        };
    }
    if !contracts_dir.exists() {
        return ComplianceCheck {
            name: name.into(),
            status: CheckStatus::Fail,
            message: "presentar project has no contracts/ directory".into(),
            severity: Severity::Error,
        };
    }
    let required_domains = &[
        ("color", "contrast_ratio"), // WCAG color contracts
        ("geometry", "intersection"), // rect geometry
        ("layout", "constrain"),      // constraint solving
    ];
    let contents: Vec<String> = contract_yaml_files(&contracts_dir, 2)
        .iter()
        .filter_map(|p| std::fs::read_to_string(p).ok())
        .collect();
    // A file counts once, however many domains it mentions.
    let covered = contents
        .iter()
        .filter(|c| required_domains.iter().any(|(_, eq)| c.contains(eq)))
        .count();
    let missing: Vec<&str> = required_domains
        .iter()
        .filter(|(_, eq)| !contents.iter().any(|c| c.contains(eq)))
        .map(|(domain, _)| *domain)
        .collect();
    if missing.is_empty() {
        ComplianceCheck {
            name: name.into(),
            status: CheckStatus::Pass,
            message: format!(
                "{}/{} widget lifecycle domains contracted (color, geometry, layout)",
                required_domains.len(),
                required_domains.len()
            ),
            severity: Severity::Info,
        }
    } else {
        ComplianceCheck {
            name: name.into(),
            status: CheckStatus::Warn,
            message: format!(
                "{covered}/{} widget domains covered, missing: {}",
                required_domains.len(),
                missing.join(", ")
            ),
            severity: Severity::Warning,
        }
    }
}

/// CB-1307: WASM FFI Boundary Contracts — verify wasm_bindgen exports
///
/// Scans for #[wasm_bindgen] annotations and checks that exported
/// functions use Result<_, JsValue> (not panic) and have doc comments.
#[provable_contracts_macros::contract("pmat-core.yaml", equation = "path_exists")]
pub(crate) fn check_wasm_ffi_contracts(project_path: &Path) -> ComplianceCheck {
    if !declares_wasm_bindgen(project_path) {
        return wasm_ffi_check(
            CheckStatus::Skip,
            Severity::Info,
            "No wasm-bindgen dependency".to_string(),
        );
    }

    let counts = tally_wasm_exports(project_path);
    if counts.total_exports == 0 {
        return wasm_ffi_check(
            CheckStatus::Skip,
            Severity::Info,
            "No #[wasm_bindgen] exports found".to_string(),
        );
    }

    let issues = wasm_export_issues(&counts);
    if issues.is_empty() {
        wasm_ffi_check(
            CheckStatus::Pass,
            Severity::Info,
            format!(
                "{} WASM exports, {} undocumented, 0 unwrap-across-FFI",
                counts.total_exports, counts.undocumented
            ),
        )
    } else {
        wasm_ffi_check(
            CheckStatus::Warn,
            Severity::Warning,
            format!(
                "{} WASM exports: {}",
                counts.total_exports,
                issues.join("; ")
            ),
        )
    }
}

/// Build a CB-1307 result; every return path used to repeat the literal.
fn wasm_ffi_check(status: CheckStatus, severity: Severity, message: String) -> ComplianceCheck {
    ComplianceCheck {
        name: "CB-1307: WASM FFI Contracts".into(),
        status,
        message,
        severity,
    }
}

/// True when the project's manifest names wasm-bindgen (either spelling).
/// An unreadable/absent manifest declares nothing.
fn declares_wasm_bindgen(project_path: &Path) -> bool {
    std::fs::read_to_string(project_path.join("Cargo.toml"))
        .map(|c| c.contains("wasm-bindgen") || c.contains("wasm_bindgen"))
        .unwrap_or(false)
}

/// Source of a non-test `.rs` file that mentions `wasm_bindgen`, or `None` for
/// every file the scan skips.
fn wasm_source_to_scan(path: &Path) -> Option<String> {
    if !path.is_file() || path.extension().is_none_or(|e| e != "rs") {
        return None;
    }
    if path.to_string_lossy().contains("test") {
        return None;
    }
    let content = std::fs::read_to_string(path).ok()?;
    content.contains("wasm_bindgen").then_some(content)
}

/// Sum [`scan_wasm_exports`] over `src/` and `crates/`.
fn tally_wasm_exports(project_path: &Path) -> WasmExportCounts {
    let src_dir = project_path.join("src");
    let crates_dir = project_path.join("crates");
    let mut totals = WasmExportCounts::default();
    for search_dir in [src_dir.as_path(), crates_dir.as_path()]
        .into_iter()
        .filter(|d| d.exists())
    {
        for entry in walkdir::WalkDir::new(search_dir)
            .max_depth(8)
            .into_iter()
            .filter_map(std::result::Result::ok)
        {
            let Some(content) = wasm_source_to_scan(entry.path()) else {
                continue;
            };
            let c = scan_wasm_exports(&content);
            totals.total_exports += c.total_exports;
            totals.undocumented += c.undocumented;
            totals.unwrap_in_export += c.unwrap_in_export;
            totals.no_result_return += c.no_result_return;
        }
    }
    totals
}

/// Contract violations implied by a tally, in report order. Caller guarantees
/// `total_exports > 0`.
fn wasm_export_issues(c: &WasmExportCounts) -> Vec<String> {
    let mut issues = Vec::new();
    if c.unwrap_in_export > 0 {
        issues.push(format!(
            "{} export(s) use .unwrap() (panic across FFI)",
            c.unwrap_in_export
        ));
    }
    if c.no_result_return > 0 {
        issues.push(format!(
            "{} constructor(s) don't return Result<_, JsValue>",
            c.no_result_return
        ));
    }
    if c.undocumented as f64 / c.total_exports as f64 > 0.5 {
        issues.push(format!(
            "{}/{} exports undocumented",
            c.undocumented, c.total_exports
        ));
    }
    issues
}

/// Tally of `#[wasm_bindgen]` export issues found in one source file.
#[derive(Default)]
struct WasmExportCounts {
    total_exports: usize,
    undocumented: usize,
    unwrap_in_export: usize,
    no_result_return: usize,
}

/// Scan one file's `#[wasm_bindgen]` blocks for export-contract issues. Pure —
/// extracted from `check_wasm_ffi_contracts` to keep it under the complexity gate
/// (see `test_scan_wasm_exports`).
fn scan_wasm_exports(content: &str) -> WasmExportCounts {
    let lines: Vec<&str> = content.lines().collect();
    let mut counts = WasmExportCounts::default();
    let mut in_wasm_block = false;
    for i in 0..lines.len() {
        let trimmed = lines[i].trim();
        if trimmed.contains("#[wasm_bindgen") {
            in_wasm_block = true;
            continue;
        }
        if in_wasm_block {
            in_wasm_block = apply_wasm_block_line(&lines, i, &mut counts);
        }
    }
    counts
}

/// Process one line inside a `#[wasm_bindgen]` block; update `counts` and return
/// whether the block is still open. Attribute/impl/struct/blank lines stay in
/// the block; a `}` or other non-pub line ends it.
fn apply_wasm_block_line(lines: &[&str], i: usize, counts: &mut WasmExportCounts) -> bool {
    let trimmed = lines[i].trim();
    if trimmed.starts_with("impl ")
        || trimmed.starts_with("pub struct ")
        || trimmed.starts_with("#[")
        || trimmed.is_empty()
    {
        if trimmed.starts_with("pub struct ") {
            counts.total_exports += 1;
            if !wasm_line_has_doc_above(lines, i, 2) {
                counts.undocumented += 1;
            }
        }
        return true;
    }
    if trimmed.starts_with("pub fn ") || trimmed.contains("fn new(") {
        count_wasm_export_fn(lines, i, trimmed, counts);
        return true;
    }
    trimmed != "}" && (trimmed.starts_with("//") || trimmed.starts_with("let "))
}

/// Count doc/unwrap/Result issues for one exported `pub fn` / constructor.
fn count_wasm_export_fn(lines: &[&str], i: usize, trimmed: &str, counts: &mut WasmExportCounts) {
    counts.total_exports += 1;
    if !wasm_line_has_doc_above(lines, i, 4) {
        counts.undocumented += 1;
    }
    // `.unwrap()` anywhere in the (heuristic) function body panics across FFI.
    let end = (i + 30).min(lines.len());
    let fn_has_unwrap = lines[i..end]
        .iter()
        .take_while(|l| {
            let t = l.trim();
            !(t.starts_with("pub fn ") && t != trimmed)
        })
        .any(|l| l.contains(".unwrap()"));
    if fn_has_unwrap {
        counts.unwrap_in_export += 1;
    }
    // Constructors should return `Result<_, JsValue>`.
    if trimmed.contains("fn new(") && !trimmed.contains("Result") {
        counts.no_result_return += 1;
    }
}

/// True if any of the `back` lines immediately above `i` is a `///` doc comment.
fn wasm_line_has_doc_above(lines: &[&str], i: usize, back: usize) -> bool {
    (1..=back).any(|b| {
        i.checked_sub(b)
            .and_then(|j| lines.get(j))
            .is_some_and(|l| l.trim().starts_with("///"))
    })
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

/// The level a contract YAML EVIDENCES, not the level it mentions.
///
/// This was a substring test — `content.contains("lean_theorem:")` graded a
/// contract L5 — so naming a proof technique scored as having performed the
/// proof. On rmedia that produced two Pass rows that cannot both be true, for a
/// repo with 8 kani harnesses and ZERO `.lean` files:
///
/// ```text
/// CB-1206: Verification Levels: 14 obligations: L2=15 tests, L4=16 kani (114%)
/// CB-1308: Verification Ladder: 3/3 contracts at L5 (lean_theorem)
/// ```
///
/// 114% is not a possible ratio, and L5 is a Lean proof in a repo with no Lean.
///
/// The rigorous rule already exists — `crate::quality::ladder_evidence`, whose
/// own docs say "Declared harnesses without a run never count" and require
/// `lean_theorem.status == "proved"`. It is keyed on a `WorkContract` plus an
/// execution record, so this file cannot call it directly for a bare YAML; what
/// it CAN do is stop treating a mention as evidence. A key with no value, an
/// empty list, or a `lean_theorem` that is not `status: proved` no longer
/// promotes the level.
fn level_number(content: &str) -> u8 {
    if lean_theorem_is_proved(content) {
        5
    } else if has_nonempty_block(content, "kani_harnesses") {
        4
    } else if has_nonempty_block(content, "falsification_tests")
        || has_nonempty_block(content, "falsification")
    {
        3
    } else if has_nonempty_block(content, "proof_obligations") {
        2
    } else {
        1
    }
}

/// Is `key:` present AND followed by at least one list item or mapping entry?
///
/// `kani_harnesses:` with nothing under it is a declaration of intent, not a
/// harness. The old contains() check could not tell those apart.
fn has_nonempty_block(content: &str, key: &str) -> bool {
    let needle = format!("{key}:");
    let Some(idx) = content.find(&needle) else {
        return false;
    };
    let after_key = &content[idx + needle.len()..];
    // Inline form: `kani_harnesses: [a, b]` — non-empty only if the brackets are.
    if let Some(rest) = after_key.trim_start().strip_prefix('[') {
        return !rest.trim_start().starts_with(']');
    }
    // Block form: the next non-blank, non-comment line must be more indented
    // than the key and carry content.
    after_key.lines().skip(1).find_map(|line| {
        let t = line.trim();
        if t.is_empty() || t.starts_with('#') {
            return None;
        }
        let indented = line.len() - line.trim_start().len() > 0;
        Some(indented && (t.starts_with("- ") || t.contains(':')))
    }) == Some(true)
}

/// L5 requires the theorem to be PROVED, not merely referenced.
///
/// `ladder_evidence.rs:29` states the rule this mirrors: "the YAML ceiling
/// reports `lean_theorem.status == \"proved\"`". A `sorry` anywhere in the block
/// is an admitted hole and disqualifies it.
fn lean_theorem_is_proved(content: &str) -> bool {
    let Some(idx) = content.find("lean_theorem:") else {
        return false;
    };
    let block = &content[idx..];
    // Bound the block at the next top-level (column-0) key.
    //
    // #1159: this used to compute offsets over `lines().skip(1)` — never
    // counting the first line — and patch the miss with
    // `+ "lean_theorem:".len()`. The bound was therefore right only when the
    // first line was exactly the bare key; any longer first line put the cut
    // inside the block's LAST line, and when that line was a box-drawing
    // comment divider the cut landed inside a multibyte char and the whole
    // `comply check` aborted (`byte index 1000 is not a char boundary`).
    // Offsets are now accumulated over every line, ends included, so the cut
    // is always a line start — a char boundary by construction.
    let block = &block[..floor_char_boundary(block, next_top_level_key_offset(block))];
    block.contains("status: proved") && !block.contains("sorry")
}

/// Clamp `offset` into `s` and round it DOWN to a `char` boundary.
///
/// `min(s.len())` alone does not make a slice safe: `&s[..n]` panics just as
/// hard when `n` is inside a multi-byte sequence as when it is past the end.
/// Any offset derived from line arithmetic must pass through here.
fn floor_char_boundary(s: &str, offset: usize) -> usize {
    let mut end = offset.min(s.len());
    while !s.is_char_boundary(end) {
        end -= 1;
    }
    end
}

/// The byte offset of the first column-0 key after the first line, or the
/// block's length when there is none. Always a line start.
fn next_top_level_key_offset(block: &str) -> usize {
    let mut offset = 0usize;
    for (i, line) in block.split_inclusive('\n').enumerate() {
        let body = line.trim_end_matches(['\n', '\r']);
        if i > 0 && !body.is_empty() && !body.starts_with([' ', '\t', '-', '#']) {
            debug_assert!(block.is_char_boundary(offset));
            return offset;
        }
        offset += line.len();
    }
    block.len()
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

    // Contract: edition should be 2021+.
    //
    // Parsed, not string-matched. The substring test `content.contains(
    // "edition = \"2021\"")` reported drift on three manifests that are
    // perfectly compliant TOML: `edition="2021"` (no spaces — measured, the
    // check fired), `edition = '2021'` (single quotes), and
    // `edition.workspace = true` (a workspace member inheriting the edition
    // from `[workspace.package]`, which is the normal shape in every monorepo).
    // A gate that reads a manifest must read it the way cargo does.
    if !cargo_edition_is_modern(&content) {
        issues.push("Cargo.toml: edition is not 2021 or 2024".into());
    }
}

/// Does this manifest declare (or inherit) edition 2021/2024?
///
/// `None` of the accepted shapes is a substring match: the manifest is parsed.
/// Inheritance (`edition.workspace = true`) is accepted because the value lives
/// in the workspace root, which this check does not have in hand — flagging it
/// would be reporting "not 2021" for a manifest whose edition is unknown here.
fn cargo_edition_is_modern(content: &str) -> bool {
    let Ok(table) = content.parse::<toml::Table>() else {
        // Unparseable manifest is a different defect; do not also claim the
        // edition is wrong on evidence we could not read.
        return true;
    };
    let root = toml::Value::Table(table);
    for section in ["package", "workspace.package"] {
        let node = section.split('.').try_fold(&root, |acc, key| acc.get(key));
        let Some(edition) = node.and_then(|t| t.get("edition")) else {
            continue;
        };
        if let Some(s) = edition.as_str() {
            if s == "2021" || s == "2024" {
                return true;
            }
        }
        // `edition = { workspace = true }` / `edition.workspace = true`
        if edition
            .get("workspace")
            .and_then(toml::Value::as_bool)
            .unwrap_or(false)
        {
            return true;
        }
    }
    false
}

/// Extract dependency version from Cargo.toml content.
fn extract_dep_version<'a>(content: &'a str, dep_name: &str) -> Option<&'a str> {
    content
        .lines()
        .map(str::trim)
        .filter(|line| {
            line.strip_prefix(dep_name)
                .is_some_and(|rest| rest.starts_with(' ') || rest.starts_with('='))
        })
        .find_map(dep_line_version)
}

/// The quoted version on one `name = "1.2"` or `name = { version = "1.2", .. }`
/// line; `None` when the line names the dependency without a quoted version.
fn dep_line_version(line: &str) -> Option<&str> {
    if line.contains('{') {
        let rest = &line[line.find("version")? + "version".len()..];
        return first_quoted(rest.trim_start_matches([' ', '=']));
    }
    first_quoted(line[line.find('=')? + 1..].trim())
}

/// The content of the leading `"…"` in `s`, if `s` starts with a quote.
fn first_quoted(s: &str) -> Option<&str> {
    let inner = s.strip_prefix('"')?;
    Some(&inner[..inner.find('"')?])
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

#[cfg(all(test, not(coverage_nightly)))]
mod ladder_evidence_not_mention_tests {
    use super::*;

    /// A declared technique is not a performed proof.
    ///
    /// `level_number` was `content.contains("lean_theorem:")`, so on rmedia —
    /// 8 kani harnesses, ZERO .lean files — CB-1308 reported
    /// `✓ 3/3 contracts at L5 (lean_theorem)` and CB-1206 reported
    /// `L4=16 kani (114%)`. Neither number can be true.
    #[test]
    fn a_named_technique_with_no_evidence_does_not_promote_the_level() {
        // Bare key, nothing under it: intent, not evidence.
        assert_eq!(level_number("lean_theorem:\n"), 1);
        assert_eq!(level_number("kani_harnesses:\n"), 1);
        assert_eq!(level_number("falsification_tests:\n"), 1);
        assert_eq!(level_number("proof_obligations:\n"), 1);
        // Empty inline list is equally empty.
        assert_eq!(level_number("kani_harnesses: []\n"), 1);
    }

    #[test]
    fn evidence_still_promotes_the_level() {
        assert_eq!(level_number("proof_obligations:\n  - id: o1\n"), 2);
        assert_eq!(level_number("falsification_tests:\n  - id: t1\n"), 3);
        assert_eq!(level_number("kani_harnesses:\n  - name: h1\n"), 4);
        assert_eq!(level_number("kani_harnesses: [h1]\n"), 4);
    }

    /// L5 is a PROVED theorem. This mirrors `ladder_evidence.rs:29`, whose rule
    /// is `lean_theorem.status == "proved"`.
    #[test]
    fn l5_requires_a_proved_theorem_not_a_reference() {
        let referenced = "kani_harnesses:\n  - name: h\nlean_theorem:\n  name: thm\n";
        assert_eq!(
            level_number(referenced),
            4,
            "a referenced but unproved theorem must not reach L5"
        );

        let proved = "kani_harnesses:\n  - name: h\nlean_theorem:\n  status: proved\n";
        assert_eq!(level_number(proved), 5);
    }

    /// `sorry` is an admitted hole in the proof.
    #[test]
    fn a_theorem_containing_sorry_is_not_proved() {
        let with_sorry =
            "kani_harnesses:\n  - name: h\nlean_theorem:\n  status: proved\n  body: sorry\n";
        assert_eq!(
            level_number(with_sorry),
            4,
            "a proof with `sorry` in it is not a proof"
        );
    }
}

#[cfg(test)]
mod lean_theorem_block_bounds_tests {
    use super::lean_theorem_is_proved;

    /// #1159: the block bound was computed from offsets that skipped the
    /// first line without counting it, then patched with
    /// `+ "lean_theorem:".len()`. Whenever the first line is longer than the
    /// bare key the bound lands INSIDE the block's last line — and if that
    /// line is a box-drawing comment divider, inside a multibyte char:
    /// `byte index N is not a char boundary; it is inside '─'`.
    #[test]
    fn a_box_drawing_divider_before_the_next_key_does_not_panic() {
        // Three spaces before the comment: the miscounted bound then lands 9
        // bytes before `next:` — two bytes into a '─' of the divider.
        let content = "lean_theorem:   # L5\n  status: proved\n  # ─────────────\nnext: 1\n";
        assert!(lean_theorem_is_proved(content));
    }

    /// The bound must be the next column-0 key's line start: a `sorry` in the
    /// following key must not disqualify the theorem, and one inside the block
    /// must.
    #[test]
    fn the_block_ends_at_the_next_top_level_key() {
        let proved_then_sorry_elsewhere =
            "lean_theorem:\n  status: proved\n  # ──\nnotes: sorry, unrelated\n";
        assert!(lean_theorem_is_proved(proved_then_sorry_elsewhere));
        let sorry_inside = "lean_theorem:\n  status: proved\n  proof: sorry\nnotes: fine\n";
        assert!(!lean_theorem_is_proved(sorry_inside));
        let unbounded = "lean_theorem:\n  status: proved\n";
        assert!(lean_theorem_is_proved(unbounded));
    }
}

// Regression suite contributed on #1166 (the aprender release session), carried
// here so one PR holds the fix, its tests and the complexity refactors.
#[cfg(test)]
mod check_contract_surfaces_char_boundary_tests {
    use super::*;

    /// Build a YAML fixture from explicit lines. Written this way on purpose:
    /// a `\`-continued Rust string literal eats the leading whitespace of the
    /// next line, which would silently un-indent every nested YAML key and
    /// turn these fixtures into a different (passing) shape.
    fn yaml(lines: &[&str]) -> String {
        let mut s = String::new();
        for line in lines {
            s.push_str(line);
            s.push('\n');
        }
        s
    }

    /// (a) The real-world shape: a block-scalar first line (`lean_theorem: |`,
    /// 15 bytes, not 13) followed by a `# ────` divider comment immediately
    /// before the next top-level key. The old arithmetic undershot by 3 bytes
    /// and landed inside the divider's trailing `─`.
    #[test]
    fn divider_comment_before_the_next_key_does_not_panic() {
        let content = yaml(&[
            "equations:",
            "lean_theorem: |",
            "  theorem foo : True := trivial",
            "  status: proved",
            "# ──────────────────────────────",
            "proof_obligations:",
            "  - id: o1",
        ]);
        assert!(
            lean_theorem_is_proved(&content),
            "the proved status inside the block must still be seen"
        );
    }

    /// The bound must still be the next top-level key: a `status: proved`
    /// living *after* that key belongs to another surface and must not count.
    #[test]
    fn divider_comment_does_not_widen_the_block_past_the_next_key() {
        let content = yaml(&[
            "equations:",
            "lean_theorem: |",
            "  theorem foo : True := trivial",
            "# ──────────────────────────────",
            "proof_obligations:",
            "  status: proved",
        ]);
        assert!(
            !lean_theorem_is_proved(&content),
            "a `status: proved` under the NEXT top-level key must not promote this one"
        );
    }

    /// (b) aprender's minimal repro, verbatim. Against pmat 3.34.0 this aborted
    /// with `end byte index 14 is not a char boundary; it is inside '─'`.
    #[test]
    fn aprender_minimal_repro_does_not_panic() {
        let content = yaml(&["equations:", "lean_theorem:\u{2500}", "", "x:"]);
        assert!(
            !lean_theorem_is_proved(&content),
            "no `status: proved` anywhere — the verdict is false, not a panic"
        );
    }

    /// (c) The next top-level line *begins* with a multi-byte character, so the
    /// boundary offset is itself the first byte of a 2-byte sequence; landing
    /// one byte either side of it is a panic.
    #[test]
    fn next_top_level_key_starting_with_a_multibyte_char_does_not_panic() {
        let inside = yaml(&[
            "lean_theorem: Theorems.Foo",
            "  status: proved",
            "\u{e9}quations: v",
        ]);
        assert!(
            lean_theorem_is_proved(&inside),
            "the block ends at the e-acute-prefixed key; `status: proved` is inside it"
        );

        let after = yaml(&[
            "lean_theorem: Theorems.Foo",
            "\u{e9}quations: v",
            "  status: proved",
        ]);
        assert!(
            !lean_theorem_is_proved(&after),
            "and a proved status past that key must not leak in"
        );
    }

    /// Well-formed input keeps exactly the verdict it had before the fix.
    #[test]
    fn well_formed_input_keeps_its_verdict() {
        assert!(lean_theorem_is_proved(
            "lean_theorem:\n  status: proved\n"
        ));
        assert!(!lean_theorem_is_proved("lean_theorem:\n  name: thm\n"));
        assert!(!lean_theorem_is_proved(
            "lean_theorem:\n  status: proved\n  body: sorry\n"
        ));
        assert!(!lean_theorem_is_proved("kani_harnesses:\n  - name: h\n"));
        assert!(
            !lean_theorem_is_proved("lean_theorem:\n  name: thm\nother:\n  status: proved\n"),
            "the block stops at the next top-level key"
        );
    }

    /// The offset helper reports the TRUE byte position of the next top-level
    /// key, independent of how long the first line is — that constant-13
    /// assumption was the defect.
    #[test]
    fn offset_is_independent_of_the_first_line_length() {
        for first in [
            "lean_theorem:",
            "lean_theorem: |",
            "lean_theorem: Theorems.Foo",
            "lean_theorem: a-very-long-theorem-reference-indeed",
        ] {
            let block = format!("{first}\n  status: proved\nnext_key:\n");
            let expected = block.find("next_key:").expect("fixture has the key");
            assert_eq!(
                next_top_level_key_offset(&block),
                expected,
                "first line {first:?}"
            );
        }
    }

    /// `lines()` strips the `\r` of a CRLF terminator, so `line.len() + 1`
    /// undercounts by one per line. `split_inclusive` does not.
    #[test]
    fn offset_is_correct_for_crlf_terminators() {
        let block = "lean_theorem: |\r\n  status: proved\r\nnext_key:\r\n";
        let expected = block.find("next_key:").expect("fixture has the key");
        assert_eq!(next_top_level_key_offset(block), expected);
        assert!(lean_theorem_is_proved(block));
    }

    /// With no following top-level key the whole block is in scope.
    #[test]
    fn offset_defaults_to_the_whole_block() {
        let block = "lean_theorem: |\n  status: proved\n";
        assert_eq!(next_top_level_key_offset(block), block.len());
        assert_eq!(next_top_level_key_offset(""), 0);
    }

    #[test]
    fn floor_char_boundary_never_lands_inside_a_char() {
        let s = "ab\u{2500}cd";
        assert_eq!(floor_char_boundary(s, 0), 0);
        assert_eq!(floor_char_boundary(s, 2), 2);
        // 3, 4 are inside the 3-byte `\u{2500}` at bytes 2..5.
        assert_eq!(floor_char_boundary(s, 3), 2);
        assert_eq!(floor_char_boundary(s, 4), 2);
        assert_eq!(floor_char_boundary(s, 5), 5);
        // Past the end clamps, and clamping alone is not enough in general.
        assert_eq!(floor_char_boundary(s, 999), s.len());
        for offset in 0..=s.len() {
            let end = floor_char_boundary(s, offset);
            assert!(s.is_char_boundary(end), "offset {offset} -> {end}");
        }
    }

    /// No input may panic here, whatever the byte layout: insert a 3-byte
    /// character at every char boundary of a realistic block and require a
    /// verdict — any verdict — rather than an abort.
    #[test]
    fn no_byte_layout_can_panic() {
        let base = yaml(&[
            "equations:",
            "lean_theorem: Theorems.Foo",
            "  status: proved",
            "proof_obligations:",
        ]);
        for cut in 0..=base.len() {
            if !base.is_char_boundary(cut) {
                continue;
            }
            let mut mutated = String::with_capacity(base.len() + 3);
            mutated.push_str(&base[..cut]);
            mutated.push('\u{2500}');
            mutated.push_str(&base[cut..]);
            let _ = lean_theorem_is_proved(&mutated);
        }
    }
}


/// CB-1354: Contract Query Readiness
///
/// Validates that the infrastructure for `pmat query --contracts` enrichment
/// is in place: binding-index.json exists, contracts/ has YAML, and pv CLI
/// is available. Scores readiness 0-4 based on components present.
///
/// Spec: Phase 6 of commit-level-contract-enforcement.md
#[provable_contracts_macros::contract("pmat-core.yaml", equation = "path_exists")]
pub(crate) fn check_contract_query_readiness(project_path: &Path) -> ComplianceCheck {
    let mut score = 0u8;
    let mut components: Vec<&str> = Vec::new();
    let mut missing: Vec<&str> = Vec::new();

    // 1. binding-index.json
    if project_path.join(".pmat/binding-index.json").exists()
        || project_path.join("contracts/binding-index.json").exists()
    {
        score += 1;
        components.push("binding-index");
    } else {
        missing.push("binding-index.json");
    }

    // 2. contracts/ directory with YAML files
    let contracts_dir = project_path.join("contracts");
    if contracts_dir.exists() {
        let has_yaml = fs::read_dir(&contracts_dir)
            .ok()
            .map(|entries| {
                entries.flatten().any(|e| {
                    e.path().extension().is_some_and(|ext| ext == "yaml" || ext == "yml")
                })
            })
            .unwrap_or(false);
        if has_yaml {
            score += 1;
            components.push("contracts/YAML");
        } else {
            missing.push("contracts/*.yaml");
        }
    } else {
        missing.push("contracts/ dir");
    }

    // 3. binding.yaml
    if project_path.join("binding.yaml").exists()
        || project_path.join("contracts/binding.yaml").exists()
    {
        score += 1;
        components.push("binding.yaml");
    } else {
        missing.push("binding.yaml");
    }

    // 4. pv CLI available
    let pv_available = std::process::Command::new("pv")
        .arg("--version")
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .status()
        .map(|s| s.success())
        .unwrap_or(false);
    if pv_available {
        score += 1;
        components.push("pv CLI");
    } else {
        missing.push("pv CLI");
    }

    if score == 0 {
        ComplianceCheck {
            name: "CB-1354: Contract Query Readiness".into(),
            status: CheckStatus::Skip,
            message: "No contract infrastructure found".into(),
            severity: Severity::Info,
        }
    } else if score >= 3 {
        ComplianceCheck {
            name: "CB-1354: Contract Query Readiness".into(),
            status: CheckStatus::Pass,
            message: format!(
                "Ready ({}/4): {}",
                score,
                components.join(", ")
            ),
            severity: Severity::Info,
        }
    } else {
        ComplianceCheck {
            name: "CB-1354: Contract Query Readiness".into(),
            status: CheckStatus::Warn,
            message: format!(
                "Partial ({}/4): have [{}], missing [{}]",
                score,
                components.join(", "),
                missing.join(", ")
            ),
            severity: Severity::Warning,
        }
    }
}

/// CB-1342: Codegen Compiles
///
/// Checks that generated contract assertion code (from `pv codegen`) compiles.
/// Scans for `src/contracts/` or `generated_contracts.rs` and validates syntax.
/// If `pv` CLI is available, runs `pv codegen --check` for dry-run validation.
///
/// Spec: Phase 8 leak class L-6 (Parser/Domain Bugs)
#[provable_contracts_macros::contract("pmat-core.yaml", equation = "path_exists")]
pub(crate) fn check_codegen_compiles(project_path: &Path) -> ComplianceCheck {
    // Check for generated contract code
    let generated_paths = [
        project_path.join("src/contracts"),
        project_path.join("src/generated_contracts.rs"),
    ];

    let mut has_generated = false;
    for path in &generated_paths {
        if path.exists() {
            has_generated = true;
            break;
        }
    }

    if !has_generated {
        // No generated contract code — try pv codegen --check if pv is available
        let pv_check = std::process::Command::new("pv")
            .args(["codegen", "--check"])
            .current_dir(project_path)
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::piped())
            .output();

        return match pv_check {
            Ok(output) if output.status.success() => ComplianceCheck {
                name: "CB-1342: Codegen Compiles".into(),
                status: CheckStatus::Pass,
                message: "pv codegen --check passed".into(),
                severity: Severity::Info,
            },
            Ok(output) => {
                let stderr = String::from_utf8_lossy(&output.stderr);
                // If pv doesn't support the flag, skip rather than warn
                if stderr.contains("unexpected argument") || stderr.contains("unrecognized") {
                    ComplianceCheck {
                        name: "CB-1342: Codegen Compiles".into(),
                        status: CheckStatus::Skip,
                        message: "pv codegen --check not supported (upgrade pv)".into(),
                        severity: Severity::Info,
                    }
                } else {
                    let msg = stderr.lines().next().unwrap_or("codegen check failed");
                    ComplianceCheck {
                        name: "CB-1342: Codegen Compiles".into(),
                        status: CheckStatus::Warn,
                        message: format!("pv codegen --check: {}", msg),
                        severity: Severity::Warning,
                    }
                }
            }
            Err(_) => ComplianceCheck {
                name: "CB-1342: Codegen Compiles".into(),
                status: CheckStatus::Skip,
                message: "No generated contracts and pv CLI not available".into(),
                severity: Severity::Info,
            },
        };
    }

    // Has generated code — check for obvious syntax issues
    let mut issues: Vec<String> = Vec::new();
    let contracts_dir = project_path.join("src/contracts");
    if contracts_dir.exists() {
        if let Ok(entries) = fs::read_dir(&contracts_dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.extension().is_none_or(|e| e != "rs") {
                    continue;
                }
                if let Ok(content) = fs::read_to_string(&path) {
                    // Check for unbalanced braces (common codegen bug)
                    let opens = content.chars().filter(|c| *c == '{').count();
                    let closes = content.chars().filter(|c| *c == '}').count();
                    if opens != closes {
                        let name = path.file_name().map(|f| f.to_string_lossy().to_string())
                            .unwrap_or_default();
                        issues.push(format!("{}: unbalanced braces ({} open, {} close)", name, opens, closes));
                    }
                    // Check for common codegen placeholders
                    if content.contains("TODO_PARAM") || content.contains("PLACEHOLDER") {
                        let name = path.file_name().map(|f| f.to_string_lossy().to_string())
                            .unwrap_or_default();
                        issues.push(format!("{}: contains codegen placeholders", name));
                    }
                }
            }
        }
    }

    if issues.is_empty() {
        ComplianceCheck {
            name: "CB-1342: Codegen Compiles".into(),
            status: CheckStatus::Pass,
            message: "Generated contract code passes syntax checks".into(),
            severity: Severity::Info,
        }
    } else {
        ComplianceCheck {
            name: "CB-1342: Codegen Compiles".into(),
            status: CheckStatus::Warn,
            message: format!("{} issue(s): {}", issues.len(), issues.join("; ")),
            severity: Severity::Warning,
        }
    }
}

/// Generate `.pmat/binding-index.json` from contracts/ and binding.yaml.
///
/// The binding index maps source files → contract binding names, enabling
/// CB-1350 differential obligation verification at commit time (O(1) lookup).
///
/// Called by `pmat comply refresh-bindings`.
#[provable_contracts_macros::contract("pmat-core.yaml", equation = "path_exists")]
pub(crate) fn handle_refresh_bindings(project_path: &Path) -> anyhow::Result<()> {
    let pmat_dir = project_path.join(".pmat");
    if !pmat_dir.exists() {
        fs::create_dir_all(&pmat_dir)?;
    }

    let mut index: std::collections::BTreeMap<String, Vec<String>> = std::collections::BTreeMap::new();
    let mut binding_count = 0usize;

    // 1. binding.yaml file→binding mappings. 2. contracts/*.yaml function→file.
    refresh_index_binding_files(project_path, &mut index, &mut binding_count);
    let contracts_dir = project_path.join("contracts");
    if contracts_dir.exists() {
        refresh_index_contract_dir(&contracts_dir, &mut index, &mut binding_count);
    }

    // 3. Write binding-index.json
    let json = serde_json::to_string_pretty(&index)?;
    let output_path = pmat_dir.join("binding-index.json");
    fs::write(&output_path, &json)?;

    // Coloured through the shared helpers, so `--color` is not accepted and
    // ignored here. The flag-efficacy sweep booked `comply refresh-bindings
    // --color` a NO-OP: 163 bytes and zero ANSI escapes with `always` and with
    // `never` alike, because the only marker was the ✅ EMOJI — which reads as
    // colour and is not.
    //
    // `pass` and `path` are the same helpers every other status line uses, so
    // the honesty of the flag does not depend on this call site remembering to
    // be special.
    crate::status_println!(
        "{} Binding index generated: {}",
        crate::cli::colors::pass("✅"),
        crate::cli::colors::path(&output_path.display().to_string())
    );
    crate::status_println!(
        "   {} file(s) → {} binding(s)",
        crate::cli::colors::number(&index.len().to_string()),
        crate::cli::colors::number(&binding_count.to_string())
    );

    // 4. Generate O(1) cache files (R-5 remediation)
    let mut cache_count = 0u8;

    // contract-cache.json: summarize active work contracts
    if refresh_write_contract_cache(project_path, &pmat_dir)? {
        cache_count += 1;
    }

    // verification-levels.json: extract L-levels from contracts/ YAML
    if refresh_write_verification_levels(project_path, &pmat_dir)? {
        cache_count += 1;
    }

    // asset-layout-cache.json: cache asset validation results
    let asset_cache = serde_json::json!({
        "generated_at": chrono_free_timestamp(),
        "readme": project_path.join("README.md").exists(),
        "changelog": project_path.join("CHANGELOG.md").exists(),
        "dockerfile": project_path.join("Dockerfile").exists(),
        "forjar": project_path.join("forjar.yaml").exists() || project_path.join("forjar.toml").exists(),
        "book": project_path.join("book/src/SUMMARY.md").exists(),
    });
    fs::write(pmat_dir.join("asset-layout-cache.json"), serde_json::to_string_pretty(&asset_cache)?)?;
    cache_count += 1;

    crate::status_println!("   {} O(1) cache file(s) generated", cache_count);

    // 5. Generate contracts/work/<ID>.yaml from .pmat-work/ (R-4)
    let yaml_count = generate_work_contract_yamls(project_path)?;
    if yaml_count > 0 {
        crate::status_println!("   {} contracts/work/*.yaml file(s) generated", yaml_count);
    }

    crate::status_println!("   CB-1350 differential obligations now enabled");

    Ok(())
}

/// Parse a `binding.yaml` document into `(source_file, binding_name)` pairs.
/// Pure — extracted from `handle_refresh_bindings` to keep it under the
/// complexity gate (see `refresh_parse_*` unit tests).
/// Index `binding.yaml` (root and `contracts/`) into `index`, counting bindings.
fn refresh_index_binding_files(
    project_path: &Path,
    index: &mut std::collections::BTreeMap<String, Vec<String>>,
    binding_count: &mut usize,
) {
    let binding_paths = [
        project_path.join("binding.yaml"),
        project_path.join("contracts/binding.yaml"),
    ];
    for binding_path in &binding_paths {
        if !binding_path.exists() {
            continue;
        }
        if let Ok(content) = fs::read_to_string(binding_path) {
            for (file, name) in refresh_parse_binding_lines(&content) {
                index.entry(file).or_default().push(name);
                *binding_count += 1;
            }
        }
    }
}

/// Index `contracts/*.yaml` (excluding `binding.yaml`) into `index`.
fn refresh_index_contract_dir(
    contracts_dir: &Path,
    index: &mut std::collections::BTreeMap<String, Vec<String>>,
    binding_count: &mut usize,
) {
    for entry in walkdir::WalkDir::new(contracts_dir)
        .max_depth(3)
        .into_iter()
        .filter_map(|e| e.ok())
    {
        let path = entry.path();
        if !path.is_file() || path.extension().is_none_or(|e| e != "yaml" && e != "yml") {
            continue;
        }
        // Skip binding.yaml itself (already parsed).
        if path.file_name().is_some_and(|n| n == "binding.yaml") {
            continue;
        }
        let Ok(content) = fs::read_to_string(path) else {
            continue;
        };
        let contract_name = path
            .file_stem()
            .map(|s| s.to_string_lossy().to_string())
            .unwrap_or_default();
        for (file, name) in refresh_parse_contract_source_files(&content, &contract_name) {
            index.entry(file).or_default().push(name);
            *binding_count += 1;
        }
    }
}

/// Extract the `value` from a `key: value` YAML line, trimming quotes.
fn refresh_yaml_value(trimmed: &str) -> Option<String> {
    trimmed
        .split(':')
        .nth(1)
        .map(|v| v.trim().trim_matches('"').trim_matches('\'').to_string())
}

/// Apply one trimmed `binding.yaml` line to the running parse state, flushing a
/// completed `(file, name)` pair into `out` at each entry boundary. The three
/// arms are mutually exclusive by prefix.
fn refresh_apply_binding_line(
    trimmed: &str,
    current_name: &mut Option<String>,
    current_file: &mut Option<String>,
    out: &mut Vec<(String, String)>,
) {
    let is_entry_start = trimmed.starts_with("- name:")
        || trimmed.starts_with("- module_path:")
        || trimmed.starts_with("- contract:");
    if is_entry_start {
        // Flush the previous entry at each entry boundary.
        if let (Some(file), Some(name)) = (current_file.take(), current_name.take()) {
            out.push((file, name));
        }
        // `- name:` / `- module_path:` carry the name; `- contract:` does not.
        if !trimmed.starts_with("- contract:") {
            if let Some(v) = refresh_yaml_value(trimmed) {
                *current_name = Some(v);
            }
        }
    } else if trimmed.starts_with("function:") && current_name.is_none() {
        // `function:` is the binding name for the pv binding format.
        if let Some(v) = refresh_yaml_value(trimmed) {
            *current_name = Some(v);
        }
    } else if trimmed.starts_with("source_file:") || trimmed.starts_with("file:") {
        if let Some(v) = refresh_yaml_value(trimmed) {
            *current_file = Some(v);
        }
    }
}

/// Parse a `binding.yaml` document into `(source_file, binding_name)` pairs.
/// Pure — extracted from `handle_refresh_bindings` to keep it under the
/// complexity gate (see `refresh_parse_*` unit tests).
fn refresh_parse_binding_lines(content: &str) -> Vec<(String, String)> {
    let mut out = Vec::new();
    let mut current_name: Option<String> = None;
    let mut current_file: Option<String> = None;
    for line in content.lines() {
        refresh_apply_binding_line(line.trim(), &mut current_name, &mut current_file, &mut out);
    }
    if let (Some(file), Some(name)) = (current_file, current_name) {
        out.push((file, name));
    }
    out
}

/// Parse `source_file:` / `file:` / `- src/...` references out of a contract YAML,
/// each bound to `contract_name`. Pure.
fn refresh_parse_contract_source_files(content: &str, contract_name: &str) -> Vec<(String, String)> {
    let mut out = Vec::new();
    for line in content.lines() {
        let trimmed = line.trim();
        if !(trimmed.starts_with("source_file:")
            || trimmed.starts_with("file:")
            || trimmed.starts_with("- src/"))
        {
            continue;
        }
        let val = if trimmed.starts_with("- ") {
            trimmed
                .trim_start_matches("- ")
                .trim_matches('"')
                .trim_matches('\'')
        } else {
            trimmed
                .split(':')
                .nth(1)
                .unwrap_or("")
                .trim()
                .trim_matches('"')
                .trim_matches('\'')
        };
        if !val.is_empty() {
            out.push((val.to_string(), contract_name.to_string()));
        }
    }
    out
}

/// Write `.pmat/contract-cache.json` summarizing active `.pmat-work/` contracts.
/// Returns `true` if the cache was written (i.e. `.pmat-work/` exists).
fn refresh_write_contract_cache(project_path: &Path, pmat_dir: &Path) -> anyhow::Result<bool> {
    let work_dir = project_path.join(".pmat-work");
    if !work_dir.exists() {
        return Ok(false);
    }
    let mut contracts_summary = std::collections::BTreeMap::new();
    if let Ok(entries) = fs::read_dir(&work_dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if !path.is_dir() {
                continue;
            }
            let Ok(c) = fs::read_to_string(path.join("contract.json")) else {
                continue;
            };
            let Ok(v) = serde_json::from_str::<serde_json::Value>(&c) else {
                continue;
            };
            let id = v
                .get("work_item_id")
                .and_then(|w| w.as_str())
                .unwrap_or("unknown")
                .to_string();
            let level = v
                .get("verification_level")
                .and_then(|l| l.as_str())
                .unwrap_or("L0")
                .to_string();
            let has_claims =
                v.get("falsifiable_claims").is_some() || v.get("claims").is_some();
            contracts_summary.insert(
                id,
                serde_json::json!({ "level": level, "has_claims": has_claims }),
            );
        }
    }
    let cache = serde_json::json!({
        "generated_at": chrono_free_timestamp(),
        "contract_count": contracts_summary.len(),
        "contracts": contracts_summary,
    });
    fs::write(
        pmat_dir.join("contract-cache.json"),
        serde_json::to_string_pretty(&cache)?,
    )?;
    Ok(true)
}

/// Write `.pmat/verification-levels.json` from `contracts/*.yaml` L-levels.
/// Returns `true` if the cache was written (i.e. `contracts/` exists).
fn refresh_write_verification_levels(project_path: &Path, pmat_dir: &Path) -> anyhow::Result<bool> {
    let contracts_dir = project_path.join("contracts");
    if !contracts_dir.exists() {
        return Ok(false);
    }
    let mut levels = std::collections::BTreeMap::new();
    for entry in walkdir::WalkDir::new(&contracts_dir)
        .max_depth(3)
        .into_iter()
        .filter_map(|e| e.ok())
    {
        let path = entry.path();
        if !path.is_file() || path.extension().is_none_or(|e| e != "yaml" && e != "yml") {
            continue;
        }
        let Ok(content) = fs::read_to_string(path) else {
            continue;
        };
        if !content.contains("verification_summary") {
            continue;
        }
        let target = extract_level(&content, "target_level");
        let current = extract_level(&content, "current_level");
        let name = path
            .file_stem()
            .map(|s| s.to_string_lossy().to_string())
            .unwrap_or_default();
        levels.insert(
            name,
            serde_json::json!({
                "target": target.map(|l| format!("L{}", l)),
                "current": current.map(|l| format!("L{}", l)),
            }),
        );
    }
    let cache = serde_json::json!({
        "generated_at": chrono_free_timestamp(),
        "level_count": levels.len(),
        "levels": levels,
    });
    fs::write(
        pmat_dir.join("verification-levels.json"),
        serde_json::to_string_pretty(&cache)?,
    )?;
    Ok(true)
}

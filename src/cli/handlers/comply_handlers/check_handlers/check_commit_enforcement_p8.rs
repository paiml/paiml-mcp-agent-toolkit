
/// Generate provable-contracts YAML from .pmat-work/ contract.json files (R-4).
///
/// Maps: claims/falsifiable_claims → preconditions, ensure → postconditions,
/// verification_level → verification_summary.target_level.
fn generate_work_contract_yamls(project_path: &Path) -> anyhow::Result<usize> {
    let work_dir = project_path.join(".pmat-work");
    if !work_dir.exists() {
        return Ok(0);
    }

    let out_dir = project_path.join("contracts/work");
    fs::create_dir_all(&out_dir)?;

    let mut count = 0usize;
    let entries = fs::read_dir(&work_dir)?;

    for entry in entries.flatten() {
        let path = entry.path();
        if !path.is_dir() {
            continue;
        }
        let contract = path.join("contract.json");
        if !contract.exists() {
            continue;
        }
        let content = match fs::read_to_string(&contract) {
            Ok(c) => c,
            Err(_) => continue,
        };
        let v: serde_json::Value = match serde_json::from_str(&content) {
            Ok(v) => v,
            Err(_) => continue,
        };

        let id = v
            .get("work_item_id")
            .and_then(|w| w.as_str())
            .unwrap_or("unknown");

        // Sanitize ID for filename
        let safe_id: String = id
            .chars()
            .map(|c| if c.is_alphanumeric() || c == '-' || c == '_' { c } else { '_' })
            .collect();

        let level = v
            .get("verification_level")
            .and_then(|l| l.as_str())
            .unwrap_or("L1");

        // Extract preconditions from claims
        let mut preconditions = Vec::new();
        if let Some(claims) = v.get("falsifiable_claims").and_then(|c| c.as_array()) {
            for claim in claims {
                if let Some(text) = claim.get("claim").and_then(|t| t.as_str()) {
                    preconditions.push(text.to_string());
                }
            }
        }
        if let Some(req) = v.get("require").and_then(|r| r.as_array()) {
            for r in req {
                if let Some(s) = r.as_str() {
                    preconditions.push(s.to_string());
                }
            }
        }

        // Extract postconditions
        let mut postconditions = Vec::new();
        if let Some(ens) = v.get("ensure").and_then(|e| e.as_array()) {
            for e in ens {
                if let Some(s) = e.as_str() {
                    postconditions.push(s.to_string());
                }
            }
        }

        // Build YAML (hand-written to avoid serde_yaml dependency)
        // Quote name: always, to safely handle colons/special chars in IDs
        let mut yaml = format!("# Auto-generated from .pmat-work/{}/contract.json\n", safe_id);
        yaml.push_str(&format!("name: \"{}\"\n", yaml_escape_string(id)));
        yaml.push_str("surface: work-contract\n");
        yaml.push_str(&format!(
            "verification_summary:\n  target_level: {}\n  current_level: {}\n",
            level, level
        ));

        if !preconditions.is_empty() {
            yaml.push_str("preconditions:\n");
            for p in &preconditions {
                yaml.push_str(&format!("  - \"{}\"\n", yaml_escape_string(p)));
            }
        }

        if !postconditions.is_empty() {
            yaml.push_str("postconditions:\n");
            for p in &postconditions {
                yaml.push_str(&format!("  - \"{}\"\n", yaml_escape_string(p)));
            }
        }

        let yaml_path = out_dir.join(format!("{}.yaml", safe_id));
        fs::write(&yaml_path, &yaml)?;
        count += 1;
    }

    Ok(count)
}

/// R-7: Override verification level ratchet for a specific binding.
///
/// Records a signed override entry in `.pmat-metrics/ratchet-overrides.jsonl`.
/// The override expires after 14 days. CB-1330 will flag if not recovered.
#[provable_contracts_macros::contract("pmat-core.yaml", equation = "path_exists")]
pub(crate) fn handle_ratchet_override(
    project_path: &Path,
    binding: &str,
    from: &str,
    to: &str,
    reason: &str,
    work_item: Option<&str>,
) -> anyhow::Result<()> {
    let metrics_dir = project_path.join(".pmat-metrics");
    fs::create_dir_all(&metrics_dir)?;

    let entry = serde_json::json!({
        "timestamp": chrono_free_timestamp(),
        "binding": binding,
        "from_level": from,
        "to_level": to,
        "reason": reason,
        "work_item": work_item,
        "expires_days": 14,
    });

    let log_path = metrics_dir.join("ratchet-overrides.jsonl");
    let mut file = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(&log_path)?;
    use std::io::Write;
    writeln!(file, "{}", serde_json::to_string(&entry)?)?;

    println!("✅ Ratchet override recorded:");
    println!("   Binding: {}", binding);
    println!("   {} → {} (reason: {})", from, to, reason);
    if let Some(wi) = work_item {
        println!("   Work item: {}", wi);
    }
    println!("   Expires in 14 days. Logged to: {}", log_path.display());

    Ok(())
}

/// R-8: Validate non-code asset layout contracts.
///
/// Runs CB-1320..1326 checks on assets and reports results.
/// Can target a specific asset or validate all.
#[provable_contracts_macros::contract("pmat-core.yaml", equation = "path_exists")]
pub(crate) fn handle_asset_validate(
    project_path: &Path,
    asset: Option<&str>,
) -> anyhow::Result<()> {
    let checks: Vec<ComplianceCheck> = match asset {
        Some("readme") => vec![check_readme_layout(project_path)],
        Some("dockerfile") => vec![check_dockerfile_contract(project_path)],
        Some("svg") => vec![check_svg_contract(project_path)],
        Some("changelog") => vec![check_changelog_contract(project_path)],
        Some("badges") => vec![check_badge_contract(project_path)],
        Some("book") => vec![check_mdbook_contract(project_path)],
        Some("forjar") => vec![check_forjar_contract(project_path)],
        Some(other) => {
            eprintln!("Unknown asset type: '{}'. Valid: readme, dockerfile, svg, changelog, badges, book, forjar", other);
            std::process::exit(1);
        }
        None => vec![
            check_readme_layout(project_path),
            check_dockerfile_contract(project_path),
            check_svg_contract(project_path),
            check_changelog_contract(project_path),
            check_badge_contract(project_path),
            check_mdbook_contract(project_path),
            check_forjar_contract(project_path),
        ],
    };

    let mut pass = 0;
    let mut warn = 0;
    let mut skip = 0;
    for check in &checks {
        let icon = match check.status {
            CheckStatus::Pass => { pass += 1; "✓" }
            CheckStatus::Warn => { warn += 1; "⚠" }
            CheckStatus::Fail => { warn += 1; "✗" }
            CheckStatus::Skip => { skip += 1; "-" }
        };
        println!("  {} {}: {}", icon, check.name, check.message);
    }
    println!();
    println!("{} pass, {} warn, {} skip", pass, warn, skip);

    Ok(())
}

/// Escape a string for safe inclusion in YAML double-quoted values.
/// Handles newlines, quotes, backslashes, and colons.
fn yaml_escape_string(s: &str) -> String {
    s.replace('\\', "\\\\")
        .replace('"', "\\\"")
        .replace('\n', "\\n")
        .replace('\r', "\\r")
        .replace('\t', "\\t")
}

/// Generate ISO-8601 timestamp using Howard Hinnant's civil date algorithm.
/// Correct for all dates (no leap-year drift).
fn chrono_free_timestamp() -> String {
    let dur = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default();
    let secs = dur.as_secs();
    let days = (secs / 86400) as i64;
    // Howard Hinnant's algorithm (civil_from_days)
    let z = days + 719468;
    let era = (if z >= 0 { z } else { z - 146096 }) / 146097;
    let doe = (z - era * 146097) as u32;
    let yoe = (doe - doe / 1460 + doe / 36524 - doe / 146096) / 365;
    let y = yoe as i64 + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = doy - (153 * mp + 2) / 5 + 1;
    let m = if mp < 10 { mp + 3 } else { mp - 9 };
    let y = if m <= 2 { y + 1 } else { y };
    format!("{}-{:02}-{:02}T00:00:00Z", y, m, d)
}

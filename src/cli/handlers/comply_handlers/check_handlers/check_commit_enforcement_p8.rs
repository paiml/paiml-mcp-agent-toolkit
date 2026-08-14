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
            .map(|c| {
                if c.is_alphanumeric() || c == '-' || c == '_' {
                    c
                } else {
                    '_'
                }
            })
            .collect();

        let level = v
            .get("verification_level")
            .and_then(|l| l.as_str())
            .unwrap_or("L1");

        // Extract preconditions from require objects
        let mut preconditions = Vec::new();
        if let Some(req) = v.get("require").and_then(|r| r.as_array()) {
            for r in req {
                if let Some(s) = r.get("description").and_then(|d| d.as_str()) {
                    preconditions.push(s.to_string());
                }
            }
        }

        // Extract falsifiable_claims from claims objects
        let mut falsifiable_claims = Vec::new();
        if let Some(claims) = v.get("claims").and_then(|c| c.as_array()) {
            for claim in claims {
                if let Some(text) = claim.get("hypothesis").and_then(|t| t.as_str()) {
                    falsifiable_claims.push(text.to_string());
                }
            }
        }

        // Extract postconditions
        let mut postconditions = Vec::new();
        if let Some(ens) = v.get("ensure").and_then(|e| e.as_array()) {
            for e in ens {
                // Ensure clauses are objects with a description
                if let Some(s) = e.get("description").and_then(|d| d.as_str()) {
                    postconditions.push(s.to_string());
                } else if let Some(s) = e.as_str() {
                    // Fallback for older formats if any
                    postconditions.push(s.to_string());
                }
            }
        }

        // Count all contractual obligations for verification_summary.total_obligations
        // Schema requires this field; omitting it causes pv lint PV-VAL-001 failures.
        let count_array =
            |key: &str| -> usize { v.get(key).and_then(|a| a.as_array()).map_or(0, |a| a.len()) };
        let total_obligations = count_array("claims")
            + count_array("ensure")
            + count_array("require")
            + count_array("invariant");

        // Build YAML (hand-written to avoid serde_yaml dependency)
        // Quote name: always, to safely handle colons/special chars in IDs
        let mut yaml = format!(
            "# Auto-generated from .pmat-work/{}/contract.json\n",
            safe_id
        );
        yaml.push_str(&format!("name: \"{}\"\n", yaml_escape_string(id)));
        // Emit metadata block (KAIZEN-0175): pv 0.31 schema requires metadata
        // with version/description/references. Deterministic values only —
        // no timestamps, to avoid daily churn in contracts/work/*.yaml.
        //
        // KAIZEN-0190 (SCHEMA-003): declare `kind: schema` so pv treats these
        // as reference documents, not mathematical kernel contracts. Work
        // contracts are derived from .pmat-work/*/contract.json and document
        // work-item obligations, not provable math. Without this, pv's kernel
        // validator demands `equations`, `proof_obligations`, `falsification_tests`,
        // and `kani_harnesses` — none of which apply to work-tracking artifacts.
        yaml.push_str("metadata:\n");
        yaml.push_str("  version: \"1.0.0\"\n");
        yaml.push_str("  kind: schema\n");
        yaml.push_str(&format!(
            "  description: \"Auto-generated work-contract for {}\"\n",
            yaml_escape_string(id)
        ));
        yaml.push_str("  references:\n");
        yaml.push_str(&format!("    - \".pmat-work/{}/contract.json\"\n", safe_id));
        yaml.push_str("surface: work-contract\n");
        yaml.push_str(&format!(
            "verification_summary:\n  target_level: {}\n  current_level: {}\n  total_obligations: {}\n",
            level, level, total_obligations
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

        if !falsifiable_claims.is_empty() {
            yaml.push_str("falsifiable_claims:\n");
            for c in &falsifiable_claims {
                yaml.push_str(&format!("  - id: \"{}\"\n", yaml_escape_string(c)));
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

    crate::status_println!("✅ Ratchet override recorded:");
    crate::status_println!("   Binding: {}", binding);
    crate::status_println!("   {} → {} (reason: {})", from, to, reason);
    if let Some(wi) = work_item {
        crate::status_println!("   Work item: {}", wi);
    }
    crate::status_println!("   Expires in 14 days. Logged to: {}", log_path.display());

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

    let mut pass = 0usize;
    let mut warn = 0usize;
    let mut skip = 0usize;
    for check in &checks {
        match check.status {
            CheckStatus::Pass => pass += 1,
            // A failing asset contract is counted with the warnings on purpose:
            // this command does not gate. Only the tally is shared — the glyph
            // is not.
            CheckStatus::Warn | CheckStatus::Fail => warn += 1,
            CheckStatus::Skip => skip += 1,
        }
        println!("{}", format_asset_check_line(check));
    }
    println!();
    println!("{}", format_asset_check_totals(pass, warn, skip));

    Ok(())
}

/// One `  <glyph> <name>: <message>` line for an asset contract check.
///
/// The glyph comes from [`check_status_icon`], the one renderer of a
/// `CheckStatus` in this crate. This function used to carry a second, private
/// copy of that table — the same four glyphs as bare `&'static str` literals —
/// so `comply asset-validate --color always` was byte-identical to `--color
/// never` while `comply report --format text`, printing the SAME enum, painted
/// them. A duplicated glyph table cannot be kept in sync with a colour rule it
/// does not know exists; there is now only one table.
fn format_asset_check_line(check: &ComplianceCheck) -> String {
    use crate::cli::colors as c;
    format!(
        "  {} {}: {}",
        check_status_icon(check.status),
        c::label(&check.name),
        check.message
    )
}

/// The `N pass, N warn, N skip` tally under the check list.
fn format_asset_check_totals(pass: usize, warn: usize, skip: usize) -> String {
    use crate::cli::colors as c;
    format!(
        "{} pass, {} warn, {} skip",
        c::colored(c::GREEN, &pass.to_string()),
        c::colored(c::YELLOW, &warn.to_string()),
        c::dim(&skip.to_string())
    )
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

#[cfg(test)]
mod asset_validate_color_tests {
    // `comply asset-validate` must honour `--color`, and must render a
    // CheckStatus the same way every other comply surface does.
    //
    // This handler carried a private second copy of the glyph table — the same
    // four glyphs as bare literals — so `--color always` was byte-identical to
    // `--color never` (645 bytes, 0 escapes) while `comply report --format
    // text`, printing the SAME enum, painted them. The copy is gone; these
    // tests pin that it stays gone.
    use super::*;
    use crate::cli::colors::{assert_honours_color, ForcedColor};

    fn check(status: CheckStatus, name: &str) -> ComplianceCheck {
        ComplianceCheck {
            name: name.to_string(),
            status,
            message: "2 issue(s): missing required section: install".to_string(),
            severity: Severity::Warning,
        }
    }

    const ALL: [CheckStatus; 4] = [
        CheckStatus::Pass,
        CheckStatus::Warn,
        CheckStatus::Fail,
        CheckStatus::Skip,
    ];

    /// Every status must move under `--color`, not just the interesting ones.
    #[test]
    fn every_status_line_honours_color() {
        for status in ALL {
            assert_honours_color(&format!("format_asset_check_line({status:?})"), || {
                format_asset_check_line(&check(status, "CB-1320: README Layout Contract"))
            });
        }
    }

    /// The tally under the list is part of the same report surface.
    #[test]
    fn totals_line_honours_color() {
        assert_honours_color("format_asset_check_totals", || {
            format_asset_check_totals(0, 2, 5)
        });
    }

    /// The glyph must come from the shared renderer, not a private copy. If a
    /// second table ever reappears here, it will disagree with this one.
    #[test]
    fn glyphs_come_from_the_shared_check_status_renderer() {
        let _guard = ForcedColor::off();
        for status in ALL {
            let line = format_asset_check_line(&check(status, "CB-1320"));
            let expected = check_status_icon(status);
            assert!(
                line.starts_with(&format!("  {expected} ")),
                "{status:?}: line {line:?} does not start with the shared glyph {expected:?}"
            );
        }
    }

    /// With colour off the line is exactly what it printed before, so the
    /// documented `  ⚠ CB-1320: …` shape is unchanged.
    #[test]
    fn plain_line_shape_is_unchanged() {
        let _guard = ForcedColor::off();
        assert_eq!(
            format_asset_check_line(&check(CheckStatus::Warn, "CB-1320: README Layout Contract")),
            "  ⚠ CB-1320: README Layout Contract: 2 issue(s): missing required section: install"
        );
        assert_eq!(format_asset_check_totals(0, 2, 5), "0 pass, 2 warn, 5 skip");
    }
}

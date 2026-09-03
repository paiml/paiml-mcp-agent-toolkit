// Validation logic for `pmat config --validate`.
//
// CRUX-03 (#1147): the previous version validated whatever `ConfigurationService`
// held — which, for an absent or unparsable `pmat.toml`, was the built-in
// defaults — and printed "Configuration is valid / No issues detected" over a
// file it had never read. Seven different inputs produced byte-identical stdout
// and exit 0. This version answers four separate questions and says which one
// it is answering: did the file load; does every section it declares exist in
// the schema; which keys are honoured and which are read by nothing; and, per
// setting, whether the value came from the file or from the default.

use std::collections::{BTreeMap, BTreeSet};

use crate::services::configuration_service::{
    nearest_known_section, schema_pmat_toml_keys, ConfigLoadStatus, AD_HOC_QUALITY_KEYS,
};

/// Validate configuration
async fn validate_configuration(config_service: &ConfigurationService) -> Result<()> {
    info!("Validating configuration");

    println!("PMAT Configuration Validation");
    println!("{}", "=".repeat(40));
    println!();

    let path = config_service.config_path();
    let raw = match config_service.load_status() {
        ConfigLoadStatus::Unparsable(error) => {
            // A file that did not load is not a file with zero issues. Say
            // which file and where, on stdout, and fail — the stderr warning
            // the service already printed is invisible to a CI job that keeps
            // stdout and the exit code.
            println!("Configuration could not be loaded: {}", path.display());
            println!("   {error}");
            println!();
            println!("   Every setting in that file was replaced by pmat's built-in defaults,");
            println!("   so there is nothing here to certify. Fix the file and re-run.");
            anyhow::bail!("configuration could not be loaded: {}", path.display());
        }
        ConfigLoadStatus::Absent => {
            println!(
                "No configuration file at {} — validating the built-in defaults",
                path.display()
            );
            println!();
            None
        }
        ConfigLoadStatus::Loaded => {
            println!("Configuration source: {}", path.display());
            println!();
            std::fs::read_to_string(path)
                .ok()
                .and_then(|c| c.parse::<toml::Table>().ok())
        }
    };

    let config = config_service.get_config()?;
    let schema = schema_pmat_toml_keys();

    let mut issues: Vec<String> = Vec::new();
    let mut warnings: Vec<String> = Vec::new();

    if let Some(raw) = &raw {
        let (unknown_sections, unknown_keys) = inapplicable_entries(raw, &schema);
        for (section, nearest) in unknown_sections {
            let hint = nearest.map_or_else(String::new, |n| format!(" (did you mean `[{n}]`?)"));
            issues.push(format!(
                "pmat.toml declares `[{section}]`, which no part of pmat reads{hint} — \
                 every setting under it has no effect"
            ));
        }
        for (section, key) in unknown_keys {
            warnings.push(format!(
                "`{key}` in [{section}] — not read: no schema field or known reader consumes it"
            ));
        }
    }

    let mut range_issues: Vec<&'static str> = Vec::new();
    validate_all_sections(&config, &mut range_issues);
    issues.extend(range_issues.into_iter().map(str::to_string));

    report_settings_provenance(&config, raw.as_ref(), &schema);
    report_validation_results(&issues, &warnings)?;
    print_configuration_statistics(&config, raw.as_ref(), &schema);

    Ok(())
}

/// Sections the file declares that the schema does not have (with the nearest
/// known section, if one is close enough to name), and keys under KNOWN
/// sections that neither the schema nor a known ad-hoc reader consumes.
///
/// Pure, so it is tested directly. Unknown sections are fatal — the same
/// verdict `pmat quality-gate` gives them, from the same derived list.
/// Unknown keys are reported, not fatal: the accepted key set is schema plus
/// the readers listed in `AD_HOC_QUALITY_KEYS`, and a reader this list does
/// not know about is possible, so the wording is "not read", not "invalid".
/// `(section, nearest known section)` for a section the schema lacks.
type UnknownSection = (String, Option<String>);
/// `(section, key)` for a key under a known section that nothing reads.
type UnknownKey = (String, String);

fn inapplicable_entries(
    raw: &toml::Table,
    schema: &BTreeMap<String, BTreeSet<String>>,
) -> (Vec<UnknownSection>, Vec<UnknownKey>) {
    let known_sections: BTreeSet<String> = schema.keys().cloned().collect();
    let mut unknown_sections = Vec::new();
    let mut unknown_keys = Vec::new();

    for (section, value) in raw {
        let Some(schema_keys) = schema.get(section) else {
            unknown_sections.push((
                section.clone(),
                nearest_known_section(section, &known_sections),
            ));
            continue;
        };
        // `[custom]` is free-form by design — that is what it is for.
        if section == "custom" {
            continue;
        }
        let toml::Value::Table(keys) = value else {
            continue;
        };
        for key in keys.keys() {
            let ad_hoc = section == "quality" && AD_HOC_QUALITY_KEYS.iter().any(|(k, _)| k == key);
            if !schema_keys.contains(key) && !ad_hoc {
                unknown_keys.push((section.clone(), key.clone()));
            }
        }
    }
    (unknown_sections, unknown_keys)
}

/// Per-setting provenance: for every schema key, the effective value and
/// whether it came from the file or from the built-in default. A section the
/// file omits entirely says so on one line. Ad-hoc `[quality]` keys are shown
/// only when the file sets them — a legend printed regardless would name keys
/// the file never mentions.
fn report_settings_provenance(
    config: &PmatConfig,
    raw: Option<&toml::Table>,
    schema: &BTreeMap<String, BTreeSet<String>>,
) {
    let Ok(toml::Value::Table(effective)) = toml::Value::try_from(config.clone()) else {
        return;
    };
    println!("Settings (value, and where it came from):");
    for (section, keys) in schema {
        if section == "custom" {
            continue;
        }
        let raw_section = raw.and_then(|r| r.get(section)).and_then(|v| v.as_table());
        match raw_section {
            None if raw.is_some() => {
                println!("  [{section}] — not set in pmat.toml; built-in defaults");
                continue;
            }
            None => println!("  [{section}] — built-in defaults"),
            Some(_) => println!("  [{section}]"),
        }
        let effective_section = effective.get(section).and_then(|v| v.as_table());
        for key in keys {
            let value = effective_section
                .and_then(|t| t.get(key))
                .map_or_else(|| "?".to_string(), render_toml_value);
            let origin = if raw_section.is_some_and(|t| t.contains_key(key)) {
                "pmat.toml"
            } else {
                "built-in default"
            };
            println!("    {key} = {value}  ({origin})");
        }
        if section == "quality" {
            if let Some(t) = raw_section {
                for (key, why) in AD_HOC_QUALITY_KEYS {
                    if let Some(v) = t.get(*key) {
                        println!(
                            "    {key} = {}  (pmat.toml; honoured by {why})",
                            render_toml_value(v)
                        );
                    }
                }
            }
        }
    }
    println!();
}

fn render_toml_value(v: &toml::Value) -> String {
    match v {
        toml::Value::String(s) => format!("{s:?}"),
        toml::Value::Table(t) => format!("{{{} keys}}", t.len()),
        toml::Value::Array(a) => format!("[{} items]", a.len()),
        other => other.to_string(),
    }
}

fn validate_all_sections(config: &PmatConfig, issues: &mut Vec<&'static str>) {
    validate_system_config(&config.system, issues);
    validate_quality_config(&config.quality, issues);
    validate_analysis_config(&config.analysis, issues);
    validate_performance_config(&config.performance, issues);
    validate_mcp_config(&config.mcp, issues);
}

fn validate_system_config(
    system_config: &crate::services::configuration_service::SystemConfig,
    issues: &mut Vec<&'static str>,
) {
    if system_config.project_name.is_empty() {
        issues.push("System: project_name cannot be empty");
    }

    if system_config.max_concurrent_operations == 0 {
        issues.push("System: max_concurrent_operations must be > 0");
    }
}

fn validate_quality_config(
    quality_config: &crate::services::configuration_service::QualityConfig,
    issues: &mut Vec<&'static str>,
) {
    if quality_config.max_complexity == 0 {
        issues.push("Quality: max_complexity must be > 0");
    }

    if quality_config.min_coverage > 100.0 || quality_config.min_coverage < 0.0 {
        issues.push("Quality: min_coverage must be between 0 and 100");
    }
}

fn validate_analysis_config(
    analysis_config: &crate::services::configuration_service::AnalysisConfig,
    issues: &mut Vec<&'static str>,
) {
    if analysis_config.max_file_size == 0 {
        issues.push("Analysis: max_file_size must be > 0");
    }

    if analysis_config.timeout_seconds == 0 {
        issues.push("Analysis: timeout_seconds must be > 0");
    }
}

fn validate_performance_config(
    performance_config: &crate::services::configuration_service::PerformanceConfig,
    issues: &mut Vec<&'static str>,
) {
    if performance_config.test_iterations == 0 {
        issues.push("Performance: test_iterations must be > 0");
    }
}

fn validate_mcp_config(
    mcp_config: &crate::services::configuration_service::McpConfig,
    issues: &mut Vec<&'static str>,
) {
    if mcp_config.server_name.is_empty() {
        issues.push("MCP: server_name cannot be empty");
    }

    if mcp_config.request_timeout_seconds == 0 {
        issues.push("MCP: request_timeout_seconds must be > 0");
    }
}

fn report_validation_results(issues: &[String], warnings: &[String]) -> Result<()> {
    if !warnings.is_empty() {
        println!("Settings pmat does not read ({}):", warnings.len());
        for w in warnings {
            println!("   - {w}");
        }
        println!();
    }
    if issues.is_empty() {
        report_validation_success();
        Ok(())
    } else {
        report_validation_failure(issues)
    }
}

fn report_validation_success() {
    println!("Configuration is valid");
    println!("   All settings are within acceptable ranges");
    println!("   No issues detected");
}

fn report_validation_failure(issues: &[String]) -> Result<()> {
    println!("Configuration validation failed");
    println!("   Found {} issues:", issues.len());
    for issue in issues {
        println!("   - {issue}");
    }
    Err(anyhow::anyhow!("Configuration validation failed"))
}

/// Derived from the schema and the file, never literal: the old block printed
/// `Sections: 7` and `Total Settings: ~50` as string constants, for every
/// input, on a schema with nine sections.
fn print_configuration_statistics(
    config: &PmatConfig,
    raw: Option<&toml::Table>,
    schema: &BTreeMap<String, BTreeSet<String>>,
) {
    let settings: usize = schema.values().map(BTreeSet::len).sum();
    let set_in_file: usize = raw.map_or(0, |r| {
        schema
            .iter()
            .filter_map(|(section, keys)| {
                let t = r.get(section)?.as_table()?;
                Some(keys.iter().filter(|k| t.contains_key(*k)).count())
            })
            .sum()
    });
    println!();
    println!("Configuration Statistics:");
    println!("   Sections: {}", schema.len());
    println!("   Settings: {settings}");
    println!("   Set in pmat.toml: {set_in_file}");
    println!("   Custom Settings: {}", config.custom.len());
}

#[cfg(test)]
mod validation_tests {
    use super::*;

    fn schema() -> BTreeMap<String, BTreeSet<String>> {
        schema_pmat_toml_keys()
    }

    #[test]
    fn schema_has_ten_sections_including_hooks_and_custom() {
        let s = schema();
        assert_eq!(s.len(), 10, "{:?}", s.keys().collect::<Vec<_>>());
        for want in [
            "system", "quality", "analysis", "performance", "mcp", "roadmap", "telemetry",
            "semantic", "hooks", "custom",
        ] {
            assert!(s.contains_key(want), "schema lacks [{want}]");
        }
        assert!(s["quality"].contains("max_complexity"));
        // AD-03: `[hooks]` is a section pmat honours, so `config --validate`
        // must know it — otherwise `strict = true` would be "inapplicable".
        assert!(s["hooks"].contains("strict"), "{:?}", s["hooks"]);
        assert!(s["hooks"].contains("ticket_pattern"));
        assert!(s["roadmap"].contains("git"), "nested table is a key of its parent");
    }

    #[test]
    fn unknown_section_is_named_and_near_miss_suggested() {
        let raw: toml::Table = "[quality_gate]\nmax_cyclomatic_complexity = 15\n"
            .parse()
            .expect("fixture TOML parses");
        let (sections, keys) = inapplicable_entries(&raw, &schema());
        assert_eq!(sections, vec![("quality_gate".to_string(), Some("quality".to_string()))]);
        assert!(keys.is_empty());
    }

    #[test]
    fn unrelated_unknown_section_gets_no_suggestion() {
        let raw: toml::Table = "[markdown]\nx = 1\n".parse().expect("fixture TOML parses");
        let (sections, _) = inapplicable_entries(&raw, &schema());
        assert_eq!(sections, vec![("markdown".to_string(), None)]);
    }

    #[test]
    fn renamed_section_with_same_table_count_is_still_caught() {
        let raw: toml::Table = "[telemetryy]\nenabled = true\n".parse().expect("fixture TOML parses");
        let (sections, _) = inapplicable_entries(&raw, &schema());
        assert_eq!(sections, vec![("telemetryy".to_string(), Some("telemetry".to_string()))]);
    }

    #[test]
    fn dead_key_is_distinguished_from_honoured_ad_hoc_key() {
        let raw: toml::Table =
            "[quality]\nmax_complexity = 25\nmin_pattern_diversity = 0.3\nzzz_not_read_by_anything = 1\n"
                .parse()
                .expect("fixture TOML parses");
        let (sections, keys) = inapplicable_entries(&raw, &schema());
        assert!(sections.is_empty());
        assert_eq!(keys, vec![("quality".to_string(), "zzz_not_read_by_anything".to_string())]);
    }

    #[test]
    fn custom_section_is_free_form() {
        let raw: toml::Table = "[custom]\nanything = \"goes\"\n".parse().expect("fixture TOML parses");
        let (sections, keys) = inapplicable_entries(&raw, &schema());
        assert!(sections.is_empty() && keys.is_empty());
    }

    /// The ad-hoc list is a claim about readers elsewhere in the tree. Assert
    /// each key is still consumed by name, so the list cannot outlive them.
    #[test]
    fn ad_hoc_quality_keys_are_still_read() {
        let readers = concat!(
            include_str!("../analysis_utilities/quality_gate_config.rs"),
            include_str!("../analysis_utilities/quality_checks_part1_entropy.rs"),
        );
        for (key, _) in AD_HOC_QUALITY_KEYS {
            assert!(
                readers.contains(&format!("\"{key}\"")),
                "AD_HOC_QUALITY_KEYS names `{key}` but no reader consumes it"
            );
        }
    }
}

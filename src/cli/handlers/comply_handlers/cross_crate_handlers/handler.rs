#![cfg_attr(coverage_nightly, coverage(off))]

use super::baseline::save_ratchet_baseline;
use super::detection_cc001_cc002::{detect_cc001_function_clones, detect_cc002_api_divergence};
use super::detection_cc003_cc004::{
    detect_cc003_primitive_upstream, detect_cc004_churn_correlation,
};
use super::detection_cc005::detect_cc005_example_duplication;
use super::discovery::discover_workspace_crates;
use super::helpers::{
    build_report, is_crate_pair_excluded, is_rule_enabled, load_all_crate_functions,
    parse_rules_filter,
};
use super::output::{format_markdown, format_text};
use super::types::{CrossCrateBaseline, CrossCrateFinding, CrossCrateReport, DetectionConfig};
use crate::cli::commands::ComplyOutputFormat;
use crate::models::comply_config::PmatYamlConfig;
use anyhow::Result;
use std::collections::HashSet;
use std::path::{Path, PathBuf};

// --- Main handler ---

#[allow(clippy::too_many_arguments)]
pub async fn handle_cross_crate(
    workspace_path: &Path,
    explicit_crates: Option<&[PathBuf]>,
    similarity_threshold: f64,
    churn_window_days: u32,
    rules_filter: Option<&str>,
    format: ComplyOutputFormat,
    output: Option<&Path>,
    strict: bool,
    save_baseline: bool,
) -> Result<()> {
    debug_assert!(
        workspace_path.exists(),
        "workspace_path must exist: {}",
        workspace_path.display()
    );
    let yaml_config = PmatYamlConfig::load(workspace_path).unwrap_or_default();
    let det_config = DetectionConfig::from_yaml(&yaml_config.cross_crate);

    eprintln!("Discovering workspace crates...");
    let crates = discover_workspace_crates(workspace_path, explicit_crates);
    if crates.len() < 2 {
        print_discovery_help();
        return Ok(());
    }

    eprintln!("Loading functions from {} crates...", crates.len());
    let crate_functions = load_all_crate_functions(&crates);
    let crate_names: Vec<String> = crate_functions
        .iter()
        .map(|(c, _)| c.name.clone())
        .collect();
    eprintln!(
        "Analyzing {} crates: {}",
        crate_names.len(),
        crate_names.join(", ")
    );

    let enabled_rules = parse_rules_filter(rules_filter);
    let findings = run_detection_rules(
        &crate_functions,
        &enabled_rules,
        &det_config,
        &yaml_config,
        similarity_threshold,
        churn_window_days,
    );
    let report = build_report(findings, crate_names);

    if save_baseline {
        save_ratchet_baseline(&report, workspace_path)?;
    }

    emit_report(&report, format, output)?;
    enforce_strict(&report, strict, workspace_path);

    Ok(())
}

fn print_discovery_help() {
    println!("Cross-crate analysis requires at least 2 crates.");
    println!("Discovery priority:");
    println!("  1. --crates ../foo,../bar  (explicit paths)");
    println!("  2. Cargo.toml [workspace]  (standard Cargo workspace)");
    println!("  3. batuta oracle --local    (batuta stack auto-discovery)");
    println!("  4. .pmat/workspace.toml     (manual siblings config)");
}

fn run_detection_rules(
    crate_functions: &[(
        super::types::CrateInfo,
        Vec<crate::services::agent_context::FunctionEntry>,
    )],
    enabled_rules: &Option<HashSet<String>>,
    det_config: &DetectionConfig,
    yaml_config: &PmatYamlConfig,
    similarity_threshold: f64,
    churn_window_days: u32,
) -> Vec<CrossCrateFinding> {
    let mut findings = Vec::new();

    if is_rule_enabled("cc001", enabled_rules) {
        findings.extend(detect_cc001_function_clones(
            crate_functions,
            similarity_threshold,
            det_config,
        ));
    }
    if is_rule_enabled("cc002", enabled_rules) {
        findings.extend(detect_cc002_api_divergence(crate_functions, det_config));
    }
    if is_rule_enabled("cc003", enabled_rules) {
        findings.extend(detect_cc003_primitive_upstream(crate_functions, det_config));
    }
    if is_rule_enabled("cc004", enabled_rules) {
        findings.extend(detect_cc004_churn_correlation(
            crate_functions,
            churn_window_days,
        ));
    }
    if is_rule_enabled("cc005", enabled_rules) {
        findings.extend(detect_cc005_example_duplication(
            crate_functions,
            similarity_threshold,
        ));
    }

    // Apply suppressions and crate-pair exclusions
    findings.retain(|f| {
        yaml_config
            .comply
            .is_suppressed(&f.rule, &f.file_b)
            .is_none()
    });
    findings.retain(|f| {
        !is_crate_pair_excluded(&f.crate_a, &f.crate_b, &det_config.excluded_crate_pairs)
    });
    findings
}

fn emit_report(
    report: &CrossCrateReport,
    format: ComplyOutputFormat,
    output: Option<&Path>,
) -> Result<()> {
    let output_text = match format {
        ComplyOutputFormat::Text => format_text(report),
        ComplyOutputFormat::Json | ComplyOutputFormat::Sarif => {
            serde_json::to_string_pretty(report)?
        }
        ComplyOutputFormat::Markdown => format_markdown(report),
    };

    if let Some(path) = output {
        std::fs::write(path, &output_text)?;
        eprintln!("Report written to {}", path.display());
    } else {
        println!("{output_text}");
    }
    Ok(())
}

fn enforce_strict(report: &CrossCrateReport, strict: bool, workspace_path: &Path) {
    debug_assert!(
        workspace_path.exists(),
        "workspace_path must exist: {}",
        workspace_path.display()
    );
    if !strict {
        return;
    }

    match CrossCrateBaseline::load(workspace_path) {
        Some(baseline) => {
            let violations = baseline.check_ratchet(report);
            if violations.is_empty() {
                eprintln!("Ratchet check passed (no rule count increased)");
                return;
            }
            eprintln!("\nRatchet violations (finding count increased):");
            for (rule, old, new) in &violations {
                eprintln!("  {}: {} -> {} (+{})", rule, old, new, new - old);
            }
            std::process::exit(1);
        }
        None if report.summary.total_findings > 0 => std::process::exit(1),
        None => {}
    }
}

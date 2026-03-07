#![cfg_attr(coverage_nightly, coverage(off))]

// Output formatters: text, markdown, JSON

use super::types::{CcSeverity, CrossCrateFinding, CrossCrateReport};
use std::collections::HashMap;

pub(super) fn format_text(report: &CrossCrateReport) -> String {
    let mut out = String::new();

    out.push_str("\n\x1b[1mCross-Crate Duplication Report\x1b[0m\n");
    out.push_str(&format!(
        "Crates analyzed: {}\n\n",
        report.crates_analyzed.join(", ")
    ));

    if report.findings.is_empty() {
        out.push_str("\x1b[32mNo cross-crate duplication findings.\x1b[0m\n");
        return out;
    }

    // Group findings by rule
    let mut by_rule: HashMap<&str, Vec<&CrossCrateFinding>> = HashMap::new();
    for f in &report.findings {
        by_rule.entry(&f.rule).or_default().push(f);
    }

    let rule_order = ["CC-001", "CC-002", "CC-003", "CC-004", "CC-005"];

    for rule in &rule_order {
        if let Some(rule_findings) = by_rule.get(rule) {
            let icon = match *rule {
                "CC-001" => "\x1b[31m[CC-001 Clone]\x1b[0m",
                "CC-002" => "\x1b[33m[CC-002 Diverge]\x1b[0m",
                "CC-003" => "\x1b[33m[CC-003 Upstream]\x1b[0m",
                "CC-004" => "\x1b[36m[CC-004 Churn]\x1b[0m",
                "CC-005" => "\x1b[36m[CC-005 Example]\x1b[0m",
                _ => rule,
            };

            out.push_str(&format!("{} ({} findings)\n", icon, rule_findings.len()));

            for f in rule_findings.iter().take(20) {
                let sim_str = f
                    .similarity
                    .map(|s| format!(" ({:.0}%)", s * 100.0))
                    .unwrap_or_default();
                let severity_icon = match f.severity {
                    CcSeverity::Error => "\x1b[31m✗\x1b[0m",
                    CcSeverity::Warning => "\x1b[33m⚠\x1b[0m",
                    CcSeverity::Advisory => "\x1b[36mℹ\x1b[0m",
                };
                out.push_str(&format!(
                    "  {} {}/{}::{} ↔ {}/{}::{}{}\n",
                    severity_icon,
                    f.crate_a,
                    f.file_a,
                    f.function_a,
                    f.crate_b,
                    f.file_b,
                    f.function_b,
                    sim_str
                ));
                out.push_str(&format!("    → {}\n", f.recommendation));
            }

            if rule_findings.len() > 20 {
                out.push_str(&format!("  ... and {} more\n", rule_findings.len() - 20));
            }
            out.push('\n');
        }
    }

    // Summary
    out.push_str(&format!(
        "\x1b[1mSummary:\x1b[0m {} findings ({} errors, {} warnings, {} advisories)\n",
        report.summary.total_findings,
        report.summary.errors,
        report.summary.warnings,
        report.summary.advisories,
    ));

    out
}

pub(super) fn format_markdown(report: &CrossCrateReport) -> String {
    let mut out = String::new();

    out.push_str("# Cross-Crate Duplication Report\n\n");
    out.push_str(&format!(
        "**Crates analyzed:** {}\n\n",
        report.crates_analyzed.join(", ")
    ));

    if report.findings.is_empty() {
        out.push_str("No cross-crate duplication findings.\n");
        return out;
    }

    out.push_str(
        "| Rule | Severity | Crate A | Crate B | Function | Similarity | Recommendation |\n",
    );
    out.push_str(
        "|------|----------|---------|---------|----------|------------|----------------|\n",
    );

    for f in &report.findings {
        let sim_str = f
            .similarity
            .map(|s| format!("{:.0}%", s * 100.0))
            .unwrap_or_else(|| "—".to_string());

        out.push_str(&format!(
            "| {} | {} | {} | {} | {} | {} | {} |\n",
            f.rule, f.severity, f.crate_a, f.crate_b, f.function_a, sim_str, f.recommendation,
        ));
    }

    out.push_str(&format!(
        "\n**Summary:** {} findings ({} errors, {} warnings, {} advisories)\n",
        report.summary.total_findings,
        report.summary.errors,
        report.summary.warnings,
        report.summary.advisories,
    ));

    out
}

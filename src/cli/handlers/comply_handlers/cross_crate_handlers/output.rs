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

#[cfg(test)]
mod cross_crate_output_tests {
    //! Covers format_text + format_markdown in
    //! cross_crate_handlers/output.rs (97 uncov on broad, 0% cov).
    use super::*;
    use crate::cli::handlers::comply_handlers::cross_crate_handlers::types::{
        CcSeverity, CrossCrateFinding, CrossCrateReport, CrossCrateSummary,
    };
    use std::collections::HashMap;

    fn empty_report(crates: Vec<&str>) -> CrossCrateReport {
        CrossCrateReport {
            findings: vec![],
            summary: CrossCrateSummary {
                total_findings: 0,
                errors: 0,
                warnings: 0,
                advisories: 0,
                rules_triggered: HashMap::new(),
            },
            crates_analyzed: crates.into_iter().map(String::from).collect(),
        }
    }

    fn finding(
        rule: &str,
        sev: CcSeverity,
        crate_a: &str,
        crate_b: &str,
        sim: Option<f64>,
    ) -> CrossCrateFinding {
        CrossCrateFinding {
            rule: rule.to_string(),
            severity: sev,
            crate_a: crate_a.to_string(),
            crate_b: crate_b.to_string(),
            function_a: "func_a".to_string(),
            function_b: "func_b".to_string(),
            file_a: "src/a.rs".to_string(),
            file_b: "src/b.rs".to_string(),
            similarity: sim,
            recommendation: "merge".to_string(),
        }
    }

    // ── format_text ──

    #[test]
    fn test_format_text_empty_findings_writes_no_findings_line() {
        let r = empty_report(vec!["foo", "bar"]);
        let out = format_text(&r);
        assert!(out.contains("Cross-Crate Duplication Report"));
        assert!(out.contains("foo, bar"));
        assert!(out.contains("No cross-crate duplication findings"));
    }

    #[test]
    fn test_format_text_single_finding_emits_rule_section() {
        let mut r = empty_report(vec!["a", "b"]);
        r.findings
            .push(finding("CC-001", CcSeverity::Error, "a", "b", Some(0.85)));
        r.summary.total_findings = 1;
        r.summary.errors = 1;
        let out = format_text(&r);
        // Should include the rule + crates + function names.
        assert!(out.contains("CC-001"));
        assert!(out.contains("func_a") || out.contains("func_b"));
    }

    #[test]
    fn test_format_text_findings_grouped_by_rule_in_canonical_order() {
        let mut r = empty_report(vec!["a", "b"]);
        r.findings
            .push(finding("CC-005", CcSeverity::Advisory, "a", "b", None));
        r.findings
            .push(finding("CC-001", CcSeverity::Error, "a", "b", Some(0.95)));
        r.findings
            .push(finding("CC-003", CcSeverity::Warning, "a", "b", Some(0.7)));
        r.summary.total_findings = 3;
        let out = format_text(&r);
        // CC-001 should appear before CC-003 should appear before CC-005.
        let pos_001 = out.find("CC-001").unwrap();
        let pos_003 = out.find("CC-003").unwrap();
        let pos_005 = out.find("CC-005").unwrap();
        assert!(pos_001 < pos_003);
        assert!(pos_003 < pos_005);
    }

    // ── format_markdown ──

    #[test]
    fn test_format_markdown_empty_findings_no_table() {
        let r = empty_report(vec!["x", "y"]);
        let out = format_markdown(&r);
        assert!(out.contains("# Cross-Crate Duplication Report"));
        assert!(out.contains("**Crates analyzed:** x, y"));
        assert!(out.contains("No cross-crate duplication findings"));
        assert!(!out.contains("| Rule |"));
    }

    #[test]
    fn test_format_markdown_with_findings_emits_table() {
        let mut r = empty_report(vec!["a", "b"]);
        r.findings
            .push(finding("CC-001", CcSeverity::Error, "a", "b", Some(0.85)));
        r.summary.total_findings = 1;
        r.summary.errors = 1;
        let out = format_markdown(&r);
        assert!(out.contains("| Rule |"));
        assert!(out.contains("CC-001"));
        // 0.85 → "85%"
        assert!(out.contains("85%"));
        assert!(out.contains("**Summary:**"));
        assert!(out.contains("1 findings"));
    }

    #[test]
    fn test_format_markdown_no_similarity_renders_dash() {
        let mut r = empty_report(vec!["a", "b"]);
        r.findings
            .push(finding("CC-002", CcSeverity::Warning, "a", "b", None));
        r.summary.total_findings = 1;
        r.summary.warnings = 1;
        let out = format_markdown(&r);
        // None similarity → "—" placeholder.
        assert!(out.contains("—"));
    }

    #[test]
    fn test_format_markdown_summary_counts_match_struct() {
        let mut r = empty_report(vec!["a"]);
        r.summary.total_findings = 5;
        r.summary.errors = 2;
        r.summary.warnings = 1;
        r.summary.advisories = 2;
        // Add one dummy finding so the summary section runs.
        r.findings
            .push(finding("CC-001", CcSeverity::Error, "a", "b", None));
        let out = format_markdown(&r);
        assert!(out.contains("5 findings"));
        assert!(out.contains("2 errors"));
        assert!(out.contains("1 warnings"));
        assert!(out.contains("2 advisories"));
    }
}

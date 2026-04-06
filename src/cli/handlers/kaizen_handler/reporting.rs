//! Kaizen reporting: output formatting for text, markdown, and JSON reports.

use super::{FindingSeverity, KaizenConfig, KaizenFinding, KaizenReport};
use crate::cli::commands::KaizenOutputFormat;
use anyhow::{Context, Result};

#[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
pub(crate) fn output_report(report: &KaizenReport, config: &KaizenConfig) -> Result<()> {
    let output_text = format_report(report, config.format);
    if let Some(output_path) = &config.output {
        std::fs::write(output_path, &output_text)
            .with_context(|| format!("Failed to write output to {}", output_path.display()))?;
        eprintln!("Kaizen: report written to {}", output_path.display());
    } else {
        println!("{output_text}");
    }
    Ok(())
}

#[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
pub(crate) fn format_report(report: &KaizenReport, format: KaizenOutputFormat) -> String {
    match format {
        KaizenOutputFormat::Json => serde_json::to_string_pretty(report).unwrap_or_default(),
        KaizenOutputFormat::Markdown => format_report_markdown(report),
        KaizenOutputFormat::Text => format_report_text(report),
    }
}

fn format_finding_text(finding: &KaizenFinding) -> String {
    debug_assert!(true, "contract: format_finding_text");
    let status = if finding.fix_applied {
        "FIXED"
    } else if finding.agent_fixable {
        "AGENT"
    } else {
        "TODO "
    };

    let severity = match finding.severity {
        FindingSeverity::Critical => "CRIT",
        FindingSeverity::High => "HIGH",
        FindingSeverity::Medium => "MED ",
        FindingSeverity::Low => "LOW ",
    };

    let file = finding.file.as_deref().unwrap_or("");
    format!(
        "  [{status}] [{severity}] {category} {file}\n         {msg}\n",
        category = finding.category,
        msg = finding.message,
    )
}

fn format_report_text(report: &KaizenReport) -> String {
    debug_assert!(true, "contract: format_report_text");
    let mut out = String::new();
    let is_cross_stack = !report.crates_scanned.is_empty();

    if is_cross_stack {
        out.push_str("=== Kaizen Cross-Stack Report ===\n\n");
    }

    out.push_str(&format!(
        "Kaizen Report: {} findings | {} auto-fixed | {} agent-fixed | {} remaining | {} issues filed\n",
        report.findings.len(),
        report.auto_fixed_count,
        report.agent_fixed_count,
        report.remaining_count,
        report.issues_created.len(),
    ));

    if is_cross_stack {
        out.push_str(&format!(
            "Crates scanned: {}\n",
            report.crates_scanned.join(", ")
        ));
    }

    if let Some(hash) = &report.commit_hash {
        out.push_str(&format!("Commit: {hash}\n"));
    }
    if report.pushed {
        out.push_str("Pushed: yes\n");
    }

    out.push('\n');

    if is_cross_stack {
        // Group by crate
        for crate_name in &report.crates_scanned {
            let crate_findings: Vec<&KaizenFinding> = report
                .findings
                .iter()
                .filter(|f| f.crate_name.as_deref() == Some(crate_name))
                .collect();

            if crate_findings.is_empty() {
                continue;
            }

            let fixed = crate_findings.iter().filter(|f| f.fix_applied).count();
            out.push_str(&format!(
                "--- {} ({} findings, {} fixed) ---\n",
                crate_name,
                crate_findings.len(),
                fixed
            ));

            for finding in &crate_findings {
                out.push_str(&format_finding_text(finding));
            }
            out.push('\n');
        }

        // Any findings without a crate (shouldn't happen, but be safe)
        let orphans: Vec<&KaizenFinding> = report
            .findings
            .iter()
            .filter(|f| f.crate_name.is_none())
            .collect();
        if !orphans.is_empty() {
            out.push_str(&format!(
                "--- (ungrouped) ({} findings) ---\n",
                orphans.len()
            ));
            for finding in &orphans {
                out.push_str(&format_finding_text(finding));
            }
            out.push('\n');
        }
    } else {
        for finding in &report.findings {
            out.push_str(&format_finding_text(finding));
        }
    }

    if !report.issues_created.is_empty() {
        out.push_str("\nIssues Created:\n");
        for issue in &report.issues_created {
            out.push_str(&format!(
                "  #{} {} ({})\n",
                issue.number, issue.url, issue.finding_category
            ));
        }
    }

    out
}

fn format_finding_md_row(finding: &KaizenFinding) -> String {
    debug_assert!(true, "contract: format_finding_md_row");
    let status = if finding.fix_applied {
        "Fixed"
    } else if finding.agent_fixable {
        "Agent"
    } else {
        "Todo"
    };

    let severity = format!("{:?}", finding.severity);
    let file = finding.file.as_deref().unwrap_or("-");
    let msg = finding.message.replace('|', "\\|");

    format!(
        "| {status} | {severity} | `{}` | `{file}` | {msg} |\n",
        finding.category,
    )
}

fn format_report_markdown(report: &KaizenReport) -> String {
    debug_assert!(true, "contract: format_report_markdown");
    let mut out = String::new();
    let is_cross_stack = !report.crates_scanned.is_empty();

    if is_cross_stack {
        out.push_str("# Kaizen Cross-Stack Report\n\n");
    } else {
        out.push_str("# Kaizen Report\n\n");
    }

    out.push_str(&format!(
        "| Metric | Count |\n|--------|-------|\n\
         | Findings | {} |\n| Auto-fixed | {} |\n| Agent-fixed | {} |\n| Remaining | {} |\n| Issues Filed | {} |\n",
        report.findings.len(),
        report.auto_fixed_count,
        report.agent_fixed_count,
        report.remaining_count,
        report.issues_created.len(),
    ));

    if is_cross_stack {
        out.push_str(&format!("| Crates | {} |\n", report.crates_scanned.len()));
    }
    out.push('\n');

    if let Some(hash) = &report.commit_hash {
        out.push_str(&format!("**Commit**: `{hash}`\n\n"));
    }

    if is_cross_stack {
        for crate_name in &report.crates_scanned {
            let crate_findings: Vec<&KaizenFinding> = report
                .findings
                .iter()
                .filter(|f| f.crate_name.as_deref() == Some(crate_name))
                .collect();

            if crate_findings.is_empty() {
                continue;
            }

            out.push_str(&format!(
                "## {} ({} findings)\n\n",
                crate_name,
                crate_findings.len()
            ));
            out.push_str("| Status | Severity | Category | File | Message |\n");
            out.push_str("|--------|----------|----------|------|---------|\n");
            for finding in &crate_findings {
                out.push_str(&format_finding_md_row(finding));
            }
            out.push('\n');
        }
    } else {
        out.push_str("## Findings\n\n");
        out.push_str("| Status | Severity | Category | File | Message |\n");
        out.push_str("|--------|----------|----------|------|---------|\n");
        for finding in &report.findings {
            out.push_str(&format_finding_md_row(finding));
        }
    }

    if !report.issues_created.is_empty() {
        out.push_str("\n## Issues Created\n\n");
        out.push_str("| # | Category | URL |\n");
        out.push_str("|---|----------|-----|\n");
        for issue in &report.issues_created {
            out.push_str(&format!(
                "| {} | `{}` | {} |\n",
                issue.number, issue.finding_category, issue.url
            ));
        }
    }

    out
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cli::handlers::kaizen_handler::{
        FindingSource, GithubIssueRef, KaizenFinding, KaizenReport,
    };

    #[test]
    fn test_format_report_text() {
        let report = KaizenReport {
            findings: vec![KaizenFinding {
                source: FindingSource::Clippy,
                severity: FindingSeverity::Medium,
                category: "clippy::needless_return".to_string(),
                message: "unnecessary return".to_string(),
                file: Some("src/lib.rs".to_string()),
                auto_fixable: true,
                agent_fixable: false,
                fix_applied: true,
                agent_prompt: None,
                suspiciousness_score: None,
                crate_name: None,
            }],
            auto_fixed_count: 1,
            agent_fixed_count: 0,
            remaining_count: 0,
            issues_created: vec![],
            commit_hash: Some("abc1234".to_string()),
            pushed: false,
            crates_scanned: Vec::new(),
        };
        let text = format_report_text(&report);
        assert!(text.contains("1 findings"));
        assert!(text.contains("1 auto-fixed"));
        assert!(text.contains("FIXED"));
        assert!(text.contains("clippy::needless_return"));
    }

    #[test]
    fn test_format_report_json() {
        let report = KaizenReport {
            findings: vec![],
            auto_fixed_count: 0,
            agent_fixed_count: 0,
            remaining_count: 0,
            issues_created: vec![],
            commit_hash: None,
            pushed: false,
            crates_scanned: Vec::new(),
        };
        let json = format_report(&report, KaizenOutputFormat::Json);
        let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed["auto_fixed_count"], 0);
    }

    #[test]
    fn test_format_report_markdown() {
        let report = KaizenReport {
            findings: vec![KaizenFinding {
                source: FindingSource::GitHubIssue,
                severity: FindingSeverity::High,
                category: "github::issue#42".to_string(),
                message: "Fix the widget".to_string(),
                file: None,
                auto_fixable: false,
                agent_fixable: true,
                fix_applied: false,
                agent_prompt: Some("Fix it".to_string()),
                suspiciousness_score: None,
                crate_name: None,
            }],
            auto_fixed_count: 0,
            agent_fixed_count: 0,
            remaining_count: 1,
            issues_created: vec![],
            commit_hash: None,
            pushed: false,
            crates_scanned: Vec::new(),
        };
        let md = format_report_markdown(&report);
        assert!(md.contains("# Kaizen Report"));
        assert!(md.contains("github::issue#42"));
        assert!(md.contains("Agent"));
    }

    #[test]
    fn test_report_with_issues_created() {
        let report = KaizenReport {
            findings: vec![],
            auto_fixed_count: 0,
            agent_fixed_count: 0,
            remaining_count: 0,
            issues_created: vec![GithubIssueRef {
                number: 42,
                url: "https://github.com/org/repo/issues/42".to_string(),
                finding_category: "comply::CB-200".to_string(),
            }],
            commit_hash: None,
            pushed: false,
            crates_scanned: Vec::new(),
        };

        let text = format_report_text(&report);
        assert!(text.contains("1 issues filed"));
        assert!(text.contains("#42"));
        assert!(text.contains("comply::CB-200"));

        let md = format_report_markdown(&report);
        assert!(md.contains("Issues Filed | 1"));
        assert!(md.contains("## Issues Created"));
    }

    #[test]
    fn test_cross_stack_text_report_groups_by_crate() {
        let report = KaizenReport {
            findings: vec![
                KaizenFinding {
                    source: FindingSource::Clippy,
                    severity: FindingSeverity::Medium,
                    category: "clippy::needless_return".to_string(),
                    message: "unnecessary return".to_string(),
                    file: Some("src/lib.rs".to_string()),
                    auto_fixable: true,
                    agent_fixable: false,
                    fix_applied: true,
                    agent_prompt: None,
                    suspiciousness_score: None,
                    crate_name: Some("pmat".to_string()),
                },
                KaizenFinding {
                    source: FindingSource::Rustfmt,
                    severity: FindingSeverity::Low,
                    category: "rustfmt::unformatted".to_string(),
                    message: "needs formatting".to_string(),
                    file: Some("src/main.rs".to_string()),
                    auto_fixable: true,
                    agent_fixable: false,
                    fix_applied: false,
                    agent_prompt: None,
                    suspiciousness_score: None,
                    crate_name: Some("trueno".to_string()),
                },
            ],
            auto_fixed_count: 1,
            agent_fixed_count: 0,
            remaining_count: 1,
            issues_created: vec![],
            commit_hash: None,
            pushed: false,
            crates_scanned: vec!["pmat".to_string(), "trueno".to_string()],
        };
        let text = format_report_text(&report);
        assert!(text.contains("Cross-Stack"));
        assert!(text.contains("--- pmat ("));
        assert!(text.contains("--- trueno ("));
        assert!(text.contains("clippy::needless_return"));
        assert!(text.contains("rustfmt::unformatted"));
    }

    #[test]
    fn test_cross_stack_markdown_report_groups_by_crate() {
        let report = KaizenReport {
            findings: vec![KaizenFinding {
                source: FindingSource::Defects,
                severity: FindingSeverity::High,
                category: "defect::D001".to_string(),
                message: "unwrap usage".to_string(),
                file: Some("src/tensor.rs".to_string()),
                auto_fixable: false,
                agent_fixable: true,
                fix_applied: false,
                agent_prompt: Some("Fix unwrap".to_string()),
                suspiciousness_score: None,
                crate_name: Some("aprender".to_string()),
            }],
            auto_fixed_count: 0,
            agent_fixed_count: 0,
            remaining_count: 1,
            issues_created: vec![],
            commit_hash: None,
            pushed: false,
            crates_scanned: vec!["pmat".to_string(), "aprender".to_string()],
        };
        let md = format_report_markdown(&report);
        assert!(md.contains("Cross-Stack"));
        assert!(md.contains("## aprender (1 findings)"));
        assert!(md.contains("Crates | 2"));
    }

    #[test]
    fn test_cross_stack_json_includes_crates_scanned() {
        let report = KaizenReport {
            findings: vec![],
            auto_fixed_count: 0,
            agent_fixed_count: 0,
            remaining_count: 0,
            issues_created: vec![],
            commit_hash: None,
            pushed: false,
            crates_scanned: vec!["pmat".to_string(), "trueno".to_string()],
        };
        let json = format_report(&report, KaizenOutputFormat::Json);
        let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed["crates_scanned"][0], "pmat");
        assert_eq!(parsed["crates_scanned"][1], "trueno");
    }
}

#![cfg_attr(coverage_nightly, coverage(off))]
//! Output formatting for defect reports: text, JSON, and JUnit formats

use super::types::DefectReport;
use crate::cli::colors;
use crate::services::defect_detector::{DefectPattern, Severity};
use anyhow::{Context, Result};

#[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
/// Print text report.
pub fn print_text_report(report: &DefectReport) {
    println!("\n{}", colors::header("Known Defects Report"));
    println!("{}", colors::rule());

    println!("\n\u{1f4ca} {}", colors::subheader("Summary"));
    println!(
        "  Total Files Scanned: {}",
        colors::number(&report.summary.total_files_scanned.to_string())
    );
    println!(
        "  Files with Defects: {}",
        colors::number(&report.summary.files_with_defects.to_string())
    );
    println!(
        "  Total Defects: {}",
        colors::number(&report.summary.total_defects.to_string())
    );
    println!(
        "  {}Critical:{} {}",
        colors::RED,
        colors::RESET,
        colors::number(&report.summary.by_severity.critical.to_string())
    );
    println!(
        "  {}High:{} {}",
        colors::BOLD_RED,
        colors::RESET,
        colors::number(&report.summary.by_severity.high.to_string())
    );
    println!(
        "  {}Medium:{} {}",
        colors::YELLOW,
        colors::RESET,
        colors::number(&report.summary.by_severity.medium.to_string())
    );
    println!(
        "  {}Low:{} {}",
        colors::DIM,
        colors::RESET,
        colors::number(&report.summary.by_severity.low.to_string())
    );

    // Group defects by severity
    let critical_defects: Vec<_> = report
        .defects
        .iter()
        .filter(|d| matches!(d.severity, Severity::Critical))
        .collect();
    let high_defects: Vec<_> = report
        .defects
        .iter()
        .filter(|d| matches!(d.severity, Severity::High))
        .collect();
    let medium_defects: Vec<_> = report
        .defects
        .iter()
        .filter(|d| matches!(d.severity, Severity::Medium))
        .collect();

    if !critical_defects.is_empty() {
        println!(
            "\n\u{1f534} {}CRITICAL Defects ({}){}",
            colors::RED,
            critical_defects.len(),
            colors::RESET
        );
        for defect in critical_defects {
            print_defect_pattern(defect);
        }
    }

    if !high_defects.is_empty() {
        println!(
            "\n\u{1f7e0} {}HIGH Defects ({}){}",
            colors::BOLD_RED,
            high_defects.len(),
            colors::RESET
        );
        for defect in high_defects {
            print_defect_pattern(defect);
        }
    }

    if !medium_defects.is_empty() {
        println!(
            "\n\u{1f7e1} {}MEDIUM Defects ({}){}",
            colors::YELLOW,
            medium_defects.len(),
            colors::RESET
        );
        for defect in medium_defects {
            print_defect_pattern(defect);
        }
    }

    println!("\n{}", colors::separator());
    println!(
        "{} {} {}",
        colors::subheader("Exit code:"),
        colors::number(&report.exit_code.to_string()),
        if report.has_critical_defects {
            format!("{}(critical defects found){}", colors::RED, colors::RESET)
        } else {
            format!("{}(no critical defects){}", colors::GREEN, colors::RESET)
        }
    );
}

fn print_defect_pattern(defect: &DefectPattern) {
    let severity_color = match defect.severity {
        Severity::Critical => colors::RED,
        Severity::High => colors::BOLD_RED,
        Severity::Medium => colors::YELLOW,
        Severity::Low => colors::DIM,
    };
    println!(
        "\n  {}{}{}: {} ({} instances)",
        severity_color,
        defect.id,
        colors::RESET,
        colors::label(&defect.name),
        colors::number(&defect.instances.len().to_string())
    );

    // Show first 10 instances
    for (i, instance) in defect.instances.iter().take(10).enumerate() {
        println!(
            "    - {}{}{}:{}",
            colors::CYAN,
            instance.file,
            colors::RESET,
            instance.line
        );
        if i == 9 && defect.instances.len() > 10 {
            println!(
                "    {} ({} more)",
                colors::dim("..."),
                defect.instances.len() - 10
            );
        }
    }

    println!(
        "\n  {} {}",
        colors::label("Fix:"),
        defect.fix_recommendation
    );
    println!(
        "  {} {}",
        colors::label("Evidence:"),
        defect.evidence_description
    );
}

#[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
/// Print json report.
pub fn print_json_report(report: &DefectReport) -> Result<()> {
    let json =
        serde_json::to_string_pretty(report).context("Failed to serialize report to JSON")?;
    println!("{}", json);
    Ok(())
}

#[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
/// Print junit report.
pub fn print_junit_report(report: &DefectReport) -> Result<()> {
    println!("<?xml version=\"1.0\" encoding=\"UTF-8\"?>");
    println!(
        "<testsuites name=\"Known Defects Analysis\" tests=\"{}\" failures=\"{}\" errors=\"0\">",
        report.summary.total_defects, report.summary.total_defects
    );

    for defect in &report.defects {
        println!(
            "  <testsuite name=\"{}\" tests=\"{}\" failures=\"{}\" errors=\"0\">",
            defect.id,
            defect.instances.len(),
            defect.instances.len()
        );

        for instance in &defect.instances {
            println!(
                "    <testcase name=\"{}:{}\" classname=\"{}\">",
                instance.file, instance.line, defect.id
            );
            println!("      <failure message=\"{} detected\">", defect.name);
            println!("File: {}:{}", instance.file, instance.line);
            println!("Pattern: {}", defect.id);
            println!("Severity: {}", defect.severity.as_str());
            println!("Evidence: {}", defect.evidence_description);
            println!("Fix: {}", defect.fix_recommendation);
            println!("      </failure>");
            println!("    </testcase>");
        }

        println!("  </testsuite>");
    }

    println!("</testsuites>");
    Ok(())
}

#![cfg_attr(coverage_nightly, coverage(off))]
//! Output formatting for defect reports: text, JSON, and JUnit formats

use super::types::DefectReport;
use crate::services::defect_detector::{DefectPattern, Severity};
use anyhow::{Context, Result};

pub fn print_text_report(report: &DefectReport) {
    println!("\nKnown Defects Report");
    println!("====================");

    println!("\n\u{1f4ca} Summary");
    println!(
        "  Total Files Scanned: {}",
        report.summary.total_files_scanned
    );
    println!(
        "  Files with Defects: {}",
        report.summary.files_with_defects
    );
    println!("  Total Defects: {}", report.summary.total_defects);
    println!("  Critical: {}", report.summary.by_severity.critical);
    println!("  High: {}", report.summary.by_severity.high);
    println!("  Medium: {}", report.summary.by_severity.medium);
    println!("  Low: {}", report.summary.by_severity.low);

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
        println!("\n\u{1f534} CRITICAL Defects ({})", critical_defects.len());
        for defect in critical_defects {
            print_defect_pattern(defect);
        }
    }

    if !high_defects.is_empty() {
        println!("\n\u{1f7e0} HIGH Defects ({})", high_defects.len());
        for defect in high_defects {
            print_defect_pattern(defect);
        }
    }

    if !medium_defects.is_empty() {
        println!("\n\u{1f7e1} MEDIUM Defects ({})", medium_defects.len());
        for defect in medium_defects {
            print_defect_pattern(defect);
        }
    }

    println!(
        "\nExit code: {} {}",
        report.exit_code,
        if report.has_critical_defects {
            "(critical defects found)"
        } else {
            "(no critical defects)"
        }
    );
}

fn print_defect_pattern(defect: &DefectPattern) {
    println!(
        "\n  {}: {} ({} instances)",
        defect.id,
        defect.name,
        defect.instances.len()
    );

    // Show first 10 instances
    for (i, instance) in defect.instances.iter().take(10).enumerate() {
        println!("    - {}:{}", instance.file, instance.line);
        if i == 9 && defect.instances.len() > 10 {
            println!("    ... ({} more)", defect.instances.len() - 10);
        }
    }

    println!("\n  Fix: {}", defect.fix_recommendation);
    println!("  Evidence: {}", defect.evidence_description);
}

pub fn print_json_report(report: &DefectReport) -> Result<()> {
    let json =
        serde_json::to_string_pretty(report).context("Failed to serialize report to JSON")?;
    println!("{}", json);
    Ok(())
}

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

//! CLI handler for `pmat analyze defects` command
//!
//! Scans projects for known defect patterns with text, JSON, and JUnit output formats
//! for CI/CD integration. Based on docs/issues/analyze-defects-command.md

use crate::services::defect_detector::{DefectPattern, RustDefectDetector, Severity};
use anyhow::{Context, Result};
use console::style;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;
use walkdir::WalkDir;

#[derive(Debug, Clone, Copy)]
pub enum OutputFormat {
    Text,
    Json,
    Junit,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DefectSummary {
    pub total_files_scanned: usize,
    pub files_with_defects: usize,
    pub total_defects: usize,
    pub by_severity: SeverityCount,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SeverityCount {
    pub critical: usize,
    pub high: usize,
    pub medium: usize,
    pub low: usize,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DefectReport {
    pub summary: DefectSummary,
    pub defects: Vec<DefectPattern>,
    pub exit_code: i32,
    pub has_critical_defects: bool,
}

/// Handle the `pmat analyze defects` command
pub async fn handle_analyze_defects(
    path: Option<&Path>,
    file: Option<&Path>,
    severity_filter: Option<Severity>,
    format: OutputFormat,
) -> Result<i32> {
    let detector = RustDefectDetector::new();

    let target_path = path.unwrap_or_else(|| Path::new("."));

    // Collect all Rust files to scan
    let files_to_scan = if let Some(specific_file) = file {
        vec![specific_file.to_path_buf()]
    } else {
        collect_rust_files(target_path)?
    };

    // Scan all files for defects
    let mut all_defects = Vec::new();
    let mut files_with_defects = 0;

    for file_path in &files_to_scan {
        if let Ok(content) = fs::read_to_string(file_path) {
            let defects = detector.detect(&content, file_path);
            if !defects.is_empty() {
                files_with_defects += 1;
                all_defects.extend(defects);
            }
        }
    }

    // Apply severity filter if specified
    if let Some(filter_severity) = severity_filter {
        all_defects.retain(|d| d.severity == filter_severity);
    }

    // Calculate summary
    let summary = calculate_summary(&files_to_scan, files_with_defects, &all_defects);
    let has_critical = all_defects
        .iter()
        .any(|d| matches!(d.severity, Severity::Critical));
    let exit_code = if has_critical { 1 } else { 0 };

    let report = DefectReport {
        summary,
        defects: all_defects,
        exit_code,
        has_critical_defects: has_critical,
    };

    // Output in requested format
    match format {
        OutputFormat::Text => print_text_report(&report),
        OutputFormat::Json => print_json_report(&report)?,
        OutputFormat::Junit => print_junit_report(&report)?,
    }

    Ok(exit_code)
}

fn collect_rust_files(path: &Path) -> Result<Vec<std::path::PathBuf>> {
    let mut files = Vec::new();

    for entry in WalkDir::new(path)
        .into_iter()
        .filter_entry(|e| !is_hidden(e))
        .filter_map(|e| e.ok())
    {
        let path = entry.path();
        if path.is_file() && path.extension().is_some_and(|ext| ext == "rs") {
            files.push(path.to_path_buf());
        }
    }

    Ok(files)
}

fn is_hidden(entry: &walkdir::DirEntry) -> bool {
    entry
        .file_name()
        .to_str()
        .map(|s| s.starts_with('.'))
        .unwrap_or(false)
        || entry.file_name() == "target"
}

fn calculate_summary(
    files: &[std::path::PathBuf],
    files_with_defects: usize,
    defects: &[DefectPattern],
) -> DefectSummary {
    let mut critical = 0;
    let mut high = 0;
    let mut medium = 0;
    let mut low = 0;

    for defect in defects {
        match defect.severity {
            Severity::Critical => critical += defect.instances.len(),
            Severity::High => high += defect.instances.len(),
            Severity::Medium => medium += defect.instances.len(),
            Severity::Low => low += defect.instances.len(),
        }
    }

    DefectSummary {
        total_files_scanned: files.len(),
        files_with_defects,
        total_defects: critical + high + medium + low,
        by_severity: SeverityCount {
            critical,
            high,
            medium,
            low,
        },
    }
}

fn print_text_report(report: &DefectReport) {
    println!("\n{}", style("Known Defects Report").bold());
    println!("{}", style("====================").bold());

    println!("\n📊 {}", style("Summary").bold());
    println!(
        "  Total Files Scanned: {}",
        report.summary.total_files_scanned
    );
    println!(
        "  Files with Defects: {}",
        report.summary.files_with_defects
    );
    println!("  Total Defects: {}", report.summary.total_defects);
    println!(
        "  Critical: {}",
        style(report.summary.by_severity.critical).red().bold()
    );
    println!(
        "  High: {}",
        style(report.summary.by_severity.high).yellow()
    );
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
        println!(
            "\n🔴 {} ({})",
            style("CRITICAL Defects").red().bold(),
            critical_defects.len()
        );
        for defect in critical_defects {
            print_defect_pattern(defect);
        }
    }

    if !high_defects.is_empty() {
        println!(
            "\n🟠 {} ({})",
            style("HIGH Defects").yellow().bold(),
            high_defects.len()
        );
        for defect in high_defects {
            print_defect_pattern(defect);
        }
    }

    if !medium_defects.is_empty() {
        println!("\n🟡 MEDIUM Defects ({})", medium_defects.len());
        for defect in medium_defects {
            print_defect_pattern(defect);
        }
    }

    println!(
        "\nExit code: {} {}",
        report.exit_code,
        if report.has_critical_defects {
            style("(critical defects found)").red()
        } else {
            style("(no critical defects)").green()
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

fn print_json_report(report: &DefectReport) -> Result<()> {
    let json =
        serde_json::to_string_pretty(report).context("Failed to serialize report to JSON")?;
    println!("{}", json);
    Ok(())
}

fn print_junit_report(report: &DefectReport) -> Result<()> {
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

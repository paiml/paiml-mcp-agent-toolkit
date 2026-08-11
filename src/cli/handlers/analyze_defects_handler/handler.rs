#![cfg_attr(coverage_nightly, coverage(off))]
//! Core handler logic for `pmat analyze defects` command

use super::output::{print_json_report, print_junit_report, print_text_report};
use super::types::{DefectReport, DefectSummary, OutputFormat, SeverityCount};
use crate::services::defect_detector::{DefectPattern, RustDefectDetector, Severity};
use anyhow::Result;
use std::fs;
use std::path::Path;
use walkdir::WalkDir;

/// Handle the `pmat analyze defects` command
#[provable_contracts_macros::contract("pmat-core.yaml", equation = "path_exists")]
pub async fn handle_analyze_defects(
    path: Option<&Path>,
    file: Option<&Path>,
    severity_filter: Option<Severity>,
    format: OutputFormat,
) -> Result<i32> {
    let detector = RustDefectDetector::new();

    let target_path = path.unwrap_or_else(|| Path::new("."));

    // GH-666: a nonexistent `--path` walked to zero files and printed
    // "Total Files Scanned: 0 ... Exit code: 0 (no critical defects)".
    crate::cli::ensure_analysis_path_exists(target_path)?;

    // GH-664: a nonexistent `--file` was pushed into the scan list unread, so
    // the read silently failed and the summary still claimed
    // "Total Files Scanned: 1" (and `total_files_scanned: 1` in JSON) for a file
    // that does not exist. `analyze complexity --file` already errored here.
    if let Some(specific_file) = file {
        crate::cli::ensure_analysis_path_exists(specific_file)?;
    }

    // Collect all Rust files to scan
    let files_to_scan = if let Some(specific_file) = file {
        vec![specific_file.to_path_buf()]
    } else {
        collect_rust_files(target_path)?
    };

    // Scan all files for defects
    let mut all_defects = Vec::new();

    for file_path in &files_to_scan {
        if let Ok(content) = fs::read_to_string(file_path) {
            all_defects.extend(detector.detect(&content, file_path));
        }
    }

    // Apply severity filter if specified
    if let Some(filter_severity) = severity_filter {
        all_defects.retain(|d| d.severity == filter_severity);
    }

    // Calculate summary
    let summary = calculate_summary(&files_to_scan, &all_defects);
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
        OutputFormat::Text | OutputFormat::Plain => print_text_report(&report),
        OutputFormat::Json => print_json_report(&report)?,
        OutputFormat::Junit => print_junit_report(&report)?,
        _ => print_text_report(&report),
    }

    Ok(exit_code)
}

#[provable_contracts_macros::contract("pmat-core.yaml", equation = "path_exists")]
pub(crate) fn collect_rust_files(path: &Path) -> Result<Vec<std::path::PathBuf>> {
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

#[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
pub(crate) fn is_hidden(entry: &walkdir::DirEntry) -> bool {
    // Never filter out the root entry (depth 0) — fixes `--path .` scanning 0 files
    if entry.depth() == 0 {
        return false;
    }
    entry
        .file_name()
        .to_str()
        .map(|s| s.starts_with('.'))
        .unwrap_or(false)
        || entry.file_name() == "target"
}

#[provable_contracts_macros::contract("pmat-core.yaml", equation = "path_exists")]
pub(crate) fn calculate_summary(
    files: &[std::path::PathBuf],
    defects: &[DefectPattern],
) -> DefectSummary {
    // `files_with_defects` used to be passed in, tallied while scanning — i.e.
    // BEFORE the severity filter ran — so `--severity low` reported
    // `files_with_defects: 22` above `total_defects: 0` and an empty `defects`
    // array. Deriving it from the defects being summarized makes a count that
    // disagrees with the list it heads impossible to construct.
    let files_with_defects = defects
        .iter()
        .flat_map(|d| d.instances.iter().map(|i| i.file.as_str()))
        .collect::<std::collections::BTreeSet<_>>()
        .len();

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

#![cfg_attr(coverage_nightly, coverage(off))]
//! Core handler logic for `pmat analyze defects` command

use super::output::{print_json_report, print_junit_report, print_text_report};
use super::types::{DefectReport, DefectSummary, OutputFormat, SeverityCount};
use crate::services::defect_detector::{
    detect_defects, exclusion_reason, is_supported, unmeasured, DefectPattern, Severity,
    SUPPORTED_EXTENSIONS,
};
use anyhow::Result;
use std::collections::BTreeMap;
use std::fs;
use std::path::Path;
use walkdir::WalkDir;

/// Exit code for "the walk took no measurement" — `ExitCode::AnalysisError`
/// in `src/bin/pmat.rs`, the same code `analyze satd` ends up at for the same
/// event.
///
/// Returned as a code rather than raised as an error on purpose: the binary
/// derives an exit code from an error by *substring-matching its message*
/// (`categorize_error`: an error is an "analysis" error iff its text contains
/// the word "analysis"), so an `Err` would make this command's exit code a
/// property of its prose. `analyze satd` reaches 5 only because its remedy
/// sentence happens to say "point the analysis at…", and reaches 1 on its
/// other refusal branch, which says nothing of the sort.
pub(crate) const EXIT_NOTHING_MEASURED: i32 = 5;

/// Handle the `pmat analyze defects` command
#[provable_contracts_macros::contract("pmat-core.yaml", equation = "path_exists")]
pub async fn handle_analyze_defects(
    path: Option<&Path>,
    file: Option<&Path>,
    severity_filter: Option<Severity>,
    format: OutputFormat,
) -> Result<i32> {
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

    // Collect every file pmat has a rule set for — #926: this walk kept
    // `ext == "rs"` while the database also defines Lua, Python and
    // TypeScript rules, so the four rule sets that emit anything other than
    // `Critical` were unreachable from this command.
    let files_to_scan = if let Some(specific_file) = file {
        vec![specific_file.to_path_buf()]
    } else {
        collect_source_files(target_path)?
    };

    // Scan all files for defects
    let (mut all_defects, scan) = scan_files(&files_to_scan);

    // #923: a walk that analysed nothing has produced no result to report.
    // "Every candidate was excluded" used to arrive here as the same empty
    // vector as "the code is clean" and be rendered as the same report with
    // exit 0 — `-p <repo>/examples` printed `total_files_scanned: 117,
    // critical: 0` over 117 files holding 32 `.unwrap()` calls. `analyze satd`
    // already refuses this; so does this command now, sharing the wording via
    // `unmeasured::refusal`.
    if scan.analysed == 0 {
        eprintln!(
            "Error: {}",
            unmeasured::refusal(
                "defect",
                // `--file` conflicts with `--path`, so naming the default "."
                // here would name a directory the user never asked about.
                file.unwrap_or(target_path),
                files_to_scan.len(),
                &scan.describe_skips(),
                &scan.remedy(),
            )
        );
        return Ok(EXIT_NOTHING_MEASURED);
    }

    // Apply severity filter if specified
    if let Some(filter_severity) = severity_filter {
        all_defects.retain(|d| d.severity == filter_severity);
    }

    // Calculate summary
    let summary = calculate_summary(scan.analysed, &all_defects);
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

/// Grade every file the walk found, keeping WHAT WAS ACTUALLY MEASURED apart
/// from what was merely discovered.
///
/// Separated from the handler so the count that reaches
/// `summary.total_files_scanned` is testable without capturing stdout.
pub(crate) fn scan_files(files: &[std::path::PathBuf]) -> (Vec<DefectPattern>, ScanTally) {
    let mut defects = Vec::new();
    let mut scan = ScanTally::default();

    for file_path in files {
        // The exclusion rule for whichever rule set grades this file, asked
        // BEFORE reading, so a file that would have been discarded is never
        // counted as scanned. `detect` asks the same question again for
        // callers that do not (new_tdg_handler).
        //
        // #926: this used to be `RustDefectDetector::exclusion_reason`, which
        // answers `None` — "measure it" — for a `.go`, `.lua` or `.py` file it
        // has no rules for. `--file bad.lua` was therefore READ, graded by a
        // Rust regex that found nothing in it, and reported as
        // `total_files_scanned: 1, total_defects: 0, exit 0`, while
        // `analyze tdg` graded the same bytes F on 15 critical defects.
        if let Some(reason) = exclusion_reason(file_path) {
            *scan.skipped.entry(reason).or_insert(0) += 1;
            continue;
        }
        match fs::read_to_string(file_path) {
            Ok(content) => {
                scan.analysed += 1;
                defects.extend(detect_defects(&content, file_path));
            }
            // Unreadable: permissions, a broken symlink, non-UTF-8. The file
            // was not measured either, and used to be counted as if it were.
            Err(_) => {
                *scan
                    .skipped
                    .entry(unmeasured::Reason::Unreadable)
                    .or_insert(0) += 1
            }
        }
    }

    (defects, scan)
}

/// What the walk actually did, file by file.
///
/// `analysed` is the only number that may be reported as "files scanned": a
/// file the detector excluded, or could not read, contributed no measurement
/// and counting it manufactures one.
#[derive(Debug, Default)]
pub(crate) struct ScanTally {
    pub(crate) analysed: usize,
    pub(crate) skipped: BTreeMap<unmeasured::Reason, usize>,
}

impl ScanTally {
    /// The skipped files, grouped by reason, for the refusal message — so a
    /// user is told WHICH rule swallowed the walk and can act on it, rather
    /// than being handed a bare "0 defects".
    pub(crate) fn describe_skips(&self) -> String {
        let parts: Vec<String> = self
            .skipped
            .iter()
            .map(|(reason, count)| format!("{count} {}", reason.as_str()))
            .collect();
        if parts.is_empty() {
            // Unreachable while `analysed == 0` and at least one file was
            // discovered, but "nothing to say" must not read as "nothing
            // happened".
            return "no reason recorded".to_string();
        }
        parts.join(", ")
    }

    /// What the user can actually do about THIS refusal.
    ///
    /// A remedy that does not match the reason is worse than none: telling
    /// someone who pointed the command at a directory of Go files to "point it
    /// at the project root" sends them to re-run a scan that will refuse
    /// again for the same reason. So the sentence is chosen from what the walk
    /// actually recorded.
    pub(crate) fn remedy(&self) -> String {
        let only_unsupported = !self.skipped.is_empty()
            && self
                .skipped
                .keys()
                .all(|reason| *reason == unmeasured::Reason::NoRuleSet);
        if only_unsupported {
            return format!(
                "pmat's Known-Defects database has rule sets for {} files only, so there is \
                 nothing it can say about these; run `pmat analyze complexity` or `pmat analyze \
                 satd`, which are language-agnostic.",
                SUPPORTED_EXTENSIONS.join(", ")
            );
        }
        NON_PRODUCTION_REMEDY.to_string()
    }
}

/// What a user can actually do about it. Deliberately does NOT name a flag:
/// `analyze defects` has no include/exclude switch, and advising one that does
/// not exist is the defect this message is here to remove.
const NON_PRODUCTION_REMEDY: &str =
    "point the analysis at the project root, where the production code this \
     command measures lives (a package's tests/, benches/, examples/ and fuzz/ \
     trees are never measured, and there is no flag to opt them in).";

/// Every file under `path` that some rule set can grade.
///
/// #926: this was `collect_rust_files`, keeping `ext == "rs"`, while the
/// handler separately constructed a `RustDefectDetector` — the same "which
/// language is this?" decision made twice, in two places, in two different
/// ways. It is now asked once, of [`is_supported`], which is the same table
/// [`detect_defects`] dispatches on, so the walk cannot gather a file nothing
/// can grade nor skip one something could.
#[provable_contracts_macros::contract("pmat-core.yaml", equation = "path_exists")]
pub(crate) fn collect_source_files(path: &Path) -> Result<Vec<std::path::PathBuf>> {
    let mut files = Vec::new();

    for entry in WalkDir::new(path)
        .into_iter()
        .filter_entry(|e| !is_hidden(e))
        .filter_map(|e| e.ok())
    {
        let path = entry.path();
        if path.is_file() && is_supported(path) {
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

/// Summarise `defects` over the `files_analysed` files that produced them.
///
/// #923: the count used to be `files_to_scan.len()` — every `.rs` file the
/// walk *discovered*, including the ones the detector excluded unread. So
/// `-p <repo>/examples` reported `total_files_scanned: 117` for a run that
/// examined none of them: a count of files whose findings were all suppressed
/// is not a count of files analysed.
#[provable_contracts_macros::contract("pmat-core.yaml", equation = "path_exists")]
pub(crate) fn calculate_summary(files_analysed: usize, defects: &[DefectPattern]) -> DefectSummary {
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
        total_files_scanned: files_analysed,
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

// Dead code analysis logic - included from dead_code_handlers.rs
// NO `use` imports or `#!` inner attributes allowed here.

/// Dead code analysis output: the report, plus the one figure that describes
/// the PROJECT rather than the reported list.
///
/// `report.summary.dead_percentage` is scoped to the files actually listed —
/// it has to be, a count must agree with the list it heads — so it is not a
/// number a threshold can be enforced against: it shrinks as `--top-files`
/// asks for fewer files. `project_dead_percentage` is measured over every line
/// walked, before `--min-dead-lines` and `--top-files` cut the list down, and
/// is `None` when the analyzer cannot measure one.
struct DeadCodeAnalysisOutcome {
    report: crate::models::dead_code::DeadCodeResult,
    project_dead_percentage: Option<f32>,
    scope: DeadCodeReportScope,
}

/// What the summary renderer needs in order to describe its own scope, and
/// which `DeadCodeResult` itself does not carry.
///
/// Every field exists because the report made a claim it could not stand
/// behind: the skipped-file line named "tests, examples and benches" on a scan
/// that skips only tests, the omission line blamed `--min-dead-lines` for files
/// `--exclude` had removed, and the percentage was printed unqualified next to a
/// gate that then refused to measure one.
#[derive(Clone, Copy, Default)]
struct DeadCodeReportScope {
    /// Dead-code percentage over every line the analyzer walked, `None` when it
    /// cannot measure one. The summary's own percentage covers only the files it
    /// LISTS, so unqualified it read as a project figure.
    project_dead_percentage: Option<f32>,
    /// What the `total_files - analyzed_files` difference is, in the analyzer's
    /// own terms. `None` when the analyzer did not say.
    skipped_kind: Option<&'static str>,
    /// True when `--include`/`--exclude` removed files from the reported list.
    /// They filter the REPORT, not the scan.
    list_filtered: bool,
}

/// Run dead code analysis with include/exclude filters
async fn run_dead_code_analysis_with_filters(
    path: &Path,
    filters: DeadCodeAnalysisFilters,
) -> Result<DeadCodeAnalysisOutcome> {
    use crate::models::dead_code::DeadCodeAnalysisConfig;
    use crate::utils::file_filter::FileFilter;

    // Detect project language to choose the right analyzer
    let detection =
        crate::services::enhanced_language_detection::detect_project_language_enhanced(path);

    // For non-Rust projects, use the multi-language analyzer
    if detection.language != "rust" {
        return run_multi_language_dead_code(path, &filters, &detection.language);
    }

    // Create file filter
    let filter = FileFilter::new(filters.include, filters.exclude)?;

    // Use the accurate cargo-based analyzer for Rust projects
    use crate::services::cargo_dead_code_analyzer::CargoDeadCodeAnalyzer;
    let cargo_analyzer = if filters.include_tests {
        CargoDeadCodeAnalyzer::new(path)
            .include_tests()
            .with_max_depth(filters.max_depth)
    } else {
        CargoDeadCodeAnalyzer::new(path).with_max_depth(filters.max_depth)
    };

    // Run cargo-based analysis for accurate results
    let accurate_report = cargo_analyzer.analyze().await?;

    // Create config for the result
    let config = DeadCodeAnalysisConfig {
        include_unreachable: filters.include_unreachable,
        include_tests: filters.include_tests,
        min_dead_lines: filters.min_dead_lines,
    };

    // Convert cargo report to ranking format for compatibility.
    //
    // `files_with_dead_code_found` is every file cargo flagged. It is NOT the
    // count that heads the emitted list: `--min-dead-lines` (default 10) and
    // `--top-files` cut that list down, and the summary used to keep reporting
    // the pre-filter number — 26 files claimed above a 4-entry array.
    let files_with_dead_code_found = accurate_report.files_with_dead_code.len();
    let project_total_lines = accurate_report.total_lines;
    // Every .rs file in the project, against the subset actually scanned. A
    // cache entry written before `project_files` existed deserialises it as 0,
    // so fall back to the scanned count rather than claim a project smaller
    // than the scan.
    let project_files = accurate_report
        .project_files
        .max(accurate_report.total_files);
    // Measured over every line the analyzer walked, before any filter. This is
    // the only figure `--fail-on-violation` may compare against; the summary's
    // is scoped to the list that survived `--top-files`/`--min-dead-lines`.
    #[allow(clippy::cast_possible_truncation)]
    let project_dead_percentage = Some(accurate_report.dead_code_percentage as f32);
    let mut analysis_result =
        create_dead_code_ranking_result(accurate_report, filters.min_dead_lines, config);

    // Apply file filter to results if filters are active.
    //
    // This filters the REPORTED LIST, not the scan: `analyzed_files` and the
    // project-wide percentage still cover every file cargo walked. The report
    // says so rather than letting `--include 'examples/**'` on a two-file crate
    // print "Files analyzed: 2" as though the filter had narrowed the scan.
    let list_filtered = filter.has_filters();
    if list_filtered {
        analysis_result.ranked_files.retain(|file| {
            let path = std::path::Path::new(&file.path);
            filter.should_include(path)
        });
    }

    // Apply top_files limit if specified
    let mut files_truncated = false;
    if let Some(limit) = filters.top_files {
        if limit > 0 && analysis_result.ranked_files.len() > limit {
            analysis_result.ranked_files.truncate(limit);
            files_truncated = true;
        }
    }

    // The summary must describe the list it heads, always — after every filter
    // and after truncation.
    resummarize_from_listed_files(
        &mut analysis_result.summary,
        &analysis_result.ranked_files,
        project_total_lines,
    );

    // Convert to DeadCodeResult
    Ok(DeadCodeAnalysisOutcome {
        report: crate::models::dead_code::DeadCodeResult {
            summary: analysis_result.summary.clone(),
            files: analysis_result.ranked_files,
            // `total_files` is the project; `analyzed_files` is what was read.
            // They differ whenever a tree was excluded, which is what makes the
            // narrowing visible to a reader -- and to a CI gate -- instead of
            // the report reading as a clean bill of health over everything.
            total_files: project_files,
            analyzed_files: analysis_result.summary.total_files_analyzed,
            files_with_dead_code_found,
            files_truncated,
        },
        project_dead_percentage,
        scope: DeadCodeReportScope {
            project_dead_percentage,
            // Only the test tree is out of scope: `examples/` and `benches/`
            // are scanned with or without `--include-tests` (see
            // `CargoDeadCodeAnalyzer::should_analyze`). The line used to name
            // all three, directly above a Top Files list made of `examples/`.
            skipped_kind: Some("test code; --include-tests scans it too"),
            list_filtered,
        },
    })
}

/// Recompute every summary figure from the files that are actually listed.
///
/// A count must agree with the list it heads (`pmat-no-fabrication-v1`). Before
/// this, `files_with_dead_code` was the pre-filter count (26 over a 4-entry
/// array, and 1 over an EMPTY array on a small fixture) and `total_dead_lines`
/// came from a different estimator than the rows (94 vs a row sum of 76).
fn resummarize_from_listed_files(
    summary: &mut crate::models::dead_code::DeadCodeSummary,
    files: &[crate::models::dead_code::FileDeadCodeMetrics],
    project_total_lines: usize,
) {
    summary.files_with_dead_code = files.len();
    summary.total_dead_lines = files.iter().map(|f| f.dead_lines).sum();
    summary.dead_functions = files.iter().map(|f| f.dead_functions).sum();
    summary.dead_classes = files.iter().map(|f| f.dead_classes).sum();
    summary.dead_modules = files.iter().map(|f| f.dead_modules).sum();
    summary.unreachable_blocks = files.iter().map(|f| f.unreachable_blocks).sum();
    summary.dead_percentage = if project_total_lines > 0 {
        #[allow(clippy::cast_precision_loss)]
        let pct = (summary.total_dead_lines as f32 / project_total_lines as f32) * 100.0;
        pct.min(100.0)
    } else {
        0.0
    };
}

/// Run multi-language dead code analysis for non-Rust projects
fn run_multi_language_dead_code(
    path: &Path,
    filters: &DeadCodeAnalysisFilters,
    language: &str,
) -> Result<DeadCodeAnalysisOutcome> {
    use crate::models::dead_code::{
        ConfidenceLevel, DeadCodeItem, DeadCodeSummary, DeadCodeType, FileDeadCodeMetrics,
    };
    use crate::services::dead_code_multi_language::analyze_dead_code_multi_language;

    eprintln!("🌐 Using multi-language analyzer for {language}");

    let ml_result = analyze_dead_code_multi_language(path)?;

    // Group dead functions by file for FileDeadCodeMetrics
    let mut file_map: std::collections::HashMap<
        String,
        Vec<&crate::services::dead_code_multi_language::DeadFunction>,
    > = std::collections::HashMap::new();
    for dead_fn in &ml_result.dead_functions {
        file_map
            .entry(dead_fn.file.clone())
            .or_default()
            .push(dead_fn);
    }

    let mut files: Vec<FileDeadCodeMetrics> = file_map
        .into_iter()
        .map(|(file_path, dead_fns)| {
            // MEASURED (was the literal `100` for every file). `0` only when
            // the file cannot be read, in which case `update_percentage` leaves
            // the percentage at 0.0 rather than dividing by a made-up total.
            let total_lines = count_lines_of(path, &file_path);
            let mut metrics = FileDeadCodeMetrics::new(file_path);
            metrics.total_lines = total_lines;
            for dead_fn in &dead_fns {
                metrics.add_item(DeadCodeItem {
                    item_type: DeadCodeType::Function,
                    name: dead_fn.name.clone(),
                    line: dead_fn.line as u32,
                    reason: dead_fn.reason.clone(),
                });
            }
            // `add_item` bills a flat 10 lines per dead function, which is an
            // estimate, while `total_lines` above is measured. Unbounded, the
            // estimate exceeded the file it describes: a 2-line h.py with one
            // dead function reported dead_lines 10 / total_lines 2 = 500.0%,
            // and a 10-line m.py reported 20 / 10 = 200.0%. Dead code cannot
            // occupy more lines than the file physically has, so the estimate is
            // held to the measured file length before any percentage is taken.
            if total_lines > 0 {
                metrics.dead_lines = metrics.dead_lines.min(total_lines);
            }
            // Lua has dynamic dispatch, so Medium confidence for non-local functions
            metrics.confidence = ConfidenceLevel::Medium;
            metrics.update_percentage();
            metrics.calculate_score();
            metrics
        })
        .filter(|f| f.dead_lines >= filters.min_dead_lines || f.dead_functions > 0)
        .collect();

    // Sort by score descending, path ascending as the tie-break: the map above
    // is a HashMap, so equal-score files would otherwise swap between runs.
    files.sort_by(|a, b| {
        b.dead_score
            .partial_cmp(&a.dead_score)
            .unwrap_or(std::cmp::Ordering::Equal)
            .then_with(|| a.path.cmp(&b.path))
    });

    let files_with_dead_code_found = files.len();
    let mut files_truncated = false;
    if let Some(limit) = filters.top_files {
        if limit > 0 && files.len() > limit {
            files.truncate(limit);
            files_truncated = true;
        }
    }

    // from_files() derives every figure from the files it is given, so the
    // summary always agrees with the list that follows it.
    let mut summary = DeadCodeSummary::from_files(&files);
    // ...but `total_files_analyzed` is not a figure about the LISTED files: it
    // is how many files were read. `from_files` had it counting the files with
    // dead code, so one run of `analyze dead-code` reported three different
    // numbers for it -- stdout "Files analyzed: 2" (from `analyzed_files`),
    // stderr "0 files analyzed" and JSON `summary.total_files_analyzed: 0`
    // (both from here).
    summary.total_files_analyzed = ml_result.total_files;

    // Source files under `path` in a language this run did not read. The
    // multi-language analyzer picks ONE language per project, so on a 19-file /
    // 12-language tree it silently dropped 17 files and still headed the report
    // "Files analyzed: 2" with no skip line at all.
    let source_files = count_source_files(path);
    let total_files = source_files.max(ml_result.total_files);

    Ok(DeadCodeAnalysisOutcome {
        report: crate::models::dead_code::DeadCodeResult {
            summary,
            // MEASURED file count (#720). These were `total_functions.max(1)` --
            // a FUNCTION count under a FILE label, which made a 2-file Python
            // fixture with 4 functions print "Files Analyzed | 4" directly above
            // a summary that correctly said 2, and `.max(1)` invented one file
            // for an empty project.
            total_files,
            analyzed_files: ml_result.total_files,
            files,
            files_with_dead_code_found,
            files_truncated,
        },
        // The multi-language analyzer never counts the project's total lines,
        // only the lines of the files it flagged, so there is no project-wide
        // ratio to report. `--fail-on-violation` refuses rather than comparing
        // the list-scoped figure and calling the result a pass.
        project_dead_percentage: None,
        scope: DeadCodeReportScope {
            project_dead_percentage: None,
            skipped_kind: Some(
                "source in languages this run did not read; the multi-language \
                 analyzer reads one language per project",
            ),
            // `--include`/`--exclude` are not wired into this path at all, so
            // nothing here was list-filtered.
            list_filtered: false,
        },
    })
}

/// Source files under `root`, by extension.
///
/// The multi-language analyzer reports only the files of the ONE language it
/// chose, so its count cannot say how much of the tree went unread. This is the
/// denominator that makes the gap visible.
fn count_source_files(root: &Path) -> usize {
    const SOURCE_EXTENSIONS: &[&str] = &[
        "rs", "ruchy", "rh", "c", "h", "cpp", "cc", "cxx", "hpp", "hxx", "py", "pyi", "lua", "go",
        "java", "kt", "kts", "scala", "swift", "cs", "rb", "php", "js", "jsx", "mjs", "cjs", "ts",
        "tsx", "sh", "bash", "zig", "zsh", "pl", "pm", "ex", "exs", "erl", "hs", "ml", "dart",
        "vim", "r", "jl",
    ];

    walkdir::WalkDir::new(root)
        .into_iter()
        .filter_map(std::result::Result::ok)
        .filter(|e| e.file_type().is_file())
        .filter(|e| {
            e.path()
                .extension()
                .and_then(|ext| ext.to_str())
                .is_some_and(|ext| SOURCE_EXTENSIONS.contains(&ext))
        })
        .count()
}

/// Physical line count for a file discovered by the multi-language analyzer.
///
/// The reported path may be absolute or relative to the analyzed root; `0` is
/// returned only when neither resolves to a readable file.
fn count_lines_of(root: &Path, file_path: &str) -> usize {
    let candidate = Path::new(file_path);
    let full = if candidate.is_absolute() {
        candidate.to_path_buf()
    } else {
        root.join(candidate)
    };
    std::fs::read_to_string(&full)
        .or_else(|_| std::fs::read_to_string(candidate))
        .map(|content| content.lines().count())
        .unwrap_or(0)
}

/// Create dead code ranking result from cargo analysis report
fn create_dead_code_ranking_result(
    accurate_report: crate::services::cargo_dead_code_analyzer::AccurateDeadCodeReport,
    min_dead_lines: usize,
    config: crate::models::dead_code::DeadCodeAnalysisConfig,
) -> crate::models::dead_code::DeadCodeRankingResult {
    use crate::models::dead_code::DeadCodeRankingResult;
    use chrono::Utc;

    DeadCodeRankingResult {
        ranked_files: convert_cargo_files_to_metrics(
            accurate_report.files_with_dead_code.clone(),
            min_dead_lines,
        ),
        summary: create_dead_code_summary(&accurate_report),
        analysis_timestamp: Utc::now(),
        config,
    }
}

/// Convert cargo dead code files to metrics format
fn convert_cargo_files_to_metrics(
    cargo_files: Vec<crate::services::cargo_dead_code_analyzer::FileDeadCode>,
    min_dead_lines: usize,
) -> Vec<crate::models::dead_code::FileDeadCodeMetrics> {
    use crate::models::dead_code::{ConfidenceLevel, FileDeadCodeMetrics};

    cargo_files
        .into_iter()
        .map(|file| {
            let dead_functions_count = count_dead_items_by_kind(
                &file,
                &[
                    crate::services::cargo_dead_code_analyzer::DeadCodeKind::Function,
                    crate::services::cargo_dead_code_analyzer::DeadCodeKind::Method,
                ],
            );
            let dead_classes_count = count_dead_items_by_kind(
                &file,
                &[
                    crate::services::cargo_dead_code_analyzer::DeadCodeKind::Struct,
                    crate::services::cargo_dead_code_analyzer::DeadCodeKind::Enum,
                ],
            );

            let dead_modules_count = count_dead_items_by_kind(
                &file,
                &[crate::services::cargo_dead_code_analyzer::DeadCodeKind::Module],
            );

            FileDeadCodeMetrics {
                path: file.file_path.display().to_string(),
                // Same estimator as the project total (5 lines per fn/method,
                // 3 per struct/enum, 2 otherwise). It used to be
                // `dead_items.len() * 4`, which disagreed with the summary.
                dead_lines:
                    crate::services::cargo_dead_code_analyzer::estimated_dead_lines_bounded(
                        &file.dead_items,
                        file.total_lines,
                    ),
                // MEASURED (was the literal `100` for every file, which
                // contradicted the dead_percentage printed beside it: a
                // 370-line file reported dead_lines 24 / total_lines 100 and
                // dead_percentage 6.49). `0` when the file could not be read;
                // the percentage is then 0.0 too, so the row never claims a
                // ratio it did not compute.
                total_lines: file.total_lines.unwrap_or(0),
                dead_percentage: file.file_dead_percentage as f32,
                dead_functions: dead_functions_count,
                dead_classes: dead_classes_count,
                dead_modules: dead_modules_count,
                unreachable_blocks: 0,
                dead_score: file.file_dead_percentage as f32,
                confidence: ConfidenceLevel::High, // Cargo-based detection is high confidence
                items: dead_items_to_report_items(&file.dead_items),
            }
        })
        .filter(|f| f.dead_lines >= min_dead_lines)
        .collect()
}

/// Carry the dead items cargo actually reported into the result.
///
/// They were dropped (`items: Vec::new()`), which left the counts in the
/// summary unverifiable — a reader could not see WHICH items produced
/// `total_dead_lines`.
fn dead_items_to_report_items(
    items: &[crate::services::cargo_dead_code_analyzer::DeadItem],
) -> Vec<crate::models::dead_code::DeadCodeItem> {
    use crate::models::dead_code::{DeadCodeItem, DeadCodeType};
    use crate::services::cargo_dead_code_analyzer::DeadCodeKind;

    items
        .iter()
        .map(|item| DeadCodeItem {
            item_type: match item.kind {
                DeadCodeKind::Function | DeadCodeKind::Method => DeadCodeType::Function,
                DeadCodeKind::Struct | DeadCodeKind::Enum | DeadCodeKind::Trait => {
                    DeadCodeType::Class
                }
                _ => DeadCodeType::Variable,
            },
            name: item.name.clone(),
            line: u32::try_from(item.line).unwrap_or(u32::MAX),
            reason: item.message.clone(),
        })
        .collect()
}

/// Count dead items of specific kinds
fn count_dead_items_by_kind(
    file: &crate::services::cargo_dead_code_analyzer::FileDeadCode,
    kinds: &[crate::services::cargo_dead_code_analyzer::DeadCodeKind],
) -> usize {
    file.dead_items
        .iter()
        .filter(|i| kinds.contains(&i.kind))
        .count()
}

/// Create dead code summary from cargo report.
///
/// Every per-list figure here is overwritten by `resummarize_from_listed_files`
/// once the reported list is final; only `total_files_analyzed` (the .rs files
/// walked) describes the project rather than the list.
fn create_dead_code_summary(
    accurate_report: &crate::services::cargo_dead_code_analyzer::AccurateDeadCodeReport,
) -> crate::models::dead_code::DeadCodeSummary {
    use crate::models::dead_code::DeadCodeSummary;

    DeadCodeSummary {
        total_files_analyzed: accurate_report.total_files, // actual .rs files walked
        files_with_dead_code: accurate_report.files_with_dead_code.len(),
        total_dead_lines: accurate_report.dead_lines,
        dead_percentage: accurate_report.dead_code_percentage as f32,
        dead_functions: get_dead_count_by_types(accurate_report, &["function", "method"]),
        dead_classes: get_dead_count_by_types(accurate_report, &["struct", "enum"]),
        dead_modules: get_dead_count_by_types(accurate_report, &["module"]),
        unreachable_blocks: 0, // Not tracked by cargo
    }
}

/// Get total dead count for specific types
fn get_dead_count_by_types(
    report: &crate::services::cargo_dead_code_analyzer::AccurateDeadCodeReport,
    types: &[&str],
) -> usize {
    types
        .iter()
        .map(|type_name| report.dead_by_type.get(*type_name).copied().unwrap_or(0))
        .sum()
}

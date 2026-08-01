// Dead code analysis logic - included from dead_code_handlers.rs
// NO `use` imports or `#!` inner attributes allowed here.

/// Run dead code analysis with include/exclude filters
async fn run_dead_code_analysis_with_filters(
    path: &Path,
    filters: DeadCodeAnalysisFilters,
) -> Result<crate::models::dead_code::DeadCodeResult> {
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
    let mut analysis_result =
        create_dead_code_ranking_result(accurate_report, filters.min_dead_lines, config);

    // Apply file filter to results if filters are active
    if filter.has_filters() {
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
    Ok(crate::models::dead_code::DeadCodeResult {
        summary: analysis_result.summary.clone(),
        files: analysis_result.ranked_files,
        total_files: analysis_result.summary.total_files_analyzed,
        analyzed_files: analysis_result.summary.total_files_analyzed,
        files_with_dead_code_found,
        files_truncated,
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
) -> Result<crate::models::dead_code::DeadCodeResult> {
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
    let summary = DeadCodeSummary::from_files(&files);

    Ok(crate::models::dead_code::DeadCodeResult {
        summary,
        total_files: ml_result.total_functions.max(1),
        analyzed_files: ml_result.total_functions.max(1),
        files,
        files_with_dead_code_found,
        files_truncated,
    })
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
                dead_lines: crate::services::cargo_dead_code_analyzer::estimated_dead_lines(
                    &file.dead_items,
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

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

    // Convert cargo report to ranking format for compatibility
    let files_with_dead_code_count = accurate_report.files_with_dead_code.len();
    let mut analysis_result = create_dead_code_ranking_result(
        accurate_report,
        files_with_dead_code_count,
        filters.min_dead_lines,
        config,
    );

    // Apply file filter to results if filters are active
    if filter.has_filters() {
        analysis_result.ranked_files.retain(|file| {
            let path = std::path::Path::new(&file.path);
            filter.should_include(path)
        });

        // Update summary counts
        analysis_result.summary.files_with_dead_code = analysis_result.ranked_files.len();
        analysis_result.summary.total_dead_lines = analysis_result
            .ranked_files
            .iter()
            .map(|f| f.dead_lines)
            .sum();
    }

    // Apply top_files limit if specified
    if let Some(limit) = filters.top_files {
        if limit > 0 && analysis_result.ranked_files.len() > limit {
            analysis_result.ranked_files.truncate(limit);
        }
    }

    // Convert to DeadCodeResult
    Ok(crate::models::dead_code::DeadCodeResult {
        summary: analysis_result.summary.clone(),
        files: analysis_result.ranked_files,
        total_files: analysis_result.summary.total_files_analyzed,
        analyzed_files: analysis_result.summary.total_files_analyzed,
    })
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
            let mut metrics = FileDeadCodeMetrics::new(file_path);
            metrics.total_lines = 100; // Estimate
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

    // Sort by score descending
    files.sort_by(|a, b| {
        b.dead_score
            .partial_cmp(&a.dead_score)
            .unwrap_or(std::cmp::Ordering::Equal)
    });

    if let Some(limit) = filters.top_files {
        if limit > 0 && files.len() > limit {
            files.truncate(limit);
        }
    }

    let summary = DeadCodeSummary::from_files(&files);

    Ok(crate::models::dead_code::DeadCodeResult {
        summary,
        total_files: ml_result.total_functions.max(1),
        analyzed_files: ml_result.total_functions.max(1),
        files,
    })
}

/// Create dead code ranking result from cargo analysis report
fn create_dead_code_ranking_result(
    accurate_report: crate::services::cargo_dead_code_analyzer::AccurateDeadCodeReport,
    files_with_dead_code_count: usize,
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
        summary: create_dead_code_summary(&accurate_report, files_with_dead_code_count),
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

            FileDeadCodeMetrics {
                path: file.file_path.display().to_string(),
                dead_lines: file.dead_items.len() * 4, // Estimate lines per item
                total_lines: 100,                      // Will be updated later if needed
                dead_percentage: file.file_dead_percentage as f32,
                dead_functions: dead_functions_count,
                dead_classes: dead_classes_count,
                dead_modules: 0,
                unreachable_blocks: 0,
                dead_score: file.file_dead_percentage as f32,
                confidence: ConfidenceLevel::High, // Cargo-based detection is high confidence
                items: Vec::new(), // Will be populated if needed for detailed reporting
            }
        })
        .filter(|f| f.dead_lines >= min_dead_lines)
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

/// Create dead code summary from cargo report
fn create_dead_code_summary(
    accurate_report: &crate::services::cargo_dead_code_analyzer::AccurateDeadCodeReport,
    files_with_dead_code_count: usize,
) -> crate::models::dead_code::DeadCodeSummary {
    use crate::models::dead_code::DeadCodeSummary;

    DeadCodeSummary {
        total_files_analyzed: accurate_report.total_files, // actual .rs files walked
        files_with_dead_code: files_with_dead_code_count,
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

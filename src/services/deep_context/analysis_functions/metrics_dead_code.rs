// --- Dead code analysis ---

#[allow(clippy::cast_possible_truncation)]
pub async fn analyze_dead_code(
    path: &std::path::Path,
) -> anyhow::Result<crate::models::dead_code::DeadCodeRankingResult> {
    use crate::models::dead_code::{
        DeadCodeAnalysisConfig, DeadCodeRankingResult, DeadCodeSummary,
    };
    use crate::services::file_discovery::ProjectFileDiscovery;

    // Phase 1: Discover files for analysis without async AST parsing
    let discovery_service = ProjectFileDiscovery::new(path.to_path_buf());
    let all_files = discovery_service.discover_files()?;

    // Filter for source code files, excluding test files
    let files: Vec<_> = all_files
        .into_iter()
        .filter(|file| {
            if let Some(ext) = file.extension().and_then(|e| e.to_str()) {
                matches!(ext, "rs" | "ts" | "js" | "py")
                    && !crate::services::deep_context::is_test_file(file)
            } else {
                false
            }
        })
        .collect();

    // Phase 2: Perform lightweight static analysis for dead code detection
    // Use parallel processing for file I/O and analysis
    let mut file_metrics: Vec<crate::models::dead_code::FileDeadCodeMetrics> = files
        .par_iter()
        .filter_map(|file_path| {
            std::fs::read_to_string(file_path)
                .ok()
                .map(|content| analyze_file_for_dead_code(file_path, &content))
        })
        .collect();

    // Aggregate metrics
    let total_dead_functions: usize = file_metrics.par_iter().map(|m| m.dead_functions).sum();
    let total_dead_classes: usize = file_metrics.par_iter().map(|m| m.dead_classes).sum();
    let total_dead_lines: usize = file_metrics.par_iter().map(|m| m.dead_lines).sum();

    // Phase 3: Calculate summary statistics
    let files_with_dead_code = file_metrics
        .par_iter()
        .filter(|f| f.dead_score > 0.0)
        .count();
    let total_lines_estimate: usize = file_metrics.par_iter().map(|f| f.total_lines).sum();
    let dead_percentage = if total_lines_estimate > 0 {
        (total_dead_lines as f32 / total_lines_estimate as f32) * 100.0
    } else {
        0.0
    };

    // Phase 4: Sort files by dead code score
    file_metrics.sort_unstable_by(|a, b| {
        b.dead_score
            .partial_cmp(&a.dead_score)
            .unwrap_or(std::cmp::Ordering::Equal)
    });

    Ok(DeadCodeRankingResult {
        summary: DeadCodeSummary {
            total_files_analyzed: files.len(),
            files_with_dead_code,
            total_dead_lines,
            dead_percentage,
            dead_functions: total_dead_functions,
            dead_classes: total_dead_classes,
            dead_modules: 0,
            unreachable_blocks: 0,
        },
        ranked_files: file_metrics,
        analysis_timestamp: chrono::Utc::now(),
        config: DeadCodeAnalysisConfig {
            include_unreachable: true,
            include_tests: false,
            min_dead_lines: 5,
        },
    })
}

#[allow(clippy::cast_possible_truncation)]
fn analyze_file_for_dead_code(
    file_path: &std::path::Path,
    content: &str,
) -> crate::models::dead_code::FileDeadCodeMetrics {
    use crate::models::dead_code::{ConfidenceLevel, FileDeadCodeMetrics};

    let lines: Vec<&str> = content.lines().collect();
    let total_lines = lines.len();
    let file_ext = file_path
        .extension()
        .and_then(|ext| ext.to_str())
        .unwrap_or("");

    let mut dead_functions = 0;
    let mut dead_classes = 0;
    let mut dead_items = Vec::new();

    // Analyze based on file type
    match file_ext {
        "rs" => analyze_rust_dead_code(
            &lines,
            &mut dead_functions,
            &mut dead_classes,
            &mut dead_items,
        ),
        "ts" | "js" => analyze_typescript_dead_code(
            &lines,
            &mut dead_functions,
            &mut dead_classes,
            &mut dead_items,
        ),
        "py" => analyze_python_dead_code(
            &lines,
            &mut dead_functions,
            &mut dead_classes,
            &mut dead_items,
        ),
        _ => {}
    }

    let dead_lines = dead_items.len() * 5; // Conservative estimate
    let dead_percentage = if total_lines > 0 {
        (dead_lines as f32 / total_lines as f32) * 100.0
    } else {
        0.0
    };

    let confidence = if dead_items.is_empty() {
        ConfidenceLevel::High // High confidence in no dead code
    } else if dead_percentage > 20.0 {
        ConfidenceLevel::Medium
    } else {
        ConfidenceLevel::Low
    };

    let mut metrics = FileDeadCodeMetrics {
        path: file_path.to_string_lossy().to_string(),
        dead_lines,
        total_lines,
        dead_percentage,
        dead_functions,
        dead_classes,
        dead_modules: 0,
        unreachable_blocks: 0,
        dead_score: 0.0,
        confidence,
        items: dead_items,
    };

    metrics.calculate_score();
    metrics
}

fn analyze_rust_dead_code(
    lines: &[&str],
    dead_functions: &mut usize,
    dead_classes: &mut usize,
    dead_items: &mut Vec<crate::models::dead_code::DeadCodeItem>,
) {
    analyze_rust_dead_functions(lines, dead_functions, dead_items);
    analyze_rust_dead_structs(lines, dead_classes, dead_items);
}

/// Analyze dead functions in Rust code
#[allow(clippy::cast_possible_truncation)]
fn analyze_rust_dead_functions(
    lines: &[&str],
    dead_functions: &mut usize,
    dead_items: &mut Vec<crate::models::dead_code::DeadCodeItem>,
) {
    use crate::models::dead_code::{DeadCodeItem, DeadCodeType};

    for (line_num, line) in lines.iter().enumerate() {
        let trimmed = line.trim();

        if trimmed.starts_with("fn ") && !trimmed.contains("pub ") {
            if let Some(function_name) = extract_function_name_if_unused(lines, trimmed) {
                *dead_functions += 1;
                dead_items.push(DeadCodeItem {
                    item_type: DeadCodeType::Function,
                    name: function_name,
                    line: (line_num + 1) as u32,
                    reason: "Private function with no apparent callers".to_string(),
                });
            }
        }
    }
}

/// Analyze dead structs in Rust code
#[allow(clippy::cast_possible_truncation)]
fn analyze_rust_dead_structs(
    lines: &[&str],
    dead_classes: &mut usize,
    dead_items: &mut Vec<crate::models::dead_code::DeadCodeItem>,
) {
    use crate::models::dead_code::{DeadCodeItem, DeadCodeType};

    for (line_num, line) in lines.iter().enumerate() {
        let trimmed = line.trim();

        if trimmed.starts_with("struct ") && !trimmed.contains("pub ") {
            if let Some(struct_name) = extract_struct_name_if_unused(lines, trimmed) {
                *dead_classes += 1;
                dead_items.push(DeadCodeItem {
                    item_type: DeadCodeType::Class,
                    name: struct_name,
                    line: (line_num + 1) as u32,
                    reason: "Private struct with no apparent usage".to_string(),
                });
            }
        }
    }
}

/// Extract function name if unused
fn extract_function_name_if_unused(lines: &[&str], trimmed: &str) -> Option<String> {
    let function_name = extract_function_name(trimmed);
    if !function_name.is_empty() && !is_function_called_in_file(lines, &function_name) {
        Some(function_name)
    } else {
        None
    }
}

/// Extract struct name if unused
fn extract_struct_name_if_unused(lines: &[&str], trimmed: &str) -> Option<String> {
    let struct_name = extract_struct_name(trimmed);
    if !struct_name.is_empty() && !is_type_used_in_file(lines, &struct_name) {
        Some(struct_name)
    } else {
        None
    }
}

fn analyze_typescript_dead_code(
    lines: &[&str],
    dead_functions: &mut usize,
    dead_classes: &mut usize,
    dead_items: &mut Vec<crate::models::dead_code::DeadCodeItem>,
) {
    analyze_typescript_dead_functions(lines, dead_functions, dead_items);
    analyze_typescript_dead_classes(lines, dead_classes, dead_items);
}

/// Analyze dead functions in TypeScript code
#[allow(clippy::cast_possible_truncation)]
fn analyze_typescript_dead_functions(
    lines: &[&str],
    dead_functions: &mut usize,
    dead_items: &mut Vec<crate::models::dead_code::DeadCodeItem>,
) {
    use crate::models::dead_code::{DeadCodeItem, DeadCodeType};

    for (line_num, line) in lines.iter().enumerate() {
        let trimmed = line.trim();

        if trimmed.starts_with("function ") && !trimmed.contains("export") {
            if let Some(function_name) = extract_js_function_name_if_unused(lines, trimmed) {
                *dead_functions += 1;
                dead_items.push(DeadCodeItem {
                    item_type: DeadCodeType::Function,
                    name: function_name,
                    line: (line_num + 1) as u32,
                    reason: "Non-exported function with no apparent callers".to_string(),
                });
            }
        }
    }
}

/// Analyze dead classes in TypeScript code
#[allow(clippy::cast_possible_truncation)]
fn analyze_typescript_dead_classes(
    lines: &[&str],
    dead_classes: &mut usize,
    dead_items: &mut Vec<crate::models::dead_code::DeadCodeItem>,
) {
    use crate::models::dead_code::{DeadCodeItem, DeadCodeType};

    for (line_num, line) in lines.iter().enumerate() {
        let trimmed = line.trim();

        if trimmed.starts_with("class ") && !trimmed.contains("export") {
            if let Some(class_name) = extract_class_name_if_unused(lines, trimmed) {
                *dead_classes += 1;
                dead_items.push(DeadCodeItem {
                    item_type: DeadCodeType::Class,
                    name: class_name,
                    line: (line_num + 1) as u32,
                    reason: "Non-exported class with no apparent usage".to_string(),
                });
            }
        }
    }
}

/// Extract JS function name if unused
fn extract_js_function_name_if_unused(lines: &[&str], trimmed: &str) -> Option<String> {
    let function_name = extract_js_function_name(trimmed);
    if !function_name.is_empty() && !is_function_called_in_file(lines, &function_name) {
        Some(function_name)
    } else {
        None
    }
}

/// Extract class name if unused
fn extract_class_name_if_unused(lines: &[&str], trimmed: &str) -> Option<String> {
    let class_name = extract_class_name(trimmed);
    if !class_name.is_empty() && !is_type_used_in_file(lines, &class_name) {
        Some(class_name)
    } else {
        None
    }
}


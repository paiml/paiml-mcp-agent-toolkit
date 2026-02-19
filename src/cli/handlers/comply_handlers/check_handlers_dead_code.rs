/// CB-304: Dead Code Percentage (COMPLY-044)
/// Enforces the dead_code_threshold from DeepContextConfig.
/// Scans source files for dead code indicators (#[allow(dead_code)], unused items)
/// and flags when the estimated dead code percentage exceeds the threshold (default 15%).
pub(crate) fn check_dead_code_percentage(project_path: &Path) -> ComplianceCheck {
    let config = crate::models::deep_context_config::DeepContextConfig::default();
    let threshold_pct = config.dead_code_threshold * 100.0; // 15.0%

    // Check multiple source directory conventions
    let source_dirs: Vec<std::path::PathBuf> = ["src", "crates", "lean", "lib"]
        .iter()
        .map(|d| project_path.join(d))
        .filter(|d| d.exists() && d.is_dir())
        .collect();

    if source_dirs.is_empty() {
        return ComplianceCheck {
            name: "CB-304: Dead Code Percentage".to_string(),
            status: CheckStatus::Skip,
            message: "No source directory found (checked src/, crates/, lean/, lib/)".to_string(),
            severity: Severity::Info,
        };
    }

    let (mut total_items, mut dead_items, mut total_lines, mut dead_lines) = (0, 0, 0, 0);
    for src_dir in &source_dirs {
        let (ti, di, tl, dl) = scan_dead_code_indicators(src_dir);
        total_items += ti;
        dead_items += di;
        total_lines += tl;
        dead_lines += dl;
    }

    if total_items == 0 {
        return ComplianceCheck {
            name: "CB-304: Dead Code Percentage".to_string(),
            status: CheckStatus::Pass,
            message: "No code items found to analyze".to_string(),
            severity: Severity::Info,
        };
    }

    // Use line-based percentage if available, otherwise item-based
    let dead_pct = if total_lines > 0 && dead_lines > 0 {
        (dead_lines as f64 / total_lines as f64) * 100.0
    } else {
        (dead_items as f64 / total_items as f64) * 100.0
    };

    let message = format!(
        "Dead code: {:.1}% ({} dead items/{} total, ~{} dead lines/{} total) [threshold: {:.0}%]",
        dead_pct, dead_items, total_items, dead_lines, total_lines, threshold_pct,
    );

    if dead_pct <= threshold_pct {
        ComplianceCheck {
            name: "CB-304: Dead Code Percentage".to_string(),
            status: CheckStatus::Pass,
            message,
            severity: Severity::Info,
        }
    } else if dead_pct <= threshold_pct * 2.0 {
        ComplianceCheck {
            name: "CB-304: Dead Code Percentage".to_string(),
            status: CheckStatus::Warn,
            message: format!("{message} - exceeds threshold"),
            severity: Severity::Warning,
        }
    } else {
        ComplianceCheck {
            name: "CB-304: Dead Code Percentage".to_string(),
            status: CheckStatus::Fail,
            message: format!("{message} - significantly exceeds threshold"),
            severity: Severity::Error,
        }
    }
}

/// Scan source files for dead code indicators.
/// Returns (total_items, dead_items, total_lines, estimated_dead_lines).
pub(crate) fn scan_dead_code_indicators(src_dir: &Path) -> (usize, usize, usize, usize) {
    let mut total_items = 0usize;
    let mut dead_items = 0usize;
    let mut total_lines = 0usize;
    let mut estimated_dead_lines = 0usize;

    let source_files = collect_production_rs_files(src_dir);

    for path in &source_files {
        let Ok(content) = fs::read_to_string(path) else {
            continue;
        };
        // Fix #137: Skip files with heavy cfg/SIMD usage (cfg-gated code appears dead on wrong arch)
        if is_heavily_cfg_gated(&content) {
            continue;
        }
        let lines: Vec<&str> = content.lines().collect();
        let file_result = analyze_file_dead_code(&lines);
        total_items += file_result.0;
        dead_items += file_result.1;
        total_lines += file_result.2;
        estimated_dead_lines += file_result.3;
    }

    (total_items, dead_items, total_lines, estimated_dead_lines)
}

/// Check if a file is heavily cfg-gated (SIMD, arch-specific code).
/// These files have code that only compiles on certain architectures, causing false dead code reports.
pub(crate) fn is_heavily_cfg_gated(content: &str) -> bool {
    let cfg_count = content.matches("#[cfg(target").count()
        + content.matches("#[target_feature").count()
        + content.matches("#[cfg(feature").count();
    // If more than 3 cfg attributes, likely SIMD/arch-specific code
    cfg_count > 3
}

/// Collect production .rs files (skip test files, falsification modules, SIMD code).
pub(crate) fn collect_production_rs_files(src_dir: &Path) -> Vec<std::path::PathBuf> {
    walkdir::WalkDir::new(src_dir)
        .max_depth(10)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| {
            let p = e.path();
            let path_str = p.to_string_lossy();
            p.is_file()
                && p.extension().is_some_and(|ext| ext == "rs")
                && !path_str.ends_with("_tests.rs")
                && !path_str.contains("/tests/")
                // Fix #135: Exclude falsification test modules (used for property testing)
                && !path_str.contains("/falsification/")
                && !path_str.contains("_falsification")
                // Fix #137: Exclude SIMD-heavy directories (cfg-gated code)
                && !path_str.contains("/quantize/")
                && !path_str.contains("/simd/")
        })
        .map(|e| e.path().to_path_buf())
        .collect()
}

/// Analyze a single file for dead code indicators.
/// Returns (total_items, dead_items, prod_lines, estimated_dead_lines).
pub(crate) fn analyze_file_dead_code(lines: &[&str]) -> (usize, usize, usize, usize) {
    let prod_lines: Vec<&str> = filter_production_lines(lines);
    let (total_items, dead_items, allow_dead_count) = count_dead_items(&prod_lines);
    let block_comment_lines = count_block_comment_code_lines(lines);
    let line_comment_lines = count_commented_code_lines(lines);
    let estimated_dead_lines = allow_dead_count * 10 + line_comment_lines + block_comment_lines;
    (total_items, dead_items, prod_lines.len(), estimated_dead_lines)
}

/// Filter out test module lines, returning only production lines.
pub(crate) fn filter_production_lines<'a>(lines: &[&'a str]) -> Vec<&'a str> {
    let mut result = Vec::new();
    let mut in_test_module = false;
    for line in lines {
        let trimmed = line.trim();
        if trimmed == "#[cfg(test)]" {
            in_test_module = true;
            continue;
        }
        if !in_test_module {
            result.push(*line);
        }
    }
    result
}

/// Count total items and dead items from production lines.
/// Returns (total_items, dead_items, annotation_count).
/// Excludes fn declarations inside macro_rules! blocks (they inflate the denominator).
pub(crate) fn count_dead_items(lines: &[&str]) -> (usize, usize, usize) {
    let mut total = 0usize;
    let mut dead = 0usize;
    let mut annotations = 0usize;
    let mut next_is_dead = false;
    let mut macro_depth: Option<i32> = None;

    for line in lines {
        let trimmed = line.trim();
        macro_depth = update_macro_depth(trimmed, macro_depth);
        if macro_depth.is_some() {
            continue;
        }
        classify_item_line(trimmed, &mut total, &mut dead, &mut annotations, &mut next_is_dead);
    }
    (total, dead, annotations)
}

/// Classify a single line for item counting.
pub(crate) fn classify_item_line(
    trimmed: &str,
    total: &mut usize,
    dead: &mut usize,
    annotations: &mut usize,
    next_is_dead: &mut bool,
) {
    if is_dead_code_annotation(trimmed) {
        *next_is_dead = true;
        *annotations += 1;
    } else if is_code_item_declaration(trimmed) {
        *total += 1;
        if *next_is_dead {
            *dead += 1;
        }
        *next_is_dead = false;
    } else if !trimmed.is_empty() && !trimmed.starts_with("//") && !trimmed.starts_with('#') {
        *next_is_dead = false;
    }
}

/// Track brace depth inside macro_rules! blocks.
/// Returns Some(depth) while inside a macro, None when outside.
pub(crate) fn update_macro_depth(trimmed: &str, current: Option<i32>) -> Option<i32> {
    let mut depth = if trimmed.starts_with("macro_rules!") {
        Some(current.unwrap_or(0))
    } else {
        current
    };
    if let Some(ref mut d) = depth {
        for ch in trimmed.chars() {
            match ch {
                '{' => *d += 1,
                '}' => *d -= 1,
                _ => {}
            }
        }
        if *d <= 0 && trimmed.contains('}') {
            return None;
        }
    }
    depth
}

/// Check if a line is a dead code annotation.
pub(crate) fn is_dead_code_annotation(trimmed: &str) -> bool {
    trimmed.starts_with("#[allow(dead_code)]") || trimmed.starts_with("#[allow(unused")
}

/// Check if a line declares a code item (fn, struct, enum, trait, const, static).
pub(crate) fn is_code_item_declaration(trimmed: &str) -> bool {
    const ITEM_PREFIXES: &[&str] = &[
        "pub fn ", "pub async fn ", "fn ", "async fn ",
        "pub struct ", "struct ", "pub enum ", "enum ",
        "pub trait ", "pub(crate) fn ", "pub(crate) struct ",
        "pub const ", "pub static ",
    ];
    ITEM_PREFIXES.iter().any(|p| trimmed.starts_with(p))
}

/// Count lines inside `/* ... */` block comments that look like code.
pub(crate) fn count_block_comment_code_lines(lines: &[&str]) -> usize {
    let mut dead_lines = 0usize;
    let mut in_block = false;
    let mut block_lines = 0usize;

    for line in lines {
        let trimmed = line.trim();
        let (new_in_block, add_dead, new_block_lines) =
            process_block_comment_line(trimmed, in_block, block_lines);
        in_block = new_in_block;
        dead_lines += add_dead;
        block_lines = new_block_lines;
    }
    dead_lines
}

/// Process a single line for block comment detection.
/// Returns (still_in_block, dead_lines_to_add, updated_block_lines).
fn process_block_comment_line(trimmed: &str, in_block: bool, block_lines: usize) -> (bool, usize, usize) {
    if !in_block {
        return handle_outside_block(trimmed);
    }
    handle_inside_block(trimmed, block_lines)
}

/// Handle a line when we are outside a block comment.
fn handle_outside_block(trimmed: &str) -> (bool, usize, usize) {
    let Some(rest) = trimmed.strip_prefix("/*") else {
        return (false, 0, 0);
    };
    // Single-line block comment (opens and closes on same line)
    if rest.contains("*/") {
        let add = if has_code_markers(rest) { 1 } else { 0 };
        return (false, add, 0);
    }
    // Block comment starts, continues on next lines
    (true, 0, 0)
}

/// Handle a line when we are inside a block comment.
fn handle_inside_block(trimmed: &str, block_lines: usize) -> (bool, usize, usize) {
    if trimmed.contains("*/") {
        let add = if block_lines >= 2 { block_lines } else { 0 };
        return (false, add, 0);
    }
    let new_block_lines = if has_code_markers(trimmed) { block_lines + 1 } else { block_lines };
    (true, 0, new_block_lines)
}

/// Check if text contains code-like markers.
pub(crate) fn has_code_markers(text: &str) -> bool {
    const MARKERS: &[&str] = &["fn ", "let ", "if ", "return ", ";", "struct ", "impl ", "pub "];
    MARKERS.iter().any(|m| text.contains(m))
}

/// Count lines in large blocks of `//` commented-out code (3+ consecutive lines).
pub(crate) fn count_commented_code_lines(lines: &[&str]) -> usize {
    let mut dead_lines = 0usize;
    let mut run = 0usize;

    for line in lines {
        if is_commented_out_code(line.trim()) {
            run += 1;
        } else {
            dead_lines += flush_comment_run(run);
            run = 0;
        }
    }
    dead_lines + flush_comment_run(run)
}

/// Flush a run of consecutive code comments (count if >= 3).
pub(crate) fn flush_comment_run(run: usize) -> usize {
    if run >= 3 { run } else { 0 }
}

/// Check if a comment line looks like commented-out code.
pub(crate) fn is_commented_out_code(trimmed: &str) -> bool {
    let body = if let Some(b) = trimmed.strip_prefix("// ") {
        b
    } else if let Some(b) = trimmed.strip_prefix("//\t") {
        b
    } else {
        return false;
    };
    const CODE_MARKERS: &[&str] = &["fn ", "let ", "if ", "return ", ";", "{", "}"];
    CODE_MARKERS.iter().any(|m| body.contains(m))
}

// Dead code analysis helpers for CB-304 compliance check
//
// Originally check_handlers_dead_code.rs, now a proper submodule.

use std::fs;
use std::path::Path;

/// Scan source files for dead code indicators.
/// Returns (total_items, dead_items, total_lines, estimated_dead_lines).
pub(crate) fn scan_dead_code_indicators(src_dir: &Path) -> (usize, usize, usize, usize) {
    let mut total_items = 0usize;
    let mut dead_items = 0usize;
    let mut total_lines = 0usize;
    let mut estimated_dead_lines = 0usize;

    let source_files = collect_production_rs_files(src_dir);

    for path in &source_files {
        let Ok(content) = fs::read_to_string(path) else { continue };
        if is_heavily_cfg_gated(&content) { continue; }
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
pub(crate) fn is_heavily_cfg_gated(content: &str) -> bool {
    let cfg_count = content.matches("#[cfg(target").count()
        + content.matches("#[target_feature").count()
        + content.matches("#[cfg(feature").count();
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
                && !path_str.contains("/falsification/")
                && !path_str.contains("_falsification")
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
        if trimmed == "#[cfg(test)]" { in_test_module = true; continue; }
        if !in_test_module { result.push(*line); }
    }
    result
}

/// Count total items and dead items from production lines.
/// Returns (total_items, dead_items, annotation_count).
pub(crate) fn count_dead_items(lines: &[&str]) -> (usize, usize, usize) {
    let mut total = 0usize;
    let mut dead = 0usize;
    let mut annotations = 0usize;
    let mut next_is_dead = false;
    let mut macro_depth: Option<i32> = None;
    for line in lines {
        let trimmed = line.trim();
        macro_depth = update_macro_depth(trimmed, macro_depth);
        if macro_depth.is_some() { continue; }
        classify_item_line(trimmed, &mut total, &mut dead, &mut annotations, &mut next_is_dead);
    }
    (total, dead, annotations)
}

/// Classify a single line for item counting.
pub(crate) fn classify_item_line(trimmed: &str, total: &mut usize, dead: &mut usize, annotations: &mut usize, next_is_dead: &mut bool) {
    if is_dead_code_annotation(trimmed) {
        *next_is_dead = true;
        *annotations += 1;
    } else if is_code_item_declaration(trimmed) {
        *total += 1;
        if *next_is_dead { *dead += 1; }
        *next_is_dead = false;
    } else if !trimmed.is_empty() && !trimmed.starts_with("//") && !trimmed.starts_with('#') {
        *next_is_dead = false;
    }
}

/// Track brace depth inside macro_rules! blocks.
pub(crate) fn update_macro_depth(trimmed: &str, current: Option<i32>) -> Option<i32> {
    let mut depth = if trimmed.starts_with("macro_rules!") { Some(current.unwrap_or(0)) } else { current };
    if let Some(ref mut d) = depth {
        for ch in trimmed.chars() {
            match ch { '{' => *d += 1, '}' => *d -= 1, _ => {} }
        }
        if *d <= 0 && trimmed.contains('}') { return None; }
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
        let (new_in_block, add_dead, new_block_lines) = process_block_comment_line(trimmed, in_block, block_lines);
        in_block = new_in_block;
        dead_lines += add_dead;
        block_lines = new_block_lines;
    }
    dead_lines
}

fn process_block_comment_line(trimmed: &str, in_block: bool, block_lines: usize) -> (bool, usize, usize) {
    if !in_block { return handle_outside_block(trimmed); }
    handle_inside_block(trimmed, block_lines)
}

fn handle_outside_block(trimmed: &str) -> (bool, usize, usize) {
    let Some(rest) = trimmed.strip_prefix("/*") else { return (false, 0, 0); };
    if rest.contains("*/") { let add = if has_code_markers(rest) { 1 } else { 0 }; return (false, add, 0); }
    (true, 0, 0)
}

fn handle_inside_block(trimmed: &str, block_lines: usize) -> (bool, usize, usize) {
    if trimmed.contains("*/") { let add = if block_lines >= 2 { block_lines } else { 0 }; return (false, add, 0); }
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
        if is_commented_out_code(line.trim()) { run += 1; } else { dead_lines += flush_comment_run(run); run = 0; }
    }
    dead_lines + flush_comment_run(run)
}

/// Flush a run of consecutive code comments (count if >= 3).
pub(crate) fn flush_comment_run(run: usize) -> usize {
    if run >= 3 { run } else { 0 }
}

/// Check if a comment line looks like commented-out code.
pub(crate) fn is_commented_out_code(trimmed: &str) -> bool {
    let body = if let Some(b) = trimmed.strip_prefix("// ") { b }
    else if let Some(b) = trimmed.strip_prefix("//\t") { b }
    else { return false; };
    const CODE_MARKERS: &[&str] = &["fn ", "let ", "if ", "return ", ";", "{", "}"];
    CODE_MARKERS.iter().any(|m| body.contains(m))
}

#[cfg_attr(coverage_nightly, coverage(off))]
#[cfg(test)]
mod dead_code_tests {
    use super::*;

    #[test]
    fn test_is_heavily_cfg_gated_true() {
        let content = "#[cfg(target_arch = \"x86_64\")]\n#[cfg(target_arch = \"aarch64\")]\n#[cfg(feature = \"simd\")]\n#[target_feature(enable = \"avx2\")]\nfn simd_function() {}";
        assert!(is_heavily_cfg_gated(content));
    }

    #[test]
    fn test_is_heavily_cfg_gated_false() {
        let content = "fn regular_function() {}\nfn another_function() {}";
        assert!(!is_heavily_cfg_gated(content));
    }

    #[test]
    fn test_is_heavily_cfg_gated_boundary() {
        let content = "#[cfg(target_arch = \"x86_64\")]\n#[cfg(feature = \"simd\")]\n#[target_feature(enable = \"avx2\")]";
        assert!(!is_heavily_cfg_gated(content));
    }

    #[test]
    fn test_filter_production_lines_basic() {
        let lines = vec!["fn main() {}", "let x = 1;", "#[cfg(test)]", "mod tests {", "    #[test]", "    fn test_something() {}", "}"];
        let result = filter_production_lines(&lines);
        assert_eq!(result.len(), 2);
    }

    #[test]
    fn test_filter_production_lines_no_test_module() {
        let lines = vec!["fn main() {}", "let x = 1;"];
        let result = filter_production_lines(&lines);
        assert_eq!(result.len(), 2);
    }

    #[test]
    fn test_count_dead_items_no_dead() {
        let lines = vec!["pub fn active() {}", "pub struct Active {}"];
        let (total, dead, _) = count_dead_items(&lines);
        assert!(total >= 2);
        assert_eq!(dead, 0);
    }

    #[test]
    fn test_count_dead_items_with_dead() {
        let lines = vec!["#[allow(dead_code)]", "fn unused() {}", "pub fn active() {}"];
        let (total, dead, _) = count_dead_items(&lines);
        assert!(total >= 2);
        assert!(dead >= 1);
    }

    #[test]
    fn test_classify_item_line_function() {
        let mut total = 0; let mut dead = 0; let mut annotations = 0; let mut next_is_dead = false;
        classify_item_line("pub fn test() {}", &mut total, &mut dead, &mut annotations, &mut next_is_dead);
        assert_eq!(total, 1);
        assert_eq!(dead, 0);
    }

    #[test]
    fn test_classify_item_line_dead_annotation() {
        let mut total = 0; let mut dead = 0; let mut annotations = 0; let mut next_is_dead = false;
        classify_item_line("#[allow(dead_code)]", &mut total, &mut dead, &mut annotations, &mut next_is_dead);
        assert_eq!(annotations, 1);
        assert!(next_is_dead);
    }
}

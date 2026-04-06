#![cfg_attr(coverage_nightly, coverage(off))]
//! CB-500 Series: Performance-related checks and data integrity.
//!
//! - CB-517: Stale Debug Artifacts
//! - CB-518: Expensive Clone in Loop
//! - CB-519: Lossy Data Pipeline
//! - CB-520: Expensive Init in Hot Path
//! - CB-521: Format Detection Without Magic Bytes

use super::utilities::{
    classify_cb521_line, find_lossy_pair, is_fn_start, is_loop_start, is_spawn_call,
    is_test_fn_definition, EXPENSIVE_INIT_PATTERNS,
};
use crate::cli::handlers::comply_cb_detect::types::*;
use std::fs;
use std::path::Path;

/// CB-517: Stale Debug Artifacts - leftover debug instrumentation in production code
pub fn detect_cb517_stale_debug_artifacts(project_path: &Path) -> Vec<CbPatternViolation> {
    debug_assert!(
        project_path.exists(),
        "project_path must exist: {}",
        project_path.display()
    );
    let src_dir = project_path.join("src");
    let entries = match walkdir_rs_files(&src_dir) {
        Ok(e) => e,
        Err(_) => return Vec::new(),
    };

    let mut violations = Vec::new();

    for entry in &entries {
        if is_test_file(entry) {
            continue;
        }
        let content = match fs::read_to_string(entry) {
            Ok(c) => c,
            Err(_) => continue,
        };
        let lines: Vec<&str> = content.lines().collect();
        let test_lines = compute_test_code_lines(&lines);
        let file = entry
            .strip_prefix(project_path)
            .unwrap_or(entry)
            .display()
            .to_string();

        for (i, line) in lines.iter().enumerate() {
            if test_lines.contains(&i) {
                continue;
            }
            let trimmed = line.trim();
            if trimmed.starts_with("//") || trimmed.starts_with("*") {
                continue;
            }

            // Detect static AtomicUsize/AtomicBool debug counters outside const context
            if trimmed.contains("static")
                && (trimmed.contains("AtomicUsize") || trimmed.contains("AtomicBool"))
                && !trimmed.starts_with("const ")
            {
                violations.push(CbPatternViolation {
                    pattern_id: "CB-517".to_string(),
                    file: file.clone(),
                    line: i + 1,
                    description:
                        "Stale debug artifact: static Atomic counter (likely debug instrumentation)"
                            .to_string(),
                    severity: Severity::Warning,
                });
                continue;
            }

            // Detect #[allow(unused)] on static variables (often leftover instrumentation)
            if trimmed == "#[allow(unused)]" || trimmed == "#[allow(dead_code)]" {
                // Check if next non-empty line is a static declaration
                for j in (i + 1)..std::cmp::min(i + 3, lines.len()) {
                    let next = lines[j].trim();
                    if next.is_empty() {
                        continue;
                    }
                    if next.starts_with("static ") {
                        violations.push(CbPatternViolation {
                            pattern_id: "CB-517".to_string(),
                            file: file.clone(),
                            line: i + 1,
                            description:
                                "Stale debug artifact: #[allow(unused)] on static variable"
                                    .to_string(),
                            severity: Severity::Warning,
                        });
                    }
                    break;
                }
            }
        }
    }

    violations
}

/// CB-518: Expensive Clone in Loop - .clone() calls inside loop bodies
pub fn detect_cb518_expensive_clone_in_loop(project_path: &Path) -> Vec<CbPatternViolation> {
    debug_assert!(
        project_path.exists(),
        "project_path must exist: {}",
        project_path.display()
    );
    let src_dir = project_path.join("src");
    let entries = match walkdir_rs_files(&src_dir) {
        Ok(e) => e,
        Err(_) => return Vec::new(),
    };

    let mut violations = Vec::new();

    for entry in &entries {
        if is_test_file(entry) {
            continue;
        }
        let content = match fs::read_to_string(entry) {
            Ok(c) => c,
            Err(_) => continue,
        };
        let lines: Vec<&str> = content.lines().collect();
        let test_lines = compute_test_code_lines(&lines);
        let file = entry
            .strip_prefix(project_path)
            .unwrap_or(entry)
            .display()
            .to_string();

        // Track loop bodies via brace depth
        let mut in_loop = false;
        let mut loop_depth: u32 = 0;
        let mut loop_start: usize = 0;
        let mut clone_count: u32 = 0;
        let mut clone_lines: Vec<usize> = Vec::new();

        for (i, line) in lines.iter().enumerate() {
            if test_lines.contains(&i) {
                continue;
            }
            let trimmed = line.trim();

            // Detect loop starts
            if !in_loop
                && (trimmed.starts_with("for ")
                    || trimmed.starts_with("while ")
                    || trimmed == "loop {"
                    || trimmed.starts_with("loop {"))
            {
                in_loop = true;
                loop_depth = 0;
                loop_start = i;
                clone_count = 0;
                clone_lines.clear();
            }

            if in_loop {
                loop_depth += trimmed.matches('{').count() as u32;
                loop_depth = loop_depth.saturating_sub(trimmed.matches('}').count() as u32);

                if trimmed.contains(".clone()") {
                    clone_count += 1;
                    clone_lines.push(i + 1);
                }

                // End of loop body
                if loop_depth == 0 && i > loop_start {
                    if clone_count > 3 {
                        violations.push(CbPatternViolation {
                            pattern_id: "CB-518".to_string(),
                            file: file.clone(),
                            line: loop_start + 1,
                            description: format!(
                                "Expensive clone in loop: {} .clone() calls (lines: {})",
                                clone_count,
                                clone_lines
                                    .iter()
                                    .take(5)
                                    .map(|l| l.to_string())
                                    .collect::<Vec<_>>()
                                    .join(", ")
                            ),
                            severity: Severity::Info,
                        });
                    }
                    in_loop = false;
                }
            }
        }
    }

    violations
}

/// CB-519: Lossy Data Pipeline - detect quantize/dequantize/encode/decode round-trip chains
pub fn detect_cb519_lossy_data_pipeline(project_path: &Path) -> Vec<CbPatternViolation> {
    debug_assert!(
        project_path.exists(),
        "project_path must exist: {}",
        project_path.display()
    );
    let src_dir = project_path.join("src");
    let entries = match walkdir_rs_files(&src_dir) {
        Ok(e) => e,
        Err(_) => return Vec::new(),
    };

    let mut violations = Vec::new();

    for entry in &entries {
        if is_test_file(entry) {
            continue;
        }
        let content = match fs::read_to_string(entry) {
            Ok(c) => c,
            Err(_) => continue,
        };
        let lines: Vec<&str> = content.lines().collect();
        let test_lines = compute_test_code_lines(&lines);
        let file = entry
            .strip_prefix(project_path)
            .unwrap_or(entry)
            .display()
            .to_string();

        let mut fn_start: Option<usize> = None;
        let mut fn_depth: u32 = 0;
        let mut fn_content = String::new();
        let mut skip_fn = false;

        for (i, line) in lines.iter().enumerate() {
            if test_lines.contains(&i) {
                continue;
            }
            let trimmed = line.trim();

            if is_fn_start(trimmed) && fn_start.is_none() {
                skip_fn = is_test_fn_definition(trimmed, i, &lines);
                fn_start = Some(i);
                fn_depth = 0;
                fn_content.clear();
            }

            if fn_start.is_none() {
                continue;
            }
            fn_depth += trimmed.matches('{').count() as u32;
            fn_depth = fn_depth.saturating_sub(trimmed.matches('}').count() as u32);
            fn_content.push_str(trimmed);
            fn_content.push('\n');

            // End of function
            if fn_depth == 0 && i > fn_start.unwrap_or(i) {
                if !skip_fn {
                    if let Some((fwd, rev)) = find_lossy_pair(&fn_content, &file) {
                        violations.push(CbPatternViolation {
                            pattern_id: "CB-519".to_string(),
                            file: file.clone(),
                            line: fn_start.unwrap_or(0) + 1,
                            description: format!(
                                "Lossy data pipeline: both {fwd}() and {rev}() in same function — possible round-trip data corruption"
                            ),
                            severity: Severity::Warning,
                        });
                    }
                }
                fn_start = None;
                fn_content.clear();
            }
        }
    }

    violations
}

/// Scan a single file for CB-520 expensive init in loop violations.
fn scan_cb520_file(
    lines: &[&str],
    test_lines: &std::collections::HashSet<usize>,
    file: &str,
    violations: &mut Vec<CbPatternViolation>,
) {
    let mut in_loop = false;
    let mut loop_depth: u32 = 0;
    let mut loop_start: usize = 0;
    let mut init_count: u32 = 0;
    let mut init_examples: Vec<String> = Vec::new();
    let mut spawn_depth: u32 = 0;

    for (i, line) in lines.iter().enumerate() {
        if test_lines.contains(&i) {
            continue;
        }
        let trimmed = line.trim();
        if trimmed.starts_with("//") || trimmed.starts_with("*") {
            continue;
        }

        if !in_loop && is_loop_start(trimmed) {
            in_loop = true;
            loop_depth = 0;
            loop_start = i;
            init_count = 0;
            init_examples.clear();
            spawn_depth = 0;
        }
        if !in_loop {
            continue;
        }
        loop_depth += trimmed.matches('{').count() as u32;
        loop_depth = loop_depth.saturating_sub(trimmed.matches('}').count() as u32);

        // Track spawn closures -- constructors inside these are per-thread init
        if is_spawn_call(trimmed) {
            spawn_depth += trimmed.matches('{').count() as u32;
        }
        if spawn_depth > 0 {
            spawn_depth = spawn_depth.saturating_sub(trimmed.matches('}').count() as u32);
        }

        // Only count expensive init if NOT inside a spawn closure
        if spawn_depth == 0 {
            if let Some(pat) = EXPENSIVE_INIT_PATTERNS
                .iter()
                .find(|p| trimmed.contains(**p))
            {
                init_count += 1;
                if init_examples.len() < 3 {
                    init_examples.push(pat.trim_start_matches("::").to_string());
                }
            }
        }

        if loop_depth == 0 && i > loop_start {
            if init_count >= 2 {
                violations.push(CbPatternViolation {
                    pattern_id: "CB-520".to_string(),
                    file: file.to_string(),
                    line: loop_start + 1,
                    description: format!(
                        "Expensive initialization in loop: {} constructor/load calls ({})",
                        init_count,
                        init_examples.join(", ")
                    ),
                    severity: Severity::Warning,
                });
            }
            in_loop = false;
        }
    }
}

/// CB-520: Expensive Init in Hot Path - constructor/load/open calls inside loops
pub fn detect_cb520_expensive_init_in_loop(project_path: &Path) -> Vec<CbPatternViolation> {
    debug_assert!(
        project_path.exists(),
        "project_path must exist: {}",
        project_path.display()
    );
    let src_dir = project_path.join("src");
    let entries = match walkdir_rs_files(&src_dir) {
        Ok(e) => e,
        Err(_) => return Vec::new(),
    };

    let mut violations = Vec::new();

    for entry in &entries {
        if is_test_file(entry) {
            continue;
        }
        let content = match fs::read_to_string(entry) {
            Ok(c) => c,
            Err(_) => continue,
        };
        let lines: Vec<&str> = content.lines().collect();
        let test_lines = compute_test_code_lines(&lines);
        let file = entry
            .strip_prefix(project_path)
            .unwrap_or(entry)
            .display()
            .to_string();

        scan_cb520_file(&lines, &test_lines, &file, &mut violations);
    }

    violations
}

/// CB-521: Format Detection Without Magic Bytes - binary parsing without header validation
pub fn detect_cb521_format_without_magic_bytes(project_path: &Path) -> Vec<CbPatternViolation> {
    debug_assert!(
        project_path.exists(),
        "project_path must exist: {}",
        project_path.display()
    );
    let src_dir = project_path.join("src");
    let entries = match walkdir_rs_files(&src_dir) {
        Ok(e) => e,
        Err(_) => return Vec::new(),
    };

    let mut violations = Vec::new();

    for entry in &entries {
        if is_test_file(entry) {
            continue;
        }
        let content = match fs::read_to_string(entry) {
            Ok(c) => c,
            Err(_) => continue,
        };
        let lines: Vec<&str> = content.lines().collect();
        let test_lines = compute_test_code_lines(&lines);
        let file = entry
            .strip_prefix(project_path)
            .unwrap_or(entry)
            .display()
            .to_string();

        let mut fn_start: Option<usize> = None;
        let mut fn_depth: u32 = 0;
        let mut has_binary_read = false;
        let mut has_magic_check = false;
        let mut has_io_context = false;
        let mut binary_line = 0usize;

        for (i, line) in lines.iter().enumerate() {
            if test_lines.contains(&i) {
                continue;
            }
            let trimmed = line.trim();

            if is_fn_start(trimmed) && fn_start.is_none() {
                fn_start = Some(i);
                fn_depth = 0;
                has_binary_read = false;
                has_magic_check = false;
                binary_line = 0;
                // Check function signature for I/O types
                let (_, _, io) = classify_cb521_line(trimmed);
                has_io_context = io;
            }

            if fn_start.is_none() {
                continue;
            }
            fn_depth += trimmed.matches('{').count() as u32;
            fn_depth = fn_depth.saturating_sub(trimmed.matches('}').count() as u32);

            if !trimmed.starts_with("//") {
                let (binary, magic, io) = classify_cb521_line(trimmed);
                if binary && !has_binary_read {
                    binary_line = i;
                }
                has_binary_read |= binary;
                has_magic_check |= magic;
                has_io_context |= io;
            }

            if fn_depth == 0 && i > fn_start.unwrap_or(i) {
                // Only flag if actual I/O context is present -- pure byte math
                // (hash functions, quantized matvec) is not binary format parsing
                if has_binary_read && !has_magic_check && has_io_context {
                    violations.push(CbPatternViolation {
                        pattern_id: "CB-521".to_string(),
                        file: file.clone(),
                        line: binary_line + 1,
                        description: "Binary format parsing without magic byte/header validation"
                            .to_string(),
                        severity: Severity::Warning,
                    });
                }
                fn_start = None;
            }
        }
    }

    violations
}

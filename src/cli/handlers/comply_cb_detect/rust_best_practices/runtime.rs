#![cfg_attr(coverage_nightly, coverage(off))]
//! CB-500 Series: Runtime behavior checks.
//!
//! - CB-507: Panic Macros
//! - CB-510: include!() Macro Hygiene
//! - CB-511: Flaky Timing Tests
//! - CB-512: Error Propagation Gap
//! - CB-513: Silent Error Swallowing
//! - CB-514: Debug Eprintln Leaks

use super::utilities::{
    is_macro_in_string_literal, DOT_UNWRAP, EPRINTLN_DBG, EPRINTLN_DEBUG, EPRINTLN_TRACE,
    MAP_ERR_DISCARD, UNWRAP_OR_ELSE_DISCARD,
};
use crate::cli::handlers::comply_cb_detect::types::*;
use std::fs;
use std::path::Path;

/// CB-507: Panic Macros - todo!(), unimplemented!() in production code
pub fn detect_cb507_panic_macros(project_path: &Path) -> Vec<CbPatternViolation> {
    let src_dir = project_path.join("src");
    let entries = match walkdir_rs_files(&src_dir) {
        Ok(e) => e,
        Err(_) => return Vec::new(),
    };

    let panic_macros = ["todo!(", "unimplemented!("];
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
            if is_macro_in_string_literal(trimmed, &panic_macros) {
                continue;
            }
            for mac in &panic_macros {
                if trimmed.contains(mac) {
                    violations.push(CbPatternViolation {
                        pattern_id: "CB-507".to_string(),
                        file: file.clone(),
                        line: i + 1,
                        description: format!(
                            "Panic macro `{}` in production code",
                            mac.trim_end_matches('(')
                        ),
                        severity: Severity::Warning,
                    });
                    break;
                }
            }
        }
    }

    violations
}

/// CB-510: include!() Macro Hygiene - non-standalone files included via include!()
pub fn detect_cb510_include_macro_hygiene(project_path: &Path) -> Vec<CbPatternViolation> {
    let src_dir = project_path.join("src");
    let entries = match walkdir_rs_files(&src_dir) {
        Ok(e) => e,
        Err(_) => return Vec::new(),
    };

    let mut violations = Vec::new();

    for entry in &entries {
        let content = match fs::read_to_string(entry) {
            Ok(c) => c,
            Err(_) => continue,
        };
        let file = entry
            .strip_prefix(project_path)
            .unwrap_or(entry)
            .display()
            .to_string();

        for (i, line) in content.lines().enumerate() {
            let trimmed = line.trim();
            if trimmed.starts_with("//") {
                continue;
            }
            if trimmed.contains("include!(") && !trimmed.contains("include_str!") {
                violations.push(CbPatternViolation {
                    pattern_id: "CB-510".to_string(),
                    file: file.clone(),
                    line: i + 1,
                    description: "include!() macro - included files are not standalone compilable"
                        .to_string(),
                    severity: Severity::Info,
                });
            }
        }
    }

    violations
}

/// CB-511: Flaky Timing Tests - tests with Instant::now() and tight duration assertions
pub fn detect_cb511_flaky_timing_tests(project_path: &Path) -> Vec<CbPatternViolation> {
    let src_dir = project_path.join("src");
    let test_dir = project_path.join("tests");

    let mut all_files = walkdir_rs_files(&src_dir).unwrap_or_default();
    all_files.extend(walkdir_rs_files(&test_dir).unwrap_or_default());

    let mut violations = Vec::new();

    for entry in &all_files {
        let content = match fs::read_to_string(entry) {
            Ok(c) => c,
            Err(_) => continue,
        };

        // Only check files with test attributes
        if !content.contains("#[test]") && !content.contains("#[tokio::test]") {
            continue;
        }

        let file = entry
            .strip_prefix(project_path)
            .unwrap_or(entry)
            .display()
            .to_string();

        // Per-test-function analysis: only flag when Instant::now(), .elapsed(),
        // and a duration assertion all appear in the SAME test function
        let lines: Vec<&str> = content.lines().collect();
        let mut in_test_fn = false;
        let mut test_fn_start: usize = 0;
        let mut brace_depth: u32 = 0;
        let mut fn_has_instant = false;
        let mut fn_has_elapsed = false;
        let mut fn_has_duration_assert = false;

        for (i, line) in lines.iter().enumerate() {
            let trimmed = line.trim();

            // Detect test function start (look for #[test] or #[tokio::test] preceding fn)
            if !in_test_fn
                && (trimmed.starts_with("fn ") || trimmed.starts_with("async fn "))
                && i > 0
            {
                // Look back for #[test] or #[tokio::test] attribute
                let has_test_attr = (1..=3).any(|back| {
                    i >= back && {
                        let prev = lines[i - back].trim();
                        prev == "#[test]" || prev == "#[tokio::test]"
                    }
                });
                if has_test_attr {
                    in_test_fn = true;
                    test_fn_start = i + 1;
                    brace_depth = 0;
                    fn_has_instant = false;
                    fn_has_elapsed = false;
                    fn_has_duration_assert = false;
                }
            }

            if in_test_fn {
                brace_depth += trimmed.matches('{').count() as u32;
                brace_depth = brace_depth.saturating_sub(trimmed.matches('}').count() as u32);

                if trimmed.contains("Instant::now()") {
                    fn_has_instant = true;
                }
                if trimmed.contains(".elapsed()") {
                    fn_has_elapsed = true;
                }
                // Require .elapsed() specifically in an assertion context,
                // not just any line with "assert!" and the word "elapsed"
                if (trimmed.contains("assert!")
                    || trimmed.contains("assert_eq!")
                    || trimmed.contains("assert_ne!")
                    || trimmed.contains("assert_lt!")
                    || trimmed.contains("assert_le!"))
                    && trimmed.contains(".elapsed()")
                {
                    fn_has_duration_assert = true;
                }

                // End of test function
                if brace_depth == 0 && i > test_fn_start {
                    if fn_has_instant && fn_has_elapsed && fn_has_duration_assert {
                        violations.push(CbPatternViolation {
                            pattern_id: "CB-511".to_string(),
                            file: file.clone(),
                            line: test_fn_start,
                            description: "Test uses Instant::now() with duration assertions — may be flaky under load".to_string(),
                            severity: Severity::Warning,
                        });
                    }
                    in_test_fn = false;
                }
            }
        }
    }

    violations
}

/// CB-512: Error Propagation Gap - functions returning Result but using unwrap() internally
pub fn detect_cb512_error_propagation_gap(project_path: &Path) -> Vec<CbPatternViolation> {
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

        let mut in_result_fn = false;
        let mut fn_line = 0;
        let mut fn_depth = 0u32;
        let mut unwrap_count = 0u32;

        for (i, line) in lines.iter().enumerate() {
            if test_lines.contains(&i) {
                continue;
            }

            let trimmed = line.trim();

            // Detect function returning Result
            if (trimmed.starts_with("pub fn ")
                || trimmed.starts_with("fn ")
                || trimmed.starts_with("pub async fn ")
                || trimmed.starts_with("async fn "))
                && trimmed.contains("Result<")
            {
                in_result_fn = true;
                fn_line = i;
                fn_depth = 0;
                unwrap_count = 0;
            }

            if in_result_fn {
                fn_depth += trimmed.matches('{').count() as u32;
                fn_depth = fn_depth.saturating_sub(trimmed.matches('}').count() as u32);

                if trimmed.contains(DOT_UNWRAP) {
                    unwrap_count += 1;
                }

                // End of function
                if fn_depth == 0 && i > fn_line {
                    if unwrap_count >= 3 {
                        violations.push(CbPatternViolation {
                            pattern_id: "CB-512".to_string(),
                            file: file.clone(),
                            line: fn_line + 1,
                            description: format!(
                                "Function returns Result but has {unwrap_count} unwrap() calls - consider using ? operator"
                            ),
                            severity: Severity::Warning,
                        });
                    }
                    in_result_fn = false;
                }
            }
        }
    }

    violations
}

/// CB-513: Silent Error Swallowing - discarding error context with |_| closures
pub fn detect_cb513_silent_error_swallowing(project_path: &Path) -> Vec<CbPatternViolation> {
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

            // Detect .unwrap_or_else(|_| -- intentionally discarded error
            if trimmed.contains(UNWRAP_OR_ELSE_DISCARD) {
                violations.push(CbPatternViolation {
                    pattern_id: "CB-513".to_string(),
                    file: file.clone(),
                    line: i + 1,
                    description:
                        "Silent error swallowing: .unwrap_or_else(|_| discards error context"
                            .to_string(),
                    severity: Severity::Warning,
                });
                continue;
            }

            // Detect .map_err(|_| -- discards original error
            if trimmed.contains(MAP_ERR_DISCARD) {
                violations.push(CbPatternViolation {
                    pattern_id: "CB-513".to_string(),
                    file: file.clone(),
                    line: i + 1,
                    description:
                        "Silent error swallowing: .map_err(|_| discards original error context"
                            .to_string(),
                    severity: Severity::Warning,
                });
            }
        }
    }

    violations
}

/// CB-514: Debug Eprintln Leaks - debug print statements in production code
pub fn detect_cb514_debug_eprintln_leaks(project_path: &Path) -> Vec<CbPatternViolation> {
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

            if trimmed.contains(EPRINTLN_DEBUG)
                || trimmed.contains(EPRINTLN_DBG)
                || trimmed.contains(EPRINTLN_TRACE)
            {
                violations.push(CbPatternViolation {
                    pattern_id: "CB-514".to_string(),
                    file: file.clone(),
                    line: i + 1,
                    description: "Debug eprintln! leak in production code".to_string(),
                    severity: Severity::Warning,
                });
            }
        }
    }

    violations
}

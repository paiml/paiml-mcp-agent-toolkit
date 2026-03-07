#![cfg_attr(coverage_nightly, coverage(off))]
//! CB-608: Unchecked nil,err Return Pattern, CB-609: assert() in Library Code,
//! and CB-610: Smarter String Concatenation (accumulator pattern) detection.

use super::super::types::*;
use super::constants::NIL_ERR_FUNCTIONS;
use super::helpers::{
    compute_lua_production_lines, is_in_lua_string, is_lua_test_file, walkdir_lua_files,
};
use std::fs;
use std::path::Path;

// =============================================================================
// CB-608: Unchecked nil, err Return Pattern (#181)
// =============================================================================

/// CB-608: Unchecked return nil, err -- caller ignores error return.
/// Priority P0: Dominant Lua error handling pattern (>80% of real-world error handling).
/// Reference: Kong (1,725 instances), APISIX (716), xmake (254).
pub fn detect_cb608_unchecked_nil_err(project_path: &Path) -> Vec<CbPatternViolation> {
    let files = walkdir_lua_files(project_path);
    let mut violations = Vec::new();

    for file_path in &files {
        if is_lua_test_file(file_path) {
            continue;
        }
        let content = match fs::read_to_string(file_path) {
            Ok(c) => c,
            Err(_) => continue,
        };
        let prod_lines = compute_lua_production_lines(&content);
        let rel = file_path
            .strip_prefix(project_path)
            .unwrap_or(file_path)
            .display()
            .to_string();

        for (line_num, trimmed) in &prod_lines {
            if let Some(func_name) = calls_nil_err_function(trimmed) {
                if !captures_error_return(trimmed) {
                    violations.push(CbPatternViolation {
                        pattern_id: "CB-608".to_string(),
                        file: rel.clone(),
                        line: *line_num,
                        description: format!(
                            "Unchecked `{func_name}()` — returns nil,err but error not captured"
                        ),
                        severity: Severity::Warning,
                    });
                }
            }
        }
    }

    violations
}

/// Check if a line calls a known nil-returning function and return which one.
fn calls_nil_err_function(line: &str) -> Option<&'static str> {
    for &func in NIL_ERR_FUNCTIONS {
        let call = format!("{func}(");
        if line.contains(&call) {
            return Some(func);
        }
    }
    None
}

/// Check if a line captures both return values (e.g., `local ok, err = ...`).
fn captures_error_return(line: &str) -> bool {
    // Pattern: `local x, y = ...` or `x, y = ...`
    let trimmed = line.trim_start_matches("local ");
    if let Some(eq_pos) = trimmed.find('=') {
        let lhs = &trimmed[..eq_pos];
        // Check for at least two comma-separated captures
        return lhs.contains(',');
    }
    false
}

// =============================================================================
// CB-609: assert() in Library Code (#193)
// =============================================================================

/// CB-609: assert() in library code -- terminates without allowing recovery.
/// assert() is appropriate in tests but problematic in library code.
/// Reference: AwesomeWM (1,817 asserts), xmake (913).
pub fn detect_cb609_assert_in_library(project_path: &Path) -> Vec<CbPatternViolation> {
    let files = walkdir_lua_files(project_path);
    let mut violations = Vec::new();

    for file_path in &files {
        if is_lua_test_file(file_path) {
            continue; // assert() is fine in tests
        }
        let content = match fs::read_to_string(file_path) {
            Ok(c) => c,
            Err(_) => continue,
        };
        let prod_lines = compute_lua_production_lines(&content);
        let rel = file_path
            .strip_prefix(project_path)
            .unwrap_or(file_path)
            .display()
            .to_string();

        // Module-level require guards are acceptable (first 5 lines)
        let skip_lines = 5;

        for (line_num, trimmed) in &prod_lines {
            if *line_num <= skip_lines {
                continue;
            }
            if is_assert_call(trimmed) && !is_in_lua_string(trimmed, "assert") {
                violations.push(CbPatternViolation {
                    pattern_id: "CB-609".to_string(),
                    file: rel.clone(),
                    line: *line_num,
                    description: "assert() in library code — use error(msg, 2) or return nil, err"
                        .to_string(),
                    severity: Severity::Info,
                });
            }
        }
    }

    violations
}

/// Check if line contains an assert() call (not assert.xxx from test frameworks).
fn is_assert_call(line: &str) -> bool {
    if let Some(pos) = line.find("assert(") {
        // Exclude `assert.is_true(` etc. (test framework methods)
        if pos > 0 && line.as_bytes()[pos - 1] == b'.' {
            return false;
        }
        // Exclude `-- assert(` comments (already filtered by compute_lua_production_lines)
        true
    } else {
        false
    }
}

// =============================================================================
// CB-610: Smarter String Concatenation -- accumulator pattern only (#190)
// =============================================================================

/// CB-610: String accumulator in loop -- `result = result .. x` is O(n^2).
/// Only flags accumulator patterns (assigning back to same variable).
/// Single-use concatenation like `log("msg: " .. x)` is not flagged.
/// Reference: Issue #190 false positive reduction.
pub fn detect_cb610_string_accumulator_in_loop(project_path: &Path) -> Vec<CbPatternViolation> {
    let files = walkdir_lua_files(project_path);
    let mut violations = Vec::new();

    for file_path in &files {
        if is_lua_test_file(file_path) {
            continue;
        }
        let content = match fs::read_to_string(file_path) {
            Ok(c) => c,
            Err(_) => continue,
        };
        let prod_lines = compute_lua_production_lines(&content);
        let rel = file_path
            .strip_prefix(project_path)
            .unwrap_or(file_path)
            .display()
            .to_string();

        let mut loop_depth: i32 = 0;

        for (line_num, trimmed) in &prod_lines {
            if trimmed.starts_with("for ")
                || trimmed.starts_with("while ")
                || trimmed.starts_with("repeat")
            {
                loop_depth += 1;
            }

            if loop_depth > 0 && is_string_accumulator(trimmed) {
                violations.push(CbPatternViolation {
                    pattern_id: "CB-610".to_string(),
                    file: rel.clone(),
                    line: *line_num,
                    description:
                        "String accumulator in loop — O(n²), use table.insert + table.concat"
                            .to_string(),
                    severity: Severity::Warning,
                });
            }

            if trimmed == "end" || trimmed.starts_with("end ") || trimmed.starts_with("until ") {
                loop_depth = (loop_depth - 1).max(0);
            }
        }
    }

    violations
}

/// Check if a line is a string accumulator pattern: `var = var .. expr`.
fn is_string_accumulator(line: &str) -> bool {
    // Match `x = x .. y` pattern
    if let Some(eq_pos) = line.find('=') {
        // Skip `==` comparisons
        if line[eq_pos + 1..].starts_with('=') {
            return false;
        }
        let lhs = line[..eq_pos].trim();
        let rhs = line[eq_pos + 1..].trim();
        // Check if rhs starts with `lhs .. ` (accumulator)
        if rhs.starts_with(lhs) && rhs[lhs.len()..].trim_start().starts_with("..") {
            return true;
        }
    }
    false
}

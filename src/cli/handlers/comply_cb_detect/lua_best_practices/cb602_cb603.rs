#![cfg_attr(coverage_nightly, coverage(off))]
//! CB-602: pcall Error Handling and CB-603: Deprecated/Dangerous API detection.

use super::super::types::*;
use super::constants::{LUA_DANGEROUS_APIS, LUA_DEPRECATED_APIS};
use super::detection_helpers::{extract_pcall_status_var, has_status_check};
use super::helpers::{
    compute_lua_production_lines, is_in_lua_string, is_lua_test_file, is_suppressed,
    walkdir_lua_files,
};
use std::fs;
use std::path::Path;

/// CB-602: pcall Error Handling -- uncaptured or unchecked pcall/xpcall.
/// Based on FLuaScan progressive taint analysis.
pub fn detect_cb602_pcall_error_handling(project_path: &Path) -> Vec<CbPatternViolation> {
    debug_assert!(
        project_path.exists(),
        "project_path must exist: {}",
        project_path.display()
    );
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

        for (idx, (line_num, trimmed)) in prod_lines.iter().enumerate() {
            let has_pcall = trimmed.contains("pcall(") || trimmed.contains("xpcall(");
            if !has_pcall || is_in_lua_string(trimmed, "pcall") {
                continue;
            }

            // Case 1: pcall without capturing return value (no `=` before pcall)
            if !trimmed.contains('=') {
                violations.push(CbPatternViolation {
                    pattern_id: "CB-602".to_string(),
                    file: rel.clone(),
                    line: *line_num,
                    description: "pcall/xpcall return value not captured".to_string(),
                    severity: Severity::Warning,
                });
                continue;
            }

            // Case 2: captured but status not checked within next 5 lines
            // Extract the status variable name from `local ok, err = pcall(...)`
            let status_var = extract_pcall_status_var(trimmed);
            if !has_status_check(&prod_lines, idx, status_var.as_deref()) {
                violations.push(CbPatternViolation {
                    pattern_id: "CB-602".to_string(),
                    file: rel.clone(),
                    line: *line_num,
                    description: "pcall/xpcall status not checked within 5 lines".to_string(),
                    severity: Severity::Warning,
                });
            }
        }
    }

    violations
}

/// CB-603: Deprecated/Dangerous API usage.
/// Based on LuaTaint and FLuaScan -- os.execute(), io.popen(), loadstring(), setfenv().
///
/// Supports inline suppression: `-- pmat:ignore CB-603` on the same line.
/// Distinguishes safe usage (hardcoded string arg) from dangerous (concatenation/variable).
pub fn detect_cb603_deprecated_dangerous_api(project_path: &Path) -> Vec<CbPatternViolation> {
    debug_assert!(
        project_path.exists(),
        "project_path must exist: {}",
        project_path.display()
    );
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
        // Build map of original lines for suppression comment checking
        let original_lines: Vec<&str> = content.lines().collect();
        let prod_lines = compute_lua_production_lines(&content);
        let rel = file_path
            .strip_prefix(project_path)
            .unwrap_or(file_path)
            .display()
            .to_string();

        for (line_num, trimmed) in &prod_lines {
            // Check inline suppression on original line (before comment stripping)
            if is_suppressed(&original_lines, *line_num, "CB-603") {
                continue;
            }
            check_deprecated_apis(trimmed, &rel, *line_num, &mut violations);
            check_dangerous_apis(trimmed, &rel, *line_num, &mut violations);
        }
    }

    violations
}

fn check_deprecated_apis(
    trimmed: &str,
    rel: &str,
    line_num: usize,
    violations: &mut Vec<CbPatternViolation>,
) {
    for api in LUA_DEPRECATED_APIS {
        if trimmed.contains(api) && !is_in_lua_string(trimmed, api) {
            violations.push(CbPatternViolation {
                pattern_id: "CB-603".to_string(),
                file: rel.to_string(),
                line: line_num,
                description: format!(
                    "Deprecated API: `{}` — use `load()` or modern equivalent",
                    api.trim_end_matches('(')
                ),
                severity: Severity::Warning,
            });
        }
    }
}

fn check_dangerous_apis(
    trimmed: &str,
    rel: &str,
    line_num: usize,
    violations: &mut Vec<CbPatternViolation>,
) {
    for api in LUA_DANGEROUS_APIS {
        if !trimmed.contains(api) || is_in_lua_string(trimmed, api) {
            continue;
        }
        // Distinguish safe (hardcoded string arg) from dangerous (concatenation/variable)
        let severity = if has_hardcoded_string_arg(trimmed, api) {
            Severity::Info
        } else {
            Severity::Warning
        };
        violations.push(CbPatternViolation {
            pattern_id: "CB-603".to_string(),
            file: rel.to_string(),
            line: line_num,
            description: format!(
                "Dangerous API: `{}` — {}",
                api.trim_end_matches('('),
                if severity == Severity::Warning {
                    "potential command injection (variable/concatenation in argument)"
                } else {
                    "hardcoded string argument (lower risk)"
                }
            ),
            severity,
        });
    }
}

/// Check if a dangerous API call uses a hardcoded string argument.
/// `os.execute("make clean")` -> true (safe)
/// `os.execute(cmd)` or `os.execute("rm " .. x)` -> false (dangerous)
fn has_hardcoded_string_arg(line: &str, api: &str) -> bool {
    let Some(api_pos) = line.find(api) else {
        return false;
    };
    let after = &line[api_pos + api.len()..];

    // Check if argument starts with a string literal
    let trimmed = after.trim_start();
    let starts_with_string = trimmed.starts_with('"') || trimmed.starts_with('\'');
    if !starts_with_string {
        return false;
    }

    // Check there's no concatenation operator (..) in the argument
    !after.contains("..")
}

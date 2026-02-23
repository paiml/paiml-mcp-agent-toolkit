#![cfg_attr(coverage_nightly, coverage(off))]
//! CB-606: Missing Module Return and CB-607: Colon/Dot Confusion detection.

use super::constants::LUA_STD_TABLES;
use super::detection_helpers::{extract_method_call, extract_module_table_var};
use super::helpers::{
    compute_lua_production_lines, is_lua_test_file, walkdir_lua_files,
};
use super::super::types::*;
use std::fs;
use std::path::Path;

/// CB-606: Missing Module Return -- `local M = {}` pattern without final `return M`.
pub fn detect_cb606_missing_module_return(project_path: &Path) -> Vec<CbPatternViolation> {
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

        // Look for `local M = {}` or `local ModuleName = {}` near the top
        let module_var = extract_module_table_var(&prod_lines);

        if let Some(var) = module_var {
            let has_return = prod_lines.iter().rev().any(|(_, trimmed)| {
                *trimmed == format!("return {var}")
                    || trimmed.starts_with(&format!("return {var} "))
            });

            if !has_return {
                violations.push(CbPatternViolation {
                    pattern_id: "CB-606".to_string(),
                    file: rel,
                    line: 1,
                    description: format!(
                        "Module table `{var}` defined but no `return {var}` at end of file"
                    ),
                    severity: Severity::Info,
                });
            }
        }
    }

    violations
}

/// CB-607: Colon/Dot Confusion -- mixed `:` and `.` method calls on same table.
/// Based on Luau type system research.
pub fn detect_cb607_colon_dot_confusion(project_path: &Path) -> Vec<CbPatternViolation> {
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

        let table_usage = build_table_call_map(&prod_lines);
        emit_colon_dot_violations(&table_usage, &rel, &mut violations);
    }

    violations
}

/// Build per-table usage map: table_name -> (colon_lines, dot_lines).
fn build_table_call_map(
    prod_lines: &[(usize, String)],
) -> std::collections::HashMap<String, (Vec<usize>, Vec<usize>)> {
    use std::collections::HashMap;
    let mut table_usage: HashMap<String, (Vec<usize>, Vec<usize>)> = HashMap::new();

    for (line_num, trimmed) in prod_lines {
        if let Some(name) = extract_method_call(trimmed, ':') {
            if !LUA_STD_TABLES.contains(&name.as_str()) {
                table_usage.entry(name).or_default().0.push(*line_num);
            }
        }
        if let Some(name) = extract_method_call(trimmed, '.') {
            if !LUA_STD_TABLES.contains(&name.as_str()) {
                table_usage.entry(name).or_default().1.push(*line_num);
            }
        }
    }

    table_usage
}

/// Emit violations for tables with mixed colon and dot method calls.
fn emit_colon_dot_violations(
    table_usage: &std::collections::HashMap<String, (Vec<usize>, Vec<usize>)>,
    rel: &str,
    violations: &mut Vec<CbPatternViolation>,
) {
    for (table_name, (colon_lines, dot_lines)) in table_usage {
        if !colon_lines.is_empty() && !dot_lines.is_empty() {
            let first_line = *colon_lines
                .iter()
                .chain(dot_lines.iter())
                .min()
                .unwrap_or(&1);
            violations.push(CbPatternViolation {
                pattern_id: "CB-607".to_string(),
                file: rel.to_string(),
                line: first_line,
                description: format!(
                    "Mixed `:` and `.` method calls on `{table_name}` — use consistent style"
                ),
                severity: Severity::Warning,
            });
        }
    }
}

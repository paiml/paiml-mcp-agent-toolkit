#![cfg_attr(coverage_nightly, coverage(off))]
//! CB-614: Global Protection and Sandbox Pattern Detection
//! and CB-615: Coroutine Complexity Scoring.

use super::super::types::*;
use super::helpers::{is_lua_test_file, walkdir_lua_files};
use std::fs;
use std::path::{Path, PathBuf};

// =============================================================================
// CB-614: Lua Global Protection and Sandbox Pattern Detection (#191)
// =============================================================================

/// CB-614: Detect global protection patterns and security-sensitive load calls.
/// - Checks for setmetatable(_G) with __index/__newindex
/// - Flags loadfile/load without "t" mode (bytecode injection risk)
/// - Reports protection level: full, partial, or none
#[provable_contracts_macros::contract("pmat-core.yaml", equation = "path_exists")]
pub fn detect_cb614_global_protection(project_path: &Path) -> Vec<CbPatternViolation> {
    let files = walkdir_lua_files(project_path);
    if files.is_empty() {
        return Vec::new();
    }

    let mut violations = Vec::new();
    let mut has_newindex_protection = false;
    let mut has_index_protection = false;

    for file_path in &files {
        if is_lua_test_file(file_path) {
            continue;
        }
        let content = match fs::read_to_string(file_path) {
            Ok(c) => c,
            Err(_) => continue,
        };
        let rel = file_path
            .strip_prefix(project_path)
            .unwrap_or(file_path)
            .display()
            .to_string();

        check_global_metatables(
            &content,
            &mut has_newindex_protection,
            &mut has_index_protection,
        );
        check_unsafe_load_calls(&content, &rel, &mut violations);
    }

    report_protection_level(
        &files,
        has_newindex_protection,
        has_index_protection,
        &mut violations,
    );
    violations
}

/// Check if content sets metatable on _G with __index/__newindex.
fn check_global_metatables(content: &str, has_newindex: &mut bool, has_index: &mut bool) {
    for line in content.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with("--") {
            continue;
        }
        if trimmed.contains("setmetatable") && trimmed.contains("_G") {
            // Found setmetatable(_G, ...) -- check surrounding lines won't have
            // both protections on same line, so track across the file
            *has_newindex = true; // setmetatable(_G) implies at least __newindex
        }
        if trimmed.contains("__newindex") && (trimmed.contains("_G") || trimmed.contains("error")) {
            *has_newindex = true;
        }
        if trimmed.contains("__index")
            && !trimmed.contains("__newindex")
            && (trimmed.contains("_G")
                || trimmed.contains("error")
                || trimmed.contains("undefined"))
        {
            *has_index = true;
        }
    }
}

/// Flag loadfile/load calls without "t" mode (allows bytecode injection).
fn check_unsafe_load_calls(content: &str, rel: &str, violations: &mut Vec<CbPatternViolation>) {
    for (i, line) in content.lines().enumerate() {
        let trimmed = line.trim();
        if trimmed.starts_with("--") {
            continue;
        }
        check_single_load_call(trimmed, "loadfile", i + 1, rel, violations);
        // load(chunk, name, mode, env) -- mode is 3rd arg
        if trimmed.contains("load(") && !trimmed.contains("loadfile") {
            check_load_function_call(trimmed, i + 1, rel, violations);
        }
    }
}

/// Check a single loadfile() call for missing "t" mode.
fn check_single_load_call(
    trimmed: &str,
    func: &str,
    line_num: usize,
    rel: &str,
    violations: &mut Vec<CbPatternViolation>,
) {
    let pattern = format!("{func}(");
    let Some(pos) = trimmed.find(&pattern) else {
        return;
    };
    let after = &trimmed[pos + pattern.len()..];
    // loadfile(path, mode, env) -- mode is 2nd arg
    // If no "t" in the args, flag it
    if !after.contains("\"t\"") && !after.contains("'t'") && after.contains('"') {
        violations.push(CbPatternViolation {
            pattern_id: "CB-614".to_string(),
            file: rel.to_string(),
            line: line_num,
            description: format!(
                "`{func}()` without \"t\" mode — allows bytecode injection. Use {func}(path, \"t\", env)"
            ),
            severity: Severity::Warning,
        });
    }
}

/// Check load(chunk, name, mode, env) for missing "t" mode.
fn check_load_function_call(
    trimmed: &str,
    line_num: usize,
    rel: &str,
    violations: &mut Vec<CbPatternViolation>,
) {
    let Some(pos) = trimmed.find("load(") else {
        return;
    };
    // Ensure it's not loadfile/loadstring
    if pos > 0 {
        let before = trimmed.as_bytes()[pos - 1];
        if before.is_ascii_alphanumeric() || before == b'_' {
            return;
        }
    }
    let after = &trimmed[pos + 5..];
    // Count commas to check if mode arg is present
    let comma_count = after
        .chars()
        .take_while(|c| *c != ')')
        .filter(|c| *c == ',')
        .count();
    if comma_count >= 2 && !after.contains("\"t\"") && !after.contains("'t'") {
        violations.push(CbPatternViolation {
            pattern_id: "CB-614".to_string(),
            file: rel.to_string(),
            line: line_num,
            description:
                "`load()` without \"t\" mode — allows bytecode injection. Use load(chunk, name, \"t\", env)"
                    .to_string(),
            severity: Severity::Warning,
        });
    }
}

/// Report overall protection level for the project.
fn report_protection_level(
    files: &[PathBuf],
    has_newindex: bool,
    has_index: bool,
    violations: &mut Vec<CbPatternViolation>,
) {
    if files.len() < 3 {
        return; // Too few files to assess
    }
    match (has_newindex, has_index) {
        (true, true) => {
            violations.push(CbPatternViolation {
                pattern_id: "CB-614".to_string(),
                file: "project".to_string(),
                line: 0,
                description: "Global protection: full (both __index and __newindex on _G)"
                    .to_string(),
                severity: Severity::Info,
            });
        }
        (true, false) => {
            violations.push(CbPatternViolation {
                pattern_id: "CB-614".to_string(),
                file: "project".to_string(),
                line: 0,
                description:
                    "Global protection: partial (__newindex only) — reading undefined globals returns nil silently"
                        .to_string(),
                severity: Severity::Warning,
            });
        }
        _ => {} // No protection -- CB-600 already flags implicit globals
    }
}

// =============================================================================
// CB-615: Lua Coroutine Complexity Scoring (#188)
// =============================================================================

/// CB-615: Detect coroutine defect patterns and report usage.
/// - coroutine.resume without pcall (crashes on error)
/// - Coroutine usage counts for complexity awareness
#[provable_contracts_macros::contract("pmat-core.yaml", equation = "path_exists")]
pub fn detect_cb615_coroutine_checks(project_path: &Path) -> Vec<CbPatternViolation> {
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
        let rel = file_path
            .strip_prefix(project_path)
            .unwrap_or(file_path)
            .display()
            .to_string();

        check_coroutine_patterns(&content, &rel, &mut violations);
    }

    violations
}

/// Check for coroutine defect patterns in file content.
fn check_coroutine_patterns(content: &str, rel: &str, violations: &mut Vec<CbPatternViolation>) {
    for (i, line) in content.lines().enumerate() {
        let trimmed = line.trim();
        if trimmed.starts_with("--") {
            continue;
        }
        check_resume_without_pcall(trimmed, i + 1, rel, content, violations);
    }
}

/// Flag coroutine.resume() not wrapped in pcall/xpcall.
fn check_resume_without_pcall(
    trimmed: &str,
    line_num: usize,
    rel: &str,
    content: &str,
    violations: &mut Vec<CbPatternViolation>,
) {
    if !trimmed.contains("coroutine.resume") {
        return;
    }
    // Check if the resume is wrapped in pcall/xpcall on same line or previous line
    let safe = trimmed.contains("pcall") || trimmed.contains("xpcall");
    let prev_safe = line_num >= 2
        && content.lines().nth(line_num - 2).is_some_and(|prev| {
            let p = prev.trim();
            p.contains("pcall") || p.contains("xpcall")
        });
    // Also safe if assigned to ok, err pattern: `local ok, err = coroutine.resume(...)`
    let has_err_capture = trimmed.contains("ok,") || trimmed.contains("ok ,");

    if !safe && !prev_safe && !has_err_capture {
        violations.push(CbPatternViolation {
            pattern_id: "CB-615".to_string(),
            file: rel.to_string(),
            line: line_num,
            description: "coroutine.resume() without pcall/xpcall — errors propagate to caller"
                .to_string(),
            severity: Severity::Warning,
        });
    }
}

#![cfg_attr(coverage_nightly, coverage(off))]
//! Helper functions for Lua file walking, test detection, production line extraction,
//! and string/identifier utilities.

use super::constants::SKIP_DIRS;
use std::fs;
use std::path::{Path, PathBuf};

/// Walk directory recursively for `.lua` files, skipping common non-source dirs.
pub fn walkdir_lua_files(dir: &Path) -> Vec<PathBuf> {
    debug_assert!(dir.exists(), "dir must exist: {}", dir.display());
    let mut files = Vec::new();
    walk_lua_recursive(dir, &mut files);
    files
}

fn walk_lua_recursive(dir: &Path, files: &mut Vec<PathBuf>) {
    debug_assert!(dir.exists(), "dir must exist: {}", dir.display());
    let entries = match fs::read_dir(dir) {
        Ok(e) => e,
        Err(_) => return,
    };
    for entry in entries {
        let entry = match entry {
            Ok(e) => e,
            Err(_) => continue,
        };
        let path = entry.path();
        if path.is_dir() {
            let dir_name = path.file_name().and_then(|n| n.to_str()).unwrap_or("");
            if !SKIP_DIRS.contains(&dir_name) {
                walk_lua_recursive(&path, files);
            }
        } else if path.extension().map(|e| e == "lua").unwrap_or(false) {
            files.push(path);
        }
    }
}

/// Check if a file is a Lua test file based on naming conventions.
pub fn is_lua_test_file(path: &Path) -> bool {
    debug_assert!(path.exists(), "path must exist: {}", path.display());
    let stem = path.file_stem().and_then(|s| s.to_str()).unwrap_or("");
    if stem.ends_with("_test") || stem.ends_with("_spec") || stem.starts_with("test_") {
        return true;
    }
    path.components().any(|c| {
        let s = c.as_os_str().to_str().unwrap_or("");
        s == "tests" || s == "test" || s == "spec"
    })
}

/// Extract production (non-comment) lines from Lua source.
/// Returns Vec<(1-based line number, trimmed line content)>.
pub fn compute_lua_production_lines(content: &str) -> Vec<(usize, String)> {
    debug_assert!(!content.is_empty(), "content must not be empty");
    let mut result = Vec::new();
    let mut in_block_comment = false;

    for (i, line) in content.lines().enumerate() {
        let trimmed = line.trim();

        // Handle block comments --[[ ... ]]
        if in_block_comment {
            if trimmed.contains("]]") {
                in_block_comment = false;
            }
            continue;
        }

        if let Some(rest) = trimmed.strip_prefix("--[[") {
            if rest.contains("]]") {
                continue;
            }
            in_block_comment = true;
            continue;
        }

        // Skip single-line comments
        if trimmed.starts_with("--") {
            continue;
        }

        // Skip empty lines
        if trimmed.is_empty() {
            continue;
        }

        // Strip trailing inline comments (heuristic: not inside string)
        let effective = strip_trailing_comment(trimmed);
        if !effective.is_empty() {
            result.push((i + 1, effective));
        }
    }

    result
}

/// Simple heuristic to check if a pattern appears inside a Lua string literal.
pub(super) fn is_in_lua_string(line: &str, pattern: &str) -> bool {
    if let Some(pos) = line.find(pattern) {
        let before = &line[..pos];
        let double_quotes = before.chars().filter(|c| *c == '"').count();
        let single_quotes = before.chars().filter(|c| *c == '\'').count();
        double_quotes % 2 == 1 || single_quotes % 2 == 1
    } else {
        false
    }
}

/// Strip trailing `--` comment from a line (heuristic: not inside string).
fn strip_trailing_comment(line: &str) -> String {
    debug_assert!(!line.is_empty(), "line must not be empty");
    if let Some(pos) = line.find("--") {
        let before = &line[..pos];
        let double_q = before.chars().filter(|c| *c == '"').count();
        let single_q = before.chars().filter(|c| *c == '\'').count();
        if double_q % 2 == 0 && single_q % 2 == 0 {
            return before.trim().to_string();
        }
    }
    line.to_string()
}

/// Check if byte is start of a Lua identifier.
pub(super) fn is_ident_start(b: u8) -> bool {
    b.is_ascii_alphabetic() || b == b'_'
}

/// Check if byte can continue a Lua identifier.
pub(super) fn is_ident_cont(b: u8) -> bool {
    b.is_ascii_alphanumeric() || b == b'_'
}

/// Skip past an identifier at position `i`, returning new position.
pub(super) fn skip_identifier(bytes: &[u8], mut i: usize) -> usize {
    while i < bytes.len() && is_ident_cont(bytes[i]) {
        i += 1;
    }
    i
}

/// Count consecutive field accesses in a line (e.g., `a.b.c.d` = 4 segments).
/// Skips over string literals and bracket expressions to avoid false positives
/// on patterns like `tbl["H.N.S.W."]` where dots are inside strings.
pub fn count_consecutive_field_access(line: &str) -> usize {
    debug_assert!(!line.is_empty(), "line must not be empty");
    let mut max_depth = 0;
    let bytes = line.as_bytes();
    let mut i = 0;
    while i < bytes.len() {
        match bytes[i] {
            b'"' | b'\'' => {
                i = skip_lua_string(bytes, i);
            }
            b'[' => {
                i = skip_bracket_expr(bytes, i);
            }
            b if is_ident_start(b) => {
                let (depth, new_i) = measure_access_chain(bytes, i);
                i = new_i;
                max_depth = max_depth.max(depth);
            }
            _ => {
                i += 1;
            }
        }
    }
    max_depth
}

/// Measure one access chain starting at an identifier. Returns (depth, new_position).
fn measure_access_chain(bytes: &[u8], start: usize) -> (usize, usize) {
    let mut depth = 1;
    let mut i = skip_identifier(bytes, start);
    while i < bytes.len() {
        if bytes[i] == b'[' {
            depth += 1;
            i = skip_bracket_expr(bytes, i);
        } else if bytes[i] == b'.' && i + 1 < bytes.len() && is_ident_start(bytes[i + 1]) {
            depth += 1;
            i = skip_identifier(bytes, i + 1);
        } else {
            break;
        }
    }
    (depth, i)
}

/// Skip past a quoted string (single or double), returning position after closing quote.
fn skip_lua_string(bytes: &[u8], start: usize) -> usize {
    let quote = bytes[start];
    let mut i = start + 1;
    while i < bytes.len() {
        if bytes[i] == b'\\' {
            i += 2; // skip escaped character
            continue;
        }
        if bytes[i] == quote {
            return i + 1;
        }
        i += 1;
    }
    i // unterminated string, skip to end
}

/// Skip past a bracket expression `[...]`, handling nested brackets and strings.
fn skip_bracket_expr(bytes: &[u8], start: usize) -> usize {
    let mut i = start + 1;
    let mut depth = 1;
    while i < bytes.len() && depth > 0 {
        if bytes[i] == b'"' || bytes[i] == b'\'' {
            i = skip_lua_string(bytes, i);
            continue;
        }
        if bytes[i] == b'[' {
            depth += 1;
        } else if bytes[i] == b']' {
            depth -= 1;
        }
        i += 1;
    }
    i
}

/// Check if `line` contains `name` as a whole identifier (not substring).
pub(super) fn contains_identifier(line: &str, name: &str) -> bool {
    let mut start = 0;
    while let Some(pos) = line[start..].find(name) {
        let abs_pos = start + pos;
        let before_ok = abs_pos == 0 || !is_ident_cont(line.as_bytes()[abs_pos - 1]);
        let after_pos = abs_pos + name.len();
        let after_ok = after_pos >= line.len() || !is_ident_cont(line.as_bytes()[after_pos]);
        if before_ok && after_ok {
            return true;
        }
        start = abs_pos + 1;
    }
    false
}

/// Check if a line contains the `..` concat operator but not `...` (varargs).
pub(super) fn contains_concat_operator(line: &str) -> bool {
    let bytes = line.as_bytes();
    let mut i = 0;
    while i + 1 < bytes.len() {
        if bytes[i] == b'.' && bytes[i + 1] == b'.' {
            if i + 2 < bytes.len() && bytes[i + 2] == b'.' {
                i += 3;
                continue;
            }
            return true;
        }
        i += 1;
    }
    false
}

/// Check if a line has an inline suppression comment: `-- pmat:ignore CB-XXX`
pub(super) fn is_suppressed(original_lines: &[&str], line_num: usize, pattern_id: &str) -> bool {
    if line_num == 0 || line_num > original_lines.len() {
        return false;
    }
    let line = original_lines[line_num - 1];
    // Look for `-- pmat:ignore` with the specific pattern ID or bare `-- pmat:ignore`
    if let Some(pos) = line.find("-- pmat:ignore") {
        let after = &line[pos + 14..].trim_start();
        // Bare `-- pmat:ignore` suppresses all patterns on that line
        if after.is_empty() {
            return true;
        }
        // `-- pmat:ignore CB-603` suppresses specific pattern
        return after.contains(pattern_id);
    }
    false
}

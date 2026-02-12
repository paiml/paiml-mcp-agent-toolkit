#![cfg_attr(coverage_nightly, coverage(off))]
//! CB-600 Series: Lua Best Practices Detection
//!
//! Generic Lua defect detection for `pmat comply check`.
//! Based on: LuaTaint (Xiang et al. 2025), FLuaScan (Gao et al. 2023),
//! Luau type system (Brown et al. 2021/2023), luacheck W111/W113/W211.

use super::types::*;
use std::fs;
use std::path::{Path, PathBuf};

/// Known Lua standard library globals that should not be flagged by CB-600.
const LUA_STD_GLOBALS: &[&str] = &[
    "assert", "collectgarbage", "dofile", "error", "getmetatable", "ipairs",
    "load", "loadfile", "next", "pairs", "pcall", "print", "rawequal",
    "rawget", "rawlen", "rawset", "require", "select", "setmetatable",
    "tonumber", "tostring", "type", "unpack", "xpcall",
    // Standard library tables
    "coroutine", "debug", "io", "math", "os", "package", "string", "table",
    "utf8", "bit32", "arg",
    // Common environment globals
    "self", "true", "false", "nil", "_G", "_ENV", "_VERSION",
];

/// Directories to skip when walking for Lua files.
const SKIP_DIRS: &[&str] = &[
    ".git", "node_modules", "target", ".pmat", "vendor", "build", "dist",
];

// =============================================================================
// Helper functions
// =============================================================================

/// Walk directory recursively for `.lua` files, skipping common non-source dirs.
pub fn walkdir_lua_files(dir: &Path) -> Vec<PathBuf> {
    let mut files = Vec::new();
    walk_lua_recursive(dir, &mut files);
    files
}

fn walk_lua_recursive(dir: &Path, files: &mut Vec<PathBuf>) {
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
            let dir_name = path
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("");
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

        if trimmed.starts_with("--[[") {
            if trimmed[4..].contains("]]") {
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
fn is_in_lua_string(line: &str, pattern: &str) -> bool {
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
fn is_ident_start(b: u8) -> bool {
    b.is_ascii_alphabetic() || b == b'_'
}

/// Check if byte can continue a Lua identifier.
fn is_ident_cont(b: u8) -> bool {
    b.is_ascii_alphanumeric() || b == b'_'
}

/// Skip past an identifier at position `i`, returning new position.
fn skip_identifier(bytes: &[u8], mut i: usize) -> usize {
    while i < bytes.len() && is_ident_cont(bytes[i]) {
        i += 1;
    }
    i
}

/// Count consecutive field accesses in a line (e.g., `a.b.c.d` = 4 segments).
fn count_consecutive_field_access(line: &str) -> usize {
    let mut max_depth = 0;
    let bytes = line.as_bytes();
    let mut i = 0;
    while i < bytes.len() {
        if !is_ident_start(bytes[i]) {
            i += 1;
            continue;
        }
        let mut depth = 1;
        i = skip_identifier(bytes, i);
        while i < bytes.len() && bytes[i] == b'.' {
            i += 1;
            if i >= bytes.len() || !is_ident_start(bytes[i]) {
                break;
            }
            depth += 1;
            i = skip_identifier(bytes, i);
        }
        max_depth = max_depth.max(depth);
    }
    max_depth
}

// =============================================================================
// Detection functions
// =============================================================================

/// Lua keywords and control flow prefixes that cannot be implicit globals.
const LUA_KEYWORD_PREFIXES: &[&str] = &[
    "local ", "function ", "if ", "for ", "while ", "repeat", "return",
    "end", "else", "elseif ", "until ", "break", "goto ", "::",
];

/// Check if line starts with a Lua keyword/control flow statement.
fn starts_with_lua_keyword(trimmed: &str) -> bool {
    LUA_KEYWORD_PREFIXES.iter().any(|kw| trimmed.starts_with(kw))
}

/// Check if `=` at position is a comparison operator (==, ~=, <=, >=), not assignment.
fn is_comparison_eq(trimmed: &str, eq_pos: usize) -> bool {
    let bytes = trimmed.as_bytes();
    if eq_pos > 0 && matches!(bytes.get(eq_pos - 1), Some(b'~') | Some(b'<') | Some(b'>') | Some(b'=')) {
        return true;
    }
    bytes.get(eq_pos + 1) == Some(&b'=')
}

/// Extract implicit global name from an assignment line, or None if not an implicit global.
fn extract_implicit_global(trimmed: &str) -> Option<&str> {
    let eq_pos = trimmed.find('=')?;
    if is_comparison_eq(trimmed, eq_pos) {
        return None;
    }
    let lhs = trimmed[..eq_pos].trim();
    // Skip table field assignments (contains `.`, `[`, `:`)
    if lhs.contains('.') || lhs.contains('[') || lhs.contains(':') {
        return None;
    }
    // Must be a valid bare identifier, not a std global
    let is_valid_ident = !lhs.is_empty()
        && lhs.chars().all(|c| c.is_alphanumeric() || c == '_')
        && !lhs.starts_with(|c: char| c.is_ascii_digit());
    if is_valid_ident && !LUA_STD_GLOBALS.contains(&lhs) {
        Some(lhs)
    } else {
        None
    }
}

/// CB-600: Implicit Globals — assignment without `local` keyword.
/// Based on luacheck W111/W113.
pub fn detect_cb600_implicit_globals(project_path: &Path) -> Vec<CbPatternViolation> {
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
            if starts_with_lua_keyword(trimmed) {
                continue;
            }
            if let Some(lhs) = extract_implicit_global(trimmed) {
                violations.push(CbPatternViolation {
                    pattern_id: "CB-600".to_string(),
                    file: rel.clone(),
                    line: *line_num,
                    description: format!(
                        "Implicit global `{lhs}` — missing `local` keyword"
                    ),
                    severity: Severity::Warning,
                });
            }
        }
    }

    violations
}

/// CB-601: Nil-Unsafe Access — chained calls on function returns or deep field access.
/// Based on Luau type system and LuaTaint taint analysis.
pub fn detect_cb601_nil_unsafe_access(project_path: &Path) -> Vec<CbPatternViolation> {
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
            // Pattern 1: function return chained — `):` or `).`
            if (trimmed.contains(").") || trimmed.contains("):"))
                && !is_in_lua_string(trimmed, ").")
                && !is_in_lua_string(trimmed, "):")
            {
                violations.push(CbPatternViolation {
                    pattern_id: "CB-601".to_string(),
                    file: rel.clone(),
                    line: *line_num,
                    description: "Nil-unsafe: chained access on function return value"
                        .to_string(),
                    severity: Severity::Warning,
                });
                continue;
            }

            // Pattern 2: 3+ consecutive field accesses (a.b.c.d)
            if count_consecutive_field_access(trimmed) >= 4 {
                violations.push(CbPatternViolation {
                    pattern_id: "CB-601".to_string(),
                    file: rel.clone(),
                    line: *line_num,
                    description: "Nil-unsafe: deep field access chain (3+ levels)"
                        .to_string(),
                    severity: Severity::Warning,
                });
            }
        }
    }

    violations
}

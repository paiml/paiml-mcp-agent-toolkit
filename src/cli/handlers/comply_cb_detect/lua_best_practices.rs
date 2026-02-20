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
    "assert",
    "collectgarbage",
    "dofile",
    "error",
    "getmetatable",
    "ipairs",
    "load",
    "loadfile",
    "next",
    "pairs",
    "pcall",
    "print",
    "rawequal",
    "rawget",
    "rawlen",
    "rawset",
    "require",
    "select",
    "setmetatable",
    "tonumber",
    "tostring",
    "type",
    "unpack",
    "xpcall",
    // Standard library tables
    "coroutine",
    "debug",
    "io",
    "math",
    "os",
    "package",
    "string",
    "table",
    "utf8",
    "bit32",
    "arg",
    // Common environment globals
    "self",
    "true",
    "false",
    "nil",
    "_G",
    "_ENV",
    "_VERSION",
];

/// Directories to skip when walking for Lua files.
const SKIP_DIRS: &[&str] = &[
    ".git",
    "node_modules",
    "target",
    ".pmat",
    "vendor",
    "build",
    "dist",
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
/// Skips over string literals and bracket expressions to avoid false positives
/// on patterns like `tbl["H.N.S.W."]` where dots are inside strings.
pub(crate) fn count_consecutive_field_access(line: &str) -> usize {
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

// =============================================================================
// Detection functions
// =============================================================================

/// Lua keywords and control flow prefixes that cannot be implicit globals.
const LUA_KEYWORD_PREFIXES: &[&str] = &[
    "local ",
    "function ",
    "if ",
    "for ",
    "while ",
    "repeat",
    "return",
    "end",
    "else",
    "elseif ",
    "until ",
    "break",
    "goto ",
    "::",
];

/// Check if line starts with a Lua keyword/control flow statement.
fn starts_with_lua_keyword(trimmed: &str) -> bool {
    LUA_KEYWORD_PREFIXES
        .iter()
        .any(|kw| trimmed.starts_with(kw))
}

/// Check if `=` at position is a comparison operator (==, ~=, <=, >=), not assignment.
fn is_comparison_eq(trimmed: &str, eq_pos: usize) -> bool {
    let bytes = trimmed.as_bytes();
    if eq_pos > 0
        && matches!(
            bytes.get(eq_pos - 1),
            Some(b'~') | Some(b'<') | Some(b'>') | Some(b'=')
        )
    {
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

/// Count unbalanced braces on a line (outside strings), returning (opens, closes).
fn count_braces(line: &str) -> (i32, i32) {
    let mut opens = 0i32;
    let mut closes = 0i32;
    let mut in_dq = false;
    let mut in_sq = false;
    let bytes = line.as_bytes();
    let mut i = 0;
    while i < bytes.len() {
        match bytes[i] {
            b'"' if !in_sq => in_dq = !in_dq,
            b'\'' if !in_dq => in_sq = !in_sq,
            b'{' if !in_dq && !in_sq => opens += 1,
            b'}' if !in_dq && !in_sq => closes += 1,
            _ => {}
        }
        i += 1;
    }
    (opens, closes)
}

/// Extract comma-separated identifiers from a parameter/variable list string.
/// E.g., "a, b, c" -> ["a", "b", "c"]; "k, v" -> ["k", "v"]
fn extract_comma_separated_idents(s: &str) -> Vec<String> {
    s.split(',')
        .map(|part| part.trim())
        .filter(|part| !part.is_empty())
        .map(|part| {
            part.chars()
                .take_while(|c| c.is_alphanumeric() || *c == '_')
                .collect::<String>()
        })
        .filter(|name| !name.is_empty())
        .collect()
}

/// Collect all known local identifiers from a Lua file's production lines.
/// This includes: function parameters, for-loop variables, and local declarations.
fn collect_known_locals(prod_lines: &[(usize, String)]) -> std::collections::HashSet<String> {
    use std::collections::HashSet;
    let mut locals = HashSet::new();

    for (_, trimmed) in prod_lines {
        collect_function_params(trimmed, &mut locals);
        collect_for_loop_vars(trimmed, &mut locals);
        collect_local_decl_vars(trimmed, &mut locals);
    }

    locals
}

/// Extract function parameter names from lines like `function foo(a, b, c)`.
fn collect_function_params(trimmed: &str, locals: &mut std::collections::HashSet<String>) {
    // Match both `function name(...)` and `function M.name(...)` and `function M:name(...)`
    if let Some(open) = trimmed.find('(') {
        let prefix = &trimmed[..open];
        if prefix.contains("function") || prefix.trim_start().starts_with("function") {
            if let Some(close) = trimmed[open..].find(')') {
                let params = &trimmed[open + 1..open + close];
                for name in extract_comma_separated_idents(params) {
                    if name != "..." && name != "self" {
                        locals.insert(name);
                    }
                }
            }
        }
    }
}

/// Extract for-loop variable names from `for i = ...` and `for k, v in ...`.
fn collect_for_loop_vars(trimmed: &str, locals: &mut std::collections::HashSet<String>) {
    let rest = match trimmed.strip_prefix("for ") {
        Some(r) => r,
        None => return,
    };
    // Numeric for: `for i = 1, 10 do`
    // Generic for: `for k, v in pairs(t) do`
    // Find the delimiter: `=` for numeric, `in` for generic
    let var_part = if let Some(eq_pos) = rest.find('=') {
        let in_pos = rest.find(" in ");
        match in_pos {
            Some(ip) if ip < eq_pos => &rest[..ip],
            _ => &rest[..eq_pos],
        }
    } else if let Some(in_pos) = rest.find(" in ") {
        &rest[..in_pos]
    } else {
        return;
    };

    for name in extract_comma_separated_idents(var_part) {
        locals.insert(name);
    }
}

/// Extract variable names from `local x = ...` and `local a, b = ...` declarations.
fn collect_local_decl_vars(trimmed: &str, locals: &mut std::collections::HashSet<String>) {
    let after = match trimmed.strip_prefix("local ") {
        Some(a) => a,
        None => return,
    };
    if after.starts_with("function ") {
        return;
    }
    // Take everything before `=` (or the whole thing if no `=`)
    let var_part = match after.find('=') {
        Some(pos) => &after[..pos],
        None => after,
    };
    for name in extract_comma_separated_idents(var_part) {
        locals.insert(name);
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

        // Collect all known local identifiers: function params, for-loop vars, local decls
        let known_locals = collect_known_locals(&prod_lines);

        // Track brace depth: assignments inside { } are table constructor fields, not globals
        let mut brace_depth: i32 = 0;

        for (line_num, trimmed) in &prod_lines {
            let (opens, closes) = count_braces(trimmed);
            // Apply opens before checking (a line like `{ key = val }` starts inside)
            brace_depth += opens;

            if brace_depth <= 0 && !starts_with_lua_keyword(trimmed) {
                if let Some(lhs) = extract_implicit_global(trimmed) {
                    // Skip identifiers known to be local (params, loop vars, local decls)
                    if !known_locals.contains(lhs) {
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

            brace_depth -= closes;
            brace_depth = brace_depth.max(0);
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
                    description: "Nil-unsafe: chained access on function return value".to_string(),
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
                    description: "Nil-unsafe: deep field access chain (3+ levels)".to_string(),
                    severity: Severity::Warning,
                });
            }
        }
    }

    violations
}

/// CB-602: pcall Error Handling — uncaptured or unchecked pcall/xpcall.
/// Based on FLuaScan progressive taint analysis.
pub fn detect_cb602_pcall_error_handling(project_path: &Path) -> Vec<CbPatternViolation> {
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

/// Extract the first variable name from a pcall assignment.
/// E.g. `local wrap_ok, err = pcall(...)` → Some("wrap_ok")
///      `local ok = pcall(...)` → Some("ok")
///      `status = pcall(...)` → Some("status")
pub(crate) fn extract_pcall_status_var(line: &str) -> Option<String> {
    // Find the `=` that precedes pcall
    let eq_pos = line.find('=')?;
    let lhs = line[..eq_pos].trim();

    // Strip `local` prefix if present
    let lhs = lhs
        .strip_prefix("local")
        .map(|s| s.trim_start())
        .unwrap_or(lhs);

    // Take the first variable (before any comma for multi-return)
    let first_var = lhs.split(',').next()?.trim();

    if first_var.is_empty() || !first_var.chars().all(|c| c.is_alphanumeric() || c == '_') {
        return None;
    }

    Some(first_var.to_string())
}

/// Check if pcall status variable is checked within 5 lines after index `idx`.
/// Generic status-check patterns for common naming conventions.
const STATUS_CHECK_PATTERNS: &[&str] = &[
    "if ok",
    "if not ok",
    "if success",
    "if not success",
    "if status",
    "if not status",
    "assert(ok",
    "assert(success",
];

fn has_status_check(prod_lines: &[(usize, String)], idx: usize, status_var: Option<&str>) -> bool {
    let lookahead_end = std::cmp::min(idx + 6, prod_lines.len());
    prod_lines[idx + 1..lookahead_end]
        .iter()
        .any(|(_, l)| line_matches_status_check(l, status_var))
}

/// Check if a single line matches a status-check pattern.
fn line_matches_status_check(line: &str, status_var: Option<&str>) -> bool {
    // Check for the specific captured variable name (e.g. "if wrap_ok then")
    if let Some(var) = status_var {
        if line.contains(&format!("if {var}"))
            || line.contains(&format!("if not {var}"))
            || line.contains(&format!("assert({var}"))
        {
            return true;
        }
    }
    // Fallback: generic patterns
    STATUS_CHECK_PATTERNS.iter().any(|pat| line.contains(pat))
}

/// Deprecated Lua APIs that have modern replacements.
const LUA_DEPRECATED_APIS: &[&str] = &["loadstring(", "setfenv(", "getfenv(", "module("];

/// Dangerous Lua APIs that enable command injection.
const LUA_DANGEROUS_APIS: &[&str] = &["os.execute(", "io.popen("];

/// CB-603: Deprecated/Dangerous API usage.
/// Based on LuaTaint and FLuaScan — os.execute(), io.popen(), loadstring(), setfenv().
///
/// Supports inline suppression: `-- pmat:ignore CB-603` on the same line.
/// Distinguishes safe usage (hardcoded string arg) from dangerous (concatenation/variable).
pub fn detect_cb603_deprecated_dangerous_api(project_path: &Path) -> Vec<CbPatternViolation> {
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

/// Check if a line has an inline suppression comment: `-- pmat:ignore CB-XXX`
fn is_suppressed(original_lines: &[&str], line_num: usize, pattern_id: &str) -> bool {
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
/// `os.execute("make clean")` → true (safe)
/// `os.execute(cmd)` or `os.execute("rm " .. x)` → false (dangerous)
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

/// CB-604: Unused Variables — `local var = ...` where var is never referenced again.
/// Based on luacheck W211.
pub fn detect_cb604_unused_variables(project_path: &Path) -> Vec<CbPatternViolation> {
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

        let declarations = collect_local_declarations(&prod_lines);

        for (line_num, var_name) in &declarations {
            let count = prod_lines
                .iter()
                .filter(|(_, l)| contains_identifier(l, var_name))
                .count();
            if count <= 1 {
                violations.push(CbPatternViolation {
                    pattern_id: "CB-604".to_string(),
                    file: rel.clone(),
                    line: *line_num,
                    description: format!(
                        "Unused variable `{var_name}` — prefix with `_` if intentional"
                    ),
                    severity: Severity::Info,
                });
            }
        }
    }

    violations
}

/// Collect `local var = ...` declarations, excluding `local function` and `_`-prefixed vars.
fn collect_local_declarations(prod_lines: &[(usize, String)]) -> Vec<(usize, String)> {
    let mut declarations = Vec::new();
    for (line_num, trimmed) in prod_lines {
        if !trimmed.starts_with("local ") {
            continue;
        }
        let after_local = &trimmed[6..];
        if after_local.starts_with("function ") {
            continue;
        }
        let var_name: String = after_local
            .chars()
            .take_while(|c| c.is_alphanumeric() || *c == '_')
            .collect();
        if !var_name.is_empty() && !var_name.starts_with('_') {
            declarations.push((*line_num, var_name));
        }
    }
    declarations
}

/// Check if `line` contains `name` as a whole identifier (not substring).
fn contains_identifier(line: &str, name: &str) -> bool {
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

/// CB-605: String Concat in Loop — `..` operator inside for/while/repeat (O(n²)).
pub fn detect_cb605_string_concat_in_loop(project_path: &Path) -> Vec<CbPatternViolation> {
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

            if loop_depth > 0
                && contains_concat_operator(trimmed)
                && !is_in_lua_string(trimmed, "..")
            {
                violations.push(CbPatternViolation {
                    pattern_id: "CB-605".to_string(),
                    file: rel.clone(),
                    line: *line_num,
                    description: "String concatenation (`..`) in loop — O(n²), use table.concat()"
                        .to_string(),
                    severity: Severity::Info,
                });
            }

            if trimmed == "end" || trimmed.starts_with("end ") || trimmed.starts_with("until ") {
                loop_depth = (loop_depth - 1).max(0);
            }
        }
    }

    violations
}

/// Check if a line contains the `..` concat operator but not `...` (varargs).
fn contains_concat_operator(line: &str) -> bool {
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

/// CB-606: Missing Module Return — `local M = {}` pattern without final `return M`.
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

/// Extract module table variable name from first 20 production lines (e.g., `local M = {}`).
fn extract_module_table_var(prod_lines: &[(usize, String)]) -> Option<String> {
    prod_lines.iter().take(20).find_map(|(_, trimmed)| {
        if trimmed.starts_with("local ") && trimmed.contains("= {}") {
            let after_local = &trimmed[6..];
            let var: String = after_local
                .chars()
                .take_while(|c| c.is_alphanumeric() || *c == '_')
                .collect();
            if !var.is_empty() {
                Some(var)
            } else {
                None
            }
        } else {
            None
        }
    })
}

/// Standard library tables that commonly use dot notation (not colon).
const LUA_STD_TABLES: &[&str] = &[
    "math",
    "string",
    "table",
    "io",
    "os",
    "debug",
    "coroutine",
    "package",
    "utf8",
    "bit32",
];

/// CB-607: Colon/Dot Confusion — mixed `:` and `.` method calls on same table.
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

/// Extract the table name from a method call pattern: `name:method(` or `name.method(`.
fn extract_method_call(line: &str, separator: char) -> Option<String> {
    let sep_str = separator.to_string();
    for (i, _) in line.match_indices(&sep_str) {
        if i == 0 {
            continue;
        }
        let before = &line[..i];
        let name: String = before
            .chars()
            .rev()
            .take_while(|c| c.is_alphanumeric() || *c == '_')
            .collect::<String>()
            .chars()
            .rev()
            .collect();

        if name.is_empty() {
            continue;
        }

        let after = &line[i + 1..];
        let method: String = after
            .chars()
            .take_while(|c| c.is_alphanumeric() || *c == '_')
            .collect();

        if !method.is_empty() && after[method.len()..].starts_with('(') {
            return Some(name);
        }
    }
    None
}

// =============================================================================
// CB-608: Unchecked nil, err Return Pattern (#181)
// =============================================================================

/// Known Lua standard library functions that return `nil, err` on failure.
const NIL_ERR_FUNCTIONS: &[&str] = &[
    "io.open",
    "io.popen",
    "io.lines",
    "io.tmpfile",
    "os.execute",
    "os.rename",
    "os.remove",
    "load",
    "loadfile",
    "loadstring",
    "pcall",
    "xpcall",
    "require",
];

/// CB-608: Unchecked return nil, err — caller ignores error return.
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

/// CB-609: assert() in library code — terminates without allowing recovery.
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
// CB-610: Smarter String Concatenation — accumulator pattern only (#190)
// =============================================================================

/// CB-610: String accumulator in loop — `result = result .. x` is O(n²).
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

// =============================================================================
// CB-611: Weak Table Misuse Detection (#186)
// =============================================================================

/// CB-611: Detect weak table misuse patterns.
/// - String/numeric keys with `__mode = "k"` (never GC'd — value types)
/// - Unbounded caches without weak references or eviction
///
/// Reference: Kong, AwesomeWM, KOReader weak table usage.
pub fn detect_cb611_weak_table_misuse(project_path: &Path) -> Vec<CbPatternViolation> {
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

        detect_weak_key_with_value_types(&prod_lines, &rel, &mut violations);
    }

    violations
}

/// Check if a line declares a weak-key-only table (`__mode = "k"`, not "v" or "kv").
fn is_weak_key_only_declaration(line: &str) -> bool {
    line.contains("__mode")
        && (line.contains("\"k\"") || line.contains("'k'"))
        && !line.contains("\"v\"")
        && !line.contains("'v'")
        && !line.contains("\"kv\"")
        && !line.contains("'kv'")
}

/// Classify the key type after `var[...` — returns "string", "numeric", or None.
fn classify_bracket_key(after_bracket: &str) -> Option<&'static str> {
    if after_bracket.starts_with('"') || after_bracket.starts_with('\'') {
        Some("string")
    } else if after_bracket.starts_with(|c: char| c.is_ascii_digit()) {
        Some("numeric")
    } else {
        None
    }
}

/// Detect `__mode = "k"` tables being indexed with string or numeric keys.
fn detect_weak_key_with_value_types(
    prod_lines: &[(usize, String)],
    rel: &str,
    violations: &mut Vec<CbPatternViolation>,
) {
    // Phase 1: Find variables assigned weak-key tables
    let weak_key_vars: std::collections::HashSet<String> = prod_lines
        .iter()
        .filter(|(_, trimmed)| is_weak_key_only_declaration(trimmed))
        .filter_map(|(_, trimmed)| extract_weak_table_var(trimmed))
        .collect();

    if weak_key_vars.is_empty() {
        return;
    }

    // Phase 2: Check if weak-key vars are indexed with value-type literals
    for (line_num, trimmed) in prod_lines {
        for var in &weak_key_vars {
            let bracket_pattern = format!("{var}[");
            let Some(pos) = trimmed.find(&bracket_pattern) else {
                continue;
            };
            let after = &trimmed[pos + bracket_pattern.len()..];
            let Some(key_type) = classify_bracket_key(after) else {
                continue;
            };
            violations.push(CbPatternViolation {
                pattern_id: "CB-611".to_string(),
                file: rel.to_string(),
                line: *line_num,
                description: format!(
                    "Weak-key table `{var}` indexed with {key_type} key — \
                     {key_type} keys are value types and never GC'd, defeating __mode=\"k\""
                ),
                severity: Severity::Warning,
            });
        }
    }
}

/// Extract variable name from weak table assignment.
/// E.g. `local cache = setmetatable({}, { __mode = "k" })` → Some("cache")
fn extract_weak_table_var(line: &str) -> Option<String> {
    let eq_pos = line.find('=')?;
    let lhs = line[..eq_pos].trim();
    let lhs = lhs.strip_prefix("local ").unwrap_or(lhs).trim();
    if lhs.is_empty() || lhs.contains('.') || lhs.contains('[') {
        return None;
    }
    let var: String = lhs
        .chars()
        .take_while(|c| c.is_alphanumeric() || *c == '_')
        .collect();
    if var.is_empty() {
        None
    } else {
        Some(var)
    }
}

// =============================================================================
// CB-612: Lua Test Framework Detection (#184)
// =============================================================================

/// Detected Lua test framework.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LuaTestFramework {
    Busted,
    TestNginx,
    LuaUnit,
    Telescope,
    Custom,
}

impl std::fmt::Display for LuaTestFramework {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            LuaTestFramework::Busted => write!(f, "busted"),
            LuaTestFramework::TestNginx => write!(f, "Test::Nginx"),
            LuaTestFramework::LuaUnit => write!(f, "LuaUnit"),
            LuaTestFramework::Telescope => write!(f, "telescope"),
            LuaTestFramework::Custom => write!(f, "custom"),
        }
    }
}

/// CB-612: Auto-detect Lua test framework(s) and report as informational.
/// Supports hybrid projects (e.g., Kong uses both busted and Test::Nginx).
/// Reference: Kong, APISIX, xmake, KOReader framework patterns.
pub fn detect_cb612_test_framework(project_path: &Path) -> Vec<CbPatternViolation> {
    let frameworks = detect_lua_test_frameworks(project_path);
    let mut violations = Vec::new();

    if frameworks.is_empty() {
        // No Lua test framework detected — only flag if Lua files exist
        let lua_files = walkdir_lua_files(project_path);
        if lua_files.len() >= 3 {
            violations.push(CbPatternViolation {
                pattern_id: "CB-612".to_string(),
                file: "project".to_string(),
                line: 0,
                description: format!(
                    "No Lua test framework detected ({} Lua files) — consider adding busted or LuaUnit",
                    lua_files.len()
                ),
                severity: Severity::Info,
            });
        }
    } else {
        let names: Vec<String> = frameworks.iter().map(|f| f.to_string()).collect();
        violations.push(CbPatternViolation {
            pattern_id: "CB-612".to_string(),
            file: "project".to_string(),
            line: 0,
            description: format!("Lua test framework(s) detected: {}", names.join(", ")),
            severity: Severity::Info,
        });
    }

    violations
}

/// Check if test file content references a specific framework.
fn has_require_pattern(content: &str, module: &str) -> bool {
    content.contains(&format!("require(\"{module}\")"))
        || content.contains(&format!("require('{module}')"))
        || content.contains(&format!("require \"{module}\""))
}

/// Detect which Lua test frameworks are in use based on file patterns and require statements.
pub fn detect_lua_test_frameworks(project_path: &Path) -> Vec<LuaTestFramework> {
    let mut frameworks = Vec::new();

    if has_busted_indicators(project_path) {
        frameworks.push(LuaTestFramework::Busted);
    }
    if has_test_nginx_indicators(project_path) {
        frameworks.push(LuaTestFramework::TestNginx);
    }

    let (found_luaunit, found_telescope, found_custom) = scan_test_file_requires(project_path);

    if found_luaunit {
        frameworks.push(LuaTestFramework::LuaUnit);
    }
    if found_telescope {
        frameworks.push(LuaTestFramework::Telescope);
    }
    if found_custom && frameworks.is_empty() {
        frameworks.push(LuaTestFramework::Custom);
    }

    frameworks
}

/// Scan Lua test files for require('luaunit'), require('telescope'), or custom test patterns.
fn scan_test_file_requires(project_path: &Path) -> (bool, bool, bool) {
    let lua_files = walkdir_lua_files(project_path);
    let mut luaunit = false;
    let mut telescope = false;
    let mut custom = false;

    for file_path in &lua_files {
        if !is_lua_test_file(file_path) {
            continue;
        }
        let content = match fs::read_to_string(file_path) {
            Ok(c) => c,
            Err(_) => continue,
        };
        if !luaunit && has_require_pattern(&content, "luaunit") {
            luaunit = true;
        }
        if !telescope && has_require_pattern(&content, "telescope") {
            telescope = true;
        }
        if !custom
            && !luaunit
            && !telescope
            && (content.contains("function test_") || content.contains("function Test"))
        {
            custom = true;
        }
    }
    (luaunit, telescope, custom)
}

/// Check for busted test framework indicators.
fn has_busted_indicators(project_path: &Path) -> bool {
    // Check for .busted config file
    if project_path.join(".busted").exists() {
        return true;
    }
    // Check for _spec.lua files
    let lua_files = walkdir_lua_files(project_path);
    lua_files.iter().any(|p| {
        p.file_stem()
            .and_then(|s| s.to_str())
            .is_some_and(|s| s.ends_with("_spec"))
    })
}

/// Check for Test::Nginx indicators.
fn has_test_nginx_indicators(project_path: &Path) -> bool {
    let t_dir = project_path.join("t");
    if !t_dir.is_dir() {
        return false;
    }
    // Look for .t files in t/ directory
    match fs::read_dir(&t_dir) {
        Ok(entries) => entries
            .flatten()
            .any(|e| e.path().extension().map(|ext| ext == "t").unwrap_or(false)),
        Err(_) => false,
    }
}

// =============================================================================
// CB-613: Lua Require Cycle Detection (#187)
// =============================================================================

/// CB-613: Detect circular require() chains in Lua projects.
/// Builds a directed graph from top-level require() calls and finds cycles via DFS.
/// Function-scoped requires are excluded (they're safe — deferred loading).
pub fn detect_cb613_require_cycles(project_path: &Path) -> Vec<CbPatternViolation> {
    let files = walkdir_lua_files(project_path);
    if files.len() < 2 {
        return Vec::new();
    }

    // Build require graph: module_name -> Vec<required_module>
    let graph = build_require_graph(project_path, &files);
    if graph.is_empty() {
        return Vec::new();
    }

    // Find cycles via DFS
    let cycles = find_require_cycles(&graph);
    cycles
        .into_iter()
        .map(|cycle| {
            let chain = cycle.join(" -> ");
            CbPatternViolation {
                pattern_id: "CB-613".to_string(),
                file: cycle
                    .first()
                    .map(|s| format!("{s}.lua"))
                    .unwrap_or_default(),
                line: 0,
                description: format!("Circular require chain: {chain}"),
                severity: Severity::Warning,
            }
        })
        .collect()
}

/// Build a directed graph of top-level require() calls.
/// Returns module_name -> Vec<required_module_name>.
fn build_require_graph(
    project_path: &Path,
    files: &[PathBuf],
) -> std::collections::HashMap<String, Vec<String>> {
    let mut graph: std::collections::HashMap<String, Vec<String>> =
        std::collections::HashMap::new();

    for file_path in files {
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
            .with_extension("")
            .display()
            .to_string()
            .replace(['/', '\\'], ".");

        let requires = extract_top_level_requires(&content);
        if !requires.is_empty() {
            graph.insert(rel, requires);
        }
    }

    graph
}

/// Extract module names from top-level require() calls (not inside functions).
fn extract_top_level_requires(content: &str) -> Vec<String> {
    let mut requires = Vec::new();
    let mut func_depth: i32 = 0;

    for line in content.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with("--") {
            continue;
        }
        // Track function nesting
        if trimmed.starts_with("function ")
            || trimmed.starts_with("local function ")
            || trimmed.contains("= function(")
        {
            func_depth += 1;
        }
        // Only capture top-level requires
        if func_depth == 0 {
            if let Some(module) = extract_require_module(trimmed) {
                requires.push(module);
            }
        }
        if trimmed == "end" || trimmed.starts_with("end ") || trimmed.starts_with("end)") {
            func_depth = (func_depth - 1).max(0);
        }
    }
    requires
}

/// Extract module name from a require() call.
/// Matches: require("foo"), require('foo'), require "foo", require 'foo'
fn extract_require_module(line: &str) -> Option<String> {
    let req_idx = line.find("require")?;
    let after = line[req_idx + 7..].trim();
    // Skip if require is part of a larger word
    if req_idx > 0 {
        let before_char = line.as_bytes()[req_idx - 1];
        if before_char.is_ascii_alphanumeric() || before_char == b'_' {
            return None;
        }
    }
    let after = after.strip_prefix('(').unwrap_or(after).trim();
    let (quote, rest) = if let Some(stripped) = after.strip_prefix('"') {
        ('"', stripped)
    } else if let Some(stripped) = after.strip_prefix('\'') {
        ('\'', stripped)
    } else {
        return None;
    };
    let end = rest.find(quote)?;
    let module = rest[..end].to_string();
    if module.is_empty() {
        None
    } else {
        Some(module)
    }
}

/// Find cycles in the require graph using DFS.
fn find_require_cycles(graph: &std::collections::HashMap<String, Vec<String>>) -> Vec<Vec<String>> {
    use std::collections::HashSet;
    let mut cycles = Vec::new();
    let mut visited = HashSet::new();
    let mut rec_stack = Vec::new();

    for start in graph.keys() {
        if visited.contains(start) {
            continue;
        }
        dfs_find_cycle(start, graph, &mut visited, &mut rec_stack, &mut cycles);
    }
    cycles
}

/// DFS helper to detect back edges (cycles).
fn dfs_find_cycle(
    node: &str,
    graph: &std::collections::HashMap<String, Vec<String>>,
    visited: &mut std::collections::HashSet<String>,
    rec_stack: &mut Vec<String>,
    cycles: &mut Vec<Vec<String>>,
) {
    visited.insert(node.to_string());
    rec_stack.push(node.to_string());

    if let Some(deps) = graph.get(node) {
        for dep in deps {
            if let Some(pos) = rec_stack.iter().position(|n| n == dep) {
                // Found a cycle: extract the cycle from stack
                let cycle: Vec<String> = rec_stack[pos..].to_vec();
                cycles.push(cycle);
            } else if !visited.contains(dep.as_str()) {
                dfs_find_cycle(dep, graph, visited, rec_stack, cycles);
            }
        }
    }

    rec_stack.pop();
}

include!("lua_best_practices_part2.rs");

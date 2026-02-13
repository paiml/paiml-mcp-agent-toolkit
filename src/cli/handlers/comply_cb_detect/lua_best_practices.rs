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
            b'"' | b'\'' => { i = skip_lua_string(bytes, i); }
            b'[' => { i = skip_bracket_expr(bytes, i); }
            b if is_ident_start(b) => {
                let (depth, new_i) = measure_access_chain(bytes, i);
                i = new_i;
                max_depth = max_depth.max(depth);
            }
            _ => { i += 1; }
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

            if brace_depth <= 0 {
                if !starts_with_lua_keyword(trimmed) {
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
                    description: "pcall/xpcall status not checked within 5 lines"
                        .to_string(),
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
    let lhs = lhs.strip_prefix("local").map(|s| s.trim_start()).unwrap_or(lhs);

    // Take the first variable (before any comma for multi-return)
    let first_var = lhs.split(',').next()?.trim();

    if first_var.is_empty() || !first_var.chars().all(|c| c.is_alphanumeric() || c == '_') {
        return None;
    }

    Some(first_var.to_string())
}

/// Check if pcall status variable is checked within 5 lines after index `idx`.
fn has_status_check(prod_lines: &[(usize, String)], idx: usize, status_var: Option<&str>) -> bool {
    let lookahead_end = std::cmp::min(idx + 6, prod_lines.len());
    prod_lines[idx + 1..lookahead_end].iter().any(|(_, l)| {
        // Check for the specific captured variable name (e.g. "if wrap_ok then")
        if let Some(var) = status_var {
            if l.contains(&format!("if {var}"))
                || l.contains(&format!("if not {var}"))
                || l.contains(&format!("assert({var}"))
            {
                return true;
            }
        }

        // Fallback: generic patterns for common naming conventions
        l.contains("if ok") || l.contains("if not ok")
            || l.contains("if success") || l.contains("if not success")
            || l.contains("if status") || l.contains("if not status")
            || l.contains("assert(ok") || l.contains("assert(success")
    })
}

/// Deprecated Lua APIs that have modern replacements.
const LUA_DEPRECATED_APIS: &[&str] = &["loadstring(", "setfenv(", "getfenv(", "module("];

/// Dangerous Lua APIs that enable command injection.
const LUA_DANGEROUS_APIS: &[&str] = &["os.execute(", "io.popen("];

/// CB-603: Deprecated/Dangerous API usage.
/// Based on LuaTaint and FLuaScan — os.execute(), io.popen(), loadstring(), setfenv().
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
        let prod_lines = compute_lua_production_lines(&content);
        let rel = file_path
            .strip_prefix(project_path)
            .unwrap_or(file_path)
            .display()
            .to_string();

        for (line_num, trimmed) in &prod_lines {
            check_deprecated_apis(trimmed, &rel, *line_num, &mut violations);
            check_dangerous_apis(trimmed, &rel, *line_num, &mut violations);
        }
    }

    violations
}

fn check_deprecated_apis(trimmed: &str, rel: &str, line_num: usize, violations: &mut Vec<CbPatternViolation>) {
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

fn check_dangerous_apis(trimmed: &str, rel: &str, line_num: usize, violations: &mut Vec<CbPatternViolation>) {
    for api in LUA_DANGEROUS_APIS {
        if trimmed.contains(api) && !is_in_lua_string(trimmed, api) {
            violations.push(CbPatternViolation {
                pattern_id: "CB-603".to_string(),
                file: rel.to_string(),
                line: line_num,
                description: format!(
                    "Dangerous API: `{}` — potential command injection",
                    api.trim_end_matches('(')
                ),
                severity: Severity::Warning,
            });
        }
    }
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
                    description: format!("Unused variable `{var_name}` — prefix with `_` if intentional"),
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
        let before_ok = abs_pos == 0
            || !is_ident_cont(line.as_bytes()[abs_pos - 1]);
        let after_pos = abs_pos + name.len();
        let after_ok = after_pos >= line.len()
            || !is_ident_cont(line.as_bytes()[after_pos]);
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

            if loop_depth > 0 && contains_concat_operator(trimmed) && !is_in_lua_string(trimmed, "..") {
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
            let has_return = prod_lines
                .iter()
                .rev()
                .any(|(_, trimmed)| {
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
            if !var.is_empty() { Some(var) } else { None }
        } else {
            None
        }
    })
}

/// Standard library tables that commonly use dot notation (not colon).
const LUA_STD_TABLES: &[&str] = &[
    "math", "string", "table", "io", "os", "debug", "coroutine",
    "package", "utf8", "bit32",
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

#![cfg_attr(coverage_nightly, coverage(off))]
//! Shared detection helper functions for Lua CB pattern checks.
//! Includes keyword checks, implicit global extraction, brace counting,
//! local variable collection, and pcall status checking.

use super::constants::{LUA_KEYWORD_PREFIXES, LUA_STD_GLOBALS, STATUS_CHECK_PATTERNS};

/// Check if line starts with a Lua keyword/control flow statement.
pub(super) fn starts_with_lua_keyword(trimmed: &str) -> bool {
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
pub(super) fn extract_implicit_global(trimmed: &str) -> Option<&str> {
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
pub(super) fn count_braces(line: &str) -> (i32, i32) {
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
pub(super) fn collect_known_locals(
    prod_lines: &[(usize, String)],
) -> std::collections::HashSet<String> {
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

/// Extract the first variable name from a pcall assignment.
/// E.g. `local wrap_ok, err = pcall(...)` -> Some("wrap_ok")
///      `local ok = pcall(...)` -> Some("ok")
///      `status = pcall(...)` -> Some("status")
pub fn extract_pcall_status_var(line: &str) -> Option<String> {
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
pub(super) fn has_status_check(
    prod_lines: &[(usize, String)],
    idx: usize,
    status_var: Option<&str>,
) -> bool {
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

/// Collect `local var = ...` declarations, excluding `local function` and `_`-prefixed vars.
pub(super) fn collect_local_declarations(prod_lines: &[(usize, String)]) -> Vec<(usize, String)> {
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

/// Extract module table variable name from first 20 production lines (e.g., `local M = {}`).
pub(super) fn extract_module_table_var(prod_lines: &[(usize, String)]) -> Option<String> {
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

/// Extract the table name from a method call pattern: `name:method(` or `name.method(`.
pub(super) fn extract_method_call(line: &str, separator: char) -> Option<String> {
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

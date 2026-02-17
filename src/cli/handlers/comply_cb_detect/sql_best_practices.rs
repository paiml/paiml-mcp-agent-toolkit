#![cfg_attr(coverage_nightly, coverage(off))]
//! CB-700 Series: SQL Best Practices Detection
//!
//! Pattern-based SQL defect detection for `pmat comply check`.
//! Based on: Chamberlin & Boyce (1974), SQL anti-pattern literature,
//! OWASP SQL injection guidelines.

use super::types::*;
use std::fs;
use std::path::{Path, PathBuf};

/// Directories to skip when walking for SQL files.
const SKIP_DIRS: &[&str] = &[
    ".git",
    "node_modules",
    "target",
    ".pmat",
    "vendor",
    "build",
    "dist",
    "migrations_backup",
    "__pycache__",
];

/// SQL keywords that start statements (not assignments).
const SQL_KEYWORDS: &[&str] = &[
    "select", "insert", "update", "delete", "create", "alter", "drop", "grant", "revoke", "begin",
    "commit", "rollback", "explain", "with", "merge", "truncate", "set", "declare", "exec",
    "execute",
];

// =============================================================================
// File walking
// =============================================================================

/// Walk directory recursively for `.sql` files.
pub fn walkdir_sql_files(dir: &Path) -> Vec<PathBuf> {
    let mut files = Vec::new();
    walk_sql_recursive(dir, &mut files);
    files
}

fn walk_sql_recursive(dir: &Path, files: &mut Vec<PathBuf>) {
    let entries = match fs::read_dir(dir) {
        Ok(e) => e,
        Err(_) => return,
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            let dir_name = path.file_name().and_then(|n| n.to_str()).unwrap_or("");
            if !SKIP_DIRS.contains(&dir_name) {
                walk_sql_recursive(&path, files);
            }
        } else if path
            .extension()
            .and_then(|e| e.to_str())
            .map(|e| matches!(e, "sql" | "ddl" | "dml"))
            .unwrap_or(false)
        {
            files.push(path);
        }
    }
}

/// Check if a SQL file is a test file.
pub fn is_sql_test_file(path: &Path) -> bool {
    let stem = path.file_stem().and_then(|s| s.to_str()).unwrap_or("");
    if stem.ends_with("_test") || stem.starts_with("test_") {
        return true;
    }
    path.components().any(|c| {
        let s = c.as_os_str().to_str().unwrap_or("");
        s == "tests" || s == "test" || s == "fixtures" || s == "testdata"
    })
}

/// Compute production lines (strip SQL comments).
pub fn compute_sql_production_lines(content: &str) -> Vec<(usize, String)> {
    let mut result = Vec::new();
    let mut in_block_comment = false;

    for (i, line) in content.lines().enumerate() {
        let trimmed = line.trim();

        if in_block_comment {
            if trimmed.contains("*/") {
                in_block_comment = false;
            }
            continue;
        }

        if trimmed.starts_with("/*") {
            if trimmed.contains("*/") {
                continue;
            }
            in_block_comment = true;
            continue;
        }

        if trimmed.starts_with("--") || trimmed.is_empty() {
            continue;
        }

        // Strip inline comments
        let line_content = if let Some(pos) = trimmed.find("--") {
            trimmed[..pos].trim()
        } else {
            trimmed
        };

        if !line_content.is_empty() {
            result.push((i + 1, line_content.to_string()));
        }
    }

    result
}

// =============================================================================
// CB-700: SELECT * Usage
// =============================================================================

pub fn detect_cb700_select_star(project_path: &Path) -> Vec<CbPatternViolation> {
    let files = walkdir_sql_files(project_path);
    let mut violations = Vec::new();

    for file_path in &files {
        if is_sql_test_file(file_path) {
            continue;
        }
        let content = match fs::read_to_string(file_path) {
            Ok(c) => c,
            Err(_) => continue,
        };
        let prod_lines = compute_sql_production_lines(&content);
        let rel = file_path
            .strip_prefix(project_path)
            .unwrap_or(file_path)
            .display()
            .to_string();

        for (line_num, line) in &prod_lines {
            let lower = line.to_lowercase();
            // Match SELECT * but not SELECT COUNT(*) or SELECT EXISTS(*)
            if lower.contains("select")
                && lower.contains('*')
                && !lower.contains("count(*")
                && !lower.contains("count (*")
                && !lower.contains("exists(*")
                && !lower.contains("exists (*")
            {
                // Check it's actually SELECT * pattern
                if let Some(sel_pos) = lower.find("select") {
                    let after_select = &lower[sel_pos + 6..].trim_start();
                    if after_select.starts_with('*')
                        || after_select.starts_with("distinct *")
                        || after_select.starts_with("all *")
                    {
                        violations.push(CbPatternViolation {
                            pattern_id: "CB-700".to_string(),
                            file: rel.clone(),
                            line: *line_num,
                            description: "SELECT * — specify columns explicitly".to_string(),
                            severity: Severity::Warning,
                        });
                    }
                }
            }
        }
    }

    violations
}

// =============================================================================
// CB-701: Missing WHERE on UPDATE/DELETE
// =============================================================================

pub fn detect_cb701_missing_where(project_path: &Path) -> Vec<CbPatternViolation> {
    let files = walkdir_sql_files(project_path);
    let mut violations = Vec::new();

    for file_path in &files {
        if is_sql_test_file(file_path) {
            continue;
        }
        let content = match fs::read_to_string(file_path) {
            Ok(c) => c,
            Err(_) => continue,
        };
        let prod_lines = compute_sql_production_lines(&content);
        let rel = file_path
            .strip_prefix(project_path)
            .unwrap_or(file_path)
            .display()
            .to_string();

        // Join lines to handle multi-line statements
        let joined = prod_lines
            .iter()
            .map(|(_, l)| l.as_str())
            .collect::<Vec<_>>()
            .join(" ");
        let lower = joined.to_lowercase();

        // Split on semicolons to get individual statements
        for statement in lower.split(';') {
            let trimmed = statement.trim();
            if (trimmed.starts_with("update ") || trimmed.starts_with("delete "))
                && !trimmed.contains("where")
            {
                // Find which line this statement starts on
                let start_word = if trimmed.starts_with("update") {
                    "update"
                } else {
                    "delete"
                };
                let line_num = prod_lines
                    .iter()
                    .find(|(_, l)| l.to_lowercase().contains(start_word))
                    .map(|(n, _)| *n)
                    .unwrap_or(1);

                violations.push(CbPatternViolation {
                    pattern_id: "CB-701".to_string(),
                    file: rel.clone(),
                    line: line_num,
                    description: format!(
                        "{} without WHERE clause — affects all rows",
                        start_word.to_uppercase()
                    ),
                    severity: Severity::Error,
                });
            }
        }
    }

    violations
}

// =============================================================================
// CB-702: Implicit JOIN (Comma Join)
// =============================================================================

/// Check if a FROM clause has comma-separated tables (implicit join).
fn has_implicit_join(lower: &str) -> bool {
    let from_pos = match lower.find("from ") {
        Some(p) => p,
        None => return false,
    };
    let after_from = &lower[from_pos + 5..];

    // Skip if has unbalanced parens (subquery)
    let paren_depth: i32 = after_from
        .chars()
        .map(|c| match c {
            '(' => 1,
            ')' => -1,
            _ => 0,
        })
        .sum();
    if paren_depth != 0 || !after_from.contains(',') {
        return false;
    }

    let parts: Vec<&str> = after_from.split(',').collect();
    parts.len() >= 2
        && parts[0].trim().chars().any(|c| c.is_alphanumeric())
        && parts[1].trim().chars().any(|c| c.is_alphanumeric())
}

pub fn detect_cb702_implicit_join(project_path: &Path) -> Vec<CbPatternViolation> {
    let files = walkdir_sql_files(project_path);
    let mut violations = Vec::new();

    for file_path in &files {
        if is_sql_test_file(file_path) {
            continue;
        }
        let content = match fs::read_to_string(file_path) {
            Ok(c) => c,
            Err(_) => continue,
        };
        let prod_lines = compute_sql_production_lines(&content);
        let rel = file_path
            .strip_prefix(project_path)
            .unwrap_or(file_path)
            .display()
            .to_string();

        for (line_num, line) in &prod_lines {
            if has_implicit_join(&line.to_lowercase()) {
                violations.push(CbPatternViolation {
                    pattern_id: "CB-702".to_string(),
                    file: rel.clone(),
                    line: *line_num,
                    description: "Implicit JOIN (comma syntax) — use explicit JOIN".to_string(),
                    severity: Severity::Warning,
                });
            }
        }
    }

    violations
}

// =============================================================================
// CB-703: SQL Injection Pattern
// =============================================================================

pub fn detect_cb703_sql_injection(project_path: &Path) -> Vec<CbPatternViolation> {
    let files = walkdir_sql_files(project_path);
    let mut violations = Vec::new();

    // Also check non-SQL files for SQL string construction
    let code_extensions = ["py", "rs", "js", "ts", "rb", "php", "java", "go", "lua"];
    let mut code_files: Vec<PathBuf> = Vec::new();
    collect_code_files(project_path, &code_extensions, &mut code_files);

    for file_path in code_files.iter().chain(files.iter()) {
        let content = match fs::read_to_string(file_path) {
            Ok(c) => c,
            Err(_) => continue,
        };
        let rel = file_path
            .strip_prefix(project_path)
            .unwrap_or(file_path)
            .display()
            .to_string();

        for (i, line) in content.lines().enumerate() {
            let trimmed = line.trim();
            let lower = trimmed.to_lowercase();

            // Look for SQL keywords + string concatenation patterns
            let has_sql_keyword = SQL_KEYWORDS.iter().any(|kw| lower.contains(kw));
            if !has_sql_keyword {
                continue;
            }

            // Pattern: string concatenation with SQL
            let has_concat = trimmed.contains("+ \"")
                || trimmed.contains("\" +")
                || trimmed.contains(".. \"")
                || trimmed.contains("\" ..")
                || trimmed.contains("f\"")  // Python f-strings
                || trimmed.contains("format!")  // Rust format
                || (trimmed.contains('{') && trimmed.contains('}') && lower.contains("select"));

            if has_concat && !is_test_context(file_path) {
                violations.push(CbPatternViolation {
                    pattern_id: "CB-703".to_string(),
                    file: rel.clone(),
                    line: i + 1,
                    description: "SQL string concatenation — use parameterized queries".to_string(),
                    severity: Severity::Warning,
                });
            }
        }
    }

    violations
}

fn collect_code_files(dir: &Path, extensions: &[&str], files: &mut Vec<PathBuf>) {
    for entry in fs::read_dir(dir).into_iter().flatten().flatten() {
        let path = entry.path();
        let name = path.file_name().and_then(|n| n.to_str()).unwrap_or("");
        if path.is_dir() && !SKIP_DIRS.contains(&name) {
            collect_code_files(&path, extensions, files);
        } else if path
            .extension()
            .and_then(|e| e.to_str())
            .map(|e| extensions.contains(&e))
            .unwrap_or(false)
        {
            files.push(path);
        }
    }
}

fn is_test_context(path: &Path) -> bool {
    let s = path.to_str().unwrap_or("");
    s.contains("test") || s.contains("spec") || s.contains("fixture")
}

// =============================================================================
// CB-704: Missing Index Hint (placeholder — Info only)
// =============================================================================

pub fn detect_cb704_missing_index_hint(project_path: &Path) -> Vec<CbPatternViolation> {
    let files = walkdir_sql_files(project_path);
    let mut violations = Vec::new();

    for file_path in &files {
        if is_sql_test_file(file_path) {
            continue;
        }
        let content = match fs::read_to_string(file_path) {
            Ok(c) => c,
            Err(_) => continue,
        };
        let prod_lines = compute_sql_production_lines(&content);
        let rel = file_path
            .strip_prefix(project_path)
            .unwrap_or(file_path)
            .display()
            .to_string();

        // Count JOINs without index hints
        let mut join_count = 0;
        for (_, line) in &prod_lines {
            let lower = line.to_lowercase();
            join_count += lower.matches(" join ").count();
        }

        if join_count > 3 {
            let line_num = prod_lines
                .iter()
                .find(|(_, l)| l.to_lowercase().contains(" join "))
                .map(|(n, _)| *n)
                .unwrap_or(1);

            violations.push(CbPatternViolation {
                pattern_id: "CB-704".to_string(),
                file: rel.clone(),
                line: line_num,
                description: format!(
                    "{} JOINs in file — consider adding index hints or reviewing query plan",
                    join_count
                ),
                severity: Severity::Info,
            });
        }
    }

    violations
}

// =============================================================================
// CB-705: N+1 Query Pattern
// =============================================================================

/// Detect SQL queries embedded inside loops in co-located code files.
/// Looks for patterns like `for row in results: cursor.execute("SELECT ...")`.
pub fn detect_cb705_n_plus_1_query(project_path: &Path) -> Vec<CbPatternViolation> {
    let code_extensions = ["py", "rb", "php", "java", "js", "ts"];
    let mut code_files: Vec<PathBuf> = Vec::new();
    collect_code_files(project_path, &code_extensions, &mut code_files);

    let mut violations = Vec::new();

    for file_path in &code_files {
        if is_test_context(file_path) {
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

        detect_n_plus_1_in_content(&content, &rel, &mut violations);
    }

    violations
}

fn detect_n_plus_1_in_content(content: &str, rel: &str, violations: &mut Vec<CbPatternViolation>) {
    let mut in_loop = false;
    let mut loop_depth: i32 = 0;

    for (i, line) in content.lines().enumerate() {
        let trimmed = line.trim().to_lowercase();

        // Detect loop starts
        if trimmed.starts_with("for ")
            || trimmed.starts_with("while ")
            || trimmed.starts_with("foreach")
            || trimmed.contains(".foreach")
            || trimmed.contains(".map(")
            || trimmed.contains(".each ")
        {
            in_loop = true;
            loop_depth += 1;
        }

        // Track braces for loop scope
        for c in trimmed.chars() {
            if c == '}' && in_loop {
                loop_depth -= 1;
                if loop_depth <= 0 {
                    in_loop = false;
                    loop_depth = 0;
                }
            }
        }

        // Check for SQL execution inside loop
        if in_loop {
            let has_sql_exec = trimmed.contains("execute(")
                || trimmed.contains(".query(")
                || trimmed.contains(".raw(")
                || trimmed.contains("cursor.execute")
                || trimmed.contains("db.query")
                || trimmed.contains("connection.execute");

            let has_sql_keyword = trimmed.contains("select ")
                || trimmed.contains("insert ")
                || trimmed.contains("update ")
                || trimmed.contains("delete ");

            if has_sql_exec && has_sql_keyword {
                violations.push(CbPatternViolation {
                    pattern_id: "CB-705".to_string(),
                    file: rel.to_string(),
                    line: i + 1,
                    description: "N+1 query pattern — SQL execution inside loop".to_string(),
                    severity: Severity::Info,
                });
            }
        }
    }
}

#![cfg_attr(coverage_nightly, coverage(off))]
//! CB-800 Series: Scala Best Practices Detection
//!
//! Pattern-based Scala defect detection for `pmat comply check`.
//! Based on: Odersky et al. (2004), Scalastyle, WartRemover,
//! Scala style guide conventions.

use super::types::*;
use std::fs;
use std::path::{Path, PathBuf};

/// Directories to skip when walking for Scala files.
const SKIP_DIRS: &[&str] = &[
    ".git",
    "node_modules",
    "target",
    ".pmat",
    "vendor",
    "build",
    "dist",
    ".bsp",
    ".metals",
    ".bloop",
    ".idea",
    "project/target",
];

// =============================================================================
// File walking
// =============================================================================

/// Walk directory recursively for `.scala` and `.sc` files.
pub fn walkdir_scala_files(dir: &Path) -> Vec<PathBuf> {
    let mut files = Vec::new();
    walk_scala_recursive(dir, &mut files);
    files
}

fn walk_scala_recursive(dir: &Path, files: &mut Vec<PathBuf>) {
    let entries = match fs::read_dir(dir) {
        Ok(e) => e,
        Err(_) => return,
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            let dir_name = path.file_name().and_then(|n| n.to_str()).unwrap_or("");
            if !SKIP_DIRS.contains(&dir_name) {
                walk_scala_recursive(&path, files);
            }
        } else if path
            .extension()
            .and_then(|e| e.to_str())
            .map(|e| matches!(e, "scala" | "sc"))
            .unwrap_or(false)
        {
            files.push(path);
        }
    }
}

/// Check if a Scala file is a test file.
pub fn is_scala_test_file(path: &Path) -> bool {
    let stem = path.file_stem().and_then(|s| s.to_str()).unwrap_or("");
    if stem.ends_with("Test")
        || stem.ends_with("Spec")
        || stem.ends_with("Suite")
        || stem.starts_with("Test")
    {
        return true;
    }
    path.components().any(|c| {
        let s = c.as_os_str().to_str().unwrap_or("");
        s == "test" || s == "tests" || s == "it" || s == "spec"
    })
}

/// Compute production lines (strip Scala comments).
pub fn compute_scala_production_lines(content: &str) -> Vec<(usize, String)> {
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
            if !trimmed.contains("*/") {
                in_block_comment = true;
            }
            continue;
        }

        if trimmed.starts_with("//") || trimmed.is_empty() {
            continue;
        }

        // Strip inline comments
        let line_content = if let Some(pos) = trimmed.find("//") {
            // Avoid stripping URLs (http://) and string literals
            if pos > 0 && &trimmed[pos - 1..pos] == ":" {
                trimmed
            } else {
                trimmed[..pos].trim()
            }
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
// CB-800: Mutable Collection Usage
// =============================================================================

/// Mutable collection types that should be avoided in non-local scope.
const MUTABLE_COLLECTIONS: &[&str] = &[
    "mutable.Map",
    "mutable.Set",
    "mutable.Buffer",
    "mutable.ListBuffer",
    "mutable.ArrayBuffer",
    "mutable.HashMap",
    "mutable.HashSet",
    "mutable.LinkedHashMap",
    "mutable.LinkedHashSet",
    "mutable.Queue",
    "mutable.Stack",
    "mutable.TreeMap",
    "mutable.TreeSet",
];

pub fn detect_cb800_mutable_collection(project_path: &Path) -> Vec<CbPatternViolation> {
    let files = walkdir_scala_files(project_path);
    let mut violations = Vec::new();

    for file_path in &files {
        if is_scala_test_file(file_path) {
            continue;
        }
        let content = match fs::read_to_string(file_path) {
            Ok(c) => c,
            Err(_) => continue,
        };
        let prod_lines = compute_scala_production_lines(&content);
        let rel = file_path
            .strip_prefix(project_path)
            .unwrap_or(file_path)
            .display()
            .to_string();

        for (line_num, line) in &prod_lines {
            // Skip import lines (imports are fine)
            if line.starts_with("import ") {
                continue;
            }
            for mc in MUTABLE_COLLECTIONS {
                if line.contains(mc) {
                    violations.push(CbPatternViolation {
                        pattern_id: "CB-800".to_string(),
                        file: rel.clone(),
                        line: *line_num,
                        description: format!(
                            "Mutable collection `{}` — prefer immutable collections",
                            mc
                        ),
                        severity: Severity::Warning,
                    });
                    break; // One violation per line
                }
            }
        }
    }

    violations
}

// =============================================================================
// CB-801: Null Usage
// =============================================================================

pub fn detect_cb801_null_usage(project_path: &Path) -> Vec<CbPatternViolation> {
    let files = walkdir_scala_files(project_path);
    let mut violations = Vec::new();

    for file_path in &files {
        if is_scala_test_file(file_path) {
            continue;
        }
        let content = match fs::read_to_string(file_path) {
            Ok(c) => c,
            Err(_) => continue,
        };
        let prod_lines = compute_scala_production_lines(&content);
        let rel = file_path
            .strip_prefix(project_path)
            .unwrap_or(file_path)
            .display()
            .to_string();

        for (line_num, line) in &prod_lines {
            // Look for `null` as a word (not inside identifiers like "nullable")
            if contains_null_literal(line) {
                // Allow Java interop annotations
                if line.contains("@Nullable")
                    || line.contains("@javax")
                    || line.contains("@java")
                    || line.contains("JNI")
                {
                    continue;
                }
                violations.push(CbPatternViolation {
                    pattern_id: "CB-801".to_string(),
                    file: rel.clone(),
                    line: *line_num,
                    description: "Null literal — use Option[T] instead".to_string(),
                    severity: Severity::Warning,
                });
            }
        }
    }

    violations
}

/// Check if line contains `null` as a standalone keyword.
fn contains_null_literal(line: &str) -> bool {
    let bytes = line.as_bytes();
    let null_bytes = b"null";
    let len = bytes.len();
    if len < 4 {
        return false;
    }
    for i in 0..=len - 4 {
        if &bytes[i..i + 4] == null_bytes {
            let before_ok = i == 0 || !bytes[i - 1].is_ascii_alphanumeric() && bytes[i - 1] != b'_';
            let after_ok =
                i + 4 >= len || !bytes[i + 4].is_ascii_alphanumeric() && bytes[i + 4] != b'_';
            if before_ok && after_ok {
                return true;
            }
        }
    }
    false
}

// =============================================================================
// CB-802: Unrestricted Wildcard Import
// =============================================================================

pub fn detect_cb802_wildcard_import(project_path: &Path) -> Vec<CbPatternViolation> {
    let files = walkdir_scala_files(project_path);
    let mut violations = Vec::new();

    for file_path in &files {
        if is_scala_test_file(file_path) {
            continue;
        }
        let content = match fs::read_to_string(file_path) {
            Ok(c) => c,
            Err(_) => continue,
        };
        let prod_lines = compute_scala_production_lines(&content);
        let rel = file_path
            .strip_prefix(project_path)
            .unwrap_or(file_path)
            .display()
            .to_string();

        for (line_num, line) in &prod_lines {
            if !line.starts_with("import ") {
                continue;
            }
            // Scala 2: import pkg._ | Scala 3: import pkg.*
            if line.ends_with("._") || line.ends_with(".*") {
                // Allow standard library wildcards
                if is_allowed_wildcard_import(line) {
                    continue;
                }
                violations.push(CbPatternViolation {
                    pattern_id: "CB-802".to_string(),
                    file: rel.clone(),
                    line: *line_num,
                    description: "Wildcard import — import specific members".to_string(),
                    severity: Severity::Info,
                });
            }
        }
    }

    violations
}

/// Wildcard imports from standard library are generally acceptable.
fn is_allowed_wildcard_import(line: &str) -> bool {
    let allowed = [
        "scala.collection.",
        "scala.concurrent.",
        "scala.util.",
        "java.lang.",
        "java.util.",
        "scala.Predef.",
    ];
    allowed.iter().any(|prefix| line.contains(prefix))
}

// =============================================================================
// CB-803: Return Statement
// =============================================================================

pub fn detect_cb803_return_statement(project_path: &Path) -> Vec<CbPatternViolation> {
    let files = walkdir_scala_files(project_path);
    let mut violations = Vec::new();

    for file_path in &files {
        if is_scala_test_file(file_path) {
            continue;
        }
        let content = match fs::read_to_string(file_path) {
            Ok(c) => c,
            Err(_) => continue,
        };
        let prod_lines = compute_scala_production_lines(&content);
        let rel = file_path
            .strip_prefix(project_path)
            .unwrap_or(file_path)
            .display()
            .to_string();

        for (line_num, line) in &prod_lines {
            if contains_return_keyword(line) {
                violations.push(CbPatternViolation {
                    pattern_id: "CB-803".to_string(),
                    file: rel.clone(),
                    line: *line_num,
                    description: "Explicit `return` — anti-idiomatic in Scala".to_string(),
                    severity: Severity::Info,
                });
            }
        }
    }

    violations
}

/// Check for `return` as a standalone keyword, not inside string literals or comments.
fn contains_return_keyword(line: &str) -> bool {
    let bytes = line.as_bytes();
    let ret = b"return";
    let len = bytes.len();
    if len < 6 {
        return false;
    }
    for i in 0..=len - 6 {
        if &bytes[i..i + 6] == ret {
            let before_ok = i == 0 || !bytes[i - 1].is_ascii_alphanumeric() && bytes[i - 1] != b'_';
            let after_ok =
                i + 6 >= len || !bytes[i + 6].is_ascii_alphanumeric() && bytes[i + 6] != b'_';
            if before_ok && after_ok {
                // Skip if inside a string literal (simple heuristic)
                let before_text = &line[..i];
                let double_quotes = before_text.chars().filter(|&c| c == '"').count();
                if double_quotes % 2 == 0 {
                    return true;
                }
            }
        }
    }
    false
}

// =============================================================================
// CB-804: var Declaration
// =============================================================================

pub fn detect_cb804_var_declaration(project_path: &Path) -> Vec<CbPatternViolation> {
    let files = walkdir_scala_files(project_path);
    let mut violations = Vec::new();

    for file_path in &files {
        if is_scala_test_file(file_path) {
            continue;
        }
        let content = match fs::read_to_string(file_path) {
            Ok(c) => c,
            Err(_) => continue,
        };
        let prod_lines = compute_scala_production_lines(&content);
        let rel = file_path
            .strip_prefix(project_path)
            .unwrap_or(file_path)
            .display()
            .to_string();

        for (line_num, line) in &prod_lines {
            if contains_var_keyword(line) {
                violations.push(CbPatternViolation {
                    pattern_id: "CB-804".to_string(),
                    file: rel.clone(),
                    line: *line_num,
                    description: "`var` declaration — prefer `val` for immutability".to_string(),
                    severity: Severity::Warning,
                });
            }
        }
    }

    violations
}

/// Check for `var` as a standalone keyword at declaration position.
fn contains_var_keyword(line: &str) -> bool {
    // Match lines starting with var or containing var after whitespace/keywords
    let trimmed = line.trim();
    if trimmed.starts_with("var ") {
        return true;
    }
    // Also catch: `private var`, `protected var`, `override var`, etc.
    let modifiers = [
        "private var ",
        "protected var ",
        "override var ",
        "lazy var ",
    ];
    modifiers.iter().any(|m| trimmed.contains(m))
}

// =============================================================================
// CB-805: Blocking in Future
// =============================================================================

/// Blocking calls that should not appear inside Future blocks.
const BLOCKING_CALLS: &[&str] = &[
    "Thread.sleep",
    "Await.result",
    "Await.ready",
    ".wait()",
    "synchronized",
];

pub fn detect_cb805_blocking_in_future(project_path: &Path) -> Vec<CbPatternViolation> {
    let files = walkdir_scala_files(project_path);
    let mut violations = Vec::new();

    for file_path in &files {
        if is_scala_test_file(file_path) {
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

        detect_blocking_in_future_content(&content, &rel, &mut violations);
    }

    violations
}

fn detect_blocking_in_future_content(
    content: &str,
    rel: &str,
    violations: &mut Vec<CbPatternViolation>,
) {
    let mut in_future_block = false;
    let mut brace_depth: i32 = 0;
    let mut future_start_depth: i32 = 0;

    for (i, line) in content.lines().enumerate() {
        let trimmed = line.trim();

        // Skip comments
        if trimmed.starts_with("//") || trimmed.starts_with("/*") || trimmed.starts_with("*") {
            continue;
        }

        // Detect Future block start
        if (trimmed.contains("Future {") || trimmed.contains("Future.apply {")) && !in_future_block
        {
            in_future_block = true;
            future_start_depth = brace_depth;
        }

        // Track brace depth
        for c in trimmed.chars() {
            match c {
                '{' => brace_depth += 1,
                '}' => {
                    brace_depth -= 1;
                    if in_future_block && brace_depth <= future_start_depth {
                        in_future_block = false;
                    }
                }
                _ => {}
            }
        }

        // Check for blocking calls inside Future
        if in_future_block {
            for blocking in BLOCKING_CALLS {
                if trimmed.contains(blocking) {
                    violations.push(CbPatternViolation {
                        pattern_id: "CB-805".to_string(),
                        file: rel.to_string(),
                        line: i + 1,
                        description: format!(
                            "Blocking call `{}` inside Future — use non-blocking alternative",
                            blocking
                        ),
                        severity: Severity::Warning,
                    });
                }
            }
        }
    }
}

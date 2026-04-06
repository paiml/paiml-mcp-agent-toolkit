#![cfg_attr(coverage_nightly, coverage(off))]
//! CB-500 Series: Type safety and value correctness checks.
//!
//! - CB-501: Unwrap Density
//! - CB-502: Expect Quality
//! - CB-506: String Byte Indexing
//! - CB-508: Lossy Numeric Casts
//! - CB-515: Catch-All Match Default
//! - CB-516: Hardcoded Magic Numbers

use super::utilities::{is_cast_allowed, DOT_EXPECT_QUOTE, DOT_UNWRAP};
use crate::cli::handlers::comply_cb_detect::types::*;
use std::fs;
use std::path::Path;

/// CB-501: Unwrap Density - too many .unwrap() per file in production code
pub fn detect_cb501_unwrap_density(project_path: &Path) -> Vec<CbPatternViolation> {
    debug_assert!(
        project_path.exists(),
        "project_path must exist: {}",
        project_path.display()
    );
    let src_dir = project_path.join("src");
    let entries = match walkdir_rs_files(&src_dir) {
        Ok(e) => e,
        Err(_) => return Vec::new(),
    };

    let mut violations = Vec::new();

    for entry in &entries {
        if is_test_file(entry) {
            continue;
        }
        let content = match fs::read_to_string(entry) {
            Ok(c) => c,
            Err(_) => continue,
        };
        let lines: Vec<&str> = content.lines().collect();
        let test_lines = compute_test_code_lines(&lines);

        let count = lines
            .iter()
            .enumerate()
            .filter(|(i, _)| !test_lines.contains(i))
            .filter(|(_, line)| line.contains(DOT_UNWRAP))
            .count();

        let file = entry
            .strip_prefix(project_path)
            .unwrap_or(entry)
            .display()
            .to_string();

        if count > 10 {
            violations.push(CbPatternViolation {
                pattern_id: "CB-501".to_string(),
                file,
                line: 0,
                description: format!("{count} unwrap() calls in production code (threshold: 10)"),
                severity: Severity::Error,
            });
        } else if count > 5 {
            violations.push(CbPatternViolation {
                pattern_id: "CB-501".to_string(),
                file,
                line: 0,
                description: format!("{count} unwrap() calls in production code (threshold: 5)"),
                severity: Severity::Warning,
            });
        }
    }

    violations
}

/// CB-502: Expect Quality - lazy or empty .expect() messages
pub fn detect_cb502_expect_quality(project_path: &Path) -> Vec<CbPatternViolation> {
    debug_assert!(
        project_path.exists(),
        "project_path must exist: {}",
        project_path.display()
    );
    let src_dir = project_path.join("src");
    let entries = match walkdir_rs_files(&src_dir) {
        Ok(e) => e,
        Err(_) => return Vec::new(),
    };

    let lazy_messages = [
        "\"\")",
        "\"failed\")",
        "\"error\")",
        "\"unexpected\")",
        "\"should not happen\")",
        "\"todo\")",
        "\"bug\")",
        "\"impossible\")",
    ];

    let mut violations = Vec::new();

    for entry in &entries {
        if is_test_file(entry) {
            continue;
        }
        let content = match fs::read_to_string(entry) {
            Ok(c) => c,
            Err(_) => continue,
        };
        let lines: Vec<&str> = content.lines().collect();
        let test_lines = compute_test_code_lines(&lines);
        let file = entry
            .strip_prefix(project_path)
            .unwrap_or(entry)
            .display()
            .to_string();

        for (i, line) in lines.iter().enumerate() {
            if test_lines.contains(&i) {
                continue;
            }
            if !line.contains(DOT_EXPECT_QUOTE) {
                continue;
            }
            for lazy in &lazy_messages {
                if line.contains(&format!(
                    "{DOT_EXPECT_QUOTE}{}",
                    lazy.get(1..).unwrap_or_default()
                )) {
                    violations.push(CbPatternViolation {
                        pattern_id: "CB-502".to_string(),
                        file: file.clone(),
                        line: i + 1,
                        description: format!("Lazy .expect() message: {lazy}"),
                        severity: Severity::Warning,
                    });
                    break;
                }
            }
        }
    }

    violations
}

/// CB-506: String Byte Indexing - &str[n..m] can panic on non-ASCII
pub fn detect_cb506_string_byte_indexing(project_path: &Path) -> Vec<CbPatternViolation> {
    debug_assert!(
        project_path.exists(),
        "project_path must exist: {}",
        project_path.display()
    );
    let src_dir = project_path.join("src");
    let entries = match walkdir_rs_files(&src_dir) {
        Ok(e) => e,
        Err(_) => return Vec::new(),
    };

    let mut violations = Vec::new();
    // Match patterns like &foo[n..m], &s[..n], &name[1..] on string-like variables
    let index_pattern = regex::Regex::new(r"&\w+\[\d*\.\.\d*\]").expect("valid regex");

    for entry in &entries {
        if is_test_file(entry) {
            continue;
        }
        let content = match fs::read_to_string(entry) {
            Ok(c) => c,
            Err(_) => continue,
        };
        let lines: Vec<&str> = content.lines().collect();
        let test_lines = compute_test_code_lines(&lines);
        let file = entry
            .strip_prefix(project_path)
            .unwrap_or(entry)
            .display()
            .to_string();

        for (i, line) in lines.iter().enumerate() {
            if test_lines.contains(&i) {
                continue;
            }
            if index_pattern.is_match(line) && !line.trim().starts_with("//") {
                violations.push(CbPatternViolation {
                    pattern_id: "CB-506".to_string(),
                    file: file.clone(),
                    line: i + 1,
                    description: "String byte indexing (&str[n..m]) can panic on non-ASCII input"
                        .to_string(),
                    severity: Severity::Warning,
                });
            }
        }
    }

    violations
}

/// CB-508: Lossy Numeric Casts - `as u8`, `as i32`, etc. without bounds checking
pub fn detect_cb508_lossy_numeric_casts(project_path: &Path) -> Vec<CbPatternViolation> {
    debug_assert!(
        project_path.exists(),
        "project_path must exist: {}",
        project_path.display()
    );
    let src_dir = project_path.join("src");
    let entries = match walkdir_rs_files(&src_dir) {
        Ok(e) => e,
        Err(_) => return Vec::new(),
    };

    let cast_patterns = [
        " as u8", " as u16", " as u32", " as i8", " as i16", " as i32", " as f32",
    ];
    let mut violations = Vec::new();

    for entry in &entries {
        if is_test_file(entry) {
            continue;
        }
        let content = match fs::read_to_string(entry) {
            Ok(c) => c,
            Err(_) => continue,
        };
        let lines: Vec<&str> = content.lines().collect();
        let test_lines = compute_test_code_lines(&lines);

        let count = lines
            .iter()
            .enumerate()
            .filter(|(i, _)| !test_lines.contains(i))
            .filter(|(i, line)| {
                let trimmed = line.trim();
                !trimmed.starts_with("//")
                    && cast_patterns.iter().any(|p| trimmed.contains(p))
                    && !is_cast_allowed(&lines, *i)
            })
            .count();

        if count > 10 {
            let file = entry
                .strip_prefix(project_path)
                .unwrap_or(entry)
                .display()
                .to_string();
            violations.push(CbPatternViolation {
                pattern_id: "CB-508".to_string(),
                file,
                line: 0,
                description: format!("{count} lossy numeric casts without bounds checking"),
                severity: Severity::Warning,
            });
        }
    }

    violations
}

/// CB-515: Catch-All Match Default - `_ =>` returning concrete values instead of errors
pub fn detect_cb515_catch_all_match_default(project_path: &Path) -> Vec<CbPatternViolation> {
    debug_assert!(
        project_path.exists(),
        "project_path must exist: {}",
        project_path.display()
    );
    let src_dir = project_path.join("src");
    let entries = match walkdir_rs_files(&src_dir) {
        Ok(e) => e,
        Err(_) => return Vec::new(),
    };

    let mut violations = Vec::new();

    // These are safe catch-all patterns that indicate proper error handling
    let safe_patterns = [
        "Err(",
        "None",
        "unreachable!",
        "panic!",
        "return Err",
        "return None",
        "bail!",
        "anyhow!",
        "todo!",
        "unimplemented!",
        "Default::default()",
        "default()",
    ];

    for entry in &entries {
        if is_test_file(entry) {
            continue;
        }
        let content = match fs::read_to_string(entry) {
            Ok(c) => c,
            Err(_) => continue,
        };
        let lines: Vec<&str> = content.lines().collect();
        let test_lines = compute_test_code_lines(&lines);
        let file = entry
            .strip_prefix(project_path)
            .unwrap_or(entry)
            .display()
            .to_string();

        for (i, line) in lines.iter().enumerate() {
            if test_lines.contains(&i) {
                continue;
            }
            let trimmed = line.trim();
            if trimmed.starts_with("//") || trimmed.starts_with("*") {
                continue;
            }

            // Match `_ =>` pattern
            if !trimmed.starts_with("_ =>") && !trimmed.starts_with("_=>") {
                continue;
            }

            // Extract what comes after `_ =>`
            let after = trimmed
                .trim_start_matches("_ =>")
                .trim_start_matches("_=>")
                .trim();

            // Strip inline comments before pattern checks
            let after = if let Some(pos) = after.find("//") {
                after[..pos].trim()
            } else {
                after
            };

            // Skip empty arms (multi-line blocks)
            if after.is_empty() || after == "{" {
                continue;
            }

            // Skip safe patterns
            if safe_patterns.iter().any(|p| after.contains(p)) {
                continue;
            }

            // Skip if it's just a closing brace, comma, or empty block (unit return)
            if after == "}" || after == "}," || after == "," || after == "{}" || after == "{}," {
                continue;
            }

            violations.push(CbPatternViolation {
                pattern_id: "CB-515".to_string(),
                file: file.clone(),
                line: i + 1,
                description: format!(
                    "Catch-all match arm `_ =>` returns concrete value: {}",
                    if after.len() > 60 {
                        &after[..60]
                    } else {
                        after
                    }
                ),
                severity: Severity::Warning,
            });
        }
    }

    violations
}

/// CB-516: Hardcoded Magic Numbers - large numeric literals in configuration contexts
pub fn detect_cb516_hardcoded_magic_numbers(project_path: &Path) -> Vec<CbPatternViolation> {
    debug_assert!(
        project_path.exists(),
        "project_path must exist: {}",
        project_path.display()
    );
    let src_dir = project_path.join("src");
    let entries = match walkdir_rs_files(&src_dir) {
        Ok(e) => e,
        Err(_) => return Vec::new(),
    };

    let magic_number_re =
        regex::Regex::new(r"(?:Some\(|:\s*)(\d{3,}(?:\.\d+)?)\s*[,\)]").expect("valid regex");

    // Common non-magic constants to exclude
    let common_values: std::collections::HashSet<&str> = [
        "100", "128", "256", "512", "1024", "2048", "4096", "8192", "1000", "1024", "65535",
        "65536",
    ]
    .iter()
    .copied()
    .collect();

    let mut violations = Vec::new();

    for entry in &entries {
        if is_test_file(entry) {
            continue;
        }
        let content = match fs::read_to_string(entry) {
            Ok(c) => c,
            Err(_) => continue,
        };
        let lines: Vec<&str> = content.lines().collect();
        let test_lines = compute_test_code_lines(&lines);
        let file = entry
            .strip_prefix(project_path)
            .unwrap_or(entry)
            .display()
            .to_string();

        for (i, line) in lines.iter().enumerate() {
            if test_lines.contains(&i) {
                continue;
            }
            let trimmed = line.trim();
            if trimmed.starts_with("//") || trimmed.starts_with("*") {
                continue;
            }
            // Skip const/static declarations (intentional named constants)
            if trimmed.starts_with("const ") || trimmed.starts_with("static ") {
                continue;
            }

            for cap in magic_number_re.captures_iter(trimmed) {
                let num_str = cap.get(1).map(|m| m.as_str()).unwrap_or("");
                if common_values.contains(num_str) {
                    continue;
                }
                // Only flag numbers > 100 (to reduce noise)
                if let Ok(val) = num_str.parse::<f64>() {
                    if val > 100.0 {
                        violations.push(CbPatternViolation {
                            pattern_id: "CB-516".to_string(),
                            file: file.clone(),
                            line: i + 1,
                            description: format!(
                                "Hardcoded magic number {num_str} — consider a named constant"
                            ),
                            severity: Severity::Info,
                        });
                    }
                }
            }
        }
    }

    violations
}

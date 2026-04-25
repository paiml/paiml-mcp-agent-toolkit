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
#[provable_contracts_macros::contract("pmat-core.yaml", equation = "path_exists")]
pub fn detect_cb501_unwrap_density(project_path: &Path) -> Vec<CbPatternViolation> {
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
#[provable_contracts_macros::contract("pmat-core.yaml", equation = "path_exists")]
pub fn detect_cb502_expect_quality(project_path: &Path) -> Vec<CbPatternViolation> {
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
#[provable_contracts_macros::contract("pmat-core.yaml", equation = "path_exists")]
pub fn detect_cb506_string_byte_indexing(project_path: &Path) -> Vec<CbPatternViolation> {
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
#[provable_contracts_macros::contract("pmat-core.yaml", equation = "path_exists")]
pub fn detect_cb508_lossy_numeric_casts(project_path: &Path) -> Vec<CbPatternViolation> {
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
#[provable_contracts_macros::contract("pmat-core.yaml", equation = "path_exists")]
pub fn detect_cb515_catch_all_match_default(project_path: &Path) -> Vec<CbPatternViolation> {
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
                    crate::utils::string_truncate::truncate_at_char_boundary(after, 60)
                ),
                severity: Severity::Warning,
            });
        }
    }

    violations
}

/// CB-516: Hardcoded Magic Numbers - large numeric literals in configuration contexts
#[provable_contracts_macros::contract("pmat-core.yaml", equation = "path_exists")]
pub fn detect_cb516_hardcoded_magic_numbers(project_path: &Path) -> Vec<CbPatternViolation> {
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    fn write_rs(dir: &Path, name: &str, content: &str) {
        let src_dir = dir.join("src");
        fs::create_dir_all(&src_dir).unwrap();
        fs::write(src_dir.join(name), content).unwrap();
    }

    fn unwrap_calls(n: usize) -> String {
        let mut s = String::from("fn f() {\n");
        for i in 0..n {
            s.push_str(&format!("  let x{i} = opt.unwrap();\n"));
        }
        s.push_str("}\n");
        s
    }

    // ── CB-501 ──────────────────────────────────────────────────────────────

    #[test]
    fn test_cb501_no_src_dir_returns_empty() {
        let tmp = TempDir::new().unwrap();
        assert!(detect_cb501_unwrap_density(tmp.path()).is_empty());
    }

    #[test]
    fn test_cb501_zero_unwraps_clean() {
        let tmp = TempDir::new().unwrap();
        write_rs(tmp.path(), "lib.rs", "fn f() { let x = 1; }\n");
        assert!(detect_cb501_unwrap_density(tmp.path()).is_empty());
    }

    #[test]
    fn test_cb501_at_threshold_5_clean() {
        // Threshold is > 5; 5 unwraps still clean
        let tmp = TempDir::new().unwrap();
        write_rs(tmp.path(), "lib.rs", &unwrap_calls(5));
        assert!(detect_cb501_unwrap_density(tmp.path()).is_empty());
    }

    #[test]
    fn test_cb501_above_5_warning() {
        let tmp = TempDir::new().unwrap();
        write_rs(tmp.path(), "lib.rs", &unwrap_calls(7));
        let v = detect_cb501_unwrap_density(tmp.path());
        assert_eq!(v.len(), 1);
        assert_eq!(v[0].severity, Severity::Warning);
    }

    #[test]
    fn test_cb501_above_10_error() {
        let tmp = TempDir::new().unwrap();
        write_rs(tmp.path(), "lib.rs", &unwrap_calls(15));
        let v = detect_cb501_unwrap_density(tmp.path());
        assert_eq!(v.len(), 1);
        assert_eq!(v[0].severity, Severity::Error);
    }

    // ── CB-502 ──────────────────────────────────────────────────────────────

    #[test]
    fn test_cb502_no_src_dir_returns_empty() {
        let tmp = TempDir::new().unwrap();
        assert!(detect_cb502_expect_quality(tmp.path()).is_empty());
    }

    #[test]
    fn test_cb502_quality_message_clean() {
        let tmp = TempDir::new().unwrap();
        write_rs(
            tmp.path(),
            "lib.rs",
            "fn f() { opt.expect(\"config file must be readable\"); }\n",
        );
        assert!(detect_cb502_expect_quality(tmp.path()).is_empty());
    }

    #[test]
    fn test_cb502_lazy_failed_message_flagged() {
        let tmp = TempDir::new().unwrap();
        write_rs(tmp.path(), "lib.rs", "fn f() { opt.expect(\"failed\"); }\n");
        let v = detect_cb502_expect_quality(tmp.path());
        assert!(!v.is_empty());
        assert_eq!(v[0].pattern_id, "CB-502");
    }

    #[test]
    fn test_cb502_lazy_empty_message_flagged() {
        let tmp = TempDir::new().unwrap();
        write_rs(tmp.path(), "lib.rs", "fn f() { opt.expect(\"\"); }\n");
        let v = detect_cb502_expect_quality(tmp.path());
        assert!(!v.is_empty());
    }

    // ── CB-506 ──────────────────────────────────────────────────────────────

    #[test]
    fn test_cb506_no_src_dir_returns_empty() {
        let tmp = TempDir::new().unwrap();
        assert!(detect_cb506_string_byte_indexing(tmp.path()).is_empty());
    }

    #[test]
    fn test_cb506_no_indexing_clean() {
        let tmp = TempDir::new().unwrap();
        write_rs(tmp.path(), "lib.rs", "fn f() { let x = s.chars(); }\n");
        assert!(detect_cb506_string_byte_indexing(tmp.path()).is_empty());
    }

    #[test]
    fn test_cb506_byte_slice_flagged() {
        let tmp = TempDir::new().unwrap();
        write_rs(tmp.path(), "lib.rs", "fn f() { let p = &name[0..3]; }\n");
        let v = detect_cb506_string_byte_indexing(tmp.path());
        assert!(!v.is_empty());
        assert_eq!(v[0].pattern_id, "CB-506");
    }

    #[test]
    fn test_cb506_open_ended_slice_flagged() {
        let tmp = TempDir::new().unwrap();
        write_rs(tmp.path(), "lib.rs", "fn f() { let p = &foo[..5]; }\n");
        assert!(!detect_cb506_string_byte_indexing(tmp.path()).is_empty());
    }

    #[test]
    fn test_cb506_comment_indexing_skipped() {
        let tmp = TempDir::new().unwrap();
        write_rs(tmp.path(), "lib.rs", "// let p = &name[0..3];\n");
        assert!(detect_cb506_string_byte_indexing(tmp.path()).is_empty());
    }

    // ── CB-508 ──────────────────────────────────────────────────────────────

    #[test]
    fn test_cb508_no_src_dir_returns_empty() {
        let tmp = TempDir::new().unwrap();
        assert!(detect_cb508_lossy_numeric_casts(tmp.path()).is_empty());
    }

    #[test]
    fn test_cb508_below_threshold_clean() {
        let tmp = TempDir::new().unwrap();
        // 5 casts < threshold of 10
        let mut content = String::from("fn f() {\n");
        for i in 0..5 {
            content.push_str(&format!("  let x{i} = (y{i} as u8);\n"));
        }
        content.push_str("}\n");
        write_rs(tmp.path(), "lib.rs", &content);
        assert!(detect_cb508_lossy_numeric_casts(tmp.path()).is_empty());
    }

    #[test]
    fn test_cb508_above_threshold_flagged() {
        let tmp = TempDir::new().unwrap();
        let mut content = String::from("fn f() {\n");
        for i in 0..15 {
            content.push_str(&format!("  let x{i} = (y{i} as u8);\n"));
        }
        content.push_str("}\n");
        write_rs(tmp.path(), "lib.rs", &content);
        let v = detect_cb508_lossy_numeric_casts(tmp.path());
        assert!(!v.is_empty());
        assert_eq!(v[0].pattern_id, "CB-508");
    }

    // ── CB-515 ──────────────────────────────────────────────────────────────

    #[test]
    fn test_cb515_no_src_dir_returns_empty() {
        let tmp = TempDir::new().unwrap();
        assert!(detect_cb515_catch_all_match_default(tmp.path()).is_empty());
    }

    #[test]
    fn test_cb515_no_match_clean() {
        let tmp = TempDir::new().unwrap();
        write_rs(tmp.path(), "lib.rs", "fn f() { let x = 1; }\n");
        assert!(detect_cb515_catch_all_match_default(tmp.path()).is_empty());
    }

    #[test]
    fn test_cb515_concrete_value_arm_flagged() {
        // The parser checks `trimmed.starts_with("_ =>")` — `_ =>` must be the
        // line's leading non-whitespace token (multi-line match formatting).
        let tmp = TempDir::new().unwrap();
        write_rs(
            tmp.path(),
            "lib.rs",
            "fn f() {\n  match x {\n    1 => \"one\",\n    _ => \"unknown\",\n  };\n}\n",
        );
        let v = detect_cb515_catch_all_match_default(tmp.path());
        assert!(!v.is_empty());
        assert_eq!(v[0].pattern_id, "CB-515");
    }

    #[test]
    fn test_cb515_safe_pattern_returning_err_skipped() {
        let tmp = TempDir::new().unwrap();
        write_rs(
            tmp.path(),
            "lib.rs",
            "fn f() {\n  match x {\n    1 => Ok(()),\n    _ => Err(\"x\"),\n  };\n}\n",
        );
        // `_ => Err(...)` — safe pattern → no violation
        assert!(detect_cb515_catch_all_match_default(tmp.path()).is_empty());
    }

    #[test]
    fn test_cb515_safe_panic_skipped() {
        let tmp = TempDir::new().unwrap();
        write_rs(
            tmp.path(),
            "lib.rs",
            "fn f() {\n  match x {\n    1 => 1,\n    _ => panic!(\"bad\"),\n  };\n}\n",
        );
        assert!(detect_cb515_catch_all_match_default(tmp.path()).is_empty());
    }

    #[test]
    fn test_cb515_empty_block_arm_skipped() {
        let tmp = TempDir::new().unwrap();
        write_rs(
            tmp.path(),
            "lib.rs",
            "fn f() {\n  match x {\n    1 => 1,\n    _ => {},\n  };\n}\n",
        );
        assert!(detect_cb515_catch_all_match_default(tmp.path()).is_empty());
    }

    // ── CB-516 ──────────────────────────────────────────────────────────────

    #[test]
    fn test_cb516_no_src_dir_returns_empty() {
        let tmp = TempDir::new().unwrap();
        assert!(detect_cb516_hardcoded_magic_numbers(tmp.path()).is_empty());
    }

    #[test]
    fn test_cb516_small_number_clean() {
        let tmp = TempDir::new().unwrap();
        write_rs(
            tmp.path(),
            "lib.rs",
            "fn f(timeout: u32) { x(timeout: 50); }\n",
        );
        // 50 doesn't match the regex (\d{3,}) — needs ≥3 digits
        assert!(detect_cb516_hardcoded_magic_numbers(tmp.path()).is_empty());
    }

    #[test]
    fn test_cb516_common_value_skipped() {
        let tmp = TempDir::new().unwrap();
        write_rs(
            tmp.path(),
            "lib.rs",
            "fn f() { Config { capacity: 1024, threshold: 256 }; }\n",
        );
        // 1024 and 256 are in the common_values set → skipped
        assert!(detect_cb516_hardcoded_magic_numbers(tmp.path()).is_empty());
    }

    #[test]
    fn test_cb516_unusual_large_number_flagged() {
        let tmp = TempDir::new().unwrap();
        write_rs(
            tmp.path(),
            "lib.rs",
            "fn f() { Config { timeout: 7777, retry: 3333 }; }\n",
        );
        let v = detect_cb516_hardcoded_magic_numbers(tmp.path());
        assert!(!v.is_empty());
        assert_eq!(v[0].pattern_id, "CB-516");
    }

    #[test]
    fn test_cb516_const_declaration_skipped() {
        let tmp = TempDir::new().unwrap();
        write_rs(tmp.path(), "lib.rs", "const FOO: u32 = 7777;\n");
        // const-prefixed lines are intentionally named constants
        assert!(detect_cb516_hardcoded_magic_numbers(tmp.path()).is_empty());
    }

    #[test]
    fn test_cb516_static_declaration_skipped() {
        let tmp = TempDir::new().unwrap();
        write_rs(tmp.path(), "lib.rs", "static FOO: u32 = 7777;\n");
        assert!(detect_cb516_hardcoded_magic_numbers(tmp.path()).is_empty());
    }
}

#![cfg_attr(coverage_nightly, coverage(off))]
//! CB-500 Series: Rust Best Practices Detection
//!
//! Generic Rust defect detection for `pmat comply check`.
//! These checks apply to ANY Rust project, not just PAIML-specific patterns.

use super::types::*;
use std::fs;
use std::path::Path;

// Use concat! to avoid self-detection by CB-501/CB-502 scanners
const DOT_UNWRAP: &str = concat!(".unwr", "ap()");
const DOT_EXPECT_QUOTE: &str = concat!(".expe", "ct(\"");

/// CB-500: Publish Hygiene - missing `exclude` in Cargo.toml
pub fn detect_cb500_publish_hygiene(project_path: &Path) -> Vec<CbPatternViolation> {
    let cargo_toml = project_path.join("Cargo.toml");
    let content = match fs::read_to_string(&cargo_toml) {
        Ok(c) => c,
        Err(_) => return Vec::new(),
    };

    let mut violations = Vec::new();

    let has_exclude = content.contains("exclude = [") || content.contains("exclude=[");
    let has_include = content.contains("include = [") || content.contains("include=[");

    if !has_exclude && !has_include {
        violations.push(CbPatternViolation {
            pattern_id: "CB-500".to_string(),
            file: "Cargo.toml".to_string(),
            line: 1,
            description: "Missing `exclude` field - published crate may include unnecessary files"
                .to_string(),
            severity: Severity::Warning,
        });
    }

    if has_include && has_exclude {
        violations.push(CbPatternViolation {
            pattern_id: "CB-500".to_string(),
            file: "Cargo.toml".to_string(),
            line: 1,
            description: "Both `include` and `exclude` present - Cargo ignores `exclude` when `include` is set"
                .to_string(),
            severity: Severity::Warning,
        });
    }

    if has_exclude {
        let critical_patterns = [
            "target/", ".profraw", ".profdata", ".vscode/", ".idea/", ".pmat",
            "proptest-regressions",
        ];
        let matched = critical_patterns
            .iter()
            .filter(|p| content.contains(*p))
            .count();
        if matched < 3 {
            violations.push(CbPatternViolation {
                pattern_id: "CB-500".to_string(),
                file: "Cargo.toml".to_string(),
                line: 1,
                description: format!(
                    "Only {matched}/7 critical patterns in exclude (target/, .profraw, .profdata, .vscode/, .idea/, .pmat, proptest-regressions)"
                ),
                severity: Severity::Info,
            });
        }
    }

    violations
}

/// CB-501: Unwrap Density - too many .unwrap() per file in production code
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
pub fn detect_cb502_expect_quality(project_path: &Path) -> Vec<CbPatternViolation> {
    let src_dir = project_path.join("src");
    let entries = match walkdir_rs_files(&src_dir) {
        Ok(e) => e,
        Err(_) => return Vec::new(),
    };

    let lazy_messages = [
        "\"\")", "\"failed\")", "\"error\")", "\"unexpected\")",
        "\"should not happen\")", "\"todo\")", "\"bug\")", "\"impossible\")",
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
                if line.contains(&format!("{DOT_EXPECT_QUOTE}{}", lazy.get(1..).unwrap_or_default())) {
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

/// CB-503: Clippy Configuration - missing .clippy.toml
pub fn detect_cb503_clippy_config(project_path: &Path) -> Vec<CbPatternViolation> {
    let clippy_toml = project_path.join(".clippy.toml");
    let clippy_toml_alt = project_path.join("clippy.toml");
    let mut violations = Vec::new();

    if !clippy_toml.exists() && !clippy_toml_alt.exists() {
        violations.push(CbPatternViolation {
            pattern_id: "CB-503".to_string(),
            file: ".clippy.toml".to_string(),
            line: 0,
            description: "No clippy configuration file found".to_string(),
            severity: Severity::Info,
        });
    } else {
        let path = if clippy_toml.exists() {
            &clippy_toml
        } else {
            &clippy_toml_alt
        };
        if let Ok(content) = fs::read_to_string(path) {
            if !content.contains("disallowed-methods") {
                violations.push(CbPatternViolation {
                    pattern_id: "CB-503".to_string(),
                    file: path.file_name().unwrap_or_default().to_string_lossy().to_string(),
                    line: 0,
                    description: "Clippy config missing `disallowed-methods` section".to_string(),
                    severity: Severity::Info,
                });
            }
        }
    }

    violations
}

/// CB-504: Deny Configuration - missing deny.toml for supply chain security
pub fn detect_cb504_deny_config(project_path: &Path) -> Vec<CbPatternViolation> {
    let deny_toml = project_path.join("deny.toml");
    if deny_toml.exists() {
        return Vec::new();
    }
    vec![CbPatternViolation {
        pattern_id: "CB-504".to_string(),
        file: "deny.toml".to_string(),
        line: 0,
        description: "No cargo-deny configuration for supply chain security".to_string(),
        severity: Severity::Info,
    }]
}

/// CB-505: Workspace Lint Hygiene - missing [lints] or [workspace.lints]
pub fn detect_cb505_workspace_lint_hygiene(project_path: &Path) -> Vec<CbPatternViolation> {
    let cargo_toml = project_path.join("Cargo.toml");
    let content = match fs::read_to_string(&cargo_toml) {
        Ok(c) => c,
        Err(_) => return Vec::new(),
    };

    let has_workspace_lints =
        content.contains("[workspace.lints]") || content.contains("[workspace.lints.");
    let has_lints = content.contains("[lints]") || content.contains("[lints.");

    if has_workspace_lints || has_lints {
        return Vec::new();
    }

    vec![CbPatternViolation {
        pattern_id: "CB-505".to_string(),
        file: "Cargo.toml".to_string(),
        line: 1,
        description: "Missing [lints] section - no project-wide lint configuration".to_string(),
        severity: Severity::Warning,
    }]
}

/// CB-506: String Byte Indexing - &str[n..m] can panic on non-ASCII
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

/// CB-507: Panic Macros - todo!(), unimplemented!() in production code
pub fn detect_cb507_panic_macros(project_path: &Path) -> Vec<CbPatternViolation> {
    let src_dir = project_path.join("src");
    let entries = match walkdir_rs_files(&src_dir) {
        Ok(e) => e,
        Err(_) => return Vec::new(),
    };

    let panic_macros = ["todo!(", "unimplemented!("];
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
            if is_macro_in_string_literal(trimmed, &panic_macros) {
                continue;
            }
            for mac in &panic_macros {
                if trimmed.contains(mac) {
                    violations.push(CbPatternViolation {
                        pattern_id: "CB-507".to_string(),
                        file: file.clone(),
                        line: i + 1,
                        description: format!(
                            "Panic macro `{}` in production code",
                            mac.trim_end_matches('(')
                        ),
                        severity: Severity::Warning,
                    });
                    break;
                }
            }
        }
    }

    violations
}

/// CB-508: Lossy Numeric Casts - `as u8`, `as i32`, etc. without bounds checking
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
            .filter(|(_, line)| {
                let trimmed = line.trim();
                !trimmed.starts_with("//")
                    && !trimmed.contains("allow(clippy::cast")
                    && cast_patterns.iter().any(|p| trimmed.contains(p))
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

/// CB-509: Feature Gate Coverage - features defined but never tested
pub fn detect_cb509_feature_gate_coverage(project_path: &Path) -> Vec<CbPatternViolation> {
    let cargo_toml = project_path.join("Cargo.toml");
    let content = match fs::read_to_string(&cargo_toml) {
        Ok(c) => c,
        Err(_) => return Vec::new(),
    };

    // Count features defined
    let in_features = content
        .lines()
        .skip_while(|l| !l.starts_with("[features]"))
        .skip(1)
        .take_while(|l| !l.starts_with('['))
        .filter(|l| l.contains('='))
        .count();

    if in_features == 0 {
        return Vec::new();
    }

    // Check for CI matrix testing features
    let ci_dir = project_path.join(".github/workflows");
    let has_feature_matrix = if ci_dir.exists() {
        walkdir_files_with_ext(&ci_dir, "yml")
            .unwrap_or_default()
            .iter()
            .chain(
                walkdir_files_with_ext(&ci_dir, "yaml")
                    .unwrap_or_default()
                    .iter(),
            )
            .any(|f| {
                fs::read_to_string(f)
                    .map(|c| c.contains("features") || c.contains("--features"))
                    .unwrap_or(false)
            })
    } else {
        false
    };

    if !has_feature_matrix && in_features > 3 {
        vec![CbPatternViolation {
            pattern_id: "CB-509".to_string(),
            file: "Cargo.toml".to_string(),
            line: 0,
            description: format!(
                "{in_features} features defined but no CI feature matrix testing detected"
            ),
            severity: Severity::Info,
        }]
    } else {
        Vec::new()
    }
}

/// CB-510: include!() Macro Hygiene - non-standalone files included via include!()
pub fn detect_cb510_include_macro_hygiene(project_path: &Path) -> Vec<CbPatternViolation> {
    let src_dir = project_path.join("src");
    let entries = match walkdir_rs_files(&src_dir) {
        Ok(e) => e,
        Err(_) => return Vec::new(),
    };

    let mut violations = Vec::new();

    for entry in &entries {
        let content = match fs::read_to_string(entry) {
            Ok(c) => c,
            Err(_) => continue,
        };
        let file = entry
            .strip_prefix(project_path)
            .unwrap_or(entry)
            .display()
            .to_string();

        for (i, line) in content.lines().enumerate() {
            let trimmed = line.trim();
            if trimmed.starts_with("//") {
                continue;
            }
            if trimmed.contains("include!(") && !trimmed.contains("include_str!") {
                violations.push(CbPatternViolation {
                    pattern_id: "CB-510".to_string(),
                    file: file.clone(),
                    line: i + 1,
                    description: "include!() macro - included files are not standalone compilable"
                        .to_string(),
                    severity: Severity::Info,
                });
            }
        }
    }

    violations
}

/// CB-511: Flaky Timing Tests - tests with Instant::now() and tight duration assertions
pub fn detect_cb511_flaky_timing_tests(project_path: &Path) -> Vec<CbPatternViolation> {
    let src_dir = project_path.join("src");
    let test_dir = project_path.join("tests");

    let mut all_files = walkdir_rs_files(&src_dir).unwrap_or_default();
    all_files.extend(walkdir_rs_files(&test_dir).unwrap_or_default());

    let mut violations = Vec::new();

    for entry in &all_files {
        let content = match fs::read_to_string(entry) {
            Ok(c) => c,
            Err(_) => continue,
        };

        // Only check files with test attributes
        if !content.contains("#[test]") && !content.contains("#[tokio::test]") {
            continue;
        }

        let file = entry
            .strip_prefix(project_path)
            .unwrap_or(entry)
            .display()
            .to_string();

        // Per-test-function analysis: only flag when Instant::now(), .elapsed(),
        // and a duration assertion all appear in the SAME test function
        let lines: Vec<&str> = content.lines().collect();
        let mut in_test_fn = false;
        let mut test_fn_start: usize = 0;
        let mut brace_depth: u32 = 0;
        let mut fn_has_instant = false;
        let mut fn_has_elapsed = false;
        let mut fn_has_duration_assert = false;

        for (i, line) in lines.iter().enumerate() {
            let trimmed = line.trim();

            // Detect test function start (look for #[test] or #[tokio::test] preceding fn)
            if !in_test_fn
                && (trimmed.starts_with("fn ") || trimmed.starts_with("async fn "))
                && i > 0
            {
                // Look back for #[test] or #[tokio::test] attribute
                let has_test_attr = (1..=3).any(|back| {
                    i >= back && {
                        let prev = lines[i - back].trim();
                        prev == "#[test]" || prev == "#[tokio::test]"
                    }
                });
                if has_test_attr {
                    in_test_fn = true;
                    test_fn_start = i + 1;
                    brace_depth = 0;
                    fn_has_instant = false;
                    fn_has_elapsed = false;
                    fn_has_duration_assert = false;
                }
            }

            if in_test_fn {
                brace_depth += trimmed.matches('{').count() as u32;
                brace_depth = brace_depth.saturating_sub(trimmed.matches('}').count() as u32);

                if trimmed.contains("Instant::now()") {
                    fn_has_instant = true;
                }
                if trimmed.contains(".elapsed()") {
                    fn_has_elapsed = true;
                }
                if trimmed.contains("assert!") && trimmed.contains("elapsed") {
                    fn_has_duration_assert = true;
                }

                // End of test function
                if brace_depth == 0 && i > test_fn_start {
                    if fn_has_instant && fn_has_elapsed && fn_has_duration_assert {
                        violations.push(CbPatternViolation {
                            pattern_id: "CB-511".to_string(),
                            file: file.clone(),
                            line: test_fn_start,
                            description: "Test uses Instant::now() with duration assertions — may be flaky under load".to_string(),
                            severity: Severity::Warning,
                        });
                    }
                    in_test_fn = false;
                }
            }
        }
    }

    violations
}

/// CB-512: Error Propagation Gap - functions returning Result but using unwrap() internally
pub fn detect_cb512_error_propagation_gap(project_path: &Path) -> Vec<CbPatternViolation> {
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
        let file = entry
            .strip_prefix(project_path)
            .unwrap_or(entry)
            .display()
            .to_string();

        let mut in_result_fn = false;
        let mut fn_line = 0;
        let mut fn_depth = 0u32;
        let mut unwrap_count = 0u32;

        for (i, line) in lines.iter().enumerate() {
            if test_lines.contains(&i) {
                continue;
            }

            let trimmed = line.trim();

            // Detect function returning Result
            if (trimmed.starts_with("pub fn ")
                || trimmed.starts_with("fn ")
                || trimmed.starts_with("pub async fn ")
                || trimmed.starts_with("async fn "))
                && trimmed.contains("Result<")
            {
                in_result_fn = true;
                fn_line = i;
                fn_depth = 0;
                unwrap_count = 0;
            }

            if in_result_fn {
                fn_depth += trimmed.matches('{').count() as u32;
                fn_depth = fn_depth.saturating_sub(trimmed.matches('}').count() as u32);

                if trimmed.contains(DOT_UNWRAP) {
                    unwrap_count += 1;
                }

                // End of function
                if fn_depth == 0 && i > fn_line {
                    if unwrap_count >= 3 {
                        violations.push(CbPatternViolation {
                            pattern_id: "CB-512".to_string(),
                            file: file.clone(),
                            line: fn_line + 1,
                            description: format!(
                                "Function returns Result but has {unwrap_count} unwrap() calls - consider using ? operator"
                            ),
                            severity: Severity::Warning,
                        });
                    }
                    in_result_fn = false;
                }
            }
        }
    }

    violations
}

// Use concat! to avoid self-detection by CB-513 scanner
const UNWRAP_OR_ELSE_DISCARD: &str = concat!(".unwrap_or_el", "se(|_|");
const MAP_ERR_DISCARD: &str = concat!(".map_er", "r(|_|");
/// CB-513: Silent Error Swallowing - discarding error context with |_| closures
pub fn detect_cb513_silent_error_swallowing(project_path: &Path) -> Vec<CbPatternViolation> {
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

            // Detect .unwrap_or_else(|_| — intentionally discarded error
            if trimmed.contains(UNWRAP_OR_ELSE_DISCARD) {
                violations.push(CbPatternViolation {
                    pattern_id: "CB-513".to_string(),
                    file: file.clone(),
                    line: i + 1,
                    description: "Silent error swallowing: .unwrap_or_else(|_| discards error context".to_string(),
                    severity: Severity::Warning,
                });
                continue;
            }

            // Detect .map_err(|_| — discards original error
            if trimmed.contains(MAP_ERR_DISCARD) {
                violations.push(CbPatternViolation {
                    pattern_id: "CB-513".to_string(),
                    file: file.clone(),
                    line: i + 1,
                    description: "Silent error swallowing: .map_err(|_| discards original error context".to_string(),
                    severity: Severity::Warning,
                });
            }
        }
    }

    violations
}

// Use concat! to avoid self-detection by CB-514 scanner
const EPRINTLN_DEBUG: &str = concat!("eprintln!(\"[DEB", "UG");
const EPRINTLN_DBG: &str = concat!("eprintln!(\"[DB", "G");
const EPRINTLN_TRACE: &str = concat!("eprintln!(\"[TRA", "CE");

/// CB-514: Debug Eprintln Leaks - debug print statements in production code
pub fn detect_cb514_debug_eprintln_leaks(project_path: &Path) -> Vec<CbPatternViolation> {
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

            if trimmed.contains(EPRINTLN_DEBUG)
                || trimmed.contains(EPRINTLN_DBG)
                || trimmed.contains(EPRINTLN_TRACE)
            {
                violations.push(CbPatternViolation {
                    pattern_id: "CB-514".to_string(),
                    file: file.clone(),
                    line: i + 1,
                    description: "Debug eprintln! leak in production code".to_string(),
                    severity: Severity::Warning,
                });
            }
        }
    }

    violations
}

/// CB-515: Catch-All Match Default - `_ =>` returning concrete values instead of errors
pub fn detect_cb515_catch_all_match_default(project_path: &Path) -> Vec<CbPatternViolation> {
    let src_dir = project_path.join("src");
    let entries = match walkdir_rs_files(&src_dir) {
        Ok(e) => e,
        Err(_) => return Vec::new(),
    };

    let mut violations = Vec::new();

    // These are safe catch-all patterns that indicate proper error handling
    let safe_patterns = [
        "Err(", "None", "unreachable!", "panic!", "return Err",
        "return None", "bail!", "anyhow!", "todo!", "unimplemented!",
        "Default::default()", "default()",
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

            // Skip empty arms (multi-line blocks)
            if after.is_empty() || after == "{" {
                continue;
            }

            // Skip safe patterns
            if safe_patterns.iter().any(|p| after.contains(p)) {
                continue;
            }

            // Skip if it's just a closing brace or comma
            if after == "}" || after == "}," || after == "," {
                continue;
            }

            violations.push(CbPatternViolation {
                pattern_id: "CB-515".to_string(),
                file: file.clone(),
                line: i + 1,
                description: format!(
                    "Catch-all match arm `_ =>` returns concrete value: {}",
                    if after.len() > 60 { &after[..60] } else { after }
                ),
                severity: Severity::Warning,
            });
        }
    }

    violations
}

/// CB-516: Hardcoded Magic Numbers - large numeric literals in configuration contexts
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
        "100", "128", "256", "512", "1024", "2048", "4096", "8192",
        "1000", "1024", "65535", "65536",
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

/// CB-517: Stale Debug Artifacts - leftover debug instrumentation in production code
pub fn detect_cb517_stale_debug_artifacts(project_path: &Path) -> Vec<CbPatternViolation> {
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

            // Detect static AtomicUsize/AtomicBool debug counters outside const context
            if trimmed.contains("static")
                && (trimmed.contains("AtomicUsize") || trimmed.contains("AtomicBool"))
                && !trimmed.starts_with("const ")
            {
                violations.push(CbPatternViolation {
                    pattern_id: "CB-517".to_string(),
                    file: file.clone(),
                    line: i + 1,
                    description: "Stale debug artifact: static Atomic counter (likely debug instrumentation)".to_string(),
                    severity: Severity::Warning,
                });
                continue;
            }

            // Detect #[allow(unused)] on static variables (often leftover instrumentation)
            if trimmed == "#[allow(unused)]" || trimmed == "#[allow(dead_code)]" {
                // Check if next non-empty line is a static declaration
                for j in (i + 1)..std::cmp::min(i + 3, lines.len()) {
                    let next = lines[j].trim();
                    if next.is_empty() {
                        continue;
                    }
                    if next.starts_with("static ") {
                        violations.push(CbPatternViolation {
                            pattern_id: "CB-517".to_string(),
                            file: file.clone(),
                            line: i + 1,
                            description: "Stale debug artifact: #[allow(unused)] on static variable".to_string(),
                            severity: Severity::Warning,
                        });
                    }
                    break;
                }
            }
        }
    }

    violations
}

/// CB-518: Expensive Clone in Loop - .clone() calls inside loop bodies
pub fn detect_cb518_expensive_clone_in_loop(project_path: &Path) -> Vec<CbPatternViolation> {
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
        let file = entry
            .strip_prefix(project_path)
            .unwrap_or(entry)
            .display()
            .to_string();

        // Track loop bodies via brace depth
        let mut in_loop = false;
        let mut loop_depth: u32 = 0;
        let mut loop_start: usize = 0;
        let mut clone_count: u32 = 0;
        let mut clone_lines: Vec<usize> = Vec::new();

        for (i, line) in lines.iter().enumerate() {
            if test_lines.contains(&i) {
                continue;
            }
            let trimmed = line.trim();

            // Detect loop starts
            if !in_loop
                && (trimmed.starts_with("for ")
                    || trimmed.starts_with("while ")
                    || trimmed == "loop {"
                    || trimmed.starts_with("loop {"))
            {
                in_loop = true;
                loop_depth = 0;
                loop_start = i;
                clone_count = 0;
                clone_lines.clear();
            }

            if in_loop {
                loop_depth += trimmed.matches('{').count() as u32;
                loop_depth = loop_depth.saturating_sub(trimmed.matches('}').count() as u32);

                if trimmed.contains(".clone()") {
                    clone_count += 1;
                    clone_lines.push(i + 1);
                }

                // End of loop body
                if loop_depth == 0 && i > loop_start {
                    if clone_count > 3 {
                        violations.push(CbPatternViolation {
                            pattern_id: "CB-518".to_string(),
                            file: file.clone(),
                            line: loop_start + 1,
                            description: format!(
                                "Expensive clone in loop: {} .clone() calls (lines: {})",
                                clone_count,
                                clone_lines
                                    .iter()
                                    .take(5)
                                    .map(|l| l.to_string())
                                    .collect::<Vec<_>>()
                                    .join(", ")
                            ),
                            severity: Severity::Info,
                        });
                    }
                    in_loop = false;
                }
            }
        }
    }

    violations
}

/// CB-519: Lossy Data Pipeline - detect quantize/dequantize/encode/decode round-trip chains
pub fn detect_cb519_lossy_data_pipeline(project_path: &Path) -> Vec<CbPatternViolation> {
    let src_dir = project_path.join("src");
    let entries = match walkdir_rs_files(&src_dir) {
        Ok(e) => e,
        Err(_) => return Vec::new(),
    };

    // Lossy transform pairs: if both halves appear in the same function, it's suspicious
    let transform_pairs: &[(&str, &str)] = &[
        ("quantize", "dequantize"),
        ("encode", "decode"),
        ("compress", "decompress"),
        ("serialize", "deserialize"),
        ("pack", "unpack"),
        ("to_bytes", "from_bytes"),
        ("to_f16", "to_f32"),
        ("to_bf16", "to_f32"),
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

        // Track per-function: detect functions containing both halves of a lossy pair
        let mut fn_start: Option<usize> = None;
        let mut fn_depth: u32 = 0;
        let mut fn_content = String::new();

        for (i, line) in lines.iter().enumerate() {
            if test_lines.contains(&i) {
                continue;
            }
            let trimmed = line.trim();

            // Detect function start
            if (trimmed.starts_with("pub fn ")
                || trimmed.starts_with("fn ")
                || trimmed.starts_with("pub async fn ")
                || trimmed.starts_with("async fn "))
                && fn_start.is_none()
            {
                fn_start = Some(i);
                fn_depth = 0;
                fn_content.clear();
            }

            if fn_start.is_some() {
                fn_depth += trimmed.matches('{').count() as u32;
                fn_depth = fn_depth.saturating_sub(trimmed.matches('}').count() as u32);
                fn_content.push_str(trimmed);
                fn_content.push('\n');

                // End of function
                if fn_depth == 0 && i > fn_start.unwrap_or(i) {
                    // Strip derive annotations and comments to avoid false positives
                    // from #[derive(Serialize, Deserialize)] or doc comments
                    let filtered: String = fn_content
                        .lines()
                        .filter(|l| {
                            let t = l.trim();
                            !t.starts_with("#[derive(") && !t.starts_with("//")
                        })
                        .collect::<Vec<_>>()
                        .join("\n")
                        .to_lowercase();
                    for (fwd, rev) in transform_pairs {
                        if filtered.contains(fwd) && filtered.contains(rev) {
                            violations.push(CbPatternViolation {
                                pattern_id: "CB-519".to_string(),
                                file: file.clone(),
                                line: fn_start.unwrap_or(0) + 1,
                                description: format!(
                                    "Lossy data pipeline: both {fwd}() and {rev}() in same function — possible round-trip data corruption"
                                ),
                                severity: Severity::Warning,
                            });
                            break;
                        }
                    }
                    fn_start = None;
                    fn_content.clear();
                }
            }
        }
    }

    violations
}

/// CB-520: Expensive Init in Hot Path - constructor/load/open calls inside loops
pub fn detect_cb520_expensive_init_in_loop(project_path: &Path) -> Vec<CbPatternViolation> {
    let src_dir = project_path.join("src");
    let entries = match walkdir_rs_files(&src_dir) {
        Ok(e) => e,
        Err(_) => return Vec::new(),
    };

    let expensive_patterns = [
        "::new(", "::open(", "::connect(", "::create(", "::load(",
        "::init(", "::build(", "::from_file(", "::from_path(",
        "::read_to_string(", "File::open(",
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

        let mut in_loop = false;
        let mut loop_depth: u32 = 0;
        let mut loop_start: usize = 0;
        let mut init_count: u32 = 0;
        let mut init_examples: Vec<String> = Vec::new();

        for (i, line) in lines.iter().enumerate() {
            if test_lines.contains(&i) {
                continue;
            }
            let trimmed = line.trim();
            if trimmed.starts_with("//") || trimmed.starts_with("*") {
                continue;
            }

            if !in_loop
                && (trimmed.starts_with("for ")
                    || trimmed.starts_with("while ")
                    || trimmed == "loop {"
                    || trimmed.starts_with("loop {"))
            {
                in_loop = true;
                loop_depth = 0;
                loop_start = i;
                init_count = 0;
                init_examples.clear();
            }

            if in_loop {
                loop_depth += trimmed.matches('{').count() as u32;
                loop_depth = loop_depth.saturating_sub(trimmed.matches('}').count() as u32);

                for pat in &expensive_patterns {
                    if trimmed.contains(pat) {
                        init_count += 1;
                        if init_examples.len() < 3 {
                            init_examples.push(pat.trim_start_matches("::").to_string());
                        }
                        break;
                    }
                }

                if loop_depth == 0 && i > loop_start {
                    if init_count >= 2 {
                        violations.push(CbPatternViolation {
                            pattern_id: "CB-520".to_string(),
                            file: file.clone(),
                            line: loop_start + 1,
                            description: format!(
                                "Expensive initialization in loop: {} constructor/load calls ({})",
                                init_count,
                                init_examples.join(", ")
                            ),
                            severity: Severity::Warning,
                        });
                    }
                    in_loop = false;
                }
            }
        }
    }

    violations
}

/// CB-521: Format Detection Without Magic Bytes - binary parsing without header validation
pub fn detect_cb521_format_without_magic_bytes(project_path: &Path) -> Vec<CbPatternViolation> {
    let src_dir = project_path.join("src");
    let entries = match walkdir_rs_files(&src_dir) {
        Ok(e) => e,
        Err(_) => return Vec::new(),
    };

    // Binary read patterns that should be preceded by magic byte validation
    let binary_read_patterns = [
        "read_exact(", "from_le_bytes(", "from_be_bytes(",
        "read_u32::", "read_u64::", "read_i32::", "read_i64::",
    ];

    let magic_validation_patterns = [
        "magic", "MAGIC", "signature", "SIGNATURE", "header_magic",
        "file_type", "format_version", "FILE_MAGIC",
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

        // Per-function analysis: functions that read binary but never check magic bytes
        let mut fn_start: Option<usize> = None;
        let mut fn_depth: u32 = 0;
        let mut has_binary_read = false;
        let mut has_magic_check = false;
        let mut binary_line = 0usize;

        for (i, line) in lines.iter().enumerate() {
            if test_lines.contains(&i) {
                continue;
            }
            let trimmed = line.trim();

            if (trimmed.starts_with("pub fn ")
                || trimmed.starts_with("fn ")
                || trimmed.starts_with("pub async fn ")
                || trimmed.starts_with("async fn "))
                && fn_start.is_none()
            {
                fn_start = Some(i);
                fn_depth = 0;
                has_binary_read = false;
                has_magic_check = false;
                binary_line = 0;
            }

            if fn_start.is_some() {
                fn_depth += trimmed.matches('{').count() as u32;
                fn_depth = fn_depth.saturating_sub(trimmed.matches('}').count() as u32);

                if !trimmed.starts_with("//") {
                    if binary_read_patterns.iter().any(|p| trimmed.contains(p)) {
                        if !has_binary_read {
                            binary_line = i;
                        }
                        has_binary_read = true;
                    }
                    if magic_validation_patterns.iter().any(|p| trimmed.contains(p)) {
                        has_magic_check = true;
                    }
                }

                if fn_depth == 0 && i > fn_start.unwrap_or(i) {
                    if has_binary_read && !has_magic_check {
                        violations.push(CbPatternViolation {
                            pattern_id: "CB-521".to_string(),
                            file: file.clone(),
                            line: binary_line + 1,
                            description: "Binary format parsing without magic byte/header validation".to_string(),
                            severity: Severity::Warning,
                        });
                    }
                    fn_start = None;
                }
            }
        }
    }

    violations
}

/// CB-522: Untested Path Normalization - path manipulation without edge case handling
pub fn detect_cb522_untested_path_normalization(project_path: &Path) -> Vec<CbPatternViolation> {
    let src_dir = project_path.join("src");
    let entries = match walkdir_rs_files(&src_dir) {
        Ok(e) => e,
        Err(_) => return Vec::new(),
    };

    // Path manipulation patterns that indicate URL/path normalization
    let path_manip_patterns = [
        ".strip_prefix(\"http", ".replace(\"//\"", ".replace(\"resolve/\"",
        "split(\"://\")", "trim_start_matches(\"http", "Url::parse(",
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

        let mut path_manip_count = 0u32;
        let mut first_line = 0usize;

        for (i, line) in lines.iter().enumerate() {
            if test_lines.contains(&i) {
                continue;
            }
            let trimmed = line.trim();
            if trimmed.starts_with("//") {
                continue;
            }

            if path_manip_patterns.iter().any(|p| trimmed.contains(p)) {
                if path_manip_count == 0 {
                    first_line = i;
                }
                path_manip_count += 1;
            }
        }

        // Multiple path manipulations in one file suggest complex URL/path normalization
        if path_manip_count >= 3 {
            violations.push(CbPatternViolation {
                pattern_id: "CB-522".to_string(),
                file,
                line: first_line + 1,
                description: format!(
                    "{path_manip_count} path/URL manipulation operations — verify edge cases (double slashes, web URLs, relative paths) are tested"
                ),
                severity: Severity::Info,
            });
        }
    }

    violations
}

/// CB-523: External Config Over Embedded Metadata - filesystem heuristics instead of embedded data
pub fn detect_cb523_external_config_over_embedded(project_path: &Path) -> Vec<CbPatternViolation> {
    let src_dir = project_path.join("src");
    let entries = match walkdir_rs_files(&src_dir) {
        Ok(e) => e,
        Err(_) => return Vec::new(),
    };

    let mut violations = Vec::new();

    // Filesystem heuristic patterns
    let fs_heuristic_patterns = [
        ".with_file_name(", ".with_extension(",
    ];
    let config_discovery = [
        "config.json", "tokenizer.json", "generation_config",
        "model.json", "params.json", "hyperparams",
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
            if trimmed.starts_with("//") {
                continue;
            }

            // Detect: path.with_file_name("config.json") or similar sibling file discovery
            let has_fs_heuristic = fs_heuristic_patterns.iter().any(|p| trimmed.contains(p));
            let has_config_discovery = config_discovery.iter().any(|p| trimmed.contains(p));

            if has_fs_heuristic && has_config_discovery {
                violations.push(CbPatternViolation {
                    pattern_id: "CB-523".to_string(),
                    file: file.clone(),
                    line: i + 1,
                    description: "External config discovery via filesystem heuristic — prefer embedded metadata if available".to_string(),
                    severity: Severity::Info,
                });
            }
        }
    }

    violations
}

/// CB-524: Incomplete Enum Match Coverage - wildcard matches on project enums across functions
pub fn detect_cb524_incomplete_enum_match(project_path: &Path) -> Vec<CbPatternViolation> {
    let src_dir = project_path.join("src");
    let entries = match walkdir_rs_files(&src_dir) {
        Ok(e) => e,
        Err(_) => return Vec::new(),
    };

    let mut violations = Vec::new();

    // Track: for each file, count match blocks that use _ => catch-all
    // If a file has many _ => catch-all arms with different concrete return types,
    // it's likely dispatching on the same enum inconsistently
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

        let mut wildcard_match_count = 0u32;
        let mut wildcard_lines: Vec<usize> = Vec::new();

        for (i, line) in lines.iter().enumerate() {
            if test_lines.contains(&i) {
                continue;
            }
            let trimmed = line.trim();
            if trimmed.starts_with("//") {
                continue;
            }

            // Count _ => arms that return concrete values (not errors)
            if trimmed.starts_with("_ =>") || trimmed.starts_with("_=>") {
                let after = trimmed
                    .trim_start_matches("_ =>")
                    .trim_start_matches("_=>")
                    .trim();

                // Skip error/none/panic patterns — these are deliberate catch-alls
                let safe_patterns = [
                    "Err(", "None", "unreachable!", "panic!", "return Err",
                    "bail!", "todo!", "unimplemented!", "Default::default()",
                ];
                let is_safe = after.is_empty()
                    || after == "{"
                    || after == "}"
                    || after == "},"
                    || safe_patterns.iter().any(|p| after.contains(p));

                if !is_safe {
                    wildcard_match_count += 1;
                    wildcard_lines.push(i + 1);
                }
            }
        }

        // If a single file has 3+ wildcard match arms with concrete returns,
        // it's dispatching on an enum in multiple places with catch-all defaults
        if wildcard_match_count >= 3 {
            violations.push(CbPatternViolation {
                pattern_id: "CB-524".to_string(),
                file,
                line: wildcard_lines.first().copied().unwrap_or(0),
                description: format!(
                    "{wildcard_match_count} catch-all match arms with concrete defaults in single file (lines: {}) — enum variants may be inconsistently handled",
                    wildcard_lines.iter().take(5).map(|l| l.to_string()).collect::<Vec<_>>().join(", ")
                ),
                severity: Severity::Warning,
            });
        }
    }

    violations
}

/// CB-525: Hardcoded Field Names Without Aliases - JSON .get("field") chains without fallbacks
pub fn detect_cb525_hardcoded_field_names(project_path: &Path) -> Vec<CbPatternViolation> {
    let src_dir = project_path.join("src");
    let entries = match walkdir_rs_files(&src_dir) {
        Ok(e) => e,
        Err(_) => return Vec::new(),
    };

    let json_get_re = regex::Regex::new(r#"\.get\(\s*""#).expect("valid regex");

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

        // Per-function: count .get("field") calls without .or_else fallback
        let mut fn_start: Option<usize> = None;
        let mut fn_depth: u32 = 0;
        let mut get_count: u32 = 0;
        let mut has_or_fallback = false;

        for (i, line) in lines.iter().enumerate() {
            if test_lines.contains(&i) {
                continue;
            }
            let trimmed = line.trim();

            if (trimmed.starts_with("pub fn ")
                || trimmed.starts_with("fn ")
                || trimmed.starts_with("pub async fn ")
                || trimmed.starts_with("async fn "))
                && fn_start.is_none()
            {
                fn_start = Some(i);
                fn_depth = 0;
                get_count = 0;
                has_or_fallback = false;
            }

            if fn_start.is_some() {
                fn_depth += trimmed.matches('{').count() as u32;
                fn_depth = fn_depth.saturating_sub(trimmed.matches('}').count() as u32);

                if !trimmed.starts_with("//") {
                    if json_get_re.is_match(trimmed) {
                        get_count += 1;
                    }
                    if trimmed.contains(".or_else(") || trimmed.contains(".or(") {
                        has_or_fallback = true;
                    }
                }

                if fn_depth == 0 && i > fn_start.unwrap_or(i) {
                    // 5+ .get("field") without any .or_else/.or fallback alias support
                    if get_count >= 5 && !has_or_fallback {
                        violations.push(CbPatternViolation {
                            pattern_id: "CB-525".to_string(),
                            file: file.clone(),
                            line: fn_start.unwrap_or(0) + 1,
                            description: format!(
                                "{get_count} hardcoded .get(\"field\") calls without alias fallbacks — schemas with alternative field names will fail silently"
                            ),
                            severity: Severity::Info,
                        });
                    }
                    fn_start = None;
                }
            }
        }
    }

    violations
}

/// CB-526: Single-Path File Resolution - file lookup without fallback search
pub fn detect_cb526_single_path_resolution(project_path: &Path) -> Vec<CbPatternViolation> {
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
            if trimmed.starts_with("//") {
                continue;
            }

            // Pattern: path.join("specific_file.ext").exists() without fallback
            // or: path.join("specific_file.ext") followed by read without exists check
            if trimmed.contains(".join(\"") && trimmed.contains(".exists()") {
                // Check if there's a fallback on same or next line
                let next_trimmed = lines
                    .get(i + 1)
                    .map(|l| l.trim())
                    .unwrap_or("");
                let has_fallback = trimmed.contains("||")
                    || trimmed.contains(".or_else")
                    || next_trimmed.contains("||")
                    || next_trimmed.contains("else {")
                    || next_trimmed.contains(".parent()");

                if !has_fallback {
                    violations.push(CbPatternViolation {
                        pattern_id: "CB-526".to_string(),
                        file: file.clone(),
                        line: i + 1,
                        description: "Single-path file resolution without fallback — consider parent directory or recursive search".to_string(),
                        severity: Severity::Info,
                    });
                }
            }
        }
    }

    violations
}

/// CB-527: Incomplete Pattern List - contains()/starts_with() classification chains
pub fn detect_cb527_incomplete_pattern_list(project_path: &Path) -> Vec<CbPatternViolation> {
    let src_dir = project_path.join("src");
    let entries = match walkdir_rs_files(&src_dir) {
        Ok(e) => e,
        Err(_) => return Vec::new(),
    };

    let classification_re = regex::Regex::new(
        r#"\.contains\(\s*"[a-z_]+"\s*\)\s*\|\|"#,
    )
    .expect("valid regex");

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

        // Look for chains of .contains("x") || .contains("y") || ... — classification patterns
        for (i, line) in lines.iter().enumerate() {
            if test_lines.contains(&i) {
                continue;
            }
            let trimmed = line.trim();
            if trimmed.starts_with("//") {
                continue;
            }

            // Count contains() calls chained with ||
            let chain_count = classification_re.find_iter(trimmed).count();

            // Only check continuation on next line if current line starts a chain
            let next_chain = if chain_count > 0 {
                lines
                    .get(i + 1)
                    .map(|l| classification_re.find_iter(l.trim()).count())
                    .unwrap_or(0)
            } else {
                0
            };

            let total = chain_count + next_chain;

            if total >= 3 {
                violations.push(CbPatternViolation {
                    pattern_id: "CB-527".to_string(),
                    file: file.clone(),
                    line: i + 1,
                    description: format!(
                        "Classification chain with {total}+ .contains() patterns — may be incomplete; consider a centralized pattern registry"
                    ),
                    severity: Severity::Info,
                });
            }
        }
    }

    violations
}

/// Helper: check if panic macro text appears only inside a string literal
fn is_macro_in_string_literal(trimmed: &str, macros: &[&str]) -> bool {
    if !trimmed.contains('"') {
        return false;
    }
    let before_string = trimmed.split('"').next().unwrap_or("");
    !macros.iter().any(|m| before_string.contains(m))
}

/// Helper: walk directory for files with a specific extension
fn walkdir_files_with_ext(
    dir: &Path,
    ext: &str,
) -> Result<Vec<std::path::PathBuf>, std::io::Error> {
    let mut files = Vec::new();
    for entry in fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();
        if path.is_dir() {
            files.extend(walkdir_files_with_ext(&path, ext)?);
        } else if path.extension().map(|e| e == ext).unwrap_or(false) {
            files.push(path);
        }
    }
    Ok(files)
}

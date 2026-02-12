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

    let has_workspace_lints = content.contains("[workspace.lints]");
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

        // Only check test code
        if !content.contains("#[test]") && !content.contains("#[tokio::test]") {
            continue;
        }

        let file = entry
            .strip_prefix(project_path)
            .unwrap_or(entry)
            .display()
            .to_string();

        let has_instant = content.contains("Instant::now()");
        let has_elapsed = content.contains(".elapsed()");
        let has_duration_assert = content.contains("assert!(") && content.contains("elapsed");

        if has_instant && has_elapsed && has_duration_assert {
            violations.push(CbPatternViolation {
                pattern_id: "CB-511".to_string(),
                file,
                line: 0,
                description: "Test uses Instant::now() with duration assertions - may be flaky under load".to_string(),
                severity: Severity::Warning,
            });
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

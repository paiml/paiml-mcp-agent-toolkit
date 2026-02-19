#![cfg_attr(coverage_nightly, coverage(off))]
//! CB-1050 Series: Lean 4 Best Practices Detection
//!
//! Proof quality checks for `pmat comply check` on Lean 4 projects.
//!
//! CB-1050: Sorry count (incomplete proofs)
//! CB-1051: Axiom usage (non-standard axioms weaken trust)
//! CB-1052: Theorem coverage (ratio of theorems to sorry)
//! CB-1053: Missing doc comments on public theorems

use super::types::*;
use std::fs;
use std::path::{Path, PathBuf};

/// Directories to skip when walking for Lean files.
const SKIP_DIRS: &[&str] = &[
    ".git",
    "node_modules",
    "target",
    ".pmat",
    "build",
    ".lake",
    "lake-packages",
];

/// Walk directory recursively for `.lean` files, skipping common non-source dirs.
pub fn walkdir_lean_files(dir: &Path) -> Vec<PathBuf> {
    let mut files = Vec::new();
    walk_lean_recursive(dir, &mut files);
    files
}

fn walk_lean_recursive(dir: &Path, files: &mut Vec<PathBuf>) {
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
                walk_lean_recursive(&path, files);
            }
        } else if path.extension().is_some_and(|e| e == "lean") {
            // Skip lakefile.lean and lean-toolchain
            let name = path.file_name().and_then(|n| n.to_str()).unwrap_or("");
            if name != "lakefile.lean" {
                files.push(path);
            }
        }
    }
}

/// CB-1050: Detect sorry usage (incomplete proofs).
/// Sorry is Lean's escape hatch for unfinished proofs — it should be zero in production.
pub fn detect_cb1050_sorry_usage(project_path: &Path) -> Vec<CbPatternViolation> {
    let files = walkdir_lean_files(project_path);
    let mut violations = Vec::new();

    for file_path in &files {
        let content = match fs::read_to_string(file_path) {
            Ok(c) => c,
            Err(_) => continue,
        };
        let file = file_path.display().to_string();
        let mut in_block_comment = 0i32;

        for (i, line) in content.lines().enumerate() {
            let trimmed = line.trim();

            if trimmed.starts_with("--") {
                continue;
            }

            // Track block comments
            let cleaned = strip_lean_block_comments(trimmed, &mut in_block_comment);
            if in_block_comment > 0 {
                continue;
            }

            if contains_sorry_word(&cleaned) {
                violations.push(CbPatternViolation {
                    pattern_id: "CB-1050".to_string(),
                    file: file.clone(),
                    line: i + 1,
                    description: "Incomplete proof: sorry used".to_string(),
                    severity: Severity::Error,
                });
            }
        }
    }

    violations
}

/// CB-1051: Detect non-standard axiom usage.
/// Custom axioms weaken the trust base — flag any `axiom` declaration.
pub fn detect_cb1051_axiom_usage(project_path: &Path) -> Vec<CbPatternViolation> {
    let files = walkdir_lean_files(project_path);
    let mut violations = Vec::new();

    for file_path in &files {
        let content = match fs::read_to_string(file_path) {
            Ok(c) => c,
            Err(_) => continue,
        };
        let file = file_path.display().to_string();

        for (i, line) in content.lines().enumerate() {
            let trimmed = line.trim();
            if trimmed.starts_with("axiom ") {
                violations.push(CbPatternViolation {
                    pattern_id: "CB-1051".to_string(),
                    file: file.clone(),
                    line: i + 1,
                    description: "Custom axiom declaration weakens trust base".to_string(),
                    severity: Severity::Warning,
                });
            }
        }
    }

    violations
}

/// CB-1052: Theorem coverage check.
/// Warns if sorry/theorem ratio exceeds 20% (more than 1 in 5 theorems incomplete).
pub fn detect_cb1052_theorem_coverage(project_path: &Path) -> Vec<CbPatternViolation> {
    let files = walkdir_lean_files(project_path);
    let mut total_theorems = 0usize;
    let mut total_sorrys = 0usize;

    for file_path in &files {
        let content = match fs::read_to_string(file_path) {
            Ok(c) => c,
            Err(_) => continue,
        };

        let mut in_block_comment = 0i32;
        for line in content.lines() {
            let trimmed = line.trim();
            if trimmed.starts_with("--") {
                continue;
            }
            let cleaned = strip_lean_block_comments(trimmed, &mut in_block_comment);
            if in_block_comment > 0 {
                continue;
            }
            if cleaned.trim_start().starts_with("theorem ")
                || cleaned.trim_start().starts_with("lemma ")
            {
                total_theorems += 1;
            }
            if contains_sorry_word(&cleaned) {
                total_sorrys += 1;
            }
        }
    }

    if total_theorems == 0 {
        return Vec::new();
    }

    let ratio = total_sorrys as f64 / total_theorems as f64;
    if ratio > 0.2 {
        vec![CbPatternViolation {
            pattern_id: "CB-1052".to_string(),
            file: project_path.display().to_string(),
            line: 0,
            description: format!(
                "Low theorem coverage: {}/{} theorems incomplete ({:.0}% sorry ratio, threshold: 20%)",
                total_sorrys, total_theorems, ratio * 100.0
            ),
            severity: Severity::Warning,
        }]
    } else {
        Vec::new()
    }
}

/// CB-1053: Missing doc comments on public theorems.
/// Public theorems should have doc comments (`/-- ... -/`) for maintainability.
pub fn detect_cb1053_undocumented_theorems(project_path: &Path) -> Vec<CbPatternViolation> {
    let files = walkdir_lean_files(project_path);
    let mut violations = Vec::new();

    for file_path in &files {
        let content = match fs::read_to_string(file_path) {
            Ok(c) => c,
            Err(_) => continue,
        };
        let file = file_path.display().to_string();
        let lines: Vec<&str> = content.lines().collect();
        let mut in_block_comment = 0i32;

        for (i, line) in lines.iter().enumerate() {
            let trimmed = line.trim();

            // Skip line comments
            if trimmed.starts_with("--") {
                continue;
            }

            // Track block comments (including /- and /-! doc blocks)
            let cleaned = strip_lean_block_comments(trimmed, &mut in_block_comment);
            if in_block_comment > 0 {
                continue;
            }

            let cleaned_trimmed = cleaned.trim();
            if cleaned_trimmed.starts_with("theorem ") || cleaned_trimmed.starts_with("lemma ") {
                // Check if preceding non-empty line is a doc comment
                let has_doc = (0..i).rev().any(|j| {
                    let prev = lines[j].trim();
                    if prev.is_empty() {
                        return false; // skip blanks, keep looking
                    }
                    prev.ends_with("-/") || prev.starts_with("/--") || prev.starts_with("--")
                });
                if !has_doc {
                    let name = cleaned_trimmed
                        .split_whitespace()
                        .nth(1)
                        .unwrap_or("unknown");
                    violations.push(CbPatternViolation {
                        pattern_id: "CB-1053".to_string(),
                        file: file.clone(),
                        line: i + 1,
                        description: format!(
                            "Public theorem '{}' missing doc comment",
                            name
                        ),
                        severity: Severity::Info,
                    });
                }
            }
        }
    }

    violations
}

/// Strip block comment content from a line, updating nesting depth.
fn strip_lean_block_comments(line: &str, depth: &mut i32) -> String {
    let bytes = line.as_bytes();
    let mut result = String::with_capacity(line.len());
    let mut i = 0;

    while i < bytes.len() {
        if i + 1 < bytes.len() && bytes[i] == b'/' && bytes[i + 1] == b'-' {
            *depth += 1;
            i += 2;
            continue;
        }
        if i + 1 < bytes.len() && bytes[i] == b'-' && bytes[i + 1] == b'/' && *depth > 0 {
            *depth -= 1;
            i += 2;
            continue;
        }
        if *depth == 0 {
            result.push(bytes[i] as char);
        }
        i += 1;
    }

    result
}

/// Check if line contains "sorry" as a standalone word.
fn contains_sorry_word(line: &str) -> bool {
    let bytes = line.as_bytes();
    let sorry = b"sorry";
    let mut pos = 0;
    while pos + sorry.len() <= bytes.len() {
        if let Some(idx) = line[pos..].find("sorry") {
            let abs_idx = pos + idx;
            let before_ok = abs_idx == 0
                || !(bytes[abs_idx - 1].is_ascii_alphanumeric() || bytes[abs_idx - 1] == b'_');
            let after_ok = abs_idx + sorry.len() >= bytes.len()
                || !(bytes[abs_idx + sorry.len()].is_ascii_alphanumeric()
                    || bytes[abs_idx + sorry.len()] == b'_');
            if before_ok && after_ok {
                return true;
            }
            pos = abs_idx + 1;
        } else {
            break;
        }
    }
    false
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn test_walkdir_lean_files_basic() {
        let dir = TempDir::new().unwrap();
        fs::write(dir.path().join("test.lean"), "theorem t : True := by trivial").unwrap();
        fs::write(dir.path().join("lakefile.lean"), "-- lakefile").unwrap();

        let files = walkdir_lean_files(dir.path());
        assert_eq!(files.len(), 1, "Should find 1 lean file (not lakefile)");
    }

    #[test]
    fn test_detect_cb1050_sorry() {
        let dir = TempDir::new().unwrap();
        fs::write(
            dir.path().join("test.lean"),
            "theorem t : True := by sorry\ntheorem t2 : True := by trivial",
        )
        .unwrap();

        let violations = detect_cb1050_sorry_usage(dir.path());
        assert_eq!(violations.len(), 1);
        assert_eq!(violations[0].pattern_id, "CB-1050");
    }

    #[test]
    fn test_detect_cb1050_sorry_in_comment_ignored() {
        let dir = TempDir::new().unwrap();
        fs::write(
            dir.path().join("test.lean"),
            "-- sorry this is a comment\n/- sorry in block -/\ntheorem t : True := by trivial",
        )
        .unwrap();

        let violations = detect_cb1050_sorry_usage(dir.path());
        assert_eq!(violations.len(), 0);
    }

    #[test]
    fn test_detect_cb1051_axiom() {
        let dir = TempDir::new().unwrap();
        fs::write(
            dir.path().join("test.lean"),
            "axiom choice : ∀ (α : Type), Nonempty α → α",
        )
        .unwrap();

        let violations = detect_cb1051_axiom_usage(dir.path());
        assert_eq!(violations.len(), 1);
        assert_eq!(violations[0].pattern_id, "CB-1051");
    }

    #[test]
    fn test_detect_cb1052_low_coverage() {
        let dir = TempDir::new().unwrap();
        fs::write(
            dir.path().join("test.lean"),
            "theorem t1 : True := by sorry\ntheorem t2 : True := by sorry\ntheorem t3 : True := by trivial",
        )
        .unwrap();

        let violations = detect_cb1052_theorem_coverage(dir.path());
        assert_eq!(violations.len(), 1);
        assert_eq!(violations[0].pattern_id, "CB-1052");
    }

    #[test]
    fn test_detect_cb1052_good_coverage() {
        let dir = TempDir::new().unwrap();
        fs::write(
            dir.path().join("test.lean"),
            "theorem t1 : True := by trivial\ntheorem t2 : True := by trivial\ntheorem t3 : True := by trivial",
        )
        .unwrap();

        let violations = detect_cb1052_theorem_coverage(dir.path());
        assert_eq!(violations.len(), 0);
    }

    #[test]
    fn test_detect_cb1053_undocumented() {
        let dir = TempDir::new().unwrap();
        fs::write(
            dir.path().join("test.lean"),
            "theorem undoc : True := by trivial\n/-- documented -/\ntheorem documented : True := by trivial",
        )
        .unwrap();

        let violations = detect_cb1053_undocumented_theorems(dir.path());
        assert_eq!(violations.len(), 1);
        assert_eq!(violations[0].pattern_id, "CB-1053");
    }

    #[test]
    fn test_sorry_word_boundary() {
        assert!(!contains_sorry_word("def sorry_helper := 42"));
        assert!(contains_sorry_word("theorem t := by sorry"));
        assert!(contains_sorry_word("  sorry"));
        // Note: comment stripping is done by the caller, not contains_sorry_word
        assert!(contains_sorry_word("-- sorry")); // word boundary only, no comment logic
    }
}

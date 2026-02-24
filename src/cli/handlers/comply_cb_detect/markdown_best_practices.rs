#![cfg_attr(coverage_nightly, coverage(off))]
//! CB-900 Series: Markdown Best Practices Detection
//!
//! Pattern-based Markdown quality detection for `pmat comply check`.
//! Focuses on documentation quality: heading structure, link validation,
//! and readability.

use super::types::*;
use std::fs;
use std::path::{Path, PathBuf};

/// Directories to skip when walking for Markdown files.
const SKIP_DIRS: &[&str] = &[
    ".git",
    ".claude",
    "node_modules",
    "target",
    ".pmat",
    "vendor",
    "build",
    "dist",
    "__pycache__",
    ".venv",
    "site-packages",
];

// =============================================================================
// File walking
// =============================================================================

/// Walk directory recursively for `.md`/`.mdx` files.
pub fn walkdir_markdown_files(dir: &Path) -> Vec<PathBuf> {
    let mut files = Vec::new();
    walk_md_recursive(dir, &mut files);
    files
}

fn walk_md_recursive(dir: &Path, files: &mut Vec<PathBuf>) {
    let entries = match fs::read_dir(dir) {
        Ok(e) => e,
        Err(_) => return,
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            let dir_name = path.file_name().and_then(|n| n.to_str()).unwrap_or("");
            if !SKIP_DIRS.contains(&dir_name) {
                walk_md_recursive(&path, files);
            }
        } else if path
            .extension()
            .and_then(|e| e.to_str())
            .map(|e| matches!(e, "md" | "mdx" | "markdown"))
            .unwrap_or(false)
        {
            files.push(path);
        }
    }
}

// =============================================================================
// CB-900: Internal link validation
// =============================================================================

pub fn detect_cb900_broken_internal_link(project_path: &Path) -> Vec<CbPatternViolation> {
    let files = walkdir_markdown_files(project_path);
    let mut violations = Vec::new();

    for file_path in &files {
        let content = match fs::read_to_string(file_path) {
            Ok(c) => c,
            Err(_) => continue,
        };
        let rel = file_path
            .strip_prefix(project_path)
            .unwrap_or(file_path)
            .display()
            .to_string();
        let file_dir = file_path.parent().unwrap_or(project_path);

        for (i, line) in content.lines().enumerate() {
            // Skip code blocks
            if line.trim().starts_with("```") {
                continue;
            }

            // Find markdown links: [text](path)
            let mut search_pos = 0;
            while let Some(start) = line[search_pos..].find("](") {
                let abs_start = search_pos + start + 2;
                if let Some(end) = line[abs_start..].find(')') {
                    let link_target = &line[abs_start..abs_start + end];

                    // Only check internal links (not http/https/mailto/#anchors)
                    if !link_target.starts_with("http")
                        && !link_target.starts_with("mailto:")
                        && !link_target.starts_with('#')
                        && !link_target.is_empty()
                    {
                        // Strip anchor from link
                        let file_part = link_target.split('#').next().unwrap_or(link_target);
                        if !file_part.is_empty() {
                            let target_path = file_dir.join(file_part);
                            if !target_path.exists() {
                                violations.push(CbPatternViolation {
                                    pattern_id: "CB-900".to_string(),
                                    file: rel.clone(),
                                    line: i + 1,
                                    description: format!(
                                        "Broken internal link `{}` — target does not exist",
                                        link_target
                                    ),
                                    severity: Severity::Warning,
                                });
                            }
                        }
                    }

                    search_pos = abs_start + end + 1;
                } else {
                    break;
                }
            }
        }
    }

    violations
}

// =============================================================================
// CB-901: Heading Hierarchy Skip
// =============================================================================

pub fn detect_cb901_heading_hierarchy_skip(project_path: &Path) -> Vec<CbPatternViolation> {
    let files = walkdir_markdown_files(project_path);
    let mut violations = Vec::new();

    for file_path in &files {
        let content = match fs::read_to_string(file_path) {
            Ok(c) => c,
            Err(_) => continue,
        };
        let rel = file_path
            .strip_prefix(project_path)
            .unwrap_or(file_path)
            .display()
            .to_string();

        let mut last_level: usize = 0;
        let mut in_code_block = false;

        for (i, line) in content.lines().enumerate() {
            let trimmed = line.trim();

            // Track code blocks
            if trimmed.starts_with("```") {
                in_code_block = !in_code_block;
                continue;
            }
            if in_code_block {
                continue;
            }

            // Count heading level
            if trimmed.starts_with('#') {
                let level = trimmed.chars().take_while(|c| *c == '#').count();
                if (1..=6).contains(&level) {
                    // Check for skip: e.g., h1 -> h3 (skip h2)
                    if last_level > 0 && level > last_level + 1 {
                        violations.push(CbPatternViolation {
                            pattern_id: "CB-901".to_string(),
                            file: rel.clone(),
                            line: i + 1,
                            description: format!(
                                "Heading hierarchy skip: h{} to h{} — missing h{}",
                                last_level,
                                level,
                                last_level + 1
                            ),
                            severity: Severity::Info,
                        });
                    }
                    last_level = level;
                }
            }
        }
    }

    violations
}

// =============================================================================
// CB-902: Missing Alt Text on Images
// =============================================================================

pub fn detect_cb902_missing_alt_text(project_path: &Path) -> Vec<CbPatternViolation> {
    let files = walkdir_markdown_files(project_path);
    let mut violations = Vec::new();

    for file_path in &files {
        let content = match fs::read_to_string(file_path) {
            Ok(c) => c,
            Err(_) => continue,
        };
        let rel = file_path
            .strip_prefix(project_path)
            .unwrap_or(file_path)
            .display()
            .to_string();

        let mut in_code_block = false;

        for (i, line) in content.lines().enumerate() {
            let trimmed = line.trim();
            if trimmed.starts_with("```") {
                in_code_block = !in_code_block;
                continue;
            }
            if in_code_block {
                continue;
            }

            // Find ![](url) pattern — missing alt text
            if line.contains("![]") {
                violations.push(CbPatternViolation {
                    pattern_id: "CB-902".to_string(),
                    file: rel.clone(),
                    line: i + 1,
                    description:
                        "Image missing alt text — add descriptive text in `![alt text](url)`"
                            .to_string(),
                    severity: Severity::Info,
                });
            }
        }
    }

    violations
}

// =============================================================================
// CB-903: Bare URL
// =============================================================================

pub fn detect_cb903_bare_url(project_path: &Path) -> Vec<CbPatternViolation> {
    let files = walkdir_markdown_files(project_path);
    let mut violations = Vec::new();

    for file_path in &files {
        let content = match fs::read_to_string(file_path) {
            Ok(c) => c,
            Err(_) => continue,
        };
        let rel = file_path
            .strip_prefix(project_path)
            .unwrap_or(file_path)
            .display()
            .to_string();

        let mut in_code_block = false;

        for (i, line) in content.lines().enumerate() {
            let trimmed = line.trim();
            if trimmed.starts_with("```") {
                in_code_block = !in_code_block;
                continue;
            }
            if in_code_block {
                continue;
            }

            // Find bare URLs (http/https not wrapped in markdown link or angle brackets)
            if let Some(http_pos) = line.find("http://").or_else(|| line.find("https://")) {
                // Check if it's already in a markdown link or angle brackets
                if http_pos > 0 {
                    let before = line.as_bytes()[http_pos - 1];
                    if before == b'(' || before == b'<' || before == b'"' || before == b'\'' {
                        continue;
                    }
                }
                // Check if line is a markdown link definition or image
                if trimmed.starts_with('[') || trimmed.starts_with("![") {
                    continue;
                }
                // Check if the URL is the only thing on the line (common in link lists)
                if trimmed.starts_with("http://") || trimmed.starts_with("https://") {
                    violations.push(CbPatternViolation {
                        pattern_id: "CB-903".to_string(),
                        file: rel.clone(),
                        line: i + 1,
                        description: "Bare URL — wrap in markdown link `[text](url)` or angle brackets `<url>`"
                            .to_string(),
                        severity: Severity::Info,
                    });
                }
            }
        }
    }

    violations
}

// =============================================================================
// CB-904: Long Line
// =============================================================================

/// Default line length threshold for markdown files.
const MD_LINE_LENGTH_THRESHOLD: usize = 120;

pub fn detect_cb904_long_line(project_path: &Path) -> Vec<CbPatternViolation> {
    let files = walkdir_markdown_files(project_path);
    let mut violations = Vec::new();

    for file_path in &files {
        let content = match fs::read_to_string(file_path) {
            Ok(c) => c,
            Err(_) => continue,
        };
        let rel = file_path
            .strip_prefix(project_path)
            .unwrap_or(file_path)
            .display()
            .to_string();

        let mut in_code_block = false;

        for (i, line) in content.lines().enumerate() {
            let trimmed = line.trim();
            if trimmed.starts_with("```") {
                in_code_block = !in_code_block;
                continue;
            }
            // Skip code blocks (long lines are expected in code examples)
            if in_code_block {
                continue;
            }
            // Skip tables (lines with pipes)
            if trimmed.starts_with('|') {
                continue;
            }
            // Skip lines that are mostly URLs
            if trimmed.contains("http://") || trimmed.contains("https://") {
                continue;
            }

            if line.len() > MD_LINE_LENGTH_THRESHOLD {
                violations.push(CbPatternViolation {
                    pattern_id: "CB-904".to_string(),
                    file: rel.clone(),
                    line: i + 1,
                    description: format!(
                        "Line length {} exceeds {} characters",
                        line.len(),
                        MD_LINE_LENGTH_THRESHOLD
                    ),
                    severity: Severity::Info,
                });
            }
        }
    }

    violations
}

#![cfg_attr(coverage_nightly, coverage(off))]
//! CB-900 Series: Markdown Best Practices Detection
//!
//! Pattern-based Markdown quality detection for `pmat comply check`.
//! Focuses on documentation quality: heading structure, link validation,
//! and readability.

use super::types::*;
use ignore::WalkBuilder;
use std::fs;
use std::path::{Path, PathBuf};

/// Directories to skip when walking for Markdown files.
///
/// These are additional hard-coded skips layered on top of `.gitignore`,
/// `.pmatignore`, and `.paimlignore` (honored via `ignore::WalkBuilder`).
/// They catch build artifacts and vendored directories that may not be
/// listed in an ignore file.
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
///
/// Honors `.gitignore`, `.pmatignore`, and `.paimlignore` plus exclude
/// patterns from `.pmat-gates.toml [exclude] paths` and `.pmat.yaml
/// comply.thresholds.file_health_exclude` (GH-278).
#[provable_contracts_macros::contract("pmat-core.yaml", equation = "path_exists")]
pub fn walkdir_markdown_files(dir: &Path) -> Vec<PathBuf> {
    let excludes = load_markdown_excludes(dir);
    walkdir_markdown_files_with_excludes(dir, &excludes)
}

/// Walk for Markdown files applying explicit glob excludes in addition to
/// the ignore-file rules baked into [`WalkBuilder`].
pub fn walkdir_markdown_files_with_excludes(dir: &Path, excludes: &[String]) -> Vec<PathBuf> {
    let mut files = Vec::new();

    let walker = WalkBuilder::new(dir)
        .hidden(false)
        .git_ignore(true)
        .git_exclude(true)
        .add_custom_ignore_filename(".pmatignore")
        .add_custom_ignore_filename(".paimlignore")
        .filter_entry(|entry| {
            let name = entry
                .path()
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("");
            !SKIP_DIRS.contains(&name)
        })
        .build();

    for entry in walker.flatten() {
        let path = entry.path();
        if !path.is_file() {
            continue;
        }
        let is_md = path
            .extension()
            .and_then(|e| e.to_str())
            .map(|e| matches!(e, "md" | "mdx" | "markdown"))
            .unwrap_or(false);
        if !is_md {
            continue;
        }
        if path_matches_any_exclude(path, dir, excludes) {
            continue;
        }
        files.push(path.to_path_buf());
    }

    files
}

/// Load exclude patterns applied to CB-9xx Markdown checks. Merges entries
/// from `.pmat-gates.toml [exclude] paths`, `.pmat-gates.toml [file_health]
/// exclude`, and `.pmat.yaml comply.thresholds.file_health_exclude`.
fn load_markdown_excludes(dir: &Path) -> Vec<String> {
    let mut out: Vec<String> = Vec::new();

    let gates = dir.join(".pmat-gates.toml");
    if let Ok(content) = fs::read_to_string(&gates) {
        if let Ok(table) = content.parse::<toml::Table>() {
            push_str_array(&mut out, table.get("exclude").and_then(|e| e.get("paths")));
            push_str_array(
                &mut out,
                table.get("file_health").and_then(|fh| fh.get("exclude")),
            );
        }
    }

    if let Ok(cfg) = crate::models::comply_config::PmatYamlConfig::load(dir) {
        for pat in &cfg.comply.thresholds.file_health_exclude {
            if !out.iter().any(|p| p == pat) {
                out.push(pat.clone());
            }
        }
    }

    out
}

fn push_str_array(out: &mut Vec<String>, v: Option<&toml::Value>) {
    let Some(arr) = v.and_then(|x| x.as_array()) else {
        return;
    };
    for item in arr {
        if let Some(s) = item.as_str() {
            let s = s.to_string();
            if !out.iter().any(|p| p == &s) {
                out.push(s);
            }
        }
    }
}

fn path_matches_any_exclude(path: &Path, root: &Path, patterns: &[String]) -> bool {
    if patterns.is_empty() {
        return false;
    }
    let rel = path.strip_prefix(root).unwrap_or(path);
    let rel_str = rel.to_string_lossy();
    let file_name = path.file_name().and_then(|n| n.to_str()).unwrap_or("");
    for pattern in patterns {
        if glob_like_match(&rel_str, file_name, pattern) {
            return true;
        }
    }
    false
}

fn glob_like_match(path_str: &str, file_name: &str, pattern: &str) -> bool {
    if let Some(suffix) = pattern.strip_prefix("**/") {
        if suffix.ends_with("/**") {
            let segment = suffix.trim_end_matches("/**");
            return path_str.contains(segment);
        }
        if suffix.contains('*') {
            return glob::Pattern::new(suffix)
                .map(|p| p.matches(file_name))
                .unwrap_or(false);
        }
        return file_name == suffix || path_str.contains(suffix);
    }
    if let Some(prefix) = pattern.strip_suffix("/**") {
        return path_str.starts_with(prefix) || path_str.contains(&format!("/{prefix}/"));
    }
    if pattern.contains('/') {
        return path_str.contains(pattern);
    }
    if pattern.contains('*') {
        return glob::Pattern::new(pattern)
            .map(|p| p.matches(file_name))
            .unwrap_or(false);
    }
    file_name == pattern
}

// =============================================================================
// CB-900: Internal link validation
// =============================================================================

#[provable_contracts_macros::contract("pmat-core.yaml", equation = "path_exists")]
/// Detect cb900 broken internal link.
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

#[provable_contracts_macros::contract("pmat-core.yaml", equation = "path_exists")]
/// Detect cb901 heading hierarchy skip.
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

#[provable_contracts_macros::contract("pmat-core.yaml", equation = "path_exists")]
/// Detect cb902 missing alt text.
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

#[provable_contracts_macros::contract("pmat-core.yaml", equation = "path_exists")]
/// Detect cb903 bare url.
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

#[provable_contracts_macros::contract("pmat-core.yaml", equation = "path_exists")]
/// Detect cb904 long line.
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

#[cfg(test)]
mod markdown_best_practices_tests {
    //! Covers pure helpers in markdown_best_practices.rs (487 lines, 0
    //! prior tests). Skips `walkdir_*` and the CB-9xx detect_* fns that
    //! traverse the filesystem (those need tempdir fixtures + roadmap
    //! integration; covered indirectly via cb_detect E2E).
    use super::*;

    // ── push_str_array ──

    #[test]
    fn test_push_str_array_appends_string_array_items() {
        let mut out: Vec<String> = vec![];
        let toml_val: toml::Value = "x = [\"a\", \"b\", \"c\"]"
            .parse::<toml::Table>()
            .unwrap()
            .get("x")
            .unwrap()
            .clone();
        push_str_array(&mut out, Some(&toml_val));
        assert_eq!(out, vec!["a", "b", "c"]);
    }

    #[test]
    fn test_push_str_array_dedupes_existing_entries() {
        let mut out: Vec<String> = vec!["a".to_string()];
        let toml_val: toml::Value = "x = [\"a\", \"b\"]"
            .parse::<toml::Table>()
            .unwrap()
            .get("x")
            .unwrap()
            .clone();
        push_str_array(&mut out, Some(&toml_val));
        assert_eq!(out, vec!["a", "b"]);
    }

    #[test]
    fn test_push_str_array_skips_non_array_or_none() {
        let mut out: Vec<String> = vec![];
        push_str_array(&mut out, None);
        assert!(out.is_empty());
        let scalar: toml::Value = "x = 42"
            .parse::<toml::Table>()
            .unwrap()
            .get("x")
            .unwrap()
            .clone();
        push_str_array(&mut out, Some(&scalar));
        assert!(out.is_empty());
    }

    #[test]
    fn test_push_str_array_skips_non_string_array_items() {
        let mut out: Vec<String> = vec![];
        let mixed: toml::Value = "x = [\"a\", 42, true]"
            .parse::<toml::Table>()
            .unwrap()
            .get("x")
            .unwrap()
            .clone();
        push_str_array(&mut out, Some(&mixed));
        // Only the string "a" is pushed.
        assert_eq!(out, vec!["a"]);
    }

    // ── glob_like_match ──

    #[test]
    fn test_glob_like_match_double_star_prefix_with_double_star_suffix() {
        // `**/foo/**` → contains "foo".
        assert!(glob_like_match("a/foo/b", "b", "**/foo/**"));
        assert!(!glob_like_match("a/bar/b", "b", "**/foo/**"));
    }

    #[test]
    fn test_glob_like_match_double_star_prefix_glob_suffix() {
        // `**/*.md` → match basename glob.
        assert!(glob_like_match("docs/foo.md", "foo.md", "**/*.md"));
        assert!(!glob_like_match("docs/foo.txt", "foo.txt", "**/*.md"));
    }

    #[test]
    fn test_glob_like_match_double_star_prefix_literal_suffix() {
        // `**/exact-name.md` → exact filename or substring.
        assert!(glob_like_match(
            "docs/sub/exact-name.md",
            "exact-name.md",
            "**/exact-name.md"
        ));
        assert!(!glob_like_match(
            "docs/other.md",
            "other.md",
            "**/exact-name.md"
        ));
    }

    #[test]
    fn test_glob_like_match_double_star_suffix_only() {
        // `prefix/**` → starts with "prefix" or path contains "/prefix/".
        assert!(glob_like_match("prefix/foo.md", "foo.md", "prefix/**"));
        assert!(glob_like_match(
            "outer/prefix/foo.md",
            "foo.md",
            "prefix/**"
        ));
        assert!(!glob_like_match("other/foo.md", "foo.md", "prefix/**"));
    }

    #[test]
    fn test_glob_like_match_path_with_slash_substring() {
        // Pattern with '/' but no '**' wildcards → substring match.
        assert!(glob_like_match("docs/foo/bar.md", "bar.md", "foo/bar"));
        assert!(!glob_like_match("docs/qux.md", "qux.md", "foo/bar"));
    }

    #[test]
    fn test_glob_like_match_basename_glob_pattern() {
        // No '/', has '*' → glob match against file_name only.
        assert!(glob_like_match("any/path/foo.md", "foo.md", "*.md"));
        assert!(!glob_like_match("any/path/foo.rs", "foo.rs", "*.md"));
    }

    #[test]
    fn test_glob_like_match_exact_filename() {
        // Plain filename → exact match against file_name.
        assert!(glob_like_match(
            "a/CHANGELOG.md",
            "CHANGELOG.md",
            "CHANGELOG.md"
        ));
        assert!(!glob_like_match(
            "a/CHANGES.md",
            "CHANGES.md",
            "CHANGELOG.md"
        ));
    }

    // ── path_matches_any_exclude ──

    #[test]
    fn test_path_matches_any_exclude_empty_patterns_returns_false() {
        let p = Path::new("/root/docs/foo.md");
        let root = Path::new("/root");
        assert!(!path_matches_any_exclude(p, root, &[]));
    }

    #[test]
    fn test_path_matches_any_exclude_first_match_short_circuits() {
        let p = Path::new("/root/docs/foo.md");
        let root = Path::new("/root");
        let patterns = vec!["**/*.md".to_string(), "never-matches".to_string()];
        assert!(path_matches_any_exclude(p, root, &patterns));
    }

    #[test]
    fn test_path_matches_any_exclude_no_match_returns_false() {
        let p = Path::new("/root/docs/foo.md");
        let root = Path::new("/root");
        let patterns = vec!["**/*.txt".to_string(), "**/*.json".to_string()];
        assert!(!path_matches_any_exclude(p, root, &patterns));
    }
}

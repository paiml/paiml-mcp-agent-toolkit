#![cfg_attr(coverage_nightly, coverage(off))]
// Changelog manager for workflow integration (Issue #75 Phase 7)
//
// Manages automatic CHANGELOG.md updates based on work item labels and types.

use anyhow::{Context, Result};
use std::fs;
use std::path::PathBuf;

/// Changelog category based on work item labels
#[derive(Debug, Clone, PartialEq)]
pub enum ChangeCategory {
    Added,
    Changed,
    Deprecated,
    Removed,
    Fixed,
    Security,
}

impl ChangeCategory {
    /// Infer category from GitHub labels
    pub fn from_labels(labels: &[String]) -> Option<Self> {
        for label in labels {
            let lower = label.to_lowercase();
            if lower.contains("feature") || lower.contains("enhancement") {
                return Some(ChangeCategory::Added);
            }
            if lower.contains("bug") || lower.contains("fix") {
                return Some(ChangeCategory::Fixed);
            }
            if lower.contains("security") {
                return Some(ChangeCategory::Security);
            }
            if lower.contains("breaking") || lower.contains("change") {
                return Some(ChangeCategory::Changed);
            }
            if lower.contains("deprecat") {
                return Some(ChangeCategory::Deprecated);
            }
            if lower.contains("removal") {
                return Some(ChangeCategory::Removed);
            }
        }
        None
    }

    /// Get section header for changelog
    pub fn section_header(&self) -> &'static str {
        match self {
            ChangeCategory::Added => "### Added",
            ChangeCategory::Changed => "### Changed",
            ChangeCategory::Deprecated => "### Deprecated",
            ChangeCategory::Removed => "### Removed",
            ChangeCategory::Fixed => "### Fixed",
            ChangeCategory::Security => "### Security",
        }
    }
}

/// Changelog entry
#[derive(Debug, Clone)]
pub struct ChangelogEntry {
    pub category: ChangeCategory,
    pub description: String,
    pub issue_number: Option<u64>,
}

impl ChangelogEntry {
    /// Create new changelog entry
    pub fn new(category: ChangeCategory, description: String, issue_number: Option<u64>) -> Self {
        Self {
            category,
            description,
            issue_number,
        }
    }

    /// Format entry as markdown line
    pub fn to_markdown(&self) -> String {
        if let Some(issue) = self.issue_number {
            format!("- {} (#{})", self.description, issue)
        } else {
            format!("- {}", self.description)
        }
    }
}

/// Add entry to CHANGELOG.md
pub fn add_to_changelog(project_path: &PathBuf, entry: ChangelogEntry) -> Result<()> {
    debug_assert!(
        project_path.exists(),
        "project_path must exist: {}",
        project_path.display()
    );
    let changelog_path = project_path.join("CHANGELOG.md");

    // Create CHANGELOG.md if it doesn't exist
    if !changelog_path.exists() {
        create_changelog(&changelog_path)?;
    }

    let content = fs::read_to_string(&changelog_path).context("Failed to read CHANGELOG.md")?;

    // Find or create Unreleased section
    let updated_content = insert_entry(&content, &entry)?;

    fs::write(&changelog_path, updated_content).context("Failed to write CHANGELOG.md")?;

    Ok(())
}

/// Create new CHANGELOG.md with standard structure
fn create_changelog(path: &PathBuf) -> Result<()> {
    debug_assert!(path.exists(), "path must exist: {}", path.display());
    let template = r#"# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added

### Changed

### Deprecated

### Removed

### Fixed

### Security
"#;

    fs::write(path, template)?;
    Ok(())
}

/// Check if a line is the Unreleased section header
fn is_unreleased_header(line: &str) -> bool {
    line.starts_with("## [Unreleased]")
}

/// Check if a line is a versioned section header (e.g., "## [1.0.0]")
fn is_version_header(line: &str) -> bool {
    line.starts_with("## [") && !is_unreleased_header(line)
}

/// Check if a line is a section boundary (subsection or version header)
fn is_section_boundary(line: &str) -> bool {
    line.starts_with("### ") || line.starts_with("## ")
}

/// Insert entry into changelog content
fn insert_entry(content: &str, entry: &ChangelogEntry) -> Result<String> {
    let lines: Vec<&str> = content.lines().collect();
    let mut result = Vec::new();
    let mut in_unreleased = false;
    let mut in_target_section = false;
    let mut inserted = false;

    let section_header = entry.category.section_header();

    for line in lines.iter() {
        // Entering Unreleased section
        if is_unreleased_header(line) {
            in_unreleased = true;
            result.push(line.to_string());
            continue;
        }

        // Leaving Unreleased section (hit a versioned header)
        if in_unreleased && is_version_header(line) {
            in_unreleased = false;
        }

        // Entering the target subsection within Unreleased
        if in_unreleased && line.starts_with(section_header) {
            in_target_section = true;
            result.push(line.to_string());
            continue;
        }

        // Leaving the target subsection (hit next section boundary)
        if in_target_section && is_section_boundary(line) {
            in_target_section = false;
        }

        // Insert entry before first non-empty line in target section
        if in_target_section && !inserted && !line.trim().is_empty() {
            result.push(entry.to_markdown());
            inserted = true;
        }

        result.push(line.to_string());
    }

    // If we didn't insert (empty section), add at the end of section
    if !inserted {
        // This shouldn't happen with the template, but handle it anyway
        result.push(entry.to_markdown());
    }

    Ok(result.join("\n"))
}

#[cfg_attr(coverage_nightly, coverage(off))]
#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_category_from_labels() {
        assert_eq!(
            ChangeCategory::from_labels(&["feature".to_string()]),
            Some(ChangeCategory::Added)
        );
        assert_eq!(
            ChangeCategory::from_labels(&["bug".to_string()]),
            Some(ChangeCategory::Fixed)
        );
        assert_eq!(
            ChangeCategory::from_labels(&["security".to_string()]),
            Some(ChangeCategory::Security)
        );
    }

    #[test]
    fn test_entry_to_markdown() {
        let entry =
            ChangelogEntry::new(ChangeCategory::Added, "New feature".to_string(), Some(123));
        assert_eq!(entry.to_markdown(), "- New feature (#123)");

        let entry2 = ChangelogEntry::new(ChangeCategory::Fixed, "Bug fix".to_string(), None);
        assert_eq!(entry2.to_markdown(), "- Bug fix");
    }

    #[test]
    fn test_create_changelog() {
        let temp = TempDir::new().unwrap();
        let path = temp.path().join("CHANGELOG.md");

        create_changelog(&path).unwrap();
        assert!(path.exists());

        let content = fs::read_to_string(&path).unwrap();
        assert!(content.contains("## [Unreleased]"));
        assert!(content.contains("### Added"));
        assert!(content.contains("### Fixed"));
    }

    #[test]
    fn test_add_to_changelog() {
        let temp = TempDir::new().unwrap();
        let project_path = temp.path().to_path_buf();

        let entry =
            ChangelogEntry::new(ChangeCategory::Added, "Test feature".to_string(), Some(42));

        add_to_changelog(&project_path, entry.clone()).unwrap();

        let changelog_path = project_path.join("CHANGELOG.md");
        assert!(changelog_path.exists());

        let content = fs::read_to_string(&changelog_path).unwrap();
        assert!(content.contains("- Test feature (#42)"));
    }

    #[test]
    fn test_insert_entry() {
        let content = r#"# Changelog

## [Unreleased]

### Added

### Fixed
"#;

        let entry = ChangelogEntry::new(ChangeCategory::Added, "New feature".to_string(), Some(10));

        let result = insert_entry(content, &entry).unwrap();
        assert!(result.contains("- New feature (#10)"));
    }

    #[test]
    fn test_multiple_entries_same_section() {
        let temp = TempDir::new().unwrap();
        let project_path = temp.path().to_path_buf();

        let entry1 = ChangelogEntry::new(ChangeCategory::Fixed, "Fix bug 1".to_string(), Some(1));
        let entry2 = ChangelogEntry::new(ChangeCategory::Fixed, "Fix bug 2".to_string(), Some(2));

        add_to_changelog(&project_path, entry1).unwrap();
        add_to_changelog(&project_path, entry2).unwrap();

        let content = fs::read_to_string(project_path.join("CHANGELOG.md")).unwrap();
        assert!(content.contains("- Fix bug 1 (#1)"));
        assert!(content.contains("- Fix bug 2 (#2)"));
    }
}

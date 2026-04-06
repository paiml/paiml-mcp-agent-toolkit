#![cfg_attr(coverage_nightly, coverage(off))]
//! Scorer trait for Popper Falsifiability Score v1.1
//!
//! Defines the common interface for all 6 scoring category analyzers.
//! Each scorer analyzes a project and returns a PopperCategoryScore.

use super::models::PopperCategoryScore;
use std::path::{Path, PathBuf};

/// Result type for scoring operations
pub type PopperScorerResult<T> = Result<T, PopperScorerError>;

/// Errors that can occur during scoring
#[derive(Debug, Clone, thiserror::Error)]
pub enum PopperScorerError {
    #[error("Failed to read file: {0}")]
    FileReadError(String),

    #[error("Failed to parse content: {0}")]
    ParseError(String),

    #[error("Tool not found: {0}")]
    ToolNotFound(String),

    #[error("Invalid project structure: {0}")]
    InvalidProject(String),

    #[error("IO error: {0}")]
    IoError(String),

    #[error("Command execution failed: {0}")]
    CommandError(String),
}

impl From<std::io::Error> for PopperScorerError {
    fn from(e: std::io::Error) -> Self {
        PopperScorerError::IoError(e.to_string())
    }
}

/// Workspace detection utilities for Rust projects
///
/// Helps scorers properly analyze Cargo workspaces where code/tests
/// live in member directories but docs/CI/LICENSE live at root.
pub mod workspace {
    use super::*;
    use regex::Regex;

    /// Information about a Cargo workspace
    #[derive(Debug, Clone)]
    pub struct WorkspaceInfo {
        /// Whether this is a workspace root
        pub is_workspace: bool,
        /// Workspace member paths (absolute)
        pub members: Vec<PathBuf>,
    }

    /// Detect if a project is a Cargo workspace and return member paths
    ///
    /// # Arguments
    /// * `project_path` - Path to the project root
    ///
    /// # Returns
    /// * `WorkspaceInfo` - Information about the workspace
    pub fn detect_workspace(project_path: &Path) -> WorkspaceInfo {
        debug_assert!(
            project_path.exists(),
            "project_path must exist: {}",
            project_path.display()
        );
        let cargo_path = project_path.join("Cargo.toml");

        if !cargo_path.exists() {
            return WorkspaceInfo {
                is_workspace: false,
                members: vec![project_path.to_path_buf()],
            };
        }

        if let Ok(content) = std::fs::read_to_string(&cargo_path) {
            if content.contains("[workspace]") {
                // Parse workspace members
                let members = parse_workspace_members(&content, project_path);
                if !members.is_empty() {
                    return WorkspaceInfo {
                        is_workspace: true,
                        members,
                    };
                }
            }
        }

        // Not a workspace - treat as single-crate project
        WorkspaceInfo {
            is_workspace: false,
            members: vec![project_path.to_path_buf()],
        }
    }

    /// Parse workspace members from Cargo.toml content
    fn parse_workspace_members(content: &str, project_path: &Path) -> Vec<PathBuf> {
        let mut members = Vec::new();

        // Match members = ["member1", "member2"] or members = [\n  "member1",\n  "member2"\n]
        let members_regex = Regex::new(r#"members\s*=\s*\[([\s\S]*?)\]"#).ok();

        if let Some(re) = members_regex {
            if let Some(captures) = re.captures(content) {
                if let Some(members_str) = captures.get(1) {
                    // Extract quoted strings
                    let quote_regex = Regex::new(r#""([^"]+)""#).ok();
                    if let Some(qre) = quote_regex {
                        for cap in qre.captures_iter(members_str.as_str()) {
                            if let Some(member) = cap.get(1) {
                                let member_path = project_path.join(member.as_str());
                                if member_path.exists() {
                                    members.push(member_path);
                                }
                            }
                        }
                    }
                }
            }
        }

        members
    }

    /// Get all paths to check for code-related files (tests, src, benches)
    ///
    /// For workspaces: returns all member paths
    /// For single crates: returns just the project path
    pub fn get_code_paths(project_path: &Path) -> Vec<PathBuf> {
        debug_assert!(
            project_path.exists(),
            "project_path must exist: {}",
            project_path.display()
        );
        let info = detect_workspace(project_path);
        info.members
    }

    /// Check if any workspace member has a specific directory (e.g., "tests", "benches")
    pub fn any_member_has_dir(project_path: &Path, dir_name: &str) -> bool {
        debug_assert!(
            project_path.exists(),
            "project_path must exist: {}",
            project_path.display()
        );
        for member_path in get_code_paths(project_path) {
            if member_path.join(dir_name).exists() {
                return true;
            }
        }
        false
    }

    /// Check if any workspace member has a file matching a pattern
    pub fn any_member_has_file(project_path: &Path, file_name: &str) -> bool {
        debug_assert!(
            project_path.exists(),
            "project_path must exist: {}",
            project_path.display()
        );
        for member_path in get_code_paths(project_path) {
            if member_path.join(file_name).exists() {
                return true;
            }
        }
        false
    }

    /// Read content from a specific directory across all workspace members
    pub fn read_member_dir_content(project_path: &Path, dir_name: &str, extension: &str) -> String {
        debug_assert!(
            project_path.exists(),
            "project_path must exist: {}",
            project_path.display()
        );
        let mut content = String::new();

        for member_path in get_code_paths(project_path) {
            let dir_path = member_path.join(dir_name);
            if dir_path.exists() {
                read_dir_recursive(&dir_path, extension, &mut content);
            }
        }

        content
    }

    /// Recursively read files with given extension from directory
    fn read_dir_recursive(dir: &Path, extension: &str, content: &mut String) {
        if let Ok(entries) = std::fs::read_dir(dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_dir() {
                    read_dir_recursive(&path, extension, content);
                } else if path.extension().is_some_and(|e| e == extension) {
                    if let Ok(file_content) = std::fs::read_to_string(&path) {
                        content.push_str(&file_content);
                        content.push('\n');
                    }
                }
            }
        }
    }
}

/// Common trait for all Popper scoring category analyzers
///
/// Each scorer implements this trait to analyze a specific category
/// and return a score with recommendations.
pub trait PopperScorer: Send + Sync {
    /// Name of this scoring category
    fn name(&self) -> &str;

    /// Category identifier (A-F)
    fn category_id(&self) -> char;

    /// Maximum possible points for this category
    fn max_points(&self) -> f64;

    /// Whether this category is the gateway (Category A)
    fn is_gateway(&self) -> bool {
        self.category_id() == 'A'
    }

    /// Analyze a project and return the score for this category
    ///
    /// # Arguments
    /// * `project_path` - Path to the root of the project
    ///
    /// # Returns
    /// * `PopperScorerResult<PopperCategoryScore>` - The score earned with sub-scores
    fn score(&self, project_path: &Path) -> PopperScorerResult<PopperCategoryScore>;
}

#[cfg_attr(coverage_nightly, coverage(off))]
#[cfg(test)]
mod tests {
    use super::*;

    struct MockScorer {
        name: String,
        category: char,
        max: f64,
    }

    impl PopperScorer for MockScorer {
        fn name(&self) -> &str {
            &self.name
        }

        fn category_id(&self) -> char {
            self.category
        }

        fn max_points(&self) -> f64 {
            self.max
        }

        fn score(&self, _project_path: &Path) -> PopperScorerResult<PopperCategoryScore> {
            Ok(PopperCategoryScore::new(&self.name, 10.0, self.max))
        }
    }

    #[test]
    fn test_gateway_detection() {
        let gateway = MockScorer {
            name: "Falsifiability".to_string(),
            category: 'A',
            max: 25.0,
        };
        assert!(gateway.is_gateway());

        let non_gateway = MockScorer {
            name: "Reproducibility".to_string(),
            category: 'B',
            max: 25.0,
        };
        assert!(!non_gateway.is_gateway());
    }

    #[test]
    fn test_scorer_interface() {
        let scorer = MockScorer {
            name: "Test".to_string(),
            category: 'T',
            max: 10.0,
        };

        assert_eq!(scorer.name(), "Test");
        assert_eq!(scorer.category_id(), 'T');
        assert_eq!(scorer.max_points(), 10.0);
    }
}

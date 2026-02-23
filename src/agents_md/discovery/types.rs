#![cfg_attr(coverage_nightly, coverage(off))]
//! Types for AGENTS.md Discovery System

use super::super::AgentsMdDocument;
use std::collections::HashMap;
use std::path::PathBuf;
use std::time::SystemTime;

/// Discovery configuration
#[derive(Debug, Clone)]
pub struct DiscoveryConfig {
    /// File name to search for
    pub file_name: String,

    /// Maximum depth to search
    pub max_depth: usize,

    /// Enable file watching
    pub watch_enabled: bool,

    /// Cache TTL in seconds
    pub cache_ttl: u64,

    /// Ignore patterns
    pub ignore_patterns: Vec<String>,
}

impl Default for DiscoveryConfig {
    fn default() -> Self {
        Self {
            file_name: "AGENTS.md".to_string(),
            max_depth: 10,
            watch_enabled: false,
            cache_ttl: 300, // 5 minutes
            ignore_patterns: vec![
                ".git".to_string(),
                "node_modules".to_string(),
                "target".to_string(),
                ".venv".to_string(),
            ],
        }
    }
}

/// Discovered AGENTS.md file
#[derive(Debug, Clone)]
pub struct AgentsMdFile {
    /// File path
    pub path: PathBuf,

    /// Parent directory
    pub parent: PathBuf,

    /// Depth from root
    pub depth: usize,

    /// Last modified time
    pub modified: SystemTime,

    /// File content (cached)
    pub content: Option<String>,

    /// Parsed document (cached)
    pub document: Option<AgentsMdDocument>,
}

/// Hierarchy of AGENTS.md files
#[derive(Debug, Clone)]
pub struct AgentsMdHierarchy {
    /// Root directory
    pub root: PathBuf,

    /// All discovered files
    pub files: Vec<AgentsMdFile>,

    /// Hierarchy tree
    pub tree: HierarchyNode,
}

/// Node in hierarchy tree
#[derive(Debug, Clone)]
pub struct HierarchyNode {
    /// Directory path
    pub path: PathBuf,

    /// AGENTS.md file in this directory
    pub agents_file: Option<AgentsMdFile>,

    /// Child directories
    pub children: HashMap<String, HierarchyNode>,
}

/// File change event
#[derive(Debug, Clone)]
pub struct FileChange {
    /// File path
    pub path: PathBuf,

    /// Type of change
    pub change_type: FileChangeType,

    /// Timestamp
    pub timestamp: SystemTime,
}

/// Type of file change
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FileChangeType {
    Created,
    Modified,
    Removed,
}

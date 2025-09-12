//! AGENTS.md Discovery System
//!
//! Discovers and monitors AGENTS.md files in project hierarchies with caching.

use super::*;
use crate::utils::path_validator::PathValidator;
use anyhow::Result;
use dashmap::DashMap;
use notify::{Event as NotifyEvent, EventKind, RecommendedWatcher};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::SystemTime;
use tokio::sync::mpsc;

/// Discovery system for AGENTS.md files
pub struct AgentsMdDiscovery {
    /// Cache of discovered files
    cache: Arc<DashMap<PathBuf, AgentsMdFile>>,

    /// File watcher
    watcher: Option<RecommendedWatcher>,

    /// Configuration
    config: DiscoveryConfig,
}

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

impl Default for AgentsMdDiscovery {
    fn default() -> Self {
        Self::new()
    }
}

impl AgentsMdDiscovery {
    /// Create new discovery system
    pub fn new() -> Self {
        Self::with_config(DiscoveryConfig::default())
    }

    /// Create with custom configuration
    pub fn with_config(config: DiscoveryConfig) -> Self {
        Self {
            cache: Arc::new(DashMap::new()),
            watcher: None,
            config,
        }
    }

    /// Find nearest AGENTS.md file from path
    pub fn find_nearest(&self, path: &Path) -> Option<PathBuf> {
        // Check cache first
        if let Some(cached) = self.get_from_cache(path) {
            return Some(cached.path);
        }

        // Start from the given path and traverse up
        let mut current = PathValidator::get_valid_parent(path).ok()?;

        let mut depth = 0;

        loop {
            let agents_path = current.join(&self.config.file_name);

            if PathValidator::ensure_file(&agents_path).is_ok() {
                // Cache the discovery
                self.cache_file(&agents_path, depth);
                return Some(agents_path);
            }

            // Move up to parent
            current = current.parent()?;
            depth += 1;

            // Check depth limit
            if depth > self.config.max_depth {
                break;
            }
        }

        None
    }

    /// Discover all AGENTS.md files in project
    pub fn discover_all(&self, root: &Path) -> Vec<AgentsMdFile> {
        let mut files = Vec::new();
        self.discover_recursive(root, 0, &mut files);

        // Sort by depth (nearest first)
        files.sort_by_key(|f| f.depth);

        files
    }

    /// Build hierarchy for monorepo
    pub fn build_hierarchy(&self, files: Vec<AgentsMdFile>) -> AgentsMdHierarchy {
        if files.is_empty() {
            return AgentsMdHierarchy {
                root: PathBuf::new(),
                files: Vec::new(),
                tree: HierarchyNode {
                    path: PathBuf::new(),
                    agents_file: None,
                    children: HashMap::new(),
                },
            };
        }

        // Find common root
        let root = self.find_common_root(&files);

        // Build tree
        let mut tree = HierarchyNode {
            path: root.clone(),
            agents_file: None,
            children: HashMap::new(),
        };

        for file in &files {
            self.insert_into_tree(&mut tree, file);
        }

        AgentsMdHierarchy { root, files, tree }
    }

    /// Start watching for changes
    pub async fn start_watching(&mut self) -> Result<mpsc::Receiver<FileChange>> {
        let (tx, rx) = mpsc::channel(100);

        let cache = self.cache.clone();
        let config = self.config.clone();

        let watcher =
            notify::recommended_watcher(move |event: Result<NotifyEvent, notify::Error>| {
                if let Ok(event) = event {
                    match event.kind {
                        EventKind::Create(_) | EventKind::Modify(_) | EventKind::Remove(_) => {
                            for path in &event.paths {
                                if path.file_name() == Some(std::ffi::OsStr::new(&config.file_name))
                                {
                                    let change = match event.kind {
                                        EventKind::Create(_) => FileChangeType::Created,
                                        EventKind::Modify(_) => FileChangeType::Modified,
                                        EventKind::Remove(_) => FileChangeType::Removed,
                                        _ => continue,
                                    };

                                    // Update cache
                                    match change {
                                        FileChangeType::Removed => {
                                            cache.remove(path);
                                        }
                                        _ => {
                                            // Re-cache the file
                                            if let Ok(metadata) = std::fs::metadata(path) {
                                                if let Ok(modified) = metadata.modified() {
                                                    cache.insert(
                                                        path.clone(),
                                                        AgentsMdFile {
                                                            path: path.clone(),
                                                            parent: path
                                                                .parent()
                                                                .unwrap_or(path)
                                                                .to_path_buf(),
                                                            depth: 0,
                                                            modified,
                                                            content: None,
                                                            document: None,
                                                        },
                                                    );
                                                }
                                            }
                                        }
                                    }

                                    // Send change notification
                                    let _ = tx.blocking_send(FileChange {
                                        path: path.clone(),
                                        change_type: change,
                                        timestamp: SystemTime::now(),
                                    });
                                }
                            }
                        }
                        _ => {}
                    }
                }
            })?;

        self.watcher = Some(watcher);

        Ok(rx)
    }

    /// Stop watching for changes
    pub fn stop_watching(&mut self) {
        self.watcher = None;
    }

    /// Clear cache
    pub fn clear_cache(&self) {
        self.cache.clear();
    }

    /// Get from cache if valid
    fn get_from_cache(&self, path: &Path) -> Option<AgentsMdFile> {
        self.cache.get(path).map(|entry| entry.clone())
    }

    /// Cache a discovered file
    fn cache_file(&self, path: &Path, depth: usize) {
        if let Ok(metadata) = std::fs::metadata(path) {
            if let Ok(modified) = metadata.modified() {
                self.cache.insert(
                    path.to_path_buf(),
                    AgentsMdFile {
                        path: path.to_path_buf(),
                        parent: path.parent().unwrap_or(path).to_path_buf(),
                        depth,
                        modified,
                        content: None,
                        document: None,
                    },
                );
            }
        }
    }

    /// Recursive discovery
    fn discover_recursive(&self, dir: &Path, depth: usize, files: &mut Vec<AgentsMdFile>) {
        if depth > self.config.max_depth {
            return;
        }

        // Check if this directory should be ignored
        if let Some(dir_name) = dir.file_name() {
            if let Some(name_str) = dir_name.to_str() {
                if self
                    .config
                    .ignore_patterns
                    .iter()
                    .any(|pattern| name_str == pattern)
                {
                    return;
                }
            }
        }

        // Check for AGENTS.md in this directory
        let agents_path = dir.join(&self.config.file_name);
        if PathValidator::ensure_file(&agents_path).is_ok() {
            if let Ok(metadata) = std::fs::metadata(&agents_path) {
                if let Ok(modified) = metadata.modified() {
                    let file = AgentsMdFile {
                        path: agents_path.clone(),
                        parent: dir.to_path_buf(),
                        depth,
                        modified,
                        content: None,
                        document: None,
                    };

                    files.push(file.clone());
                    self.cache.insert(agents_path, file);
                }
            }
        }

        // Recurse into subdirectories
        if let Ok(entries) = std::fs::read_dir(dir) {
            for entry in entries.flatten() {
                if let Ok(file_type) = entry.file_type() {
                    if file_type.is_dir() {
                        self.discover_recursive(&entry.path(), depth + 1, files);
                    }
                }
            }
        }
    }

    /// Find common root of files
    fn find_common_root(&self, files: &[AgentsMdFile]) -> PathBuf {
        if files.is_empty() {
            return PathBuf::new();
        }

        let mut common = files[0].parent.clone();

        for file in files.iter().skip(1) {
            while !file.parent.starts_with(&common) {
                if let Some(parent) = common.parent() {
                    common = parent.to_path_buf();
                } else {
                    return PathBuf::from("/");
                }
            }
        }

        common
    }

    /// Insert file into hierarchy tree
    #[allow(clippy::only_used_in_recursion)]
    fn insert_into_tree(&self, node: &mut HierarchyNode, file: &AgentsMdFile) {
        if file.parent == node.path {
            node.agents_file = Some(file.clone());
            return;
        }

        // Find relative path
        if let Ok(relative) = file.parent.strip_prefix(&node.path) {
            if let Some(first) = relative.components().next() {
                if let Some(first_str) = first.as_os_str().to_str() {
                    let child_path = node.path.join(first_str);

                    let child = node
                        .children
                        .entry(first_str.to_string())
                        .or_insert_with(|| HierarchyNode {
                            path: child_path,
                            agents_file: None,
                            children: HashMap::new(),
                        });

                    self.insert_into_tree(child, file);
                }
            }
        }
    }
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn test_find_nearest_in_same_directory() {
        let temp_dir = TempDir::new().unwrap();
        let agents_path = temp_dir.path().join("AGENTS.md");
        fs::write(&agents_path, "# Test").unwrap();

        let discovery = AgentsMdDiscovery::new();
        let found = discovery.find_nearest(temp_dir.path());

        assert_eq!(found, Some(agents_path));
    }

    #[test]
    fn test_find_nearest_in_parent() {
        let temp_dir = TempDir::new().unwrap();
        let agents_path = temp_dir.path().join("AGENTS.md");
        fs::write(&agents_path, "# Test").unwrap();

        let subdir = temp_dir.path().join("subdir");
        fs::create_dir(&subdir).unwrap();

        let discovery = AgentsMdDiscovery::new();
        let found = discovery.find_nearest(&subdir);

        assert_eq!(found, Some(agents_path));
    }

    #[test]
    fn test_find_nearest_not_found() {
        let temp_dir = TempDir::new().unwrap();

        let discovery = AgentsMdDiscovery::with_config(DiscoveryConfig {
            max_depth: 1,
            ..Default::default()
        });

        let found = discovery.find_nearest(temp_dir.path());
        assert_eq!(found, None);
    }

    #[test]
    fn test_discover_all() {
        let temp_dir = TempDir::new().unwrap();

        // Create multiple AGENTS.md files
        fs::write(temp_dir.path().join("AGENTS.md"), "# Root").unwrap();

        let sub1 = temp_dir.path().join("sub1");
        fs::create_dir(&sub1).unwrap();
        fs::write(sub1.join("AGENTS.md"), "# Sub1").unwrap();

        let sub2 = temp_dir.path().join("sub2");
        fs::create_dir(&sub2).unwrap();
        fs::write(sub2.join("AGENTS.md"), "# Sub2").unwrap();

        let discovery = AgentsMdDiscovery::new();
        let files = discovery.discover_all(temp_dir.path());

        assert_eq!(files.len(), 3);
        assert_eq!(files[0].depth, 0); // Root file should be first
    }

    #[test]
    fn test_ignore_patterns() {
        let temp_dir = TempDir::new().unwrap();

        // Create AGENTS.md in ignored directory
        let node_modules = temp_dir.path().join("node_modules");
        fs::create_dir(&node_modules).unwrap();
        fs::write(node_modules.join("AGENTS.md"), "# Ignored").unwrap();

        // Create AGENTS.md in allowed directory
        fs::write(temp_dir.path().join("AGENTS.md"), "# Root").unwrap();

        let discovery = AgentsMdDiscovery::new();
        let files = discovery.discover_all(temp_dir.path());

        assert_eq!(files.len(), 1);
        assert!(!files[0].path.to_string_lossy().contains("node_modules"));
    }

    #[test]
    fn test_build_hierarchy() {
        let temp_dir = TempDir::new().unwrap();

        // Create hierarchy
        fs::write(temp_dir.path().join("AGENTS.md"), "# Root").unwrap();

        let sub = temp_dir.path().join("sub");
        fs::create_dir(&sub).unwrap();
        fs::write(sub.join("AGENTS.md"), "# Sub").unwrap();

        let discovery = AgentsMdDiscovery::new();
        let files = discovery.discover_all(temp_dir.path());
        let hierarchy = discovery.build_hierarchy(files);

        assert_eq!(hierarchy.root, temp_dir.path());
        assert_eq!(hierarchy.files.len(), 2);
        assert!(hierarchy.tree.agents_file.is_some());
        assert_eq!(hierarchy.tree.children.len(), 1);
    }

    #[test]
    fn test_cache_operations() {
        let temp_dir = TempDir::new().unwrap();
        let agents_path = temp_dir.path().join("AGENTS.md");
        fs::write(&agents_path, "# Test").unwrap();

        let discovery = AgentsMdDiscovery::new();

        // First call should discover and cache
        let found1 = discovery.find_nearest(temp_dir.path());
        assert_eq!(found1, Some(agents_path.clone()));

        // Second call should use cache
        let found2 = discovery.find_nearest(temp_dir.path());
        assert_eq!(found2, Some(agents_path.clone()));

        // Clear cache
        discovery.clear_cache();

        // Should still find after cache clear
        let found3 = discovery.find_nearest(temp_dir.path());
        assert_eq!(found3, Some(agents_path));
    }

    #[test]
    fn test_depth_limit() {
        let temp_dir = TempDir::new().unwrap();

        // Create deep hierarchy
        let mut current = temp_dir.path().to_path_buf();
        for i in 0..5 {
            current = current.join(format!("level{}", i));
            fs::create_dir(&current).unwrap();
        }

        // Put AGENTS.md at root
        fs::write(temp_dir.path().join("AGENTS.md"), "# Root").unwrap();

        // Discovery with depth limit
        let discovery = AgentsMdDiscovery::with_config(DiscoveryConfig {
            max_depth: 3,
            ..Default::default()
        });

        // Should not find from deep directory
        let found = discovery.find_nearest(&current);
        assert_eq!(found, None);
    }
}

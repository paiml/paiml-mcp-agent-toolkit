#![cfg_attr(coverage_nightly, coverage(off))]
//! Core implementation of AgentsMdDiscovery.

use super::types::{
    AgentsMdDiscovery, AgentsMdFile, AgentsMdHierarchy, DiscoveryConfig, FileChange,
    FileChangeType, HierarchyNode,
};
use crate::utils::path_validator::PathValidator;
use anyhow::Result;
use dashmap::DashMap;
use notify::{Event as NotifyEvent, EventKind};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::SystemTime;
use tokio::sync::mpsc;

fn event_to_change_type(kind: &EventKind) -> Option<FileChangeType> {
    match kind {
        EventKind::Create(_) => Some(FileChangeType::Created),
        EventKind::Modify(_) => Some(FileChangeType::Modified),
        EventKind::Remove(_) => Some(FileChangeType::Removed),
        _ => None,
    }
}

fn update_cache_for_change(
    cache: &DashMap<PathBuf, AgentsMdFile>,
    path: &Path,
    change: &FileChangeType,
) {
    if matches!(change, FileChangeType::Removed) {
        cache.remove(path);
        return;
    }
    if let Ok(metadata) = std::fs::metadata(path) {
        if let Ok(modified) = metadata.modified() {
            cache.insert(
                path.to_path_buf(),
                AgentsMdFile {
                    path: path.to_path_buf(),
                    parent: path.parent().unwrap_or(path).to_path_buf(),
                    depth: 0,
                    modified,
                    content: None,
                    document: None,
                },
            );
        }
    }
}

fn process_watch_event(
    event: &NotifyEvent,
    file_name: &str,
    cache: &DashMap<PathBuf, AgentsMdFile>,
    tx: &mpsc::Sender<FileChange>,
) {
    let Some(change) = event_to_change_type(&event.kind) else {
        return;
    };
    for path in &event.paths {
        if path.file_name() != Some(std::ffi::OsStr::new(file_name)) {
            continue;
        }
        update_cache_for_change(cache, path, &change);
        let _ = tx.blocking_send(FileChange {
            path: path.clone(),
            change_type: change.clone(),
            timestamp: SystemTime::now(),
        });
    }
}

fn should_ignore_dir(dir: &Path, ignore_patterns: &[String]) -> bool {
    dir.file_name()
        .and_then(|n| n.to_str())
        .is_some_and(|name| ignore_patterns.iter().any(|p| name == p))
}

fn try_discover_agents_file(
    dir: &Path,
    file_name: &str,
    depth: usize,
    cache: &DashMap<PathBuf, AgentsMdFile>,
) -> Option<AgentsMdFile> {
    let agents_path = dir.join(file_name);
    if PathValidator::ensure_file(&agents_path).is_err() {
        return None;
    }
    let metadata = std::fs::metadata(&agents_path).ok()?;
    let modified = metadata.modified().ok()?;
    let file = AgentsMdFile {
        path: agents_path.clone(),
        parent: dir.to_path_buf(),
        depth,
        modified,
        content: None,
        document: None,
    };
    cache.insert(agents_path, file.clone());
    Some(file)
}

impl Default for AgentsMdDiscovery {
    fn default() -> Self {
        Self::new()
    }
}

impl AgentsMdDiscovery {
    /// Create new discovery system
    #[must_use]
    pub fn new() -> Self {
        Self::with_config(DiscoveryConfig::default())
    }

    /// Create with custom configuration
    #[must_use]
    pub fn with_config(config: DiscoveryConfig) -> Self {
        Self {
            cache: Arc::new(DashMap::new()),
            watcher: None,
            config,
        }
    }

    /// Find nearest AGENTS.md file from path
    #[must_use]
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
    #[must_use]
    pub fn discover_all(&self, root: &Path) -> Vec<AgentsMdFile> {
        let mut files = Vec::new();
        self.discover_recursive(root, 0, &mut files);

        // Sort by depth (nearest first)
        files.sort_by_key(|f| f.depth);

        files
    }

    /// Build hierarchy for monorepo
    #[must_use]
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
                    process_watch_event(&event, &config.file_name, &cache, &tx);
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
    pub(super) fn get_from_cache(&self, path: &Path) -> Option<AgentsMdFile> {
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
        if should_ignore_dir(dir, &self.config.ignore_patterns) {
            return;
        }
        if let Some(file) =
            try_discover_agents_file(dir, &self.config.file_name, depth, &self.cache)
        {
            files.push(file);
        }
        // Recurse into subdirectories
        let Ok(entries) = std::fs::read_dir(dir) else {
            return;
        };
        for entry in entries.flatten() {
            if entry.file_type().is_some_and(|ft| ft.is_dir()) {
                self.discover_recursive(&entry.path(), depth + 1, files);
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

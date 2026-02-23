#![cfg_attr(coverage_nightly, coverage(off))]
//! Core AgentsMdDiscovery struct and main implementation

use super::types::{AgentsMdFile, AgentsMdHierarchy, DiscoveryConfig, FileChange, HierarchyNode};
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
    pub(super) cache: Arc<DashMap<PathBuf, AgentsMdFile>>,

    /// File watcher
    pub(super) watcher: Option<RecommendedWatcher>,

    /// Configuration
    pub(super) config: DiscoveryConfig,
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
                let Ok(event) = event else { return };
                let is_relevant = matches!(
                    event.kind,
                    EventKind::Create(_) | EventKind::Modify(_) | EventKind::Remove(_)
                );
                if !is_relevant {
                    return;
                }
                for path in &event.paths {
                    if path.file_name() != Some(std::ffi::OsStr::new(&config.file_name)) {
                        continue;
                    }
                    let change = classify_event_kind(event.kind);
                    let Some(change) = change else { continue };
                    update_cache_for_change(&cache, path, change);
                    let _ = tx.blocking_send(FileChange {
                        path: path.clone(),
                        change_type: change,
                        timestamp: SystemTime::now(),
                    });
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
    pub(super) fn cache_file(&self, path: &Path, depth: usize) {
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
    pub(super) fn discover_recursive(&self, dir: &Path, depth: usize, files: &mut Vec<AgentsMdFile>) {
        if depth > self.config.max_depth {
            return;
        }

        if self.is_ignored_dir(dir) {
            return;
        }

        self.try_collect_agents_file(dir, depth, files);

        // Recurse into subdirectories
        let Ok(entries) = std::fs::read_dir(dir) else { return };
        for entry in entries.flatten() {
            let Ok(file_type) = entry.file_type() else { continue };
            if file_type.is_dir() {
                self.discover_recursive(&entry.path(), depth + 1, files);
            }
        }
    }

    fn is_ignored_dir(&self, dir: &Path) -> bool {
        dir.file_name()
            .and_then(|n| n.to_str())
            .map_or(false, |name| self.config.ignore_patterns.iter().any(|p| name == p))
    }

    fn try_collect_agents_file(&self, dir: &Path, depth: usize, files: &mut Vec<AgentsMdFile>) {
        let agents_path = dir.join(&self.config.file_name);
        if PathValidator::ensure_file(&agents_path).is_err() {
            return;
        }
        let Ok(metadata) = std::fs::metadata(&agents_path) else { return };
        let Ok(modified) = metadata.modified() else { return };
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

    /// Find common root of files
    pub(super) fn find_common_root(&self, files: &[AgentsMdFile]) -> PathBuf {
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
    pub(super) fn insert_into_tree(&self, node: &mut HierarchyNode, file: &AgentsMdFile) {
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

fn classify_event_kind(kind: EventKind) -> Option<super::types::FileChangeType> {
    match kind {
        EventKind::Create(_) => Some(super::types::FileChangeType::Created),
        EventKind::Modify(_) => Some(super::types::FileChangeType::Modified),
        EventKind::Remove(_) => Some(super::types::FileChangeType::Removed),
        _ => None,
    }
}

fn update_cache_for_change(
    cache: &DashMap<PathBuf, AgentsMdFile>,
    path: &Path,
    change: super::types::FileChangeType,
) {
    if matches!(change, super::types::FileChangeType::Removed) {
        cache.remove(path);
        return;
    }
    let Ok(metadata) = std::fs::metadata(path) else { return };
    let Ok(modified) = metadata.modified() else { return };
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

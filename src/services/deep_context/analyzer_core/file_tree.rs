#![cfg_attr(coverage_nightly, coverage(off))]

use std::path::PathBuf;

use rustc_hash::FxHashMap;

use crate::models::dag::DependencyGraph;
use crate::services::deep_context::scope::FileScope;
use crate::services::deep_context::DeepContextAnalyzer;
use crate::services::deep_context::{AnnotatedFileTree, AnnotatedNode, NodeAnnotations, NodeType};

impl DeepContextAnalyzer {
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "path_exists")]
    pub(crate) async fn discover_project_structure(
        &self,
        project_path: &PathBuf,
    ) -> anyhow::Result<AnnotatedFileTree> {
        let mut total_files = 0;
        let mut total_size_bytes = 0;

        let root =
            self.build_file_tree_recursive(project_path, &mut total_files, &mut total_size_bytes)?;

        Ok(AnnotatedFileTree {
            root,
            total_files,
            total_size_bytes,
        })
    }

    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "path_exists")]
    pub(crate) fn build_file_tree_recursive(
        &self,
        path: &PathBuf,
        total_files: &mut usize,
        total_size: &mut u64,
    ) -> anyhow::Result<AnnotatedNode> {
        let metadata = std::fs::metadata(path)?;
        let name = path
            .file_name()
            .unwrap_or_default()
            .to_string_lossy()
            .to_string();

        if metadata.is_dir() {
            let children = self.build_child_nodes(path, total_files, total_size);
            Ok(AnnotatedNode {
                name,
                path: path.clone(),
                node_type: NodeType::Directory,
                children,
                annotations: NodeAnnotations::default(),
            })
        } else {
            *total_files += 1;
            *total_size += metadata.len();

            Ok(AnnotatedNode {
                name,
                path: path.clone(),
                node_type: NodeType::File,
                children: Vec::new(),
                annotations: NodeAnnotations::default(),
            })
        }
    }

    /// Build the in-scope children of a directory.
    fn build_child_nodes(
        &self,
        path: &PathBuf,
        total_files: &mut usize,
        total_size: &mut u64,
    ) -> Vec<AnnotatedNode> {
        let scope = self.file_scope();
        let mut children = Vec::new();

        let Ok(entries) = std::fs::read_dir(path) else {
            return children;
        };

        for entry in entries.flatten() {
            let child_path = entry.path();
            if !Self::child_is_in_scope(&scope, &entry, &child_path) {
                continue;
            }
            let Ok(child_node) =
                self.build_file_tree_recursive(&child_path, total_files, total_size)
            else {
                continue;
            };
            // A directory that the include filter emptied is not part of the
            // reported tree.
            if scope.has_include_filter()
                && child_node.node_type == NodeType::Directory
                && child_node.children.is_empty()
            {
                continue;
            }
            children.push(child_node);
        }

        children
    }

    /// One scope predicate for both halves of the rule: exclude patterns prune
    /// directories and files alike, include patterns select files.
    ///
    /// Before R18 only the exclude half was applied here, which is why
    /// `--include-pattern` changed nothing about the report.
    fn child_is_in_scope(
        scope: &FileScope,
        entry: &std::fs::DirEntry,
        child_path: &std::path::Path,
    ) -> bool {
        let is_dir = entry
            .file_type()
            .map_or_else(|_| child_path.is_dir(), |t| t.is_dir());
        if is_dir {
            scope.may_contain_files(child_path)
        } else {
            scope.contains_file(child_path)
        }
    }

    /// The one membership rule this analyzer applies to files (R18).
    ///
    /// Both halves of the rule live in `deep_context::scope::FileScope`; nothing
    /// in `analyzer_core` decides scope for itself.
    pub(crate) fn file_scope(&self) -> FileScope {
        FileScope::from_config(&self.config)
    }

    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "path_exists")]
    pub(crate) fn should_exclude_path(&self, path: &std::path::Path) -> bool {
        self.file_scope().is_excluded(path)
    }

    /// Enrich the file tree with centrality scores from the dependency graph
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub(crate) fn enrich_file_tree_with_centrality(
        &self,
        file_tree: &mut AnnotatedFileTree,
        dag: &DependencyGraph,
    ) -> anyhow::Result<()> {
        // Create a map of file paths to centrality scores
        let mut centrality_map: FxHashMap<PathBuf, f32> = FxHashMap::default();

        for node in dag.nodes.values() {
            if let Some(centrality_str) = node.metadata.get("centrality") {
                if let Ok(centrality) = centrality_str.parse::<f32>() {
                    let file_path = PathBuf::from(&node.file_path);
                    centrality_map.insert(file_path, centrality);
                }
            }
        }

        // Recursively update the file tree with centrality scores
        Self::update_node_centrality(&mut file_tree.root, &centrality_map);

        Ok(())
    }

    /// Recursively update node centrality scores
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "path_exists")]
    pub(crate) fn update_node_centrality(
        node: &mut AnnotatedNode,
        centrality_map: &FxHashMap<PathBuf, f32>,
    ) {
        // Update this node's centrality if it's a file
        if node.node_type == NodeType::File {
            if let Some(&centrality) = centrality_map.get(&node.path) {
                node.annotations.centrality = Some(centrality);
            }
        }

        // Recursively update children
        for child in &mut node.children {
            Self::update_node_centrality(child, centrality_map);
        }
    }

    /// Collect all file paths from the annotated tree
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub(crate) fn collect_file_paths(&self, node: &AnnotatedNode) -> Vec<String> {
        let mut paths = Vec::new();
        Self::collect_paths_recursive(node, &mut paths);
        paths
    }

    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub(crate) fn collect_paths_recursive(node: &AnnotatedNode, paths: &mut Vec<String>) {
        match node.node_type {
            NodeType::File => {
                paths.push(node.path.to_string_lossy().to_string());
            }
            NodeType::Directory => {
                for child in &node.children {
                    Self::collect_paths_recursive(child, paths);
                }
            }
        }
    }
}

#![cfg_attr(coverage_nightly, coverage(off))]

use std::path::PathBuf;

use rustc_hash::FxHashMap;

use crate::models::dag::DependencyGraph;
use crate::services::deep_context::DeepContextAnalyzer;
use crate::services::deep_context::{AnnotatedFileTree, AnnotatedNode, NodeAnnotations, NodeType};

impl DeepContextAnalyzer {
    pub(crate) async fn discover_project_structure(
        &self,
        project_path: &PathBuf,
    ) -> anyhow::Result<AnnotatedFileTree> {
        debug_assert!(
            project_path.exists(),
            "project_path must exist: {}",
            project_path.display()
        );
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

    pub(crate) fn build_file_tree_recursive(
        &self,
        path: &PathBuf,
        total_files: &mut usize,
        total_size: &mut u64,
    ) -> anyhow::Result<AnnotatedNode> {
        debug_assert!(path.exists(), "path must exist: {}", path.display());
        let metadata = std::fs::metadata(path)?;
        let name = path
            .file_name()
            .unwrap_or_default()
            .to_string_lossy()
            .to_string();

        if metadata.is_dir() {
            let mut children = Vec::new();

            if let Ok(entries) = std::fs::read_dir(path) {
                for entry in entries.flatten() {
                    let child_path = entry.path();

                    // Apply exclude patterns
                    if self.should_exclude_path(&child_path) {
                        continue;
                    }

                    if let Ok(child_node) =
                        self.build_file_tree_recursive(&child_path, total_files, total_size)
                    {
                        children.push(child_node);
                    }
                }
            }

            Ok(AnnotatedNode {
                name,
                path: path.clone(),
                node_type: NodeType::Directory,
                children,
                annotations: NodeAnnotations {
                    defect_score: None,
                    complexity_score: None,
                    cognitive_complexity: None,
                    churn_score: None,
                    dead_code_items: 0,
                    satd_items: 0,
                    centrality: None,
                    test_coverage: None,
                    big_o_complexity: None,
                    memory_complexity: None,
                    duplication_score: None,
                },
            })
        } else {
            *total_files += 1;
            *total_size += metadata.len();

            Ok(AnnotatedNode {
                name,
                path: path.clone(),
                node_type: NodeType::File,
                children: Vec::new(),
                annotations: NodeAnnotations {
                    defect_score: None,
                    complexity_score: None,
                    cognitive_complexity: None,
                    churn_score: None,
                    dead_code_items: 0,
                    satd_items: 0,
                    centrality: None,
                    test_coverage: None,
                    big_o_complexity: None,
                    memory_complexity: None,
                    duplication_score: None,
                },
            })
        }
    }

    pub(crate) fn should_exclude_path(&self, path: &std::path::Path) -> bool {
        debug_assert!(path.exists(), "path must exist: {}", path.display());
        let path_str = path.to_string_lossy();

        for pattern in &self.config.exclude_patterns {
            if path_str.contains(pattern.trim_matches('*')) {
                return true;
            }
        }

        false
    }

    /// Enrich the file tree with centrality scores from the dependency graph
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
    pub(crate) fn collect_file_paths(&self, node: &AnnotatedNode) -> Vec<String> {
        let mut paths = Vec::new();
        Self::collect_paths_recursive(node, &mut paths);
        paths
    }

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

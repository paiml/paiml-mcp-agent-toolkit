#![cfg_attr(coverage_nightly, coverage(off))]
//! Cross-language dependency detection and analysis.
//! Detects relationships (inheritance, implementation, usage) across language boundaries.

use crate::ast::polyglot::unified_node::{NodeReference, ReferenceKind};
use crate::ast::polyglot::{Language, NodeKind, UnifiedNode};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};

/// Represents a dependency between nodes in different languages
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CrossLanguageDependency {
    /// The source node ID
    pub source_id: String,
    /// The target node ID
    pub target_id: String,
    /// The source language
    pub source_language: Language,
    /// The target language
    pub target_language: Language,
    /// The type of dependency
    pub kind: ReferenceKind,
    /// Confidence score for the dependency (0.0-1.0)
    pub confidence: f64,
    /// Additional metadata about the dependency
    pub metadata: HashMap<String, String>,
}

/// Detects and manages cross-language dependencies
#[derive(Default)]
pub struct CrossLanguageDependencies {
    /// Map of node IDs to nodes
    nodes: HashMap<String, UnifiedNode>,
    /// Map of fully qualified names to node IDs
    fqn_map: HashMap<String, HashSet<String>>,
    /// Language-specific name resolvers
    name_resolvers: HashMap<Language, Box<dyn NameResolver>>,
    /// Detected cross-language dependencies
    dependencies: Vec<CrossLanguageDependency>,
}

/// Trait for language-specific name resolution
pub trait NameResolver: Send + Sync {
    /// Check if this resolver can resolve a reference
    fn can_resolve(
        &self,
        source_language: Language,
        target_language: Language,
        source: &UnifiedNode,
        reference: &crate::ast::polyglot::unified_node::NodeReference,
        target: &UnifiedNode,
    ) -> bool;
}

impl CrossLanguageDependencies {
    /// Create a new instance
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn new() -> Self {
        Self {
            nodes: HashMap::new(),
            fqn_map: HashMap::new(),
            name_resolvers: HashMap::new(),
            dependencies: Vec::new(),
        }
    }

    /// Add language-specific name resolver
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn add_name_resolver(&mut self, language: Language, resolver: Box<dyn NameResolver>) {
        self.name_resolvers.insert(language, resolver);
    }

    /// Add nodes to the dependency detector
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn add_nodes(&mut self, nodes: Vec<UnifiedNode>) {
        debug_assert!(!nodes.is_empty(), "nodes must not be empty");
        for node in nodes {
            self.fqn_map
                .entry(node.fqn.clone())
                .or_default()
                .insert(node.id.clone());
            self.nodes.insert(node.id.clone(), node);
        }
    }

    /// Detect dependencies between two sets of nodes
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn detect(nodes1: &[UnifiedNode], nodes2: &[UnifiedNode]) -> Vec<CrossLanguageDependency> {
        debug_assert!(!nodes1.is_empty(), "nodes1 must not be empty");
        let mut detector = Self::new();
        detector.add_nodes(nodes1.to_vec());
        detector.add_nodes(nodes2.to_vec());
        detector.detect_all();
        detector.dependencies
    }

    /// Detect all cross-language dependencies
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn detect_all(&mut self) -> &Vec<CrossLanguageDependency> {
        self.dependencies.clear();

        let mut nodes_by_language: HashMap<Language, Vec<String>> = HashMap::new();
        for (id, node) in &self.nodes {
            nodes_by_language
                .entry(node.language)
                .or_default()
                .push(id.clone());
        }

        let languages: Vec<Language> = nodes_by_language.keys().cloned().collect();
        let mut new_dependencies = Vec::new();

        for (i, lang1) in languages.iter().enumerate() {
            for lang2 in languages.iter().skip(i + 1) {
                if lang1 != lang2 {
                    if let (Some(ids1), Some(ids2)) =
                        (nodes_by_language.get(lang1), nodes_by_language.get(lang2))
                    {
                        let deps = self.detect_between_language_groups(ids1, *lang1, ids2, *lang2);
                        new_dependencies.extend(deps);
                    }
                }
            }
        }

        self.dependencies.extend(new_dependencies);
        self.resolve_references();

        // Deduplicate dependencies
        let mut seen = std::collections::HashSet::new();
        self.dependencies.retain(|dep| {
            let key = (dep.source_id.clone(), dep.target_id.clone(), dep.kind);
            seen.insert(key)
        });

        &self.dependencies
    }

    /// Get all dependencies
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn get_dependencies(&self) -> &Vec<CrossLanguageDependency> {
        &self.dependencies
    }

    /// Filter dependencies by source language
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn filter_by_source_language(&self, language: Language) -> Vec<&CrossLanguageDependency> {
        self.dependencies
            .iter()
            .filter(|dep| dep.source_language == language)
            .collect()
    }

    /// Filter dependencies by target language
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn filter_by_target_language(&self, language: Language) -> Vec<&CrossLanguageDependency> {
        self.dependencies
            .iter()
            .filter(|dep| dep.target_language == language)
            .collect()
    }

    /// Filter dependencies by type
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn filter_by_kind(&self, kind: ReferenceKind) -> Vec<&CrossLanguageDependency> {
        self.dependencies
            .iter()
            .filter(|dep| dep.kind == kind)
            .collect()
    }

    /// Get dependencies between two specific languages
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn get_dependencies_between(
        &self,
        source_language: Language,
        target_language: Language,
    ) -> Vec<&CrossLanguageDependency> {
        self.dependencies
            .iter()
            .filter(|dep| {
                dep.source_language == source_language && dep.target_language == target_language
            })
            .collect()
    }
}

// Detection: detect_between_language_groups, find_matching_targets,
// detect_between_languages, is_reference_match, add_dependency
include!("cross_language_dependencies_detection.rs");

// Resolution: resolve_references, build_name_map,
// collect_unresolved_references, resolve_against_name_map
include!("cross_language_dependencies_resolution.rs");

// DOT graph generation: to_dot, language_color
include!("cross_language_dependencies_dot.rs");

// Language-specific resolvers: JavaKotlinResolver, JavaScalaResolver, TypeScriptJavaResolver
include!("cross_language_dependencies_resolvers.rs");

#[cfg(test)]
#[path = "cross_language_dependencies_tests.rs"]
mod tests;

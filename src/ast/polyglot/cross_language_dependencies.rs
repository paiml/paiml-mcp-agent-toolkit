//! Cross-language dependency detection and analysis
//!
//! This module provides functionality to detect and analyze dependencies
//! between nodes in different programming languages. It can identify
//! relationships such as inheritance, implementation, and usage across
//! language boundaries.

use crate::ast::polyglot::unified_node::ReferenceKind;
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

impl CrossLanguageDependencies {
    /// Create a new instance
    pub fn new() -> Self {
        Self {
            nodes: HashMap::new(),
            fqn_map: HashMap::new(),
            name_resolvers: HashMap::new(),
            dependencies: Vec::new(),
        }
    }

    /// Add language-specific name resolver
    pub fn add_name_resolver(&mut self, language: Language, resolver: Box<dyn NameResolver>) {
        self.name_resolvers.insert(language, resolver);
    }

    /// Add nodes to the dependency detector
    pub fn add_nodes(&mut self, nodes: Vec<UnifiedNode>) {
        for node in nodes {
            // Add to FQN map
            self.fqn_map
                .entry(node.fqn.clone())
                .or_default()
                .insert(node.id.clone());

            // Add to node map
            self.nodes.insert(node.id.clone(), node);
        }
    }

    /// Detect dependencies between two sets of nodes
    pub fn detect(nodes1: &[UnifiedNode], nodes2: &[UnifiedNode]) -> Vec<CrossLanguageDependency> {
        let mut detector = Self::new();
        detector.add_nodes(nodes1.to_vec());
        detector.add_nodes(nodes2.to_vec());
        detector.detect_all();
        detector.dependencies
    }

    /// Detect all cross-language dependencies
    pub fn detect_all(&mut self) -> &Vec<CrossLanguageDependency> {
        self.dependencies.clear();

        // Group nodes by language
        let mut nodes_by_language: HashMap<Language, Vec<String>> = HashMap::new();
        for (id, node) in &self.nodes {
            nodes_by_language
                .entry(node.language)
                .or_default()
                .push(id.clone());
        }

        // Compare nodes between different languages
        let languages: Vec<Language> = nodes_by_language.keys().cloned().collect();

        // Create a list of dependencies to add (to avoid borrowing self mutably while iterating)
        let mut new_dependencies = Vec::new();

        for (i, lang1) in languages.iter().enumerate() {
            for lang2 in languages.iter().skip(i + 1) {
                if lang1 != lang2 {
                    if let (Some(node_ids1), Some(node_ids2)) =
                        (nodes_by_language.get(lang1), nodes_by_language.get(lang2))
                    {
                        // Get dependencies between these language groups
                        let deps = self
                            .detect_between_language_groups(node_ids1, *lang1, node_ids2, *lang2);
                        new_dependencies.extend(deps);
                    }
                }
            }
        }

        // Add the collected dependencies
        self.dependencies.extend(new_dependencies);

        // Resolve unresolved references (moved outside)
        self.resolve_references();

        // Deduplicate dependencies (same source_id + target_id + kind = duplicate)
        let mut seen = std::collections::HashSet::new();
        self.dependencies.retain(|dep| {
            let key = (dep.source_id.clone(), dep.target_id.clone(), dep.kind);
            seen.insert(key)
        });

        &self.dependencies
    }

    /// Detect dependencies between nodes of two specific language groups
    /// This function avoids borrowing self mutably while iterating
    fn detect_between_language_groups(
        &self,
        node_ids1: &[String],
        lang1: Language,
        node_ids2: &[String],
        lang2: Language,
    ) -> Vec<CrossLanguageDependency> {
        let mut dependencies = Vec::new();

        // For each node in first language
        for id1 in node_ids1 {
            if let Some(source) = self.nodes.get(id1) {
                // For each reference in the node
                for reference in &source.references {
                    // For each node in second language
                    for id2 in node_ids2 {
                        if let Some(target) = self.nodes.get(id2) {
                            // Check if reference matches target
                            if self.is_reference_match(source, reference, target, lang1, lang2) {
                                // Create a dependency (without modifying self)
                                let dependency = CrossLanguageDependency {
                                    source_id: source.id.clone(),
                                    target_id: target.id.clone(),
                                    source_language: lang1,
                                    target_language: lang2,
                                    kind: reference.kind,
                                    confidence: 1.0,
                                    metadata: HashMap::new(),
                                };
                                dependencies.push(dependency);
                            }
                        }
                    }
                }
            }
        }

        dependencies
    }

    /// Detect dependencies between nodes of two specific languages
    #[allow(dead_code)]
    fn detect_between_languages(
        &mut self,
        nodes1: &[&UnifiedNode],
        lang1: Language,
        nodes2: &[&UnifiedNode],
        lang2: Language,
    ) {
        // For each node in first language
        for &source in nodes1 {
            // For each reference in the node
            for reference in &source.references {
                // For each node in second language
                for &target in nodes2 {
                    // Check if reference matches target
                    if self.is_reference_match(source, reference, target, lang1, lang2) {
                        self.add_dependency(source, target, reference.kind, 1.0);
                    }
                }
            }
        }
    }

    /// Check if a reference from one node matches a target node
    fn is_reference_match(
        &self,
        source: &UnifiedNode,
        reference: &crate::ast::polyglot::unified_node::NodeReference,
        target: &UnifiedNode,
        source_lang: Language,
        target_lang: Language,
    ) -> bool {
        // Direct ID match
        if !reference.target_id.is_empty() && reference.target_id == target.id {
            return true;
        }

        // Name match
        if reference.target_name == target.name || reference.target_name == target.fqn {
            return true;
        }

        // Try to resolve using language-specific resolver
        if let Some(resolver) = self.name_resolvers.get(&source_lang) {
            if resolver.can_resolve(source_lang, target_lang, source, reference, target) {
                return true;
            }
        }

        false
    }

    /// Add a dependency between two nodes
    #[allow(dead_code)]
    fn add_dependency(
        &mut self,
        source: &UnifiedNode,
        target: &UnifiedNode,
        kind: ReferenceKind,
        confidence: f64,
    ) {
        let dependency = CrossLanguageDependency {
            source_id: source.id.clone(),
            target_id: target.id.clone(),
            source_language: source.language,
            target_language: target.language,
            kind,
            confidence,
            metadata: HashMap::new(),
        };

        self.dependencies.push(dependency);
    }

    /// Resolve unresolved references
    fn resolve_references(&mut self) {
        // First collect all unresolved references we need to process
        let mut to_resolve = Vec::new();

        // Collect nodes by name for faster lookup (using String keys instead of refs)
        let mut name_map: HashMap<String, Vec<String>> = HashMap::new();

        // Build the name map
        for (id, node) in &self.nodes {
            name_map
                .entry(node.name.clone())
                .or_default()
                .push(id.clone());

            name_map
                .entry(node.fqn.clone())
                .or_default()
                .push(id.clone());
        }

        // Collect references that need resolution
        for (source_id, source) in &self.nodes {
            for reference in &source.references {
                if reference.target_id.is_empty() {
                    // Store info about unresolved reference
                    to_resolve.push((
                        source_id.clone(),
                        reference.target_name.clone(),
                        reference.kind,
                    ));
                }
            }
        }

        // Process all unresolved references
        let mut new_dependencies = Vec::new();
        for (source_id, target_name, kind) in to_resolve {
            // Get the source node
            if let Some(source) = self.nodes.get(&source_id) {
                // Look for candidate targets by name
                if let Some(target_ids) = name_map.get(&target_name) {
                    for target_id in target_ids {
                        if let Some(target) = self.nodes.get(target_id) {
                            // Check if this is a cross-language reference
                            if source.language != target.language {
                                let dependency = CrossLanguageDependency {
                                    source_id: source_id.clone(),
                                    target_id: target_id.clone(),
                                    source_language: source.language,
                                    target_language: target.language,
                                    kind,
                                    confidence: 0.8, // Lower confidence for name-based resolution
                                    metadata: HashMap::new(),
                                };
                                new_dependencies.push(dependency);
                            }
                        }
                    }
                }
            }
        }

        // Add all new dependencies
        self.dependencies.extend(new_dependencies);
    }

    /// Get all dependencies
    pub fn get_dependencies(&self) -> &Vec<CrossLanguageDependency> {
        &self.dependencies
    }

    /// Filter dependencies by source language
    pub fn filter_by_source_language(&self, language: Language) -> Vec<&CrossLanguageDependency> {
        self.dependencies
            .iter()
            .filter(|dep| dep.source_language == language)
            .collect()
    }

    /// Filter dependencies by target language
    pub fn filter_by_target_language(&self, language: Language) -> Vec<&CrossLanguageDependency> {
        self.dependencies
            .iter()
            .filter(|dep| dep.target_language == language)
            .collect()
    }

    /// Filter dependencies by type
    pub fn filter_by_kind(&self, kind: ReferenceKind) -> Vec<&CrossLanguageDependency> {
        self.dependencies
            .iter()
            .filter(|dep| dep.kind == kind)
            .collect()
    }

    /// Get dependencies between two specific languages
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

    /// Generate a dependency graph in DOT format
    pub fn to_dot(&self) -> String {
        let mut dot = String::from("digraph CrossLanguageDependencies {\n");

        // Add nodes
        for node in self.nodes.values() {
            let shape = match node.kind {
                NodeKind::Class => "box",
                NodeKind::Interface | NodeKind::Trait => "ellipse",
                NodeKind::Method | NodeKind::Function => "octagon",
                _ => "plaintext",
            };

            let label = format!("{} ({})", node.name, node.language.name());
            dot.push_str(&format!(
                "    \"{}\" [label=\"{}\" shape={} style=filled fillcolor={}];\n",
                node.id,
                label,
                shape,
                self.language_color(node.language)
            ));
        }

        // Add edges
        for dep in &self.dependencies {
            let style = match dep.kind {
                ReferenceKind::Inherits => "bold",
                ReferenceKind::Implements => "dashed",
                _ => "solid",
            };

            let label = format!("{:?}", dep.kind);
            dot.push_str(&format!(
                "    \"{}\" -> \"{}\" [label=\"{}\" style={}];\n",
                dep.source_id, dep.target_id, label, style
            ));
        }

        dot.push_str("}\n");
        dot
    }

    /// Get color for a language in DOT format
    fn language_color(&self, language: Language) -> &'static str {
        match language {
            Language::Java => "\"#b07219\"",       // Java brown
            Language::Kotlin => "\"#A97BFF\"",     // Kotlin purple
            Language::Scala => "\"#c22d40\"",      // Scala red
            Language::TypeScript => "\"#2b7489\"", // TypeScript blue
            Language::JavaScript => "\"#f1e05a\"", // JavaScript yellow
            Language::Python => "\"#3572A5\"",     // Python blue
            Language::Rust => "\"#dea584\"",       // Rust orange
            Language::Go => "\"#00ADD8\"",         // Go blue
            Language::Cpp => "\"#f34b7d\"",        // C++ pink
            _ => "\"#bbbbbb\"",                    // Gray for others
        }
    }
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

/// Java-to-Kotlin name resolver
pub struct JavaKotlinResolver;

impl NameResolver for JavaKotlinResolver {
    fn can_resolve(
        &self,
        source_language: Language,
        target_language: Language,
        _source: &UnifiedNode,
        reference: &crate::ast::polyglot::unified_node::NodeReference,
        target: &UnifiedNode,
    ) -> bool {
        // Only handle Java->Kotlin and Kotlin->Java
        if !((source_language == Language::Java && target_language == Language::Kotlin)
            || (source_language == Language::Kotlin && target_language == Language::Java))
        {
            return false;
        }

        // Direct name match
        if reference.target_name == target.name {
            return true;
        }

        // FQN match
        if reference.target_name == target.fqn {
            return true;
        }

        // Java package name conversion (com.example -> com.example)
        let src_parts: Vec<&str> = reference.target_name.split('.').collect();
        let tgt_parts: Vec<&str> = target.fqn.split('.').collect();

        if src_parts.len() == tgt_parts.len() {
            // Check if all parts except the last match exactly
            if src_parts[0..src_parts.len() - 1] == tgt_parts[0..tgt_parts.len() - 1] {
                // Check if the last part (class/method name) matches
                if src_parts.last().expect("internal error")
                    == tgt_parts.last().expect("internal error")
                {
                    return true;
                }
            }
        }

        false
    }
}

/// Java-to-Scala name resolver
pub struct JavaScalaResolver;

impl NameResolver for JavaScalaResolver {
    fn can_resolve(
        &self,
        source_language: Language,
        target_language: Language,
        _source: &UnifiedNode,
        reference: &crate::ast::polyglot::unified_node::NodeReference,
        target: &UnifiedNode,
    ) -> bool {
        // Only handle Java->Scala and Scala->Java
        if !((source_language == Language::Java && target_language == Language::Scala)
            || (source_language == Language::Scala && target_language == Language::Java))
        {
            return false;
        }

        // Direct name match
        if reference.target_name == target.name {
            return true;
        }

        // FQN match
        if reference.target_name == target.fqn {
            return true;
        }

        // Scala package name conversion (com.example -> com.example)
        let src_parts: Vec<&str> = reference.target_name.split('.').collect();
        let tgt_parts: Vec<&str> = target.fqn.split('.').collect();

        if src_parts.len() == tgt_parts.len() {
            // Check if all parts except the last match exactly
            if src_parts[0..src_parts.len() - 1] == tgt_parts[0..tgt_parts.len() - 1] {
                // Check if the last part (class/method name) matches
                if src_parts.last().expect("internal error")
                    == tgt_parts.last().expect("internal error")
                {
                    return true;
                }
            }
        }

        false
    }
}

/// TypeScript-to-Java name resolver
pub struct TypeScriptJavaResolver;

impl NameResolver for TypeScriptJavaResolver {
    fn can_resolve(
        &self,
        source_language: Language,
        target_language: Language,
        _source: &UnifiedNode,
        reference: &crate::ast::polyglot::unified_node::NodeReference,
        target: &UnifiedNode,
    ) -> bool {
        // Only handle TypeScript->Java and Java->TypeScript
        if !((source_language == Language::TypeScript && target_language == Language::Java)
            || (source_language == Language::Java && target_language == Language::TypeScript))
        {
            return false;
        }

        // Direct name match
        if reference.target_name == target.name {
            return true;
        }

        // Handle common naming conventions in web applications
        // e.g., UserService.ts might reference com.example.UserService
        if target.fqn.ends_with(&reference.target_name) {
            return true;
        }

        // Handle TypeScript interface to Java class mapping (IUser -> User)
        if source_language == Language::TypeScript
            && reference.target_name.starts_with('I')
            && reference.target_name.len() > 1
        {
            let name_without_i = &reference.target_name[1..];
            if name_without_i == target.name {
                return true;
            }
            if target.fqn.ends_with(name_without_i) {
                return true;
            }
        }

        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ast::polyglot::unified_node::SourcePosition;
    use std::path::PathBuf;

    fn create_test_node(
        id: &str,
        kind: NodeKind,
        name: &str,
        fqn: &str,
        language: Language,
    ) -> UnifiedNode {
        UnifiedNode {
            id: id.to_string(),
            kind,
            name: name.to_string(),
            fqn: fqn.to_string(),
            language,
            file_path: PathBuf::from("/test/path"),
            position: SourcePosition::default(),
            attributes: HashMap::new(),
            children: Vec::new(),
            parent: None,
            references: Vec::new(),
            type_info: None,
            signature: None,
            documentation: None,
            original_item: None,
            metadata: HashMap::new(),
        }
    }

    fn create_test_node_with_references(
        id: &str,
        kind: NodeKind,
        name: &str,
        fqn: &str,
        language: Language,
        references: Vec<crate::ast::polyglot::unified_node::NodeReference>,
    ) -> UnifiedNode {
        UnifiedNode {
            id: id.to_string(),
            kind,
            name: name.to_string(),
            fqn: fqn.to_string(),
            language,
            file_path: PathBuf::from("/test/path"),
            position: SourcePosition::default(),
            attributes: HashMap::new(),
            children: Vec::new(),
            parent: None,
            references,
            type_info: None,
            signature: None,
            documentation: None,
            original_item: None,
            metadata: HashMap::new(),
        }
    }

    #[test]
    fn test_detect_dependencies() {
        // Create Java class
        let mut java_class = create_test_node(
            "Java:class:User",
            NodeKind::Class,
            "User",
            "com.example.User",
            Language::Java,
        );

        // Create Kotlin class
        let kotlin_class = create_test_node(
            "Kotlin:class:KotlinUser",
            NodeKind::Class,
            "KotlinUser",
            "com.example.KotlinUser",
            Language::Kotlin,
        );

        // Add reference from Java to Kotlin
        java_class.add_reference(
            ReferenceKind::Inherits,
            "com.example.KotlinUser".to_string(),
            None,
        );

        // Detect dependencies
        let mut dependencies = CrossLanguageDependencies::detect(&[java_class], &[kotlin_class]);

        // Sort dependencies by source_id to make test deterministic
        dependencies.sort_by(|a, b| {
            a.source_id
                .cmp(&b.source_id)
                .then(a.target_id.cmp(&b.target_id))
                .then(a.kind.cmp(&b.kind))
        });

        // Verify - exactly one dependency
        assert_eq!(
            dependencies.len(),
            1,
            "Expected exactly 1 dependency, found {}",
            dependencies.len()
        );
        assert_eq!(dependencies[0].source_id, "Java:class:User");
        assert_eq!(dependencies[0].target_id, "Kotlin:class:KotlinUser");
        assert_eq!(dependencies[0].kind, ReferenceKind::Inherits);
        assert_eq!(dependencies[0].source_language, Language::Java);
        assert_eq!(dependencies[0].target_language, Language::Kotlin);
    }

    #[test]
    fn test_name_resolvers() {
        // Java-Kotlin resolver
        let java_kotlin_resolver = JavaKotlinResolver;

        let java_node = create_test_node(
            "Java:class:User",
            NodeKind::Class,
            "User",
            "com.example.User",
            Language::Java,
        );

        let kotlin_node = create_test_node(
            "Kotlin:class:User",
            NodeKind::Class,
            "User",
            "com.example.User",
            Language::Kotlin,
        );

        let reference = crate::ast::polyglot::unified_node::NodeReference {
            kind: ReferenceKind::Inherits,
            target_id: String::new(),
            target_name: "com.example.User".to_string(),
            target_language: Some(Language::Kotlin),
        };

        assert!(java_kotlin_resolver.can_resolve(
            Language::Java,
            Language::Kotlin,
            &java_node,
            &reference,
            &kotlin_node,
        ));
    }

    // EXTREME TDD TESTS - Cross-Language Dependency Coverage

    // Test CrossLanguageDependencies::new() and basic initialization
    #[test]
    fn test_new_creates_empty_instance() {
        let deps = CrossLanguageDependencies::new();
        assert!(deps.get_dependencies().is_empty());
    }

    // Test add_nodes functionality
    #[test]
    fn test_add_nodes_basic() {
        let mut deps = CrossLanguageDependencies::new();
        let node1 = create_test_node(
            "Java:class:User",
            NodeKind::Class,
            "User",
            "com.example.User",
            Language::Java,
        );
        let node2 = create_test_node(
            "Kotlin:class:Service",
            NodeKind::Class,
            "Service",
            "com.example.Service",
            Language::Kotlin,
        );

        deps.add_nodes(vec![node1, node2]);
        // Verify nodes were added by running detect_all
        let result = deps.detect_all();
        // No references = no dependencies
        assert!(result.is_empty());
    }

    // Test add_name_resolver functionality
    #[test]
    fn test_add_name_resolver() {
        let mut deps = CrossLanguageDependencies::new();
        deps.add_name_resolver(Language::Java, Box::new(JavaKotlinResolver));
        deps.add_name_resolver(Language::Kotlin, Box::new(JavaKotlinResolver));

        // Create nodes with references that can be resolved
        let mut java_node = create_test_node(
            "Java:class:User",
            NodeKind::Class,
            "User",
            "com.example.User",
            Language::Java,
        );
        java_node.add_reference(
            ReferenceKind::Uses,
            "com.example.KotlinService".to_string(),
            None,
        );

        let kotlin_node = create_test_node(
            "Kotlin:class:KotlinService",
            NodeKind::Class,
            "KotlinService",
            "com.example.KotlinService",
            Language::Kotlin,
        );

        deps.add_nodes(vec![java_node, kotlin_node]);
        let result = deps.detect_all();
        assert!(!result.is_empty());
    }

    // Test filter_by_source_language
    #[test]
    fn test_filter_by_source_language() {
        let mut java_class = create_test_node(
            "Java:class:User",
            NodeKind::Class,
            "User",
            "com.example.User",
            Language::Java,
        );
        java_class.add_reference(ReferenceKind::Inherits, "KotlinBase".to_string(), None);

        let kotlin_class = create_test_node(
            "Kotlin:class:KotlinBase",
            NodeKind::Class,
            "KotlinBase",
            "com.example.KotlinBase",
            Language::Kotlin,
        );

        let mut deps = CrossLanguageDependencies::new();
        deps.add_nodes(vec![java_class, kotlin_class]);
        deps.detect_all();

        let java_deps = deps.filter_by_source_language(Language::Java);
        assert!(!java_deps.is_empty());
        for dep in &java_deps {
            assert_eq!(dep.source_language, Language::Java);
        }

        let python_deps = deps.filter_by_source_language(Language::Python);
        assert!(python_deps.is_empty());
    }

    // Test filter_by_target_language
    #[test]
    fn test_filter_by_target_language() {
        let mut java_class = create_test_node(
            "Java:class:User",
            NodeKind::Class,
            "User",
            "com.example.User",
            Language::Java,
        );
        java_class.add_reference(
            ReferenceKind::Implements,
            "KotlinInterface".to_string(),
            None,
        );

        let kotlin_interface = create_test_node(
            "Kotlin:interface:KotlinInterface",
            NodeKind::Interface,
            "KotlinInterface",
            "com.example.KotlinInterface",
            Language::Kotlin,
        );

        let mut deps = CrossLanguageDependencies::new();
        deps.add_nodes(vec![java_class, kotlin_interface]);
        deps.detect_all();

        let kotlin_deps = deps.filter_by_target_language(Language::Kotlin);
        assert!(!kotlin_deps.is_empty());
        for dep in &kotlin_deps {
            assert_eq!(dep.target_language, Language::Kotlin);
        }

        let rust_deps = deps.filter_by_target_language(Language::Rust);
        assert!(rust_deps.is_empty());
    }

    // Test filter_by_kind
    #[test]
    fn test_filter_by_kind() {
        let mut java_class = create_test_node(
            "Java:class:User",
            NodeKind::Class,
            "User",
            "com.example.User",
            Language::Java,
        );
        java_class.add_reference(ReferenceKind::Inherits, "KotlinBase".to_string(), None);
        java_class.add_reference(
            ReferenceKind::Implements,
            "KotlinInterface".to_string(),
            None,
        );

        let kotlin_base = create_test_node(
            "Kotlin:class:KotlinBase",
            NodeKind::Class,
            "KotlinBase",
            "com.example.KotlinBase",
            Language::Kotlin,
        );

        let kotlin_interface = create_test_node(
            "Kotlin:interface:KotlinInterface",
            NodeKind::Interface,
            "KotlinInterface",
            "com.example.KotlinInterface",
            Language::Kotlin,
        );

        let mut deps = CrossLanguageDependencies::new();
        deps.add_nodes(vec![java_class, kotlin_base, kotlin_interface]);
        deps.detect_all();

        let inherits_deps = deps.filter_by_kind(ReferenceKind::Inherits);
        let implements_deps = deps.filter_by_kind(ReferenceKind::Implements);
        let calls_deps = deps.filter_by_kind(ReferenceKind::Calls);

        // At least one inherits and one implements dependency
        assert!(!inherits_deps.is_empty());
        assert!(!implements_deps.is_empty());
        assert!(calls_deps.is_empty());
    }

    // Test get_dependencies_between
    #[test]
    fn test_get_dependencies_between() {
        let mut java_class = create_test_node(
            "Java:class:User",
            NodeKind::Class,
            "User",
            "com.example.User",
            Language::Java,
        );
        java_class.add_reference(ReferenceKind::Inherits, "KotlinBase".to_string(), None);

        let kotlin_base = create_test_node(
            "Kotlin:class:KotlinBase",
            NodeKind::Class,
            "KotlinBase",
            "com.example.KotlinBase",
            Language::Kotlin,
        );

        let mut deps = CrossLanguageDependencies::new();
        deps.add_nodes(vec![java_class, kotlin_base]);
        deps.detect_all();

        let java_to_kotlin = deps.get_dependencies_between(Language::Java, Language::Kotlin);
        assert!(!java_to_kotlin.is_empty());

        let kotlin_to_java = deps.get_dependencies_between(Language::Kotlin, Language::Java);
        assert!(kotlin_to_java.is_empty());

        let java_to_python = deps.get_dependencies_between(Language::Java, Language::Python);
        assert!(java_to_python.is_empty());
    }

    // Test to_dot graph generation
    #[test]
    fn test_to_dot_generation() {
        let mut java_class = create_test_node(
            "Java:class:User",
            NodeKind::Class,
            "User",
            "com.example.User",
            Language::Java,
        );
        java_class.add_reference(ReferenceKind::Inherits, "KotlinBase".to_string(), None);

        let kotlin_base = create_test_node(
            "Kotlin:class:KotlinBase",
            NodeKind::Class,
            "KotlinBase",
            "com.example.KotlinBase",
            Language::Kotlin,
        );

        let mut deps = CrossLanguageDependencies::new();
        deps.add_nodes(vec![java_class, kotlin_base]);
        deps.detect_all();

        let dot = deps.to_dot();

        // Verify basic DOT structure
        assert!(dot.starts_with("digraph CrossLanguageDependencies {"));
        assert!(dot.ends_with("}\n"));
        assert!(dot.contains("\"Java:class:User\""));
        assert!(dot.contains("\"Kotlin:class:KotlinBase\""));
        assert!(dot.contains("User (Java)"));
        assert!(dot.contains("KotlinBase (Kotlin)"));
        // Should have an edge for the inheritance
        assert!(dot.contains("->"));
        assert!(dot.contains("Inherits"));
        assert!(dot.contains("bold")); // Inherits style
    }

    // Test to_dot with different node kinds
    #[test]
    fn test_to_dot_node_shapes() {
        let class_node = create_test_node(
            "Java:class:MyClass",
            NodeKind::Class,
            "MyClass",
            "com.example.MyClass",
            Language::Java,
        );

        let interface_node = create_test_node(
            "Java:interface:MyInterface",
            NodeKind::Interface,
            "MyInterface",
            "com.example.MyInterface",
            Language::Java,
        );

        let trait_node = create_test_node(
            "Rust:trait:MyTrait",
            NodeKind::Trait,
            "MyTrait",
            "crate::MyTrait",
            Language::Rust,
        );

        let method_node = create_test_node(
            "Java:method:doSomething",
            NodeKind::Method,
            "doSomething",
            "com.example.MyClass.doSomething",
            Language::Java,
        );

        let function_node = create_test_node(
            "Python:function:process",
            NodeKind::Function,
            "process",
            "module.process",
            Language::Python,
        );

        let field_node = create_test_node(
            "Java:field:name",
            NodeKind::Field,
            "name",
            "com.example.MyClass.name",
            Language::Java,
        );

        let mut deps = CrossLanguageDependencies::new();
        deps.add_nodes(vec![
            class_node,
            interface_node,
            trait_node,
            method_node,
            function_node,
            field_node,
        ]);

        let dot = deps.to_dot();

        // Check shapes
        assert!(dot.contains("shape=box")); // Class
        assert!(dot.contains("shape=ellipse")); // Interface/Trait
        assert!(dot.contains("shape=octagon")); // Method/Function
        assert!(dot.contains("shape=plaintext")); // Field and others
    }

    // Test to_dot with implements relationship (dashed style)
    #[test]
    fn test_to_dot_implements_style() {
        let mut java_class = create_test_node(
            "Java:class:User",
            NodeKind::Class,
            "User",
            "com.example.User",
            Language::Java,
        );
        java_class.add_reference(
            ReferenceKind::Implements,
            "KotlinInterface".to_string(),
            None,
        );

        let kotlin_interface = create_test_node(
            "Kotlin:interface:KotlinInterface",
            NodeKind::Interface,
            "KotlinInterface",
            "com.example.KotlinInterface",
            Language::Kotlin,
        );

        let mut deps = CrossLanguageDependencies::new();
        deps.add_nodes(vec![java_class, kotlin_interface]);
        deps.detect_all();

        let dot = deps.to_dot();
        assert!(dot.contains("dashed")); // Implements style
    }

    // Test language_color for all languages
    #[test]
    fn test_language_colors() {
        let deps = CrossLanguageDependencies::new();

        // Test all specific language colors
        assert_eq!(deps.language_color(Language::Java), "\"#b07219\"");
        assert_eq!(deps.language_color(Language::Kotlin), "\"#A97BFF\"");
        assert_eq!(deps.language_color(Language::Scala), "\"#c22d40\"");
        assert_eq!(deps.language_color(Language::TypeScript), "\"#2b7489\"");
        assert_eq!(deps.language_color(Language::JavaScript), "\"#f1e05a\"");
        assert_eq!(deps.language_color(Language::Python), "\"#3572A5\"");
        assert_eq!(deps.language_color(Language::Rust), "\"#dea584\"");
        assert_eq!(deps.language_color(Language::Go), "\"#00ADD8\"");
        assert_eq!(deps.language_color(Language::Cpp), "\"#f34b7d\"");
        // Test fallback for other languages
        assert_eq!(deps.language_color(Language::Ruby), "\"#bbbbbb\"");
        assert_eq!(deps.language_color(Language::Swift), "\"#bbbbbb\"");
        assert_eq!(deps.language_color(Language::Php), "\"#bbbbbb\"");
        assert_eq!(deps.language_color(Language::CSharp), "\"#bbbbbb\"");
        assert_eq!(deps.language_color(Language::Other(999)), "\"#bbbbbb\"");
    }

    // Test JavaScalaResolver
    #[test]
    fn test_java_scala_resolver_direct_name_match() {
        let resolver = JavaScalaResolver;

        let java_node = create_test_node(
            "Java:class:User",
            NodeKind::Class,
            "User",
            "com.example.User",
            Language::Java,
        );

        let scala_node = create_test_node(
            "Scala:class:User",
            NodeKind::Class,
            "User",
            "com.example.User",
            Language::Scala,
        );

        let reference = crate::ast::polyglot::unified_node::NodeReference {
            kind: ReferenceKind::Inherits,
            target_id: String::new(),
            target_name: "User".to_string(),
            target_language: Some(Language::Scala),
        };

        assert!(resolver.can_resolve(
            Language::Java,
            Language::Scala,
            &java_node,
            &reference,
            &scala_node,
        ));
    }

    #[test]
    fn test_java_scala_resolver_fqn_match() {
        let resolver = JavaScalaResolver;

        let java_node = create_test_node(
            "Java:class:User",
            NodeKind::Class,
            "User",
            "com.example.User",
            Language::Java,
        );

        let scala_node = create_test_node(
            "Scala:class:ScalaUser",
            NodeKind::Class,
            "ScalaUser",
            "com.example.ScalaUser",
            Language::Scala,
        );

        let reference = crate::ast::polyglot::unified_node::NodeReference {
            kind: ReferenceKind::Uses,
            target_id: String::new(),
            target_name: "com.example.ScalaUser".to_string(),
            target_language: Some(Language::Scala),
        };

        assert!(resolver.can_resolve(
            Language::Java,
            Language::Scala,
            &java_node,
            &reference,
            &scala_node,
        ));
    }

    #[test]
    fn test_java_scala_resolver_package_conversion() {
        let resolver = JavaScalaResolver;

        let java_node = create_test_node(
            "Java:class:User",
            NodeKind::Class,
            "User",
            "com.example.User",
            Language::Java,
        );

        let scala_node = create_test_node(
            "Scala:class:Service",
            NodeKind::Class,
            "Service",
            "com.example.Service",
            Language::Scala,
        );

        let reference = crate::ast::polyglot::unified_node::NodeReference {
            kind: ReferenceKind::Uses,
            target_id: String::new(),
            target_name: "com.example.Service".to_string(),
            target_language: Some(Language::Scala),
        };

        assert!(resolver.can_resolve(
            Language::Java,
            Language::Scala,
            &java_node,
            &reference,
            &scala_node,
        ));
    }

    #[test]
    fn test_java_scala_resolver_wrong_languages() {
        let resolver = JavaScalaResolver;

        let java_node = create_test_node(
            "Java:class:User",
            NodeKind::Class,
            "User",
            "com.example.User",
            Language::Java,
        );

        let kotlin_node = create_test_node(
            "Kotlin:class:User",
            NodeKind::Class,
            "User",
            "com.example.User",
            Language::Kotlin,
        );

        let reference = crate::ast::polyglot::unified_node::NodeReference {
            kind: ReferenceKind::Inherits,
            target_id: String::new(),
            target_name: "User".to_string(),
            target_language: Some(Language::Kotlin),
        };

        // Should return false for non Java-Scala combination
        assert!(!resolver.can_resolve(
            Language::Java,
            Language::Kotlin,
            &java_node,
            &reference,
            &kotlin_node,
        ));
    }

    #[test]
    fn test_scala_to_java_resolver() {
        let resolver = JavaScalaResolver;

        let scala_node = create_test_node(
            "Scala:class:ScalaUser",
            NodeKind::Class,
            "ScalaUser",
            "com.example.ScalaUser",
            Language::Scala,
        );

        let java_node = create_test_node(
            "Java:class:JavaService",
            NodeKind::Class,
            "JavaService",
            "com.example.JavaService",
            Language::Java,
        );

        let reference = crate::ast::polyglot::unified_node::NodeReference {
            kind: ReferenceKind::Uses,
            target_id: String::new(),
            target_name: "JavaService".to_string(),
            target_language: Some(Language::Java),
        };

        // Scala to Java should also work
        assert!(resolver.can_resolve(
            Language::Scala,
            Language::Java,
            &scala_node,
            &reference,
            &java_node,
        ));
    }

    // Test TypeScriptJavaResolver
    #[test]
    fn test_typescript_java_resolver_direct_name_match() {
        let resolver = TypeScriptJavaResolver;

        let ts_node = create_test_node(
            "TypeScript:class:UserService",
            NodeKind::Class,
            "UserService",
            "UserService",
            Language::TypeScript,
        );

        let java_node = create_test_node(
            "Java:class:UserService",
            NodeKind::Class,
            "UserService",
            "com.example.UserService",
            Language::Java,
        );

        let reference = crate::ast::polyglot::unified_node::NodeReference {
            kind: ReferenceKind::Uses,
            target_id: String::new(),
            target_name: "UserService".to_string(),
            target_language: Some(Language::Java),
        };

        assert!(resolver.can_resolve(
            Language::TypeScript,
            Language::Java,
            &ts_node,
            &reference,
            &java_node,
        ));
    }

    #[test]
    fn test_typescript_java_resolver_fqn_ends_with_match() {
        let resolver = TypeScriptJavaResolver;

        let ts_node = create_test_node(
            "TypeScript:class:UserService",
            NodeKind::Class,
            "UserService",
            "UserService",
            Language::TypeScript,
        );

        let java_node = create_test_node(
            "Java:class:UserService",
            NodeKind::Class,
            "UserService",
            "com.example.api.UserService",
            Language::Java,
        );

        let reference = crate::ast::polyglot::unified_node::NodeReference {
            kind: ReferenceKind::Uses,
            target_id: String::new(),
            target_name: "UserService".to_string(),
            target_language: Some(Language::Java),
        };

        assert!(resolver.can_resolve(
            Language::TypeScript,
            Language::Java,
            &ts_node,
            &reference,
            &java_node,
        ));
    }

    #[test]
    fn test_typescript_java_resolver_interface_prefix() {
        let resolver = TypeScriptJavaResolver;

        let ts_node = create_test_node(
            "TypeScript:interface:IUser",
            NodeKind::Interface,
            "IUser",
            "IUser",
            Language::TypeScript,
        );

        let java_node = create_test_node(
            "Java:class:User",
            NodeKind::Class,
            "User",
            "com.example.User",
            Language::Java,
        );

        let reference = crate::ast::polyglot::unified_node::NodeReference {
            kind: ReferenceKind::Uses,
            target_id: String::new(),
            target_name: "IUser".to_string(),
            target_language: Some(Language::Java),
        };

        // IUser -> User mapping
        assert!(resolver.can_resolve(
            Language::TypeScript,
            Language::Java,
            &ts_node,
            &reference,
            &java_node,
        ));
    }

    #[test]
    fn test_typescript_java_resolver_interface_prefix_fqn() {
        let resolver = TypeScriptJavaResolver;

        let ts_node = create_test_node(
            "TypeScript:interface:IUserService",
            NodeKind::Interface,
            "IUserService",
            "IUserService",
            Language::TypeScript,
        );

        let java_node = create_test_node(
            "Java:class:UserService",
            NodeKind::Class,
            "UserService",
            "com.example.api.UserService",
            Language::Java,
        );

        let reference = crate::ast::polyglot::unified_node::NodeReference {
            kind: ReferenceKind::Uses,
            target_id: String::new(),
            target_name: "IUserService".to_string(),
            target_language: Some(Language::Java),
        };

        // IUserService -> UserService FQN ends_with mapping
        assert!(resolver.can_resolve(
            Language::TypeScript,
            Language::Java,
            &ts_node,
            &reference,
            &java_node,
        ));
    }

    #[test]
    fn test_typescript_java_resolver_wrong_languages() {
        let resolver = TypeScriptJavaResolver;

        let ts_node = create_test_node(
            "TypeScript:class:UserService",
            NodeKind::Class,
            "UserService",
            "UserService",
            Language::TypeScript,
        );

        let python_node = create_test_node(
            "Python:class:UserService",
            NodeKind::Class,
            "UserService",
            "user_service.UserService",
            Language::Python,
        );

        let reference = crate::ast::polyglot::unified_node::NodeReference {
            kind: ReferenceKind::Uses,
            target_id: String::new(),
            target_name: "UserService".to_string(),
            target_language: Some(Language::Python),
        };

        // Should return false for non TypeScript-Java combination
        assert!(!resolver.can_resolve(
            Language::TypeScript,
            Language::Python,
            &ts_node,
            &reference,
            &python_node,
        ));
    }

    #[test]
    fn test_java_to_typescript_resolver() {
        let resolver = TypeScriptJavaResolver;

        let java_node = create_test_node(
            "Java:class:JavaService",
            NodeKind::Class,
            "JavaService",
            "com.example.JavaService",
            Language::Java,
        );

        let ts_node = create_test_node(
            "TypeScript:class:TypeScriptClient",
            NodeKind::Class,
            "TypeScriptClient",
            "TypeScriptClient",
            Language::TypeScript,
        );

        let reference = crate::ast::polyglot::unified_node::NodeReference {
            kind: ReferenceKind::Uses,
            target_id: String::new(),
            target_name: "TypeScriptClient".to_string(),
            target_language: Some(Language::TypeScript),
        };

        // Java to TypeScript should also work
        assert!(resolver.can_resolve(
            Language::Java,
            Language::TypeScript,
            &java_node,
            &reference,
            &ts_node,
        ));
    }

    // Test JavaKotlinResolver with all edge cases
    #[test]
    fn test_java_kotlin_resolver_direct_name() {
        let resolver = JavaKotlinResolver;

        let java_node = create_test_node(
            "Java:class:User",
            NodeKind::Class,
            "User",
            "com.example.User",
            Language::Java,
        );

        let kotlin_node = create_test_node(
            "Kotlin:class:User",
            NodeKind::Class,
            "User",
            "com.example.User",
            Language::Kotlin,
        );

        let reference = crate::ast::polyglot::unified_node::NodeReference {
            kind: ReferenceKind::Inherits,
            target_id: String::new(),
            target_name: "User".to_string(),
            target_language: Some(Language::Kotlin),
        };

        assert!(resolver.can_resolve(
            Language::Java,
            Language::Kotlin,
            &java_node,
            &reference,
            &kotlin_node,
        ));
    }

    #[test]
    fn test_java_kotlin_resolver_wrong_languages() {
        let resolver = JavaKotlinResolver;

        let java_node = create_test_node(
            "Java:class:User",
            NodeKind::Class,
            "User",
            "com.example.User",
            Language::Java,
        );

        let python_node = create_test_node(
            "Python:class:User",
            NodeKind::Class,
            "User",
            "user.User",
            Language::Python,
        );

        let reference = crate::ast::polyglot::unified_node::NodeReference {
            kind: ReferenceKind::Inherits,
            target_id: String::new(),
            target_name: "User".to_string(),
            target_language: Some(Language::Python),
        };

        // Should return false for non Java-Kotlin combination
        assert!(!resolver.can_resolve(
            Language::Java,
            Language::Python,
            &java_node,
            &reference,
            &python_node,
        ));
    }

    #[test]
    fn test_kotlin_to_java_resolver() {
        let resolver = JavaKotlinResolver;

        let kotlin_node = create_test_node(
            "Kotlin:class:KotlinUser",
            NodeKind::Class,
            "KotlinUser",
            "com.example.KotlinUser",
            Language::Kotlin,
        );

        let java_node = create_test_node(
            "Java:class:JavaService",
            NodeKind::Class,
            "JavaService",
            "com.example.JavaService",
            Language::Java,
        );

        let reference = crate::ast::polyglot::unified_node::NodeReference {
            kind: ReferenceKind::Uses,
            target_id: String::new(),
            target_name: "JavaService".to_string(),
            target_language: Some(Language::Java),
        };

        // Kotlin to Java should also work
        assert!(resolver.can_resolve(
            Language::Kotlin,
            Language::Java,
            &kotlin_node,
            &reference,
            &java_node,
        ));
    }

    #[test]
    fn test_java_kotlin_resolver_package_name_conversion() {
        let resolver = JavaKotlinResolver;

        let java_node = create_test_node(
            "Java:class:User",
            NodeKind::Class,
            "User",
            "com.example.User",
            Language::Java,
        );

        let kotlin_node = create_test_node(
            "Kotlin:class:Service",
            NodeKind::Class,
            "Service",
            "com.example.Service",
            Language::Kotlin,
        );

        let reference = crate::ast::polyglot::unified_node::NodeReference {
            kind: ReferenceKind::Uses,
            target_id: String::new(),
            target_name: "com.example.Service".to_string(),
            target_language: Some(Language::Kotlin),
        };

        assert!(resolver.can_resolve(
            Language::Java,
            Language::Kotlin,
            &java_node,
            &reference,
            &kotlin_node,
        ));
    }

    // Test direct ID match in is_reference_match
    #[test]
    fn test_is_reference_match_direct_id() {
        let deps = CrossLanguageDependencies::new();

        let source = create_test_node(
            "Java:class:User",
            NodeKind::Class,
            "User",
            "com.example.User",
            Language::Java,
        );

        let target = create_test_node(
            "Kotlin:class:KotlinUser",
            NodeKind::Class,
            "KotlinUser",
            "com.example.KotlinUser",
            Language::Kotlin,
        );

        let reference = crate::ast::polyglot::unified_node::NodeReference {
            kind: ReferenceKind::Inherits,
            target_id: "Kotlin:class:KotlinUser".to_string(),
            target_name: "".to_string(),
            target_language: Some(Language::Kotlin),
        };

        assert!(deps.is_reference_match(
            &source,
            &reference,
            &target,
            Language::Java,
            Language::Kotlin
        ));
    }

    // Test unresolved reference resolution
    #[test]
    fn test_resolve_references_by_name() {
        let java_ref = crate::ast::polyglot::unified_node::NodeReference {
            kind: ReferenceKind::Uses,
            target_id: String::new(), // Empty ID = unresolved
            target_name: "SharedComponent".to_string(),
            target_language: None,
        };

        let java_node = create_test_node_with_references(
            "Java:class:JavaClient",
            NodeKind::Class,
            "JavaClient",
            "com.example.JavaClient",
            Language::Java,
            vec![java_ref],
        );

        // Create a TypeScript node with the same name
        let ts_node = create_test_node(
            "TypeScript:class:SharedComponent",
            NodeKind::Class,
            "SharedComponent",
            "SharedComponent",
            Language::TypeScript,
        );

        let mut deps = CrossLanguageDependencies::new();
        deps.add_nodes(vec![java_node, ts_node]);
        deps.detect_all();

        let resolved_deps = deps.get_dependencies();
        // Should have resolved the reference by name
        assert!(!resolved_deps.is_empty());

        // Find the specific dependency
        let dep = resolved_deps.iter().find(|d| {
            d.source_id == "Java:class:JavaClient"
                && d.target_id == "TypeScript:class:SharedComponent"
        });
        assert!(dep.is_some());
        // Name-based resolution should have reasonable confidence
        // Value may vary based on detection order (0.8 for name-based, 1.0 for certain matches)
        let confidence = dep.unwrap().confidence;
        assert!(
            confidence >= 0.8 && confidence <= 1.0,
            "Confidence should be in valid range, got: {confidence}"
        );
    }

    // Test resolve_references with FQN match
    #[test]
    fn test_resolve_references_by_fqn() {
        let java_ref = crate::ast::polyglot::unified_node::NodeReference {
            kind: ReferenceKind::Imports,
            target_id: String::new(), // Empty ID = unresolved
            target_name: "com.example.SharedModule".to_string(),
            target_language: None,
        };

        let java_node = create_test_node_with_references(
            "Java:class:JavaClient",
            NodeKind::Class,
            "JavaClient",
            "com.example.JavaClient",
            Language::Java,
            vec![java_ref],
        );

        // Create a Kotlin node with the same FQN
        let kotlin_node = create_test_node(
            "Kotlin:module:SharedModule",
            NodeKind::Module,
            "SharedModule",
            "com.example.SharedModule", // Same FQN
            Language::Kotlin,
        );

        let mut deps = CrossLanguageDependencies::new();
        deps.add_nodes(vec![java_node, kotlin_node]);
        deps.detect_all();

        let resolved_deps = deps.get_dependencies();
        assert!(!resolved_deps.is_empty());
    }

    // Test deduplication of dependencies
    #[test]
    fn test_dependency_deduplication() {
        // Create a node with multiple references to the same target
        let ref1 = crate::ast::polyglot::unified_node::NodeReference {
            kind: ReferenceKind::Uses,
            target_id: String::new(),
            target_name: "SharedService".to_string(),
            target_language: None,
        };

        let ref2 = crate::ast::polyglot::unified_node::NodeReference {
            kind: ReferenceKind::Uses,
            target_id: String::new(),
            target_name: "SharedService".to_string(), // Same target, same kind
            target_language: None,
        };

        let java_node = create_test_node_with_references(
            "Java:class:JavaClient",
            NodeKind::Class,
            "JavaClient",
            "com.example.JavaClient",
            Language::Java,
            vec![ref1, ref2],
        );

        let kotlin_node = create_test_node(
            "Kotlin:class:SharedService",
            NodeKind::Class,
            "SharedService",
            "com.example.SharedService",
            Language::Kotlin,
        );

        let mut deps = CrossLanguageDependencies::new();
        deps.add_nodes(vec![java_node, kotlin_node]);
        deps.detect_all();

        // Count dependencies with same source_id, target_id, and kind
        let uses_deps: Vec<_> = deps
            .get_dependencies()
            .iter()
            .filter(|d| {
                d.source_id == "Java:class:JavaClient"
                    && d.target_id == "Kotlin:class:SharedService"
                    && d.kind == ReferenceKind::Uses
            })
            .collect();

        // Should be deduplicated to 1
        assert_eq!(uses_deps.len(), 1);
    }

    // Test multiple languages interaction
    #[test]
    fn test_multiple_languages() {
        let mut java_node = create_test_node(
            "Java:class:JavaService",
            NodeKind::Class,
            "JavaService",
            "com.example.JavaService",
            Language::Java,
        );
        java_node.add_reference(ReferenceKind::Uses, "KotlinHelper".to_string(), None);
        java_node.add_reference(ReferenceKind::Uses, "ScalaProcessor".to_string(), None);

        let kotlin_node = create_test_node(
            "Kotlin:class:KotlinHelper",
            NodeKind::Class,
            "KotlinHelper",
            "com.example.KotlinHelper",
            Language::Kotlin,
        );

        let scala_node = create_test_node(
            "Scala:class:ScalaProcessor",
            NodeKind::Class,
            "ScalaProcessor",
            "com.example.ScalaProcessor",
            Language::Scala,
        );

        let mut deps = CrossLanguageDependencies::new();
        deps.add_nodes(vec![java_node, kotlin_node, scala_node]);
        deps.detect_all();

        let java_to_kotlin = deps.get_dependencies_between(Language::Java, Language::Kotlin);
        let java_to_scala = deps.get_dependencies_between(Language::Java, Language::Scala);

        assert!(!java_to_kotlin.is_empty());
        assert!(!java_to_scala.is_empty());
    }

    // Test empty nodes case
    #[test]
    fn test_empty_nodes() {
        let deps = CrossLanguageDependencies::detect(&[], &[]);
        assert!(deps.is_empty());
    }

    // Test same language nodes (no cross-language deps)
    #[test]
    fn test_same_language_no_cross_deps() {
        let mut java1 = create_test_node(
            "Java:class:User",
            NodeKind::Class,
            "User",
            "com.example.User",
            Language::Java,
        );
        java1.add_reference(ReferenceKind::Uses, "Service".to_string(), None);

        let java2 = create_test_node(
            "Java:class:Service",
            NodeKind::Class,
            "Service",
            "com.example.Service",
            Language::Java,
        );

        let deps = CrossLanguageDependencies::detect(&[java1], &[java2]);
        // Same language references should not create cross-language dependencies
        assert!(deps.is_empty());
    }

    // Test different ReferenceKind types in DOT output
    #[test]
    fn test_to_dot_different_reference_kinds() {
        let mut java_class = create_test_node(
            "Java:class:User",
            NodeKind::Class,
            "User",
            "com.example.User",
            Language::Java,
        );
        // Add various reference types
        java_class.add_reference(ReferenceKind::Inherits, "KotlinBase".to_string(), None);
        java_class.add_reference(
            ReferenceKind::Implements,
            "KotlinInterface".to_string(),
            None,
        );
        java_class.add_reference(ReferenceKind::Calls, "KotlinFunction".to_string(), None);

        let kotlin_base = create_test_node(
            "Kotlin:class:KotlinBase",
            NodeKind::Class,
            "KotlinBase",
            "com.example.KotlinBase",
            Language::Kotlin,
        );

        let kotlin_interface = create_test_node(
            "Kotlin:interface:KotlinInterface",
            NodeKind::Interface,
            "KotlinInterface",
            "com.example.KotlinInterface",
            Language::Kotlin,
        );

        let kotlin_function = create_test_node(
            "Kotlin:function:KotlinFunction",
            NodeKind::Function,
            "KotlinFunction",
            "com.example.KotlinFunction",
            Language::Kotlin,
        );

        let mut deps = CrossLanguageDependencies::new();
        deps.add_nodes(vec![
            java_class,
            kotlin_base,
            kotlin_interface,
            kotlin_function,
        ]);
        deps.detect_all();

        let dot = deps.to_dot();

        // Check that different styles are used
        assert!(dot.contains("bold")); // Inherits
        assert!(dot.contains("dashed")); // Implements
        assert!(dot.contains("solid")); // Calls (and default)
    }

    // Test TypeScriptJavaResolver with short interface name
    #[test]
    fn test_typescript_interface_single_char_after_i() {
        let resolver = TypeScriptJavaResolver;

        let ts_node = create_test_node(
            "TypeScript:interface:IA",
            NodeKind::Interface,
            "IA",
            "IA",
            Language::TypeScript,
        );

        let java_node = create_test_node(
            "Java:class:A",
            NodeKind::Class,
            "A",
            "com.example.A",
            Language::Java,
        );

        let reference = crate::ast::polyglot::unified_node::NodeReference {
            kind: ReferenceKind::Uses,
            target_id: String::new(),
            target_name: "IA".to_string(),
            target_language: Some(Language::Java),
        };

        // IA -> A mapping should work
        assert!(resolver.can_resolve(
            Language::TypeScript,
            Language::Java,
            &ts_node,
            &reference,
            &java_node,
        ));
    }

    // Test JavaKotlinResolver with mismatched package depths
    #[test]
    fn test_java_kotlin_resolver_different_package_depth() {
        let resolver = JavaKotlinResolver;

        let java_node = create_test_node(
            "Java:class:User",
            NodeKind::Class,
            "User",
            "com.example.User",
            Language::Java,
        );

        let kotlin_node = create_test_node(
            "Kotlin:class:Service",
            NodeKind::Class,
            "Service",
            "com.example.api.Service",
            Language::Kotlin,
        );

        let reference = crate::ast::polyglot::unified_node::NodeReference {
            kind: ReferenceKind::Uses,
            target_id: String::new(),
            target_name: "com.example.Service".to_string(), // Different depth
            target_language: Some(Language::Kotlin),
        };

        // Should not match due to different package depths
        assert!(!resolver.can_resolve(
            Language::Java,
            Language::Kotlin,
            &java_node,
            &reference,
            &kotlin_node,
        ));
    }

    // Test with resolver that uses package parts comparison
    #[test]
    fn test_java_scala_resolver_package_parts_mismatch() {
        let resolver = JavaScalaResolver;

        let java_node = create_test_node(
            "Java:class:User",
            NodeKind::Class,
            "User",
            "com.example.User",
            Language::Java,
        );

        let scala_node = create_test_node(
            "Scala:class:DifferentService",
            NodeKind::Class,
            "DifferentService",
            "com.example.DifferentService",
            Language::Scala,
        );

        let reference = crate::ast::polyglot::unified_node::NodeReference {
            kind: ReferenceKind::Uses,
            target_id: String::new(),
            target_name: "com.example.Service".to_string(), // Different last part
            target_language: Some(Language::Scala),
        };

        // Should not match because class names differ
        assert!(!resolver.can_resolve(
            Language::Java,
            Language::Scala,
            &java_node,
            &reference,
            &scala_node,
        ));
    }

    // Test TypeScriptJavaResolver no I prefix
    #[test]
    fn test_typescript_java_no_interface_prefix() {
        let resolver = TypeScriptJavaResolver;

        let ts_node = create_test_node(
            "TypeScript:interface:UserInterface",
            NodeKind::Interface,
            "UserInterface",
            "UserInterface",
            Language::TypeScript,
        );

        let java_node = create_test_node(
            "Java:class:User",
            NodeKind::Class,
            "User",
            "com.example.User",
            Language::Java,
        );

        let reference = crate::ast::polyglot::unified_node::NodeReference {
            kind: ReferenceKind::Uses,
            target_id: String::new(),
            target_name: "UserInterface".to_string(),
            target_language: Some(Language::Java),
        };

        // Should not match - no I prefix
        assert!(!resolver.can_resolve(
            Language::TypeScript,
            Language::Java,
            &ts_node,
            &reference,
            &java_node,
        ));
    }

    // Test that CrossLanguageDependency struct fields are accessible
    #[test]
    fn test_cross_language_dependency_struct() {
        let dep = CrossLanguageDependency {
            source_id: "source".to_string(),
            target_id: "target".to_string(),
            source_language: Language::Java,
            target_language: Language::Kotlin,
            kind: ReferenceKind::Inherits,
            confidence: 0.95,
            metadata: {
                let mut m = HashMap::new();
                m.insert("key".to_string(), "value".to_string());
                m
            },
        };

        assert_eq!(dep.source_id, "source");
        assert_eq!(dep.target_id, "target");
        assert_eq!(dep.source_language, Language::Java);
        assert_eq!(dep.target_language, Language::Kotlin);
        assert_eq!(dep.kind, ReferenceKind::Inherits);
        assert!((dep.confidence - 0.95).abs() < f64::EPSILON);
        assert_eq!(dep.metadata.get("key"), Some(&"value".to_string()));
    }

    // Test Default trait implementation for CrossLanguageDependencies
    #[test]
    fn test_cross_language_dependencies_default() {
        let deps = CrossLanguageDependencies::default();
        assert!(deps.get_dependencies().is_empty());
    }

    // Test with name resolver integration
    #[test]
    fn test_detect_with_name_resolver_integration() {
        let mut deps = CrossLanguageDependencies::new();

        // Add Java->Kotlin resolver
        deps.add_name_resolver(Language::Java, Box::new(JavaKotlinResolver));

        // Create a Java node with reference
        let mut java_node = create_test_node(
            "Java:class:Client",
            NodeKind::Class,
            "Client",
            "com.example.Client",
            Language::Java,
        );

        // Reference using package pattern that the resolver handles
        let ref1 = crate::ast::polyglot::unified_node::NodeReference {
            kind: ReferenceKind::Uses,
            target_id: String::new(),
            target_name: "com.example.KotlinService".to_string(),
            target_language: Some(Language::Kotlin),
        };
        java_node.references.push(ref1);

        let kotlin_node = create_test_node(
            "Kotlin:class:KotlinService",
            NodeKind::Class,
            "KotlinService",
            "com.example.KotlinService",
            Language::Kotlin,
        );

        deps.add_nodes(vec![java_node, kotlin_node]);
        deps.detect_all();

        let result = deps.get_dependencies();
        assert!(!result.is_empty());
    }

    // Test with all ReferenceKind variants
    #[test]
    fn test_all_reference_kinds() {
        let all_kinds = [
            ReferenceKind::Inherits,
            ReferenceKind::Implements,
            ReferenceKind::Calls,
            ReferenceKind::Uses,
            ReferenceKind::Creates,
            ReferenceKind::Imports,
            ReferenceKind::Annotates,
            ReferenceKind::DependsOn,
        ];

        for kind in all_kinds {
            let mut java_node = create_test_node(
                &format!("Java:class:Source{:?}", kind),
                NodeKind::Class,
                &format!("Source{:?}", kind),
                &format!("com.example.Source{:?}", kind),
                Language::Java,
            );
            java_node.add_reference(kind, format!("Target{:?}", kind), None);

            let kotlin_node = create_test_node(
                &format!("Kotlin:class:Target{:?}", kind),
                NodeKind::Class,
                &format!("Target{:?}", kind),
                &format!("com.example.Target{:?}", kind),
                Language::Kotlin,
            );

            let mut deps = CrossLanguageDependencies::new();
            deps.add_nodes(vec![java_node, kotlin_node]);
            deps.detect_all();

            let filtered = deps.filter_by_kind(kind);
            assert!(
                !filtered.is_empty(),
                "Expected at least one dependency of kind {:?}",
                kind
            );
            assert_eq!(filtered[0].kind, kind);
        }
    }

    // Test FQN map building in add_nodes
    #[test]
    fn test_fqn_map_building() {
        let mut deps = CrossLanguageDependencies::new();

        // Create two nodes with same FQN but different IDs (simulating overloads)
        let node1 = create_test_node(
            "Java:method:process1",
            NodeKind::Method,
            "process",
            "com.example.Service.process",
            Language::Java,
        );

        let node2 = create_test_node(
            "Java:method:process2",
            NodeKind::Method,
            "process",
            "com.example.Service.process",
            Language::Java,
        );

        deps.add_nodes(vec![node1, node2]);

        // Both should be added and detectable
        let result = deps.detect_all();
        // No cross-language deps expected (same language)
        assert!(result.is_empty());
    }

    // Test nodes grouped by language correctly
    #[test]
    fn test_nodes_grouped_by_language() {
        let mut deps = CrossLanguageDependencies::new();

        let java_node = create_test_node(
            "Java:class:JavaClass",
            NodeKind::Class,
            "JavaClass",
            "com.example.JavaClass",
            Language::Java,
        );

        let kotlin_node = create_test_node(
            "Kotlin:class:KotlinClass",
            NodeKind::Class,
            "KotlinClass",
            "com.example.KotlinClass",
            Language::Kotlin,
        );

        let scala_node = create_test_node(
            "Scala:class:ScalaClass",
            NodeKind::Class,
            "ScalaClass",
            "com.example.ScalaClass",
            Language::Scala,
        );

        let ts_node = create_test_node(
            "TypeScript:class:TsClass",
            NodeKind::Class,
            "TsClass",
            "TsClass",
            Language::TypeScript,
        );

        deps.add_nodes(vec![java_node, kotlin_node, scala_node, ts_node]);
        deps.detect_all();

        // Verify we can filter by all languages
        let java_deps = deps.filter_by_source_language(Language::Java);
        let kotlin_deps = deps.filter_by_source_language(Language::Kotlin);
        let scala_deps = deps.filter_by_source_language(Language::Scala);
        let ts_deps = deps.filter_by_source_language(Language::TypeScript);

        // No references added, so all should be empty
        assert!(java_deps.is_empty());
        assert!(kotlin_deps.is_empty());
        assert!(scala_deps.is_empty());
        assert!(ts_deps.is_empty());
    }

    // Test DOT graph with no nodes
    #[test]
    fn test_to_dot_empty() {
        let deps = CrossLanguageDependencies::new();
        let dot = deps.to_dot();

        assert!(dot.starts_with("digraph CrossLanguageDependencies {"));
        assert!(dot.ends_with("}\n"));
        // No nodes or edges
        assert!(!dot.contains("->"));
    }

    // Test Clone and Debug for CrossLanguageDependency
    #[test]
    fn test_cross_language_dependency_clone_debug() {
        let dep = CrossLanguageDependency {
            source_id: "source".to_string(),
            target_id: "target".to_string(),
            source_language: Language::Java,
            target_language: Language::Kotlin,
            kind: ReferenceKind::Inherits,
            confidence: 1.0,
            metadata: HashMap::new(),
        };

        let cloned = dep.clone();
        assert_eq!(dep.source_id, cloned.source_id);
        assert_eq!(dep.target_id, cloned.target_id);

        // Test Debug
        let debug_str = format!("{:?}", dep);
        assert!(debug_str.contains("CrossLanguageDependency"));
        assert!(debug_str.contains("source"));
        assert!(debug_str.contains("target"));
    }

    // Test with deeply nested package names
    #[test]
    fn test_deeply_nested_packages() {
        let resolver = JavaKotlinResolver;

        let java_node = create_test_node(
            "Java:class:User",
            NodeKind::Class,
            "User",
            "com.example.api.v2.internal.User",
            Language::Java,
        );

        let kotlin_node = create_test_node(
            "Kotlin:class:Service",
            NodeKind::Class,
            "Service",
            "com.example.api.v2.internal.Service",
            Language::Kotlin,
        );

        let reference = crate::ast::polyglot::unified_node::NodeReference {
            kind: ReferenceKind::Uses,
            target_id: String::new(),
            target_name: "com.example.api.v2.internal.Service".to_string(),
            target_language: Some(Language::Kotlin),
        };

        assert!(resolver.can_resolve(
            Language::Java,
            Language::Kotlin,
            &java_node,
            &reference,
            &kotlin_node,
        ));
    }

    // Test serialization/deserialization of CrossLanguageDependency
    #[test]
    fn test_cross_language_dependency_serde() {
        let dep = CrossLanguageDependency {
            source_id: "source".to_string(),
            target_id: "target".to_string(),
            source_language: Language::Java,
            target_language: Language::Kotlin,
            kind: ReferenceKind::Inherits,
            confidence: 0.9,
            metadata: {
                let mut m = HashMap::new();
                m.insert("key".to_string(), "value".to_string());
                m
            },
        };

        let json = serde_json::to_string(&dep).unwrap();
        let deserialized: CrossLanguageDependency = serde_json::from_str(&json).unwrap();

        assert_eq!(dep.source_id, deserialized.source_id);
        assert_eq!(dep.target_id, deserialized.target_id);
        assert_eq!(dep.kind, deserialized.kind);
        assert!((dep.confidence - deserialized.confidence).abs() < f64::EPSILON);
    }
}

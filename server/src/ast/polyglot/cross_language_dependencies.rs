//! Cross-language dependency detection and analysis
//!
//! This module provides functionality to detect and analyze dependencies
//! between nodes in different programming languages. It can identify
//! relationships such as inheritance, implementation, and usage across
//! language boundaries.

use crate::ast::polyglot::{Language, NodeKind, UnifiedNode};
use crate::ast::polyglot::unified_node::ReferenceKind;
use std::collections::{HashMap, HashSet};
use serde::{Serialize, Deserialize};

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
                    if let (Some(node_ids1), Some(node_ids2)) = (nodes_by_language.get(lang1), nodes_by_language.get(lang2)) {
                        // Get dependencies between these language groups
                        let deps = self.detect_between_language_groups(node_ids1, *lang1, node_ids2, *lang2);
                        new_dependencies.extend(deps);
                    }
                }
            }
        }
        
        // Add the collected dependencies
        self.dependencies.extend(new_dependencies);
        
        // Resolve unresolved references (moved outside)
        self.resolve_references();
        
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
            name_map.entry(node.name.clone())
                .or_default()
                .push(id.clone());
                
            name_map.entry(node.fqn.clone())
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
                        reference.kind
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
            .filter(|dep| dep.source_language == source_language && dep.target_language == target_language)
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
                dep.source_id,
                dep.target_id,
                label,
                style
            ));
        }
        
        dot.push_str("}\n");
        dot
    }
    
    /// Get color for a language in DOT format
    fn language_color(&self, language: Language) -> &'static str {
        match language {
            Language::Java => "\"#b07219\"",      // Java brown
            Language::Kotlin => "\"#A97BFF\"",    // Kotlin purple
            Language::Scala => "\"#c22d40\"",     // Scala red
            Language::TypeScript => "\"#2b7489\"", // TypeScript blue
            Language::JavaScript => "\"#f1e05a\"", // JavaScript yellow
            Language::Python => "\"#3572A5\"",    // Python blue
            Language::Rust => "\"#dea584\"",      // Rust orange
            Language::Go => "\"#00ADD8\"",        // Go blue
            Language::Cpp => "\"#f34b7d\"",       // C++ pink
            _ => "\"#bbbbbb\"",                  // Gray for others
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
        if !((source_language == Language::Java && target_language == Language::Kotlin) ||
             (source_language == Language::Kotlin && target_language == Language::Java)) {
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
            if src_parts[0..src_parts.len()-1] == tgt_parts[0..tgt_parts.len()-1] {
                // Check if the last part (class/method name) matches
                if src_parts.last().unwrap() == tgt_parts.last().unwrap() {
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
        if !((source_language == Language::Java && target_language == Language::Scala) ||
             (source_language == Language::Scala && target_language == Language::Java)) {
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
            if src_parts[0..src_parts.len()-1] == tgt_parts[0..tgt_parts.len()-1] {
                // Check if the last part (class/method name) matches
                if src_parts.last().unwrap() == tgt_parts.last().unwrap() {
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
        if !((source_language == Language::TypeScript && target_language == Language::Java) ||
             (source_language == Language::Java && target_language == Language::TypeScript)) {
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
        if source_language == Language::TypeScript && 
           reference.target_name.starts_with('I') && 
           reference.target_name.len() > 1 {
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
        let dependencies = CrossLanguageDependencies::detect(&[java_class], &[kotlin_class]);

        // Verify
        assert_eq!(dependencies.len(), 1);
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
}
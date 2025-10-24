//! Polyglot analysis tools for MCP
//!
//! This module provides MCP tools for cross-language analysis, allowing
//! AI agents to detect and analyze relationships between different programming
//! languages in a project.

use crate::mcp_integration::{McpError, McpTool, ToolMetadata};
use crate::ast::polyglot::{
    Language, NodeKind, UnifiedNode, LanguageMapper, CrossLanguageDependencies,
    language_mapper::LanguageMapperFactory,
};
use crate::utils::path_validator::PathValidator;
use anyhow::Result;
use async_trait::async_trait;
use serde_json::{json, Value};
use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tokio::fs;

/// Analyzes cross-language relationships in a project
pub struct PolyglotAnalysisTool {
    agent_registry: Arc<crate::agents::registry::AgentRegistry>,
}

impl PolyglotAnalysisTool {
    /// Create a new polyglot analysis tool
    pub fn new(agent_registry: Arc<crate::agents::registry::AgentRegistry>) -> Self {
        Self { agent_registry }
    }
}

#[async_trait]
impl McpTool for PolyglotAnalysisTool {
    fn metadata(&self) -> ToolMetadata {
        ToolMetadata {
            name: "analyze_polyglot".to_string(),
            description: "Analyzes cross-language relationships in a project".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "path": {
                        "type": "string",
                        "description": "Path to directory to analyze"
                    },
                    "languages": {
                        "type": "array",
                        "items": {
                            "type": "string",
                            "enum": ["java", "kotlin", "scala", "typescript", "javascript"]
                        },
                        "description": "Languages to include (default: all supported)"
                    },
                    "max_depth": {
                        "type": "number",
                        "default": 3,
                        "description": "Maximum directory recursion depth"
                    },
                    "include_graph": {
                        "type": "boolean",
                        "default": true,
                        "description": "Include dependency graph in DOT format"
                    }
                },
                "required": ["path"]
            }),
        }
    }

    async fn execute(&self, params: Value) -> Result<Value, McpError> {
        // Extract parameters
        let path_str = params["path"]
            .as_str()
            .ok_or_else(|| McpError {
                code: crate::mcp_integration::error_codes::INVALID_PARAMS,
                message: "Missing path parameter".to_string(),
                data: None,
            })?;
            
        let path = PathBuf::from(path_str);
        let max_depth = params["max_depth"].as_u64().unwrap_or(3) as usize;
        let include_graph = params["include_graph"].as_bool().unwrap_or(true);
        
        // Validate path
        if !PathValidator::ensure_directory(&path).is_ok() {
            return Err(McpError {
                code: crate::mcp_integration::error_codes::INVALID_PARAMS,
                message: format!("Path is not a directory: {}", path.display()),
                data: Some(json!({
                    "path": path.display().to_string(),
                    "suggestion": "Please provide a valid directory path"
                })),
            });
        }
        
        // Parse languages to include
        let languages = if params["languages"].is_array() {
            let langs = params["languages"]
                .as_array()
                .unwrap()
                .iter()
                .filter_map(|l| l.as_str())
                .filter_map(|l| match l.to_lowercase().as_str() {
                    "java" => Some(Language::Java),
                    "kotlin" => Some(Language::Kotlin),
                    "scala" => Some(Language::Scala),
                    "typescript" => Some(Language::TypeScript),
                    "javascript" => Some(Language::JavaScript),
                    _ => None,
                })
                .collect::<Vec<_>>();
                
            if langs.is_empty() {
                vec![
                    Language::Java,
                    Language::Kotlin,
                    Language::Scala,
                    Language::TypeScript,
                    Language::JavaScript,
                ]
            } else {
                langs
            }
        } else {
            // Default to all supported languages
            vec![
                Language::Java,
                Language::Kotlin,
                Language::Scala,
                Language::TypeScript,
                Language::JavaScript,
            ]
        };
        
        // Analyze the directory for each language
        let mut language_nodes: HashMap<Language, Vec<UnifiedNode>> = HashMap::new();
        let mut all_nodes = Vec::new();
        
        for language in &languages {
            match LanguageMapperFactory::create(*language) {
                Ok(mapper) => {
                    match mapper.map_directory(&path, max_depth > 0).await {
                        Ok(nodes) => {
                            language_nodes.insert(*language, nodes.clone());
                            all_nodes.extend(nodes);
                        },
                        Err(e) => {
                            tracing::warn!("Error mapping {:?} files: {}", language, e);
                        }
                    }
                },
                Err(e) => {
                    tracing::warn!("Error creating mapper for {:?}: {}", language, e);
                }
            }
        }
        
        // Detect cross-language dependencies
        let mut dependencies = CrossLanguageDependencies::new();
        dependencies.add_nodes(all_nodes.clone());
        dependencies.detect_all();
        
        // Build response
        let mut result = json!({
            "status": "completed",
            "path": path.display().to_string(),
            "languages": languages.iter().map(|l| l.name()).collect::<Vec<_>>(),
            "summary": {
                "total_files": language_nodes.values().map(|nodes| {
                    nodes.iter()
                        .map(|n| n.file_path.clone())
                        .collect::<HashSet<_>>()
                        .len()
                }).sum::<usize>(),
                "total_nodes": all_nodes.len(),
                "nodes_by_language": language_nodes.iter()
                    .map(|(lang, nodes)| (lang.name(), nodes.len()))
                    .collect::<HashMap<_, _>>(),
                "total_cross_language_dependencies": dependencies.get_dependencies().len()
            }
        });
        
        // Add detailed node counts by type
        let node_counts = get_node_type_counts(&all_nodes);
        result["node_counts"] = json!(node_counts);
        
        // Add dependency information
        let deps = dependencies.get_dependencies();
        let mut dep_counts: HashMap<String, usize> = HashMap::new();
        
        for dep in deps {
            let key = format!(
                "{} -> {}",
                dep.source_language.name(),
                dep.target_language.name()
            );
            *dep_counts.entry(key).or_insert(0) += 1;
        }
        
        result["dependency_counts"] = json!(dep_counts);
        
        // Add detailed dependencies
        let mut detailed_deps = Vec::new();
        for dep in deps {
            let source_node = all_nodes.iter()
                .find(|n| n.id == dep.source_id)
                .map(|n| json!({
                    "id": n.id,
                    "name": n.name,
                    "fqn": n.fqn,
                    "kind": n.kind.as_str()
                }))
                .unwrap_or_else(|| json!({"id": dep.source_id}));
                
            let target_node = all_nodes.iter()
                .find(|n| n.id == dep.target_id)
                .map(|n| json!({
                    "id": n.id,
                    "name": n.name,
                    "fqn": n.fqn,
                    "kind": n.kind.as_str()
                }))
                .unwrap_or_else(|| json!({"id": dep.target_id}));
                
            detailed_deps.push(json!({
                "source": source_node,
                "target": target_node,
                "kind": format!("{:?}", dep.kind),
                "source_language": dep.source_language.name(),
                "target_language": dep.target_language.name(),
                "confidence": dep.confidence
            }));
        }
        
        result["dependencies"] = json!(detailed_deps);
        
        // Add graph if requested
        if include_graph {
            result["graph_dot"] = json!(dependencies.to_dot());
        }
        
        Ok(result)
    }
}

/// Get counts of node types by language
fn get_node_type_counts(nodes: &[UnifiedNode]) -> HashMap<String, HashMap<String, usize>> {
    let mut counts = HashMap::new();
    
    for node in nodes {
        let lang_name = node.language.name().to_string();
        let kind_name = node.kind.as_str().to_string();
        
        counts
            .entry(lang_name)
            .or_insert_with(HashMap::new)
            .entry(kind_name)
            .and_modify(|c| *c += 1)
            .or_insert(1);
    }
    
    counts
}

/// Detects language boundaries in a project
pub struct LanguageBoundaryTool {
    agent_registry: Arc<crate::agents::registry::AgentRegistry>,
}

impl LanguageBoundaryTool {
    /// Create a new language boundary tool
    pub fn new(agent_registry: Arc<crate::agents::registry::AgentRegistry>) -> Self {
        Self { agent_registry }
    }
}

#[async_trait]
impl McpTool for LanguageBoundaryTool {
    fn metadata(&self) -> ToolMetadata {
        ToolMetadata {
            name: "detect_language_boundaries".to_string(),
            description: "Detects language boundaries and interoperability points in a project".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "path": {
                        "type": "string",
                        "description": "Path to directory to analyze"
                    },
                    "source_language": {
                        "type": "string",
                        "description": "Source language to analyze boundaries from (optional)"
                    },
                    "target_language": {
                        "type": "string",
                        "description": "Target language to analyze boundaries to (optional)"
                    },
                    "max_depth": {
                        "type": "number",
                        "default": 3,
                        "description": "Maximum directory recursion depth"
                    }
                },
                "required": ["path"]
            }),
        }
    }

    async fn execute(&self, params: Value) -> Result<Value, McpError> {
        // Extract parameters
        let path_str = params["path"]
            .as_str()
            .ok_or_else(|| McpError {
                code: crate::mcp_integration::error_codes::INVALID_PARAMS,
                message: "Missing path parameter".to_string(),
                data: None,
            })?;
            
        let path = PathBuf::from(path_str);
        let max_depth = params["max_depth"].as_u64().unwrap_or(3) as usize;
        
        // Validate path
        if !PathValidator::ensure_directory(&path).is_ok() {
            return Err(McpError {
                code: crate::mcp_integration::error_codes::INVALID_PARAMS,
                message: format!("Path is not a directory: {}", path.display()),
                data: Some(json!({
                    "path": path.display().to_string(),
                    "suggestion": "Please provide a valid directory path"
                })),
            });
        }
        
        // Parse source and target languages
        let source_language = params["source_language"]
            .as_str()
            .and_then(|l| match l.to_lowercase().as_str() {
                "java" => Some(Language::Java),
                "kotlin" => Some(Language::Kotlin),
                "scala" => Some(Language::Scala),
                "typescript" => Some(Language::TypeScript),
                "javascript" => Some(Language::JavaScript),
                _ => None,
            });
            
        let target_language = params["target_language"]
            .as_str()
            .and_then(|l| match l.to_lowercase().as_str() {
                "java" => Some(Language::Java),
                "kotlin" => Some(Language::Kotlin),
                "scala" => Some(Language::Scala),
                "typescript" => Some(Language::TypeScript),
                "javascript" => Some(Language::JavaScript),
                _ => None,
            });
        
        // Set up languages to analyze based on parameters
        let languages = if source_language.is_none() && target_language.is_none() {
            // Analyze all supported languages if no specific ones were requested
            vec![
                Language::Java,
                Language::Kotlin,
                Language::Scala,
                Language::TypeScript,
                Language::JavaScript,
            ]
        } else {
            // Only analyze specified languages
            let mut langs = Vec::new();
            if let Some(lang) = source_language {
                langs.push(lang);
            }
            if let Some(lang) = target_language {
                if !langs.contains(&lang) {
                    langs.push(lang);
                }
            }
            langs
        };
        
        // Analyze the directory for each language
        let mut language_nodes: HashMap<Language, Vec<UnifiedNode>> = HashMap::new();
        let mut all_nodes = Vec::new();
        
        for language in &languages {
            match LanguageMapperFactory::create(*language) {
                Ok(mapper) => {
                    match mapper.map_directory(&path, max_depth > 0).await {
                        Ok(nodes) => {
                            language_nodes.insert(*language, nodes.clone());
                            all_nodes.extend(nodes);
                        },
                        Err(e) => {
                            tracing::warn!("Error mapping {:?} files: {}", language, e);
                        }
                    }
                },
                Err(e) => {
                    tracing::warn!("Error creating mapper for {:?}: {}", language, e);
                }
            }
        }
        
        // Detect cross-language dependencies
        let mut dependencies = CrossLanguageDependencies::new();
        dependencies.add_nodes(all_nodes.clone());
        dependencies.detect_all();
        
        // Filter dependencies based on source/target language if specified
        let deps = dependencies.get_dependencies();
        
        let filtered_deps = deps.iter()
            .filter(|dep| {
                let source_match = source_language.map(|l| dep.source_language == l).unwrap_or(true);
                let target_match = target_language.map(|l| dep.target_language == l).unwrap_or(true);
                source_match && target_match
            })
            .collect::<Vec<_>>();
            
        // Build response
        let mut result = json!({
            "status": "completed",
            "path": path.display().to_string(),
            "languages_analyzed": languages.iter().map(|l| l.name()).collect::<Vec<_>>(),
            "summary": {
                "total_boundaries": filtered_deps.len(),
                "source_language": source_language.map(|l| l.name()),
                "target_language": target_language.map(|l| l.name()),
            }
        });
        
        // Add boundary information
        let mut boundaries = Vec::new();
        for dep in &filtered_deps {
            let source_node = all_nodes.iter()
                .find(|n| n.id == dep.source_id)
                .map(|n| json!({
                    "id": n.id,
                    "name": n.name,
                    "fqn": n.fqn,
                    "kind": n.kind.as_str(),
                    "file": n.file_path.display().to_string()
                }))
                .unwrap_or_else(|| json!({"id": dep.source_id}));
                
            let target_node = all_nodes.iter()
                .find(|n| n.id == dep.target_id)
                .map(|n| json!({
                    "id": n.id,
                    "name": n.name,
                    "fqn": n.fqn,
                    "kind": n.kind.as_str(),
                    "file": n.file_path.display().to_string()
                }))
                .unwrap_or_else(|| json!({"id": dep.target_id}));
                
            boundaries.push(json!({
                "boundary_type": format!("{:?}", dep.kind),
                "source": {
                    "language": dep.source_language.name(),
                    "node": source_node
                },
                "target": {
                    "language": dep.target_language.name(),
                    "node": target_node
                },
                "confidence": dep.confidence
            }));
        }
        
        result["boundaries"] = json!(boundaries);
        
        // Group boundaries by type
        let mut grouped_boundaries = HashMap::new();
        for dep in &filtered_deps {
            let key = format!("{:?}", dep.kind);
            grouped_boundaries
                .entry(key)
                .or_insert_with(Vec::new)
                .push(dep);
        }
        
        let mut boundary_stats = json!({});
        for (kind, deps) in &grouped_boundaries {
            boundary_stats[kind] = json!({
                "count": deps.len(),
                "languages": deps.iter()
                    .map(|d| format!("{} → {}", d.source_language.name(), d.target_language.name()))
                    .collect::<HashSet<_>>()
            });
        }
        
        result["boundary_types"] = boundary_stats;
        
        // Add common patterns and recommendations
        result["patterns"] = analyze_boundary_patterns(filtered_deps, &all_nodes);
        
        Ok(result)
    }
}

/// Analyze patterns in language boundaries
fn analyze_boundary_patterns(
    deps: Vec<&crate::ast::polyglot::cross_language_dependencies::CrossLanguageDependency>,
    nodes: &[UnifiedNode],
) -> Value {
    let mut patterns = Vec::new();
    
    // Group boundaries by language pairs
    let mut language_pairs = HashMap::new();
    for dep in &deps {
        let key = format!("{}-{}", dep.source_language.name(), dep.target_language.name());
        language_pairs
            .entry(key)
            .or_insert_with(Vec::new)
            .push(*dep);
    }
    
    // Analyze each language pair
    for (pair, deps) in language_pairs {
        let mut pattern = json!({
            "language_pair": pair,
            "count": deps.len()
        });
        
        let parts: Vec<&str> = pair.split('-').collect();
        if parts.len() == 2 {
            match (parts[0], parts[1]) {
                ("Java", "Kotlin") | ("Kotlin", "Java") => {
                    pattern["recommendations"] = json!([
                        "Use Kotlin's @JvmName annotation to control Java-visible names",
                        "Leverage Kotlin extension functions for Java interoperability",
                        "Use Kotlin's nullable types consistently with Java's @Nullable",
                        "Consider avoiding Kotlin-specific features at boundaries (coroutines, delegation)"
                    ]);
                },
                ("Java", "Scala") | ("Scala", "Java") => {
                    pattern["recommendations"] = json!([
                        "Prefer Java interfaces at language boundaries",
                        "Be careful with Scala's implicit conversions at Java boundaries",
                        "Avoid using Scala's case classes as Java API",
                        "Use Java collections when sharing data between Java and Scala"
                    ]);
                },
                ("TypeScript", "JavaScript") | ("JavaScript", "TypeScript") => {
                    pattern["recommendations"] = json!([
                        "Use TypeScript declaration files (.d.ts) for JavaScript modules",
                        "Add JSDoc comments to JavaScript for TypeScript type inference",
                        "Consider migrating to pure TypeScript gradually",
                        "Use ES modules format for better interoperability"
                    ]);
                },
                ("Java", "TypeScript") | ("TypeScript", "Java") => {
                    pattern["recommendations"] = json!([
                        "Use consistent naming conventions across both languages",
                        "Define API contracts with OpenAPI/Swagger for REST interfaces",
                        "Consider type-safe approaches like GraphQL or gRPC",
                        "Enforce model consistency with shared schemas"
                    ]);
                },
                _ => {
                    // Generic recommendations
                    pattern["recommendations"] = json!([
                        "Define clear API contracts between languages",
                        "Use consistent naming conventions",
                        "Minimize direct cross-language dependencies",
                        "Consider using an interface language (API specs, proto files, etc.)"
                    ]);
                }
            }
        }
        
        patterns.push(pattern);
    }
    
    json!(patterns)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ast::polyglot::unified_node::{SourcePosition, ReferenceKind as PolyglotReferenceKind};
    
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
    
    #[tokio::test]
    #[ignore] // Ignoring since it requires file system access
    async fn test_analyze_boundary_patterns() {
        // Create some test dependencies
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
        
        // Create a cross-language dependency
        let dependency = crate::ast::polyglot::cross_language_dependencies::CrossLanguageDependency {
            source_id: java_node.id.clone(),
            target_id: kotlin_node.id.clone(),
            source_language: Language::Java,
            target_language: Language::Kotlin,
            kind: PolyglotReferenceKind::Inherits,
            confidence: 1.0,
            metadata: HashMap::new(),
        };
        
        // Test analyzing patterns
        let nodes = vec![java_node, kotlin_node];
        let deps = vec![&dependency];
        
        let patterns = analyze_boundary_patterns(deps, &nodes);
        
        // Verify results
        assert!(patterns.is_array());
        assert_eq!(patterns.as_array().unwrap().len(), 1);
        
        let first_pattern = &patterns.as_array().unwrap()[0];
        assert_eq!(first_pattern["language_pair"], "Java-Kotlin");
        assert!(first_pattern["recommendations"].is_array());
        assert!(first_pattern["recommendations"].as_array().unwrap().len() > 0);
    }
}
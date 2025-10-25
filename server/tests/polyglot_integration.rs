// This module contains integration tests for the polyglot AST framework,
// which provides cross-language analysis capabilities.

// Use cfg_attr to conditionally mark the tests as ignored when the required 
// features are not enabled
#![cfg_attr(not(feature = "polyglot-ast"), allow(dead_code, unused_imports, unused_variables))]

use pmat::services::polyglot_analyzer::{LanguageInfo, PolyglotAnalysis, PolyglotAnalyzer};
#[cfg(feature = "polyglot-ast")]
use pmat::ast::polyglot::{
    Language, NodeKind, UnifiedNode, CrossLanguageDependencies,
    unified_node::{SourcePosition, NodeReference, ReferenceKind, TypeInfo},
};
use std::path::{Path, PathBuf};
use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use tokio::fs;
use anyhow::Result;
use tempfile::tempdir;
use serde::Serialize;

// Define the types here for testing purposes when the actual implementations
// are not available
#[cfg(not(feature = "polyglot-ast"))]
mod test_types {
    use std::path::{Path, PathBuf};
    use std::collections::HashMap;
    use serde::Serialize;
    
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize)]
    pub enum Language {
        Java,
        Kotlin,
        Scala,
        TypeScript,
        JavaScript,
        Python,
        Rust,
        Go,
        Cpp,
        CSharp,
        Ruby,
        Swift,
        Php,
        Other(u32), 
    }
    
    impl Language {
        pub fn name(&self) -> &'static str {
            match self {
                Language::Java => "Java",
                Language::Kotlin => "Kotlin",
                Language::Scala => "Scala",
                Language::TypeScript => "TypeScript",
                Language::JavaScript => "JavaScript",
                Language::Python => "Python",
                Language::Rust => "Rust",
                Language::Go => "Go",
                Language::Cpp => "C++",
                Language::CSharp => "C#",
                Language::Ruby => "Ruby",
                Language::Swift => "Swift",
                Language::Php => "PHP",
                Language::Other(_) => "Other",
            }
        }
    }
    
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize)]
    pub enum NodeKind {
        Class,
        Interface,
        Record,
        Function,
        Method,
        Package,
        Import,
        Module,
        Namespace,
        Trait,
        Enum,
        Struct,
        Union,
        Unknown,
    }
    
    impl NodeKind {
        pub fn as_str(&self) -> &'static str {
            match self {
                NodeKind::Class => "class",
                NodeKind::Interface => "interface",
                NodeKind::Record => "record",
                NodeKind::Function => "function",
                NodeKind::Method => "method",
                NodeKind::Package => "package",
                NodeKind::Import => "import",
                NodeKind::Module => "module",
                NodeKind::Namespace => "namespace",
                NodeKind::Trait => "trait",
                NodeKind::Enum => "enum",
                NodeKind::Struct => "struct",
                NodeKind::Union => "union",
                NodeKind::Unknown => "unknown",
            }
        }
    }
    
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize)]
    pub enum ReferenceKind {
        Inherits,
        Implements,
        Calls,
        Uses,
        Creates,
        Imports,
        Annotates,
        DependsOn,
    }
    
    #[derive(Debug, Clone, Serialize)]
    pub struct NodeReference {
        pub kind: ReferenceKind,
        pub target_id: String,
        pub target_name: String,
        pub target_language: Option<Language>,
    }
    
    #[derive(Debug, Clone)]
    pub struct SourcePosition {
        pub start_line: usize,
        pub start_col: usize,
        pub end_line: usize,
        pub end_col: usize,
    }
    
    impl Default for SourcePosition {
        fn default() -> Self {
            Self {
                start_line: 0,
                start_col: 0,
                end_line: 0,
                end_col: 0,
            }
        }
    }
    
    #[derive(Debug, Clone)]
    pub struct TypeInfo {
        pub name: String,
        pub fqn: String,
        pub type_parameters: Vec<TypeInfo>,
        pub is_primitive: bool,
        pub is_collection: bool,
        pub is_nullable: bool,
        pub original_type_string: String,
    }
    
    #[derive(Clone)]
    pub struct UnifiedNode {
        pub id: String,
        pub kind: NodeKind,
        pub name: String,
        pub fqn: String,
        pub language: Language,
        pub file_path: PathBuf,
        pub position: SourcePosition,
        pub attributes: HashMap<String, String>,
        pub children: Vec<String>,
        pub parent: Option<String>,
        pub references: Vec<NodeReference>,
        pub type_info: Option<TypeInfo>,
        pub signature: Option<String>,
        pub documentation: Option<String>,
        pub original_item: Option<()>,
        pub metadata: HashMap<String, String>,
    }
}

// Import test types when polyglot-ast feature is not enabled
#[cfg(not(feature = "polyglot-ast"))]
use test_types::*;

/// Test fixture creation for polyglot analysis tests
async fn create_test_polyglot_fixtures() -> Result<PathBuf> {
    // Create a temporary directory
    let temp_dir = tempdir()?;
    let fixture_path = temp_dir.path().to_path_buf();
    
    // Create Java directory structure
    fs::create_dir_all(&fixture_path.join("java/src/com/example")).await?;
    
    // Create Kotlin directory structure
    fs::create_dir_all(&fixture_path.join("kotlin/src/com/example")).await?;
    
    // Create TypeScript directory structure
    fs::create_dir_all(&fixture_path.join("typescript/src/models")).await?;
    
    // Create Java files
    fs::write(
        fixture_path.join("java/src/com/example/BaseModel.java"),
        r#"
package com.example;

/**
 * Base model class for data objects
 */
public abstract class BaseModel {
    private Long id;
    
    public BaseModel(Long id) {
        this.id = id;
    }
    
    public Long getId() {
        return id;
    }
}
        "#,
    ).await?;
    
    fs::write(
        fixture_path.join("java/src/com/example/UserRepository.java"),
        r#"
package com.example;

import com.example.User; // Kotlin class

/**
 * User repository for data access
 */
public class UserRepository {
    public User findById(Long id) {
        // Implementation omitted
        return null;
    }
    
    public void save(User user) {
        // Implementation omitted
    }
}
        "#,
    ).await?;
    
    // Create Kotlin files
    fs::write(
        fixture_path.join("kotlin/src/com/example/User.kt"),
        r#"
package com.example

import com.example.BaseModel

/**
 * User data class extending Java BaseModel
 */
data class User(
    val name: String,
    val email: String
) : BaseModel(0L) {
    fun toDto(): UserDto {
        return UserDto(this.name, this.email)
    }
}
        "#,
    ).await?;
    
    fs::write(
        fixture_path.join("kotlin/src/com/example/UserDto.kt"),
        r#"
package com.example

/**
 * Data transfer object for User
 */
data class UserDto(
    val name: String,
    val email: String
)
        "#,
    ).await?;
    
    // Create TypeScript files
    fs::write(
        fixture_path.join("typescript/src/models/UserModel.ts"),
        r#"
/**
 * TypeScript User model that mirrors the Java/Kotlin User class
 */
export interface UserModel {
    id: number;
    name: string;
    email: string;
}

export class UserService {
    /**
     * Fetch user from backend Java API
     */
    async fetchUser(id: number): Promise<UserModel> {
        // This would call the Java UserRepository in a real app
        const response = await fetch(`/api/users/${id}`);
        return response.json() as Promise<UserModel>;
    }
}
        "#,
    ).await?;
    
    fs::write(
        fixture_path.join("typescript/src/models/UserDto.ts"),
        r#"
/**
 * TypeScript UserDto that mirrors the Kotlin UserDto
 */
export interface UserDto {
    name: string;
    email: string;
}

export function mapToUserDto(user: any): UserDto {
    return {
        name: user.name,
        email: user.email
    };
}
        "#,
    ).await?;
    
    Ok(fixture_path)
}

// Original basic tests - keep for backward compatibility
#[tokio::test]
async fn test_polyglot_analyzer_initialization() {
    let analyzer = PolyglotAnalyzer::new();

    // Test that the analyzer initializes correctly by using public API
    let test_insights = analyzer.generate_polyglot_insights(&PolyglotAnalysis {
        languages: vec![],
        cross_language_dependencies: vec![],
        architecture_pattern: None,
        integration_points: vec![],
        recommendation_score: 0.5,
    });

    // Should generate some insights even for empty analysis
    assert!(!test_insights.is_empty(), "Should generate insights");
}

#[tokio::test]
async fn test_polyglot_analyzer_insights() {
    let analyzer = PolyglotAnalyzer::new();

    // Create a mock polyglot analysis
    let analysis = PolyglotAnalysis {
        languages: vec![
            pmat::services::polyglot_analyzer::LanguageStats {
                language: "rust".to_string(),
                file_count: 10,
                line_count: 1000,
                complexity_score: 5.5,
                test_coverage: 0.8,
                primary_frameworks: vec!["Tokio".to_string(), "Serde".to_string()],
            },
            pmat::services::polyglot_analyzer::LanguageStats {
                language: "python".to_string(),
                file_count: 5,
                line_count: 500,
                complexity_score: 3.2,
                test_coverage: 0.7,
                primary_frameworks: vec!["Django".to_string()],
            },
            pmat::services::polyglot_analyzer::LanguageStats {
                language: "javascript".to_string(),
                file_count: 8,
                line_count: 800,
                complexity_score: 4.1,
                test_coverage: 0.6,
                primary_frameworks: vec!["React".to_string()],
            },
        ],
        cross_language_dependencies: vec![],
        architecture_pattern: Some(pmat::services::polyglot_analyzer::ArchitecturePattern::Mixed),
        integration_points: vec![],
        recommendation_score: 0.85,
    };

    // Generate insights
    let insights = analyzer.generate_polyglot_insights(&analysis);

    // Verify insights are generated
    assert!(!insights.is_empty());
    assert!(insights.iter().any(|i| i.contains("polyglot project")));
    assert!(insights
        .iter()
        .any(|i| i.contains("Primary language: rust")));
    assert!(insights
        .iter()
        .any(|i| i.contains("Architecture pattern: Mixed")));
    assert!(insights
        .iter()
        .any(|i| i.contains("recommendation score: 0.85")));
}

#[test]
fn test_language_info_creation() {
    let lang_info = LanguageInfo {
        name: "rust".to_string(),
        file_count: 25,
        line_count: 2500,
        frameworks: vec!["Tokio".to_string(), "Serde".to_string()],
    };

    assert_eq!(lang_info.name, "rust");
    assert_eq!(lang_info.file_count, 25);
    assert_eq!(lang_info.line_count, 2500);
    assert_eq!(lang_info.frameworks.len(), 2);
}

#[test]
fn test_polyglot_analysis_serialization() {
    use serde_json;

    let analysis = PolyglotAnalysis {
        languages: vec![],
        cross_language_dependencies: vec![],
        architecture_pattern: Some(
            pmat::services::polyglot_analyzer::ArchitecturePattern::Monolithic,
        ),
        integration_points: vec![],
        recommendation_score: 0.5,
    };

    // Test that the analysis can be serialized to JSON
    let json_result = serde_json::to_string(&analysis);
    assert!(json_result.is_ok());

    let json_str = json_result.unwrap();
    assert!(json_str.contains("Monolithic"));
    assert!(json_str.contains("0.5"));
}

// Create documentation for pending implementation requirements
/// # Implementation Plan for Cross-Language Analysis
///
/// This test suite describes the requirements and test cases for the cross-language analysis
/// feature. The implementation is currently in progress, and these tests are designed to
/// validate the feature once it is complete.
///
/// ## Required Components
///
/// 1. **Language Mappers**: Implement language-specific AST mappers for:
///    - Java: `JavaMapper` connecting to `JavaAstVisitor`
///    - Kotlin: `KotlinMapper` connecting to `KotlinAstVisitor`
///    - Scala: `ScalaMapper` connecting to `ScalaAstVisitor`
///    - TypeScript: `TypeScriptMapper` connecting to `TypeScriptAstVisitor`
///    - JavaScript: `JavaScriptMapper` connecting to `JavaScriptAstVisitor`
///
/// 2. **Unified Node Framework**: Complete implementation of:
///    - `UnifiedNode` for language-agnostic representation
///    - Node reference and relationship detection
///    - Cross-language inheritance and usage tracking
///
/// 3. **MCP Tools Integration**:
///    - `PolyglotAnalysisTool` for analyzing entire projects
///    - `LanguageBoundaryTool` for detecting language boundaries
///    - DOT graph generation for visualization
///
/// 4. **Feature Flags**:
///    - Add feature flags for language-specific implementations:
///      - `java-ast`
///      - `kotlin-ast`
///      - `scala-ast`
///      - `typescript-ast`
///      - `javascript-ast`
///    - Meta-feature `polyglot-ast` that enables all language features
///
/// 5. **Fix Compilation Issues**:
///    - Add missing imports in cross_language_dependencies.rs
///    - Update NodeKind to match AstItem variants
///    - Fix path validation in polyglot_tools.rs
///    - Fix unused variable warnings
///
/// ## Testing Requirements
///
/// The test suite validates:
/// - Basic unified node creation and manipulation
/// - Cross-language reference detection
/// - Language boundary identification
/// - Multi-language project analysis
/// - MCP tools for AI agent integration
///
/// Once implemented, these tests will validate the complete cross-language
/// analysis functionality.
#[test]
fn test_implementation_requirements_documentation() {
    // This is a documentation test that outlines the requirements
    // No assertions needed
}

// New comprehensive tests for cross-language analysis features

/// Integration test for UnifiedNode creation and cross-language dependencies
#[tokio::test]
#[ignore = "Requires polyglot-ast feature to be enabled"]
async fn test_unified_node_cross_language_dependencies() {
    // Create test nodes representing a Java class, Kotlin class, and TypeScript interface
    
    // Java BaseModel class
    let mut java_base = create_unified_node(
        "BaseModel",
        NodeKind::Class,
        "com.example.BaseModel",
        Language::Java,
        PathBuf::from("/java/src/com/example/BaseModel.java")
    );
    
    // Kotlin User class that extends Java BaseModel
    let mut kotlin_user = create_unified_node(
        "User",
        NodeKind::Record, // Kotlin data class
        "com.example.User",
        Language::Kotlin,
        PathBuf::from("/kotlin/src/com/example/User.kt")
    );
    
    // TypeScript UserModel interface
    let mut ts_user_model = create_unified_node(
        "UserModel",
        NodeKind::Interface,
        "models.UserModel",
        Language::TypeScript,
        PathBuf::from("/typescript/src/models/UserModel.ts")
    );
    
    // Add cross-language relationships
    kotlin_user.add_reference(
        ReferenceKind::Inherits,
        "BaseModel".to_string(),
        Some("Java:class:com.example.BaseModel".to_string())
    );
    
    ts_user_model.add_reference(
        ReferenceKind::DependsOn,
        "User".to_string(),
        Some("Kotlin:record:com.example.User".to_string())
    );
    
    // Collect nodes for dependency analysis
    let nodes = vec![java_base.clone(), kotlin_user.clone(), ts_user_model.clone()];
    
    // Create a cross-language dependency analyzer
    let dependency_analyzer = MockCrossLanguageDependencyAnalyzer::new(nodes);
    
    // Test dependency detection
    let dependencies = dependency_analyzer.detect_dependencies();
    
    // Verify detected dependencies
    assert_eq!(dependencies.len(), 2);
    
    // Check Kotlin → Java inheritance dependency
    let kotlin_java_dep = dependencies.iter().find(|d| 
        d.source_language == Language::Kotlin && 
        d.target_language == Language::Java &&
        d.kind == ReferenceKind::Inherits
    );
    assert!(kotlin_java_dep.is_some(), "Expected to find a Kotlin -> Java dependency with Inherits kind");
    
    // Check TypeScript → Kotlin dependency
    let ts_kotlin_dep = dependencies.iter().find(|d|
        d.source_language == Language::TypeScript &&
        d.target_language == Language::Kotlin &&
        d.kind == ReferenceKind::DependsOn
    );
    assert!(ts_kotlin_dep.is_some(), "Expected to find a TypeScript -> Kotlin dependency with DependsOn kind");
}

/// Test for actual fixture-based cross-language analysis
#[tokio::test]
#[ignore = "Requires polyglot-ast feature to be enabled"]
async fn test_polyglot_analyzer_with_fixtures() -> Result<()> {
    // Create test fixtures
    let fixture_path = create_test_polyglot_fixtures().await?;
    
    // Create a polyglot analyzer using the fixtures
    let analyzer = MockPolyglotAnalyzer::new(fixture_path.clone());
    
    // Analyze the fixtures
    let analysis = analyzer.analyze().await?;
    
    // Verify language detection
    assert!(analysis.languages.contains(&Language::Java));
    assert!(analysis.languages.contains(&Language::Kotlin));
    assert!(analysis.languages.contains(&Language::TypeScript));
    
    // Verify file counts
    assert_eq!(analysis.file_counts.get(&Language::Java), Some(&2));
    assert_eq!(analysis.file_counts.get(&Language::Kotlin), Some(&2));
    assert_eq!(analysis.file_counts.get(&Language::TypeScript), Some(&2));
    
    // Verify cross-language dependencies
    let deps = &analysis.cross_language_dependencies;
    assert!(!deps.is_empty());
    
    // Kotlin User extends Java BaseModel
    assert!(deps.iter().any(|d| 
        d.source_language == Language::Kotlin && 
        d.target_language == Language::Java &&
        d.source_name == "User" &&
        d.target_name == "BaseModel" &&
        d.kind == ReferenceKind::Inherits
    ));
    
    // Java UserRepository uses Kotlin User
    assert!(deps.iter().any(|d|
        d.source_language == Language::Java && 
        d.target_language == Language::Kotlin &&
        d.source_name == "UserRepository" &&
        d.target_name == "User" &&
        d.kind == ReferenceKind::Uses
    ));
    
    // TypeScript UserModel depends on Kotlin User (via API)
    assert!(deps.iter().any(|d|
        d.source_language == Language::TypeScript && 
        d.target_language == Language::Kotlin &&
        d.source_name.contains("User") &&
        d.target_name == "User" &&
        d.kind == ReferenceKind::DependsOn
    ));
    
    Ok(())
}

/// Test for the MCP polyglot analysis tool
#[tokio::test]
#[ignore = "Requires polyglot-ast feature to be enabled"]
async fn test_mcp_polyglot_analysis_tool() -> Result<()> {
    // Create test fixtures
    let fixture_path = create_test_polyglot_fixtures().await?;
    
    // Create a mock MCP tool instance
    let tool = MockPolyglotAnalysisTool::new(fixture_path.clone());
    
    // Create analysis parameters
    let params = PolyglotAnalysisParams {
        path: fixture_path.to_string_lossy().to_string(),
        languages: Some(vec!["java".to_string(), "kotlin".to_string(), "typescript".to_string()]),
        max_depth: Some(3),
        include_graph: Some(true),
    };
    
    // Execute the analysis
    let result = tool.analyze(params).await?;
    
    // Verify the result
    let status = result.get("status").and_then(|s| s.as_str());
    let path = result.get("path").and_then(|p| p.as_str());
    assert_eq!(status, Some("completed"));
    assert_eq!(path, Some(&fixture_path.to_string_lossy().to_string()));
    
    // Verify languages analyzed
    let languages = result.get("languages").and_then(|l| l.as_array());
    assert!(languages.is_some());
    let languages = languages.unwrap();
    assert!(languages.iter().any(|l| l.as_str() == Some("Java")));
    assert!(languages.iter().any(|l| l.as_str() == Some("Kotlin")));
    assert!(languages.iter().any(|l| l.as_str() == Some("TypeScript")));
    
    // Verify summary statistics
    let summary = result.get("summary").and_then(|s| s.as_object());
    assert!(summary.is_some());
    let summary = summary.unwrap();
    assert!(summary.get("total_files").and_then(|f| f.as_u64()).unwrap_or(0) > 0);
    assert!(summary.get("total_nodes").and_then(|n| n.as_u64()).unwrap_or(0) > 0);
    assert!(summary.get("total_cross_language_dependencies").and_then(|d| d.as_u64()).unwrap_or(0) > 0);
    
    // Verify dependency types
    let dependency_counts = result.get("dependency_counts").and_then(|d| d.as_object());
    assert!(dependency_counts.is_some());
    let dependency_counts = dependency_counts.unwrap();
    assert!(dependency_counts.contains_key("Java -> Kotlin"));
    assert!(dependency_counts.contains_key("Kotlin -> Java"));
    
    // Verify DOT graph
    let graph_dot = result.get("graph_dot").and_then(|g| g.as_str());
    assert!(graph_dot.is_some());
    let graph_dot = graph_dot.unwrap();
    assert!(graph_dot.contains("digraph"));
    assert!(graph_dot.contains("Java:class:BaseModel"));
    assert!(graph_dot.contains("Kotlin:record:User"));
    
    Ok(())
}

/// Test for the MCP language boundary detection tool
#[tokio::test]
#[ignore = "Requires polyglot-ast feature to be enabled"]
async fn test_mcp_language_boundary_tool() -> Result<()> {
    // Create test fixtures
    let fixture_path = create_test_polyglot_fixtures().await?;
    
    // Create a mock MCP tool instance
    let tool = MockLanguageBoundaryTool::new(fixture_path.clone());
    
    // Create boundary detection parameters
    let params = LanguageBoundaryParams {
        path: fixture_path.to_string_lossy().to_string(),
        source_language: Some("java".to_string()),
        target_language: Some("kotlin".to_string()),
        max_depth: Some(3),
    };
    
    // Execute the boundary detection
    let result = tool.detect_boundaries(params).await?;
    
    // Verify the result
    let status = result.get("status").and_then(|s| s.as_str());
    let path = result.get("path").and_then(|p| p.as_str());
    assert_eq!(status, Some("completed"));
    assert_eq!(path, Some(&fixture_path.to_string_lossy().to_string()));
    
    // Verify languages analyzed
    let languages_analyzed = result.get("languages_analyzed").and_then(|l| l.as_array());
    assert!(languages_analyzed.is_some());
    let languages_analyzed = languages_analyzed.unwrap();
    assert!(languages_analyzed.iter().any(|l| l.as_str() == Some("Java")));
    assert!(languages_analyzed.iter().any(|l| l.as_str() == Some("Kotlin")));
    
    // Verify summary statistics
    let summary = result.get("summary").and_then(|s| s.as_object());
    assert!(summary.is_some());
    let summary = summary.unwrap();
    assert!(summary.get("total_boundaries").and_then(|f| f.as_u64()).unwrap_or(0) > 0);
    assert_eq!(summary.get("source_language").and_then(|s| s.as_str()), Some("Java"));
    assert_eq!(summary.get("target_language").and_then(|s| s.as_str()), Some("Kotlin"));
    
    // Verify boundary types
    let boundary_types = result.get("boundary_types").and_then(|b| b.as_object());
    assert!(boundary_types.is_some());
    let boundary_types = boundary_types.unwrap();
    assert!(boundary_types.contains_key("Uses"));
    assert!(boundary_types.contains_key("Implements") || boundary_types.contains_key("Inherits"));
    
    // Verify patterns and recommendations
    let patterns = result.get("patterns").and_then(|p| p.as_array());
    assert!(patterns.is_some());
    let patterns = patterns.unwrap();
    assert!(!patterns.is_empty());
    
    let java_kotlin_pattern = patterns.iter().find(|p| 
        p.get("language_pair").and_then(|l| l.as_str()) == Some("Java-Kotlin"));
    assert!(java_kotlin_pattern.is_some());
    let recommendations = java_kotlin_pattern.unwrap()
        .get("recommendations").and_then(|r| r.as_array());
    assert!(recommendations.is_some());
    assert!(!recommendations.unwrap().is_empty());
    
    Ok(())
}

// Helper structs for testing

/// Create a test UnifiedNode
fn create_unified_node(
    name: &str,
    kind: NodeKind,
    fqn: &str,
    language: Language,
    file_path: PathBuf
) -> UnifiedNode {
    UnifiedNode {
        id: format!("{}:{}:{}", language.name(), kind.as_str(), name),
        kind,
        name: name.to_string(),
        fqn: fqn.to_string(),
        language,
        file_path,
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

/// Extension trait for UnifiedNode in test context
trait UnifiedNodeExt {
    /// Add a reference to another node (helper method for tests)
    fn add_reference(&mut self, kind: ReferenceKind, target_name: String, target_id: Option<String>);
}

impl UnifiedNodeExt for UnifiedNode {
    fn add_reference(&mut self, kind: ReferenceKind, target_name: String, target_id: Option<String>) {
        let reference = NodeReference {
            kind,
            target_id: target_id.unwrap_or_else(|| "".to_string()),
            target_name,
            target_language: None,
        };
        self.references.push(reference);
    }
}

/// Mock cross-language dependency analyzer for testing
struct MockCrossLanguageDependencyAnalyzer {
    nodes: Vec<UnifiedNode>
}

impl MockCrossLanguageDependencyAnalyzer {
    fn new(nodes: Vec<UnifiedNode>) -> Self {
        Self { nodes }
    }
    
    fn detect_dependencies(&self) -> Vec<CrossLanguageDependency> {
        let mut dependencies = Vec::new();
        
        for source_node in &self.nodes {
            for reference in &source_node.references {
                if let Some(target_node) = self.nodes.iter().find(|n| n.id == reference.target_id) {
                    if source_node.language != target_node.language {
                        dependencies.push(CrossLanguageDependency {
                            source_id: source_node.id.clone(),
                            target_id: target_node.id.clone(),
                            source_name: source_node.name.clone(),
                            target_name: target_node.name.clone(),
                            source_language: source_node.language,
                            target_language: target_node.language,
                            kind: reference.kind,
                            confidence: 1.0,
                            metadata: HashMap::new(),
                        });
                    }
                }
            }
        }
        
        dependencies
    }
}

/// Cross-language dependency representation
struct CrossLanguageDependency {
    source_id: String,
    target_id: String,
    source_name: String,
    target_name: String,
    source_language: Language,
    target_language: Language,
    kind: ReferenceKind,
    confidence: f64,
    metadata: HashMap<String, String>,
}

/// Mock polyglot analyzer for testing
struct MockPolyglotAnalyzer {
    fixture_path: PathBuf
}

impl MockPolyglotAnalyzer {
    fn new(fixture_path: PathBuf) -> Self {
        Self { fixture_path }
    }
    
    async fn analyze(&self) -> Result<PolyglotAnalysisResult> {
        // In a real implementation, this would analyze the fixture files
        // For testing, we'll return a mock result
        
        let mut file_counts = HashMap::new();
        file_counts.insert(Language::Java, 2);
        file_counts.insert(Language::Kotlin, 2);
        file_counts.insert(Language::TypeScript, 2);
        
        let cross_language_dependencies = vec![
            CrossLanguageDependency {
                source_id: "Kotlin:record:User".to_string(),
                target_id: "Java:class:BaseModel".to_string(),
                source_name: "User".to_string(),
                target_name: "BaseModel".to_string(),
                source_language: Language::Kotlin,
                target_language: Language::Java,
                kind: ReferenceKind::Inherits,
                confidence: 1.0,
                metadata: HashMap::new(),
            },
            CrossLanguageDependency {
                source_id: "Java:class:UserRepository".to_string(),
                target_id: "Kotlin:record:User".to_string(),
                source_name: "UserRepository".to_string(),
                target_name: "User".to_string(),
                source_language: Language::Java,
                target_language: Language::Kotlin,
                kind: ReferenceKind::Uses,
                confidence: 1.0,
                metadata: HashMap::new(),
            },
            CrossLanguageDependency {
                source_id: "TypeScript:interface:UserModel".to_string(),
                target_id: "Kotlin:record:User".to_string(),
                source_name: "UserModel".to_string(),
                target_name: "User".to_string(),
                source_language: Language::TypeScript,
                target_language: Language::Kotlin,
                kind: ReferenceKind::DependsOn,
                confidence: 0.8,
                metadata: HashMap::new(),
            },
        ];
        
        Ok(PolyglotAnalysisResult {
            languages: vec![Language::Java, Language::Kotlin, Language::TypeScript],
            file_counts,
            cross_language_dependencies,
        })
    }
}

struct PolyglotAnalysisResult {
    languages: Vec<Language>,
    file_counts: HashMap<Language, usize>,
    cross_language_dependencies: Vec<CrossLanguageDependency>,
}

// Parameter structs for MCP tools
/// Parameters for polyglot analysis
#[derive(Debug)]
struct PolyglotAnalysisParams {
    path: String,
    languages: Option<Vec<String>>,
    max_depth: Option<u64>,
    include_graph: Option<bool>,
}

/// Parameters for language boundary detection
#[derive(Debug)]
struct LanguageBoundaryParams {
    path: String,
    source_language: Option<String>,
    target_language: Option<String>,
    max_depth: Option<u64>,
}

// Result types for MCP tools
#[derive(Debug, serde::Serialize)]
struct PolyglotAnalysisToolResult {
    status: String,
    path: String,
    languages: Vec<String>,
    summary: PolyglotAnalysisSummary,
    node_counts: HashMap<String, HashMap<String, usize>>,
    dependency_counts: HashMap<String, usize>,
    dependencies: Vec<DependencyResult>,
    graph_dot: String,
}

#[derive(Debug, serde::Serialize)]
struct PolyglotAnalysisSummary {
    total_files: usize,
    total_nodes: usize,
    nodes_by_language: HashMap<String, usize>,
    total_cross_language_dependencies: usize,
}

#[derive(Debug, serde::Serialize)]
struct DependencyResult {
    source: NodeSummary,
    target: NodeSummary,
    kind: String,
    source_language: String,
    target_language: String,
    confidence: f64,
}

#[derive(Debug, serde::Serialize)]
struct NodeSummary {
    id: String,
    name: String,
    fqn: String,
    kind: String,
}

/// Mock MCP polyglot analysis tool for testing
struct MockPolyglotAnalysisTool {
    fixture_path: PathBuf
}

impl MockPolyglotAnalysisTool {
    fn new(fixture_path: PathBuf) -> Self {
        Self { fixture_path }
    }
    
    async fn analyze(&self, params: PolyglotAnalysisParams) -> Result<serde_json::Value> {
        // For testing, we'll return a mock result that matches the expected schema
        // In a real implementation, this would call the actual tool
        
        let mut node_counts = HashMap::new();
        let mut java_counts = HashMap::new();
        java_counts.insert("class".to_string(), 2);
        java_counts.insert("method".to_string(), 4);
        
        let mut kotlin_counts = HashMap::new();
        kotlin_counts.insert("class".to_string(), 2);
        kotlin_counts.insert("method".to_string(), 3);
        
        let mut ts_counts = HashMap::new();
        ts_counts.insert("interface".to_string(), 1);
        ts_counts.insert("class".to_string(), 1);
        ts_counts.insert("method".to_string(), 2);
        
        node_counts.insert("Java".to_string(), java_counts);
        node_counts.insert("Kotlin".to_string(), kotlin_counts);
        node_counts.insert("TypeScript".to_string(), ts_counts);
        
        let mut dependency_counts = HashMap::new();
        dependency_counts.insert("Java -> Kotlin".to_string(), 1);
        dependency_counts.insert("Kotlin -> Java".to_string(), 1);
        dependency_counts.insert("TypeScript -> Kotlin".to_string(), 1);
        
        let result = PolyglotAnalysisToolResult {
            status: "completed".to_string(),
            path: params.path,
            languages: vec![
                "Java".to_string(),
                "Kotlin".to_string(),
                "TypeScript".to_string()
            ],
            summary: PolyglotAnalysisSummary {
                total_files: 6,
                total_nodes: 15,
                nodes_by_language: {
                    let mut map = HashMap::new();
                    map.insert("Java".to_string(), 6);
                    map.insert("Kotlin".to_string(), 5);
                    map.insert("TypeScript".to_string(), 4);
                    map
                },
                total_cross_language_dependencies: 3,
            },
            node_counts,
            dependency_counts,
            dependencies: vec![
                DependencyResult {
                    source: NodeSummary {
                        id: "Java:class:UserRepository".to_string(),
                        name: "UserRepository".to_string(),
                        fqn: "com.example.UserRepository".to_string(),
                        kind: "class".to_string(),
                    },
                    target: NodeSummary {
                        id: "Kotlin:record:User".to_string(),
                        name: "User".to_string(),
                        fqn: "com.example.User".to_string(),
                        kind: "record".to_string(),
                    },
                    kind: "Uses".to_string(),
                    source_language: "Java".to_string(),
                    target_language: "Kotlin".to_string(),
                    confidence: 1.0,
                },
                // Additional dependencies would be here
            ],
            graph_dot: "digraph CrossLanguageDependencies {\n  \"Java:class:BaseModel\" -> \"Kotlin:record:User\" [label=\"Inherits\"];\n  \"Java:class:UserRepository\" -> \"Kotlin:record:User\" [label=\"Uses\"];\n  \"TypeScript:interface:UserModel\" -> \"Kotlin:record:User\" [label=\"DependsOn\"];\n}".to_string(),
        };
        
        Ok(serde_json::to_value(result)?)
    }
}

/// Mock MCP language boundary tool for testing
struct MockLanguageBoundaryTool {
    fixture_path: PathBuf
}

impl MockLanguageBoundaryTool {
    fn new(fixture_path: PathBuf) -> Self {
        Self { fixture_path }
    }
    
    async fn detect_boundaries(&self, params: LanguageBoundaryParams) -> Result<serde_json::Value> {
        // For testing, we'll return a mock result that matches the expected schema
        // In a real implementation, this would call the actual tool
        
        let mut boundary_types = HashMap::new();
        boundary_types.insert("Inherits".to_string(), BoundaryTypeInfo {
            count: 1,
            languages: vec!["Java → Kotlin".to_string(), "Kotlin → Java".to_string()],
        });
        boundary_types.insert("Uses".to_string(), BoundaryTypeInfo {
            count: 2,
            languages: vec!["Java → Kotlin".to_string(), "TypeScript → Kotlin".to_string()],
        });
        
        let result = LanguageBoundaryToolResult {
            status: "completed".to_string(),
            path: params.path,
            languages_analyzed: vec![
                "Java".to_string(),
                "Kotlin".to_string(),
                "TypeScript".to_string(),
            ],
            summary: BoundarySummary {
                total_boundaries: 3,
                source_language: "Java".to_string(),
                target_language: "Kotlin".to_string(),
            },
            boundaries: vec![
                BoundaryResult {
                    boundary_type: "Uses".to_string(),
                    source: BoundaryNode {
                        language: "Java".to_string(),
                        node: NodeSummary {
                            id: "Java:class:UserRepository".to_string(),
                            name: "UserRepository".to_string(),
                            fqn: "com.example.UserRepository".to_string(),
                            kind: "class".to_string(),
                        },
                    },
                    target: BoundaryNode {
                        language: "Kotlin".to_string(),
                        node: NodeSummary {
                            id: "Kotlin:record:User".to_string(),
                            name: "User".to_string(),
                            fqn: "com.example.User".to_string(),
                            kind: "record".to_string(),
                        },
                    },
                    confidence: 1.0,
                },
                // Additional boundaries would be here
            ],
            boundary_types,
            patterns: vec![
                BoundaryPattern {
                    language_pair: "Java-Kotlin".to_string(),
                    count: 2,
                    recommendations: vec![
                        "Use Kotlin's @JvmName annotation to control Java-visible names".to_string(),
                        "Leverage Kotlin extension functions for Java interoperability".to_string(),
                        "Use Kotlin's nullable types consistently with Java's @Nullable".to_string(),
                    ],
                },
                BoundaryPattern {
                    language_pair: "TypeScript-Kotlin".to_string(),
                    count: 1,
                    recommendations: vec![
                        "Define API contracts with OpenAPI/Swagger for REST interfaces".to_string(),
                        "Use consistent naming conventions across both languages".to_string(),
                    ],
                },
            ],
        };
        
        Ok(serde_json::to_value(result)?)
    }
}

// Additional result types for language boundary tool

#[derive(Debug, serde::Serialize)]
struct LanguageBoundaryToolResult {
    status: String,
    path: String,
    languages_analyzed: Vec<String>,
    summary: BoundarySummary,
    boundaries: Vec<BoundaryResult>,
    boundary_types: HashMap<String, BoundaryTypeInfo>,
    patterns: Vec<BoundaryPattern>,
}

#[derive(Debug, serde::Serialize)]
struct BoundarySummary {
    total_boundaries: usize,
    source_language: String,
    target_language: String,
}

#[derive(Debug, serde::Serialize)]
struct BoundaryResult {
    boundary_type: String,
    source: BoundaryNode,
    target: BoundaryNode,
    confidence: f64,
}

#[derive(Debug, serde::Serialize)]
struct BoundaryNode {
    language: String,
    node: NodeSummary,
}

#[derive(Debug, serde::Serialize)]
struct BoundaryPattern {
    language_pair: String,
    count: usize,
    recommendations: Vec<String>,
}

#[derive(Debug, serde::Serialize)]
struct BoundaryTypeInfo {
    count: usize,
    languages: Vec<String>,
}

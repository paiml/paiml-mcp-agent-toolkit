#[cfg_attr(coverage_nightly, coverage(off))]
#[cfg(test)]
mod tests {
    use super::*;
    use crate::ast::polyglot::unified_node::{
        ReferenceKind as PolyglotReferenceKind, SourcePosition,
    };
    use crate::ast::polyglot::{Language, NodeKind, UnifiedNode};
    use std::collections::HashMap;
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
        let dependency =
            crate::ast::polyglot::cross_language_dependencies::CrossLanguageDependency {
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
        assert_eq!(patterns.as_array().expect("internal error").len(), 1);

        let first_pattern = &patterns.as_array().unwrap()[0];
        assert_eq!(first_pattern["language_pair"], "Java-Kotlin");
        assert!(first_pattern["recommendations"].is_array());
        assert!(!first_pattern["recommendations"]
            .as_array()
            .expect("internal error")
            .is_empty());
    }
}

#[cfg_attr(coverage_nightly, coverage(off))]
#[cfg(test)]
mod coverage_tests {
    use super::*;
    use crate::agents::registry::AgentRegistry;
    use crate::ast::polyglot::cross_language_dependencies::CrossLanguageDependency;
    use crate::ast::polyglot::unified_node::{
        ReferenceKind as PolyglotReferenceKind, SourcePosition,
    };
    use crate::ast::polyglot::{Language, NodeKind, UnifiedNode};
    use std::collections::HashMap;
    use std::path::PathBuf;
    use std::sync::Arc;
    use tempfile::TempDir;

    // ============================================================
    // Test Fixtures
    // ============================================================

    fn create_test_registry() -> Arc<AgentRegistry> {
        Arc::new(AgentRegistry::new())
    }

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

    fn create_test_node_with_path(
        id: &str,
        kind: NodeKind,
        name: &str,
        fqn: &str,
        language: Language,
        file_path: PathBuf,
    ) -> UnifiedNode {
        UnifiedNode {
            id: id.to_string(),
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

    fn create_test_dependency(
        source_id: &str,
        target_id: &str,
        source_language: Language,
        target_language: Language,
        kind: PolyglotReferenceKind,
    ) -> CrossLanguageDependency {
        CrossLanguageDependency {
            source_id: source_id.to_string(),
            target_id: target_id.to_string(),
            source_language,
            target_language,
            kind,
            confidence: 1.0,
            metadata: HashMap::new(),
        }
    }

    // ============================================================
    // PolyglotAnalysisTool Tests
    // ============================================================

    #[test]
    fn test_polyglot_analysis_tool_creation() {
        let registry = create_test_registry();
        let tool = PolyglotAnalysisTool::new(registry);

        let metadata = tool.metadata();
        assert_eq!(metadata.name, "analyze_polyglot");
        assert!(metadata.description.contains("cross-language"));
    }

    #[test]
    fn test_polyglot_analysis_tool_metadata() {
        let registry = create_test_registry();
        let tool = PolyglotAnalysisTool::new(registry);

        let metadata = tool.metadata();

        // Check input schema
        assert!(metadata.input_schema["properties"]["path"].is_object());
        assert!(metadata.input_schema["properties"]["languages"].is_object());
        assert!(metadata.input_schema["properties"]["max_depth"].is_object());
        assert!(metadata.input_schema["properties"]["include_graph"].is_object());

        // Check required fields
        let required = metadata.input_schema["required"].as_array().unwrap();
        assert!(required.iter().any(|v| v == "path"));
    }

    #[tokio::test]
    async fn test_polyglot_analysis_tool_missing_path() {
        let registry = create_test_registry();
        let tool = PolyglotAnalysisTool::new(registry);

        let params = json!({});
        let result = tool.execute(params).await;

        assert!(result.is_err());
        let err = result.unwrap_err();
        assert_eq!(
            err.code,
            crate::mcp_integration::error_codes::INVALID_PARAMS
        );
        assert!(err.message.contains("Missing path"));
    }

    #[tokio::test]
    async fn test_polyglot_analysis_tool_invalid_path() {
        let registry = create_test_registry();
        let tool = PolyglotAnalysisTool::new(registry);

        let params = json!({
            "path": "/nonexistent/path/that/does/not/exist"
        });
        let result = tool.execute(params).await;

        assert!(result.is_err());
        let err = result.unwrap_err();
        assert_eq!(
            err.code,
            crate::mcp_integration::error_codes::INVALID_PARAMS
        );
        assert!(err.message.contains("Invalid directory path"));
        assert!(err.data.is_some());
    }

    #[tokio::test]
    async fn test_polyglot_analysis_tool_valid_directory() {
        let registry = create_test_registry();
        let tool = PolyglotAnalysisTool::new(registry);

        // Create a temporary directory for testing
        let temp_dir = TempDir::new().unwrap();

        let params = json!({
            "path": temp_dir.path().to_str().unwrap()
        });
        let result = tool.execute(params).await;

        assert!(result.is_ok());
        let value = result.unwrap();

        // Check basic response structure
        assert_eq!(value["status"], "completed");
        assert!(value["summary"]["total_files"].is_number());
        assert!(value["summary"]["total_nodes"].is_number());
        assert!(value["dependencies"].is_array());
    }

    #[tokio::test]
    async fn test_polyglot_analysis_tool_with_languages_filter() {
        let registry = create_test_registry();
        let tool = PolyglotAnalysisTool::new(registry);

        let temp_dir = TempDir::new().unwrap();

        let params = json!({
            "path": temp_dir.path().to_str().unwrap(),
            "languages": ["java", "kotlin"]
        });
        let result = tool.execute(params).await;

        assert!(result.is_ok());
        let value = result.unwrap();

        let languages = value["languages"].as_array().unwrap();
        assert!(languages.iter().any(|l| l == "Java"));
        assert!(languages.iter().any(|l| l == "Kotlin"));
    }

    #[tokio::test]
    async fn test_polyglot_analysis_tool_with_empty_languages() {
        let registry = create_test_registry();
        let tool = PolyglotAnalysisTool::new(registry);

        let temp_dir = TempDir::new().unwrap();

        // Empty languages array should default to all supported
        let params = json!({
            "path": temp_dir.path().to_str().unwrap(),
            "languages": []
        });
        let result = tool.execute(params).await;

        assert!(result.is_ok());
        let value = result.unwrap();

        // Should have all 5 default languages
        let languages = value["languages"].as_array().unwrap();
        assert_eq!(languages.len(), 5);
    }

    #[tokio::test]
    async fn test_polyglot_analysis_tool_with_invalid_language() {
        let registry = create_test_registry();
        let tool = PolyglotAnalysisTool::new(registry);

        let temp_dir = TempDir::new().unwrap();

        // Invalid language should be filtered out
        let params = json!({
            "path": temp_dir.path().to_str().unwrap(),
            "languages": ["invalid_language", "java"]
        });
        let result = tool.execute(params).await;

        assert!(result.is_ok());
        let value = result.unwrap();

        let languages = value["languages"].as_array().unwrap();
        assert_eq!(languages.len(), 1);
        assert!(languages.iter().any(|l| l == "Java"));
    }

    #[tokio::test]
    async fn test_polyglot_analysis_tool_max_depth() {
        let registry = create_test_registry();
        let tool = PolyglotAnalysisTool::new(registry);

        let temp_dir = TempDir::new().unwrap();

        let params = json!({
            "path": temp_dir.path().to_str().unwrap(),
            "max_depth": 5
        });
        let result = tool.execute(params).await;

        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_polyglot_analysis_tool_no_graph() {
        let registry = create_test_registry();
        let tool = PolyglotAnalysisTool::new(registry);

        let temp_dir = TempDir::new().unwrap();

        let params = json!({
            "path": temp_dir.path().to_str().unwrap(),
            "include_graph": false
        });
        let result = tool.execute(params).await;

        assert!(result.is_ok());
        let value = result.unwrap();

        // graph_dot should not be present when include_graph is false
        assert!(value.get("graph_dot").is_none());
    }

    #[tokio::test]
    async fn test_polyglot_analysis_tool_with_graph() {
        let registry = create_test_registry();
        let tool = PolyglotAnalysisTool::new(registry);

        let temp_dir = TempDir::new().unwrap();

        let params = json!({
            "path": temp_dir.path().to_str().unwrap(),
            "include_graph": true
        });
        let result = tool.execute(params).await;

        assert!(result.is_ok());
        let value = result.unwrap();

        // graph_dot should be present
        assert!(value["graph_dot"].is_string());
    }

    // ============================================================
    // LanguageBoundaryTool Tests
    // ============================================================

    #[test]
    fn test_language_boundary_tool_creation() {
        let registry = create_test_registry();
        let tool = LanguageBoundaryTool::new(registry);

        let metadata = tool.metadata();
        assert_eq!(metadata.name, "analyze_language_boundaries");
        assert!(metadata.description.contains("language boundaries"));
    }

    #[test]
    fn test_language_boundary_tool_metadata() {
        let registry = create_test_registry();
        let tool = LanguageBoundaryTool::new(registry);

        let metadata = tool.metadata();

        // Check input schema
        assert!(metadata.input_schema["properties"]["path"].is_object());
        assert!(metadata.input_schema["properties"]["source_language"].is_object());
        assert!(metadata.input_schema["properties"]["target_language"].is_object());
        assert!(metadata.input_schema["properties"]["max_depth"].is_object());

        // Check required fields
        let required = metadata.input_schema["required"].as_array().unwrap();
        assert!(required.iter().any(|v| v == "path"));
    }

    #[tokio::test]
    async fn test_language_boundary_tool_missing_path() {
        let registry = create_test_registry();
        let tool = LanguageBoundaryTool::new(registry);

        let params = json!({});
        let result = tool.execute(params).await;

        assert!(result.is_err());
        let err = result.unwrap_err();
        assert_eq!(
            err.code,
            crate::mcp_integration::error_codes::INVALID_PARAMS
        );
        assert!(err.message.contains("Missing path"));
    }

    #[tokio::test]
    async fn test_language_boundary_tool_invalid_path() {
        let registry = create_test_registry();
        let tool = LanguageBoundaryTool::new(registry);

        let params = json!({
            "path": "/nonexistent/path/that/does/not/exist"
        });
        let result = tool.execute(params).await;

        assert!(result.is_err());
        let err = result.unwrap_err();
        assert_eq!(
            err.code,
            crate::mcp_integration::error_codes::INVALID_PARAMS
        );
        assert!(err.message.contains("Invalid directory path"));
    }

    #[tokio::test]
    async fn test_language_boundary_tool_valid_directory() {
        let registry = create_test_registry();
        let tool = LanguageBoundaryTool::new(registry);

        let temp_dir = TempDir::new().unwrap();

        let params = json!({
            "path": temp_dir.path().to_str().unwrap()
        });
        let result = tool.execute(params).await;

        assert!(result.is_ok());
        let value = result.unwrap();

        assert_eq!(value["status"], "completed");
        assert!(value["summary"]["total_boundaries"].is_number());
        assert!(value["boundaries"].is_array());
        assert!(value["patterns"].is_array());
    }

    #[tokio::test]
    async fn test_language_boundary_tool_with_source_language() {
        let registry = create_test_registry();
        let tool = LanguageBoundaryTool::new(registry);

        let temp_dir = TempDir::new().unwrap();

        let params = json!({
            "path": temp_dir.path().to_str().unwrap(),
            "source_language": "java"
        });
        let result = tool.execute(params).await;

        assert!(result.is_ok());
        let value = result.unwrap();

        // Source language should be set in summary
        assert_eq!(value["summary"]["source_language"], "Java");
    }

    #[tokio::test]
    async fn test_language_boundary_tool_with_target_language() {
        let registry = create_test_registry();
        let tool = LanguageBoundaryTool::new(registry);

        let temp_dir = TempDir::new().unwrap();

        let params = json!({
            "path": temp_dir.path().to_str().unwrap(),
            "target_language": "kotlin"
        });
        let result = tool.execute(params).await;

        assert!(result.is_ok());
        let value = result.unwrap();

        assert_eq!(value["summary"]["target_language"], "Kotlin");
    }

    #[tokio::test]
    async fn test_language_boundary_tool_with_both_languages() {
        let registry = create_test_registry();
        let tool = LanguageBoundaryTool::new(registry);

        let temp_dir = TempDir::new().unwrap();

        let params = json!({
            "path": temp_dir.path().to_str().unwrap(),
            "source_language": "java",
            "target_language": "kotlin"
        });
        let result = tool.execute(params).await;

        assert!(result.is_ok());
        let value = result.unwrap();

        assert_eq!(value["summary"]["source_language"], "Java");
        assert_eq!(value["summary"]["target_language"], "Kotlin");

        // Only those two languages should be analyzed
        let languages = value["languages_analyzed"].as_array().unwrap();
        assert_eq!(languages.len(), 2);
    }

    #[tokio::test]
    async fn test_language_boundary_tool_with_same_source_target() {
        let registry = create_test_registry();
        let tool = LanguageBoundaryTool::new(registry);

        let temp_dir = TempDir::new().unwrap();

        // Same source and target language - should not duplicate
        let params = json!({
            "path": temp_dir.path().to_str().unwrap(),
            "source_language": "java",
            "target_language": "java"
        });
        let result = tool.execute(params).await;

        assert!(result.is_ok());
        let value = result.unwrap();

        let languages = value["languages_analyzed"].as_array().unwrap();
        assert_eq!(languages.len(), 1);
    }

    #[tokio::test]
    async fn test_language_boundary_tool_with_invalid_languages() {
        let registry = create_test_registry();
        let tool = LanguageBoundaryTool::new(registry);

        let temp_dir = TempDir::new().unwrap();

        let params = json!({
            "path": temp_dir.path().to_str().unwrap(),
            "source_language": "invalid_lang",
            "target_language": "another_invalid"
        });
        let result = tool.execute(params).await;

        assert!(result.is_ok());
        let value = result.unwrap();

        // With no valid languages, should analyze all
        let languages = value["languages_analyzed"].as_array().unwrap();
        assert_eq!(languages.len(), 5);
    }

    #[tokio::test]
    async fn test_language_boundary_tool_max_depth() {
        let registry = create_test_registry();
        let tool = LanguageBoundaryTool::new(registry);

        let temp_dir = TempDir::new().unwrap();

        let params = json!({
            "path": temp_dir.path().to_str().unwrap(),
            "max_depth": 10
        });
        let result = tool.execute(params).await;

        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_language_boundary_tool_all_language_combinations() {
        let registry = create_test_registry();
        let tool = LanguageBoundaryTool::new(registry);

        let temp_dir = TempDir::new().unwrap();

        // Test all valid language options
        let languages = ["java", "kotlin", "scala", "typescript", "javascript"];

        for lang in &languages {
            let params = json!({
                "path": temp_dir.path().to_str().unwrap(),
                "source_language": lang
            });
            let result = tool.execute(params).await;
            assert!(result.is_ok(), "Failed for source_language: {}", lang);
        }
    }

    // ============================================================
    // get_node_type_counts Tests
    // ============================================================

    #[test]
    fn test_get_node_type_counts_empty() {
        let nodes: Vec<UnifiedNode> = vec![];
        let counts = get_node_type_counts(&nodes);
        assert!(counts.is_empty());
    }

    #[test]
    fn test_get_node_type_counts_single_node() {
        let node = create_test_node(
            "Java:class:Test",
            NodeKind::Class,
            "Test",
            "com.example.Test",
            Language::Java,
        );
        let nodes = vec![node];

        let counts = get_node_type_counts(&nodes);

        assert_eq!(counts.len(), 1);
        assert!(counts.contains_key("Java"));
        assert_eq!(counts["Java"]["class"], 1);
    }

    #[test]
    fn test_get_node_type_counts_multiple_nodes_same_language() {
        let nodes = vec![
            create_test_node(
                "Java:class:Test1",
                NodeKind::Class,
                "Test1",
                "com.example.Test1",
                Language::Java,
            ),
            create_test_node(
                "Java:class:Test2",
                NodeKind::Class,
                "Test2",
                "com.example.Test2",
                Language::Java,
            ),
            create_test_node(
                "Java:method:doSomething",
                NodeKind::Method,
                "doSomething",
                "com.example.Test1.doSomething",
                Language::Java,
            ),
        ];

        let counts = get_node_type_counts(&nodes);

        assert_eq!(counts.len(), 1);
        assert_eq!(counts["Java"]["class"], 2);
        assert_eq!(counts["Java"]["method"], 1);
    }

    #[test]
    fn test_get_node_type_counts_multiple_languages() {
        let nodes = vec![
            create_test_node(
                "Java:class:JavaClass",
                NodeKind::Class,
                "JavaClass",
                "com.example.JavaClass",
                Language::Java,
            ),
            create_test_node(
                "Kotlin:class:KotlinClass",
                NodeKind::Class,
                "KotlinClass",
                "com.example.KotlinClass",
                Language::Kotlin,
            ),
            create_test_node(
                "TypeScript:interface:IModel",
                NodeKind::Interface,
                "IModel",
                "models/IModel",
                Language::TypeScript,
            ),
        ];

        let counts = get_node_type_counts(&nodes);

        assert_eq!(counts.len(), 3);
        assert_eq!(counts["Java"]["class"], 1);
        assert_eq!(counts["Kotlin"]["class"], 1);
        assert_eq!(counts["TypeScript"]["interface"], 1);
    }

    #[test]
    fn test_get_node_type_counts_all_node_kinds() {
        let nodes = vec![
            create_test_node("id1", NodeKind::Class, "n", "f", Language::Java),
            create_test_node("id2", NodeKind::Interface, "n", "f", Language::Java),
            create_test_node("id3", NodeKind::Method, "n", "f", Language::Java),
            create_test_node("id4", NodeKind::Function, "n", "f", Language::Java),
            create_test_node("id5", NodeKind::Field, "n", "f", Language::Java),
            create_test_node("id6", NodeKind::Enum, "n", "f", Language::Java),
            create_test_node("id7", NodeKind::Struct, "n", "f", Language::Java),
            create_test_node("id8", NodeKind::Trait, "n", "f", Language::Java),
        ];

        let counts = get_node_type_counts(&nodes);

        assert_eq!(counts["Java"].len(), 8);
    }

    // ============================================================
    // analyze_boundary_patterns Tests
    // ============================================================

    #[test]
    fn test_analyze_boundary_patterns_empty() {
        let deps: Vec<&CrossLanguageDependency> = vec![];
        let nodes: Vec<UnifiedNode> = vec![];

        let patterns = analyze_boundary_patterns(deps, &nodes);

        assert!(patterns.is_array());
        assert!(patterns.as_array().unwrap().is_empty());
    }

    #[test]
    fn test_analyze_boundary_patterns_java_kotlin() {
        let dep = create_test_dependency(
            "Java:class:Test",
            "Kotlin:class:KotlinTest",
            Language::Java,
            Language::Kotlin,
            PolyglotReferenceKind::Inherits,
        );
        let nodes: Vec<UnifiedNode> = vec![];

        let patterns = analyze_boundary_patterns(vec![&dep], &nodes);

        assert!(patterns.is_array());
        let patterns_arr = patterns.as_array().unwrap();
        assert_eq!(patterns_arr.len(), 1);

        let pattern = &patterns_arr[0];
        assert_eq!(pattern["language_pair"], "Java-Kotlin");
        assert_eq!(pattern["count"], 1);
        assert!(pattern["recommendations"].is_array());

        // Check Java-Kotlin specific recommendations
        let recommendations = pattern["recommendations"].as_array().unwrap();
        assert!(!recommendations.is_empty());
        assert!(recommendations
            .iter()
            .any(|r| r.as_str().unwrap().contains("@JvmName")));
    }

    #[test]
    fn test_analyze_boundary_patterns_kotlin_java() {
        let dep = create_test_dependency(
            "Kotlin:class:Test",
            "Java:class:JavaTest",
            Language::Kotlin,
            Language::Java,
            PolyglotReferenceKind::Inherits,
        );
        let nodes: Vec<UnifiedNode> = vec![];

        let patterns = analyze_boundary_patterns(vec![&dep], &nodes);

        let patterns_arr = patterns.as_array().unwrap();
        let pattern = &patterns_arr[0];

        // Should still get Java-Kotlin recommendations
        let recommendations = pattern["recommendations"].as_array().unwrap();
        assert!(recommendations
            .iter()
            .any(|r| r.as_str().unwrap().contains("@JvmName")));
    }

    #[test]
    fn test_analyze_boundary_patterns_java_scala() {
        let dep = create_test_dependency(
            "Java:class:Test",
            "Scala:class:ScalaTest",
            Language::Java,
            Language::Scala,
            PolyglotReferenceKind::Inherits,
        );
        let nodes: Vec<UnifiedNode> = vec![];

        let patterns = analyze_boundary_patterns(vec![&dep], &nodes);

        let patterns_arr = patterns.as_array().unwrap();
        let pattern = &patterns_arr[0];

        // Check Java-Scala specific recommendations
        let recommendations = pattern["recommendations"].as_array().unwrap();
        assert!(recommendations
            .iter()
            .any(|r| r.as_str().unwrap().contains("implicit")));
    }

    #[test]
    fn test_analyze_boundary_patterns_scala_java() {
        let dep = create_test_dependency(
            "Scala:class:Test",
            "Java:class:JavaTest",
            Language::Scala,
            Language::Java,
            PolyglotReferenceKind::Inherits,
        );
        let nodes: Vec<UnifiedNode> = vec![];

        let patterns = analyze_boundary_patterns(vec![&dep], &nodes);

        let patterns_arr = patterns.as_array().unwrap();
        let pattern = &patterns_arr[0];

        let recommendations = pattern["recommendations"].as_array().unwrap();
        assert!(recommendations
            .iter()
            .any(|r| r.as_str().unwrap().contains("case class")));
    }

    #[test]
    fn test_analyze_boundary_patterns_typescript_javascript() {
        let dep = create_test_dependency(
            "TypeScript:interface:ITest",
            "JavaScript:function:test",
            Language::TypeScript,
            Language::JavaScript,
            PolyglotReferenceKind::Uses,
        );
        let nodes: Vec<UnifiedNode> = vec![];

        let patterns = analyze_boundary_patterns(vec![&dep], &nodes);

        let patterns_arr = patterns.as_array().unwrap();
        let pattern = &patterns_arr[0];

        // Check TypeScript-JavaScript specific recommendations
        let recommendations = pattern["recommendations"].as_array().unwrap();
        assert!(recommendations
            .iter()
            .any(|r| r.as_str().unwrap().contains(".d.ts")));
    }

    #[test]
    fn test_analyze_boundary_patterns_javascript_typescript() {
        let dep = create_test_dependency(
            "JavaScript:function:test",
            "TypeScript:interface:ITest",
            Language::JavaScript,
            Language::TypeScript,
            PolyglotReferenceKind::Uses,
        );
        let nodes: Vec<UnifiedNode> = vec![];

        let patterns = analyze_boundary_patterns(vec![&dep], &nodes);

        let patterns_arr = patterns.as_array().unwrap();
        let pattern = &patterns_arr[0];

        let recommendations = pattern["recommendations"].as_array().unwrap();
        assert!(recommendations
            .iter()
            .any(|r| r.as_str().unwrap().contains("JSDoc")));
    }

    #[test]
    fn test_analyze_boundary_patterns_java_typescript() {
        let dep = create_test_dependency(
            "Java:class:Test",
            "TypeScript:interface:ITest",
            Language::Java,
            Language::TypeScript,
            PolyglotReferenceKind::Uses,
        );
        let nodes: Vec<UnifiedNode> = vec![];

        let patterns = analyze_boundary_patterns(vec![&dep], &nodes);

        let patterns_arr = patterns.as_array().unwrap();
        let pattern = &patterns_arr[0];

        // Check Java-TypeScript specific recommendations
        let recommendations = pattern["recommendations"].as_array().unwrap();
        assert!(recommendations
            .iter()
            .any(|r| r.as_str().unwrap().contains("OpenAPI")
                || r.as_str().unwrap().contains("Swagger")));
    }

    #[test]
    fn test_analyze_boundary_patterns_typescript_java() {
        let dep = create_test_dependency(
            "TypeScript:interface:ITest",
            "Java:class:Test",
            Language::TypeScript,
            Language::Java,
            PolyglotReferenceKind::Uses,
        );
        let nodes: Vec<UnifiedNode> = vec![];

        let patterns = analyze_boundary_patterns(vec![&dep], &nodes);

        let patterns_arr = patterns.as_array().unwrap();
        let pattern = &patterns_arr[0];

        let recommendations = pattern["recommendations"].as_array().unwrap();
        assert!(recommendations.iter().any(
            |r| r.as_str().unwrap().contains("GraphQL") || r.as_str().unwrap().contains("gRPC")
        ));
    }

    #[test]
    fn test_analyze_boundary_patterns_generic_languages() {
        // Use a language pair that doesn't have specific recommendations
        let dep = create_test_dependency(
            "Python:class:Test",
            "Rust:struct:Test",
            Language::Python,
            Language::Rust,
            PolyglotReferenceKind::Uses,
        );
        let nodes: Vec<UnifiedNode> = vec![];

        let patterns = analyze_boundary_patterns(vec![&dep], &nodes);

        let patterns_arr = patterns.as_array().unwrap();
        let pattern = &patterns_arr[0];

        // Should get generic recommendations
        let recommendations = pattern["recommendations"].as_array().unwrap();
        assert!(recommendations
            .iter()
            .any(|r| r.as_str().unwrap().contains("API contracts")));
    }

    #[test]
    fn test_analyze_boundary_patterns_multiple_dependencies() {
        let deps = vec![
            create_test_dependency(
                "Java:class:Test1",
                "Kotlin:class:KotlinTest1",
                Language::Java,
                Language::Kotlin,
                PolyglotReferenceKind::Inherits,
            ),
            create_test_dependency(
                "Java:class:Test2",
                "Kotlin:class:KotlinTest2",
                Language::Java,
                Language::Kotlin,
                PolyglotReferenceKind::Uses,
            ),
            create_test_dependency(
                "TypeScript:interface:IModel",
                "JavaScript:function:createModel",
                Language::TypeScript,
                Language::JavaScript,
                PolyglotReferenceKind::Uses,
            ),
        ];
        let dep_refs: Vec<&CrossLanguageDependency> = deps.iter().collect();
        let nodes: Vec<UnifiedNode> = vec![];

        let patterns = analyze_boundary_patterns(dep_refs, &nodes);

        let patterns_arr = patterns.as_array().unwrap();
        // Should have 2 patterns (Java-Kotlin and TypeScript-JavaScript)
        assert_eq!(patterns_arr.len(), 2);

        // Find Java-Kotlin pattern and check count
        let java_kotlin_pattern = patterns_arr
            .iter()
            .find(|p| p["language_pair"] == "Java-Kotlin");
        assert!(java_kotlin_pattern.is_some());
        assert_eq!(java_kotlin_pattern.unwrap()["count"], 2);
    }

    #[test]
    fn test_analyze_boundary_patterns_aggregates_same_pair() {
        let deps = vec![
            create_test_dependency(
                "Java:class:A",
                "Kotlin:class:B",
                Language::Java,
                Language::Kotlin,
                PolyglotReferenceKind::Inherits,
            ),
            create_test_dependency(
                "Java:class:C",
                "Kotlin:class:D",
                Language::Java,
                Language::Kotlin,
                PolyglotReferenceKind::Implements,
            ),
            create_test_dependency(
                "Java:class:E",
                "Kotlin:class:F",
                Language::Java,
                Language::Kotlin,
                PolyglotReferenceKind::Uses,
            ),
        ];
        let dep_refs: Vec<&CrossLanguageDependency> = deps.iter().collect();
        let nodes: Vec<UnifiedNode> = vec![];

        let patterns = analyze_boundary_patterns(dep_refs, &nodes);

        let patterns_arr = patterns.as_array().unwrap();
        // All dependencies are between Java and Kotlin, so only 1 pattern
        assert_eq!(patterns_arr.len(), 1);
        assert_eq!(patterns_arr[0]["count"], 3);
    }

    // ============================================================
    // Integration Tests
    // ============================================================

    #[tokio::test]
    async fn test_polyglot_analysis_full_workflow() {
        let registry = create_test_registry();
        let tool = PolyglotAnalysisTool::new(registry);

        let temp_dir = TempDir::new().unwrap();

        // Create a Java file in the temp directory
        std::fs::write(temp_dir.path().join("Test.java"), "public class Test {}").unwrap();

        let params = json!({
            "path": temp_dir.path().to_str().unwrap(),
            "languages": ["java"],
            "max_depth": 3,
            "include_graph": true
        });

        let result = tool.execute(params).await;
        assert!(result.is_ok());

        let value = result.unwrap();
        assert_eq!(value["status"], "completed");
        assert!(value["graph_dot"].is_string());
        assert!(value["node_counts"].is_object());
        assert!(value["dependency_counts"].is_object());
    }

    #[tokio::test]
    async fn test_language_boundary_full_workflow() {
        let registry = create_test_registry();
        let tool = LanguageBoundaryTool::new(registry);

        let temp_dir = TempDir::new().unwrap();

        // Create files in the temp directory
        std::fs::write(temp_dir.path().join("Model.java"), "public class Model {}").unwrap();
        std::fs::write(temp_dir.path().join("Model.kt"), "class Model").unwrap();

        let params = json!({
            "path": temp_dir.path().to_str().unwrap(),
            "source_language": "java",
            "target_language": "kotlin",
            "max_depth": 3
        });

        let result = tool.execute(params).await;
        assert!(result.is_ok());

        let value = result.unwrap();
        assert_eq!(value["status"], "completed");
        assert!(value["boundaries"].is_array());
        assert!(value["boundary_types"].is_object());
        assert!(value["patterns"].is_array());
    }

    // ============================================================
    // Edge Cases and Error Handling
    // ============================================================

    #[tokio::test]
    async fn test_polyglot_tool_with_file_not_directory() {
        let registry = create_test_registry();
        let tool = PolyglotAnalysisTool::new(registry);

        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("test.txt");
        std::fs::write(&file_path, "content").unwrap();

        let params = json!({
            "path": file_path.to_str().unwrap()
        });

        let result = tool.execute(params).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_language_boundary_tool_with_file_not_directory() {
        let registry = create_test_registry();
        let tool = LanguageBoundaryTool::new(registry);

        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("test.txt");
        std::fs::write(&file_path, "content").unwrap();

        let params = json!({
            "path": file_path.to_str().unwrap()
        });

        let result = tool.execute(params).await;
        assert!(result.is_err());
    }

    #[test]
    fn test_mcp_tool_trait_implementation() {
        // Verify that both tools implement McpTool trait properly
        let registry = create_test_registry();

        let polyglot_tool: Arc<dyn McpTool> = Arc::new(PolyglotAnalysisTool::new(registry.clone()));
        let boundary_tool: Arc<dyn McpTool> = Arc::new(LanguageBoundaryTool::new(registry));

        // Check metadata is accessible through trait
        assert!(!polyglot_tool.metadata().name.is_empty());
        assert!(!boundary_tool.metadata().name.is_empty());
    }

    #[tokio::test]
    async fn test_case_insensitive_language_parsing() {
        let registry = create_test_registry();
        let tool = LanguageBoundaryTool::new(registry);

        let temp_dir = TempDir::new().unwrap();

        // Test various case combinations
        let cases = ["JAVA", "Java", "java", "JaVa"];

        for case in cases {
            let params = json!({
                "path": temp_dir.path().to_str().unwrap(),
                "source_language": case
            });

            let result = tool.execute(params).await;
            assert!(result.is_ok(), "Failed for case: {}", case);

            let value = result.unwrap();
            assert_eq!(value["summary"]["source_language"], "Java");
        }
    }

    #[tokio::test]
    async fn test_default_max_depth() {
        let registry = create_test_registry();
        let tool = PolyglotAnalysisTool::new(registry);

        let temp_dir = TempDir::new().unwrap();

        // Don't specify max_depth - should default to 3
        let params = json!({
            "path": temp_dir.path().to_str().unwrap()
        });

        let result = tool.execute(params).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_default_include_graph() {
        let registry = create_test_registry();
        let tool = PolyglotAnalysisTool::new(registry);

        let temp_dir = TempDir::new().unwrap();

        // Don't specify include_graph - should default to true
        let params = json!({
            "path": temp_dir.path().to_str().unwrap()
        });

        let result = tool.execute(params).await;
        assert!(result.is_ok());

        let value = result.unwrap();
        // Graph should be included by default
        assert!(value["graph_dot"].is_string());
    }

    #[test]
    fn test_node_with_custom_file_path() {
        let custom_path = PathBuf::from("/custom/path/to/file.java");
        let node = create_test_node_with_path(
            "Java:class:Test",
            NodeKind::Class,
            "Test",
            "com.example.Test",
            Language::Java,
            custom_path.clone(),
        );

        assert_eq!(node.file_path, custom_path);
    }

    #[test]
    fn test_dependency_confidence_levels() {
        let dep1 = CrossLanguageDependency {
            source_id: "src".to_string(),
            target_id: "tgt".to_string(),
            source_language: Language::Java,
            target_language: Language::Kotlin,
            kind: PolyglotReferenceKind::Inherits,
            confidence: 1.0,
            metadata: HashMap::new(),
        };

        let dep2 = CrossLanguageDependency {
            confidence: 0.5,
            ..dep1.clone()
        };

        let dep3 = CrossLanguageDependency {
            confidence: 0.0,
            ..dep1.clone()
        };

        assert_eq!(dep1.confidence, 1.0);
        assert_eq!(dep2.confidence, 0.5);
        assert_eq!(dep3.confidence, 0.0);
    }

    #[test]
    fn test_dependency_metadata() {
        let mut metadata = HashMap::new();
        metadata.insert("key1".to_string(), "value1".to_string());
        metadata.insert("key2".to_string(), "value2".to_string());

        let dep = CrossLanguageDependency {
            source_id: "src".to_string(),
            target_id: "tgt".to_string(),
            source_language: Language::Java,
            target_language: Language::Kotlin,
            kind: PolyglotReferenceKind::Inherits,
            confidence: 1.0,
            metadata,
        };

        assert_eq!(dep.metadata.len(), 2);
        assert_eq!(dep.metadata.get("key1"), Some(&"value1".to_string()));
        assert_eq!(dep.metadata.get("key2"), Some(&"value2".to_string()));
    }

    #[test]
    fn test_all_reference_kinds_in_patterns() {
        let kinds = [
            PolyglotReferenceKind::Inherits,
            PolyglotReferenceKind::Implements,
            PolyglotReferenceKind::Uses,
            PolyglotReferenceKind::Calls,
            PolyglotReferenceKind::Creates,
            PolyglotReferenceKind::Imports,
            PolyglotReferenceKind::Annotates,
            PolyglotReferenceKind::DependsOn,
        ];

        for kind in kinds {
            let dep = create_test_dependency(
                "Java:class:Test",
                "Kotlin:class:KotlinTest",
                Language::Java,
                Language::Kotlin,
                kind,
            );
            let nodes: Vec<UnifiedNode> = vec![];

            let patterns = analyze_boundary_patterns(vec![&dep], &nodes);

            // Should still generate valid patterns regardless of reference kind
            assert!(patterns.is_array());
            assert_eq!(patterns.as_array().unwrap().len(), 1);
        }
    }
}

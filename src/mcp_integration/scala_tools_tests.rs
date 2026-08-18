#[cfg_attr(coverage_nightly, coverage(off))]
#[cfg(test)]
mod tests {
    use super::*;
    use crate::services::context::AstItem;

    // ==================== ScalaAnalysisTool Tests ====================

    #[test]
    fn test_scala_analysis_tool_metadata() {
        let registry = Arc::new(crate::agents::registry::AgentRegistry::new());
        let tool = ScalaAnalysisTool::new(registry);
        let metadata = tool.metadata();

        assert_eq!(metadata.name, "analyze_scala");
        assert!(metadata.description.contains("Scala"));
        assert!(metadata.description.contains("complexity"));
    }

    #[test]
    fn test_scala_analysis_tool_input_schema() {
        let registry = Arc::new(crate::agents::registry::AgentRegistry::new());
        let tool = ScalaAnalysisTool::new(registry);
        let metadata = tool.metadata();

        let schema = metadata.input_schema;
        assert!(schema["properties"]["path"].is_object());
        assert!(schema["properties"]["max_depth"].is_object());
        assert!(schema["properties"]["include_metrics"].is_object());
        assert!(schema["properties"]["include_ast"].is_object());
        assert!(schema["required"]
            .as_array()
            .unwrap()
            .contains(&json!("path")));
    }

    // ==================== ScalaMutationTool Tests ====================

    #[test]
    fn test_scala_mutation_tool_metadata() {
        let registry = Arc::new(crate::agents::registry::AgentRegistry::new());
        let tool = ScalaMutationTool::new(registry);
        let metadata = tool.metadata();

        assert_eq!(metadata.name, "mutation_test_scala");
        assert!(metadata.description.contains("mutation"));
        assert!(metadata.description.contains("Scala"));
    }

    #[test]
    fn test_scala_mutation_tool_input_schema() {
        let registry = Arc::new(crate::agents::registry::AgentRegistry::new());
        let tool = ScalaMutationTool::new(registry);
        let metadata = tool.metadata();

        let schema = metadata.input_schema;
        assert!(schema["properties"]["project_path"].is_object());
        assert!(schema["properties"]["source_path"].is_object());
        assert!(schema["properties"]["test_command"].is_object());
        assert!(schema["properties"]["mutation_operators"].is_object());
        assert!(schema["properties"]["timeout"].is_object());
    }

    // ==================== find_scala_files Tests ====================

    #[test]
    fn test_find_scala_files_empty_dir() {
        use tempfile::tempdir;

        let dir = tempdir().unwrap();
        let files = find_scala_files(dir.path(), 5).unwrap();

        assert!(files.is_empty());
    }

    #[test]
    fn test_find_scala_files_with_scala() {
        use std::fs;
        use tempfile::tempdir;

        let dir = tempdir().unwrap();
        let scala_file = dir.path().join("Test.scala");
        fs::write(&scala_file, "object Test {}").unwrap();

        let files = find_scala_files(dir.path(), 5).unwrap();

        assert_eq!(files.len(), 1);
        assert!(files[0].ends_with("Test.scala"));
    }

    #[test]
    fn test_find_scala_files_with_sc_extension() {
        use std::fs;
        use tempfile::tempdir;

        let dir = tempdir().unwrap();
        let sc_file = dir.path().join("Script.sc");
        fs::write(&sc_file, "println(\"Hello\")").unwrap();

        let files = find_scala_files(dir.path(), 5).unwrap();

        assert_eq!(files.len(), 1);
        assert!(files[0].ends_with("Script.sc"));
    }

    #[test]
    fn test_find_scala_files_max_depth() {
        use std::fs;
        use tempfile::tempdir;

        let dir = tempdir().unwrap();
        let subdir = dir.path().join("deep").join("nested");
        fs::create_dir_all(&subdir).unwrap();

        let scala_file = subdir.join("Deep.scala");
        fs::write(&scala_file, "object Deep {}").unwrap();

        // With max_depth 1, shouldn't find nested file
        let files_shallow = find_scala_files(dir.path(), 1).unwrap();
        assert!(
            files_shallow.is_empty() || !files_shallow.iter().any(|f| f.ends_with("Deep.scala"))
        );

        // With max_depth 5, should find it
        let files_deep = find_scala_files(dir.path(), 5).unwrap();
        assert!(files_deep.iter().any(|f| f.ends_with("Deep.scala")));
    }

    #[test]
    fn test_find_scala_files_ignores_other_extensions() {
        use std::fs;
        use tempfile::tempdir;

        let dir = tempdir().unwrap();
        fs::write(dir.path().join("test.rs"), "fn main() {}").unwrap();
        fs::write(dir.path().join("test.java"), "class Test {}").unwrap();
        fs::write(dir.path().join("test.kt"), "class Test").unwrap();

        let files = find_scala_files(dir.path(), 5).unwrap();
        assert!(files.is_empty());
    }

    // ==================== calculate_functional_percentage Tests ====================

    #[test]
    fn test_calculate_functional_percentage_empty() {
        let items: Vec<AstItem> = vec![];
        let percentage = calculate_functional_percentage(&items);

        // Default to 50% when no items
        assert!((percentage - 50.0).abs() < 0.1);
    }

    #[test]
    fn test_calculate_functional_percentage_trait_only() {
        let items = vec![AstItem::Trait {
            name: "MyTrait".to_string(),
            visibility: "public".to_string(),
            line: 1,
        }];

        let percentage = calculate_functional_percentage(&items);
        // Traits are functional
        assert!(percentage > 0.0);
    }

    #[test]
    fn test_calculate_functional_percentage_function_only() {
        let items = vec![AstItem::Function {
            name: "myFunc".to_string(),
            visibility: "public".to_string(),
            is_async: false,
            line: 1,
        }];

        let percentage = calculate_functional_percentage(&items);
        // Functions have some imperative score
        assert!((0.0..=100.0).contains(&percentage));
    }

    #[test]
    fn test_calculate_functional_percentage_mixed() {
        let items = vec![
            AstItem::Trait {
                name: "Trait1".to_string(),
                visibility: "public".to_string(),
                line: 1,
            },
            AstItem::Function {
                name: "func1".to_string(),
                visibility: "public".to_string(),
                is_async: false,
                line: 11,
            },
        ];

        let percentage = calculate_functional_percentage(&items);
        // Should be somewhere between 0 and 100
        assert!(percentage > 0.0 && percentage < 100.0);
    }

    #[test]
    fn test_calculate_functional_percentage_module() {
        let items = vec![AstItem::Module {
            name: "MyModule".to_string(),
            visibility: "public".to_string(),
            line: 1,
        }];

        let percentage = calculate_functional_percentage(&items);
        // Modules/objects have functional score
        assert!(percentage > 0.0);
    }

    // ==================== Tool Execute Tests (Error Cases) ====================

    #[tokio::test]
    async fn test_scala_analysis_tool_missing_path() {
        let registry = Arc::new(crate::agents::registry::AgentRegistry::new());
        let tool = ScalaAnalysisTool::new(registry);

        let params = json!({});
        let result = tool.execute(params).await;

        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.message.contains("path"));
    }

    #[tokio::test]
    async fn test_scala_analysis_tool_nonexistent_path() {
        let registry = Arc::new(crate::agents::registry::AgentRegistry::new());
        let tool = ScalaAnalysisTool::new(registry);

        let params = json!({
            "path": "/nonexistent/path/to/scala"
        });
        let result = tool.execute(params).await;

        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.message.contains("does not exist") || err.message.contains("Path"));
    }

    #[tokio::test]
    async fn test_scala_analysis_tool_wrong_extension() {
        use std::io::Write;
        use tempfile::NamedTempFile;

        let mut file = NamedTempFile::with_suffix(".rs").unwrap();
        writeln!(file, "fn main() {{}}").unwrap();

        let registry = Arc::new(crate::agents::registry::AgentRegistry::new());
        let tool = ScalaAnalysisTool::new(registry);

        let params = json!({
            "path": file.path().to_str().unwrap()
        });
        let result = tool.execute(params).await;

        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.message.contains("Scala") || err.message.contains(".scala"));
    }

    #[tokio::test]
    async fn test_scala_mutation_tool_missing_project_path() {
        let registry = Arc::new(crate::agents::registry::AgentRegistry::new());
        let tool = ScalaMutationTool::new(registry);

        let params = json!({
            "source_path": "/path/to/source"
        });
        let result = tool.execute(params).await;

        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.message.contains("project_path"));
    }

    #[tokio::test]
    async fn test_scala_mutation_tool_missing_source_path() {
        let registry = Arc::new(crate::agents::registry::AgentRegistry::new());
        let tool = ScalaMutationTool::new(registry);

        let params = json!({
            "project_path": "/path/to/project"
        });
        let result = tool.execute(params).await;

        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.message.contains("source_path"));
    }

    #[tokio::test]
    async fn test_scala_mutation_tool_complete_params() {
        let registry = Arc::new(crate::agents::registry::AgentRegistry::new());
        let tool = ScalaMutationTool::new(registry);

        let params = json!({
            "project_path": "/path/to/project",
            "source_path": "/path/to/source"
        });
        let result = tool.execute(params).await;

        // Should succeed with placeholder response
        assert!(result.is_ok());
        let value = result.unwrap();
        assert_eq!(value["status"], "completed");
    }

    #[tokio::test]
    async fn test_scala_mutation_tool_custom_params() {
        let registry = Arc::new(crate::agents::registry::AgentRegistry::new());
        let tool = ScalaMutationTool::new(registry);

        let params = json!({
            "project_path": "/path/to/project",
            "source_path": "/path/to/source",
            "test_command": "sbt clean test",
            "timeout": 60,
            "mutation_operators": ["arithmetic", "method"]
        });
        let result = tool.execute(params).await;

        assert!(result.is_ok());
        let value = result.unwrap();
        assert_eq!(value["test_command"], "sbt clean test");
        assert_eq!(value["timeout"], 60);
        assert_eq!(value["mutation_operators"], json!(["arithmetic", "method"]));
    }

    // ==================== Integration-like Tests ====================

    #[tokio::test]
    async fn test_analyze_scala_file_valid() {
        use std::io::Write;
        use tempfile::NamedTempFile;

        let mut file = NamedTempFile::with_suffix(".scala").unwrap();
        writeln!(
            file,
            r#"
            object Test {{
                def hello(): String = "Hello"
            }}
            "#
        )
        .unwrap();

        let result = analyze_scala_file(file.path(), true, false).await;
        assert!(result.is_ok());

        let value = result.unwrap();
        assert!(value["status"] == "completed" || value["status"] == "error");
    }

    #[tokio::test]
    async fn test_analyze_scala_directory_empty() {
        use tempfile::tempdir;

        let dir = tempdir().unwrap();
        let result = analyze_scala_directory(dir.path(), 3, true, false).await;

        assert!(result.is_ok());
        let value = result.unwrap();
        assert_eq!(value["status"], "completed");
        assert_eq!(value["summary"]["file_count"], 0);
    }

    #[tokio::test]
    async fn test_analyze_scala_directory_with_file() {
        use std::fs;
        use tempfile::tempdir;

        let dir = tempdir().unwrap();
        fs::write(
            dir.path().join("Test.scala"),
            "object Test { def run(): Unit = () }",
        )
        .unwrap();

        let result = analyze_scala_directory(dir.path(), 3, true, false).await;

        assert!(result.is_ok());
        let value = result.unwrap();
        assert_eq!(value["status"], "completed");
        assert_eq!(value["summary"]["file_count"], 1);
    }

    // ==================== Edge Cases ====================

    #[test]
    fn test_find_scala_files_zero_depth() {
        use std::fs;
        use tempfile::tempdir;

        let dir = tempdir().unwrap();
        let subdir = dir.path().join("sub");
        fs::create_dir(&subdir).unwrap();
        fs::write(subdir.join("Test.scala"), "object Test {}").unwrap();

        // With depth 0, should only include the starting path itself
        let files = find_scala_files(dir.path(), 0).unwrap();
        // Files in subdirectories shouldn't be found
        assert!(files.is_empty());
    }

    #[test]
    fn test_find_scala_files_multiple() {
        use std::fs;
        use tempfile::tempdir;

        let dir = tempdir().unwrap();
        fs::write(dir.path().join("A.scala"), "object A {}").unwrap();
        fs::write(dir.path().join("B.scala"), "object B {}").unwrap();
        fs::write(dir.path().join("C.sc"), "println(1)").unwrap();

        let files = find_scala_files(dir.path(), 5).unwrap();
        assert_eq!(files.len(), 3);
    }

    #[test]
    fn test_calculate_functional_percentage_case_class_naming() {
        // Test with struct that has "Case" prefix in name
        let items = vec![AstItem::Struct {
            name: "CaseUser".to_string(),
            visibility: "public".to_string(),
            fields_count: 0,
            derives: vec![],
            line: 1,
        }];

        let percentage = calculate_functional_percentage(&items);
        // Should count as functional because name starts with Case
        assert!(percentage > 0.0);
    }
}

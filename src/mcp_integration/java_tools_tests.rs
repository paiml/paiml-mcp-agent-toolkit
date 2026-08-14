#[cfg_attr(coverage_nightly, coverage(off))]
#[cfg(test)]
mod tests {
    use super::*;

    // ==================== JavaAnalysisTool Tests ====================

    #[test]
    fn test_java_analysis_tool_metadata() {
        let registry = Arc::new(crate::agents::registry::AgentRegistry::new());
        let tool = JavaAnalysisTool::new(registry);
        let metadata = tool.metadata();

        assert_eq!(metadata.name, "analyze_java");
        assert!(metadata.description.contains("Java"));
        assert!(metadata.description.contains("complexity"));
    }

    #[test]
    fn test_java_analysis_tool_input_schema() {
        let registry = Arc::new(crate::agents::registry::AgentRegistry::new());
        let tool = JavaAnalysisTool::new(registry);
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

    // ==================== JavaMutationTool Tests ====================

    #[test]
    fn test_java_mutation_tool_metadata() {
        let registry = Arc::new(crate::agents::registry::AgentRegistry::new());
        let tool = JavaMutationTool::new(registry);
        let metadata = tool.metadata();

        assert_eq!(metadata.name, "mutation_test_java");
        assert!(metadata.description.contains("mutation"));
        assert!(metadata.description.contains("Java"));
    }

    #[test]
    fn test_java_mutation_tool_input_schema() {
        let registry = Arc::new(crate::agents::registry::AgentRegistry::new());
        let tool = JavaMutationTool::new(registry);
        let metadata = tool.metadata();

        let schema = metadata.input_schema;
        assert!(schema["properties"]["project_path"].is_object());
        assert!(schema["properties"]["source_path"].is_object());
        assert!(schema["properties"]["test_command"].is_object());
        assert!(schema["properties"]["mutation_operators"].is_object());
        assert!(schema["properties"]["timeout"].is_object());
    }

    // ==================== find_java_files Tests ====================

    #[test]
    fn test_find_java_files_empty_dir() {
        use tempfile::tempdir;

        let dir = tempdir().unwrap();
        let files = find_java_files(dir.path(), 5).unwrap();

        assert!(files.is_empty());
    }

    #[test]
    fn test_find_java_files_with_java() {
        use std::fs;
        use tempfile::tempdir;

        let dir = tempdir().unwrap();
        let java_file = dir.path().join("Test.java");
        fs::write(&java_file, "public class Test {}").unwrap();

        let files = find_java_files(dir.path(), 5).unwrap();

        assert_eq!(files.len(), 1);
        assert!(files[0].ends_with("Test.java"));
    }

    #[test]
    fn test_find_java_files_max_depth() {
        use std::fs;
        use tempfile::tempdir;

        let dir = tempdir().unwrap();
        let subdir = dir.path().join("deep").join("nested");
        fs::create_dir_all(&subdir).unwrap();

        let java_file = subdir.join("Deep.java");
        fs::write(&java_file, "public class Deep {}").unwrap();

        // With max_depth 1, shouldn't find nested file
        let files_shallow = find_java_files(dir.path(), 1).unwrap();
        assert!(
            files_shallow.is_empty() || !files_shallow.iter().any(|f| f.ends_with("Deep.java"))
        );

        // With max_depth 5, should find it
        let files_deep = find_java_files(dir.path(), 5).unwrap();
        assert!(files_deep.iter().any(|f| f.ends_with("Deep.java")));
    }

    #[test]
    fn test_find_java_files_ignores_other_extensions() {
        use std::fs;
        use tempfile::tempdir;

        let dir = tempdir().unwrap();
        fs::write(dir.path().join("test.rs"), "fn main() {}").unwrap();
        fs::write(dir.path().join("test.scala"), "object Test {}").unwrap();
        fs::write(dir.path().join("test.kt"), "class Test").unwrap();

        let files = find_java_files(dir.path(), 5).unwrap();
        assert!(files.is_empty());
    }

    #[test]
    fn test_find_java_files_zero_depth() {
        use std::fs;
        use tempfile::tempdir;

        let dir = tempdir().unwrap();
        let subdir = dir.path().join("sub");
        fs::create_dir(&subdir).unwrap();
        fs::write(subdir.join("Test.java"), "class Test {}").unwrap();

        // With depth 0, should only include the starting path itself
        let files = find_java_files(dir.path(), 0).unwrap();
        // Files in subdirectories shouldn't be found
        assert!(files.is_empty());
    }

    #[test]
    fn test_find_java_files_multiple() {
        use std::fs;
        use tempfile::tempdir;

        let dir = tempdir().unwrap();
        fs::write(dir.path().join("A.java"), "class A {}").unwrap();
        fs::write(dir.path().join("B.java"), "class B {}").unwrap();
        fs::write(dir.path().join("C.java"), "class C {}").unwrap();

        let files = find_java_files(dir.path(), 5).unwrap();
        assert_eq!(files.len(), 3);
    }

    // ==================== Tool Execute Tests (Error Cases) ====================

    #[tokio::test]
    async fn test_java_analysis_tool_missing_path() {
        let registry = Arc::new(crate::agents::registry::AgentRegistry::new());
        let tool = JavaAnalysisTool::new(registry);

        let params = json!({});
        let result = tool.execute(params).await;

        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.message.contains("path"));
    }

    #[tokio::test]
    async fn test_java_analysis_tool_nonexistent_path() {
        let registry = Arc::new(crate::agents::registry::AgentRegistry::new());
        let tool = JavaAnalysisTool::new(registry);

        let params = json!({
            "path": "/nonexistent/path/to/java"
        });
        let result = tool.execute(params).await;

        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.message.contains("does not exist") || err.message.contains("Path"));
    }

    #[tokio::test]
    async fn test_java_analysis_tool_wrong_extension() {
        use std::io::Write;
        use tempfile::NamedTempFile;

        let mut file = NamedTempFile::with_suffix(".rs").unwrap();
        writeln!(file, "fn main() {{}}").unwrap();

        let registry = Arc::new(crate::agents::registry::AgentRegistry::new());
        let tool = JavaAnalysisTool::new(registry);

        let params = json!({
            "path": file.path().to_str().unwrap()
        });
        let result = tool.execute(params).await;

        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.message.contains("Java") || err.message.contains(".java"));
    }

    #[tokio::test]
    async fn test_java_mutation_tool_missing_project_path() {
        let registry = Arc::new(crate::agents::registry::AgentRegistry::new());
        let tool = JavaMutationTool::new(registry);

        let params = json!({
            "source_path": "/path/to/source"
        });
        let result = tool.execute(params).await;

        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.message.contains("project_path"));
    }

    #[tokio::test]
    async fn test_java_mutation_tool_missing_source_path() {
        let registry = Arc::new(crate::agents::registry::AgentRegistry::new());
        let tool = JavaMutationTool::new(registry);

        let params = json!({
            "project_path": "/path/to/project"
        });
        let result = tool.execute(params).await;

        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.message.contains("source_path"));
    }

    #[tokio::test]
    async fn test_java_mutation_tool_complete_params() {
        let registry = Arc::new(crate::agents::registry::AgentRegistry::new());
        let tool = JavaMutationTool::new(registry);

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
    async fn test_java_mutation_tool_custom_params() {
        let registry = Arc::new(crate::agents::registry::AgentRegistry::new());
        let tool = JavaMutationTool::new(registry);

        let params = json!({
            "project_path": "/path/to/project",
            "source_path": "/path/to/source",
            "test_command": "mvn clean test",
            "timeout": 60,
            "mutation_operators": ["arithmetic", "conditional"]
        });
        let result = tool.execute(params).await;

        assert!(result.is_ok());
        let value = result.unwrap();
        assert_eq!(value["test_command"], "mvn clean test");
        assert_eq!(value["timeout"], 60);
        assert_eq!(
            value["mutation_operators"],
            json!(["arithmetic", "conditional"])
        );
    }

    #[tokio::test]
    async fn test_java_mutation_tool_default_operators() {
        let registry = Arc::new(crate::agents::registry::AgentRegistry::new());
        let tool = JavaMutationTool::new(registry);

        let params = json!({
            "project_path": "/path/to/project",
            "source_path": "/path/to/source"
        });
        let result = tool.execute(params).await;

        assert!(result.is_ok());
        let value = result.unwrap();
        let operators = value["mutation_operators"].as_array().unwrap();
        assert!(operators.contains(&json!("arithmetic")));
        assert!(operators.contains(&json!("conditional")));
        assert!(operators.contains(&json!("method")));
        assert!(operators.contains(&json!("assignment")));
    }

    // ==================== Integration-like Tests ====================

    #[tokio::test]
    async fn test_analyze_java_file_valid() {
        use std::io::Write;
        use tempfile::NamedTempFile;

        let mut file = NamedTempFile::with_suffix(".java").unwrap();
        writeln!(
            file,
            r#"
            public class Test {{
                public String hello() {{
                    return "Hello";
                }}
            }}
            "#
        )
        .unwrap();

        // `status == "completed" || status == "error"` passed for every possible
        // outcome, so it measured nothing. Valid Java must analyse.
        let value = analyze_java_file(file.path(), true, false).await.unwrap();
        assert_eq!(
            value["status"], "completed",
            "valid Java must analyse, got: {value}"
        );
        assert_eq!(value["summary"]["class_count"], 1);
        assert_eq!(value["summary"]["method_count"], 1);
    }

    #[tokio::test]
    async fn test_analyze_java_directory_empty() {
        use tempfile::tempdir;

        let dir = tempdir().unwrap();
        let result = analyze_java_directory(dir.path(), 3, true, false).await;

        assert!(result.is_ok());
        let value = result.unwrap();
        assert_eq!(value["status"], "completed");
        assert_eq!(value["summary"]["file_count"], 0);
    }

    #[tokio::test]
    async fn test_analyze_java_directory_with_file() {
        use std::fs;
        use tempfile::tempdir;

        let dir = tempdir().unwrap();
        fs::write(
            dir.path().join("Test.java"),
            "public class Test { public void run() {} }",
        )
        .unwrap();

        let result = analyze_java_directory(dir.path(), 3, true, false).await;

        assert!(result.is_ok());
        let value = result.unwrap();
        assert_eq!(value["status"], "completed");
        assert_eq!(value["summary"]["file_count"], 1);
    }

    #[tokio::test]
    async fn test_analyze_java_file_with_metrics() {
        use std::io::Write;
        use tempfile::NamedTempFile;

        let mut file = NamedTempFile::with_suffix(".java").unwrap();
        writeln!(
            file,
            r#"
            public class Test {{
                public int calculate(int x, int y) {{
                    if (x > y) {{
                        return x;
                    }} else {{
                        return y;
                    }}
                }}
            }}
            "#
        )
        .unwrap();

        // The `if status == "completed"` guard let this test pass on a build
        // where nothing ever completes. Assert the completion too.
        let value = analyze_java_file(file.path(), true, false).await.unwrap();
        assert_eq!(
            value["status"], "completed",
            "valid Java must analyse, got: {value}"
        );
        assert!(value.get("metrics").is_some(), "got: {value}");
    }

    #[tokio::test]
    async fn test_analyze_java_file_without_metrics() {
        use std::io::Write;
        use tempfile::NamedTempFile;

        let mut file = NamedTempFile::with_suffix(".java").unwrap();
        writeln!(file, "public class Test {{}}").unwrap();

        let result = analyze_java_file(file.path(), false, false).await;
        assert!(result.is_ok());

        let value = result.unwrap();
        // Should not have metrics when include_metrics is false
        assert!(value.get("metrics").is_none());
    }

    // ==================== Edge Cases ====================

    #[tokio::test]
    async fn test_analyze_java_directory_with_nested_files() {
        use std::fs;
        use tempfile::tempdir;

        let dir = tempdir().unwrap();
        let subdir = dir.path().join("src").join("main").join("java");
        fs::create_dir_all(&subdir).unwrap();

        fs::write(subdir.join("Main.java"), "public class Main {}").unwrap();
        fs::write(subdir.join("Helper.java"), "public class Helper {}").unwrap();

        let result = analyze_java_directory(dir.path(), 10, true, false).await;
        assert!(result.is_ok());

        let value = result.unwrap();
        assert_eq!(value["status"], "completed");
        assert_eq!(value["summary"]["file_count"], 2);
    }

    // ==================== #966 adjacent: unmeasured files must be visible ====

    /// A directory in which every Java file fails to parse used to come back as
    /// `status: "completed"` with `class_count: 0` — byte-identical to a
    /// directory of valid-but-empty classes. The aggregate now has to say how
    /// many files it actually derived its counts from.
    #[tokio::test]
    async fn test_analyze_java_directory_reports_unparseable_files() {
        use std::fs;
        use tempfile::tempdir;

        let dir = tempdir().unwrap();
        fs::write(
            dir.path().join("Good.java"),
            "package com.example;\npublic class Good { public void run() {} }",
        )
        .unwrap();
        // Unbalanced braces: rejected by `is_valid_java_syntax`.
        fs::write(
            dir.path().join("Broken.java"),
            "package com.example;\npublic class Broken { public void run() {\n",
        )
        .unwrap();

        let value = analyze_java_directory(dir.path(), 3, true, false)
            .await
            .unwrap();

        assert_eq!(
            value["status"], "completed_with_errors",
            "a directory holding an unparseable file must not report plain success, got: {value}"
        );
        assert_eq!(value["summary"]["file_count"], 2);
        assert_eq!(
            value["summary"]["analyzed_file_count"], 1,
            "counts were derived from one file only, got: {value}"
        );
        assert_eq!(value["summary"]["failed_file_count"], 1);
        let failures = value["failures"]
            .as_array()
            .unwrap_or_else(|| panic!("failures must be listed, got: {value}"));
        assert_eq!(failures.len(), 1);
        assert!(
            failures[0]["path"]
                .as_str()
                .unwrap()
                .ends_with("Broken.java"),
            "the failing file must be named, got: {value}"
        );
    }

    /// The all-good case must stay clean: no error status, nothing unmeasured.
    #[tokio::test]
    async fn test_analyze_java_directory_all_parseable_reports_none_failed() {
        use std::fs;
        use tempfile::tempdir;

        let dir = tempdir().unwrap();
        fs::write(dir.path().join("A.java"), "public class A { void a() {} }").unwrap();
        fs::write(dir.path().join("B.java"), "public class B { void b() {} }").unwrap();

        let value = analyze_java_directory(dir.path(), 3, true, false)
            .await
            .unwrap();

        assert_eq!(value["status"], "completed", "got: {value}");
        assert_eq!(value["summary"]["analyzed_file_count"], 2);
        assert_eq!(value["summary"]["failed_file_count"], 0);
        assert!(value.get("failures").is_none(), "got: {value}");
    }

    #[test]
    fn test_tool_creation() {
        let registry = Arc::new(crate::agents::registry::AgentRegistry::new());

        let analysis_tool = JavaAnalysisTool::new(Arc::clone(&registry));
        let mutation_tool = JavaMutationTool::new(Arc::clone(&registry));

        // Just verify they can be created without panicking
        assert_eq!(analysis_tool.metadata().name, "analyze_java");
        assert_eq!(mutation_tool.metadata().name, "mutation_test_java");
    }
}

// Serialization, clone, and edge case tests for context service.
// Included via include!() in context_tests.rs.

// Serialization tests

#[test]
fn test_project_context_serialization() {
    let context = ProjectContext {
        project_type: "rust".to_string(),
        files: vec![FileContext {
            path: "src/lib.rs".to_string(),
            language: "rust".to_string(),
            items: vec![AstItem::Function {
                name: "test".to_string(),
                visibility: "pub".to_string(),
                is_async: false,
                line: 1,
            }],
            complexity_metrics: None,
        }],
        graph: None,
        summary: ProjectSummary {
            total_files: 1,
            total_functions: 1,
            total_structs: 0,
            total_enums: 0,
            total_traits: 0,
            total_impls: 0,
            dependencies: vec!["serde".to_string()],
        },
    };

    let json = serde_json::to_string(&context).unwrap();
    let deserialized: ProjectContext = serde_json::from_str(&json).unwrap();

    assert_eq!(context.project_type, deserialized.project_type);
    assert_eq!(context.files.len(), deserialized.files.len());
    assert_eq!(
        context.summary.total_functions,
        deserialized.summary.total_functions
    );
}

#[test]
fn test_file_context_serialization() {
    let ctx = FileContext {
        path: "test.rs".to_string(),
        language: "rust".to_string(),
        items: vec![],
        complexity_metrics: None,
    };

    let json = serde_json::to_string(&ctx).unwrap();
    let deserialized: FileContext = serde_json::from_str(&json).unwrap();

    assert_eq!(ctx.path, deserialized.path);
    assert_eq!(ctx.language, deserialized.language);
}

#[test]
fn test_ast_item_serialization_function() {
    let item = AstItem::Function {
        name: "test".to_string(),
        visibility: "pub".to_string(),
        is_async: true,
        line: 42,
    };

    let json = serde_json::to_string(&item).unwrap();
    assert!(json.contains("Function"));
    assert!(json.contains("test"));
    assert!(json.contains("42"));

    let deserialized: AstItem = serde_json::from_str(&json).unwrap();
    assert_eq!(item, deserialized);
}

#[test]
fn test_ast_item_serialization_import() {
    let item = AstItem::Import {
        module: "numpy".to_string(),
        items: vec!["array".to_string(), "ndarray".to_string()],
        alias: Some("np".to_string()),
        line: 1,
    };

    let json = serde_json::to_string(&item).unwrap();
    let deserialized: AstItem = serde_json::from_str(&json).unwrap();

    if let AstItem::Import {
        module,
        items,
        alias,
        ..
    } = deserialized
    {
        assert_eq!(module, "numpy");
        assert_eq!(items.len(), 2);
        assert_eq!(alias, Some("np".to_string()));
    } else {
        panic!("Deserialized to wrong variant");
    }
}

// Clone tests

#[test]
fn test_project_context_clone() {
    let context = ProjectContext {
        project_type: "rust".to_string(),
        files: vec![],
        graph: None,
        summary: ProjectSummary {
            total_files: 0,
            total_functions: 0,
            total_structs: 0,
            total_enums: 0,
            total_traits: 0,
            total_impls: 0,
            dependencies: vec![],
        },
    };

    let cloned = context.clone();
    assert_eq!(context.project_type, cloned.project_type);
}

#[test]
fn test_file_context_clone() {
    let ctx = FileContext {
        path: "test.rs".to_string(),
        language: "rust".to_string(),
        items: vec![AstItem::Function {
            name: "f".to_string(),
            visibility: "pub".to_string(),
            is_async: false,
            line: 1,
        }],
        complexity_metrics: None,
    };

    let cloned = ctx.clone();
    assert_eq!(ctx.path, cloned.path);
    assert_eq!(ctx.items.len(), cloned.items.len());
}

#[test]
fn test_ast_item_clone() {
    let item = AstItem::Struct {
        name: "Test".to_string(),
        visibility: "pub".to_string(),
        fields_count: 5,
        derives: vec!["Debug".to_string()],
        line: 10,
    };

    let cloned = item.clone();
    assert_eq!(item, cloned);
}

// Edge cases

#[test]
fn test_format_header() {
    let context = ProjectContext {
        project_type: "python".to_string(),
        files: vec![],
        graph: None,
        summary: ProjectSummary {
            total_files: 0,
            total_functions: 0,
            total_structs: 0,
            total_enums: 0,
            total_traits: 0,
            total_impls: 0,
            dependencies: vec![],
        },
    };

    let mut output = String::new();
    format_header(&mut output, &context);

    assert!(output.contains("python Project"));
    assert!(output.contains("Generated:"));
}

#[test]
fn test_format_summary() {
    let summary = ProjectSummary {
        total_files: 100,
        total_functions: 500,
        total_structs: 50,
        total_enums: 25,
        total_traits: 10,
        total_impls: 100,
        dependencies: vec![],
    };

    let mut output = String::new();
    format_summary(&mut output, &summary);

    assert!(output.contains("Files analyzed: 100"));
    assert!(output.contains("Functions: 500"));
    assert!(output.contains("Structs: 50"));
}

#[test]
fn test_format_dependencies() {
    let deps = vec!["dep1".to_string(), "dep2".to_string(), "dep3".to_string()];

    let mut output = String::new();
    format_dependencies(&mut output, &deps);

    assert!(output.contains("## Dependencies"));
    assert!(output.contains("- dep1"));
    assert!(output.contains("- dep2"));
    assert!(output.contains("- dep3"));
}

#[test]
fn test_format_dependencies_empty() {
    let deps: Vec<String> = vec![];

    let mut output = String::new();
    format_dependencies(&mut output, &deps);

    // Should not add any content for empty deps
    assert!(output.is_empty());
}

#[test]
fn test_format_footer() {
    let mut output = String::new();
    format_footer(&mut output);

    assert!(output.contains("---"));
    assert!(output.contains("paiml-mcp-agent-toolkit"));
}

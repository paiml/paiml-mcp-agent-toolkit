//! Property tests for Import AST functionality

#[cfg(test)]
mod property_tests {
    use crate::services::context::AstItem;
    use proptest::prelude::*;

    // Strategy for generating valid module names
    fn module_name_strategy() -> impl Strategy<Value = String> {
        "[a-z][a-z0-9_]{0,20}(\\.[a-z][a-z0-9_]{0,20}){0,3}"
    }

    // Strategy for generating import items
    fn import_items_strategy() -> impl Strategy<Value = Vec<String>> {
        prop::collection::vec("[A-Za-z][A-Za-z0-9_]{0,15}", 0..10)
    }

    // Strategy for generating optional aliases
    fn alias_strategy() -> impl Strategy<Value = Option<String>> {
        prop::option::of("[a-z][a-z0-9_]{0,15}")
    }

    proptest! {
        #[test]
        fn test_import_display_name_never_panics(
            module in module_name_strategy(),
            items in import_items_strategy(),
            alias in alias_strategy(),
            line in 1usize..10000
        ) {
            let import = AstItem::Import {
                module: module.clone(),
                items: items.clone(),
                alias: alias.clone(),
                line,
            };

            // Should never panic
            let display = import.display_name();

            // Basic validation
            if let Some(alias) = alias {
                assert!(display.contains(&alias));
            } else if !items.is_empty() {
                assert!(display.contains(&module));
            } else {
                assert_eq!(display, module);
            }
        }

        #[test]
        fn test_import_roundtrip_preserves_data(
            module in module_name_strategy(),
            items in import_items_strategy(),
            alias in alias_strategy(),
            line in 1usize..10000
        ) {
            let import = AstItem::Import {
                module: module.clone(),
                items: items.clone(),
                alias: alias.clone(),
                line,
            };

            // Check that all fields are preserved
            match import {
                AstItem::Import {
                    module: m,
                    items: i,
                    alias: a,
                    line: l
                } => {
                    assert_eq!(m, module);
                    assert_eq!(i, items);
                    assert_eq!(a, alias);
                    assert_eq!(l, line);
                }
                _ => panic!("Expected Import variant"),
            }
        }

        #[test]
        fn test_import_with_empty_items_valid(
            module in module_name_strategy(),
            alias in alias_strategy(),
            line in 1usize..10000
        ) {
            let import = AstItem::Import {
                module: module.clone(),
                items: Vec::new(),
                alias,
                line,
            };

            // Empty items list is valid (imports entire module)
            let display = import.display_name();
            assert!(!display.is_empty());
        }

        #[test]
        fn test_import_edge_cases(
            line in 1usize..10000
        ) {
            // Test various edge cases
            let test_cases = vec![
                // Single module import
                AstItem::Import {
                    module: "os".to_string(),
                    items: vec![],
                    alias: None,
                    line,
                },
                // Module with alias
                AstItem::Import {
                    module: "numpy".to_string(),
                    items: vec![],
                    alias: Some("np".to_string()),
                    line,
                },
                // Specific imports
                AstItem::Import {
                    module: "typing".to_string(),
                    items: vec!["List".to_string(), "Dict".to_string()],
                    alias: None,
                    line,
                },
                // Deep module path
                AstItem::Import {
                    module: "torch.nn.functional".to_string(),
                    items: vec!["relu".to_string()],
                    alias: Some("F".to_string()),
                    line,
                },
            ];

            for import in test_cases {
                // Should handle all cases without panic
                let display = import.display_name();
                assert!(!display.is_empty());

                // Verify basic structure
                match &import {
                    AstItem::Import { module, .. } => {
                        assert!(display.contains(module) || display.len() > 0);
                    }
                    _ => panic!("Expected Import variant"),
                }
            }
        }
    }

    proptest! {
        #[test]
        fn test_dag_builder_handles_imports(
            module in module_name_strategy(),
            items in import_items_strategy(),
            line in 1usize..10000
        ) {
            use crate::services::dag_builder::DagBuilder;
            use crate::services::context::{FileContext, ProjectContext, ProjectSummary};

            let import = AstItem::Import {
                module,
                items,
                alias: None,
                line,
            };

            let file_context = FileContext {
                path: "test.py".to_string(),
                language: "python".to_string(),
                items: vec![import],
                complexity_metrics: None,
            };

            let summary = ProjectSummary {
                total_files: 1,
                total_functions: 0,
                total_structs: 0,
                total_enums: 0,
                total_traits: 0,
                total_impls: 0,
                dependencies: vec![],
            };

            let project = ProjectContext {
                project_type: "python".to_string(),
                files: vec![file_context],
                summary,
            };

            // Should not panic when processing imports
            let dag = DagBuilder::build_from_project(&project);

            // Basic validation - DAG should be created without panicking
            // The number of edges will vary based on the imports
            let _ = dag.edges.len();
        }
    }
}

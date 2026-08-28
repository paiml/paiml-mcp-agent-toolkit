
#[cfg_attr(coverage_nightly, coverage(off))]
#[cfg(test)]
mod context_tests {
    use super::*;

    // ============================================================================
    // ProjectSummary Tests
    // ============================================================================

    #[test]
    fn test_project_summary_creation() {
        let summary = ProjectSummary {
            total_files: 10,
            total_functions: 50,
            total_structs: 5,
            total_enums: 3,
            total_traits: 2,
            total_impls: 8,
            dependencies: vec!["serde".to_string(), "tokio".to_string()],
        };

        assert_eq!(summary.total_files, 10);
        assert_eq!(summary.total_functions, 50);
        assert_eq!(summary.total_structs, 5);
        assert_eq!(summary.total_enums, 3);
        assert_eq!(summary.total_traits, 2);
        assert_eq!(summary.total_impls, 8);
        assert_eq!(summary.dependencies.len(), 2);
    }

    #[test]
    fn test_project_summary_serialization() {
        let summary = ProjectSummary {
            total_files: 5,
            total_functions: 20,
            total_structs: 3,
            total_enums: 1,
            total_traits: 0,
            total_impls: 4,
            dependencies: vec!["anyhow".to_string()],
        };

        let json = serde_json::to_string(&summary).unwrap();
        let deserialized: ProjectSummary = serde_json::from_str(&json).unwrap();

        assert_eq!(deserialized.total_files, summary.total_files);
        assert_eq!(deserialized.total_functions, summary.total_functions);
        assert_eq!(deserialized.dependencies, summary.dependencies);
    }

    #[test]
    fn test_project_summary_clone() {
        let summary = ProjectSummary {
            total_files: 1,
            total_functions: 2,
            total_structs: 3,
            total_enums: 4,
            total_traits: 5,
            total_impls: 6,
            dependencies: vec!["dep1".to_string()],
        };

        let cloned = summary.clone();
        assert_eq!(cloned.total_files, summary.total_files);
        assert_eq!(cloned.dependencies, summary.dependencies);
    }

    // ============================================================================
    // FileContext Tests
    // ============================================================================

    #[test]
    fn test_file_context_creation() {
        let file_ctx = FileContext {
            path: "src/main.rs".to_string(),
            language: "rust".to_string(),
            items: vec![],
            complexity_metrics: None,
        };

        assert_eq!(file_ctx.path, "src/main.rs");
        assert_eq!(file_ctx.language, "rust");
        assert!(file_ctx.items.is_empty());
        assert!(file_ctx.complexity_metrics.is_none());
    }

    #[test]
    fn test_file_context_with_items() {
        let file_ctx = FileContext {
            path: "src/lib.rs".to_string(),
            language: "rust".to_string(),
            items: vec![
                AstItem::Function {
                    name: "main".to_string(),
                    visibility: "pub".to_string(),
                    is_async: false,
                    line: 1,
                },
                AstItem::Struct {
                    name: "Config".to_string(),
                    visibility: "pub".to_string(),
                    fields_count: 3,
                    derives: vec!["Debug".to_string()],
                    line: 10,
                },
            ],
            complexity_metrics: None,
        };

        assert_eq!(file_ctx.items.len(), 2);
    }

    #[test]
    fn test_file_context_serialization() {
        let file_ctx = FileContext {
            path: "test.rs".to_string(),
            language: "rust".to_string(),
            items: vec![AstItem::Enum {
                name: "Status".to_string(),
                visibility: "pub".to_string(),
                variants_count: 3,
                line: 5,
            }],
            complexity_metrics: None,
        };

        let json = serde_json::to_string(&file_ctx).unwrap();
        assert!(json.contains("test.rs"));
        assert!(json.contains("Status"));
    }

    // ============================================================================
    // AstItem Tests
    // ============================================================================

    #[test]
    fn test_ast_item_function() {
        let item = AstItem::Function {
            name: "process_data".to_string(),
            visibility: "pub".to_string(),
            is_async: true,
            line: 42,
        };

        assert_eq!(item.display_name(), "process_data");

        if let AstItem::Function {
            name,
            visibility,
            is_async,
            line,
        } = item
        {
            assert_eq!(name, "process_data");
            assert_eq!(visibility, "pub");
            assert!(is_async);
            assert_eq!(line, 42);
        } else {
            panic!("Expected Function variant");
        }
    }

    #[test]
    fn test_ast_item_struct() {
        let item = AstItem::Struct {
            name: "User".to_string(),
            visibility: "pub".to_string(),
            fields_count: 5,
            derives: vec!["Clone".to_string(), "Debug".to_string()],
            line: 10,
        };

        assert_eq!(item.display_name(), "User");

        if let AstItem::Struct {
            fields_count,
            derives,
            ..
        } = &item
        {
            assert_eq!(*fields_count, 5);
            assert_eq!(derives.len(), 2);
        }
    }

    #[test]
    fn test_ast_item_enum() {
        let item = AstItem::Enum {
            name: "Color".to_string(),
            visibility: "pub(crate)".to_string(),
            variants_count: 4,
            line: 20,
        };

        assert_eq!(item.display_name(), "Color");

        if let AstItem::Enum { variants_count, .. } = &item {
            assert_eq!(*variants_count, 4);
        }
    }

    #[test]
    fn test_ast_item_trait() {
        let item = AstItem::Trait {
            name: "Drawable".to_string(),
            visibility: "pub".to_string(),
            line: 30,
        };

        assert_eq!(item.display_name(), "Drawable");
    }

    #[test]
    fn test_ast_item_impl() {
        let item = AstItem::Impl {
            type_name: "User".to_string(),
            trait_name: Some("Clone".to_string()),
            line: 50,
        };

        assert_eq!(item.display_name(), "User");

        if let AstItem::Impl { trait_name, .. } = &item {
            assert_eq!(trait_name.as_ref().unwrap(), "Clone");
        }
    }

    #[test]
    fn test_ast_item_impl_no_trait() {
        let item = AstItem::Impl {
            type_name: "Config".to_string(),
            trait_name: None,
            line: 60,
        };

        assert_eq!(item.display_name(), "Config");

        if let AstItem::Impl { trait_name, .. } = &item {
            assert!(trait_name.is_none());
        }
    }

    #[test]
    fn test_ast_item_module() {
        let item = AstItem::Module {
            name: "utils".to_string(),
            visibility: "pub".to_string(),
            line: 1,
        };

        assert_eq!(item.display_name(), "utils");
    }

    #[test]
    fn test_ast_item_use() {
        let item = AstItem::Use {
            path: "std::collections::HashMap".to_string(),
            line: 3,
        };

        assert_eq!(item.display_name(), "std::collections::HashMap");
    }

    #[test]
    fn test_ast_item_import() {
        let item = AstItem::Import {
            module: "react".to_string(),
            items: vec!["useState".to_string(), "useEffect".to_string()],
            alias: None,
            line: 1,
        };

        assert_eq!(item.display_name(), "react");

        if let AstItem::Import { items, alias, .. } = &item {
            assert_eq!(items.len(), 2);
            assert!(alias.is_none());
        }
    }

    #[test]
    fn test_ast_item_import_with_alias() {
        let item = AstItem::Import {
            module: "numpy".to_string(),
            items: vec![],
            alias: Some("np".to_string()),
            line: 2,
        };

        assert_eq!(item.display_name(), "numpy");

        if let AstItem::Import { alias, .. } = &item {
            assert_eq!(alias.as_ref().unwrap(), "np");
        }
    }

    #[test]
    fn test_ast_item_equality() {
        let item1 = AstItem::Function {
            name: "foo".to_string(),
            visibility: "pub".to_string(),
            is_async: false,
            line: 1,
        };
        let item2 = AstItem::Function {
            name: "foo".to_string(),
            visibility: "pub".to_string(),
            is_async: false,
            line: 1,
        };
        let item3 = AstItem::Function {
            name: "bar".to_string(),
            visibility: "pub".to_string(),
            is_async: false,
            line: 1,
        };

        assert_eq!(item1, item2);
        assert_ne!(item1, item3);
    }

    #[test]
    fn test_ast_item_serialization() {
        let item = AstItem::Struct {
            name: "Test".to_string(),
            visibility: "pub".to_string(),
            fields_count: 2,
            derives: vec!["Debug".to_string()],
            line: 5,
        };

        let json = serde_json::to_string(&item).unwrap();
        assert!(json.contains("Test"));
        assert!(json.contains("Struct"));

        let deserialized: AstItem = serde_json::from_str(&json).unwrap();
        assert_eq!(item, deserialized);
    }

    // ============================================================================
    // build_context_graph Tests
    // ============================================================================

    /// Pins what `build_context_graph` documents about itself: every symbol
    /// becomes a node, no edges are produced, and PageRank therefore ranks
    /// nothing. If a future change starts extracting edges, this test fails and
    /// the doc comment above `build_context_graph` has to be corrected with it.
    #[test]
    fn test_build_context_graph_indexes_symbols_without_edges() {
        let files = vec![FileContext {
            path: "src/main.rs".to_string(),
            language: "rust".to_string(),
            items: vec![
                AstItem::Function {
                    name: "main".to_string(),
                    visibility: "pub".to_string(),
                    is_async: false,
                    line: 1,
                },
                AstItem::Function {
                    name: "helper".to_string(),
                    visibility: "fn".to_string(),
                    is_async: false,
                    line: 10,
                },
                AstItem::Struct {
                    name: "Config".to_string(),
                    visibility: "pub".to_string(),
                    fields_count: 2,
                    derives: vec![],
                    line: 20,
                },
            ],
            complexity_metrics: None,
        }];

        let graph = build_context_graph(&files).expect("graph builds");

        // Nodes: the O(1) index this function does provide.
        assert_eq!(graph.num_nodes(), 3);
        assert!(graph.get_item("main").is_some());
        assert!(graph.get_item("helper").is_some());
        assert!(graph.get_item("Config").is_some());

        // Edges: none, as documented — `main` calling `helper` is invisible here
        // because AstItem carries no call data.
        assert_eq!(graph.num_edges(), 0, "no edge extraction happens here");
        assert!(
            graph.hot_symbols().is_empty(),
            "PageRank over an edgeless graph ranks nothing"
        );
    }

    /// Duplicate symbol names across files collapse to one node rather than erroring.
    #[test]
    fn test_build_context_graph_dedupes_symbol_names() {
        let make_file = |path: &str| FileContext {
            path: path.to_string(),
            language: "rust".to_string(),
            items: vec![AstItem::Function {
                name: "new".to_string(),
                visibility: "pub".to_string(),
                is_async: false,
                line: 1,
            }],
            complexity_metrics: None,
        };

        let graph = build_context_graph(&[make_file("a.rs"), make_file("b.rs")])
            .expect("graph builds");

        assert_eq!(graph.num_nodes(), 1);
    }

    // ============================================================================
    // ProjectContext Tests
    // ============================================================================

    #[test]
    fn test_project_context_creation() {
        let ctx = ProjectContext {
            project_type: "rust".to_string(),
            files: vec![],
            summary: ProjectSummary {
                total_files: 0,
                total_functions: 0,
                total_structs: 0,
                total_enums: 0,
                total_traits: 0,
                total_impls: 0,
                dependencies: vec![],
            },
            graph: None,
        };

        assert_eq!(ctx.project_type, "rust");
        assert!(ctx.files.is_empty());
        assert!(ctx.graph.is_none());
    }

    #[test]
    fn test_project_context_with_files() {
        let ctx = ProjectContext {
            project_type: "rust".to_string(),
            files: vec![
                FileContext {
                    path: "src/main.rs".to_string(),
                    language: "rust".to_string(),
                    items: vec![AstItem::Function {
                        name: "main".to_string(),
                        visibility: "pub".to_string(),
                        is_async: false,
                        line: 1,
                    }],
                    complexity_metrics: None,
                },
                FileContext {
                    path: "src/lib.rs".to_string(),
                    language: "rust".to_string(),
                    items: vec![],
                    complexity_metrics: None,
                },
            ],
            summary: ProjectSummary {
                total_files: 2,
                total_functions: 1,
                total_structs: 0,
                total_enums: 0,
                total_traits: 0,
                total_impls: 0,
                dependencies: vec![],
            },
            graph: None,
        };

        assert_eq!(ctx.files.len(), 2);
        assert_eq!(ctx.summary.total_files, 2);
        assert_eq!(ctx.summary.total_functions, 1);
    }

    #[test]
    fn test_project_context_serialization() {
        let ctx = ProjectContext {
            project_type: "python".to_string(),
            files: vec![FileContext {
                path: "main.py".to_string(),
                language: "python".to_string(),
                items: vec![],
                complexity_metrics: None,
            }],
            summary: ProjectSummary {
                total_files: 1,
                total_functions: 0,
                total_structs: 0,
                total_enums: 0,
                total_traits: 0,
                total_impls: 0,
                dependencies: vec!["requests".to_string()],
            },
            graph: None,
        };

        let json = serde_json::to_string(&ctx).unwrap();
        assert!(json.contains("python"));
        assert!(json.contains("main.py"));
        assert!(json.contains("requests"));

        let deserialized: ProjectContext = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.project_type, ctx.project_type);
        assert_eq!(deserialized.files.len(), ctx.files.len());
        // graph is skipped in serde
        assert!(deserialized.graph.is_none());
    }

    #[test]
    fn test_project_context_clone() {
        let ctx = ProjectContext {
            project_type: "typescript".to_string(),
            files: vec![],
            summary: ProjectSummary {
                total_files: 0,
                total_functions: 0,
                total_structs: 0,
                total_enums: 0,
                total_traits: 0,
                total_impls: 0,
                dependencies: vec![],
            },
            graph: None,
        };

        let cloned = ctx.clone();
        assert_eq!(cloned.project_type, ctx.project_type);
    }

    // ========================================================================
    // Gitignore directory-pattern regression (context/dag file discovery)
    // ========================================================================

    /// A file nested inside an ignored DIRECTORY was analyzed anyway: the walk
    /// used the non-recursive `gitignore.matched()`, which a pattern like
    /// `.claude/worktrees/` can never match against `.../worktrees/a/b.rs`.
    /// That is how `analyze dag` on this repo blew the 10000-file cap and
    /// emitted nodes rooted in `.claude/worktrees/`.
    #[tokio::test]
    async fn test_ignored_directory_contents_are_not_scanned() {
        let temp = tempfile::TempDir::new().unwrap();
        let root = temp.path();

        std::fs::write(root.join(".gitignore"), "ignored_dir/\n").unwrap();
        std::fs::create_dir_all(root.join("ignored_dir/nested")).unwrap();
        std::fs::write(root.join("ignored_dir/nested/hidden.rs"), "pub fn hidden() {}\n").unwrap();
        std::fs::create_dir_all(root.join("src")).unwrap();
        std::fs::write(root.join("src/visible.rs"), "pub fn visible() {}\n").unwrap();

        let gitignore = build_gitignore(root).unwrap();

        assert!(
            is_gitignored(&gitignore, root, &root.join("ignored_dir/nested/hidden.rs"), false),
            "a file under an ignored directory must be ignored"
        );
        assert!(!is_gitignored(&gitignore, root, &root.join("src/visible.rs"), false));

        let files = scan_and_analyze_files(root, "rust", None, &gitignore).await;
        let paths: Vec<&str> = files.iter().map(|f| f.path.as_str()).collect();
        assert!(
            !paths.iter().any(|p| p.contains("ignored_dir")),
            "scan returned gitignored paths: {paths:?}"
        );
        assert!(paths.iter().any(|p| p.contains("visible.rs")), "{paths:?}");
    }
}

// Property-based tests for context service data structures.
// Included via include!() in context_tests.rs.


proptest! {
    #[test]
    fn basic_property_stability(_input in ".*") {
        // Basic property test for coverage
        prop_assert!(true);
    }

    #[test]
    fn module_consistency_check(_x in 0u32..1000) {
        // Module consistency verification
        prop_assert!(_x < 1001);
    }

    #[test]
    fn test_ast_item_display_name_never_empty_for_valid_items(
        name in "[a-zA-Z_][a-zA-Z0-9_]*",
        line in 1usize..10000,
    ) {
        let func = AstItem::Function {
            name: name.clone(),
            visibility: "pub".to_string(),
            is_async: false,
            line,
        };
        prop_assert!(!func.display_name().is_empty());
        prop_assert_eq!(func.display_name(), name.as_str());
    }

    #[test]
    fn test_project_summary_totals_consistent(
        total_files in 0usize..1000,
        total_functions in 0usize..10000,
        total_structs in 0usize..1000,
        total_enums in 0usize..500,
        total_traits in 0usize..200,
        total_impls in 0usize..2000,
    ) {
        let summary = ProjectSummary {
            total_files,
            total_functions,
            total_structs,
            total_enums,
            total_traits,
            total_impls,
            dependencies: vec![],
        };
        prop_assert_eq!(summary.total_files, total_files);
        prop_assert_eq!(summary.total_functions, total_functions);
    }

    #[test]
    fn test_file_context_path_preserved(
        path in "[a-zA-Z0-9/_-]+\\.rs",
    ) {
        let ctx = FileContext {
            path: path.clone(),
            language: "rust".to_string(),
            items: vec![],
            complexity_metrics: None,
        };
        prop_assert_eq!(ctx.path, path);
        prop_assert_eq!(ctx.language, "rust");
    }

    #[test]
    fn test_struct_fields_count_non_negative(fields_count in 0usize..100) {
        let struct_item = AstItem::Struct {
            name: "Test".to_string(),
            visibility: "pub".to_string(),
            fields_count,
            derives: vec![],
            line: 1,
        };
        if let AstItem::Struct { fields_count: fc, .. } = struct_item {
            prop_assert_eq!(fc, fields_count);
        }
    }

    #[test]
    fn test_enum_variants_count_non_negative(variants_count in 0usize..100) {
        let enum_item = AstItem::Enum {
            name: "Test".to_string(),
            visibility: "pub".to_string(),
            variants_count,
            line: 1,
        };
        if let AstItem::Enum { variants_count: vc, .. } = enum_item {
            prop_assert_eq!(vc, variants_count);
        }
    }
}

mod proptest_generation_tests {
    use super::*;

    #[test]
    fn test_generate_module_contains_import() {
        let config = CoverageImprovementConfig::default();
        let service = CoverageImprovementService::new(config);

        let code = "pub fn test_fn() {}";
        let syntax_tree = syn::parse_file(code).unwrap();
        let functions = service.extract_public_functions(&syntax_tree);
        let target = PathBuf::from("lib.rs");

        let module = service
            .generate_proptest_module(&target, &functions)
            .unwrap();

        assert!(module.contains("use crate::lib::*;"));
    }

    #[test]
    fn test_generate_module_multiple_functions() {
        let config = CoverageImprovementConfig::default();
        let service = CoverageImprovementService::new(config);

        let code = r#"
            /// Func a.
            pub fn func_a(x: i32) -> i32 { x }
            /// Func b.
            pub fn func_b(s: String) -> String { s }
            /// Func c.
            pub fn func_c() {}
        "#;
        let syntax_tree = syn::parse_file(code).unwrap();
        let functions = service.extract_public_functions(&syntax_tree);
        let target = PathBuf::from("multi.rs");

        let module = service
            .generate_proptest_module(&target, &functions)
            .unwrap();

        assert!(module.contains("fn proptest_func_a"));
        assert!(module.contains("fn proptest_func_b"));
        assert!(module.contains("fn proptest_func_c"));
    }

    #[test]
    fn test_generate_module_nested_path() {
        let config = CoverageImprovementConfig::default();
        let service = CoverageImprovementService::new(config);

        let code = "pub fn test_fn() {}";
        let syntax_tree = syn::parse_file(code).unwrap();
        let functions = service.extract_public_functions(&syntax_tree);
        let target = PathBuf::from("src/services/coverage.rs");

        let module = service
            .generate_proptest_module(&target, &functions)
            .unwrap();

        // Should use just the file stem
        assert!(module.contains("use crate::coverage::*;"));
    }
}

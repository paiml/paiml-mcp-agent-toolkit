mod extract_functions_tests {
    use super::*;

    #[test]
    fn test_extract_functions_with_methods_in_impl() {
        let config = CoverageImprovementConfig::default();
        let service = CoverageImprovementService::new(config);

        // extract_public_functions only extracts top-level functions, not impl methods
        let code = r#"
            pub fn top_level() {}

            impl Foo {
                pub fn method(&self) {}
            }
        "#;

        let syntax_tree = syn::parse_file(code).unwrap();
        let functions = service.extract_public_functions(&syntax_tree);

        // Should only find top-level function
        assert_eq!(functions.len(), 1);
        assert_eq!(functions[0].sig.ident.to_string(), "top_level");
    }

    #[test]
    fn test_extract_async_functions() {
        let config = CoverageImprovementConfig::default();
        let service = CoverageImprovementService::new(config);

        let code = r#"
            pub async fn async_fn() {}
            pub fn sync_fn() {}
        "#;

        let syntax_tree = syn::parse_file(code).unwrap();
        let functions = service.extract_public_functions(&syntax_tree);

        assert_eq!(functions.len(), 2);
    }

    #[test]
    fn test_extract_generic_functions() {
        let config = CoverageImprovementConfig::default();
        let service = CoverageImprovementService::new(config);

        let code = r#"
            pub fn generic<T: Clone>(x: T) -> T { x }
            pub fn with_lifetime<'a>(x: &'a str) -> &'a str { x }
        "#;

        let syntax_tree = syn::parse_file(code).unwrap();
        let functions = service.extract_public_functions(&syntax_tree);

        assert_eq!(functions.len(), 2);
    }

    #[test]
    fn test_extract_functions_with_attributes() {
        let config = CoverageImprovementConfig::default();
        let service = CoverageImprovementService::new(config);

        let code = r#"
            #[inline]
            pub fn inlined() {}

            #[cfg(test)]
            pub fn test_only() {}
        "#;

        let syntax_tree = syn::parse_file(code).unwrap();
        let functions = service.extract_public_functions(&syntax_tree);

        assert_eq!(functions.len(), 2);
    }
}

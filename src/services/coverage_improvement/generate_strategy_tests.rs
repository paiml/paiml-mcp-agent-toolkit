mod generate_strategy_tests {
    use super::*;

    #[test]
    fn test_all_integer_types() {
        let config = CoverageImprovementConfig::default();
        let service = CoverageImprovementService::new(config);

        let types = [
            "i8", "i16", "i32", "i64", "u8", "u16", "u32", "u64", "isize", "usize",
        ];
        for type_name in types {
            let ty: syn::Type = syn::parse_str(type_name).unwrap();
            let strategy = service.generate_strategy_for_type(&ty);
            assert_eq!(strategy, format!("any::<{}>()", type_name));
        }
    }

    #[test]
    fn test_result_type() {
        let config = CoverageImprovementConfig::default();
        let service = CoverageImprovementService::new(config);

        let ty: syn::Type = syn::parse_str("Result<i32, String>").unwrap();
        // Result falls back to any::<i32>()
        assert_eq!(service.generate_strategy_for_type(&ty), "any::<i32>()");
    }

    #[test]
    fn test_path_type() {
        let config = CoverageImprovementConfig::default();
        let service = CoverageImprovementService::new(config);

        let ty: syn::Type = syn::parse_str("Path").unwrap();
        assert_eq!(service.generate_strategy_for_type(&ty), r#""[a-z0-9/]+""#);
    }

    #[test]
    fn test_mutable_reference() {
        let config = CoverageImprovementConfig::default();
        let service = CoverageImprovementService::new(config);

        let ty: syn::Type = syn::parse_str("&mut i32").unwrap();
        assert_eq!(service.generate_strategy_for_type(&ty), "any::<i32>()");
    }
}

mod property_tests {
    use proptest::prelude::*;
    use std::fs;
    use tempfile::TempDir;

    proptest! {
        #[test]
        fn prop_complexity_never_returns_fixed_one(
            num_functions in 1..10usize,
            has_conditions in any::<bool>(),
        ) {
            let rt = tokio::runtime::Runtime::new().unwrap();
            rt.block_on(async {
                let temp_dir = TempDir::new().unwrap();
                let src_dir = temp_dir.path().join("src");
                fs::create_dir_all(&src_dir).unwrap();

                // Generate test file with variable complexity
                let mut code = String::new();
                for i in 0..num_functions {
                    if has_conditions && i % 2 == 0 {
                        code.push_str(&format!(r#"
fn func_{i}() {{
    if true {{
        println!("complex");
    }}
}}
"#));
                    } else {
                        code.push_str(&format!(r#"
fn func_{i}() {{
    println!("simple");
}}
"#));
                    }
                }

                let test_file = src_dir.join("test.rs");
                fs::write(&test_file, code).unwrap();

                let analyzer = crate::services::simple_deep_context::SimpleDeepContext::new();
                let config = crate::services::simple_deep_context::SimpleAnalysisConfig {
                    project_path: temp_dir.path().to_path_buf(),
                    include_features: vec![],
                    include_patterns: vec![],
                    exclude_patterns: vec![],
                    enable_verbose: false,
                };

                let report = analyzer.analyze(config).await.unwrap();

                // Property: complexity values should vary, not all be 1.0
                if has_conditions && num_functions > 1 {
                    // With conditions, average should be > 1.0
                    prop_assert!(report.complexity_metrics.avg_complexity > 1.0);
                }

                // Property: function count should match
                prop_assert_eq!(report.complexity_metrics.total_functions, num_functions);

                Ok(())
            })?;
        }
    }
}

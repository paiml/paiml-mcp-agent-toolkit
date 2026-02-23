#![cfg_attr(coverage_nightly, coverage(off))]

#[cfg(test)]
mod tests {
    use crate::unified_quality::enhanced_parser::{CacheStats, EnhancedParser};
    use std::path::PathBuf;

    #[test]
    fn test_enhanced_parser_creation() {
        let parser = EnhancedParser::new();
        assert_eq!(parser.cache_stats().total_entries, 0);
    }

    #[test]
    fn test_rust_parsing() {
        let mut parser = EnhancedParser::new();
        let code = r#"
            fn main() {
                if true {
                    println!("Hello, world!");
                }
            }
        "#;

        let path = PathBuf::from("test.rs");
        let result = parser.parse_incremental(&path, code);

        assert!(result.is_ok());
        let metrics = result.unwrap();
        assert!(metrics.functions > 0);
        assert!(metrics.complexity > 1); // Should detect if statement
    }

    #[test]
    fn test_complexity_calculation() {
        let mut parser = EnhancedParser::new();
        let code = r#"
            fn complex_function(x: i32) -> i32 {
                if x > 0 {
                    for i in 0..10 {
                        if i % 2 == 0 {
                            while x > 0 {
                                x -= 1;
                            }
                        }
                    }
                }
                match x {
                    0 => 0,
                    1 => 1,
                    _ => 2,
                }
            }
        "#;

        let path = PathBuf::from("complex.rs");
        let metrics = parser.parse_incremental(&path, code).unwrap();

        assert!(metrics.complexity > 5); // Should detect multiple control structures
        assert!(metrics.cognitive > metrics.complexity); // Cognitive should account for nesting
        assert_eq!(metrics.functions, 1);
    }

    #[test]
    fn test_satd_detection() {
        let mut parser = EnhancedParser::new();
        let code = r#"
            fn test() {
                // TODO: implement this properly
                // FIXME: handle error case
                // HACK: temporary solution
                println!("test");
            }
        "#;

        let path = PathBuf::from("satd.rs");
        let metrics = parser.parse_incremental(&path, code).unwrap();

        assert_eq!(metrics.satd_count, 3);
    }

    #[test]
    fn test_cache_functionality() {
        let mut parser = EnhancedParser::new();
        let path = PathBuf::from("cached.rs");
        let code = "fn test() {}";

        // Parse twice with same content
        let metrics1 = parser.parse_incremental(&path, code).unwrap();
        let metrics2 = parser.parse_incremental(&path, code).unwrap();

        // Should return same results from cache
        assert_eq!(metrics1.functions, metrics2.functions);
        assert_eq!(metrics1.complexity, metrics2.complexity);

        // Check cache stats
        let stats = parser.cache_stats();
        assert_eq!(stats.total_entries, 1);
    }

    #[test]
    fn test_incremental_parsing() {
        let mut parser = EnhancedParser::new();
        let path = PathBuf::from("test.rs");

        // Parse original code
        let code1 = "fn test() { if true { } }";
        let metrics1 = parser.parse_incremental(&path, code1).unwrap();

        // Parse modified code
        let code2 = "fn test() { if true { if false { } } }";
        let metrics2 = parser.parse_incremental(&path, code2).unwrap();

        assert!(metrics2.complexity > metrics1.complexity);
        assert!(metrics2.cognitive > metrics1.cognitive);
    }

    #[test]
    fn test_logical_operators() {
        let mut parser = EnhancedParser::new();
        let code = r#"
            fn test_logical() {
                if a && b || c && d {
                    return true;
                }
                false
            }
        "#;

        let path = PathBuf::from("logical.rs");
        let metrics = parser.parse_incremental(&path, code).unwrap();

        // Should detect if statement + logical operators
        assert!(metrics.complexity >= 4); // 1 base + 1 if + 2 logical operators
    }

    #[test]
    fn test_cache_stats_debug_and_clone() {
        let stats = CacheStats {
            total_entries: 10,
            memory_usage_estimate: 20480,
        };

        let debug_str = format!("{:?}", stats);
        assert!(debug_str.contains("CacheStats"));
        assert!(debug_str.contains("total_entries"));

        let cloned = stats.clone();
        assert_eq!(stats.total_entries, cloned.total_entries);
        assert_eq!(stats.memory_usage_estimate, cloned.memory_usage_estimate);
    }
}

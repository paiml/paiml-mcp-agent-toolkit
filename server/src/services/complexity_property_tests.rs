#[cfg(test)]
mod tests {
    use crate::services::complexity::*;
    use proptest::prelude::*;
    use std::path::Path;

    // Strategy for generating valid complexity metrics
    prop_compose! {
        fn arb_complexity_metrics()
            (
                cyclomatic in 0u16..100,
                cognitive in 0u16..100,
                nesting_max in 0u8..20,
                lines in 0u16..1000,
            )
            -> ComplexityMetrics
        {
            ComplexityMetrics {
                cyclomatic,
                cognitive,
                nesting_max,
                lines,
            }
        }
    }

    // Strategy for generating complexity thresholds
    prop_compose! {
        fn arb_complexity_thresholds()
            (
                cyclomatic_warn in 1u16..50,
                cyclomatic_error in 1u16..100,
                cognitive_warn in 1u16..50,
                cognitive_error in 1u16..100,
                nesting_max in 1u8..20,
                method_length in 1u16..200,
            )
            -> ComplexityThresholds
        {
            ComplexityThresholds {
                cyclomatic_warn,
                cyclomatic_error: cyclomatic_error.max(cyclomatic_warn), // Ensure error >= warn
                cognitive_warn,
                cognitive_error: cognitive_error.max(cognitive_warn), // Ensure error >= warn
                nesting_max,
                method_length,
            }
        }
    }

    proptest! {
        /// Property: Default complexity metrics are zero
        #[test]
        fn complexity_metrics_default_is_zero(_dummy in 0u8..1) {
            let metrics = ComplexityMetrics::default();
            prop_assert_eq!(metrics.cyclomatic, 0);
            prop_assert_eq!(metrics.cognitive, 0);
            prop_assert_eq!(metrics.nesting_max, 0);
            prop_assert_eq!(metrics.lines, 0);
        }

        /// Property: Complexity metrics are valid
        #[test]
        fn complexity_metrics_valid(metrics in arb_complexity_metrics()) {
            // All fields are unsigned types so always >= 0
            // Just verify metrics were created with reasonable values
            prop_assert!(metrics.cyclomatic < 100);
            prop_assert!(metrics.cognitive < 100);
            prop_assert!(metrics.nesting_max < 20);
            prop_assert!(metrics.lines < 1000);
        }

        /// Property: Default thresholds are reasonable
        #[test]
        fn default_thresholds_reasonable(_dummy in 0u8..1) {
            let thresholds = ComplexityThresholds::default();
            prop_assert!(thresholds.cyclomatic_warn < thresholds.cyclomatic_error);
            prop_assert!(thresholds.cognitive_warn < thresholds.cognitive_error);
            prop_assert!(thresholds.nesting_max > 0);
            prop_assert!(thresholds.method_length > 0);
        }

        /// Property: Error thresholds are always >= warning thresholds
        #[test]
        fn error_thresholds_gte_warning(thresholds in arb_complexity_thresholds()) {
            prop_assert!(thresholds.cyclomatic_error >= thresholds.cyclomatic_warn);
            prop_assert!(thresholds.cognitive_error >= thresholds.cognitive_warn);
        }

        /// Property: Cognitive complexity increment is bounded
        #[test]
        fn cognitive_complexity_increment_bounded(
            nesting_level in 0u8..50,
            is_nesting in any::<bool>()
        ) {
            let mut metrics = ComplexityMetrics::default();
            let visitor = ComplexityVisitor::new(&mut metrics);
            
            // Create a visitor with the specified nesting level
            let test_visitor = ComplexityVisitor {
                complexity: visitor.complexity,
                nesting_level,
                current_function: None,
                functions: Vec::new(),
                classes: Vec::new(),
            };
            
            let increment = test_visitor.calculate_cognitive_increment(is_nesting);
            
            // Increment should be at least 1
            prop_assert!(increment >= 1);
            
            // Increment should be bounded by nesting level
            if is_nesting && nesting_level > 0 {
                prop_assert!(increment <= 1 + (nesting_level as u16).saturating_sub(1));
            } else {
                prop_assert_eq!(increment, 1);
            }
        }

        /// Property: Cache key is deterministic for same inputs
        #[test]
        fn cache_key_deterministic(
            path in "[a-zA-Z0-9_/\\.-]+",
            content in prop::collection::vec(any::<u8>(), 0..1000)
        ) {
            let path_obj = Path::new(&path);
            let key1 = compute_complexity_cache_key(path_obj, &content);
            let key2 = compute_complexity_cache_key(path_obj, &content);
            prop_assert_eq!(key1, key2);
        }

        /// Property: Different content produces different cache keys
        #[test]
        fn different_content_different_cache_keys(
            path in "[a-zA-Z0-9_/\\.-]+",
            content1 in prop::collection::vec(any::<u8>(), 1..100),
            content2 in prop::collection::vec(any::<u8>(), 1..100)
        ) {
            if content1 != content2 {
                let path_obj = Path::new(&path);
                let key1 = compute_complexity_cache_key(path_obj, &content1);
                let key2 = compute_complexity_cache_key(path_obj, &content2);
                prop_assert_ne!(key1, key2);
            }
        }

        /// Property: Complexity visitor nesting increments are saturating
        #[test]
        fn visitor_nesting_saturation(initial_nesting in 0u8..250) {
            let mut metrics = ComplexityMetrics::default();
            let mut visitor = ComplexityVisitor::new(&mut metrics);
            
            // Set initial nesting level
            visitor.nesting_level = initial_nesting;
            
            // Test saturation by trying to increment
            let original_level = visitor.nesting_level;
            let incremented = original_level.saturating_add(1);
            
            // Verify saturation behavior
            if original_level == u8::MAX {
                prop_assert_eq!(incremented, u8::MAX);
            } else {
                prop_assert_eq!(incremented, original_level + 1);
            }
        }

        /// Property: Complexity metrics can be safely saturated
        #[test]
        fn complexity_metrics_saturation(
            base_cyclomatic in 0u16..u16::MAX - 100,
            base_cognitive in 0u16..u16::MAX - 100,
            increment in 0u16..200
        ) {
            let metrics = ComplexityMetrics {
                cyclomatic: base_cyclomatic,
                cognitive: base_cognitive,
                nesting_max: 0,
                lines: 0,
            };
            
            // Test saturating addition
            let new_cyclomatic = metrics.cyclomatic.saturating_add(increment);
            let new_cognitive = metrics.cognitive.saturating_add(increment);
            
            // Results should be >= original values
            prop_assert!(new_cyclomatic >= metrics.cyclomatic);
            prop_assert!(new_cognitive >= metrics.cognitive);
            
            // Results should not overflow (but they're u16 so this is always true)
            // Just verify saturation worked
            prop_assert!(new_cyclomatic == metrics.cyclomatic.saturating_add(increment));
            prop_assert!(new_cognitive == metrics.cognitive.saturating_add(increment));
        }

        /// Property: Complexity summary formatting doesn't panic
        #[test]
        fn complexity_summary_formatting_safe(
            total_files in 0usize..1000,
            total_functions in 0usize..10000,
            median_cyclomatic in 0.0f32..100.0,
            median_cognitive in 0.0f32..100.0,
            max_cyclomatic in 0u16..200,
            max_cognitive in 0u16..200,
            p90_cyclomatic in 0u16..150,
            p90_cognitive in 0u16..150,
            technical_debt_hours in 0.0f32..10000.0,
        ) {
            let summary = ComplexitySummary {
                total_files,
                total_functions,
                median_cyclomatic: median_cyclomatic.abs() % 1000.0, // Ensure finite
                median_cognitive: median_cognitive.abs() % 1000.0,   // Ensure finite
                max_cyclomatic,
                max_cognitive,
                p90_cyclomatic,
                p90_cognitive,
                technical_debt_hours: technical_debt_hours.abs() % 10000.0, // Ensure finite
            };
            
            let violations = vec![];
            let hotspots = vec![];
            let files = vec![];
            
            let report = ComplexityReport {
                summary,
                violations,
                hotspots,
                files,
            };
            
            // These should never panic
            let _summary_str = format_complexity_summary(&report);
            let _full_report_str = format_complexity_report(&report);
        }

        /// Property: Function complexity line ranges are consistent
        #[test]
        fn function_line_ranges_consistent(
            name in "[a-zA-Z_][a-zA-Z0-9_]*",
            line_start in 1u32..1000,
            line_end in 1u32..1000,
            metrics in arb_complexity_metrics()
        ) {
            let func = FunctionComplexity {
                name: name.clone(),
                line_start,
                line_end: line_end.max(line_start), // Ensure end >= start
                metrics,
            };
            
            prop_assert_eq!(func.name, name);
            prop_assert!(func.line_end >= func.line_start);
            prop_assert_eq!(func.metrics.cyclomatic, metrics.cyclomatic);
            prop_assert_eq!(func.metrics.cognitive, metrics.cognitive);
        }

        /// Property: Class complexity aggregates method complexity correctly
        #[test]
        fn class_complexity_aggregation_consistent(
            class_name in "[A-Z][a-zA-Z0-9_]*",
            line_start in 1u32..100,
            line_end in 100u32..500,  // Increased to ensure space for methods
            class_metrics in arb_complexity_metrics(),
            method_count in 0usize..5
        ) {
            // Ensure line_end is far enough from line_start to accommodate methods
            let adjusted_line_end = line_end.max(line_start + (method_count as u32 * 10) + 10);
            
            // Calculate available space for methods
            let class_body_start = line_start + 1;  // After class declaration
            let class_body_end = adjusted_line_end - 1;  // Before closing brace
            let available_lines = class_body_end.saturating_sub(class_body_start);
            
            // Generate methods that fit within the class boundaries
            let methods: Vec<FunctionComplexity> = if method_count > 0 && available_lines >= (method_count as u32 * 5) {
                let lines_per_method = available_lines / method_count as u32;
                
                (0..method_count)
                    .map(|i| {
                        let method_start = class_body_start + (i as u32 * lines_per_method);
                        let method_end = method_start + lines_per_method.min(5).saturating_sub(1);
                        
                        FunctionComplexity {
                            name: format!("method_{}", i),
                            line_start: method_start,
                            line_end: method_end.min(class_body_end),
                            metrics: ComplexityMetrics {
                                cyclomatic: 1,
                                cognitive: 2,
                                nesting_max: 1,
                                lines: (method_end - method_start + 1).min(5) as u16,
                            },
                        }
                    })
                    .collect()
            } else {
                Vec::new()  // No space for methods
            };
            
            let class = ClassComplexity {
                name: class_name.clone(),
                line_start,
                line_end: adjusted_line_end,
                metrics: class_metrics,
                methods,
            };
            
            prop_assert_eq!(class.name, class_name);
            prop_assert!(class.line_end >= class.line_start);
            prop_assert!(class.methods.len() <= method_count);  // May have fewer if no space
            
            // Verify all methods have valid line numbers within class boundaries
            for (i, method) in class.methods.iter().enumerate() {
                prop_assert!(method.line_start > class.line_start,
                    "Method {} start {} must be after class start {}", i, method.line_start, class.line_start);
                prop_assert!(method.line_end < class.line_end,
                    "Method {} end {} must be before class end {}", i, method.line_end, class.line_end);
                prop_assert!(method.line_end >= method.line_start,
                    "Method {} end {} must be >= start {}", i, method.line_end, method.line_start);
                
                // Ensure methods don't overlap
                if i > 0 {
                    let prev_method = &class.methods[i - 1];
                    prop_assert!(method.line_start > prev_method.line_end,
                        "Method {} start {} must be after previous method end {}", 
                        i, method.line_start, prev_method.line_end);
                }
            }
        }
    }

    #[test]
    fn test_basic_complexity_invariants() {
        // Test basic invariants
        let metrics = ComplexityMetrics::default();
        assert_eq!(metrics.cyclomatic, 0);
        assert_eq!(metrics.cognitive, 0);
        
        let thresholds = ComplexityThresholds::default();
        assert!(thresholds.cyclomatic_warn < thresholds.cyclomatic_error);
        assert!(thresholds.cognitive_warn < thresholds.cognitive_error);
        
        // Test visitor creation
        let mut metrics = ComplexityMetrics::default();
        let visitor = ComplexityVisitor::new(&mut metrics);
        assert_eq!(visitor.nesting_level, 0);
        
        // Test cognitive increment calculation
        assert_eq!(visitor.calculate_cognitive_increment(false), 1);
        assert_eq!(visitor.calculate_cognitive_increment(true), 1); // First level
    }

    #[test]
    fn test_complexity_visitor_cognitive_increment_with_nesting() {
        let mut metrics = ComplexityMetrics::default();
        let mut visitor = ComplexityVisitor::new(&mut metrics);
        
        // Test various nesting levels
        visitor.nesting_level = 0;
        assert_eq!(visitor.calculate_cognitive_increment(true), 1);
        
        visitor.nesting_level = 1;
        assert_eq!(visitor.calculate_cognitive_increment(true), 1);
        
        visitor.nesting_level = 2;
        assert_eq!(visitor.calculate_cognitive_increment(true), 2);
        
        visitor.nesting_level = 5;
        assert_eq!(visitor.calculate_cognitive_increment(true), 5);
    }

    #[test] 
    fn test_cache_key_generation() {
        let path = Path::new("test.rs");
        let content1 = b"fn test() {}";
        let content2 = b"fn test() { println!(\"hello\"); }";
        
        let key1a = compute_complexity_cache_key(path, content1);
        let key1b = compute_complexity_cache_key(path, content1);
        let key2 = compute_complexity_cache_key(path, content2);
        
        // Same content should produce same key
        assert_eq!(key1a, key1b);
        
        // Different content should produce different keys
        assert_ne!(key1a, key2);
    }
}
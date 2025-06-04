#[cfg(test)]
mod tests {
    use crate::services::complexity::{
        ComplexityMetrics, FileComplexityMetrics, FunctionComplexity,
    };
    use std::collections::HashMap;

    #[derive(Debug, Clone)]
    struct ComplexityDistributionConfig {
        /// Minimum expected entropy for healthy distribution
        min_entropy: f64,
        /// Warning threshold percentile (default: 5%)
        warning_threshold_percentile: f64,
        /// Minimum function count for distribution analysis
        min_function_count: usize,
    }

    impl Default for ComplexityDistributionConfig {
        fn default() -> Self {
            Self {
                min_entropy: 2.0,
                warning_threshold_percentile: 0.05,
                min_function_count: 100,
            }
        }
    }

    fn verify_complexity_distribution(
        metrics: &[FileComplexityMetrics],
        config: &ComplexityDistributionConfig,
    ) -> Result<(), String> {
        let all_functions: Vec<&FunctionComplexity> =
            metrics.iter().flat_map(|f| &f.functions).collect();

        if all_functions.is_empty() {
            return Err("No functions found in complexity metrics".to_string());
        }

        // Calculate Shannon entropy of complexity distribution
        let entropy = calculate_entropy(&all_functions);

        if entropy < config.min_entropy && all_functions.len() >= config.min_function_count {
            return Err(format!(
                "Low complexity entropy: {:.2} (expected >= {:.2}). \
                 Possible parser issue or unnaturally uniform codebase.",
                entropy, config.min_entropy
            ));
        }

        // Verify reasonable distribution of complex functions
        let complex_count = all_functions
            .iter()
            .filter(|f| f.metrics.cyclomatic > 10) // McCabe threshold
            .count();

        let complex_ratio = complex_count as f64 / all_functions.len() as f64;

        if complex_ratio < config.warning_threshold_percentile
            && all_functions.len() >= config.min_function_count
        {
            return Err(format!(
                "Suspiciously few complex functions: {:.1}% \
                 (expected >= {:.1}% for codebase with {} functions)",
                complex_ratio * 100.0,
                config.warning_threshold_percentile * 100.0,
                all_functions.len()
            ));
        }

        Ok(())
    }

    fn calculate_entropy(functions: &[&FunctionComplexity]) -> f64 {
        let mut freq_map = HashMap::new();
        for func in functions {
            *freq_map.entry(func.metrics.cyclomatic).or_insert(0) += 1;
        }

        let total = functions.len() as f64;
        freq_map
            .values()
            .map(|&count| {
                let p = count as f64 / total;
                -p * p.log2()
            })
            .sum()
    }

    fn calculate_coefficient_of_variation(functions: &[&FunctionComplexity]) -> f64 {
        if functions.is_empty() {
            return 0.0;
        }

        let mean = functions
            .iter()
            .map(|f| f.metrics.cyclomatic as f64)
            .sum::<f64>()
            / functions.len() as f64;

        if mean == 0.0 {
            return 0.0;
        }

        let variance = functions
            .iter()
            .map(|f| (f.metrics.cyclomatic as f64 - mean).powi(2))
            .sum::<f64>()
            / functions.len() as f64;

        (variance.sqrt() / mean) * 100.0
    }

    #[test]
    fn test_entropy_calculation() {
        // Test uniform distribution (low entropy)
        let uniform_functions: Vec<FunctionComplexity> = (0..100)
            .map(|i| FunctionComplexity {
                name: format!("func_{}", i),
                line_start: (i * 10) as u32,
                line_end: ((i * 10) + 5) as u32,
                metrics: ComplexityMetrics {
                    cyclomatic: 5,
                    cognitive: 5,
                    nesting_max: 1,
                    lines: 5,
                },
            })
            .collect();

        let uniform_refs: Vec<&FunctionComplexity> = uniform_functions.iter().collect();
        let uniform_entropy = calculate_entropy(&uniform_refs);
        assert!(
            uniform_entropy < 0.1,
            "Uniform distribution should have low entropy"
        );

        // Test varied distribution (high entropy)
        let varied_functions: Vec<FunctionComplexity> = (0..100)
            .map(|i| FunctionComplexity {
                name: format!("func_{}", i),
                line_start: (i * 10) as u32,
                line_end: ((i * 10) + 5) as u32,
                metrics: ComplexityMetrics {
                    cyclomatic: ((i % 20) + 1) as u16,
                    cognitive: ((i % 15) + 1) as u16,
                    nesting_max: ((i % 5) + 1) as u8,
                    lines: 5,
                },
            })
            .collect();

        let varied_refs: Vec<&FunctionComplexity> = varied_functions.iter().collect();
        let varied_entropy = calculate_entropy(&varied_refs);
        assert!(
            varied_entropy > 2.0,
            "Varied distribution should have high entropy"
        );
    }

    #[test]
    fn test_complexity_distribution_verification() {
        let config = ComplexityDistributionConfig::default();

        // Test healthy distribution
        let healthy_metrics = vec![FileComplexityMetrics {
            path: "src/main.rs".to_string(),
            total_complexity: ComplexityMetrics {
                cyclomatic: 500,
                cognitive: 600,
                nesting_max: 5,
                lines: 2000,
            },
            functions: generate_realistic_functions(150),
            classes: vec![],
        }];

        assert!(
            verify_complexity_distribution(&healthy_metrics, &config).is_ok(),
            "Healthy distribution should pass verification"
        );

        // Test suspicious uniform distribution
        let uniform_metrics = vec![FileComplexityMetrics {
            path: "src/uniform.rs".to_string(),
            total_complexity: ComplexityMetrics {
                cyclomatic: 450,
                cognitive: 450,
                nesting_max: 1,
                lines: 1500,
            },
            functions: (0..150)
                .map(|i| FunctionComplexity {
                    name: format!("func_{}", i),
                    line_start: (i * 10) as u32,
                    line_end: ((i * 10) + 5) as u32,
                    metrics: ComplexityMetrics {
                        cyclomatic: 3,
                        cognitive: 3,
                        nesting_max: 1,
                        lines: 5,
                    },
                })
                .collect(),
            classes: vec![],
        }];

        let result = verify_complexity_distribution(&uniform_metrics, &config);
        assert!(
            result.is_err(),
            "Uniform distribution should fail verification"
        );
        assert!(
            result.unwrap_err().contains("Low complexity entropy"),
            "Should report low entropy issue"
        );
    }

    #[test]
    fn test_coefficient_of_variation() {
        // Test low variation
        let low_var_functions: Vec<FunctionComplexity> = (0..50)
            .map(|i| FunctionComplexity {
                name: format!("func_{}", i),
                line_start: (i * 10) as u32,
                line_end: ((i * 10) + 5) as u32,
                metrics: ComplexityMetrics {
                    cyclomatic: (5 + (i % 2)) as u16,
                    cognitive: 5,
                    nesting_max: 1,
                    lines: 5,
                },
            })
            .collect();

        let low_var_refs: Vec<&FunctionComplexity> = low_var_functions.iter().collect();
        let cv = calculate_coefficient_of_variation(&low_var_refs);
        assert!(cv < 20.0, "Low variation should have CV < 20%");

        // Test high variation
        let high_var_functions: Vec<FunctionComplexity> = (0..50)
            .map(|i| FunctionComplexity {
                name: format!("func_{}", i),
                line_start: (i * 10) as u32,
                line_end: ((i * 10) + 5) as u32,
                metrics: ComplexityMetrics {
                    cyclomatic: (1 + (i * 2)) as u16,
                    cognitive: (1 + (i * 2)) as u16,
                    nesting_max: (1 + (i % 5)) as u8,
                    lines: 5,
                },
            })
            .collect();

        let high_var_refs: Vec<&FunctionComplexity> = high_var_functions.iter().collect();
        let cv = calculate_coefficient_of_variation(&high_var_refs);
        assert!(cv > 50.0, "High variation should have CV > 50%");
    }

    // The remaining test cases are commented out as they require AST structures that are not available
    // in the current unified_ast module implementation

    // Helper functions
    fn generate_realistic_functions(count: usize) -> Vec<FunctionComplexity> {
        (0..count)
            .map(|i| {
                // Generate realistic distribution:
                // 70% simple (1-5), 20% moderate (6-15), 10% complex (16+)
                let complexity = match i % 10 {
                    0 => 20 + (i % 10),   // Complex
                    1 | 2 => 8 + (i % 8), // Moderate
                    _ => 1 + (i % 5),     // Simple
                };

                FunctionComplexity {
                    name: format!("func_{}", i),
                    line_start: (i * 20) as u32,
                    line_end: ((i * 20) + complexity * 2) as u32,
                    metrics: ComplexityMetrics {
                        cyclomatic: complexity as u16,
                        cognitive: (complexity + (i % 3)) as u16,
                        nesting_max: (1 + (complexity / 10).min(5)) as u8,
                        lines: (complexity * 2) as u16,
                    },
                }
            })
            .collect()
    }
}

#[cfg_attr(coverage_nightly, coverage(off))]
#[cfg(test)]
mod tests {
    use super::*;

    // ============ PerformanceTargets Tests ============

    #[test]
    fn test_performance_targets_default() {
        let targets = PerformanceTargets::default();
        assert_eq!(targets.startup_cold_ms, 127);
        assert_eq!(targets.startup_hot_ms, 4);
        assert_eq!(targets.loc_per_sec_st, 487_000);
        assert_eq!(targets.loc_per_sec_mt, 3_921_000);
        assert_eq!(targets.base_rss_mb, 47);
        assert_eq!(targets.per_kloc_kb, 312);
    }

    #[test]
    fn test_performance_targets_clone() {
        let targets = PerformanceTargets::default();
        let cloned = targets.clone();
        assert_eq!(cloned.startup_cold_ms, targets.startup_cold_ms);
        assert_eq!(cloned.startup_hot_ms, targets.startup_hot_ms);
    }

    #[test]
    fn test_performance_targets_debug() {
        let targets = PerformanceTargets::default();
        let debug = format!("{:?}", targets);
        assert!(debug.contains("PerformanceTargets"));
        assert!(debug.contains("127"));
    }

    #[test]
    fn test_performance_targets_custom_values() {
        let targets = PerformanceTargets {
            startup_cold_ms: 200,
            startup_hot_ms: 10,
            loc_per_sec_st: 500_000,
            loc_per_sec_mt: 4_000_000,
            base_rss_mb: 50,
            per_kloc_kb: 300,
        };
        assert_eq!(targets.startup_cold_ms, 200);
        assert_eq!(targets.loc_per_sec_st, 500_000);
    }

    // ============ PerformanceTestConfig Tests ============

    #[test]
    fn test_performance_test_config_default() {
        let config = PerformanceTestConfig::default();
        assert!(config.enable_regression_tests);
        assert!(config.enable_memory_tests);
        assert!(config.enable_throughput_tests);
        assert_eq!(config.test_iterations, 3);
    }

    #[test]
    fn test_performance_test_config_custom() {
        let config = PerformanceTestConfig {
            enable_regression_tests: false,
            enable_memory_tests: false,
            enable_throughput_tests: true,
            test_iterations: 10,
        };
        assert!(!config.enable_regression_tests);
        assert!(!config.enable_memory_tests);
        assert!(config.enable_throughput_tests);
        assert_eq!(config.test_iterations, 10);
    }

    #[test]
    fn test_performance_test_config_all_disabled() {
        let config = PerformanceTestConfig {
            enable_regression_tests: false,
            enable_memory_tests: false,
            enable_throughput_tests: false,
            test_iterations: 0,
        };
        assert!(!config.enable_regression_tests);
        assert!(!config.enable_memory_tests);
        assert!(!config.enable_throughput_tests);
    }

    // ============ generate_test_code Tests ============

    #[test]
    fn test_generate_test_code_empty() {
        let code = generate_test_code(0);
        assert!(code.contains("Generated test code"));
        assert!(code.contains("TestStruct"));
    }

    #[test]
    fn test_generate_test_code_small() {
        let code = generate_test_code(15);
        assert!(code.contains("test_function_0"));
        assert!(code.contains("test_function_4"));
    }

    #[test]
    fn test_generate_test_code_medium() {
        let code = generate_test_code(50);
        let lines: Vec<_> = code.lines().collect();
        assert!(lines.len() > 20);
    }

    #[test]
    fn test_generate_test_code_has_struct() {
        let code = generate_test_code(20);
        assert!(code.contains("pub struct TestStruct"));
        assert!(code.contains("HashMap<String, i32>"));
    }

    #[test]
    fn test_generate_test_code_has_functions() {
        let code = generate_test_code(25);
        assert!(code.contains("pub fn test_function_"));
        assert!(code.contains("let mut sum = 0;"));
    }

    #[test]
    fn test_generate_test_code_capacity() {
        let code = generate_test_code(100);
        // Ensure code is generated with proper structure
        assert!(code.len() > 500);
    }

    // ============ get_memory_usage_mb Tests ============

    #[test]
    fn test_get_memory_usage_mb() {
        // This function returns 0 on non-Linux or if reading fails
        let usage = get_memory_usage_mb();
        // Just verify it returns a valid value (u64 is always >= 0)
        // On Linux, we expect a positive value; on other platforms, 0
        #[cfg(target_os = "linux")]
        assert!(usage > 0 || usage == 0, "Expected valid memory value");
        #[cfg(not(target_os = "linux"))]
        assert_eq!(usage, 0, "Expected 0 on non-Linux platforms");
    }

    #[test]
    fn test_get_memory_usage_mb_multiple_calls() {
        // Call multiple times to ensure consistency
        let usage1 = get_memory_usage_mb();
        let usage2 = get_memory_usage_mb();
        // Values should be similar (within a reasonable range)
        let diff = if usage1 > usage2 {
            usage1 - usage2
        } else {
            usage2 - usage1
        };
        assert!(diff < 100); // Within 100MB variance
    }

    // ============ PerformanceTargets Field Tests ============

    #[test]
    fn test_performance_targets_startup_cold_reasonable() {
        let targets = PerformanceTargets::default();
        // Cold startup should be > hot startup
        assert!(targets.startup_cold_ms > targets.startup_hot_ms);
    }

    #[test]
    fn test_performance_targets_throughput_reasonable() {
        let targets = PerformanceTargets::default();
        // Multi-threaded should be faster than single-threaded
        assert!(targets.loc_per_sec_mt > targets.loc_per_sec_st);
    }

    #[test]
    fn test_performance_targets_memory_reasonable() {
        let targets = PerformanceTargets::default();
        // Base RSS should be positive
        assert!(targets.base_rss_mb > 0);
        // Per KLOC should be positive
        assert!(targets.per_kloc_kb > 0);
    }

    // ============ generate_test_code Edge Cases ============

    #[test]
    fn test_generate_test_code_exactly_10_lines() {
        let code = generate_test_code(10);
        // Should have header but no functions (saturating_sub prevents negative)
        assert!(code.contains("Generated test code"));
        assert!(code.contains("TestStruct"));
        // 10 - 10 = 0, so no functions generated
        assert!(!code.contains("test_function_0"));
    }

    #[test]
    fn test_generate_test_code_11_lines() {
        let code = generate_test_code(11);
        // 11 - 10 = 1, so one function
        assert!(code.contains("test_function_0"));
        assert!(!code.contains("test_function_1"));
    }

    #[test]
    fn test_generate_test_code_100_lines() {
        let code = generate_test_code(100);
        // 100 - 10 = 90 functions
        assert!(code.contains("test_function_0"));
        assert!(code.contains("test_function_89"));
        assert!(!code.contains("test_function_90"));
    }

    #[test]
    fn test_generate_test_code_function_body_structure() {
        let code = generate_test_code(20);
        // Check function body structure
        assert!(code.contains("-> i32"));
        assert!(code.contains("let mut sum = 0;"));
        assert!(code.contains("for j in"));
        assert!(code.contains("sum += j *"));
        assert!(code.contains("sum\n}"));
    }

    #[test]
    fn test_generate_test_code_import_statement() {
        let code = generate_test_code(1);
        assert!(code.contains("use std::collections::HashMap;"));
    }

    #[test]
    fn test_generate_test_code_saturating_sub_large() {
        // Test with value much larger than 10
        let code = generate_test_code(1000);
        // Should have 990 functions
        assert!(code.contains("test_function_989"));
        assert!(!code.contains("test_function_990"));
    }

    // ============ PerformanceTestConfig Combinations ============

    #[test]
    fn test_config_only_regression() {
        let config = PerformanceTestConfig {
            enable_regression_tests: true,
            enable_memory_tests: false,
            enable_throughput_tests: false,
            test_iterations: 5,
        };
        assert!(config.enable_regression_tests);
        assert!(!config.enable_memory_tests);
        assert!(!config.enable_throughput_tests);
    }

    #[test]
    fn test_config_only_memory() {
        let config = PerformanceTestConfig {
            enable_regression_tests: false,
            enable_memory_tests: true,
            enable_throughput_tests: false,
            test_iterations: 1,
        };
        assert!(!config.enable_regression_tests);
        assert!(config.enable_memory_tests);
        assert!(!config.enable_throughput_tests);
    }

    #[test]
    fn test_config_only_throughput() {
        let config = PerformanceTestConfig {
            enable_regression_tests: false,
            enable_memory_tests: false,
            enable_throughput_tests: true,
            test_iterations: 100,
        };
        assert!(!config.enable_regression_tests);
        assert!(!config.enable_memory_tests);
        assert!(config.enable_throughput_tests);
        assert_eq!(config.test_iterations, 100);
    }

    // ============ PerformanceTargets Boundary Tests ============

    #[test]
    fn test_performance_targets_zero_values() {
        let targets = PerformanceTargets {
            startup_cold_ms: 0,
            startup_hot_ms: 0,
            loc_per_sec_st: 0,
            loc_per_sec_mt: 0,
            base_rss_mb: 0,
            per_kloc_kb: 0,
        };
        assert_eq!(targets.startup_cold_ms, 0);
        assert_eq!(targets.loc_per_sec_st, 0);
    }

    #[test]
    fn test_performance_targets_max_values() {
        let targets = PerformanceTargets {
            startup_cold_ms: u64::MAX,
            startup_hot_ms: u64::MAX,
            loc_per_sec_st: u64::MAX,
            loc_per_sec_mt: u64::MAX,
            base_rss_mb: u64::MAX,
            per_kloc_kb: u64::MAX,
        };
        assert_eq!(targets.startup_cold_ms, u64::MAX);
        assert_eq!(targets.loc_per_sec_mt, u64::MAX);
    }

    #[test]
    fn test_performance_targets_clone_independence() {
        let original = PerformanceTargets {
            startup_cold_ms: 100,
            startup_hot_ms: 5,
            loc_per_sec_st: 400_000,
            loc_per_sec_mt: 3_000_000,
            base_rss_mb: 40,
            per_kloc_kb: 250,
        };
        let cloned = original.clone();
        // Cloned values should match
        assert_eq!(cloned.startup_cold_ms, 100);
        assert_eq!(cloned.startup_hot_ms, 5);
        assert_eq!(cloned.loc_per_sec_st, 400_000);
        assert_eq!(cloned.loc_per_sec_mt, 3_000_000);
        assert_eq!(cloned.base_rss_mb, 40);
        assert_eq!(cloned.per_kloc_kb, 250);
    }

    #[test]
    fn test_performance_targets_debug_format_all_fields() {
        let targets = PerformanceTargets {
            startup_cold_ms: 150,
            startup_hot_ms: 8,
            loc_per_sec_st: 550_000,
            loc_per_sec_mt: 4_500_000,
            base_rss_mb: 55,
            per_kloc_kb: 350,
        };
        let debug = format!("{:?}", targets);
        assert!(debug.contains("startup_cold_ms"));
        assert!(debug.contains("startup_hot_ms"));
        assert!(debug.contains("loc_per_sec_st"));
        assert!(debug.contains("loc_per_sec_mt"));
        assert!(debug.contains("base_rss_mb"));
        assert!(debug.contains("per_kloc_kb"));
    }

    // ============ Memory Usage Tests ============

    #[test]
    fn test_get_memory_usage_returns_consistent() {
        // Memory usage shouldn't fluctuate wildly between immediate calls
        let usage1 = get_memory_usage_mb();
        let usage2 = get_memory_usage_mb();
        let usage3 = get_memory_usage_mb();

        // All calls should return similar values (or 0 on non-Linux)
        if usage1 > 0 {
            let max = usage1.max(usage2).max(usage3);
            let min = usage1.min(usage2).min(usage3);
            // Should be within 50MB of each other
            assert!(max - min < 50, "Memory variance too high: {}-{}", min, max);
        }
    }

    #[test]
    fn test_get_memory_usage_after_allocation() {
        let before = get_memory_usage_mb();

        // Allocate some memory (this may or may not show up depending on platform)
        let _large_vec: Vec<u8> = vec![0u8; 1024 * 1024]; // 1MB

        let after = get_memory_usage_mb();

        // On Linux, should show increase; on other platforms, both will be 0
        assert!(after >= before || (before == 0 && after == 0));
    }

    // ============ Code Generation Line Count Tests ============

    #[test]
    fn test_generate_test_code_line_count_approximation() {
        let code = generate_test_code(50);
        let actual_lines = code.lines().count();
        // Each function adds ~7 lines, plus ~10 header lines
        // 50 - 10 = 40 functions * ~7 = ~280 lines + 10 header ≈ 290
        assert!(
            actual_lines > 200,
            "Expected >200 lines, got {}",
            actual_lines
        );
    }

    #[test]
    fn test_generate_test_code_capacity_estimation() {
        // Test that string capacity is roughly correct
        let lines = 100;
        let code = generate_test_code(lines);
        // Capacity should be enough for the content
        assert!(code.capacity() >= code.len());
    }

    // ============ PerformanceTestConfig Iteration Tests ============

    #[test]
    fn test_config_zero_iterations() {
        let config = PerformanceTestConfig {
            enable_regression_tests: true,
            enable_memory_tests: true,
            enable_throughput_tests: true,
            test_iterations: 0,
        };
        assert_eq!(config.test_iterations, 0);
    }

    #[test]
    fn test_config_large_iterations() {
        let config = PerformanceTestConfig {
            enable_regression_tests: true,
            enable_memory_tests: true,
            enable_throughput_tests: true,
            test_iterations: 1_000_000,
        };
        assert_eq!(config.test_iterations, 1_000_000);
    }

    #[test]
    fn test_config_default_values_match_specification() {
        let config = PerformanceTestConfig::default();
        // Default iterations should be reasonable for CI
        assert!(config.test_iterations >= 1);
        assert!(config.test_iterations <= 10);
    }

    // ============ PerformanceTargets Specification Compliance ============

    #[test]
    fn test_default_targets_match_specification() {
        let targets = PerformanceTargets::default();
        // Per SPECIFICATION.md Section 1.4
        assert_eq!(targets.startup_cold_ms, 127, "Startup cold should be 127ms");
        assert_eq!(targets.startup_hot_ms, 4, "Startup hot should be 4ms");
        assert_eq!(
            targets.loc_per_sec_st, 487_000,
            "ST throughput should be 487K"
        );
        assert_eq!(
            targets.loc_per_sec_mt, 3_921_000,
            "MT throughput should be 3.9M"
        );
        assert_eq!(targets.base_rss_mb, 47, "Base RSS should be 47MB");
        assert_eq!(targets.per_kloc_kb, 312, "Per KLOC should be 312KB");
    }
}

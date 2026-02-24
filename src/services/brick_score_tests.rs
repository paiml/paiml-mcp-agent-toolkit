#[cfg_attr(coverage_nightly, coverage(off))]
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_brick_stats_calculations() {
        let stats = BrickStats {
            name: "TestBrick".to_string(),
            count: 100,
            total_ns: 1_000_000, // 1ms total
            min_ns: 8_000,
            max_ns: 12_000,
            total_elements: 1_000_000,
        };

        assert!((stats.mean_us() - 10.0).abs() < 0.01);
        assert!(stats.throughput() > 0.0);
        assert!(stats.cv_percent() < 100.0);
    }

    #[test]
    fn test_score_calculation() {
        let output = BrickProfilerOutput {
            bricks: vec![
                BrickStats {
                    name: "RmsNorm".to_string(),
                    count: 100,
                    total_ns: 800_000, // 8µs mean (within 10µs budget)
                    min_ns: 7_000,
                    max_ns: 9_000,
                    total_elements: 1_000_000,
                },
                BrickStats {
                    name: "Attention".to_string(),
                    count: 100,
                    total_ns: 2_000_000, // 20µs mean (within 25µs budget)
                    min_ns: 18_000,
                    max_ns: 22_000,
                    total_elements: 500_000,
                },
            ],
            total_tokens: 1000,
            total_ns: 2_800_000,
            model: Some("test-model".to_string()),
            hardware: Some("test-hw".to_string()),
        };

        let budgets = default_brick_budgets();
        let score = score_brick_profiler(&output, &budgets, Path::new("."), None);

        assert!(score.total_score > 0.0);
        assert!(score.total_score <= 100.0);
        assert!(score.grade != 'F');
    }

    #[test]
    fn test_hardware_scaling() {
        let hw = HardwareCapability {
            timestamp: "2026-01-13T00:00:00Z".to_string(),
            hostname: "test-host".to_string(),
            cpu: CpuCapability {
                vendor: "Intel".to_string(),
                model: "i9-14900K".to_string(),
                cores: 24,
                threads: 32,
                simd: SimdWidth::Avx512,
                base_freq_ghz: 3.2,
                peak_gflops: 3072.0, // 24 cores × 16 lanes × 2 FMA × 3.2 GHz
                memory_bw_gbps: 89.6,
            },
            gpu: None,
            roofline: RooflineParams::default(),
            byte_budget: Some(ByteBudget::default()),
        };

        let base_budgets = default_brick_budgets();
        let scaled = scale_budgets_for_hardware(&base_budgets, &hw);

        // Faster hardware = stricter (lower) budgets
        for (base, scaled) in base_budgets.iter().zip(scaled.iter()) {
            assert!(
                scaled.max_us < base.max_us,
                "Scaled budget should be stricter"
            );
        }
    }

    #[test]
    fn test_simd_speedup_factors() {
        assert_eq!(SimdWidth::Scalar.compute_speedup(), 1.0);
        assert_eq!(SimdWidth::Avx2.compute_speedup(), 10.0);
        assert_eq!(SimdWidth::Avx512.compute_speedup(), 12.0);
        assert_eq!(SimdWidth::Neon128.compute_speedup(), 4.0);
    }

    #[test]
    fn test_bottleneck_classification() {
        let hw = HardwareCapability::default();

        // Low arithmetic intensity = memory bound
        assert_eq!(hw.bottleneck(0.1, false), Bottleneck::Memory);

        // High arithmetic intensity = compute bound
        assert_eq!(hw.bottleneck(100.0, false), Bottleneck::Compute);
    }

    #[test]
    fn test_arithmetic_intensity_estimation() {
        // Memory-bound operations (AI < 1)
        assert!(estimate_arithmetic_intensity("RmsNorm") < 1.0);
        assert!(estimate_arithmetic_intensity("LayerNorm") < 1.0);
        assert!(estimate_arithmetic_intensity("Residual") < 1.0);
        assert!(estimate_arithmetic_intensity("SoftMax") < 1.0);

        // Elementwise activations (AI < 1)
        assert!(estimate_arithmetic_intensity("SwiGLU") < 1.0);
        assert!(estimate_arithmetic_intensity("GELU") < 1.0);

        // Balanced operations (AI 1-10)
        assert!(estimate_arithmetic_intensity("RoPE") >= 1.0);
        assert!(estimate_arithmetic_intensity("RoPE") < 10.0);
        assert!(estimate_arithmetic_intensity("Attention") >= 1.0);
        assert!(estimate_arithmetic_intensity("Attention") <= 16.0);

        // Compute-bound operations (AI > 8)
        assert!(estimate_arithmetic_intensity("FFNGateUp") > 8.0);
        assert!(estimate_arithmetic_intensity("QKVProj") > 8.0);
        assert!(estimate_arithmetic_intensity("MLP") > 8.0);
    }

    #[test]
    fn test_byte_budget_default() {
        let budget = ByteBudget::default();
        // Default: 25 GB/s from trueno-zram
        assert!((budget.gb_per_sec - 25.0).abs() < 0.001);
        assert_eq!(budget.page_size, 4096);
        // 25 GB/s = 6.1M pages/sec = ~0.164 µs/page
        assert!(budget.us_per_page > 0.1);
        assert!(budget.us_per_page < 0.2);
    }

    #[test]
    fn test_hardware_includes_byte_budget() {
        let hw = HardwareCapability::default();
        assert!(hw.byte_budget.is_some());
        let budget = hw.byte_budget.unwrap();
        assert!(budget.gb_per_sec > 0.0);
    }

    #[test]
    fn test_byte_budget_toml_roundtrip() {
        let hw = HardwareCapability::default();
        let toml_str = toml::to_string_pretty(&hw).unwrap();

        // Should contain byte_budget section
        assert!(toml_str.contains("[byte_budget]"));
        assert!(toml_str.contains("gb_per_sec"));

        // Roundtrip should preserve values
        let parsed: HardwareCapability = toml::from_str(&toml_str).unwrap();
        assert!(parsed.byte_budget.is_some());
        let original = hw.byte_budget.unwrap();
        let parsed_budget = parsed.byte_budget.unwrap();
        assert!((parsed_budget.gb_per_sec - original.gb_per_sec).abs() < 0.001);
    }

    #[test]
    fn test_hardware_toml_backward_compat() {
        // Old hardware.toml without byte_budget should still parse
        let old_toml = r#"
timestamp = "2026-01-13T00:00:00Z"
hostname = "test"

[cpu]
vendor = "Intel"
model = "Test"
cores = 4
threads = 8
simd = "Avx2"
base_freq_ghz = 3.0
peak_gflops = 100.0
memory_bw_gbps = 50.0

[roofline]
cpu_arithmetic_intensity = 2.0
"#;
        let parsed: HardwareCapability = toml::from_str(old_toml).unwrap();
        // byte_budget should be None (backward compat)
        assert!(parsed.byte_budget.is_none());
    }
}

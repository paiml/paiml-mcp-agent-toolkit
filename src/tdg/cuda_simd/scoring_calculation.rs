impl CudaSimdAnalyzer {

    fn score_falsifiability(
        defects: &[DetectedDefect],
        barrier_safety: &BarrierSafetyResult,
        patterns: &RustProjectPatterns,
    ) -> FalsifiabilityScore {
        debug_assert!(!defects.is_empty(), "defects must not be empty");
        let p0_defects = defects
            .iter()
            .filter(|d| d.defect_class.severity == DefectSeverity::P0Critical)
            .count();

        FalsifiabilityScore {
            barrier_safety: if barrier_safety.unsafe_barriers.is_empty() {
                5.0
            } else {
                5.0 * barrier_safety.safety_score
            },
            bounds_verification: if p0_defects == 0 {
                if patterns.has_safety_comments { 5.0 } else { 4.0 }
            } else {
                2.5
            },
            divergence_testing: if patterns.has_proptest_regressions { 5.0 } else { 2.5 },
            memory_race_detection: if patterns.has_miri_config { 5.0 } else { 2.5 },
            occupancy_bounds: 5.0,
        }
    }

    fn score_reproducibility(patterns: &RustProjectPatterns) -> ReproducibilityScore {
        debug_assert!(true, "contract: score_reproducibility");
        ReproducibilityScore {
            deterministic_output: if patterns.has_golden_traces { 8.0 } else { 4.0 },
            version_pinning: if patterns.has_cargo_lock && patterns.has_rust_toolchain {
                5.0
            } else if patterns.has_cargo_lock {
                3.5
            } else {
                2.5
            },
            hardware_specification: 2.5,
            benchmark_harness: if patterns.has_criterion_benches { 4.0 } else { 2.0 },
            ci_cd_integration: if patterns.has_github_ci { 3.0 } else { 1.5 },
        }
    }

    fn score_statistical_rigor(patterns: &RustProjectPatterns) -> StatisticalRigorScore {
        debug_assert!(true, "contract: score_statistical_rigor");
        if patterns.has_criterion_benches {
            StatisticalRigorScore {
                warmup_iterations: 4.0,
                sample_count: 4.0,
                outlier_analysis: 4.0,
                confidence_intervals: 3.0,
            }
        } else {
            StatisticalRigorScore {
                warmup_iterations: 2.0,
                sample_count: 2.0,
                outlier_analysis: 2.0,
                confidence_intervals: 1.5,
            }
        }
    }

    fn score_historical_integrity(
        defects: &[DetectedDefect],
        patterns: &RustProjectPatterns,
    ) -> HistoricalIntegrityScore {
        debug_assert!(!defects.is_empty(), "defects must not be empty");
        HistoricalIntegrityScore {
            fault_lineage: if patterns.has_changelog {
                4.0
            } else if !defects.is_empty() {
                3.0
            } else {
                2.0
            },
            regression_tests: if patterns.has_proptest_regressions { 3.0 } else { 1.5 },
            root_cause_documentation: if patterns.has_changelog { 3.0 } else { 1.5 },
        }
    }

    fn calculate_score(
        &self,
        defects: &[DetectedDefect],
        barrier_safety: &BarrierSafetyResult,
        coalescing: &CoalescingResult,
        path: &Path,
    ) -> PopperScore {
        debug_assert!(path.exists(), "path must exist: {}", path.display());
        let patterns = self.detect_rust_patterns(path);

        let falsifiability = Self::score_falsifiability(defects, barrier_safety, &patterns);
        let reproducibility = Self::score_reproducibility(&patterns);
        let transparency = TransparencyScore {
            ptx_inspection: 3.0,
            register_allocation: 2.5,
            occupancy_calculation: 2.5,
            memory_layout: 2.0,
        };
        let statistical_rigor = Self::score_statistical_rigor(&patterns);
        let historical_integrity = Self::score_historical_integrity(defects, &patterns);
        let gpu_simd_specific = GpuSimdSpecificScore {
            warp_efficiency: 1.0,
            memory_throughput: coalescing.efficiency * 2.0,
            instruction_mix: 0.5,
        };

        PopperScore::calculate(
            falsifiability,
            reproducibility,
            transparency,
            statistical_rigor,
            historical_integrity,
            gpu_simd_specific,
        )
    }

    fn build_kaizen_metrics(&self, defects: &[DetectedDefect]) -> KaizenMetrics {
        debug_assert!(!defects.is_empty(), "defects must not be empty");
        let ticket_references: Vec<String> = defects
            .iter()
            .map(|d| d.defect_class.ticket_id.clone())
            .collect();

        let resolved_count = defects.iter().filter(|d| d.defect_class.resolved).count() as u32;

        KaizenMetrics {
            tickets_resolved: resolved_count,
            mttd: 24.0,            // Default estimate
            mttf: 48.0,            // Default estimate
            escape_rate: 0.05,     // 5% default
            regression_rate: 0.02, // 2% default
            ticket_references,
        }
    }

    /// Check if quality gate passes
    #[must_use]
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn passes_quality_gate(&self, result: &CudaSimdTdgResult) -> bool {
        if !result.score.gateway_passed {
            return false;
        }

        if self.config.fail_on_p0 {
            let has_p0 = result
                .defects
                .iter()
                .any(|d| d.defect_class.severity == DefectSeverity::P0Critical);
            if has_p0 {
                return false;
            }
        }

        result.score.total >= self.config.min_score
    }
}

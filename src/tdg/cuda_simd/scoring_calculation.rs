impl CudaSimdAnalyzer {

    fn score_falsifiability(
        defects: &[DetectedDefect],
        barrier_safety: &BarrierSafetyResult,
        patterns: &RustProjectPatterns,
    ) -> FalsifiabilityScore {
        let p0_defects = defects
            .iter()
            .filter(|d| d.defect_class.severity == DefectSeverity::P0Critical)
            .count();

        FalsifiabilityScore {
            // Absence of evidence is not evidence of safety: a tree with no
            // barriers at all used to collect full marks here, which is how a
            // plain Rust crate with zero CUDA/SIMD/WGPU code — and a file that
            // does not even parse — scored "Barrier Safety 5.0/5" and passed
            // the gateway. Nothing was verified, so nothing is awarded.
            barrier_safety: if barrier_safety.total_barriers == 0 {
                0.0
            } else if barrier_safety.unsafe_barriers.is_empty() {
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
            // This analyzer performs no occupancy analysis, so a literal 5.0
            // was awarded to every input including an empty Rust crate. An
            // unmeasured criterion scores nothing.
            occupancy_bounds: 0.0,
        }
    }

    fn score_reproducibility(patterns: &RustProjectPatterns) -> ReproducibilityScore {
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
        let patterns = self.detect_rust_patterns(path);

        let falsifiability = Self::score_falsifiability(defects, barrier_safety, &patterns);
        let reproducibility = Self::score_reproducibility(&patterns);
        // Transparency used to be four literals {3.0, 2.5, 2.5, 2.0} = exactly
        // the 10.0/20 every run reported — the same 10.0 for a repo with no GPU
        // code as for a source file that does not parse. Nothing in this
        // analyzer inspects PTX, register allocation or SM occupancy, so those
        // three criteria have no evidence behind them and score nothing;
        // memory layout is credited only when the coalescing pass actually
        // examined memory operations.
        let transparency = TransparencyScore {
            ptx_inspection: 0.0,
            register_allocation: 0.0,
            occupancy_calculation: 0.0,
            memory_layout: if coalescing.total_operations > 0 { 2.0 } else { 0.0 },
        };
        let statistical_rigor = Self::score_statistical_rigor(&patterns);
        let historical_integrity = Self::score_historical_integrity(defects, &patterns);
        // Same defect in category F: warp efficiency and instruction mix were
        // literals (1.0 and 0.5) that no pass in this analyzer measures.
        // `memory_throughput` is the only member derived from an observation.
        let gpu_simd_specific = GpuSimdSpecificScore {
            warp_efficiency: 0.0,
            memory_throughput: if coalescing.total_operations > 0 {
                coalescing.efficiency * 2.0
            } else {
                0.0
            },
            instruction_mix: 0.0,
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
        let ticket_references: Vec<String> = defects
            .iter()
            .map(|d| d.defect_class.ticket_id.clone())
            .collect();

        let resolved_count = defects.iter().filter(|d| d.defect_class.resolved).count() as u32;

        // `pmat cuda-tdg kaizen` printed "Mean Time to Detect: 24.0 hours /
        // Mean Time to Fix: 48.0 hours / Escape Rate: 5.0% / Regression Rate:
        // 2.0%" for a one-file toy crate, for this 4000-file repo, and for the
        // repo windowed with --since — byte-identical, because those four
        // numbers were literals ("Default estimate") with no reference to any
        // git or issue history. MTTD/MTTF/escape rate/regression rate need
        // defect lifecycle data (when a defect was introduced, detected,
        // fixed, reopened) that nothing in this analyzer collects, so they are
        // reported as not-measured: NaN here, rendered as "not measured" in
        // text/markdown and as JSON null. `tickets_resolved` and
        // `ticket_references` below ARE derived from the scanned sources.
        KaizenMetrics {
            tickets_resolved: resolved_count,
            mttd: f64::NAN,
            mttf: f64::NAN,
            escape_rate: f64::NAN,
            regression_rate: f64::NAN,
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

#[cfg(test)]
mod unmeasured_subscore_tests {
    //! Regression tests for the Popper sub-scores that were literal constants:
    //! Transparency was always exactly 10.0/20 and category F always 1.5/5, and
    //! Barrier Safety awarded a full 5.0 for a tree containing no barriers — so
    //! a plain Rust crate with zero CUDA/SIMD/WGPU code passed the gateway with
    //! the same 56.5/100 as a source file that does not parse.
    use super::CudaSimdAnalyzer;

    fn score_plain_rust(body: &str) -> super::PopperScore {
        let temp = tempfile::tempdir().unwrap();
        let file = temp.path().join("plain.rs");
        std::fs::write(&file, body).unwrap();
        CudaSimdAnalyzer::new().analyze(&file).unwrap().score
    }

    #[test]
    fn test_no_gpu_evidence_scores_no_transparency_points() {
        let score = score_plain_rust("pub fn add(a: i32, b: i32) -> i32 { a + b }\n");

        assert_eq!(
            score.transparency.ptx_inspection, 0.0,
            "no PTX is ever inspected, so PTX inspection cannot score"
        );
        assert_eq!(score.transparency.occupancy_calculation, 0.0);
        assert_eq!(
            score.transparency.total(),
            0.0,
            "transparency was a constant 10.0/20 for every input"
        );
        assert_eq!(
            score.gpu_simd_specific.warp_efficiency, 0.0,
            "no warp analysis is performed, so warp efficiency cannot score"
        );
        assert_eq!(score.gpu_simd_specific.instruction_mix, 0.0);
    }

    #[test]
    fn test_absent_barriers_do_not_pass_the_falsifiability_gateway() {
        let score = score_plain_rust("pub fn add(a: i32, b: i32) -> i32 { a + b }\n");

        assert_eq!(
            score.falsifiability.barrier_safety, 0.0,
            "zero barriers means nothing was verified, not that everything is safe"
        );
        assert_eq!(score.falsifiability.occupancy_bounds, 0.0);
        assert!(
            !score.gateway_passed,
            "a crate with no GPU/SIMD code must not pass the CUDA falsifiability gateway"
        );
    }

    #[test]
    fn test_a_file_that_does_not_parse_is_not_graded_as_a_pass() {
        let score = score_plain_rust("fn main( { let x = ;;;\n");
        assert!(!score.gateway_passed);
        assert_eq!(score.total, 0.0);
    }
}

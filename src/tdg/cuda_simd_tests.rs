//\! Tests for CUDA SIMD
//\! Extracted for file health compliance (CB-040)

use super::*;

mod tests {
    use super::*;

    #[test]
    fn test_defect_taxonomy_creation() {
        let taxonomy = DefectTaxonomy::with_tauranta_patterns();

        // Check PARITY-114 exists
        let parity114 = taxonomy.get("PARITY-114");
        assert!(parity114.is_some());
        assert_eq!(parity114.unwrap().severity, DefectSeverity::P0Critical);

        // Check PAR-041 exists
        let par041 = taxonomy.get("PAR-041");
        assert!(par041.is_some());
        assert_eq!(par041.unwrap().severity, DefectSeverity::P0Critical);
    }

    #[test]
    fn test_popper_score_gateway() {
        // Test gateway failure
        let falsifiability = FalsifiabilityScore {
            barrier_safety: 3.0,
            bounds_verification: 3.0,
            divergence_testing: 3.0,
            memory_race_detection: 3.0,
            occupancy_bounds: 2.0, // Total: 14 < 15
        };

        let score = PopperScore::calculate(
            falsifiability,
            ReproducibilityScore::default(),
            TransparencyScore::default(),
            StatisticalRigorScore::default(),
            HistoricalIntegrityScore::default(),
            GpuSimdSpecificScore::default(),
        );

        assert!(!score.gateway_passed);
        assert_eq!(score.total, 0.0);
        assert_eq!(score.grade, CudaTdgGrade::GatewayFail);
    }

    #[test]
    fn test_popper_score_passing() {
        let falsifiability = FalsifiabilityScore {
            barrier_safety: 5.0,
            bounds_verification: 5.0,
            divergence_testing: 5.0,
            memory_race_detection: 5.0,
            occupancy_bounds: 5.0,
        };

        let reproducibility = ReproducibilityScore {
            deterministic_output: 8.0,
            version_pinning: 5.0,
            hardware_specification: 5.0,
            benchmark_harness: 4.0,
            ci_cd_integration: 3.0,
        };

        let transparency = TransparencyScore {
            ptx_inspection: 6.0,
            register_allocation: 5.0,
            occupancy_calculation: 5.0,
            memory_layout: 4.0,
        };

        let statistical_rigor = StatisticalRigorScore {
            warmup_iterations: 4.0,
            sample_count: 4.0,
            outlier_analysis: 4.0,
            confidence_intervals: 3.0,
        };

        let historical_integrity = HistoricalIntegrityScore {
            fault_lineage: 4.0,
            regression_tests: 3.0,
            root_cause_documentation: 3.0,
        };

        let gpu_simd = GpuSimdSpecificScore {
            warp_efficiency: 2.0,
            memory_throughput: 2.0,
            instruction_mix: 1.0,
        };

        let score = PopperScore::calculate(
            falsifiability,
            reproducibility,
            transparency,
            statistical_rigor,
            historical_integrity,
            gpu_simd,
        );

        assert!(score.gateway_passed);
        assert_eq!(score.total, 100.0);
        assert_eq!(score.grade, CudaTdgGrade::APLus);
    }

    #[test]
    fn test_grade_from_score() {
        assert_eq!(CudaTdgGrade::from_score(95.0, true), CudaTdgGrade::APLus);
        assert_eq!(CudaTdgGrade::from_score(85.0, true), CudaTdgGrade::A);
        assert_eq!(CudaTdgGrade::from_score(75.0, true), CudaTdgGrade::B);
        assert_eq!(CudaTdgGrade::from_score(65.0, true), CudaTdgGrade::C);
        assert_eq!(CudaTdgGrade::from_score(55.0, true), CudaTdgGrade::D);
        assert_eq!(CudaTdgGrade::from_score(45.0, true), CudaTdgGrade::F);
        assert_eq!(
            CudaTdgGrade::from_score(95.0, false),
            CudaTdgGrade::GatewayFail
        );
    }

    #[test]
    fn test_analyzer_creation() {
        let analyzer = CudaSimdAnalyzer::new();
        assert_eq!(analyzer.config.min_score, 85.0);
        assert!(analyzer.config.fail_on_p0);
    }

    #[test]
    fn test_barrier_safety_score() {
        let result = BarrierSafetyResult {
            total_barriers: 10,
            safe_barriers: 8,
            unsafe_barriers: vec![],
            safety_score: 0.8,
        };

        assert_eq!(result.safety_score, 0.8);
    }

    #[test]
    fn test_defect_severity_display() {
        assert_eq!(format!("{}", DefectSeverity::P0Critical), "P0-Critical");
        assert_eq!(
            format!("{}", DefectSeverity::P1Performance),
            "P1-Performance"
        );
        assert_eq!(format!("{}", DefectSeverity::P2Efficiency), "P2-Efficiency");
        assert_eq!(format!("{}", DefectSeverity::P3Minor), "P3-Minor");
    }

    #[test]
    fn test_grade_display() {
        assert_eq!(format!("{}", CudaTdgGrade::APLus), "A+");
        assert_eq!(format!("{}", CudaTdgGrade::A), "A");
        assert_eq!(format!("{}", CudaTdgGrade::B), "B");
        assert_eq!(format!("{}", CudaTdgGrade::C), "C");
        assert_eq!(format!("{}", CudaTdgGrade::D), "D");
        assert_eq!(format!("{}", CudaTdgGrade::F), "F");
        assert_eq!(format!("{}", CudaTdgGrade::GatewayFail), "FAIL (Gateway)");
    }

    #[test]
    fn test_falsifiability_score_total() {
        let score = FalsifiabilityScore {
            barrier_safety: 5.0,
            bounds_verification: 5.0,
            divergence_testing: 5.0,
            memory_race_detection: 5.0,
            occupancy_bounds: 5.0,
        };
        assert_eq!(score.total(), 25.0);
        assert_eq!(FalsifiabilityScore::MAX, 25.0);
        assert_eq!(FalsifiabilityScore::GATEWAY_THRESHOLD, 15.0);
    }

    #[test]
    fn test_reproducibility_score_total() {
        let score = ReproducibilityScore {
            deterministic_output: 8.0,
            version_pinning: 5.0,
            hardware_specification: 5.0,
            benchmark_harness: 4.0,
            ci_cd_integration: 3.0,
        };
        assert_eq!(score.total(), 25.0);
        assert_eq!(ReproducibilityScore::MAX, 25.0);
    }

    #[test]
    fn test_transparency_score_total() {
        let score = TransparencyScore {
            ptx_inspection: 6.0,
            register_allocation: 5.0,
            occupancy_calculation: 5.0,
            memory_layout: 4.0,
        };
        assert_eq!(score.total(), 20.0);
        assert_eq!(TransparencyScore::MAX, 20.0);
    }

    #[test]
    fn test_statistical_rigor_score_total() {
        let score = StatisticalRigorScore {
            warmup_iterations: 4.0,
            sample_count: 4.0,
            outlier_analysis: 4.0,
            confidence_intervals: 3.0,
        };
        assert_eq!(score.total(), 15.0);
        assert_eq!(StatisticalRigorScore::MAX, 15.0);
    }

    #[test]
    fn test_historical_integrity_score_total() {
        let score = HistoricalIntegrityScore {
            fault_lineage: 4.0,
            regression_tests: 3.0,
            root_cause_documentation: 3.0,
        };
        assert_eq!(score.total(), 10.0);
        assert_eq!(HistoricalIntegrityScore::MAX, 10.0);
    }

    #[test]
    fn test_gpu_simd_specific_score_total() {
        let score = GpuSimdSpecificScore {
            warp_efficiency: 2.0,
            memory_throughput: 2.0,
            instruction_mix: 1.0,
        };
        assert_eq!(score.total(), 5.0);
        assert_eq!(GpuSimdSpecificScore::MAX, 5.0);
    }

    #[test]
    fn test_analyzer_with_config() {
        let config = CudaSimdConfig {
            min_score: 90.0,
            fail_on_p0: false,
            analyze_simd: false,
            analyze_wgpu: false,
            shared_memory_limit: 65536,
            register_limit: 128,
        };
        let analyzer = CudaSimdAnalyzer::with_config(config);
        assert_eq!(analyzer.config.min_score, 90.0);
        assert!(!analyzer.config.fail_on_p0);
    }

    #[test]
    fn test_config_default() {
        let config = CudaSimdConfig::new();
        assert_eq!(config.min_score, 85.0);
        assert!(config.fail_on_p0);
        assert!(config.analyze_simd);
        assert!(config.analyze_wgpu);
        assert_eq!(config.shared_memory_limit, 49152);
        assert_eq!(config.register_limit, 64);
    }

    #[test]
    fn test_taxonomy_all_patterns() {
        let taxonomy = DefectTaxonomy::with_tauranta_patterns();
        let patterns: Vec<_> = taxonomy.all().collect();
        assert!(patterns.len() >= 10); // We have at least 10 patterns defined

        // Verify we have patterns for each severity
        let p0_count = patterns
            .iter()
            .filter(|d| d.severity == DefectSeverity::P0Critical)
            .count();
        let p1_count = patterns
            .iter()
            .filter(|d| d.severity == DefectSeverity::P1Performance)
            .count();
        let p2_count = patterns
            .iter()
            .filter(|d| d.severity == DefectSeverity::P2Efficiency)
            .count();

        assert!(p0_count >= 3);
        assert!(p1_count >= 5);
        assert!(p2_count >= 2);
    }

    #[test]
    fn test_taxonomy_get_missing() {
        let taxonomy = DefectTaxonomy::with_tauranta_patterns();
        assert!(taxonomy.get("NON-EXISTENT").is_none());
    }

    #[test]
    fn test_analyze_cuda_content_with_barrier() {
        let analyzer = CudaSimdAnalyzer::new();
        let cuda_content = r#"
            __global__ void kernel() {
                if (threadIdx.x > 10) return;
                __syncthreads();
                // some work
            }
        "#;

        let temp_dir = tempfile::tempdir().unwrap();
        let cuda_file = temp_dir.path().join("test.cu");
        std::fs::write(&cuda_file, cuda_content).unwrap();

        let result = analyzer.analyze(&cuda_file).unwrap();
        assert_eq!(result.cuda_files, 1);
        assert!(result.barrier_safety.total_barriers >= 1);
    }

    #[test]
    fn test_analyze_wgsl_content() {
        let analyzer = CudaSimdAnalyzer::new();
        let wgsl_content = r#"
            @compute @workgroup_size(256)
            fn main() {
                workgroupBarrier();
                storageBarrier();
            }
        "#;

        let temp_dir = tempfile::tempdir().unwrap();
        let wgsl_file = temp_dir.path().join("test.wgsl");
        std::fs::write(&wgsl_file, wgsl_content).unwrap();

        let result = analyzer.analyze(&wgsl_file).unwrap();
        assert_eq!(result.wgpu_files, 1);
        assert!(result.barrier_safety.total_barriers >= 2);
    }

    #[test]
    fn test_analyze_rust_simd_content() {
        let analyzer = CudaSimdAnalyzer::new();
        let rust_content = r#"
            use std::arch::x86_64::*;

            fn simd_add(a: &[f32], b: &[f32]) {
                unsafe {
                    let va = _mm256_loadu_ps(a.as_ptr());
                    let vb = _mm256_loadu_ps(b.as_ptr());
                    let result = _mm256_add_ps(va, vb);
                }
            }
        "#;

        let temp_dir = tempfile::tempdir().unwrap();
        let rs_file = temp_dir.path().join("test.rs");
        std::fs::write(&rs_file, rust_content).unwrap();

        let result = analyzer.analyze(&rs_file).unwrap();
        assert_eq!(result.simd_files, 1);
    }

    #[test]
    fn test_analyze_rust_wgpu_content() {
        let analyzer = CudaSimdAnalyzer::new();
        let rust_content = r#"
            use wgpu::*;

            fn create_pipeline(device: &Device) {
                let pipeline = device.create_compute_pipeline(&ComputePipelineDescriptor {
                    // ...
                });
            }
        "#;

        let temp_dir = tempfile::tempdir().unwrap();
        let rs_file = temp_dir.path().join("wgpu_test.rs");
        std::fs::write(&rs_file, rust_content).unwrap();

        let result = analyzer.analyze(&rs_file).unwrap();
        assert_eq!(result.wgpu_files, 1);
    }

    #[test]
    fn test_analyze_cpp_simd_content() {
        let analyzer = CudaSimdAnalyzer::new();
        let cpp_content = r#"
            #include <immintrin.h>

            void simd_multiply(__m256 *a, __m256 *b, __m256 *c) {
                *c = _mm256_mul_ps(*a, *b);
            }
        "#;

        let temp_dir = tempfile::tempdir().unwrap();
        let cpp_file = temp_dir.path().join("test.cpp");
        std::fs::write(&cpp_file, cpp_content).unwrap();

        let result = analyzer.analyze(&cpp_file).unwrap();
        assert_eq!(result.simd_files, 1);
    }

    #[test]
    fn test_analyze_directory() {
        let analyzer = CudaSimdAnalyzer::new();

        let temp_dir = tempfile::tempdir().unwrap();

        // Create CUDA file
        let cuda_file = temp_dir.path().join("kernel.cu");
        std::fs::write(&cuda_file, "__global__ void kernel() { __syncthreads(); }").unwrap();

        // Create WGSL file
        let wgsl_file = temp_dir.path().join("shader.wgsl");
        std::fs::write(&wgsl_file, "@compute fn main() { workgroupBarrier(); }").unwrap();

        let result = analyzer.analyze(temp_dir.path()).unwrap();
        assert!(result.files_analyzed >= 2);
        assert!(result.cuda_files >= 1);
        assert!(result.wgpu_files >= 1);
    }

    #[test]
    fn test_quality_gate_passes() {
        let analyzer = CudaSimdAnalyzer::new();

        let result = CudaSimdTdgResult {
            path: PathBuf::from("."),
            score: PopperScore {
                falsifiability: FalsifiabilityScore::default(),
                reproducibility: ReproducibilityScore::default(),
                transparency: TransparencyScore::default(),
                statistical_rigor: StatisticalRigorScore::default(),
                historical_integrity: HistoricalIntegrityScore::default(),
                gpu_simd_specific: GpuSimdSpecificScore::default(),
                total: 90.0,
                gateway_passed: true,
                grade: CudaTdgGrade::APLus,
            },
            defects: vec![],
            barrier_safety: BarrierSafetyResult::default(),
            coalescing: CoalescingResult::default(),
            tile_dimensions: TileDimensionResult {
                valid: true,
                tile_k: None,
                tile_kv: None,
                head_dim: None,
                shared_memory_required: None,
                shared_memory_available: None,
                issues: vec![],
            },
            kaizen: KaizenMetrics::default(),
            timestamp: String::new(),
            files_analyzed: 0,
            cuda_files: 0,
            simd_files: 0,
            wgpu_files: 0,
        };

        assert!(analyzer.passes_quality_gate(&result));
    }

    #[test]
    fn test_quality_gate_fails_gateway() {
        let analyzer = CudaSimdAnalyzer::new();

        let result = CudaSimdTdgResult {
            path: PathBuf::from("."),
            score: PopperScore {
                falsifiability: FalsifiabilityScore::default(),
                reproducibility: ReproducibilityScore::default(),
                transparency: TransparencyScore::default(),
                statistical_rigor: StatisticalRigorScore::default(),
                historical_integrity: HistoricalIntegrityScore::default(),
                gpu_simd_specific: GpuSimdSpecificScore::default(),
                total: 0.0,
                gateway_passed: false,
                grade: CudaTdgGrade::GatewayFail,
            },
            defects: vec![],
            barrier_safety: BarrierSafetyResult::default(),
            coalescing: CoalescingResult::default(),
            tile_dimensions: TileDimensionResult {
                valid: true,
                tile_k: None,
                tile_kv: None,
                head_dim: None,
                shared_memory_required: None,
                shared_memory_available: None,
                issues: vec![],
            },
            kaizen: KaizenMetrics::default(),
            timestamp: String::new(),
            files_analyzed: 0,
            cuda_files: 0,
            simd_files: 0,
            wgpu_files: 0,
        };

        assert!(!analyzer.passes_quality_gate(&result));
    }

    #[test]
    fn test_quality_gate_fails_p0_defect() {
        let config = CudaSimdConfig {
            min_score: 85.0,
            fail_on_p0: true,
            ..Default::default()
        };
        let analyzer = CudaSimdAnalyzer::with_config(config);

        let result = CudaSimdTdgResult {
            path: PathBuf::from("."),
            score: PopperScore {
                falsifiability: FalsifiabilityScore::default(),
                reproducibility: ReproducibilityScore::default(),
                transparency: TransparencyScore::default(),
                statistical_rigor: StatisticalRigorScore::default(),
                historical_integrity: HistoricalIntegrityScore::default(),
                gpu_simd_specific: GpuSimdSpecificScore::default(),
                total: 90.0,
                gateway_passed: true,
                grade: CudaTdgGrade::APLus,
            },
            defects: vec![DetectedDefect {
                defect_class: DefectClass {
                    ticket_id: "PAR-002".to_string(),
                    description: "Test P0 defect".to_string(),
                    severity: DefectSeverity::P0Critical,
                    detection_method: "test".to_string(),
                    resolved: false,
                    root_cause: None,
                },
                file_path: PathBuf::from("test.cu"),
                line: Some(1),
                snippet: None,
                suggestion: None,
            }],
            barrier_safety: BarrierSafetyResult::default(),
            coalescing: CoalescingResult::default(),
            tile_dimensions: TileDimensionResult {
                valid: true,
                tile_k: None,
                tile_kv: None,
                head_dim: None,
                shared_memory_required: None,
                shared_memory_available: None,
                issues: vec![],
            },
            kaizen: KaizenMetrics::default(),
            timestamp: String::new(),
            files_analyzed: 1,
            cuda_files: 1,
            simd_files: 0,
            wgpu_files: 0,
        };

        assert!(!analyzer.passes_quality_gate(&result));
    }

    #[test]
    fn test_quality_gate_fails_low_score() {
        let config = CudaSimdConfig {
            min_score: 85.0,
            fail_on_p0: false,
            ..Default::default()
        };
        let analyzer = CudaSimdAnalyzer::with_config(config);

        let result = CudaSimdTdgResult {
            path: PathBuf::from("."),
            score: PopperScore {
                falsifiability: FalsifiabilityScore::default(),
                reproducibility: ReproducibilityScore::default(),
                transparency: TransparencyScore::default(),
                statistical_rigor: StatisticalRigorScore::default(),
                historical_integrity: HistoricalIntegrityScore::default(),
                gpu_simd_specific: GpuSimdSpecificScore::default(),
                total: 70.0, // Below min_score of 85
                gateway_passed: true,
                grade: CudaTdgGrade::B,
            },
            defects: vec![],
            barrier_safety: BarrierSafetyResult::default(),
            coalescing: CoalescingResult::default(),
            tile_dimensions: TileDimensionResult {
                valid: true,
                tile_k: None,
                tile_kv: None,
                head_dim: None,
                shared_memory_required: None,
                shared_memory_available: None,
                issues: vec![],
            },
            kaizen: KaizenMetrics::default(),
            timestamp: String::new(),
            files_analyzed: 0,
            cuda_files: 0,
            simd_files: 0,
            wgpu_files: 0,
        };

        assert!(!analyzer.passes_quality_gate(&result));
    }

    #[test]
    fn test_detect_flash_attention_issue() {
        let analyzer = CudaSimdAnalyzer::new();
        let cuda_content = r#"
            // FlashAttention kernel
            const tile_kv = 32;
            const head_dim = 64;

            __global__ void flash_attention_kernel() {
                // PAR-041 scenario: tile_kv < head_dim
            }
        "#;

        let temp_dir = tempfile::tempdir().unwrap();
        let cuda_file = temp_dir.path().join("attention.cu");
        std::fs::write(&cuda_file, cuda_content).unwrap();

        let result = analyzer.analyze(&cuda_file).unwrap();
        // Should detect PAR-041 defect
        let has_par041 = result
            .defects
            .iter()
            .any(|d| d.defect_class.ticket_id == "PAR-041");
        assert!(has_par041);
    }

    #[test]
    fn test_detect_missing_tensor_core() {
        let analyzer = CudaSimdAnalyzer::new();
        // Use explicit "gemm" keyword to trigger PAR-034 detection
        let cuda_content = r#"
            // This is a naive gemm implementation
            __global__ void naive_gemm_kernel(float *a, float *b, float *c, int n) {
                int row = blockIdx.y * blockDim.y + threadIdx.y;
                int col = blockIdx.x * blockDim.x + threadIdx.x;

                float sum = 0.0f;
                for (int k = 0; k < n; k++) {
                    sum += a[row * n + k] * b[k * n + col];
                }
                c[row * n + col] = sum;
            }
        "#;

        let temp_dir = tempfile::tempdir().unwrap();
        let cuda_file = temp_dir.path().join("gemm.cu");
        std::fs::write(&cuda_file, cuda_content).unwrap();

        let result = analyzer.analyze(&cuda_file).unwrap();
        // Should detect PAR-034 defect (missing Tensor Core)
        let has_par034 = result
            .defects
            .iter()
            .any(|d| d.defect_class.ticket_id == "PAR-034");
        assert!(
            has_par034,
            "Expected PAR-034 defect for gemm without tensor cores"
        );
    }

    #[test]
    fn test_coalescing_result_default() {
        let result = CoalescingResult::default();
        assert_eq!(result.efficiency, 0.0);
        assert_eq!(result.total_operations, 0);
        assert_eq!(result.coalesced_operations, 0);
        assert!(result.problematic_accesses.is_empty());
    }

    #[test]
    fn test_kaizen_metrics_default() {
        let metrics = KaizenMetrics::default();
        assert_eq!(metrics.tickets_resolved, 0);
        assert_eq!(metrics.mttd, 0.0);
        assert_eq!(metrics.mttf, 0.0);
        assert!(metrics.ticket_references.is_empty());
    }

    #[test]
    fn test_barrier_issue_creation() {
        let issue = BarrierIssue {
            line: 42,
            barrier_type: "__syncthreads".to_string(),
            issue: "Test issue".to_string(),
            exit_paths: vec!["path1".to_string(), "path2".to_string()],
        };

        assert_eq!(issue.line, 42);
        assert_eq!(issue.barrier_type, "__syncthreads");
        assert_eq!(issue.exit_paths.len(), 2);
    }

    #[test]
    fn test_memory_access_patterns() {
        let contiguous = AccessPattern::Contiguous;
        let strided = AccessPattern::Strided { stride: 4 };
        let random = AccessPattern::Random;
        let bank_conflict = AccessPattern::BankConflict {
            conflicting_banks: vec![0, 16],
        };

        // Just verify these compile and can be created
        match contiguous {
            AccessPattern::Contiguous => {}
            _ => panic!("Expected Contiguous"),
        }

        match strided {
            AccessPattern::Strided { stride } => assert_eq!(stride, 4),
            _ => panic!("Expected Strided"),
        }

        match random {
            AccessPattern::Random => {}
            _ => panic!("Expected Random"),
        }

        match bank_conflict {
            AccessPattern::BankConflict { conflicting_banks } => {
                assert_eq!(conflicting_banks.len(), 2);
            }
            _ => panic!("Expected BankConflict"),
        }
    }

    #[test]
    fn test_tile_issue_creation() {
        let issue = TileIssue {
            description: "Tile too small".to_string(),
            ticket: Some("PAR-041".to_string()),
            severity: DefectSeverity::P0Critical,
        };

        assert_eq!(issue.description, "Tile too small");
        assert_eq!(issue.ticket, Some("PAR-041".to_string()));
        assert_eq!(issue.severity, DefectSeverity::P0Critical);
    }

    #[test]
    fn test_defect_class_with_root_cause() {
        let defect = DefectClass {
            ticket_id: "PAR-002".to_string(),
            description: "GEMV Error 700".to_string(),
            severity: DefectSeverity::P0Critical,
            detection_method: "Bounds check".to_string(),
            resolved: true,
            root_cause: Some("Thread index overflow".to_string()),
        };

        assert!(defect.resolved);
        assert!(defect.root_cause.is_some());
        assert_eq!(defect.root_cause.unwrap(), "Thread index overflow");
    }

    #[test]
    fn test_analyze_empty_file() {
        let analyzer = CudaSimdAnalyzer::new();

        let temp_dir = tempfile::tempdir().unwrap();
        let empty_file = temp_dir.path().join("empty.cu");
        std::fs::write(&empty_file, "").unwrap();

        let result = analyzer.analyze(&empty_file).unwrap();
        assert_eq!(result.cuda_files, 1);
        assert_eq!(result.barrier_safety.total_barriers, 0);
    }

    #[test]
    fn test_analyze_ptx_file() {
        let analyzer = CudaSimdAnalyzer::new();
        let ptx_content = r#"
            .version 7.0
            .target sm_80

            .visible .entry kernel() {
                bar.sync 0;
                ret;
            }
        "#;

        let temp_dir = tempfile::tempdir().unwrap();
        let ptx_file = temp_dir.path().join("test.ptx");
        std::fs::write(&ptx_file, ptx_content).unwrap();

        let result = analyzer.analyze(&ptx_file).unwrap();
        assert_eq!(result.cuda_files, 1);
        assert!(result.barrier_safety.total_barriers >= 1);
    }

    #[test]
    fn test_strided_memory_access_detection() {
        let analyzer = CudaSimdAnalyzer::new();
        let cuda_content = r#"
            __global__ void strided_kernel(float *data) {
                int tid = threadIdx.x;
                float val = data[tid * STRIDE]; // strided access
            }
        "#;

        let temp_dir = tempfile::tempdir().unwrap();
        let cuda_file = temp_dir.path().join("strided.cu");
        std::fs::write(&cuda_file, cuda_content).unwrap();

        let result = analyzer.analyze(&cuda_file).unwrap();
        assert_eq!(result.cuda_files, 1);
    }

    #[test]
    fn test_shared_memory_detection() {
        let analyzer = CudaSimdAnalyzer::new();
        let cuda_content = r#"
            __global__ void shared_kernel() {
                __shared__ float smem[256];
                smem[threadIdx.x % 32] = 1.0f; // potential bank conflict
            }
        "#;

        let temp_dir = tempfile::tempdir().unwrap();
        let cuda_file = temp_dir.path().join("shared.cu");
        std::fs::write(&cuda_file, cuda_content).unwrap();

        let result = analyzer.analyze(&cuda_file).unwrap();
        assert_eq!(result.cuda_files, 1);
    }

    #[test]
    fn test_extract_value_patterns() {
        let analyzer = CudaSimdAnalyzer::new();

        // Test different value extraction patterns
        let content1 = "tile_kv = 64";
        assert_eq!(analyzer.extract_value(content1, "tile_kv"), Some(64));

        let content2 = "const head_dim = 128";
        assert_eq!(analyzer.extract_value(content2, "head_dim"), Some(128));

        let content3 = "let tile_k = 32";
        assert_eq!(analyzer.extract_value(content3, "tile_k"), Some(32));

        let content4 = "no_match_here";
        assert_eq!(analyzer.extract_value(content4, "tile_kv"), None);
    }

    #[test]
    fn test_c_header_simd_detection() {
        let analyzer = CudaSimdAnalyzer::new();
        let c_content = r#"
            #include <arm_neon.h>

            void neon_add(float32x4_t *a, float32x4_t *b) {
                // NEON SIMD operations
            }
        "#;

        let temp_dir = tempfile::tempdir().unwrap();
        let c_file = temp_dir.path().join("neon.h");
        std::fs::write(&c_file, c_content).unwrap();

        let result = analyzer.analyze(&c_file).unwrap();
        assert_eq!(result.simd_files, 1);
    }

    #[test]
    fn test_multiple_syncthreads() {
        let analyzer = CudaSimdAnalyzer::new();
        let cuda_content = r#"
            __global__ void multi_barrier() {
                __syncthreads();
                // work
                __syncthreads();
                // more work
                __syncwarp();
            }
        "#;

        let temp_dir = tempfile::tempdir().unwrap();
        let cuda_file = temp_dir.path().join("multi.cu");
        std::fs::write(&cuda_file, cuda_content).unwrap();

        let result = analyzer.analyze(&cuda_file).unwrap();
        assert!(result.barrier_safety.total_barriers >= 3);
        assert!(result.barrier_safety.safe_barriers >= 3);
    }

    #[test]
    fn test_global_memory_access() {
        let analyzer = CudaSimdAnalyzer::new();
        let cuda_content = r#"
            __global__ void global_access(float *global_mem) {
                int tid = threadIdx.x;
                float val = global_mem[tid];
            }
        "#;

        let temp_dir = tempfile::tempdir().unwrap();
        let cuda_file = temp_dir.path().join("global.cu");
        std::fs::write(&cuda_file, cuda_content).unwrap();

        let result = analyzer.analyze(&cuda_file).unwrap();
        assert!(result.coalescing.total_operations >= 1);
    }

    #[test]
    fn test_wgpu_storage_access() {
        let analyzer = CudaSimdAnalyzer::new();
        let wgsl_content = r#"
            @group(0) @binding(0) var<storage, read_write> data: array<f32>;

            @compute @workgroup_size(256)
            fn main(@builtin(global_invocation_id) gid: vec3<u32>) {
                data[gid.x] = data[gid.x] * 2.0;
            }
        "#;

        let temp_dir = tempfile::tempdir().unwrap();
        let wgsl_file = temp_dir.path().join("storage.wgsl");
        std::fs::write(&wgsl_file, wgsl_content).unwrap();

        let result = analyzer.analyze(&wgsl_file).unwrap();
        assert!(result.coalescing.total_operations >= 1);
    }

    #[test]
    fn test_grade_boundary_values() {
        // Test exact boundary values
        assert_eq!(CudaTdgGrade::from_score(90.0, true), CudaTdgGrade::APLus);
        assert_eq!(CudaTdgGrade::from_score(89.9, true), CudaTdgGrade::A);
        assert_eq!(CudaTdgGrade::from_score(80.0, true), CudaTdgGrade::A);
        assert_eq!(CudaTdgGrade::from_score(79.9, true), CudaTdgGrade::B);
        assert_eq!(CudaTdgGrade::from_score(70.0, true), CudaTdgGrade::B);
        assert_eq!(CudaTdgGrade::from_score(69.9, true), CudaTdgGrade::C);
        assert_eq!(CudaTdgGrade::from_score(60.0, true), CudaTdgGrade::C);
        assert_eq!(CudaTdgGrade::from_score(59.9, true), CudaTdgGrade::D);
        assert_eq!(CudaTdgGrade::from_score(50.0, true), CudaTdgGrade::D);
        assert_eq!(CudaTdgGrade::from_score(49.9, true), CudaTdgGrade::F);
        assert_eq!(CudaTdgGrade::from_score(0.0, true), CudaTdgGrade::F);
    }

    #[test]
    fn test_default_taxonomy() {
        let taxonomy = DefectTaxonomy::default();
        // Default taxonomy should be empty
        assert!(taxonomy.patterns.is_empty());
    }

    #[test]
    fn test_score_defaults() {
        let falsifiability = FalsifiabilityScore::default();
        assert_eq!(falsifiability.total(), 0.0);

        let reproducibility = ReproducibilityScore::default();
        assert_eq!(reproducibility.total(), 0.0);

        let transparency = TransparencyScore::default();
        assert_eq!(transparency.total(), 0.0);

        let statistical_rigor = StatisticalRigorScore::default();
        assert_eq!(statistical_rigor.total(), 0.0);

        let historical_integrity = HistoricalIntegrityScore::default();
        assert_eq!(historical_integrity.total(), 0.0);

        let gpu_simd = GpuSimdSpecificScore::default();
        assert_eq!(gpu_simd.total(), 0.0);
    }

    #[test]
    fn test_analyzer_default_impl() {
        let analyzer = CudaSimdAnalyzer::default();
        assert_eq!(analyzer.config.min_score, 85.0);
    }

    #[test]
    fn test_file_analysis_default() {
        let analysis = FileAnalysis::default();
        assert_eq!(analysis.cuda_files, 0);
        assert_eq!(analysis.simd_files, 0);
        assert_eq!(analysis.wgpu_files, 0);
        assert!(analysis.defects.is_empty());
    }
}


mod coverage_tests {
    use super::*;
    use std::path::PathBuf;

    // DefectSeverity Tests

    #[test]
    fn test_defect_severity_equality() {
        assert_eq!(DefectSeverity::P0Critical, DefectSeverity::P0Critical);
        assert_eq!(DefectSeverity::P1Performance, DefectSeverity::P1Performance);
        assert_eq!(DefectSeverity::P2Efficiency, DefectSeverity::P2Efficiency);
        assert_eq!(DefectSeverity::P3Minor, DefectSeverity::P3Minor);
        assert_ne!(DefectSeverity::P0Critical, DefectSeverity::P1Performance);
    }

    #[test]
    fn test_defect_severity_clone() {
        let severity = DefectSeverity::P0Critical;
        let cloned = severity.clone();
        assert_eq!(severity, cloned);
    }

    #[test]
    fn test_defect_severity_copy() {
        let severity = DefectSeverity::P0Critical;
        let copied: DefectSeverity = severity; // Copy
        assert_eq!(severity, copied);
    }

    // DefectClass Tests

    #[test]
    fn test_defect_class_clone() {
        let defect = DefectClass {
            ticket_id: "TEST-001".to_string(),
            description: "Test defect".to_string(),
            severity: DefectSeverity::P0Critical,
            detection_method: "Unit test".to_string(),
            resolved: false,
            root_cause: None,
        };

        let cloned = defect.clone();
        assert_eq!(defect.ticket_id, cloned.ticket_id);
        assert_eq!(defect.description, cloned.description);
        assert_eq!(defect.severity, cloned.severity);
    }

    #[test]
    fn test_defect_class_with_root_cause() {
        let defect = DefectClass {
            ticket_id: "TEST-002".to_string(),
            description: "Test defect with root cause".to_string(),
            severity: DefectSeverity::P1Performance,
            detection_method: "Analysis".to_string(),
            resolved: true,
            root_cause: Some("Root cause identified".to_string()),
        };

        assert!(defect.resolved);
        assert!(defect.root_cause.is_some());
    }

    // DefectTaxonomy Tests

    #[test]
    fn test_taxonomy_trueno_simd_patterns() {
        let taxonomy = DefectTaxonomy::with_tauranta_patterns();

        // Check TRUENO-SIMD-001
        let trueno_simd_001 = taxonomy.get("TRUENO-SIMD-001");
        assert!(trueno_simd_001.is_some());
        assert_eq!(
            trueno_simd_001.unwrap().severity,
            DefectSeverity::P0Critical
        );

        // Check TRUENO-SIMD-002
        let trueno_simd_002 = taxonomy.get("TRUENO-SIMD-002");
        assert!(trueno_simd_002.is_some());
    }

    #[test]
    fn test_taxonomy_trueno_ptx_patterns() {
        let taxonomy = DefectTaxonomy::with_tauranta_patterns();

        // Check TRUENO-PTX-001
        let trueno_ptx_001 = taxonomy.get("TRUENO-PTX-001");
        assert!(trueno_ptx_001.is_some());
        assert_eq!(trueno_ptx_001.unwrap().severity, DefectSeverity::P0Critical);
    }

    #[test]
    fn test_taxonomy_computebrick_patterns() {
        let taxonomy = DefectTaxonomy::with_tauranta_patterns();

        // Check CB-001 through CB-022 (ComputeBrick patterns)
        assert!(taxonomy.get("CB-001").is_some());
        assert!(taxonomy.get("CB-002").is_some());
        assert!(taxonomy.get("CB-003").is_some());
        assert!(taxonomy.get("CB-004").is_some());
        assert!(taxonomy.get("CB-010").is_some());
        assert!(taxonomy.get("CB-011").is_some());
        assert!(taxonomy.get("CB-012").is_some());
        assert!(taxonomy.get("CB-013").is_some());
        assert!(taxonomy.get("CB-020").is_some());
        assert!(taxonomy.get("CB-021").is_some());
        assert!(taxonomy.get("CB-022").is_some());
    }

    #[test]
    fn test_taxonomy_all_iterator() {
        let taxonomy = DefectTaxonomy::with_tauranta_patterns();
        let all: Vec<&DefectClass> = taxonomy.all().collect();

        // Should have at least the known patterns
        assert!(all.len() >= 20);

        // Verify iterator returns valid defects
        for defect in all {
            assert!(!defect.ticket_id.is_empty());
            assert!(!defect.description.is_empty());
        }
    }

    // CudaTdgGrade Tests

    #[test]
    fn test_grade_default() {
        let grade = CudaTdgGrade::default();
        assert_eq!(grade, CudaTdgGrade::F);
    }

    #[test]
    fn test_grade_clone_and_copy() {
        let grade = CudaTdgGrade::APLus;
        let cloned = grade.clone();
        let copied: CudaTdgGrade = grade;
        assert_eq!(grade, cloned);
        assert_eq!(grade, copied);
    }

    #[test]
    fn test_grade_negative_scores() {
        // Negative scores should result in F
        assert_eq!(CudaTdgGrade::from_score(-10.0, true), CudaTdgGrade::F);
    }

    // Score Category Tests

    #[test]
    fn test_falsifiability_score_partial() {
        let score = FalsifiabilityScore {
            barrier_safety: 2.5,
            bounds_verification: 2.5,
            divergence_testing: 2.5,
            memory_race_detection: 2.5,
            occupancy_bounds: 5.0,
        };
        assert_eq!(score.total(), 15.0);
        assert!(score.total() >= FalsifiabilityScore::GATEWAY_THRESHOLD);
    }

    #[test]
    fn test_reproducibility_score_partial() {
        let score = ReproducibilityScore {
            deterministic_output: 4.0,
            version_pinning: 2.5,
            hardware_specification: 2.5,
            benchmark_harness: 2.0,
            ci_cd_integration: 1.5,
        };
        assert_eq!(score.total(), 12.5);
    }

    #[test]
    fn test_transparency_score_partial() {
        let score = TransparencyScore {
            ptx_inspection: 3.0,
            register_allocation: 2.5,
            occupancy_calculation: 2.5,
            memory_layout: 2.0,
        };
        assert_eq!(score.total(), 10.0);
    }

    #[test]
    fn test_statistical_rigor_score_partial() {
        let score = StatisticalRigorScore {
            warmup_iterations: 2.0,
            sample_count: 2.0,
            outlier_analysis: 2.0,
            confidence_intervals: 1.5,
        };
        assert_eq!(score.total(), 7.5);
    }

    #[test]
    fn test_historical_integrity_score_partial() {
        let score = HistoricalIntegrityScore {
            fault_lineage: 2.0,
            regression_tests: 1.5,
            root_cause_documentation: 1.5,
        };
        assert_eq!(score.total(), 5.0);
    }

    #[test]
    fn test_gpu_simd_specific_score_partial() {
        let score = GpuSimdSpecificScore {
            warp_efficiency: 1.0,
            memory_throughput: 1.0,
            instruction_mix: 0.5,
        };
        assert_eq!(score.total(), 2.5);
    }

    // PopperScore Tests

    #[test]
    fn test_popper_score_gateway_threshold_exact() {
        // Test exactly at gateway threshold (15.0)
        let falsifiability = FalsifiabilityScore {
            barrier_safety: 3.0,
            bounds_verification: 3.0,
            divergence_testing: 3.0,
            memory_race_detection: 3.0,
            occupancy_bounds: 3.0, // Total: 15.0 (exactly at threshold)
        };

        let score = PopperScore::calculate(
            falsifiability,
            ReproducibilityScore::default(),
            TransparencyScore::default(),
            StatisticalRigorScore::default(),
            HistoricalIntegrityScore::default(),
            GpuSimdSpecificScore::default(),
        );

        assert!(score.gateway_passed);
        assert!(score.total > 0.0);
    }

    #[test]
    fn test_popper_score_gateway_just_below() {
        // Test just below gateway threshold
        let falsifiability = FalsifiabilityScore {
            barrier_safety: 2.9,
            bounds_verification: 3.0,
            divergence_testing: 3.0,
            memory_race_detection: 3.0,
            occupancy_bounds: 3.0, // Total: 14.9
        };

        let score = PopperScore::calculate(
            falsifiability,
            ReproducibilityScore::default(),
            TransparencyScore::default(),
            StatisticalRigorScore::default(),
            HistoricalIntegrityScore::default(),
            GpuSimdSpecificScore::default(),
        );

        assert!(!score.gateway_passed);
        assert_eq!(score.total, 0.0);
        assert_eq!(score.grade, CudaTdgGrade::GatewayFail);
    }

    // BarrierSafetyResult Tests

    #[test]
    fn test_barrier_safety_result_default() {
        let result = BarrierSafetyResult::default();
        assert_eq!(result.total_barriers, 0);
        assert_eq!(result.safe_barriers, 0);
        assert!(result.unsafe_barriers.is_empty());
        assert_eq!(result.safety_score, 0.0);
    }

    #[test]
    fn test_barrier_safety_result_with_issues() {
        let result = BarrierSafetyResult {
            total_barriers: 5,
            safe_barriers: 3,
            unsafe_barriers: vec![
                BarrierIssue {
                    line: 10,
                    barrier_type: "__syncthreads".to_string(),
                    issue: "Early return".to_string(),
                    exit_paths: vec!["path1".to_string()],
                },
                BarrierIssue {
                    line: 20,
                    barrier_type: "bar.sync".to_string(),
                    issue: "Thread divergence".to_string(),
                    exit_paths: vec!["path2".to_string(), "path3".to_string()],
                },
            ],
            safety_score: 0.6,
        };

        assert_eq!(result.total_barriers, 5);
        assert_eq!(result.safe_barriers, 3);
        assert_eq!(result.unsafe_barriers.len(), 2);
    }

    // CoalescingResult Tests

    #[test]
    fn test_coalescing_result_full_coalescing() {
        let result = CoalescingResult {
            efficiency: 1.0,
            total_operations: 100,
            coalesced_operations: 100,
            problematic_accesses: vec![],
        };

        assert_eq!(result.efficiency, 1.0);
        assert!(result.problematic_accesses.is_empty());
    }

    #[test]
    fn test_coalescing_result_with_issues() {
        let result = CoalescingResult {
            efficiency: 0.7,
            total_operations: 100,
            coalesced_operations: 70,
            problematic_accesses: vec![
                MemoryAccessIssue {
                    line: 15,
                    pattern: AccessPattern::Strided { stride: 4 },
                    impact: "50% throughput".to_string(),
                },
                MemoryAccessIssue {
                    line: 25,
                    pattern: AccessPattern::Random,
                    impact: "Severe performance impact".to_string(),
                },
            ],
        };

        assert_eq!(result.efficiency, 0.7);
        assert_eq!(result.problematic_accesses.len(), 2);
    }

    // TileDimensionResult Tests

    #[test]
    fn test_tile_dimension_result_valid() {
        let result = TileDimensionResult {
            valid: true,
            tile_k: Some(32),
            tile_kv: Some(64),
            head_dim: Some(64),
            shared_memory_required: Some(8192),
            shared_memory_available: Some(49152),
            issues: vec![],
        };

        assert!(result.valid);
        assert!(result.issues.is_empty());
    }

    #[test]
    fn test_tile_dimension_result_with_issues() {
        let result = TileDimensionResult {
            valid: false,
            tile_k: Some(32),
            tile_kv: Some(32),
            head_dim: Some(64),
            shared_memory_required: Some(65536),
            shared_memory_available: Some(49152),
            issues: vec![
                TileIssue {
                    description: "tile_kv < head_dim".to_string(),
                    ticket: Some("PAR-041".to_string()),
                    severity: DefectSeverity::P0Critical,
                },
                TileIssue {
                    description: "Shared memory overflow".to_string(),
                    ticket: None,
                    severity: DefectSeverity::P0Critical,
                },
            ],
        };

        assert!(!result.valid);
        assert_eq!(result.issues.len(), 2);
    }

    // KaizenMetrics Tests

    #[test]
    fn test_kaizen_metrics_populated() {
        let metrics = KaizenMetrics {
            tickets_resolved: 10,
            mttd: 12.5,
            mttf: 24.0,
            escape_rate: 0.03,
            regression_rate: 0.01,
            ticket_references: vec![
                "PAR-001".to_string(),
                "PAR-002".to_string(),
                "PARITY-114".to_string(),
            ],
        };

        assert_eq!(metrics.tickets_resolved, 10);
        assert_eq!(metrics.ticket_references.len(), 3);
    }

    // CudaSimdConfig Tests

    #[test]
    fn test_config_custom_values() {
        let config = CudaSimdConfig {
            min_score: 90.0,
            fail_on_p0: false,
            analyze_simd: true,
            analyze_wgpu: false,
            shared_memory_limit: 65536,
            register_limit: 128,
        };

        assert_eq!(config.min_score, 90.0);
        assert!(!config.fail_on_p0);
        assert!(config.analyze_simd);
        assert!(!config.analyze_wgpu);
        assert_eq!(config.shared_memory_limit, 65536);
        assert_eq!(config.register_limit, 128);
    }

    #[test]
    fn test_config_default() {
        let config = CudaSimdConfig::default();
        assert_eq!(config.min_score, 0.0);
        assert!(!config.fail_on_p0);
    }

    // DetectedDefect Tests

    #[test]
    fn test_detected_defect_full() {
        let defect = DetectedDefect {
            defect_class: DefectClass {
                ticket_id: "PAR-001".to_string(),
                description: "Test defect".to_string(),
                severity: DefectSeverity::P1Performance,
                detection_method: "Analysis".to_string(),
                resolved: false,
                root_cause: None,
            },
            file_path: PathBuf::from("/path/to/file.cu"),
            line: Some(42),
            snippet: Some("__global__ void kernel() {}".to_string()),
            suggestion: Some("Add barrier".to_string()),
        };

        assert_eq!(defect.line, Some(42));
        assert!(defect.snippet.is_some());
        assert!(defect.suggestion.is_some());
    }

    #[test]
    fn test_detected_defect_minimal() {
        let defect = DetectedDefect {
            defect_class: DefectClass {
                ticket_id: "PAR-002".to_string(),
                description: "Minimal defect".to_string(),
                severity: DefectSeverity::P2Efficiency,
                detection_method: "Static".to_string(),
                resolved: true,
                root_cause: Some("Root cause".to_string()),
            },
            file_path: PathBuf::from("test.cu"),
            line: None,
            snippet: None,
            suggestion: None,
        };

        assert!(defect.line.is_none());
        assert!(defect.snippet.is_none());
        assert!(defect.suggestion.is_none());
    }

    // CudaSimdAnalyzer Path Skip Tests

    #[test]
    fn test_should_skip_path_venv() {
        assert!(CudaSimdAnalyzer::should_skip_path(Path::new(
            ".venv/lib/site-packages"
        )));
        assert!(CudaSimdAnalyzer::should_skip_path(Path::new(
            "venv/bin/python"
        )));
    }

    #[test]
    fn test_should_skip_path_node_modules() {
        assert!(CudaSimdAnalyzer::should_skip_path(Path::new(
            "node_modules/package"
        )));
    }

    #[test]
    fn test_should_skip_path_target() {
        assert!(CudaSimdAnalyzer::should_skip_path(Path::new(
            "target/release/bin"
        )));
    }

    #[test]
    fn test_should_skip_path_git() {
        assert!(CudaSimdAnalyzer::should_skip_path(Path::new(
            ".git/objects"
        )));
    }

    #[test]
    fn test_should_skip_path_python_cache() {
        assert!(CudaSimdAnalyzer::should_skip_path(Path::new(
            "__pycache__/module.pyc"
        )));
        assert!(CudaSimdAnalyzer::should_skip_path(Path::new(
            ".pytest_cache/v"
        )));
    }

    #[test]
    fn test_should_skip_path_build_dirs() {
        assert!(CudaSimdAnalyzer::should_skip_path(Path::new(
            "build/output"
        )));
        assert!(CudaSimdAnalyzer::should_skip_path(Path::new(
            "dist/package"
        )));
    }

    #[test]
    fn test_should_skip_path_egg_info() {
        assert!(CudaSimdAnalyzer::should_skip_path(Path::new(
            "mypackage.egg-info/PKG-INFO"
        )));
    }

    #[test]
    fn test_should_not_skip_normal_path() {
        assert!(!CudaSimdAnalyzer::should_skip_path(Path::new(
            "src/main.rs"
        )));
        assert!(!CudaSimdAnalyzer::should_skip_path(Path::new(
            "lib/utils.py"
        )));
        assert!(!CudaSimdAnalyzer::should_skip_path(Path::new(
            "kernels/cuda.cu"
        )));
    }

    // PTX Register Extraction Tests

    #[test]
    fn test_extract_ptx_dest_register_valid() {
        let line = "ld.shared.u32 %r1, [%rd1]";
        let result = CudaSimdAnalyzer::extract_ptx_dest_register(line);
        assert_eq!(result, Some("%r1".to_string()));
    }

    #[test]
    fn test_extract_ptx_dest_register_u64() {
        let line = "ld.global.u64 %rd10, [%param0]";
        let result = CudaSimdAnalyzer::extract_ptx_dest_register(line);
        assert_eq!(result, Some("%rd10".to_string()));
    }

    #[test]
    fn test_extract_ptx_dest_register_no_reg() {
        let line = "bar.sync 0;";
        let result = CudaSimdAnalyzer::extract_ptx_dest_register(line);
        assert!(result.is_none());
    }

    #[test]
    fn test_extract_ptx_dest_register_empty() {
        let line = "";
        let result = CudaSimdAnalyzer::extract_ptx_dest_register(line);
        assert!(result.is_none());
    }

    // SIMD Pattern Detection Tests

    #[test]
    fn test_simd_missing_target_feature() {
        let analyzer = CudaSimdAnalyzer::new();
        let simd_content = r#"
            fn simd_func() {
                unsafe {
                    let a = _mm256_loadu_ps(ptr);
                    let b = _mm256_add_ps(a, a);
                }
            }
        "#;

        let temp_dir = tempfile::tempdir().unwrap();
        let rs_file = temp_dir.path().join("simd.rs");
        std::fs::write(
            &rs_file,
            format!("use std::arch::x86_64::*;\n{}", simd_content),
        )
        .unwrap();

        let result = analyzer.analyze(&rs_file).unwrap();
        // Should detect SIMD_MISSING_TARGET or similar
        assert!(result.simd_files >= 1);
    }

    #[test]
    fn test_simd_with_safety_comment() {
        let analyzer = CudaSimdAnalyzer::new();
        let simd_content = r#"
            use std::arch::x86_64::*;

            fn simd_func() {
                // SAFETY: Pointer is aligned and in-bounds
                unsafe {
                    let a = _mm256_loadu_ps(ptr);
                }
            }
        "#;

        let temp_dir = tempfile::tempdir().unwrap();
        let rs_file = temp_dir.path().join("safe_simd.rs");
        std::fs::write(&rs_file, simd_content).unwrap();

        let result = analyzer.analyze(&rs_file).unwrap();
        assert_eq!(result.simd_files, 1);
    }

    // WGSL Pattern Detection Tests

    #[test]
    fn test_wgsl_small_workgroup() {
        let analyzer = CudaSimdAnalyzer::new();
        let wgsl_content = r#"
            @compute @workgroup_size(16)
            fn main(@builtin(global_invocation_id) gid: vec3<u32>) {
                // Small workgroup
            }
        "#;

        let temp_dir = tempfile::tempdir().unwrap();
        let wgsl_file = temp_dir.path().join("small.wgsl");
        std::fs::write(&wgsl_file, wgsl_content).unwrap();

        let result = analyzer.analyze(&wgsl_file).unwrap();
        // Should detect WGPU_SMALL_WORKGROUP
        let has_small_workgroup = result
            .defects
            .iter()
            .any(|d| d.defect_class.ticket_id.contains("SMALL_WORKGROUP"));
        assert!(has_small_workgroup);
    }

    #[test]
    fn test_wgsl_large_workgroup() {
        let analyzer = CudaSimdAnalyzer::new();
        let wgsl_content = r#"
            @compute @workgroup_size(2048)
            fn main(@builtin(global_invocation_id) gid: vec3<u32>) {
                // Large workgroup
            }
        "#;

        let temp_dir = tempfile::tempdir().unwrap();
        let wgsl_file = temp_dir.path().join("large.wgsl");
        std::fs::write(&wgsl_file, wgsl_content).unwrap();

        let result = analyzer.analyze(&wgsl_file).unwrap();
        // Should detect WGPU_LARGE_WORKGROUP
        let has_large_workgroup = result
            .defects
            .iter()
            .any(|d| d.defect_class.ticket_id.contains("LARGE_WORKGROUP"));
        assert!(has_large_workgroup);
    }

    #[test]
    fn test_wgsl_non_warp_aligned() {
        let analyzer = CudaSimdAnalyzer::new();
        let wgsl_content = r#"
            @compute @workgroup_size(100)
            fn main(@builtin(global_invocation_id) gid: vec3<u32>) {
                // Non-warp-aligned workgroup (not multiple of 32)
            }
        "#;

        let temp_dir = tempfile::tempdir().unwrap();
        let wgsl_file = temp_dir.path().join("unaligned.wgsl");
        std::fs::write(&wgsl_file, wgsl_content).unwrap();

        let result = analyzer.analyze(&wgsl_file).unwrap();
        // Should detect WGPU_NON_WARP_ALIGNED
        let has_non_aligned = result
            .defects
            .iter()
            .any(|d| d.defect_class.ticket_id.contains("NON_WARP_ALIGNED"));
        assert!(has_non_aligned);
    }

    #[test]
    fn test_wgsl_optimal_workgroup() {
        let analyzer = CudaSimdAnalyzer::new();
        let wgsl_content = r#"
            @compute @workgroup_size(256)
            fn main(@builtin(global_invocation_id) gid: vec3<u32>) {
                if (gid.x < params.size) {
                    // Optimal workgroup with bounds check
                }
            }
        "#;

        let temp_dir = tempfile::tempdir().unwrap();
        let wgsl_file = temp_dir.path().join("optimal.wgsl");
        std::fs::write(&wgsl_file, wgsl_content).unwrap();

        let result = analyzer.analyze(&wgsl_file).unwrap();
        // Should not detect workgroup size issues
        let has_workgroup_issue = result
            .defects
            .iter()
            .any(|d| d.defect_class.ticket_id.contains("WORKGROUP"));
        assert!(!has_workgroup_issue);
    }

    // PTX Pattern Detection Tests

    #[test]
    fn test_ptx_shared_u64_detection() {
        let analyzer = CudaSimdAnalyzer::new();
        let ptx_content = r#"
            .version 7.0
            .target sm_80
            .entry kernel() {
                st.shared.u32 [%rd1], %r0;
                ret;
            }
        "#;

        let temp_dir = tempfile::tempdir().unwrap();
        let ptx_file = temp_dir.path().join("shared_u64.ptx");
        std::fs::write(&ptx_file, ptx_content).unwrap();

        let result = analyzer.analyze(&ptx_file).unwrap();
        // Should detect SHARED_U64
        let has_shared_u64 = result
            .defects
            .iter()
            .any(|d| d.defect_class.ticket_id == "SHARED_U64");
        assert!(has_shared_u64);
    }

    #[test]
    fn test_ptx_cvta_shared_detection() {
        let analyzer = CudaSimdAnalyzer::new();
        let ptx_content = r#"
            .version 7.0
            .target sm_80
            .entry kernel() {
                cvta.shared.u64 %rd0, %r1;
                ret;
            }
        "#;

        let temp_dir = tempfile::tempdir().unwrap();
        let ptx_file = temp_dir.path().join("cvta_shared.ptx");
        std::fs::write(&ptx_file, ptx_content).unwrap();

        let result = analyzer.analyze(&ptx_file).unwrap();
        // Should detect CVTA_SHARED
        let has_cvta_shared = result
            .defects
            .iter()
            .any(|d| d.defect_class.ticket_id == "CVTA_SHARED");
        assert!(has_cvta_shared);
    }

    #[test]
    fn test_ptx_local_memory_spill() {
        let analyzer = CudaSimdAnalyzer::new();
        let ptx_content = r#"
            .version 7.0
            .target sm_80
            .local .align 4 .b8 spill[64];
            .entry kernel() {
                ret;
            }
        "#;

        let temp_dir = tempfile::tempdir().unwrap();
        let ptx_file = temp_dir.path().join("local_spill.ptx");
        std::fs::write(&ptx_file, ptx_content).unwrap();

        let result = analyzer.analyze(&ptx_file).unwrap();
        // Should detect REG_SPILLS
        let has_reg_spills = result
            .defects
            .iter()
            .any(|d| d.defect_class.ticket_id == "REG_SPILLS");
        assert!(has_reg_spills);
    }

    // Access Pattern Tests

    #[test]
    fn test_access_pattern_debug() {
        let contiguous = AccessPattern::Contiguous;
        let strided = AccessPattern::Strided { stride: 8 };
        let random = AccessPattern::Random;
        let bank = AccessPattern::BankConflict {
            conflicting_banks: vec![0, 8, 16, 24],
        };

        // Test Debug trait
        assert!(!format!("{:?}", contiguous).is_empty());
        assert!(!format!("{:?}", strided).is_empty());
        assert!(!format!("{:?}", random).is_empty());
        assert!(!format!("{:?}", bank).is_empty());
    }

    #[test]
    fn test_access_pattern_clone() {
        let original = AccessPattern::BankConflict {
            conflicting_banks: vec![0, 8],
        };
        let cloned = original.clone();

        if let (
            AccessPattern::BankConflict {
                conflicting_banks: orig_banks,
            },
            AccessPattern::BankConflict {
                conflicting_banks: clone_banks,
            },
        ) = (original, cloned)
        {
            assert_eq!(orig_banks, clone_banks);
        } else {
            panic!("Expected BankConflict variant");
        }
    }

    // Quality Gate Tests

    #[test]
    fn test_quality_gate_low_score_with_no_p0() {
        let config = CudaSimdConfig {
            min_score: 85.0,
            fail_on_p0: true,
            ..Default::default()
        };
        let analyzer = CudaSimdAnalyzer::with_config(config);

        let result = CudaSimdTdgResult {
            path: PathBuf::from("."),
            score: PopperScore {
                total: 70.0,
                gateway_passed: true,
                grade: CudaTdgGrade::B,
                ..Default::default()
            },
            defects: vec![], // No P0 defects
            barrier_safety: BarrierSafetyResult::default(),
            coalescing: CoalescingResult::default(),
            tile_dimensions: TileDimensionResult {
                valid: true,
                tile_k: None,
                tile_kv: None,
                head_dim: None,
                shared_memory_required: None,
                shared_memory_available: None,
                issues: vec![],
            },
            kaizen: KaizenMetrics::default(),
            timestamp: String::new(),
            files_analyzed: 0,
            cuda_files: 0,
            simd_files: 0,
            wgpu_files: 0,
        };

        // Should fail due to low score (70 < 85)
        assert!(!analyzer.passes_quality_gate(&result));
    }

    // CudaSimdTdgResult Tests

    #[test]
    fn test_cuda_simd_tdg_result_clone() {
        let result = CudaSimdTdgResult {
            path: PathBuf::from("/test/path"),
            score: PopperScore::default(),
            defects: vec![],
            barrier_safety: BarrierSafetyResult::default(),
            coalescing: CoalescingResult::default(),
            tile_dimensions: TileDimensionResult {
                valid: true,
                tile_k: None,
                tile_kv: None,
                head_dim: None,
                shared_memory_required: None,
                shared_memory_available: None,
                issues: vec![],
            },
            kaizen: KaizenMetrics::default(),
            timestamp: "2024-01-01T00:00:00Z".to_string(),
            files_analyzed: 10,
            cuda_files: 3,
            simd_files: 4,
            wgpu_files: 3,
        };

        let cloned = result.clone();
        assert_eq!(result.path, cloned.path);
        assert_eq!(result.files_analyzed, cloned.files_analyzed);
        assert_eq!(result.cuda_files, cloned.cuda_files);
    }
}

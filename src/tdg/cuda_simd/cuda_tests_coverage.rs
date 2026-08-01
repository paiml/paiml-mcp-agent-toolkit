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
        let grade = CudaTdgGrade::APlus;
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
        // SAFETY: String literal test fixture -- not an actual unsafe block in this file.
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

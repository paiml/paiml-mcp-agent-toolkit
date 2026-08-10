// Tests for CUDA TDG handlers
// Extracted for file health compliance (CB-040)

use super::*;

mod tests {
    use super::*;

    #[test]
    fn test_config_creation() {
        let config = CudaTdgCommandConfig {
            path: PathBuf::from("."),
            command: None,
            format: CudaTdgOutputFormat::Terminal,
            min_score: 85.0,
            fail_on_p0: true,
            simd: true,
            wgpu: true,
            output: None,
            quiet: false,
        };

        assert_eq!(config.min_score, 85.0);
        assert!(config.fail_on_p0);
    }
}

mod coverage_tests {
    use super::*;
    use crate::tdg::{
        BarrierIssue, BarrierSafetyResult, CoalescingResult, CudaSimdTdgResult, CudaTdgGrade,
        DetectedDefect, FalsifiabilityScore, GpuSimdSpecificScore, HistoricalIntegrityScore,
        KaizenMetrics, PopperScore, ReproducibilityScore, StatisticalRigorScore,
        TileDimensionResult, TransparencyScore,
    };
    use std::io::Write;
    use std::path::Path;
    use tempfile::TempDir;

    // Helper functions to create mock data

    fn create_mock_popper_score(total: f64, gateway_passed: bool) -> PopperScore {
        let falsifiability = FalsifiabilityScore {
            barrier_safety: 4.0,
            bounds_verification: 4.0,
            divergence_testing: 4.0,
            memory_race_detection: 4.0,
            occupancy_bounds: 4.0,
        };
        PopperScore {
            falsifiability,
            reproducibility: ReproducibilityScore::default(),
            transparency: TransparencyScore::default(),
            statistical_rigor: StatisticalRigorScore::default(),
            historical_integrity: HistoricalIntegrityScore::default(),
            gpu_simd_specific: GpuSimdSpecificScore::default(),
            total,
            gateway_passed,
            grade: CudaTdgGrade::from_score(total, gateway_passed),
        }
    }

    fn create_mock_result(path: &Path) -> CudaSimdTdgResult {
        CudaSimdTdgResult {
            path: path.to_path_buf(),
            score: create_mock_popper_score(85.0, true),
            defects: vec![],
            barrier_safety: BarrierSafetyResult {
                total_barriers: 5,
                safe_barriers: 5,
                unsafe_barriers: vec![],
                safety_score: 1.0,
            },
            coalescing: CoalescingResult {
                efficiency: 0.95,
                total_operations: 100,
                coalesced_operations: 95,
                problematic_accesses: vec![],
            },
            tile_dimensions: TileDimensionResult {
                valid: true,
                tile_k: Some(64),
                tile_kv: Some(128),
                head_dim: Some(64),
                shared_memory_required: Some(16384),
                shared_memory_available: Some(49152),
                issues: vec![],
            },
            kaizen: KaizenMetrics {
                tickets_resolved: 10,
                mttd: 2.5,
                mttf: 4.0,
                escape_rate: 0.05,
                regression_rate: 0.02,
                ticket_references: vec!["PARITY-114".to_string(), "PAR-041".to_string()],
            },
            timestamp: "2025-01-09T00:00:00Z".to_string(),
            files_analyzed: 10,
            cuda_files: 3,
            simd_files: 5,
            wgpu_files: 2,
        }
    }

    fn create_mock_result_with_defects(path: &Path) -> CudaSimdTdgResult {
        let taxonomy = DefectTaxonomy::with_tauranta_patterns();
        let p0_defect = taxonomy.get("PARITY-114").unwrap().clone();
        let p1_defect = taxonomy.get("PAR-001").unwrap().clone();

        let mut result = create_mock_result(path);
        result.defects = vec![
            DetectedDefect {
                defect_class: p0_defect,
                file_path: path.join("test.cu"),
                line: Some(42),
                snippet: Some("__syncthreads();".to_string()),
                suggestion: Some("Ensure all threads reach barrier".to_string()),
            },
            DetectedDefect {
                defect_class: p1_defect,
                file_path: path.join("kernel.ptx"),
                line: Some(100),
                snippet: None,
                suggestion: None,
            },
        ];
        result.score = create_mock_popper_score(65.0, true);
        result
    }

    fn create_mock_result_with_unsafe_barriers(path: &Path) -> CudaSimdTdgResult {
        let mut result = create_mock_result(path);
        result.barrier_safety = BarrierSafetyResult {
            total_barriers: 5,
            safe_barriers: 3,
            unsafe_barriers: vec![
                BarrierIssue {
                    line: 42,
                    barrier_type: "bar.sync".to_string(),
                    issue: "Thread may exit before barrier".to_string(),
                    exit_paths: vec!["early_return_path".to_string()],
                },
                BarrierIssue {
                    line: 100,
                    barrier_type: "__syncthreads".to_string(),
                    issue: "Divergent branch before barrier".to_string(),
                    exit_paths: vec!["if_branch".to_string(), "else_branch".to_string()],
                },
            ],
            safety_score: 0.6,
        };
        result
    }

    fn create_config_with_format(format: CudaTdgOutputFormat) -> CudaTdgCommandConfig {
        CudaTdgCommandConfig {
            path: PathBuf::from("."),
            command: None,
            format,
            min_score: 85.0,
            fail_on_p0: true,
            simd: true,
            wgpu: true,
            output: None,
            quiet: false,
        }
    }

    // Tests for format_result

    #[test]
    fn test_format_result_json() {
        let temp_dir = TempDir::new().unwrap();
        let result = create_mock_result(temp_dir.path());
        let config = create_config_with_format(CudaTdgOutputFormat::Json);

        let output = format_result(&result, &config).unwrap();
        assert!(output.contains("\"total\""));
        assert!(output.contains("\"grade\""));
        assert!(output.contains("\"gateway_passed\""));
    }

    #[test]
    fn test_format_result_sarif() {
        let temp_dir = TempDir::new().unwrap();
        let result = create_mock_result_with_defects(temp_dir.path());
        let config = create_config_with_format(CudaTdgOutputFormat::Sarif);

        let output = format_result(&result, &config).unwrap();
        assert!(output.contains("\"$schema\""));
        assert!(output.contains("sarif-schema-2.1.0"));
        assert!(output.contains("\"version\": \"2.1.0\""));
        assert!(output.contains("pmat cuda-tdg"));
    }

    #[test]
    fn test_format_result_markdown() {
        let temp_dir = TempDir::new().unwrap();
        let result = create_mock_result(temp_dir.path());
        let config = create_config_with_format(CudaTdgOutputFormat::Markdown);

        let output = format_result(&result, &config).unwrap();
        assert!(output.contains("# CUDA-SIMD TDG Analysis Report"));
        assert!(output.contains("## Summary"));
        assert!(output.contains("## Score Breakdown"));
    }

    #[test]
    fn test_format_result_terminal() {
        let temp_dir = TempDir::new().unwrap();
        let result = create_mock_result(temp_dir.path());
        let config = create_config_with_format(CudaTdgOutputFormat::Terminal);

        let output = format_result(&result, &config).unwrap();
        assert!(output.contains("CUDA-SIMD TDG Analysis"));
        assert!(output.contains("Score:"));
        assert!(output.contains("Gateway:"));
    }

    #[test]
    fn test_format_result_terminal_quiet() {
        let temp_dir = TempDir::new().unwrap();
        let result = create_mock_result(temp_dir.path());
        let mut config = create_config_with_format(CudaTdgOutputFormat::Terminal);
        config.quiet = true;

        let output = format_result(&result, &config).unwrap();
        assert_eq!(output, "85.0");
    }

    // Tests for format_analysis

    #[test]
    fn test_format_analysis_json() {
        let temp_dir = TempDir::new().unwrap();
        let result = create_mock_result(temp_dir.path());
        let config = create_config_with_format(CudaTdgOutputFormat::Json);

        let output = format_analysis(&result, &config).unwrap();
        assert!(output.contains("\"files_analyzed\""));
        assert!(output.contains("\"cuda_files\""));
    }

    #[test]
    fn test_format_analysis_terminal() {
        let temp_dir = TempDir::new().unwrap();
        let result = create_mock_result(temp_dir.path());
        let config = create_config_with_format(CudaTdgOutputFormat::Terminal);

        let output = format_analysis(&result, &config).unwrap();
        assert!(output.contains("CUDA-SIMD Analysis Results"));
        assert!(output.contains("Files Analyzed:"));
        assert!(output.contains("CUDA:"));
        assert!(output.contains("SIMD:"));
        assert!(output.contains("WGPU:"));
    }

    #[test]
    fn test_format_analysis_with_defects() {
        let temp_dir = TempDir::new().unwrap();
        let result = create_mock_result_with_defects(temp_dir.path());
        let config = create_config_with_format(CudaTdgOutputFormat::Terminal);

        let output = format_analysis(&result, &config).unwrap();
        assert!(output.contains("Defects:"));
        assert!(output.contains("PARITY-114"));
        assert!(output.contains("P0Critical"));
    }

    // Tests for format_score_summary and format_score_breakdown

    #[test]
    fn test_format_score_summary_json() {
        let score = create_mock_popper_score(85.0, true);
        let config = create_config_with_format(CudaTdgOutputFormat::Json);

        let output = format_score_summary(&score, &config).unwrap();
        assert!(output.contains("\"total\""));
        assert!(output.contains("85"));
    }

    #[test]
    fn test_format_score_summary_terminal() {
        let score = create_mock_popper_score(85.0, true);
        let config = create_config_with_format(CudaTdgOutputFormat::Terminal);

        let output = format_score_summary(&score, &config).unwrap();
        assert!(output.contains("85.0/100"));
        assert!(output.contains("Gateway: PASSED"));
    }

    #[test]
    fn test_format_score_summary_gateway_failed() {
        let score = create_mock_popper_score(0.0, false);
        let config = create_config_with_format(CudaTdgOutputFormat::Terminal);

        let output = format_score_summary(&score, &config).unwrap();
        assert!(output.contains("Gateway: FAILED"));
    }

    #[test]
    fn test_format_score_breakdown_json() {
        let score = create_mock_popper_score(85.0, true);
        let config = create_config_with_format(CudaTdgOutputFormat::Json);

        let output = format_score_breakdown(&score, &config).unwrap();
        assert!(output.contains("\"falsifiability\""));
        assert!(output.contains("\"reproducibility\""));
    }

    #[test]
    fn test_format_score_breakdown_terminal() {
        let score = create_mock_popper_score(85.0, true);
        let config = create_config_with_format(CudaTdgOutputFormat::Terminal);

        let output = format_score_breakdown(&score, &config).unwrap();
        assert!(output.contains("100-Point Popper Falsification Score"));
        assert!(output.contains("Category Breakdown:"));
        assert!(output.contains("A. Falsifiability & Testability (GATEWAY)"));
        assert!(output.contains("B. Reproducibility Infrastructure"));
        assert!(output.contains("C. Transparency & Openness"));
        assert!(output.contains("D. Statistical Rigor"));
        assert!(output.contains("E. Historical Integrity"));
        assert!(output.contains("F. GPU/SIMD Specific"));
        assert!(output.contains("Barrier Safety:"));
        assert!(output.contains("Bounds Verification:"));
    }

    // Tests for format_barrier_safety

    #[test]
    fn test_format_barrier_safety_json() {
        let temp_dir = TempDir::new().unwrap();
        let result = create_mock_result(temp_dir.path());
        let config = create_config_with_format(CudaTdgOutputFormat::Json);

        let output = format_barrier_safety(&result, &config).unwrap();
        assert!(output.contains("\"total_barriers\""));
        assert!(output.contains("\"safe_barriers\""));
        assert!(output.contains("\"safety_score\""));
    }

    #[test]
    fn test_format_barrier_safety_terminal() {
        let temp_dir = TempDir::new().unwrap();
        let result = create_mock_result(temp_dir.path());
        let config = create_config_with_format(CudaTdgOutputFormat::Terminal);

        let output = format_barrier_safety(&result, &config).unwrap();
        assert!(output.contains("Barrier Safety Analysis (PARITY-114)"));
        assert!(output.contains("Total Barriers:"));
        assert!(output.contains("Safe Barriers:"));
        assert!(output.contains("Unsafe Barriers:"));
        assert!(output.contains("Safety Score:"));
    }

    #[test]
    fn test_format_barrier_safety_with_unsafe_barriers() {
        let temp_dir = TempDir::new().unwrap();
        let result = create_mock_result_with_unsafe_barriers(temp_dir.path());
        let config = create_config_with_format(CudaTdgOutputFormat::Terminal);

        let output = format_barrier_safety(&result, &config).unwrap();
        assert!(output.contains("Unsafe Barriers Detected:"));
        assert!(output.contains("bar.sync"));
        assert!(output.contains("__syncthreads"));
        assert!(output.contains("Thread may exit before barrier"));
    }

    // Tests for format_terminal_output

    #[test]
    fn test_format_terminal_output_high_score() {
        let temp_dir = TempDir::new().unwrap();
        let result = create_mock_result(temp_dir.path());

        let output = format_terminal_output(&result).unwrap();
        assert!(output.contains("CUDA-SIMD TDG Analysis"));
        assert!(output.contains("Score:"));
        assert!(output.contains("85.0/100"));
        assert!(output.contains("PASSED"));
        assert!(output.contains("Memory Coalescing:"));
    }

    #[test]
    fn test_format_terminal_output_with_defects() {
        let temp_dir = TempDir::new().unwrap();
        let result = create_mock_result_with_defects(temp_dir.path());

        let output = format_terminal_output(&result).unwrap();
        assert!(output.contains("Defects: 2 total"));
        assert!(output.contains("P0 Critical: 1"));
        assert!(output.contains("P1 Performance: 1"));
    }

    #[test]
    fn test_format_terminal_output_different_grades() {
        let temp_dir = TempDir::new().unwrap();

        // Test A+ grade
        let mut result = create_mock_result(temp_dir.path());
        result.score = create_mock_popper_score(95.0, true);
        let output = format_terminal_output(&result).unwrap();
        assert!(output.contains("95.0/100"));

        // Test B grade
        result.score = create_mock_popper_score(75.0, true);
        let output = format_terminal_output(&result).unwrap();
        assert!(output.contains("75.0/100"));

        // Test C grade
        result.score = create_mock_popper_score(65.0, true);
        let output = format_terminal_output(&result).unwrap();
        assert!(output.contains("65.0/100"));

        // Test Gateway fail
        result.score = create_mock_popper_score(0.0, false);
        let output = format_terminal_output(&result).unwrap();
        assert!(output.contains("FAILED"));
    }

    // Tests for format_markdown_report

    #[test]
    fn test_format_markdown_report_basic() {
        let temp_dir = TempDir::new().unwrap();
        let result = create_mock_result(temp_dir.path());

        let output = format_markdown_report(&result).unwrap();
        assert!(output.contains("# CUDA-SIMD TDG Analysis Report"));
        assert!(output.contains("**Path**:"));
        assert!(output.contains("**Timestamp**:"));
        assert!(output.contains("## Summary"));
        assert!(output.contains("| Metric | Value |"));
        assert!(output.contains("| Score |"));
        assert!(output.contains("| Grade |"));
        assert!(output.contains("| Gateway |"));
        assert!(output.contains("## Score Breakdown"));
    }

    #[test]
    fn test_format_markdown_report_with_defects() {
        let temp_dir = TempDir::new().unwrap();
        let result = create_mock_result_with_defects(temp_dir.path());

        let output = format_markdown_report(&result).unwrap();
        assert!(output.contains("## Defects"));
        assert!(output.contains("| Severity | Ticket | Description | File |"));
        assert!(output.contains("PARITY-114"));
    }

    // Tests for format_html_report

    #[test]
    fn test_format_html_report_basic() {
        let temp_dir = TempDir::new().unwrap();
        let result = create_mock_result(temp_dir.path());

        let output = format_html_report(&result).unwrap();
        assert!(output.contains("<!DOCTYPE html>"));
        assert!(output.contains("<title>CUDA-SIMD TDG Report</title>"));
        assert!(output.contains("<h1>CUDA-SIMD TDG Analysis Report</h1>"));
        assert!(output.contains("Score:"));
        assert!(output.contains("85.0/100"));
    }

    #[test]
    fn test_format_html_report_pass_vs_fail_class() {
        let temp_dir = TempDir::new().unwrap();

        // High score should use "pass" class
        let mut result = create_mock_result(temp_dir.path());
        result.score = create_mock_popper_score(90.0, true);
        let output = format_html_report(&result).unwrap();
        assert!(output.contains("class=\"pass\""));

        // Low score should use "fail" class
        result.score = create_mock_popper_score(50.0, true);
        let output = format_html_report(&result).unwrap();
        assert!(output.contains("class=\"fail\""));
    }

    // Tests for format_sarif

    #[test]
    fn test_format_sarif_structure() {
        let temp_dir = TempDir::new().unwrap();
        let result = create_mock_result_with_defects(temp_dir.path());

        let output = format_sarif(&result).unwrap();
        let sarif: serde_json::Value = serde_json::from_str(&output).unwrap();

        assert_eq!(sarif["version"], "2.1.0");
        assert!(sarif["runs"].is_array());
        assert!(sarif["runs"][0]["tool"]["driver"]["name"]
            .as_str()
            .unwrap()
            .contains("pmat"));
        assert!(sarif["runs"][0]["results"].is_array());
    }

    #[test]
    fn test_format_sarif_defect_levels() {
        let temp_dir = TempDir::new().unwrap();
        let result = create_mock_result_with_defects(temp_dir.path());

        let output = format_sarif(&result).unwrap();
        let sarif: serde_json::Value = serde_json::from_str(&output).unwrap();

        let results = sarif["runs"][0]["results"].as_array().unwrap();
        assert!(results.iter().any(|r| r["level"] == "error"));
        assert!(results.iter().any(|r| r["level"] == "warning"));
    }

    // Tests for write_output

    #[test]
    fn test_write_output_to_file() {
        let temp_dir = TempDir::new().unwrap();
        let output_path = &temp_dir.path().to_path_buf().join("output.txt");
        let config = CudaTdgCommandConfig {
            path: PathBuf::from("."),
            command: None,
            format: CudaTdgOutputFormat::Terminal,
            min_score: 85.0,
            fail_on_p0: true,
            simd: true,
            wgpu: true,
            output: Some(output_path.clone()),
            quiet: false,
        };

        write_output("test output content", &config).unwrap();

        let content = fs::read_to_string(output_path).unwrap();
        assert_eq!(content, "test output content");
    }

    #[test]
    fn test_write_output_to_stdout() {
        let config = CudaTdgCommandConfig {
            path: PathBuf::from("."),
            command: None,
            format: CudaTdgOutputFormat::Terminal,
            min_score: 85.0,
            fail_on_p0: true,
            simd: true,
            wgpu: true,
            output: None,
            quiet: false,
        };

        // This should not panic - it writes to stdout
        write_output("test output", &config).unwrap();
    }

    // Tests for handle_validate_tiles

    #[tokio::test]
    async fn test_handle_validate_tiles_valid_json() {
        let config = create_config_with_format(CudaTdgOutputFormat::Json);

        let result = handle_validate_tiles(64, 128, 49152, &config).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_handle_validate_tiles_valid_terminal() {
        let config = create_config_with_format(CudaTdgOutputFormat::Terminal);

        let result = handle_validate_tiles(64, 128, 49152, &config).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_handle_validate_tiles_invalid_tile_kv() {
        let config = create_config_with_format(CudaTdgOutputFormat::Terminal);

        // tile_kv < head_dim should fail
        let result = handle_validate_tiles(128, 64, 49152, &config).await;
        assert!(result.is_err());
        let err = result.unwrap_err().to_string();
        assert!(err.contains("PAR-041"));
    }

    #[tokio::test]
    async fn test_handle_validate_tiles_shared_memory_overflow() {
        let config = create_config_with_format(CudaTdgOutputFormat::Terminal);

        // Large dimensions with small shared memory: 256 * 128 * 2 = 65536 bytes
        // required against a 1024 byte limit.
        //
        // This used to assert `is_ok()` ("should succeed (tile_kv >= head_dim) but
        // show warning"). That encoded the defect: the printed verdict said
        // "Status: INVALID / Issue: Shared memory overflow" while the command still
        // exited 0, so CI gating on `validate-tiles` passed an overflowing
        // configuration. An INVALID verdict must exit nonzero.
        let result = handle_validate_tiles(128, 256, 1024, &config).await;
        assert!(
            result.is_err(),
            "a configuration overflowing shared memory must not exit 0"
        );
        let err = result.unwrap_err().to_string();
        assert!(
            err.contains("Shared memory overflow"),
            "unexpected error: {err}"
        );
    }

    // Tests for handle_taxonomy

    #[tokio::test]
    async fn test_handle_taxonomy_json() {
        let config = create_config_with_format(CudaTdgOutputFormat::Json);

        let result = handle_taxonomy(&config).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_handle_taxonomy_markdown() {
        let config = create_config_with_format(CudaTdgOutputFormat::Markdown);

        let result = handle_taxonomy(&config).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_handle_taxonomy_terminal() {
        let config = create_config_with_format(CudaTdgOutputFormat::Terminal);

        let result = handle_taxonomy(&config).await;
        assert!(result.is_ok());
    }

    // Tests for handle_kaizen

    #[tokio::test]
    async fn test_handle_kaizen_json() {
        let temp_dir = TempDir::new().unwrap();
        let config = create_config_with_format(CudaTdgOutputFormat::Json);

        let result = handle_kaizen(&temp_dir.path().to_path_buf(), None, &config).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_handle_kaizen_markdown() {
        let temp_dir = TempDir::new().unwrap();
        let config = create_config_with_format(CudaTdgOutputFormat::Markdown);

        let result = handle_kaizen(&temp_dir.path().to_path_buf(), None, &config).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_handle_kaizen_terminal() {
        let temp_dir = TempDir::new().unwrap();
        let config = create_config_with_format(CudaTdgOutputFormat::Terminal);

        let result = handle_kaizen(&temp_dir.path().to_path_buf(), None, &config).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_handle_kaizen_with_since() {
        let temp_dir = TempDir::new().unwrap();
        let config = create_config_with_format(CudaTdgOutputFormat::Terminal);

        let result =
            handle_kaizen(&temp_dir.path().to_path_buf(), Some("2025-01-01"), &config).await;
        assert!(result.is_ok());
    }

    // Tests for handle_gate

    #[tokio::test]
    async fn test_handle_gate_json() {
        let temp_dir = TempDir::new().unwrap();
        let config = create_config_with_format(CudaTdgOutputFormat::Json);

        // Empty directory should pass with high min_score allowing 0
        let result = handle_gate(&temp_dir.path().to_path_buf(), 0.0, false, &config).await;
        // May pass or fail depending on analyzer behavior with empty dirs
        let _ = result;
    }

    #[tokio::test]
    async fn test_handle_gate_terminal() {
        let temp_dir = TempDir::new().unwrap();
        let config = create_config_with_format(CudaTdgOutputFormat::Terminal);

        let result = handle_gate(&temp_dir.path().to_path_buf(), 0.0, false, &config).await;
        let _ = result;
    }

    // Tests for handle_barrier_check

    #[tokio::test]
    async fn test_handle_barrier_check_empty_dir() {
        let temp_dir = TempDir::new().unwrap();
        let config = create_config_with_format(CudaTdgOutputFormat::Terminal);

        let result = handle_barrier_check(&temp_dir.path().to_path_buf(), &config).await;
        // Empty dir should pass (no unsafe barriers)
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_handle_barrier_check_json() {
        let temp_dir = TempDir::new().unwrap();
        let config = create_config_with_format(CudaTdgOutputFormat::Json);

        let result = handle_barrier_check(&temp_dir.path().to_path_buf(), &config).await;
        assert!(result.is_ok());
    }

    // Tests for handle_report

    #[tokio::test]
    async fn test_handle_report_html() {
        let temp_dir = TempDir::new().unwrap();
        let config = create_config_with_format(CudaTdgOutputFormat::Terminal);

        let result = handle_report(&temp_dir.path().to_path_buf(), "html", None, &config).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_handle_report_json() {
        let temp_dir = TempDir::new().unwrap();
        let config = create_config_with_format(CudaTdgOutputFormat::Terminal);

        let result = handle_report(&temp_dir.path().to_path_buf(), "json", None, &config).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_handle_report_markdown() {
        let temp_dir = TempDir::new().unwrap();
        let config = create_config_with_format(CudaTdgOutputFormat::Terminal);

        let result = handle_report(&temp_dir.path().to_path_buf(), "markdown", None, &config).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_handle_report_to_file() {
        let temp_dir = TempDir::new().unwrap();
        let output_path = &temp_dir.path().to_path_buf().join("report.md");
        let config = create_config_with_format(CudaTdgOutputFormat::Terminal);

        let result = handle_report(
            &temp_dir.path().to_path_buf(),
            "markdown",
            Some(output_path),
            &config,
        )
        .await;
        assert!(result.is_ok());
        assert!(output_path.exists());
    }

    #[tokio::test]
    async fn test_handle_report_to_config_output() {
        let temp_dir = TempDir::new().unwrap();
        let output_path = &temp_dir.path().to_path_buf().join("config_report.md");
        let mut config = create_config_with_format(CudaTdgOutputFormat::Terminal);
        config.output = Some(output_path.clone());

        let result = handle_report(&temp_dir.path().to_path_buf(), "markdown", None, &config).await;
        assert!(result.is_ok());
        assert!(output_path.exists());
    }

    // Tests for handle_score

    #[tokio::test]
    async fn test_handle_score_summary() {
        let temp_dir = TempDir::new().unwrap();
        let config = create_config_with_format(CudaTdgOutputFormat::Terminal);

        let result = handle_score(&temp_dir.path().to_path_buf(), false, &config).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_handle_score_breakdown() {
        let temp_dir = TempDir::new().unwrap();
        let config = create_config_with_format(CudaTdgOutputFormat::Terminal);

        let result = handle_score(&temp_dir.path().to_path_buf(), true, &config).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_handle_score_json() {
        let temp_dir = TempDir::new().unwrap();
        let config = create_config_with_format(CudaTdgOutputFormat::Json);

        let result = handle_score(&temp_dir.path().to_path_buf(), false, &config).await;
        assert!(result.is_ok());
    }

    // Tests for handle_analyze

    #[tokio::test]
    async fn test_handle_analyze_empty_dir() {
        let temp_dir = TempDir::new().unwrap();
        let config = create_config_with_format(CudaTdgOutputFormat::Terminal);

        let result = handle_analyze(&temp_dir.path().to_path_buf(), &config).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_handle_analyze_json() {
        let temp_dir = TempDir::new().unwrap();
        let config = create_config_with_format(CudaTdgOutputFormat::Json);

        let result = handle_analyze(&temp_dir.path().to_path_buf(), &config).await;
        assert!(result.is_ok());
    }

    // Tests for handle_cuda_tdg_command (main handler)

    #[tokio::test]
    async fn test_handle_cuda_tdg_command_default() {
        let temp_dir = TempDir::new().unwrap();
        let config = CudaTdgCommandConfig {
            path: temp_dir.path().to_path_buf(),
            command: None,
            format: CudaTdgOutputFormat::Terminal,
            min_score: 0.0, // Low threshold to pass
            fail_on_p0: false,
            simd: true,
            wgpu: true,
            output: None,
            quiet: false,
        };

        let result = handle_cuda_tdg_command(config).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_handle_cuda_tdg_command_quality_gate_failure() {
        // Default report mode (command: None) should NOT hard-fail on score.
        // Quality gate enforcement is only in `pmat cuda-tdg gate` subcommand.
        let temp_dir = TempDir::new().unwrap();
        let config = CudaTdgCommandConfig {
            path: temp_dir.path().to_path_buf(),
            command: None,
            format: CudaTdgOutputFormat::Terminal,
            min_score: 100.0, // High threshold — but default mode doesn't enforce
            fail_on_p0: false,
            simd: true,
            wgpu: true,
            output: None,
            quiet: false,
        };

        let result = handle_cuda_tdg_command(config).await;
        assert!(result.is_ok(), "default report mode should not hard-fail");
    }

    // Tests for handle_cuda_tdg_subcommand

    #[tokio::test]
    async fn test_handle_cuda_tdg_subcommand_analyze() {
        let temp_dir = TempDir::new().unwrap();
        let config = create_config_with_format(CudaTdgOutputFormat::Terminal);
        let cmd = CudaTdgCommand::Analyze {
            path: temp_dir.path().to_path_buf(),
        };

        let result = handle_cuda_tdg_subcommand(&cmd, &config).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_handle_cuda_tdg_subcommand_score() {
        let temp_dir = TempDir::new().unwrap();
        let config = create_config_with_format(CudaTdgOutputFormat::Terminal);
        let cmd = CudaTdgCommand::Score {
            path: Some(temp_dir.path().to_path_buf()),
            breakdown: true,
        };

        let result = handle_cuda_tdg_subcommand(&cmd, &config).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_handle_cuda_tdg_subcommand_report() {
        let temp_dir = TempDir::new().unwrap();
        let config = create_config_with_format(CudaTdgOutputFormat::Terminal);
        let cmd = CudaTdgCommand::Report {
            path: Some(temp_dir.path().to_path_buf()),
            format: "markdown".to_string(),
            output: None,
        };

        let result = handle_cuda_tdg_subcommand(&cmd, &config).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_handle_cuda_tdg_subcommand_barrier_check() {
        let temp_dir = TempDir::new().unwrap();
        let config = create_config_with_format(CudaTdgOutputFormat::Terminal);
        let cmd = CudaTdgCommand::BarrierCheck {
            path: temp_dir.path().to_path_buf(),
        };

        let result = handle_cuda_tdg_subcommand(&cmd, &config).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_handle_cuda_tdg_subcommand_validate_tiles() {
        let config = create_config_with_format(CudaTdgOutputFormat::Terminal);
        let cmd = CudaTdgCommand::ValidateTiles {
            head_dim: 64,
            tile_kv: 128,
            shared_memory: 49152,
        };

        let result = handle_cuda_tdg_subcommand(&cmd, &config).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_handle_cuda_tdg_subcommand_gate() {
        let temp_dir = TempDir::new().unwrap();
        let config = create_config_with_format(CudaTdgOutputFormat::Terminal);
        let cmd = CudaTdgCommand::Gate {
            path: Some(temp_dir.path().to_path_buf()),
            min_score: 0.0,
            fail_on_p0: false,
        };

        let result = handle_cuda_tdg_subcommand(&cmd, &config).await;
        let _ = result; // May pass or fail
    }

    #[tokio::test]
    async fn test_handle_cuda_tdg_subcommand_kaizen() {
        let temp_dir = TempDir::new().unwrap();
        let config = create_config_with_format(CudaTdgOutputFormat::Terminal);
        let cmd = CudaTdgCommand::Kaizen {
            path: Some(temp_dir.path().to_path_buf()),
            since: Some("2025-01-01".to_string()),
        };

        let result = handle_cuda_tdg_subcommand(&cmd, &config).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_handle_cuda_tdg_subcommand_taxonomy() {
        let config = create_config_with_format(CudaTdgOutputFormat::Terminal);
        let cmd = CudaTdgCommand::Taxonomy;

        let result = handle_cuda_tdg_subcommand(&cmd, &config).await;
        assert!(result.is_ok());
    }

    // Tests with actual file content

    #[tokio::test]
    async fn test_analyze_cuda_file() {
        let temp_dir = TempDir::new().unwrap();
        let cuda_file = &temp_dir.path().to_path_buf().join("kernel.cu");
        let mut file = fs::File::create(cuda_file).unwrap();
        writeln!(
            file,
            r#"
__global__ void test_kernel(float* a, float* b, int n) {{
    int idx = blockIdx.x * blockDim.x + threadIdx.x;
    if (idx < n) {{
        a[idx] = b[idx] * 2.0f;
    }}
    __syncthreads();
}}
"#
        )
        .unwrap();

        let config = create_config_with_format(CudaTdgOutputFormat::Json);
        let result = handle_analyze(cuda_file, &config).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_analyze_ptx_file() {
        let temp_dir = TempDir::new().unwrap();
        let ptx_file = &temp_dir.path().to_path_buf().join("kernel.ptx");
        let mut file = fs::File::create(ptx_file).unwrap();
        writeln!(
            file,
            r#"
.version 7.0
.target sm_80
.address_size 64

.visible .entry test_kernel(
    .param .u64 param_a,
    .param .u64 param_b,
    .param .u32 param_n
) {{
    .reg .pred %p<2>;
    .reg .b32 %r<4>;
    .reg .b64 %rd<4>;

    mov.u32 %r0, %tid.x;
    mov.u32 %r1, %ctaid.x;
    mad.lo.s32 %r2, %r1, 256, %r0;

    bar.sync 0;

    ret;
}}
"#
        )
        .unwrap();

        let config = create_config_with_format(CudaTdgOutputFormat::Terminal);
        let result = handle_analyze(ptx_file, &config).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_analyze_simd_rust_file() {
        let temp_dir = TempDir::new().unwrap();
        let rs_file = &temp_dir.path().to_path_buf().join("simd.rs");
        let mut file = fs::File::create(rs_file).unwrap();
        writeln!(
            file,
            r#"
use std::arch::x86_64::*;

#[target_feature(enable = "avx2")]
unsafe fn add_avx2(a: &[f32], b: &[f32], c: &mut [f32]) {{
    let a_ptr = a.as_ptr();
    let b_ptr = b.as_ptr();
    let c_ptr = c.as_mut_ptr();

    let va = _mm256_loadu_ps(a_ptr);
    let vb = _mm256_loadu_ps(b_ptr);
    let vc = _mm256_add_ps(va, vb);
    _mm256_storeu_ps(c_ptr, vc);
}}
"#
        )
        .unwrap();

        let config = create_config_with_format(CudaTdgOutputFormat::Terminal);
        let result = handle_analyze(rs_file, &config).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_analyze_wgsl_file() {
        let temp_dir = TempDir::new().unwrap();
        let wgsl_file = &temp_dir.path().to_path_buf().join("compute.wgsl");
        let mut file = fs::File::create(wgsl_file).unwrap();
        writeln!(
            file,
            r#"
@group(0) @binding(0)
var<storage, read> input: array<f32>;

@group(0) @binding(1)
var<storage, read_write> output: array<f32>;

@compute @workgroup_size(256)
fn main(@builtin(global_invocation_id) gid: vec3<u32>) {{
    let idx = gid.x;
    if (idx < arrayLength(&input)) {{
        output[idx] = input[idx] * 2.0;
    }}
    workgroupBarrier();
}}
"#
        )
        .unwrap();

        let config = create_config_with_format(CudaTdgOutputFormat::Terminal);
        let result = handle_analyze(wgsl_file, &config).await;
        assert!(result.is_ok());
    }

    // Tests for KaizenMetrics output with ticket references

    #[tokio::test]
    async fn test_kaizen_markdown_with_ticket_references() {
        let temp_dir = TempDir::new().unwrap();

        // Create a file with ticket references
        let test_file = &temp_dir.path().to_path_buf().join("test_regression.rs");
        let mut file = fs::File::create(test_file).unwrap();
        writeln!(
            file,
            r#"
// Test for PARITY-114 barrier divergence fix
#[test]
fn test_barrier_convergence() {{
    // Regression test for PAR-041
}}
"#
        )
        .unwrap();

        let config = create_config_with_format(CudaTdgOutputFormat::Markdown);
        let result = handle_kaizen(&temp_dir.path().to_path_buf(), None, &config).await;
        assert!(result.is_ok());
    }

    // Edge case tests

    #[test]
    fn test_popper_score_gateway_rule() {
        // Gateway passed
        let score = create_mock_popper_score(85.0, true);
        assert_eq!(score.total, 85.0);
        assert!(score.gateway_passed);

        // Gateway failed - total should be as calculated but grade shows fail
        let score = create_mock_popper_score(0.0, false);
        assert_eq!(score.total, 0.0);
        assert!(!score.gateway_passed);
        assert_eq!(score.grade, CudaTdgGrade::GatewayFail);
    }

    #[test]
    fn test_cuda_tdg_grade_display() {
        assert_eq!(CudaTdgGrade::APlus.to_string(), "A+");
        assert_eq!(CudaTdgGrade::A.to_string(), "A");
        assert_eq!(CudaTdgGrade::B.to_string(), "B");
        assert_eq!(CudaTdgGrade::C.to_string(), "C");
        assert_eq!(CudaTdgGrade::D.to_string(), "D");
        assert_eq!(CudaTdgGrade::F.to_string(), "F");
        assert_eq!(CudaTdgGrade::GatewayFail.to_string(), "FAIL (Gateway)");
    }

    #[test]
    fn test_cuda_tdg_grade_from_score() {
        assert_eq!(CudaTdgGrade::from_score(95.0, true), CudaTdgGrade::APlus);
        assert_eq!(CudaTdgGrade::from_score(85.0, true), CudaTdgGrade::A);
        assert_eq!(CudaTdgGrade::from_score(75.0, true), CudaTdgGrade::B);
        assert_eq!(CudaTdgGrade::from_score(65.0, true), CudaTdgGrade::C);
        assert_eq!(CudaTdgGrade::from_score(55.0, true), CudaTdgGrade::D);
        assert_eq!(CudaTdgGrade::from_score(45.0, true), CudaTdgGrade::F);
        assert_eq!(
            CudaTdgGrade::from_score(85.0, false),
            CudaTdgGrade::GatewayFail
        );
    }

    #[test]
    fn test_defect_severity_display() {
        assert_eq!(DefectSeverity::P0Critical.to_string(), "P0-Critical");
        assert_eq!(DefectSeverity::P1Performance.to_string(), "P1-Performance");
        assert_eq!(DefectSeverity::P2Efficiency.to_string(), "P2-Efficiency");
        assert_eq!(DefectSeverity::P3Minor.to_string(), "P3-Minor");
    }

    #[test]
    fn test_defect_taxonomy_patterns() {
        let taxonomy = DefectTaxonomy::with_tauranta_patterns();

        // Check P0 defects
        let parity_114 = taxonomy.get("PARITY-114").unwrap();
        assert_eq!(parity_114.severity, DefectSeverity::P0Critical);

        let par_041 = taxonomy.get("PAR-041").unwrap();
        assert_eq!(par_041.severity, DefectSeverity::P0Critical);

        // Check P1 defects
        let par_001 = taxonomy.get("PAR-001").unwrap();
        assert_eq!(par_001.severity, DefectSeverity::P1Performance);

        // Check P2 defects
        let par_005 = taxonomy.get("PAR-005").unwrap();
        assert_eq!(par_005.severity, DefectSeverity::P2Efficiency);

        // Check all() returns all patterns
        let all_count = taxonomy.all().count();
        assert!(all_count >= 10); // Should have at least 10 patterns
    }

    #[test]
    fn test_falsifiability_score_total() {
        let score = FalsifiabilityScore {
            barrier_safety: 5.0,
            bounds_verification: 4.0,
            divergence_testing: 3.0,
            memory_race_detection: 5.0,
            occupancy_bounds: 4.0,
        };
        assert_eq!(score.total(), 21.0);
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
    fn test_popper_score_calculate() {
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
        let gpu_simd_specific = GpuSimdSpecificScore {
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
            gpu_simd_specific,
        );

        assert_eq!(score.total, 100.0);
        assert!(score.gateway_passed);
        assert_eq!(score.grade, CudaTdgGrade::APlus);
    }

    #[test]
    fn test_popper_score_gateway_fail() {
        let falsifiability = FalsifiabilityScore {
            barrier_safety: 2.0,
            bounds_verification: 2.0,
            divergence_testing: 2.0,
            memory_race_detection: 2.0,
            occupancy_bounds: 2.0,
        }; // Total: 10.0 < 15.0 threshold

        let score = PopperScore::calculate(
            falsifiability,
            ReproducibilityScore::default(),
            TransparencyScore::default(),
            StatisticalRigorScore::default(),
            HistoricalIntegrityScore::default(),
            GpuSimdSpecificScore::default(),
        );

        assert_eq!(score.total, 0.0);
        assert!(!score.gateway_passed);
        assert_eq!(score.grade, CudaTdgGrade::GatewayFail);
    }

    #[test]
    fn test_cuda_simd_config_default() {
        let config = CudaSimdConfig::new();
        assert_eq!(config.min_score, 85.0);
        assert!(config.fail_on_p0);
        assert!(config.analyze_simd);
        assert!(config.analyze_wgpu);
        assert_eq!(config.shared_memory_limit, 49152);
        assert_eq!(config.register_limit, 64);
    }
}

/// `pmat cuda-tdg [PATH] <SUBCOMMAND>` used to discard the top-level positional:
/// each subcommand re-defaulted its own path to `"."`, so
/// `pmat cuda-tdg /does/not/exist score` graded the current directory (81.0/100,
/// "Gateway: PASSED", exit 0) instead of the path it was given.
mod top_level_path_is_honoured_tests {
    use super::*;
    use tempfile::TempDir;

    fn config_for(path: PathBuf) -> CudaTdgCommandConfig {
        CudaTdgCommandConfig {
            path,
            command: None,
            format: CudaTdgOutputFormat::Terminal,
            min_score: 85.0,
            fail_on_p0: false,
            simd: true,
            wgpu: true,
            output: None,
            quiet: true,
        }
    }

    #[test]
    fn subcommand_without_a_path_falls_back_to_the_top_level_path() {
        let config = config_for(PathBuf::from("/top/level"));
        assert_eq!(
            resolve_subcommand_path(None, &config),
            &PathBuf::from("/top/level")
        );
    }

    #[test]
    fn an_explicit_subcommand_path_still_wins() {
        let config = config_for(PathBuf::from("/top/level"));
        let explicit = PathBuf::from("/explicit");
        assert_eq!(resolve_subcommand_path(Some(&explicit), &config), &explicit);
    }

    #[tokio::test]
    async fn score_over_a_missing_top_level_path_fails_instead_of_grading_the_cwd() {
        let dir = TempDir::new().expect("tempdir");
        let missing = dir.path().join("no-such-tree");
        let config = config_for(missing.clone());

        let err = handle_cuda_tdg_subcommand(
            &CudaTdgCommand::Score {
                path: None,
                breakdown: false,
            },
            &config,
        )
        .await
        .expect_err("a missing path must not be scored as the current directory");

        let rendered = format!("{err:#}");
        assert!(
            rendered.contains(&missing.display().to_string()),
            "the error must name the path that was given, got: {rendered}"
        );
    }
}

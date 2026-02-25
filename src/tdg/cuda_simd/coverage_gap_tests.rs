mod coverage_gap_tests {
    use super::*;
    use std::path::PathBuf;

    fn analyzer() -> CudaSimdAnalyzer {
        CudaSimdAnalyzer::new()
    }

    fn analyze_ptx(content: &str) -> FileAnalysis {
        let a = analyzer();
        let path = PathBuf::from("test.ptx");
        let mut analysis = FileAnalysis::default();
        a.detect_ptx_memory_patterns(content, &path, &mut analysis);
        analysis
    }

    fn analyze_simd(content: &str) -> FileAnalysis {
        let a = analyzer();
        let path = PathBuf::from("test.rs");
        let mut analysis = FileAnalysis::default();
        a.analyze_simd_content(content, &path, &mut analysis);
        analysis
    }

    fn analyze_wgpu(content: &str) -> FileAnalysis {
        let a = analyzer();
        let path = PathBuf::from("test.wgsl");
        let mut analysis = FileAnalysis::default();
        a.detect_wgpu_memory_patterns(content, &path, &mut analysis);
        analysis
    }

    fn has_defect(analysis: &FileAnalysis, ticket_id: &str) -> bool {
        analysis
            .defects
            .iter()
            .any(|d| d.defect_class.ticket_id == ticket_id)
    }

    fn defect_count(analysis: &FileAnalysis, ticket_id: &str) -> usize {
        analysis
            .defects
            .iter()
            .filter(|d| d.defect_class.ticket_id == ticket_id)
            .count()
    }

    fn analyze_cuda_barriers(content: &str) -> FileAnalysis {
        let a = analyzer();
        let path = PathBuf::from("test.cu");
        let mut analysis = FileAnalysis::default();
        a.detect_barrier_issues(content, &path, &mut analysis);
        analysis
    }

    fn analyze_cuda_known_patterns(content: &str) -> FileAnalysis {
        let a = analyzer();
        let path = PathBuf::from("test.cu");
        let mut analysis = FileAnalysis::default();
        a.detect_known_patterns(content, &path, &mut analysis);
        analysis
    }

    include!("coverage_gap_ptx_tests.rs");
    include!("coverage_gap_ptx_register_tests.rs");
    include!("coverage_gap_simd_tests.rs");
    include!("coverage_gap_wgpu_tests.rs");
    include!("coverage_gap_integration_tests.rs");
    include!("coverage_gap_barrier_tests.rs");
    include!("coverage_gap_known_pattern_tests.rs");
}

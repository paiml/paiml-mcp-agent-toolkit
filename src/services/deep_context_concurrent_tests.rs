// Tests for ConcurrentDeepContextAnalyzer
// Included from deep_context_concurrent.rs - do NOT add `use` imports or `#!` attributes here

#[cfg_attr(coverage_nightly, coverage(off))]
#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    // =========================================================================
    // AstCache Tests
    // =========================================================================

    #[test]
    fn test_ast_cache_new() {
        let cache = AstCache::new();
        assert!(cache.files().is_empty());
    }

    #[test]
    fn test_ast_cache_insert() {
        let mut cache = AstCache::new();
        let path = PathBuf::from("/test/file.rs");
        let ast = ParsedAst::default();

        cache.insert(path.clone(), ast);

        assert_eq!(cache.files().len(), 1);
        assert!(cache.files().contains_key(&path));
    }

    #[test]
    fn test_ast_cache_insert_multiple() {
        let mut cache = AstCache::new();

        cache.insert(PathBuf::from("/test/file1.rs"), ParsedAst::default());
        cache.insert(PathBuf::from("/test/file2.rs"), ParsedAst::default());
        cache.insert(PathBuf::from("/test/file3.rs"), ParsedAst::default());

        assert_eq!(cache.files().len(), 3);
    }

    #[test]
    fn test_ast_cache_insert_overwrites() {
        let mut cache = AstCache::new();
        let path = PathBuf::from("/test/file.rs");

        cache.insert(path.clone(), ParsedAst::default());
        cache.insert(path.clone(), ParsedAst::default());

        // Should still only have 1 entry
        assert_eq!(cache.files().len(), 1);
    }

    // =========================================================================
    // ParsedAst Tests
    // =========================================================================

    #[test]
    fn test_parsed_ast_default() {
        let ast = ParsedAst::default();
        // Just verify it can be created
        let _ = ast;
    }

    // =========================================================================
    // ComplexityResult Tests
    // =========================================================================

    #[test]
    fn test_complexity_result_default() {
        let result = ComplexityResult::default();
        let _ = result;
    }

    // =========================================================================
    // ComplexityResults Tests
    // =========================================================================

    #[test]
    fn test_complexity_results_default() {
        let results = ComplexityResults::default();
        let _ = results;
    }

    #[test]
    fn test_complexity_results_combine_empty() {
        let results = ComplexityResults::combine(vec![]);
        let _ = results;
    }

    #[test]
    fn test_complexity_results_combine_single() {
        let results = ComplexityResults::combine(vec![ComplexityResult::default()]);
        let _ = results;
    }

    #[test]
    fn test_complexity_results_combine_multiple() {
        let results = ComplexityResults::combine(vec![
            ComplexityResult::default(),
            ComplexityResult::default(),
            ComplexityResult::default(),
        ]);
        let _ = results;
    }

    // =========================================================================
    // BigOResults Tests
    // =========================================================================

    #[test]
    fn test_big_o_results_default() {
        let results = BigOResults::default();
        let _ = results;
    }

    // =========================================================================
    // TDGResults Tests
    // =========================================================================

    #[test]
    fn test_tdg_results_default() {
        let results = TDGResults::default();
        let _ = results;
    }

    // =========================================================================
    // DeadCodeResults Tests
    // =========================================================================

    #[test]
    fn test_dead_code_results_default() {
        let results = DeadCodeResults::default();
        let _ = results;
    }

    // =========================================================================
    // ConcurrentDeepContextAnalyzer Tests
    // =========================================================================

    #[test]
    fn test_concurrent_analyzer_new() {
        let config = DeepContextConfig::default();
        let analyzer = ConcurrentDeepContextAnalyzer::new(config);
        // Verify it was created
        let _ = analyzer;
    }

    #[test]
    fn test_concurrent_analyzer_progress_bar() {
        let config = DeepContextConfig::default();
        let analyzer = ConcurrentDeepContextAnalyzer::new(config);

        let pb = analyzer.create_progress_bar("Test", 100);
        assert_eq!(pb.length(), Some(100));
    }

    #[test]
    fn test_concurrent_analyzer_parse_single_file() {
        let config = DeepContextConfig::default();
        let analyzer = ConcurrentDeepContextAnalyzer::new(config);

        let result = analyzer.parse_single_file(Path::new("/test/file.rs"));
        assert!(result.is_ok());
    }

    #[test]
    fn test_concurrent_analyzer_calculate_complexity_for_ast() {
        let config = DeepContextConfig::default();
        let analyzer = ConcurrentDeepContextAnalyzer::new(config);
        let ast = ParsedAst::default();

        let result = analyzer.calculate_complexity_for_ast(&ast);
        let _ = result;
    }

    #[test]
    fn test_concurrent_analyzer_extract_functions_from_ast() {
        let config = DeepContextConfig::default();
        let analyzer = ConcurrentDeepContextAnalyzer::new(config);
        let ast = ParsedAst::default();

        let functions = analyzer.extract_functions_from_ast(&ast);
        assert!(functions.is_empty());
    }

    // =========================================================================
    // ANALYSIS_COUNT Constant Test
    // =========================================================================

    #[test]
    fn test_analysis_count_constant() {
        // Should have 8 parallel analyses
        assert_eq!(ANALYSIS_COUNT, 8);
    }

    // =========================================================================
    // Async Tests
    // =========================================================================

    #[tokio::test]
    async fn test_analyze_tdg_cached() {
        let config = DeepContextConfig::default();
        let analyzer = ConcurrentDeepContextAnalyzer::new(config);
        let cache = Arc::new(AstCache::new());

        let result = analyzer.analyze_tdg_cached(&cache).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_analyze_big_o_cached() {
        let config = DeepContextConfig::default();
        let analyzer = ConcurrentDeepContextAnalyzer::new(config);
        let cache = Arc::new(AstCache::new());

        let result = analyzer.analyze_big_o_cached(&cache).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_analyze_dead_code_cached() {
        let config = DeepContextConfig::default();
        let analyzer = ConcurrentDeepContextAnalyzer::new(config);
        let cache = Arc::new(AstCache::new());

        let result = analyzer.analyze_dead_code_cached(&cache).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_analyze_complexity_cached_empty() {
        let config = DeepContextConfig::default();
        let analyzer = ConcurrentDeepContextAnalyzer::new(config);
        let cache = Arc::new(AstCache::new());

        let result = analyzer.analyze_complexity_cached(&cache).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_analyze_complexity_cached_with_files() {
        let config = DeepContextConfig::default();
        let analyzer = ConcurrentDeepContextAnalyzer::new(config);

        let mut cache = AstCache::new();
        cache.insert(PathBuf::from("/test/file.rs"), ParsedAst::default());
        let cache = Arc::new(cache);

        let result = analyzer.analyze_complexity_cached(&cache).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_generate_output() {
        let config = DeepContextConfig::default();
        let analyzer = ConcurrentDeepContextAnalyzer::new(config);

        let analyses = CombinedAnalyses {
            complexity: ComplexityResults::default(),
            provability: vec![],
            satd: SATDAnalysisResult::default(),
            churn: ChurnAnalysis::default(),
            dag: DependencyGraph::default(),
            tdg: TDGResults::default(),
            big_o: BigOResults::default(),
            dead_code: DeadCodeResults::default(),
        };

        let result = analyzer.generate_output(analyses).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_deep_analysis_result_has_timestamp() {
        let config = DeepContextConfig::default();
        let analyzer = ConcurrentDeepContextAnalyzer::new(config);

        let analyses = CombinedAnalyses {
            complexity: ComplexityResults::default(),
            provability: vec![],
            satd: SATDAnalysisResult::default(),
            churn: ChurnAnalysis::default(),
            dag: DependencyGraph::default(),
            tdg: TDGResults::default(),
            big_o: BigOResults::default(),
            dead_code: DeadCodeResults::default(),
        };

        let result = analyzer.generate_output(analyses).await.unwrap();

        // Verify timestamp is recent (within last minute)
        let elapsed = result.timestamp.elapsed().unwrap();
        assert!(elapsed.as_secs() < 60);
    }

    // =========================================================================
    // CombinedAnalyses Tests
    // =========================================================================

    #[test]
    fn test_combined_analyses_creation() {
        let analyses = CombinedAnalyses {
            complexity: ComplexityResults::default(),
            provability: vec![],
            satd: SATDAnalysisResult::default(),
            churn: ChurnAnalysis::default(),
            dag: DependencyGraph::default(),
            tdg: TDGResults::default(),
            big_o: BigOResults::default(),
            dead_code: DeadCodeResults::default(),
        };

        // Verify all fields can be accessed
        let _ = analyses.complexity;
        let _ = analyses.provability;
        let _ = analyses.satd;
        let _ = analyses.churn;
        let _ = analyses.dag;
        let _ = analyses.tdg;
        let _ = analyses.big_o;
        let _ = analyses.dead_code;
    }

    // =========================================================================
    // DeepAnalysisResult Tests
    // =========================================================================

    #[test]
    fn test_deep_analysis_result_creation() {
        let analyses = CombinedAnalyses {
            complexity: ComplexityResults::default(),
            provability: vec![],
            satd: SATDAnalysisResult::default(),
            churn: ChurnAnalysis::default(),
            dag: DependencyGraph::default(),
            tdg: TDGResults::default(),
            big_o: BigOResults::default(),
            dead_code: DeadCodeResults::default(),
        };

        let result = DeepAnalysisResult {
            analyses,
            timestamp: std::time::SystemTime::now(),
        };

        // Verify timestamp can be accessed
        let _ = result.timestamp;
    }
}

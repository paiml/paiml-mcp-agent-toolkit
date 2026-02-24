    struct MockRanker;

    impl FileRanker for MockRanker {
        type Metric = f64;

        fn compute_score(&self, file_path: &Path) -> Self::Metric {
            // Mock score based on file name length
            file_path.to_string_lossy().len() as f64
        }

        fn format_ranking_entry(&self, file: &str, metric: &Self::Metric, rank: usize) -> String {
            format!("| {rank:>4} | {file} | {metric:.1} |")
        }

        fn ranking_type(&self) -> &'static str {
            "Mock"
        }
    }

    fn create_test_file_metrics() -> FileComplexityMetrics {
        FileComplexityMetrics {
            path: "test.rs".to_string(),
            total_complexity: ComplexityMetrics::new(23, 37, 4, 50),
            functions: vec![
                FunctionComplexity {
                    name: "test_func".to_string(),
                    line_start: 1,
                    line_end: 10,
                    metrics: ComplexityMetrics::new(5, 8, 2, 10),
                },
                FunctionComplexity {
                    name: "complex_func".to_string(),
                    line_start: 20,
                    line_end: 50,
                    metrics: ComplexityMetrics::new(15, 25, 4, 30),
                },
            ],
            classes: vec![ClassComplexity {
                name: "TestClass".to_string(),
                line_start: 60,
                line_end: 100,
                metrics: ComplexityMetrics::new(3, 4, 1, 10),
                methods: vec![FunctionComplexity {
                    name: "method".to_string(),
                    line_start: 65,
                    line_end: 75,
                    metrics: ComplexityMetrics::new(3, 4, 1, 10),
                }],
            }],
        }
    }

    #[tokio::test]
    async fn test_empty_file_list() {
        let ranker = MockRanker;
        let engine = RankingEngine::new(ranker);
        let result = engine.rank_files(&[], 5).await;
        assert_eq!(result.len(), 0);
    }

    #[tokio::test]
    async fn test_limit_exceeds_files() {
        let files = vec![PathBuf::from("a.rs"), PathBuf::from("b.rs")];
        let ranker = MockRanker;
        let engine = RankingEngine::new(ranker);

        // This will filter out non-existent files, so result will be empty
        let result = engine.rank_files(&files, 10).await;
        assert_eq!(result.len(), 0); // Files don't exist
    }

    #[test]
    fn test_vectorized_ranking() {
        let scores = vec![1.0, 5.0, 3.0, 2.0, 4.0];
        let ranked = rank_files_vectorized(&scores, 3);

        // Should be sorted by score descending: [1]=5.0, [4]=4.0, [2]=3.0
        assert_eq!(ranked, vec![1, 4, 2]);
    }

    #[test]
    fn test_composite_complexity_score_ordering() {
        let score1 = CompositeComplexityScore {
            total_score: 10.0,
            ..Default::default()
        };
        let score2 = CompositeComplexityScore {
            total_score: 5.0,
            ..Default::default()
        };

        assert!(score1 > score2);
    }

    #[test]
    fn test_composite_complexity_score_default() {
        let score = CompositeComplexityScore::default();
        assert_eq!(score.cyclomatic_max, 0);
        assert_eq!(score.cognitive_avg, 0.0);
        assert_eq!(score.halstead_effort, 0.0);
        assert_eq!(score.function_count, 0);
        assert_eq!(score.total_score, 0.0);
    }

    #[test]
    fn test_composite_complexity_score_equality() {
        let score1 = CompositeComplexityScore {
            total_score: 10.0,
            ..Default::default()
        };
        let score2 = CompositeComplexityScore {
            total_score: 10.0,
            ..Default::default()
        };
        let score3 = CompositeComplexityScore {
            total_score: 15.0,
            ..Default::default()
        };

        assert_eq!(score1, score2);
        assert_ne!(score1, score3);
    }

    #[test]
    fn test_churn_score_default_and_ordering() {
        let score1 = ChurnScore::default();
        let score2 = ChurnScore {
            score: 10.0,
            ..Default::default()
        };

        assert_eq!(score1.commit_count, 0);
        assert_eq!(score1.score, 0.0);
        assert!(score2 > score1);
    }

    #[test]
    fn test_duplication_score_default_and_ordering() {
        let score1 = DuplicationScore::default();
        let score2 = DuplicationScore {
            score: 5.0,
            exact_clones: 2,
            duplication_ratio: 0.3,
            ..Default::default()
        };

        assert_eq!(score1.exact_clones, 0);
        assert_eq!(score1.duplication_ratio, 0.0);
        assert!(score2 > score1);
    }

    #[test]
    fn test_vectorized_ranking_small_dataset() {
        let scores = vec![3.0, 1.0, 4.0, 2.0];
        let ranked = rank_files_vectorized(&scores, 2);
        assert_eq!(ranked, vec![2, 0]); // indices of highest scores
    }

    #[test]
    fn test_vectorized_ranking_large_dataset() {
        let scores: Vec<f32> = (0..2000).map(|i| i as f32).collect();
        let ranked = rank_files_vectorized(&scores, 5);
        assert_eq!(ranked, vec![1999, 1998, 1997, 1996, 1995]);
    }

    #[test]
    fn test_vectorized_ranking_empty() {
        let scores = vec![];
        let ranked = rank_files_vectorized(&scores, 5);
        assert_eq!(ranked.len(), 0);
    }

    #[test]
    fn test_complexity_ranker_default() {
        let ranker = ComplexityRanker::default();
        assert_eq!(ranker.cyclomatic_weight, 0.4);
        assert_eq!(ranker.cognitive_weight, 0.4);
        assert_eq!(ranker.function_count_weight, 0.2);
        assert_eq!(ranker.ranking_type(), "Complexity");
    }

    #[test]
    fn test_complexity_ranker_new() {
        let ranker = ComplexityRanker::new(0.5, 0.3, 0.2);
        assert_eq!(ranker.cyclomatic_weight, 0.5);
        assert_eq!(ranker.cognitive_weight, 0.3);
        assert_eq!(ranker.function_count_weight, 0.2);
    }

    #[test]
    fn test_complexity_ranker_calculate_composite_score() {
        let ranker = ComplexityRanker::default();
        let metrics = create_test_file_metrics();
        let score = ranker.calculate_composite_score(&metrics);

        assert_eq!(score.function_count, 3); // 2 functions + 1 method
        assert_eq!(score.cyclomatic_max, 15); // max from complex_func
        assert!((score.cognitive_avg - 12.333333333333334).abs() < 0.001); // (8+25+4)/3
        assert!(score.total_score > 0.0);
    }

    #[test]
    fn test_complexity_ranker_calculate_composite_score_empty() {
        let ranker = ComplexityRanker::default();
        let metrics = FileComplexityMetrics {
            path: "empty.rs".to_string(),
            total_complexity: ComplexityMetrics::default(),
            functions: vec![],
            classes: vec![],
        };
        let score = ranker.calculate_composite_score(&metrics);

        assert_eq!(score, CompositeComplexityScore::default());
    }

    #[tokio::test]
    async fn test_ranking_engine_with_temp_files() {
        let temp_dir = TempDir::new().unwrap();

        // Create test files
        let file1 = temp_dir.path().join("small.rs");
        let file2 = temp_dir.path().join("large.rs");

        {
            use std::io::BufWriter;
            let f1 = File::create(&file1).unwrap();
            let mut writer = BufWriter::new(f1);
            writeln!(writer, "fn small() {{}}").unwrap();
        }

        {
            use std::io::BufWriter;
            let f2 = File::create(&file2).unwrap();
            let mut writer = BufWriter::new(f2);
            writeln!(writer, "fn large() {{ // This is a much longer file").unwrap();
            for _ in 0..100 {
                writeln!(writer, "    println!(\"line\");").unwrap();
            }
            writeln!(writer, "}}").unwrap();
        }

        let ranker = ComplexityRanker::default();
        let engine = RankingEngine::new(ranker);

        let files = vec![file1, file2];
        let rankings = engine.rank_files(&files, 2).await;

        assert_eq!(rankings.len(), 2);
        // Larger file should have higher score
        assert!(rankings[0].1.total_score >= rankings[1].1.total_score);
    }

    #[tokio::test]
    async fn test_ranking_engine_zero_limit() {
        let ranker = ComplexityRanker::default();
        let engine = RankingEngine::new(ranker);
        let files = vec![PathBuf::from("test.rs")];
        let rankings = engine.rank_files(&files, 0).await;
        assert_eq!(rankings.len(), 0);
    }

    #[tokio::test]
    async fn test_ranking_engine_cache() {
        let temp_dir = TempDir::new().unwrap();
        let file1 = temp_dir.path().join("test.rs");
        let mut f1 = File::create(&file1).unwrap();
        writeln!(f1, "fn test() {{}}").unwrap();

        let ranker = ComplexityRanker::default();
        let engine = RankingEngine::new(ranker);

        let files = vec![file1.clone()];

        // First call should compute and cache
        let rankings1 = engine.rank_files(&files, 1).await;

        // Second call should use cache
        let rankings2 = engine.rank_files(&files, 1).await;

        assert_eq!(rankings1.len(), 1);
        assert_eq!(rankings2.len(), 1);
        assert_eq!(rankings1[0].1.total_score, rankings2[0].1.total_score);

        // Clear cache and verify
        engine.clear_cache();
        let rankings3 = engine.rank_files(&files, 1).await;
        assert_eq!(rankings3.len(), 1);
    }

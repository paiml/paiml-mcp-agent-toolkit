    #[test]
    fn test_ranking_engine_format_rankings_table_empty() {
        let ranker = ComplexityRanker::default();
        let engine = RankingEngine::new(ranker);
        let rankings = vec![];
        let output = engine.format_rankings_table(&rankings);
        assert!(output.contains("No files found"));
    }

    #[test]
    fn test_ranking_engine_format_rankings_table() {
        let ranker = ComplexityRanker::default();
        let engine = RankingEngine::new(ranker);
        let rankings = vec![
            (
                "test1.rs".to_string(),
                CompositeComplexityScore {
                    total_score: 10.0,
                    function_count: 5,
                    cyclomatic_max: 8,
                    cognitive_avg: 12.0,
                    halstead_effort: 150.0,
                },
            ),
            (
                "test2.rs".to_string(),
                CompositeComplexityScore {
                    total_score: 5.0,
                    function_count: 2,
                    cyclomatic_max: 3,
                    cognitive_avg: 4.0,
                    halstead_effort: 50.0,
                },
            ),
        ];
        let output = engine.format_rankings_table(&rankings);
        assert!(output.contains("Top 2 Complexity Files"));
        assert!(output.contains("test1.rs"));
        assert!(output.contains("test2.rs"));
        assert!(output.contains("10.0"));
        assert!(output.contains("5.0"));
    }

    #[test]
    fn test_ranking_engine_format_rankings_json() {
        let ranker = ComplexityRanker::default();
        let engine = RankingEngine::new(ranker);
        let rankings = vec![(
            "test1.rs".to_string(),
            CompositeComplexityScore {
                total_score: 10.0,
                ..Default::default()
            },
        )];
        let json = engine.format_rankings_json(&rankings);
        assert_eq!(json["analysis_type"], "Complexity");
        assert_eq!(json["top_files"]["requested"], 1);
        assert_eq!(json["rankings"][0]["rank"], 1);
        assert_eq!(json["rankings"][0]["file"], "test1.rs");
    }

    #[test]
    fn test_complexity_ranker_compute_score_rust_file() {
        let temp_dir = TempDir::new().unwrap();
        let rust_file = temp_dir.path().join("test.rs");
        let mut f = File::create(&rust_file).unwrap();
        writeln!(f, "fn test() {{ println!(\"hello\"); }}").unwrap();

        let ranker = ComplexityRanker::default();
        let score = ranker.compute_score(&rust_file);
        assert!(score.total_score > 0.0);
        // Note: compute_score uses simplified file-size-based scoring, not actual AST parsing
        // function_count is a usize and is always >= 0
    }

    #[test]
    fn test_complexity_ranker_compute_score_javascript_file() {
        let temp_dir = TempDir::new().unwrap();
        let js_file = temp_dir.path().join("test.js");
        let mut f = File::create(&js_file).unwrap();
        writeln!(f, "function test() {{ console.log('hello'); }}").unwrap();

        let ranker = ComplexityRanker::default();
        let score = ranker.compute_score(&js_file);
        assert!(score.total_score > 0.0);
    }

    #[test]
    fn test_complexity_ranker_compute_score_python_file() {
        let temp_dir = TempDir::new().unwrap();
        let py_file = temp_dir.path().join("test.py");
        let mut f = File::create(&py_file).unwrap();
        writeln!(f, "def test():\n    print('hello')").unwrap();

        let ranker = ComplexityRanker::default();
        let score = ranker.compute_score(&py_file);
        assert!(score.total_score > 0.0);
    }

    #[test]
    fn test_complexity_ranker_compute_score_unknown_file() {
        let temp_dir = TempDir::new().unwrap();
        let unknown_file = temp_dir.path().join("test.txt");
        let mut f = File::create(&unknown_file).unwrap();
        writeln!(f, "hello world").unwrap();

        let ranker = ComplexityRanker::default();
        let score = ranker.compute_score(&unknown_file);
        assert_eq!(score, CompositeComplexityScore::default());
    }

    #[test]
    fn test_complexity_ranker_compute_score_nonexistent_file() {
        let ranker = ComplexityRanker::default();
        let score = ranker.compute_score(Path::new("/nonexistent/file.rs"));
        assert_eq!(score, CompositeComplexityScore::default());
    }

    #[test]
    fn test_complexity_ranker_format_ranking_entry() {
        let ranker = ComplexityRanker::default();
        let metric = CompositeComplexityScore {
            total_score: 42.5,
            function_count: 10,
            cyclomatic_max: 15,
            cognitive_avg: 8.7,
            halstead_effort: 123.4,
        };
        let output = ranker.format_ranking_entry("test.rs", &metric, 1);
        assert!(output.contains("1"));
        assert!(output.contains("test.rs"));
        assert!(output.contains("42.5"));
        assert!(output.contains("10"));
        assert!(output.contains("15"));
        assert!(output.contains("8.7"));
        assert!(output.contains("123.4"));
    }

    #[test]
    fn test_rank_files_by_complexity() {
        let metrics = vec![
            FileComplexityMetrics {
                path: "simple.rs".to_string(),
                total_complexity: ComplexityMetrics::new(1, 1, 0, 5),
                functions: vec![FunctionComplexity {
                    name: "simple".to_string(),
                    line_start: 1,
                    line_end: 5,
                    metrics: ComplexityMetrics::new(1, 1, 0, 5),
                }],
                classes: vec![],
            },
            create_test_file_metrics(), // More complex file
        ];

        let ranker = ComplexityRanker::default();
        let rankings = rank_files_by_complexity(&metrics, 2, &ranker);

        assert_eq!(rankings.len(), 2);
        // More complex file should be ranked first
        assert_eq!(rankings[0].0, "test.rs");
        assert_eq!(rankings[1].0, "simple.rs");
        assert!(rankings[0].1.total_score > rankings[1].1.total_score);
    }

    #[test]
    fn test_rank_files_by_complexity_with_limit() {
        let metrics = vec![create_test_file_metrics()];
        let ranker = ComplexityRanker::default();
        let rankings = rank_files_by_complexity(&metrics, 0, &ranker);
        assert_eq!(rankings.len(), 1); // limit 0 means no truncation

        let rankings_limited = rank_files_by_complexity(&metrics, 1, &ranker);
        assert_eq!(rankings_limited.len(), 1);
    }

    #[test]
    fn test_rank_files_by_complexity_empty() {
        let metrics = vec![];
        let ranker = ComplexityRanker::default();
        let rankings = rank_files_by_complexity(&metrics, 5, &ranker);
        assert_eq!(rankings.len(), 0);
    }

    #[tokio::test]
    async fn test_ranking_engine_with_nonexistent_files() {
        let ranker = ComplexityRanker::default();
        let engine = RankingEngine::new(ranker);

        let files = vec![
            PathBuf::from("/nonexistent/file1.rs"),
            PathBuf::from("/nonexistent/file2.rs"),
        ];

        let rankings = engine.rank_files(&files, 5).await;
        assert_eq!(rankings.len(), 0); // All files filtered out
    }

    #[tokio::test]
    async fn test_ranking_engine_mixed_existing_nonexistent() {
        let temp_dir = TempDir::new().unwrap();
        let existing_file = temp_dir.path().join("exists.rs");
        let mut f = File::create(&existing_file).unwrap();
        writeln!(f, "fn test() {{}}").unwrap();

        let ranker = ComplexityRanker::default();
        let engine = RankingEngine::new(ranker);

        let files = vec![existing_file, PathBuf::from("/nonexistent/file.rs")];

        let rankings = engine.rank_files(&files, 5).await;
        assert_eq!(rankings.len(), 1); // Only existing file
    }

    // Test custom ranker functionality
    struct TestRanker {
        score_multiplier: f64,
    }

    impl FileRanker for TestRanker {
        type Metric = f64;

        fn compute_score(&self, file_path: &Path) -> Self::Metric {
            file_path.to_string_lossy().len() as f64 * self.score_multiplier
        }

        fn format_ranking_entry(&self, file: &str, metric: &Self::Metric, rank: usize) -> String {
            format!("{rank}. {file} ({metric})")
        }

        fn ranking_type(&self) -> &'static str {
            "Test"
        }
    }

    #[tokio::test]
    async fn test_custom_ranker() {
        let temp_dir = TempDir::new().unwrap();
        let file1 = temp_dir.path().join("a.rs");
        let file2 = temp_dir.path().join("longer_name.rs");

        File::create(&file1).unwrap();
        File::create(&file2).unwrap();

        let ranker = TestRanker {
            score_multiplier: 2.0,
        };
        let engine = RankingEngine::new(ranker);

        let files = vec![file1, file2];
        let rankings = engine.rank_files(&files, 2).await;

        assert_eq!(rankings.len(), 2);
        // Longer filename should have higher score
        assert!(rankings[0].0.contains("longer_name"));
        assert!(rankings[0].1 > rankings[1].1);
    }

    #[test]
    fn test_all_score_types_partial_ord() {
        // Test CompositeComplexityScore
        let comp1 = CompositeComplexityScore {
            total_score: 5.0,
            ..Default::default()
        };
        let comp2 = CompositeComplexityScore {
            total_score: 10.0,
            ..Default::default()
        };
        assert!(comp1 < comp2);

        // Test ChurnScore
        let churn1 = ChurnScore {
            score: 3.0,
            ..Default::default()
        };
        let churn2 = ChurnScore {
            score: 7.0,
            ..Default::default()
        };
        assert!(churn1 < churn2);

        // Test DuplicationScore
        let dup1 = DuplicationScore {
            score: 2.0,
            ..Default::default()
        };
        let dup2 = DuplicationScore {
            score: 8.0,
            ..Default::default()
        };
        assert!(dup1 < dup2);
    }

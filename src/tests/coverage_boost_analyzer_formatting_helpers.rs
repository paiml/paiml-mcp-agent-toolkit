// ---------------------------------------------------------------------------
// Helper constructors
// ---------------------------------------------------------------------------

fn make_analyzer() -> DeepContextAnalyzer {
    DeepContextAnalyzer::new(DeepContextConfig::default())
}

fn make_empty_context() -> DeepContext {
    DeepContext::default()
}

fn make_scorecard(health: f64, maintainability: f64, debt_hours: f64) -> QualityScorecard {
    QualityScorecard {
        overall_health: health,
        complexity_score: 75.0,
        maintainability_index: maintainability,
        modularity_score: 80.0,
        test_coverage: Some(90.0),
        technical_debt_hours: debt_hours,
    }
}

fn make_recommendation(
    title: &str,
    priority: Priority,
    impact: Impact,
    prereqs: Vec<&str>,
) -> PrioritizedRecommendation {
    PrioritizedRecommendation {
        title: title.to_string(),
        description: format!("Description for {title}"),
        priority,
        estimated_effort: Duration::from_secs(3600),
        impact,
        prerequisites: prereqs.into_iter().map(String::from).collect(),
    }
}

fn make_project_overview() -> ProjectOverview {
    ProjectOverview {
        compressed_description: "A test project for analysis.".to_string(),
        key_features: vec!["Feature A".to_string(), "Feature B".to_string()],
        architecture_summary: Some("Microservices architecture".to_string()),
        api_summary: None,
    }
}

fn make_build_info() -> BuildInfo {
    BuildInfo {
        toolchain: "Rust".to_string(),
        targets: vec!["pmat".to_string(), "pmat-cli".to_string()],
        dependencies: vec!["serde".to_string(), "tokio".to_string()],
        primary_command: Some("cargo build --release".to_string()),
    }
}

fn make_annotated_node(name: &str, node_type: NodeType) -> AnnotatedNode {
    AnnotatedNode {
        name: name.to_string(),
        path: PathBuf::from(name),
        node_type,
        children: Vec::new(),
        annotations: NodeAnnotations::default(),
    }
}

fn make_annotated_tree(total_files: usize, total_size: u64) -> AnnotatedFileTree {
    AnnotatedFileTree {
        root: make_annotated_node("root", NodeType::Directory),
        total_files,
        total_size_bytes: total_size,
    }
}

fn make_file_context(path: &str, language: &str, items: Vec<AstItem>) -> FileContext {
    FileContext {
        path: path.to_string(),
        language: language.to_string(),
        items,
        complexity_metrics: None,
    }
}

fn make_enhanced_file_context(path: &str, language: &str) -> EnhancedFileContext {
    EnhancedFileContext {
        base: make_file_context(path, language, Vec::new()),
        complexity_metrics: None,
        churn_metrics: None,
        defects: DefectAnnotations {
            dead_code: None,
            technical_debt: Vec::new(),
            complexity_violations: Vec::new(),
            tdg_score: None,
        },
        symbol_id: format!("sym_{path}"),
    }
}

fn make_complexity_report() -> ComplexityReport {
    ComplexityReport {
        summary: ComplexitySummary {
            total_files: 5,
            total_functions: 20,
            median_cyclomatic: 4.5,
            median_cognitive: 3.0,
            max_cyclomatic: 25,
            max_cognitive: 18,
            p90_cyclomatic: 12,
            p90_cognitive: 10,
            technical_debt_hours: 8.5,
        },
        violations: Vec::new(),
        hotspots: Vec::new(),
        files: vec![FileComplexityMetrics {
            path: "src/main.rs".to_string(),
            total_complexity: ComplexityMetrics::new(15, 12, 3, 100),
            functions: vec![
                FunctionComplexity {
                    name: "complex_function".to_string(),
                    line_start: 10,
                    line_end: 50,
                    metrics: ComplexityMetrics::new(15, 20, 4, 40),
                },
                FunctionComplexity {
                    name: "simple_function".to_string(),
                    line_start: 55,
                    line_end: 60,
                    metrics: ComplexityMetrics::new(2, 1, 1, 5),
                },
            ],
            classes: Vec::new(),
        }],
    }
}

fn make_churn_analysis() -> CodeChurnAnalysis {
    CodeChurnAnalysis {
        generated_at: Utc::now(),
        period_days: 30,
        repository_root: PathBuf::from("/test/project"),
        files: vec![FileChurnMetrics {
            path: PathBuf::from("src/lib.rs"),
            relative_path: "src/lib.rs".to_string(),
            commit_count: 42,
            unique_authors: vec!["alice".to_string(), "bob".to_string()],
            additions: 500,
            deletions: 200,
            churn_score: 0.75,
            last_modified: Utc::now(),
            first_seen: Utc::now(),
        }],
        summary: ChurnSummary {
            total_commits: 100,
            total_files_changed: 15,
            hotspot_files: vec![PathBuf::from("src/lib.rs")],
            stable_files: vec![PathBuf::from("src/config.rs")],
            author_contributions: HashMap::new(),
            mean_churn_score: 0.45,
            variance_churn_score: 0.12,
            stddev_churn_score: 0.35,
        },
    }
}

fn make_satd_result() -> SATDAnalysisResult {
    SATDAnalysisResult {
        items: vec![
            crate::services::satd_detector::TechnicalDebt {
                category: DebtCategory::Defect,
                severity: Severity::Critical,
                text: "  FIXME: critical bug here  ".to_string(),
                file: PathBuf::from("src/main.rs"),
                line: 42,
                column: 5,
                context_hash: [0u8; 16],
            },
            crate::services::satd_detector::TechnicalDebt {
                category: DebtCategory::Requirement,
                severity: Severity::Low,
                text: "TODO: add logging".to_string(),
                file: PathBuf::from("src/util.rs"),
                line: 10,
                column: 1,
                context_hash: [1u8; 16],
            },
            crate::services::satd_detector::TechnicalDebt {
                category: DebtCategory::Design,
                severity: Severity::High,
                text: "HACK: workaround for upstream".to_string(),
                file: PathBuf::from("src/hack.rs"),
                line: 5,
                column: 1,
                context_hash: [2u8; 16],
            },
            crate::services::satd_detector::TechnicalDebt {
                category: DebtCategory::Performance,
                severity: Severity::Medium,
                text: "SLOW: O(n^2) scan".to_string(),
                file: PathBuf::from("src/scan.rs"),
                line: 100,
                column: 1,
                context_hash: [3u8; 16],
            },
        ],
        summary: SATDSummary {
            total_items: 4,
            by_severity: HashMap::new(),
            by_category: HashMap::new(),
            files_with_satd: 4,
            avg_age_days: 15.0,
        },
        total_files_analyzed: 10,
        files_with_debt: 4,
        analysis_timestamp: Utc::now(),
    }
}

fn make_dead_code_result() -> DeadCodeRankingResult {
    DeadCodeRankingResult {
        summary: DeadCodeSummary {
            total_files_analyzed: 10,
            files_with_dead_code: 3,
            total_dead_lines: 150,
            dead_percentage: 5.0,
            dead_functions: 8,
            dead_classes: 1,
            dead_modules: 0,
            unreachable_blocks: 2,
        },
        ranked_files: vec![
            crate::models::dead_code::FileDeadCodeMetrics {
                path: "src/old_module.rs".to_string(),
                dead_lines: 80,
                total_lines: 200,
                dead_percentage: 40.0,
                dead_functions: 5,
                dead_classes: 1,
                dead_modules: 0,
                unreachable_blocks: 1,
                dead_score: 0.85,
                confidence: crate::models::dead_code::ConfidenceLevel::High,
                items: Vec::new(),
            },
            crate::models::dead_code::FileDeadCodeMetrics {
                path: "src/legacy.rs".to_string(),
                dead_lines: 50,
                total_lines: 300,
                dead_percentage: 16.7,
                dead_functions: 3,
                dead_classes: 0,
                dead_modules: 0,
                unreachable_blocks: 1,
                dead_score: 0.55,
                confidence: crate::models::dead_code::ConfidenceLevel::Medium,
                items: Vec::new(),
            },
            // File with zero dead functions (should be filtered out of SARIF)
            crate::models::dead_code::FileDeadCodeMetrics {
                path: "src/clean.rs".to_string(),
                dead_lines: 0,
                total_lines: 100,
                dead_percentage: 0.0,
                dead_functions: 0,
                dead_classes: 0,
                dead_modules: 0,
                unreachable_blocks: 0,
                dead_score: 0.0,
                confidence: crate::models::dead_code::ConfidenceLevel::Low,
                items: Vec::new(),
            },
        ],
        analysis_timestamp: Utc::now(),
        config: DeadCodeAnalysisConfig {
            include_unreachable: true,
            include_tests: false,
            min_dead_lines: 1,
        },
    }
}

fn make_defect_hotspot(file: &str, line: u32, score: f32, hours: f32) -> DefectHotspot {
    DefectHotspot {
        location: FileLocation {
            file: PathBuf::from(file),
            line,
            column: 1,
        },
        composite_score: score,
        contributing_factors: Vec::new(),
        refactoring_effort: RefactoringEstimate {
            estimated_hours: hours,
            priority: Priority::High,
            impact: Impact::High,
            suggested_actions: vec!["Refactor".to_string()],
        },
    }
}

fn make_cross_lang_ref(src: &str, tgt: &str, confidence: f32) -> CrossLangReference {
    CrossLangReference {
        source_file: PathBuf::from(src),
        target_file: PathBuf::from(tgt),
        reference_type: CrossLangReferenceType::FfiCall,
        confidence,
    }
}

fn make_populated_context() -> DeepContext {
    let mut ctx = DeepContext::default();
    ctx.quality_scorecard = make_scorecard(85.0, 72.0, 12.5);
    ctx.project_overview = Some(make_project_overview());
    ctx.build_info = Some(make_build_info());
    ctx.recommendations = vec![
        make_recommendation("Reduce complexity", Priority::High, Impact::High, vec![]),
        make_recommendation(
            "Add tests",
            Priority::Medium,
            Impact::Medium,
            vec!["CI setup"],
        ),
    ];
    ctx.file_tree = make_annotated_tree(42, 512_000);
    ctx.analyses.complexity_report = Some(make_complexity_report());
    ctx.analyses.churn_analysis = Some(make_churn_analysis());
    ctx.analyses.ast_contexts = vec![make_enhanced_file_context("src/lib.rs", "Rust")];
    ctx.defect_summary = DefectSummary {
        total_defects: 5,
        defect_density: 2.5,
        ..Default::default()
    };
    ctx.hotspots = vec![make_defect_hotspot("src/lib.rs", 100, 0.9, 4.0)];
    ctx
}

// ===========================================================================
// Constructor tests
// ===========================================================================

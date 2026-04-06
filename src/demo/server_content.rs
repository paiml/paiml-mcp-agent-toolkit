impl DemoContent {
    #[allow(clippy::too_many_arguments)]
    #[must_use]
    pub fn from_analysis_results(
        dag: &DependencyGraph,
        files_analyzed: usize,
        avg_complexity: f64,
        tech_debt_hours: u32,
        hotspots: Vec<Hotspot>,
        ast_time_ms: u64,
        complexity_time_ms: u64,
        churn_time_ms: u64,
        dag_time_ms: u64,
    ) -> Self {
        let mermaid_generator = MermaidGenerator::new(MermaidOptions {
            show_complexity: true,
            filter_external: true,
            ..Default::default()
        });

        let mermaid_diagram = mermaid_generator.generate(dag);

        let enhanced_hotspots: Vec<EnhancedHotspot> = hotspots
            .into_iter()
            .map(|h| EnhancedHotspot {
                function: "main".to_string(),
                file: h.file.clone(),
                path: h.file,
                complexity: h.complexity,
                loc: 50,
                language: "rust".to_string(),
                churn_score: h.churn_score,
                refactor_suggestion: "Consider extracting methods to reduce complexity".to_string(),
            })
            .collect();

        Self {
            mermaid_diagram,
            system_diagram: None,
            files_analyzed,
            functions_analyzed: enhanced_hotspots.len(),
            avg_complexity,
            p90_complexity: (avg_complexity * 1.5) as u32,
            hotspot_functions: enhanced_hotspots.len(),
            quality_score: 0.75,
            tech_debt_hours,
            hotspots: enhanced_hotspots,
            language_stats: std::collections::HashMap::new(),
            ast_time_ms,
            complexity_time_ms,
            churn_time_ms,
            dag_time_ms,
            recommendations: Vec::new(),
            polyglot_analysis: None,
        }
    }

    pub async fn with_ai_recommendations(
        mut self,
        project_path: &std::path::Path,
        detected_language: &str,
    ) -> Self {
        let recommendation_engine = RecommendationEngine::new();

        if let Ok(_detected_frameworks) =
            recommendation_engine.analyze_repository(&project_path.to_string_lossy())
        {
            self.recommendations = recommendation_engine
                .get_recommendations(detected_language, Some(ComplexityTier::Intermediate));

            if !self.recommendations.is_empty() {
                self.recommendations.truncate(5);
            }
        }

        self
    }

    pub async fn with_polyglot_analysis(mut self, project_path: &std::path::Path) -> Self {
        debug_assert!(project_path.exists(), "project_path must exist: {}", project_path.display());
        let polyglot_analyzer = PolyglotAnalyzer::new();

        if let Ok(analysis) = polyglot_analyzer.analyze_project(project_path).await {
            self.polyglot_analysis = Some(analysis);
        }

        self
    }
}

// For backwards compatibility with synchronous API
#[cfg(feature = "demo")]
pub fn spawn_sync(initial_content: DemoContent) -> Result<LocalDemoServer> {
    // Create a tokio runtime for the synchronous API
    let runtime = tokio::runtime::Runtime::new()?;
    let (server, _port) = runtime.block_on(LocalDemoServer::spawn(initial_content))?;
    Ok(server)
}

impl Default for crate::services::complexity::ComplexityReport {
    fn default() -> Self {
        Self {
            summary: crate::services::complexity::ComplexitySummary {
                total_files: 0,
                total_functions: 0,
                median_cyclomatic: 0.0,
                median_cognitive: 0.0,
                max_cyclomatic: 0,
                max_cognitive: 0,
                p90_cyclomatic: 0,
                p90_cognitive: 0,
                technical_debt_hours: 0.0,
            },
            violations: vec![],
            hotspots: vec![],
            files: vec![],
        }
    }
}

impl Default for crate::models::churn::CodeChurnAnalysis {
    fn default() -> Self {
        Self {
            generated_at: chrono::Utc::now(),
            period_days: 0,
            repository_root: std::path::PathBuf::new(),
            files: vec![],
            summary: crate::models::churn::ChurnSummary {
                total_commits: 0,
                total_files_changed: 0,
                hotspot_files: vec![],
                stable_files: vec![],
                author_contributions: std::collections::HashMap::new(),
                mean_churn_score: 0.0,
                variance_churn_score: 0.0,
                stddev_churn_score: 0.0,
            },
        }
    }
}

